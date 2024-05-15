use std::sync::Arc;

use eyre::OptionExt;
use tauri::{
    image::Image,
    menu::{MenuBuilder, MenuItem, MenuItemBuilder},
    tray::TrayIconBuilder,
    Manager, Runtime,
};
use tracing::debug;

#[derive(Debug, Clone)]
pub enum SystemAppVisibility {
    Show,
    Hide,
}

#[derive(Debug, Clone)]
pub enum SystemAppIconVariant {
    Active,
    Inactive,
}

pub struct AppTray<R: Runtime> {
    pub tray_icon: tauri::tray::TrayIcon<R>,
    pub menu: tauri::menu::Menu<R>,
    pub app_visibility_item: Arc<AppVisibilityMenuItem<R>>,
}

pub struct AppVisibilityMenuItem<R: Runtime> {
    menu_item: tauri::menu::MenuItem<R>,
    webview_window: tauri::WebviewWindow<R>,
}

impl<R: Runtime> AppVisibilityMenuItem<R> {
    pub fn try_new(app: &impl Manager<R>) -> eyre::Result<Self> {
        let webview_window = app
            .get_webview_window("main")
            .ok_or_eyre("Error finding 'main' application window")?;
        let menu_item =
            MenuItemBuilder::with_id("app_visibility", "Hide application").build(app)?;
        Ok(Self {
            menu_item,
            webview_window,
        })
    }

    pub fn set_app_visibility(&self, visible: bool) -> eyre::Result<()> {
        if visible {
            self.webview_window.hide()?;
            self.menu_item.set_text("Show application")?;
        } else {
            // TODO: this should remember the window position
            self.webview_window.show()?;
            self.webview_window.set_focus()?;
            // set_focus doesn't seem to have any effect on plasma; this trick ensure
            // the window ends up on top:
            {
                self.webview_window.set_always_on_top(true)?;
                std::thread::sleep(std::time::Duration::from_millis(5));
                self.webview_window.set_always_on_top(false)?;
            }
            self.menu_item.set_text("Hide application")?;
        }
        Ok(())
    }

    pub fn toggle_visibility(&self) -> eyre::Result<()> {
        self.set_app_visibility("Hide application" == self.menu_item.text()?)?;
        Ok(())
    }

    pub fn as_menu_item(&self) -> &MenuItem<R> {
        &self.menu_item
    }
}

impl<R: Runtime> AppTray<R> {
    pub fn setup(app: &impl Manager<R>) -> eyre::Result<Self> {
        let app_visibility_item = Arc::new(AppVisibilityMenuItem::try_new(app)?);

        let menu = MenuBuilder::new(app)
            .items(&[
                app_visibility_item.as_menu_item(),
                &MenuItemBuilder::with_id("debug", "Debug").build(app)?,
                &MenuItemBuilder::with_id("exit", "Exit").build(app)?,
            ])
            .build()?;

        let tray_icon = TrayIconBuilder::new()
            .menu(&menu)
            .icon(Image::from_path("./icons/32x32-inactive.png")?)
            .on_tray_icon_event(|_tray, event| {
                debug!("received tray event: {event:?}");
            })
            .on_menu_event({
                let app_visibility_item = app_visibility_item.clone();

                // TODO: remove unwraps, trace errors to console
                move |app, event| match event.id().as_ref() {
                    "exit" => {
                        app.exit(0);
                    }
                    "debug" => {
                        if let Some(window) = app.get_webview_window("main") {
                            window.open_devtools()
                        }
                    }
                    "app_visibility" => {
                        app_visibility_item.toggle_visibility().unwrap();
                    }
                    _ => tracing::warn!("Unhandled event: {event:?}"),
                }
            })
            .build(app)?;

        Ok(Self {
            tray_icon,
            menu,
            app_visibility_item,
        })
    }

    pub fn set_app_icon_variant(&self, variant: SystemAppIconVariant) -> eyre::Result<()> {
        let icon = match variant {
            SystemAppIconVariant::Active => Image::from_path("./icons/32x32-active.png"),
            SystemAppIconVariant::Inactive=> Image::from_path("./icons/32x32-inactive.png"),
        }?;

        self.tray_icon.set_icon(Some(icon))?;
        Ok(())
    }
}
