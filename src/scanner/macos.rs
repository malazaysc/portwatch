use crate::types::{BindAddress, PortEntry, Protocol};
use anyhow::{Context, Result};
use std::process::Command;

pub struct MacOsScanner {
    current_user: String,
}

impl MacOsScanner {
    pub fn new() -> Self {
        let current_user = std::env::var("USER").unwrap_or_default();
        Self { current_user }
    }

    fn parse_lsof_output(&self, output: &str) -> Vec<PortEntry> {
        let mut entries = Vec::new();

        for line in output.lines().skip(1) {
            if let Some(entry) = self.parse_line(line) {
                // Deduplicate by port — lsof can show multiple entries for the same port
                if !entries.iter().any(|e: &PortEntry| e.port == entry.port) {
                    entries.push(entry);
                }
            }
        }

        entries.sort_by_key(|e| e.port);
        entries
    }

    fn parse_line(&self, line: &str) -> Option<PortEntry> {
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() < 9 {
            return None;
        }

        let process_name = unescape_lsof(fields[0]);
        let pid: u32 = fields[1].parse().ok()?;
        let user = fields[2].to_string();
        let is_own = user == self.current_user;

        let protocol = match fields[4] {
            "IPv4" => Protocol::Tcp,
            "IPv6" => Protocol::Tcp6,
            _ => return None,
        };

        // The NAME field is at index 8 in standard lsof output.
        // Format: "host:port" or "*:port"
        let name_field = fields[8];
        let (bind_addr_str, port_str) = name_field.rsplit_once(':')?;
        let port: u16 = port_str.parse().ok()?;

        let bind_address = match bind_addr_str {
            "127.0.0.1" | "[::1]" | "localhost" => BindAddress::Local,
            "*" | "0.0.0.0" | "[::]" => BindAddress::Exposed,
            addr => BindAddress::Specific(addr.to_string()),
        };

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
            net_rx_bytes: None,
            net_tx_bytes: None,
            net_rx_rate: None,
            net_tx_rate: None,
            protocol,
        })
    }
}

/// Unescape `\xNN` hex sequences that lsof uses for special characters (e.g. spaces).
fn unescape_lsof(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            let mut peek: Vec<char> = Vec::new();
            if let Some(c2) = chars.next() {
                peek.push(c2);
                if c2 == 'x'
                    && let (Some(h1), Some(h2)) = (chars.next(), chars.next())
                {
                    if let Ok(byte) = u8::from_str_radix(&format!("{h1}{h2}"), 16) {
                        result.push(byte as char);
                        continue;
                    }
                    result.push(c);
                    result.push(c2);
                    result.push(h1);
                    result.push(h2);
                    continue;
                }
            }
            result.push(c);
            for p in peek {
                result.push(p);
            }
        } else {
            result.push(c);
        }
    }
    result
}

impl super::PortScanner for MacOsScanner {
    fn scan(&self) -> Result<Vec<PortEntry>> {
        let output = Command::new("lsof")
            .args(["-iTCP", "-sTCP:LISTEN", "-P", "-n", "+c", "0"])
            .output()
            .context("Failed to run lsof — are you on macOS?")?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(self.parse_lsof_output(&stdout))
    }
}
