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
        let mut entries: Vec<PortEntry> = Vec::new();

        for line in output.lines().skip(1) {
            if let Some(entry) = self.parse_line(line) {
                // Deduplicate by port — lsof can show multiple entries for the same
                // port (e.g. IPv4 + IPv6, or 127.0.0.1 + 0.0.0.0). Keep one row, but
                // escalate to the most-exposed bind so we never under-report exposure.
                if let Some(existing) = entries.iter_mut().find(|e| e.port == entry.port) {
                    if entry.bind_address.exposure_rank() > existing.bind_address.exposure_rank() {
                        existing.bind_address = entry.bind_address;
                    }
                } else {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn scanner() -> MacOsScanner {
        MacOsScanner {
            current_user: "alice".to_string(),
        }
    }

    #[test]
    fn unescape_lsof_decodes_hex_and_passes_through() {
        assert_eq!(unescape_lsof("plain"), "plain");
        // \x20 is a space — lsof escapes spaces in command names this way.
        assert_eq!(unescape_lsof("Google\\x20Chrome"), "Google Chrome");
        // A lone trailing backslash is preserved, not panicked on.
        assert_eq!(unescape_lsof("weird\\"), "weird\\");
    }

    #[test]
    fn parse_line_ipv4_local() {
        let line = "node 71775 alice 23u IPv4 0x1234 0t0 TCP 127.0.0.1:3001 (LISTEN)";
        let e = scanner().parse_line(line).unwrap();
        assert_eq!(e.port, 3001);
        assert_eq!(e.pid, 71775);
        assert_eq!(e.process_name, "node");
        assert!(e.is_own);
        assert_eq!(e.bind_address, BindAddress::Local);
        assert_eq!(e.protocol, Protocol::Tcp);
    }

    #[test]
    fn parse_line_ipv6_exposed() {
        let line = "caddy 42 root 7u IPv6 0xabc 0t0 TCP [::]:8080 (LISTEN)";
        let e = scanner().parse_line(line).unwrap();
        assert_eq!(e.port, 8080);
        assert!(!e.is_own); // owned by root, not alice
        assert_eq!(e.bind_address, BindAddress::Exposed);
        assert_eq!(e.protocol, Protocol::Tcp6);
    }

    #[test]
    fn parse_line_wildcard_and_specific_binds() {
        let star = scanner()
            .parse_line("srv 1 alice 3u IPv4 0x1 0t0 TCP *:3000 (LISTEN)")
            .unwrap();
        assert_eq!(star.bind_address, BindAddress::Exposed);

        let specific = scanner()
            .parse_line("srv 1 alice 3u IPv4 0x1 0t0 TCP 192.168.1.5:5000 (LISTEN)")
            .unwrap();
        assert_eq!(
            specific.bind_address,
            BindAddress::Specific("192.168.1.5".to_string())
        );
    }

    #[test]
    fn parse_line_rejects_malformed_input() {
        // Too few fields.
        assert!(scanner().parse_line("node 1 alice").is_none());
        // Non-IP socket type (e.g. unix) is skipped.
        assert!(
            scanner()
                .parse_line("node 1 alice 3u unix 0x1 0t0 TCP /tmp/sock (LISTEN)")
                .is_none()
        );
    }

    #[test]
    fn parse_lsof_output_dedupes_and_escalates_exposure() {
        // Same port listed twice (IPv4 local + IPv6 exposed) collapses to one
        // row, keeping the most-exposed bind regardless of line order.
        let output = "\
COMMAND PID USER FD TYPE DEVICE SIZE/OFF NODE NAME
node 1 alice 3u IPv4 0x1 0t0 TCP 127.0.0.1:3000 (LISTEN)
node 1 alice 4u IPv6 0x2 0t0 TCP [::]:3000 (LISTEN)";
        let entries = scanner().parse_lsof_output(output);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].port, 3000);
        assert_eq!(entries[0].bind_address, BindAddress::Exposed);
    }

    #[test]
    fn parse_lsof_output_escalation_is_order_independent() {
        // Exposed line first, local second — must still report Exposed.
        let output = "\
COMMAND PID USER FD TYPE DEVICE SIZE/OFF NODE NAME
node 1 alice 4u IPv6 0x2 0t0 TCP [::]:3000 (LISTEN)
node 1 alice 3u IPv4 0x1 0t0 TCP 127.0.0.1:3000 (LISTEN)";
        let entries = scanner().parse_lsof_output(output);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].bind_address, BindAddress::Exposed);
    }
}
