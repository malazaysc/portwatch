use std::fmt;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct DockerInfo {
    pub container_name: String,
    pub image: String,
    pub project: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PortEntry {
    pub port: u16,
    pub pid: u32,
    pub process_name: String,
    pub command_line: String,
    pub user: String,
    pub is_own: bool,
    pub bind_address: BindAddress,
    pub working_dir: Option<PathBuf>,
    pub tech: Option<TechInfo>,
    pub git_info: Option<GitInfo>,
    pub uptime: Option<std::time::Duration>,
    pub docker_info: Option<DockerInfo>,
    pub cpu_usage: Option<f32>,
    pub memory_mb: Option<f64>,
    pub net_rx_bytes: Option<u64>,
    pub net_tx_bytes: Option<u64>,
    pub net_rx_rate: Option<u64>,
    pub net_tx_rate: Option<u64>,
    #[allow(dead_code)]
    pub protocol: Protocol,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BindAddress {
    Local,            // 127.0.0.1, ::1
    Exposed,          // 0.0.0.0, ::
    Specific(String), // bound to a specific interface IP
}

impl BindAddress {
    /// Rank a bind by how widely reachable it is. Higher = more exposed.
    /// Used to merge duplicate sockets on the same port so we never
    /// under-report exposure (a process may listen on both 127.0.0.1 and
    /// 0.0.0.0 — we must surface the exposed one).
    pub fn exposure_rank(&self) -> u8 {
        match self {
            BindAddress::Local => 0,
            BindAddress::Specific(_) => 1,
            BindAddress::Exposed => 2,
        }
    }
}

impl fmt::Display for BindAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BindAddress::Local => write!(f, "127.0.0.1 (local)"),
            BindAddress::Exposed => write!(f, "0.0.0.0 (exposed)"),
            BindAddress::Specific(ip) => write!(f, "{ip}"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TechInfo {
    pub name: String,
    pub source: DetectionSource,
}

#[derive(Debug, Clone)]
pub enum DetectionSource {
    CommandLine,
    ProjectFile,
    PortHeuristic,
}

impl fmt::Display for DetectionSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DetectionSource::CommandLine => write!(f, "command line"),
            DetectionSource::ProjectFile => write!(f, "project file"),
            DetectionSource::PortHeuristic => write!(f, "port heuristic"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GitInfo {
    pub branch: String,
    #[allow(dead_code)]
    pub repo_root: PathBuf,
    pub is_worktree: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Protocol {
    Tcp,
    Tcp6,
}

impl fmt::Display for Protocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Protocol::Tcp => write!(f, "TCP"),
            Protocol::Tcp6 => write!(f, "TCP6"),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct NetworkStats {
    pub rx_bytes_per_sec: u64,
    pub tx_bytes_per_sec: u64,
}

pub fn format_uptime(duration: &std::time::Duration) -> String {
    let secs = duration.as_secs();
    if secs < 60 {
        "<1m".to_string()
    } else if secs < 3600 {
        format!("{}m", secs / 60)
    } else if secs < 86400 {
        let hours = secs / 3600;
        let mins = (secs % 3600) / 60;
        if mins > 0 {
            format!("{hours}h {mins}m")
        } else {
            format!("{hours}h")
        }
    } else {
        let days = secs / 86400;
        let hours = (secs % 86400) / 3600;
        if hours > 0 {
            format!("{days}d {hours}h")
        } else {
            format!("{days}d")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn format_uptime_covers_all_ranges() {
        assert_eq!(format_uptime(&Duration::from_secs(30)), "<1m");
        assert_eq!(format_uptime(&Duration::from_secs(90)), "1m");
        assert_eq!(format_uptime(&Duration::from_secs(3600)), "1h");
        assert_eq!(format_uptime(&Duration::from_secs(3660)), "1h 1m");
        assert_eq!(format_uptime(&Duration::from_secs(86_400)), "1d");
        assert_eq!(format_uptime(&Duration::from_secs(90_000)), "1d 1h");
        // Whole days with no leftover hours omit the hour component.
        assert_eq!(format_uptime(&Duration::from_secs(172_800)), "2d");
    }

    #[test]
    fn exposure_rank_orders_local_below_exposed() {
        assert!(
            BindAddress::Local.exposure_rank() < BindAddress::Specific("x".into()).exposure_rank()
        );
        assert!(
            BindAddress::Specific("x".into()).exposure_rank()
                < BindAddress::Exposed.exposure_rank()
        );
    }

    #[test]
    fn bind_address_display() {
        assert_eq!(BindAddress::Local.to_string(), "127.0.0.1 (local)");
        assert_eq!(BindAddress::Exposed.to_string(), "0.0.0.0 (exposed)");
        assert_eq!(
            BindAddress::Specific("10.0.0.5".into()).to_string(),
            "10.0.0.5"
        );
    }

    #[test]
    fn protocol_display() {
        assert_eq!(Protocol::Tcp.to_string(), "TCP");
        assert_eq!(Protocol::Tcp6.to_string(), "TCP6");
    }
}
