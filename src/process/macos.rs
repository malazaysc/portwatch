use crate::types::PortEntry;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

/// Batch-resolve process info for all entries at once to minimize subprocess spawns.
pub fn batch_resolve(entries: &mut [PortEntry]) {
    // Collect unique PIDs
    let pids: Vec<u32> = entries.iter().map(|e| e.pid).collect();
    if pids.is_empty() {
        return;
    }

    // Batch: get command line + elapsed time for all PIDs in one ps call
    let pid_list: String = pids
        .iter()
        .map(|p| p.to_string())
        .collect::<Vec<_>>()
        .join(",");
    let cmd_map = batch_ps_info(&pid_list);

    // Get working directories — unfortunately lsof doesn't support batch cwd lookup well,
    // so we use a single lsof call with all PIDs
    let cwd_map = batch_cwd_lookup(&pid_list);

    for entry in entries.iter_mut() {
        if let Some((args, etime)) = cmd_map.get(&entry.pid) {
            if !args.is_empty() {
                entry.command_line = args.clone();
            }
            entry.uptime = parse_etime(etime);
        }
        if let Some(cwd) = cwd_map.get(&entry.pid) {
            entry.working_dir = Some(cwd.clone());
        }
    }
}

fn batch_ps_info(pid_list: &str) -> HashMap<u32, (String, String)> {
    let mut map = HashMap::new();

    // Get args and elapsed time in one call
    let output = Command::new("ps")
        .args(["-p", pid_list, "-o", "pid=,etime=,args="])
        .output();

    let output = match output {
        Ok(o) => o,
        Err(_) => return map,
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        // Format: "  PID    ELAPSED ARGS..."
        let mut parts = line.splitn(3, |c: char| c.is_whitespace());
        let pid_str = match parts.next() {
            Some(s) => s.trim(),
            None => continue,
        };
        let etime = match parts.next() {
            Some(s) => s.trim().to_string(),
            None => continue,
        };
        let args = parts.next().unwrap_or("").trim().to_string();

        if let Ok(pid) = pid_str.parse::<u32>() {
            map.insert(pid, (args, etime));
        }
    }

    map
}

fn batch_cwd_lookup(pid_list: &str) -> HashMap<u32, PathBuf> {
    let mut map = HashMap::new();

    // -a ANDs the filters (without it, lsof ORs them — returning cwd for ALL processes)
    let output = Command::new("lsof")
        .args(["-a", "-p", pid_list, "-d", "cwd", "-Fpn"])
        .output();

    let output = match output {
        Ok(o) => o,
        Err(_) => return map,
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut current_pid: Option<u32> = None;
    let mut current_fd_is_cwd = false;

    for line in stdout.lines() {
        if let Some(pid_str) = line.strip_prefix('p') {
            current_pid = pid_str.parse().ok();
            current_fd_is_cwd = false;
        } else if let Some(fd) = line.strip_prefix('f') {
            current_fd_is_cwd = fd == "cwd";
        } else if let Some(path) = line.strip_prefix('n')
            && current_fd_is_cwd {
                if let Some(pid) = current_pid {
                    map.insert(pid, PathBuf::from(path));
                }
                current_fd_is_cwd = false;
            }
    }

    map
}

fn parse_etime(etime: &str) -> Option<Duration> {
    // etime formats: "MM:SS", "HH:MM:SS", "D-HH:MM:SS"
    let etime = etime.trim();
    if etime.is_empty() {
        return None;
    }

    let (days, rest) = if let Some((d, r)) = etime.split_once('-') {
        (d.parse::<u64>().ok()?, r)
    } else {
        (0, etime)
    };

    let parts: Vec<&str> = rest.split(':').collect();
    let secs = match parts.len() {
        2 => {
            let mins: u64 = parts[0].parse().ok()?;
            let secs: u64 = parts[1].parse().ok()?;
            mins * 60 + secs
        }
        3 => {
            let hours: u64 = parts[0].parse().ok()?;
            let mins: u64 = parts[1].parse().ok()?;
            let secs: u64 = parts[2].parse().ok()?;
            hours * 3600 + mins * 60 + secs
        }
        _ => return None,
    };

    Some(Duration::from_secs(days * 86400 + secs))
}
