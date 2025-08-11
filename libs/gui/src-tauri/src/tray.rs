use std::{path::PathBuf, process::Command, sync::Arc};

use eyre::OptionExt;
use tauri::{
    image::Image,
    menu::{MenuBuilder, MenuItem, MenuItemBuilder},
    tray::TrayIconBuilder,
    utils::platform::resource_dir,
    Env, Manager, PackageInfo, Runtime,
};
use tracing::debug;

use crate::utils::{detect_desktop_environment, find_binary_in_path, DesktopEnvironment};

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
    #[allow(dead_code)]
    pub menu: tauri::menu::Menu<R>,
    pub app_visibility_item: Arc<AppVisibilityMenuItem<R>>,
    pub package_info: PackageInfo,
    pub env: Env,
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

            std::thread::spawn({
                let webview_window = self.webview_window.clone(); // Ensure objects are thread-safe or use appropriate synchronization
                let menu_item = self.menu_item.clone(); // Clone if needed to transfer ownership
                move || {
                    // Ensure you handle errors within the thread
                    if let Err(e) = (|| -> Result<(), Box<dyn std::error::Error>> {
                        // Always-on-top toggle
                        webview_window.set_always_on_top(true)?;
                        std::thread::sleep(std::time::Duration::from_millis(50));
                        webview_window.set_always_on_top(false)?;

                        // KDE-specific focus workaround
                        if std::env::consts::OS == "linux" {
                            let de = detect_desktop_environment();
                            if de == DesktopEnvironment::KDE {
                                if let Some(wmctrl_path) = find_binary_in_path("wmctrl") {
                                    let a = Command::new(wmctrl_path)
                                        .args(["-a", "o324"])
                                        .output()
                                        .map_err(|e| tracing::error!("wmctrl fail with: {e:?}"));

                                    tracing::info!("Here: {:?}", a);
                                } else {
                                    tracing::warn!("missing dependency wmctrl");
                                }
                            }
                        }

                        // Update the menu item
                        menu_item.set_text("Hide application")?;
                        Ok(())
                    })() {
                        tracing::error!("Error in focus workaround thread: {:?}", e);
                    }
                }
            });
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

        let env = app.env();
        let package_info = app.package_info().clone();
        let tray_icon_path: PathBuf = resource_dir(&package_info, &env)
            .map_err(|e| eyre::eyre!("Failed to locate resource directory: {e:?}"))?
            .join("icons/32x32-inactive.png");

        let tray_icon = TrayIconBuilder::new()
            .menu(&menu)
            .icon(Image::from_path(&tray_icon_path)?) // Use the dynamically resolved path
            .on_tray_icon_event(|_tray, event| {
                debug!("received tray event: {event:?}");
            })
            .on_menu_event({
                let app_visibility_item = app_visibility_item.clone();

                move |app, event| match event.id().as_ref() {
                    "exit" => {
                        app.exit(0);
                    }
                    "debug" => {
                        // Uncomment if needed
                        // if let Some(window) = app.get_webview_window("main") {
                        //     window.open_devtools()
                        // }
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
            package_info,
            env,
        })
    }

    pub fn set_app_icon_variant(&self, variant: SystemAppIconVariant) -> eyre::Result<()> {
        // Resolve the resource directory
        let resource_dir = resource_dir(&self.package_info, &self.env).map_err(|e| {
            eyre::eyre!("Failed to locate resource directory in set_app_icon_variant: {e:?}")
        })?;

        // Match the variant and resolve the path dynamically
        let icon_path = match variant {
            SystemAppIconVariant::Active => resource_dir.join("icons/32x32-active.png"),
            SystemAppIconVariant::Inactive => resource_dir.join("icons/32x32-inactive.png"),
        };

        // Load the icon
        let icon = Image::from_path(icon_path)
            .map_err(|e| eyre::eyre!("While setting icon variant: {:?}", e))?;
        self.tray_icon.set_icon(Some(icon))?;

        //pub fn set_icon(&self, icon: Option<Image<'_>>) -> crate::Result<()> {
        Ok(())
    }
}
