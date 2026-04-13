use crate::types::{BindAddress, PortEntry, Protocol};
use anyhow::{Context, Result};
use std::process::Command;

pub struct LinuxScanner {
    current_user: String,
}

impl LinuxScanner {
    pub fn new() -> Self {
        let current_user = std::env::var("USER").unwrap_or_default();
        Self { current_user }
    }

    fn parse_ss_output(&self, output: &str) -> Vec<PortEntry> {
        let mut entries = Vec::new();

        for line in output.lines().skip(1) {
            if let Some(entry) = self.parse_line(line) {
                // Deduplicate by port
                if !entries.iter().any(|e: &PortEntry| e.port == entry.port) {
                    entries.push(entry);
                }
            }
        }

        entries.sort_by_key(|e| e.port);
        entries
    }

    fn parse_line(&self, line: &str) -> Option<PortEntry> {
        // ss -tlnp output format:
        // State  Recv-Q  Send-Q  Local Address:Port  Peer Address:Port  Process
        // LISTEN 0       4096    127.0.0.1:3000      0.0.0.0:*          users:(("node",pid=1234,fd=13))
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() < 5 {
            return None;
        }

        // fields[0] = State (LISTEN)
        // fields[1] = Recv-Q
        // fields[2] = Send-Q
        // fields[3] = Local Address:Port
        // fields[4] = Peer Address:Port
        // fields[5..] = Process info (optional)

        let local_addr = fields[3];

        // Parse address and port - handle IPv6 bracket notation [::1]:port and plain addr:port
        let (bind_addr_str, port_str) = if local_addr.starts_with('[') {
            // IPv6: [::1]:3000 or [::]:3000
            let bracket_end = local_addr.find(']')?;
            let addr = &local_addr[..bracket_end + 1]; // includes brackets
            let port = &local_addr[bracket_end + 2..]; // skip ]:
            (addr, port)
        } else {
            // IPv4: 127.0.0.1:3000 or *:3000
            local_addr.rsplit_once(':')?
        };

        let port: u16 = port_str.parse().ok()?;

        let protocol = if bind_addr_str.contains(':') || bind_addr_str.starts_with('[') {
            Protocol::Tcp6
        } else {
            Protocol::Tcp
        };

        let bind_address = match bind_addr_str {
            "127.0.0.1" | "[::1]" => BindAddress::Local,
            "*" | "0.0.0.0" | "[::]" => BindAddress::Exposed,
            addr => BindAddress::Specific(addr.trim_matches(|c| c == '[' || c == ']').to_string()),
        };

        // Parse process info from the last field(s)
        // Format: users:(("node",pid=1234,fd=13))
        let process_info = fields[5..].join(" ");
        let (pid, process_name) = parse_process_info(&process_info).unwrap_or((0, String::new()));

        let user = if pid > 0 {
            get_process_user(pid).unwrap_or_default()
        } else {
            String::new()
        };
        let is_own = !user.is_empty() && user == self.current_user;

        Some(PortEntry {
            port,
            pid,
            process_name,
            command_line: String::new(),
            user,
            is_own,
            bind_address,
            working_dir: None,
            tech: None,
            git_info: None,
            uptime: None,
            docker_info: None,
            cpu_usage: None,
            memory_mb: None,
            protocol,
        })
    }
}

/// Parse the process info field from ss output.
/// Format: users:(("node",pid=1234,fd=13))
/// Can also have multiple entries: users:(("node",pid=1234,fd=13),("node",pid=1234,fd=14))
fn parse_process_info(info: &str) -> Option<(u32, String)> {
    // Look for the first pid=NNNN pattern
    let pid_start = info.find("pid=")?;
    let pid_str_start = pid_start + 4;
    let remaining = &info[pid_str_start..];
    let pid_end = remaining.find(|c: char| !c.is_ascii_digit())?;
    let pid: u32 = remaining[..pid_end].parse().ok()?;

    // Look for process name in quotes before the pid
    // Pattern: (("name",pid=...)
    let before_pid = &info[..pid_start];
    let name = if let Some(start) = before_pid.rfind("((\"").or_else(|| before_pid.rfind("(\"")) {
        let name_start = before_pid[start..].find('"')? + start + 1;
        let name_end = before_pid[name_start..].find('"')? + name_start;
        before_pid[name_start..name_end].to_string()
    } else {
        String::new()
    };

    Some((pid, name))
}

/// Get the username owning a process from /proc/PID/status
fn get_process_user(pid: u32) -> Option<String> {
    let status = std::fs::read_to_string(format!("/proc/{pid}/status")).ok()?;
    for line in status.lines() {
        if let Some(rest) = line.strip_prefix("Uid:") {
            // Format: Uid:\t1000\t1000\t1000\t1000
            // First field is the real UID
            let uid_str = rest.split_whitespace().next()?;
            let uid: u32 = uid_str.parse().ok()?;
            // Resolve UID to username via /etc/passwd
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
    // Fallback: just return the UID as a string
    Some(uid_str)
}

impl super::PortScanner for LinuxScanner {
    fn scan(&self) -> Result<Vec<PortEntry>> {
        let output = Command::new("ss")
            .args(["-tlnp"])
            .output()
            .context("Failed to run ss — are you on Linux?")?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(self.parse_ss_output(&stdout))
    }
}
