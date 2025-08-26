use bincode::{Decode, Encode};
use chrono::Utc;
use derive_more::Display;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;
use sysinfo::{Pid, ProcessesToUpdate, System};

mod inter_process_storage;

use inter_process_storage::InterProcessStorage;

/// An internal, cloneable handle that grants write access to the supervisor state.
/// This is not part of the public API and is only created by the `SupervisedTaskManagerOwner`.
#[derive(Clone)]
struct SupervisedTaskWriter {
    manager: Arc<SupervisedTaskManagerInner>,
    failure_policy: FailurePolicy,
}

impl SupervisedTaskWriter {
    /// Removes a service from the status map
    fn cleanup_service(&self, service_name: &str) -> eyre::Result<()> {
        self.manager.db.write(|state| {
            state.services.remove(service_name);
        })
    }

    /// Updates the status of a service in shared memory and logs errors.
    fn update_and_log(&self, status: &ServiceStatus) {
        let status_clone = status.clone();
        if let Err(e) = self.manager.db.write(|state| {
            state
                .services
                .insert(status_clone.service_name.clone(), status_clone);
        }) {
            tracing::error!(
                "Failed to update status for service '{}': {}",
                status.service_name,
                e
            );
        }
    }

    /// Handles the final failure of a service according to the configured policy.
    fn handle_final_failure(&self, service_name: &str, error_msg: &str, attempt: u32) {
        let failure_msg =
            format!("Service '{service_name}' failed after {attempt} attempts: {error_msg}");

        match &self.failure_policy {
            FailurePolicy::Panic => panic!("{}", failure_msg),
            FailurePolicy::Log => tracing::error!("{}", failure_msg),
            FailurePolicy::Callback(callback) => callback(service_name, &failure_msg),
        }
    }
}

/// Handle for a supervised task that provides cleanup on drop
pub struct SupervisedTaskHandle {
    handle: tokio::task::JoinHandle<()>,
    service_name: String,
    writer: SupervisedTaskWriter,
}

#[allow(dead_code)]
impl SupervisedTaskHandle {
    pub fn abort(&self) {
        self.handle.abort();
    }

    pub async fn join(mut self) -> Result<(), tokio::task::JoinError> {
        let handle = std::mem::replace(&mut self.handle, tokio::spawn(async {}));
        std::mem::forget(self);
        handle.await
    }
}

impl Drop for SupervisedTaskHandle {
    fn drop(&mut self) {
        self.handle.abort();
        if let Err(e) = self.writer.cleanup_service(&self.service_name) {
            tracing::warn!(
                "Failed to cleanup service '{}' on handle drop: {}",
                self.service_name,
                e
            );
        }
    }
}

#[derive(Display, Encode, Decode, Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ServiceState {
    Starting,
    Running,
    Retrying,
    Stopped,
    Failed,
}

#[derive(Encode, Decode, Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ServiceStatus {
    pub service_name: String,
    pub state: ServiceState,
    pub retry_count: u32,
    pub last_error: Option<String>,
    pub started_at: Option<i64>,
    pub updated_at: i64,
}

pub type StatusMap = HashMap<String, ServiceStatus>;

#[derive(Encode, Decode, Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SupervisorMetadata {
    pub pid: u32,
    pub started_at: i64,
}

#[derive(Encode, Decode, Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct SupervisorState {
    pub metadata: Option<SupervisorMetadata>,
    pub services: StatusMap,
}

/// A handle representing exclusive "writer" ownership over the supervisor state.
///
/// This handle is not `Clone`. Its Drop implementation ensures that the supervisor's
/// entire session (metadata and services) is cleaned up from shared memory.
pub struct SupervisedTaskManagerOwner {
    inner: Arc<SupervisedTaskManagerInner>,
    failure_policy: FailurePolicy,
}

impl Drop for SupervisedTaskManagerOwner {
    fn drop(&mut self) {
        tracing::info!("Supervisor shutting down. Cleaning up all services and metadata.");
        if let Err(e) = self.inner.db.write(|state| {
            state.metadata = None;
            state.services.clear();
        }) {
            tracing::error!("Failed to clean up supervisor state on shutdown: {}", e);
        }
    }
}

#[derive(Clone)]
#[allow(dead_code)]
pub enum FailurePolicy {
    Panic,
    Log,
    Callback(Arc<dyn Fn(&str, &str) + Send + Sync>),
}

#[allow(dead_code)]
pub enum RetryStrategy {
    Exponential {
        max_attempts: Option<u32>,
        initial_delay: Duration,
        multiplier: f64,
        max_delay: Option<Duration>,
    },
    Flat {
        attempts: Option<u32>,
        delay: Duration,
    },
    NoRetry,
}

impl RetryStrategy {
    fn max_attempts_display(&self) -> String {
        match self {
            RetryStrategy::Exponential {
                max_attempts: Some(n),
                ..
            } => n.to_string(),
            RetryStrategy::Flat {
                attempts: Some(n), ..
            } => n.to_string(),
            RetryStrategy::NoRetry => "1".to_string(),
            _ => "unlimited".to_string(),
        }
    }

    fn delay_for_attempt(&self, attempt: u32) -> Option<Duration> {
        match self {
            RetryStrategy::NoRetry => {
                if attempt >= 1 {
                    None
                } else {
                    Some(Duration::ZERO)
                }
            }
            RetryStrategy::Flat { attempts, delay } => {
                if let Some(max) = attempts {
                    if attempt >= *max {
                        return None;
                    }
                }
                Some(*delay)
            }
            RetryStrategy::Exponential {
                max_attempts,
                initial_delay,
                multiplier,
                max_delay,
            } => {
                if let Some(max) = max_attempts {
                    if attempt >= *max {
                        return None;
                    }
                }
                let delay_secs =
                    initial_delay.as_secs_f64() * multiplier.powf((attempt - 1) as f64);
                let mut calculated_delay = Duration::from_secs_f64(delay_secs);
                if let Some(max) = max_delay {
                    if calculated_delay > *max {
                        calculated_delay = *max;
                    }
                }
                Some(calculated_delay)
            }
        }
    }
}

/// A client for interacting with the supervised task state in shared memory.
/// This client is cloneable, read-only, and can be used by multiple processes.
/// To perform "writer" actions, one must first claim ownership.
#[derive(Clone)]
pub struct SupervisedTaskManager(Arc<SupervisedTaskManagerInner>);

struct SupervisedTaskManagerInner {
    db: InterProcessStorage<SupervisorState>,
}

impl SupervisedTaskManager {
    /// Creates a new client to access the supervisor state. This does not
    /// claim writer ownership.
    pub fn try_new() -> eyre::Result<Self> {
        let db =
            InterProcessStorage::<SupervisorState>::try_new("supervised_task_manager_shmem_v10")?;

        Ok(Self(Arc::new(SupervisedTaskManagerInner { db })))
    }

    /// Attempts to claim exclusive "writer" ownership of the supervisor state.
    ///
    /// Fails if another supervisor process is detected to be active.
    /// If a stale lock from a crashed process is found, it will be cleaned up.
    pub fn try_claim_ownership(
        &self,
        failure_policy: FailurePolicy,
    ) -> eyre::Result<SupervisedTaskManagerOwner> {
        let current_state = self.get_state()?;

        if let Some(meta) = current_state.metadata {
            let mut sys = System::new();
            sys.refresh_processes(ProcessesToUpdate::Some(&[(meta.pid as usize).into()]), true);
            if sys.process(Pid::from(meta.pid as usize)).is_some() {
                return Err(eyre::eyre!(
                    "Another supervisor process (PID {}) is already active.",
                    meta.pid
                ));
            } else {
                tracing::warn!(
                    "Found stale supervisor metadata for crashed PID {}. Cleaning up and taking ownership.",
                    meta.pid
                );
            }
        }

        // We are clear to take ownership. This is the one write operation that the
        // client is allowed to do, as it's the entry point to becoming an owner.
        self.0.db.write(|state| {
            state.metadata = Some(SupervisorMetadata {
                pid: std::process::id(),
                started_at: Utc::now().timestamp(),
            });
            state.services.clear(); // Clear any stale services from a previous run
        })?;

        tracing::info!(
            "Successfully claimed supervisor ownership (PID {}).",
            std::process::id()
        );

        Ok(SupervisedTaskManagerOwner {
            inner: self.0.clone(),
            failure_policy,
        })
    }

    /// Reads the entire supervisor state from shared memory.
    pub fn get_state(&self) -> eyre::Result<SupervisorState> {
        self.0.db.read()
    }
}

#[allow(dead_code)]
impl SupervisedTaskManagerOwner {
    /// Reads the entire supervisor state from shared memory.
    pub fn get_state(&self) -> eyre::Result<SupervisorState> {
        self.inner.db.read()
    }

    /// Spawns a new task under supervision.
    ///
    /// The provided operation will be executed. If it returns an error, it will be
    /// restarted according to the `RetryStrategy`. The task's status is continuously
    /// updated in shared memory.
    pub fn spawn_supervised_task<F, Fut, T, E>(
        &self,
        service_name: &'static str,
        retry_strategy: RetryStrategy,
        mut op: F,
    ) -> SupervisedTaskHandle
    where
        F: FnMut() -> Fut + Send + 'static,
        Fut: Future<Output = Result<T, E>> + Send,
        T: Send + 'static,
        E: std::fmt::Display + Send + 'static,
    {
        let writer = SupervisedTaskWriter {
            manager: self.inner.clone(),
            failure_policy: self.failure_policy.clone(),
        };

        let handle = tokio::spawn(async move {
            let mut attempt = 0u32;

            let mut status = ServiceStatus {
                service_name: service_name.to_string(),
                state: ServiceState::Starting,
                retry_count: 0,
                last_error: None,
                started_at: None,
                updated_at: Utc::now().timestamp(),
            };
            writer.update_and_log(&status);

            loop {
                attempt += 1;
                status.retry_count = attempt - 1;

                if attempt == 1 {
                    status.state = ServiceState::Running;
                    status.started_at = Some(Utc::now().timestamp());
                }

                status.updated_at = Utc::now().timestamp();
                writer.update_and_log(&status);

                match op().await {
                    Ok(_) => {
                        status.state = ServiceState::Stopped;
                        status.last_error = None;
                        status.updated_at = Utc::now().timestamp();
                        writer.update_and_log(&status);
                        return;
                    }
                    Err(e) => {
                        let error_msg = e.to_string();
                        status.last_error = Some(error_msg.clone());

                        if let Some(delay) = retry_strategy.delay_for_attempt(attempt) {
                            status.state = ServiceState::Retrying;
                            status.updated_at = Utc::now().timestamp();
                            writer.update_and_log(&status);
                            tracing::info!(
                                "Service '{}' failed. Retrying in {:?} (attempt {}/{})...",
                                service_name,
                                delay,
                                attempt,
                                retry_strategy.max_attempts_display()
                            );
                            tokio::time::sleep(delay).await;
                        } else {
                            status.state = ServiceState::Failed;
                            status.updated_at = Utc::now().timestamp();
                            writer.update_and_log(&status);
                            writer.handle_final_failure(service_name, &error_msg, attempt);
                            return;
                        }
                    }
                }
            }
        });

        SupervisedTaskHandle {
            handle,
            service_name: service_name.to_string(),
            writer: SupervisedTaskWriter {
                manager: self.inner.clone(),
                failure_policy: self.failure_policy.clone(),
            },
        }
    }
}
