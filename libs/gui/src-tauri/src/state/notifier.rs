use crate::{
    event_emitter::EventEmitter,
    tray::{SystemAppIconVariant, SystemAppVisibility},
};

#[derive(Default)]
pub struct AppNotifier {
    /// emitter that sets the app visibility (shown or hidden)
    pub app_visibility_emitter: EventEmitter<SystemAppVisibility>,

    /// emitter that set the variant of the app icon (active or inactive)
    pub app_icon_emitter: EventEmitter<SystemAppIconVariant>,
}

impl AppNotifier {
    pub fn new() -> Self {
        Self::default()
    }
}
