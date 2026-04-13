use crate::types::PortEntry;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

/// Batch-resolve process info for all entries using /proc filesystem.
pub fn batch_resolve(entries: &mut [PortEntry]) {
    if entries.is_empty() {
        return;
    }

    // Collect unique PIDs
    let pids: Vec<u32> = entries.iter().map(|e| e.pid).filter(|&p| p > 0).collect();

    // Resolve all process info from /proc
    let mut cmd_map: HashMap<u32, String> = HashMap::new();
    let mut cwd_map: HashMap<u32, PathBuf> = HashMap::new();
    let mut uptime_map: HashMap<u32, Duration> = HashMap::new();
    let mut user_map: HashMap<u32, String> = HashMap::new();

    let boot_time_ticks = get_boot_time_ticks();
    let ticks_per_sec = get_clock_ticks_per_sec();

    for &pid in &pids {
        // Command line from /proc/PID/cmdline (null-separated)
        if let Ok(raw) = std::fs::read(format!("/proc/{pid}/cmdline")) {
            let cmdline = raw
                .split(|&b| b == 0)
                .filter(|s| !s.is_empty())
                .map(|s| String::from_utf8_lossy(s).into_owned())
                .collect::<Vec<_>>()
                .join(" ");
            if !cmdline.is_empty() {
                cmd_map.insert(pid, cmdline);
            }
        }

        // Working directory from /proc/PID/cwd symlink
        if let Ok(cwd) = std::fs::read_link(format!("/proc/{pid}/cwd")) {
            cwd_map.insert(pid, cwd);
        }

        // Uptime from /proc/PID/stat field 22 (starttime in clock ticks since boot)
        if let Some(duration) = calculate_uptime(pid, boot_time_ticks, ticks_per_sec) {
            uptime_map.insert(pid, duration);
        }

        // User from /proc/PID/status Uid field
        if let Some(user) = get_process_user(pid) {
            user_map.insert(pid, user);
        }
    }

    let current_user = std::env::var("USER").unwrap_or_default();

    for entry in entries.iter_mut() {
        if let Some(cmdline) = cmd_map.get(&entry.pid) {
            entry.command_line = cmdline.clone();
        }
        if let Some(cwd) = cwd_map.get(&entry.pid) {
            entry.working_dir = Some(cwd.clone());
        }
        if let Some(uptime) = uptime_map.get(&entry.pid) {
            entry.uptime = Some(*uptime);
        }
        if let Some(user) = user_map.get(&entry.pid) {
            if entry.user.is_empty() {
                entry.user = user.clone();
            }
            entry.is_own = *user == current_user;
        }
    }
}

/// Get system boot time in clock ticks from /proc/stat
fn get_boot_time_ticks() -> Option<u64> {
    let stat = std::fs::read_to_string("/proc/stat").ok()?;
    for line in stat.lines() {
        if let Some(rest) = line.strip_prefix("btime ") {
            return rest.trim().parse().ok();
        }
    }
    None
}

/// Get clock ticks per second (typically 100 on Linux)
fn get_clock_ticks_per_sec() -> u64 {
    // sysconf(_SC_CLK_TCK) is typically 100 on Linux
    // We could use libc but 100 is the standard default
    100
}

/// Calculate process uptime from /proc/PID/stat starttime field
fn calculate_uptime(pid: u32, boot_time_secs: Option<u64>, ticks_per_sec: u64) -> Option<Duration> {
    let boot_time_secs = boot_time_secs?;
    let stat = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;

    // /proc/PID/stat format: pid (comm) state ... field22_starttime ...
    // The comm field can contain spaces and parentheses, so find the last ')' first
    let after_comm = stat.find(')')? + 2; // skip ") "
    let fields: Vec<&str> = stat[after_comm..].split_whitespace().collect();

    // Field 22 (1-indexed) is starttime, but since we skipped pid and (comm),
    // starttime is at index 19 (field 22 - 3 fields already consumed = field index 19)
    // Actually: fields after comm start at field 3, so starttime (field 22) is at index 22-3 = 19
    let starttime_ticks: u64 = fields.get(19)?.parse().ok()?;

    let start_time_secs = boot_time_secs + starttime_ticks / ticks_per_sec;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?;
    let now_secs = now.as_secs();

    if now_secs > start_time_secs {
        Some(Duration::from_secs(now_secs - start_time_secs))
    } else {
        Some(Duration::from_secs(0))
    }
}

/// Get the username owning a process from /proc/PID/status
fn get_process_user(pid: u32) -> Option<String> {
    let status = std::fs::read_to_string(format!("/proc/{pid}/status")).ok()?;
    for line in status.lines() {
        if let Some(rest) = line.strip_prefix("Uid:") {
            let uid_str = rest.split_whitespace().next()?;
            let uid: u32 = uid_str.parse().ok()?;
            return uid_to_username(uid);
        }
    }
    None
}

/// Resolve a UID to a username by reading /etc/passwd
fn uid_to_username(uid: u32) -> Option<String> {
    let passwd = std::fs::read_to_string("/etc/passwd").ok()?;
    let uid_str = uid.to_string();
    for line in passwd.lines() {
        let fields: Vec<&str> = line.split(':').collect();
        if fields.len() >= 3 && fields[2] == uid_str {
            return Some(fields[0].to_string());
        }
    }
    // Fallback: return the UID as a string
    Some(uid_str)
}
