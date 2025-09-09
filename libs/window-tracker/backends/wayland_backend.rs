use crate::{WindowInfo, WindowTrackerError};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use wayland_client::{Display, EventQueue, GlobalManager, Main};

// The `nest` module is generated code and doesn't need changes,
// but ensure your Cargo.toml is correct as described above.
pub mod nest {
    #![allow(dead_code, non_camel_case_types, unused_unsafe, unused_variables)]
    #![allow(
        non_upper_case_globals,
        non_snake_case,
        unused_imports,
        static_mut_refs
    )]

    use smallvec::smallvec;
    use wayland_client::{
        protocol::{wl_output, wl_seat, wl_surface},
        AnonymousObject, Main, Proxy, ProxyMap,
    };
    use wayland_commons::wire::{Argument, ArgumentType, Message, MessageDesc};
    use wayland_commons::{
        map::{Object, ObjectMetadata},
        Interface, MessageGroup,
    };
    use wayland_sys as sys;

    // This include macro should now work correctly with the fixed smallvec version
    include!(concat!(
        env!("OUT_DIR"),
        "/wayland_protocols/wlr_foreign_toplevel_management.rs"
    ));
}

use nest::zwlr_foreign_toplevel_handle_v1::Event as TEvent;
use nest::zwlr_foreign_toplevel_handle_v1::ZwlrForeignToplevelHandleV1;
use nest::zwlr_foreign_toplevel_manager_v1::ZwlrForeignToplevelManagerV1;

#[derive(Debug, Default)]
struct WaylandState {
    windows: HashMap<u32, WindowInfo>,
    active_window_id: Option<u32>,
}

// Global state to be manipulated by Wayland's event-driven callbacks.
static WAYLAND_STATE: Lazy<Arc<Mutex<WaylandState>>> =
    Lazy::new(|| Arc::new(Mutex::new(WaylandState::default())));

pub struct WaylandBackend {
    _display: Display,
    event_queue: EventQueue,
}

impl WaylandBackend {
    pub fn new() -> Result<Self, WindowTrackerError> {
        let display = Display::connect_to_env()
            .map_err(|e| WindowTrackerError::WaylandConnection(e.to_string()))?;
        let mut event_queue = display.create_event_queue();

        let attached_display = (*display).clone().attach(event_queue.token());

        let globals = GlobalManager::new(&attached_display);

        event_queue
            // FIX: The closure must take 3 arguments.
            .sync_roundtrip(&mut (), |_, _, _| {})
            .map_err(|e| WindowTrackerError::WaylandConnection(e.to_string()))?;

        let toplevel_manager = globals
            .instantiate_exact::<ZwlrForeignToplevelManagerV1>(1)
            .map_err(|_| WindowTrackerError::WaylandProtocolMissing("wlr-foreign-toplevel-management not found. Is your compositor supported (e.g., Sway, Hyprland)?".to_string()))?;

        toplevel_manager.quick_assign(move |_manager, event, _| {
            if let nest::zwlr_foreign_toplevel_manager_v1::Event::Toplevel { toplevel } = event {
                handle_new_toplevel(toplevel);
            }
        });

        event_queue
            // FIX: The closure must take 3 arguments.
            .sync_roundtrip(&mut (), |_, _, _| {})
            .map_err(|e| WindowTrackerError::WaylandConnection(e.to_string()))?;

        Ok(Self {
            _display: display,
            event_queue,
        })
    }

    fn dispatch_events(&mut self) -> Result<(), WindowTrackerError> {
        self.event_queue
            .dispatch_pending(&mut (), |_, _, _| {})
            .map_err(|e| WindowTrackerError::WaylandConnection(e.to_string()))?;
        Ok(())
    }

    pub async fn get_active_window_backend(
        &mut self,
    ) -> Result<Option<WindowInfo>, WindowTrackerError> {
        self.dispatch_events()?;
        let state = WAYLAND_STATE.lock().unwrap();

        if let Some(id) = state.active_window_id {
            Ok(state.windows.get(&id).cloned())
        } else {
            Ok(None)
        }
    }

    pub async fn get_all_windows_backend(&mut self) -> Result<Vec<WindowInfo>, WindowTrackerError> {
        self.dispatch_events()?;
        let state = WAYLAND_STATE.lock().unwrap();
        Ok(state.windows.values().cloned().collect())
    }
}

fn handle_new_toplevel(toplevel: Main<ZwlrForeignToplevelHandleV1>) {
    let state_clone = WAYLAND_STATE.clone();

    let id = toplevel.as_ref().id();
    {
        let mut state = state_clone.lock().unwrap();
        state.windows.insert(
            id,
            WindowInfo {
                id: id.to_string(),
                title: String::new(),
                app_name: String::new(),
                pid: None,
                is_focused: false,
                workspace: None,
                geometry: None,
                details: None,
            },
        );
    }

    toplevel.quick_assign(move |_handle, event, _| {
        let mut state = state_clone.lock().unwrap();

        // Check if the window still exists before processing events for it.
        if !state.windows.contains_key(&id) {
            return;
        }

        match event {
            TEvent::Title { title } => {
                if let Some(window) = state.windows.get_mut(&id) {
                    window.title = title;
                }
            }
            TEvent::AppId { app_id } => {
                if let Some(window) = state.windows.get_mut(&id) {
                    window.app_name = app_id;
                }
            }
            TEvent::State { state: state_array } => {
                let is_activated = state_array.chunks_exact(4).any(|chunk| {
                    u32::from_ne_bytes(chunk.try_into().unwrap()) == 2 // 2 is Activated state
                });

                if is_activated {
                    // Get the PREVIOUS active ID before making any changes.
                    // .take() moves the value out, leaving None, which releases the borrow.
                    let prev_active_id = state.active_window_id.take();

                    // If there was a previous active window, and it's not the current one, deactivate it.
                    if let Some(prev_id) = prev_active_id {
                        if prev_id != id {
                            if let Some(prev_win) = state.windows.get_mut(&prev_id) {
                                prev_win.is_focused = false;
                            }
                        }
                    }

                    // Now, activate the current window.
                    if let Some(current_win) = state.windows.get_mut(&id) {
                        current_win.is_focused = true;
                    }

                    // Finally, set the new active window ID.
                    state.active_window_id = Some(id);
                } else {
                    // It's being deactivated
                    // Check if THIS window was the active one.
                    if state.active_window_id == Some(id) {
                        // Deactivate it in the map.
                        if let Some(window) = state.windows.get_mut(&id) {
                            window.is_focused = false;
                        }
                        // And clear the global active ID.
                        state.active_window_id = None;
                    }
                }
            }
            TEvent::Closed => {
                state.windows.remove(&id);
                if state.active_window_id == Some(id) {
                    state.active_window_id = None;
                }
            }
            _ => {}
        }
    });
}

// Add a specific error type to WindowTrackerError
impl From<wayland_client::ConnectError> for WindowTrackerError {
    fn from(e: wayland_client::ConnectError) -> Self {
        WindowTrackerError::WaylandConnection(e.to_string())
    }
}
