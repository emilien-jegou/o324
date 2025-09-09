use once_cell::sync::Lazy;
use std::sync::Arc;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{self, ConnectionExt, Window};
use x11rb::rust_connection::RustConnection;

use crate::{WindowGeometry, WindowInfo, WindowTrackerError};

#[derive(Debug, Clone, thiserror::Error)]
#[error("X11 Init Error: {0}")]
pub struct X11InitError(String);

#[derive(Clone, Copy)]
struct X11Atoms {
    net_active_window: xproto::Atom,
    net_client_list: xproto::Atom,
    net_wm_name: xproto::Atom,
    net_wm_pid: xproto::Atom,
    utf8_string: xproto::Atom,
    wm_class: xproto::Atom,
}

impl X11Atoms {
    fn intern_all(conn: &impl Connection) -> Result<Self, WindowTrackerError> {
        let atoms_to_intern = [
            "_NET_ACTIVE_WINDOW",
            "_NET_CLIENT_LIST",
            "_NET_WM_NAME",
            "_NET_WM_PID",
            "UTF8_STRING",
            "WM_CLASS",
        ];
        let cookies: Vec<_> = atoms_to_intern
            .iter()
            .map(|name| conn.intern_atom(false, name.as_bytes()))
            .collect();
        let mut atoms = Vec::new();
        for cookie in cookies {
            atoms.push(cookie?.reply()?.atom);
        }
        Ok(Self {
            net_active_window: atoms[0],
            net_client_list: atoms[1],
            net_wm_name: atoms[2],
            net_wm_pid: atoms[3],
            utf8_string: atoms[4],
            wm_class: atoms[5],
        })
    }
}

static X11_CONNECTION: Lazy<Result<(Arc<RustConnection>, usize), X11InitError>> = Lazy::new(|| {
    x11rb::connect(None)
        .map(|(conn, screen_num)| (Arc::new(conn), screen_num))
        .map_err(|e| X11InitError(e.to_string()))
});

#[derive(Clone)]
pub struct X11Backend {
    conn: Arc<RustConnection>,
    atoms: X11Atoms,
    root: Window,
}

impl X11Backend {
    pub fn try_new() -> Result<Self, WindowTrackerError> {
        let (conn, screen_num) = X11_CONNECTION.as_ref().map_err(Clone::clone)?.clone();
        let atoms = X11Atoms::intern_all(&*conn)?;
        let root = conn.setup().roots[screen_num].root;
        Ok(Self { conn, atoms, root })
    }

    pub fn get_window_info(
        &self,
        window: Window,
        is_focused: bool,
    ) -> Result<Option<WindowInfo>, WindowTrackerError> {
        if window == x11rb::NONE {
            return Ok(None);
        }

        let title_cookie = self.conn.get_property(
            false,
            window,
            self.atoms.net_wm_name,
            self.atoms.utf8_string,
            0,
            u32::MAX,
        )?;
        let class_cookie = self.conn.get_property(
            false,
            window,
            self.atoms.wm_class,
            xproto::AtomEnum::STRING,
            0,
            u32::MAX,
        )?;
        let pid_cookie = self.conn.get_property(
            false,
            window,
            self.atoms.net_wm_pid,
            xproto::AtomEnum::CARDINAL,
            0,
            1,
        )?;
        let geom_cookie = self.conn.get_geometry(window)?;

        let title = String::from_utf8(title_cookie.reply()?.value).unwrap_or_default();
        let class_bytes = class_cookie.reply()?.value;
        let app_name =
            String::from_utf8_lossy(class_bytes.split(|&b| b == 0).nth(1).unwrap_or(b""))
                .to_string();

        let pid = pid_cookie.reply()?.value32().and_then(|mut i| i.next());
        let geom = geom_cookie.reply()?;
        let details = pid.and_then(crate::utils::get_process_info);

        Ok(Some(WindowInfo {
            id: window.to_string(),
            title,
            app_name,
            pid,
            is_focused,
            workspace: None,
            geometry: Some(WindowGeometry {
                x: geom.x as i32,
                y: geom.y as i32,
                width: geom.width as u32,
                height: geom.height as u32,
            }),
            details,
        }))
    }

    pub async fn get_active_window_backend(
        &self,
    ) -> Result<Option<WindowInfo>, WindowTrackerError> {
        let prop = self
            .conn
            .get_property(
                false,
                self.root,
                self.atoms.net_active_window,
                xproto::AtomEnum::WINDOW,
                0,
                1,
            )?
            .reply()?;
        let active_window = prop
            .value32()
            .and_then(|mut v| v.next())
            .unwrap_or(x11rb::NONE);
        self.get_window_info(active_window, true)
    }

    pub async fn get_all_windows_backend(&self) -> Result<Vec<WindowInfo>, WindowTrackerError> {
        let active_window = self.get_active_window_backend().await?.map(|w| w.id);
        let prop = self
            .conn
            .get_property(
                false,
                self.root,
                self.atoms.net_client_list,
                xproto::AtomEnum::WINDOW,
                0,
                u32::MAX,
            )?
            .reply()?;
        let mut windows = Vec::new();
        if let Some(client_list) = prop.value32() {
            for win_id in client_list {
                let is_focused = active_window.as_deref() == Some(&win_id.to_string());
                if let Ok(Some(info)) = self.get_window_info(win_id, is_focused) {
                    windows.push(info);
                }
            }
        }
        Ok(windows)
    }
}
