use crossbeam_channel::{Receiver, SendError, Sender};
use dashmap::DashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

#[derive(Clone)]
pub struct EventEmitter<T: Clone> {
    listener_ids_counter: Arc<AtomicU64>,
    listeners: DashMap<u64, EventListener<T>>,
}

impl<T: Clone> EventEmitter<T> {
    pub fn new() -> Self {
        Self {
            listener_ids_counter: Arc::new(AtomicU64::default()),
            listeners: DashMap::new(),
        }
    }

    pub fn notify(&self, data: &T) {
        for listener in self.listeners.iter() {
            if let Err(error) = listener.send(data) {
                tracing::error!("Error while notifiying listener: {error}");
            }
        }
    }

    fn generate_listener_id(&self) -> u64 {
        self.listener_ids_counter.fetch_add(1, Ordering::SeqCst)
    }

    pub fn subscribe(&self) -> EventListener<T> {
        let listener_id = self.generate_listener_id();
        let listener = EventListener::<T>::new(listener_id);
        self.listeners.insert(listener_id, listener.clone());
        listener
    }

    pub fn unsubscribe(&self, listener_id: u64) {
        self.listeners.remove(&listener_id);
    }
}

#[derive(Clone)]
pub struct EventListener<T: Clone> {
    pub id: u64,
    channel: (Sender<T>, Receiver<T>),
}

impl<T: Clone> EventListener<T> {
    fn new(listener_id: u64) -> Self {
        Self {
            id: listener_id,
            channel: crossbeam_channel::unbounded(),
        }
    }

    fn send(&self, data: &T) -> Result<(), SendError<T>> {
        self.channel.0.send(data.clone())
    }

    pub fn try_listen(&self) -> eyre::Result<T> {
        Ok(self.channel.1.recv()?)
    }
}

impl<T: Clone + Send + 'static> EventListener<T> {
    pub fn start_listen(
        self,
        on: impl Fn(T) -> eyre::Result<()> + Send + 'static,
    ) -> eyre::Result<()> {
        tokio::task::spawn(async move {
            loop {
                let data = self.channel.1.recv().unwrap();
                if let Err(e) = on(data) {
                    tracing::error!("Error in listener: {e:?}");
                };
            }
        });
        Ok(())
    }
}
impl<T: Clone> Default for EventEmitter<T> {
    fn default() -> Self {
        Self::new()
    }
}
