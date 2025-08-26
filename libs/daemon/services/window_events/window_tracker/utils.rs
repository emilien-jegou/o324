use std::fs;

use libc::uid_t;
use nix::unistd::{Pid, Uid, User};

use crate::services::window_events::window_tracker::ProcessDetails;

/// Gathers detailed information about a process using its PID.
/// This function relies on the Linux-specific /proc filesystem.
pub fn get_process_info(pid_u32: u32) -> Option<ProcessDetails> {
    if pid_u32 == 0 {
        return None;
    }
    let pid = Pid::from_raw(pid_u32 as i32);
    let proc_path = format!("/proc/{pid}");

    // Get executable path
    let exe = fs::read_link(format!("{proc_path}/exe"))
        .ok()
        .and_then(|p| p.to_str().map(String::from));

    // Get command line arguments
    let cmd = fs::read_to_string(format!("{proc_path}/cmdline"))
        .ok()
        .map(|s| {
            s.split('\0')
                .map(String::from)
                .filter(|s| !s.is_empty())
                .collect()
        });

    // Get current working directory
    let cwd = fs::read_link(format!("{proc_path}/cwd"))
        .ok()
        .and_then(|p| p.to_str().map(String::from));

    // Get user
    let user = fs::read_to_string(format!("{proc_path}/status"))
        .ok()
        .and_then(|status| {
            status
                .lines()
                .find(|line| line.starts_with("Uid:"))
                .and_then(|line| line.split_whitespace().nth(1))
                .and_then(|uid_str| uid_str.parse::<uid_t>().ok())
                .and_then(|uid| User::from_uid(Uid::from_raw(uid)).ok())
                .flatten()
                .map(|u| u.name)
        });

    Some(ProcessDetails {
        user,
        exe,
        cmd,
        cwd,
    })
}
