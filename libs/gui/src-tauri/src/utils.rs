use std::env;

/// Finds the binary in PATH
pub fn find_binary_in_path(binary_name: &str) -> Option<String> {
    env::var("PATH")
        .ok()?
        .split(':')
        .map(|path| format!("{}/{}", path, binary_name))
        .find(|path| std::fs::metadata(path).is_ok())
}

/// Enum representing known desktop environments
#[allow(clippy::upper_case_acronyms)]
#[derive(PartialEq, Debug)]
pub enum DesktopEnvironment {
    KDE,
    GNOME,
    XFCE,
    Mate,
    Cinnamon,
    LXDE,
    LXQt,
    Unknown(String), // Fallback for unsupported or custom environments
}

pub fn detect_desktop_environment() -> DesktopEnvironment {
    // Check common environment variables
    if let Ok(session) = env::var("XDG_SESSION_DESKTOP") {
        return match_desktop_environment(&session);
    }
    if let Ok(desktop) = env::var("DESKTOP_SESSION") {
        return match_desktop_environment(&desktop);
    }
    if let Ok(gdm_session) = env::var("GDMSESSION") {
        return match_desktop_environment(&gdm_session);
    }

    // Fallbacks for KDE and GNOME
    if env::var("KDE_FULL_SESSION").is_ok() {
        return DesktopEnvironment::KDE;
    }
    if env::var("GNOME_DESKTOP_SESSION_ID").is_ok() {
        return DesktopEnvironment::GNOME;
    }

    // Unknown environment
    DesktopEnvironment::Unknown("unknown".to_string())
}

fn match_desktop_environment(session: &str) -> DesktopEnvironment {
    match session.to_lowercase().as_str() {
        "kde" => DesktopEnvironment::KDE,
        "gnome" => DesktopEnvironment::GNOME,
        "xfce" => DesktopEnvironment::XFCE,
        "mate" => DesktopEnvironment::Mate,
        "cinnamon" => DesktopEnvironment::Cinnamon,
        "lxde" => DesktopEnvironment::LXDE,
        "lxqt" => DesktopEnvironment::LXQt,
        other => {
            tracing::warn!("unknown desktop environment: {other}");
            DesktopEnvironment::Unknown(other.to_string())
        }
    }
}
