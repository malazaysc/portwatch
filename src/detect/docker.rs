use crate::types::DockerInfo;
use std::collections::HashMap;
use std::process::Command;

/// Query `docker ps` and return a mapping of host port -> container info.
///
/// Gracefully returns an empty map if Docker is not installed, not running,
/// or produces unexpected output.
pub fn detect_docker_ports() -> HashMap<u16, DockerInfo> {
    let mut map = HashMap::new();

    let output = match Command::new("docker")
        .args([
            "ps",
            "--format",
            "{{.ID}}\t{{.Names}}\t{{.Image}}\t{{.Ports}}\t{{.Labels}}",
        ])
        .output()
    {
        Ok(o) if o.status.success() => o,
        _ => return map,
    };

    let stdout = String::from_utf8_lossy(&output.stdout);

    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Fields: ID \t Names \t Image \t Ports \t Labels
        let fields: Vec<&str> = line.splitn(5, '\t').collect();
        if fields.len() < 4 {
            continue;
        }

        let name = fields[1].to_string();
        let image = fields[2].to_string();
        let ports_field = fields[3];
        let labels = fields.get(4).unwrap_or(&"");

        // Extract compose project from labels
        let project = extract_label(labels, "com.docker.compose.project");

        for mapping in ports_field.split(", ") {
            if let Some(host_port) = parse_host_port(mapping) {
                map.insert(
                    host_port,
                    DockerInfo {
                        container_name: name.clone(),
                        image: image.clone(),
                        project: project.clone(),
                    },
                );
            }
        }
    }

    map
}

/// Extract a label value from a comma-separated label string.
/// Labels look like: "com.docker.compose.project=navaris,com.supabase.cli.project=navaris,..."
fn extract_label(labels: &str, key: &str) -> Option<String> {
    for pair in labels.split(',') {
        if let Some((k, v)) = pair.split_once('=') {
            if k.trim() == key {
                return Some(v.trim().to_string());
            }
        }
    }
    None
}

/// Parse a single port mapping like "0.0.0.0:54321->5432/tcp" and return
/// the host port (54321). Returns None for mappings without a host binding
/// (e.g. "5432/tcp").
fn parse_host_port(mapping: &str) -> Option<u16> {
    let host_part = mapping.split("->").next()?;
    if !mapping.contains("->") {
        return None;
    }

    // host_part is like "0.0.0.0:54321" or ":::54321" or "[::]:54321"
    let port_str = host_part.rsplit(':').next()?;
    port_str.parse::<u16>().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_host_port_ipv4() {
        assert_eq!(parse_host_port("0.0.0.0:54321->5432/tcp"), Some(54321));
    }

    #[test]
    fn test_parse_host_port_ipv6() {
        assert_eq!(parse_host_port(":::54321->5432/tcp"), Some(54321));
    }

    #[test]
    fn test_parse_host_port_no_mapping() {
        assert_eq!(parse_host_port("5432/tcp"), None);
    }

    #[test]
    fn test_parse_host_port_bracket_ipv6() {
        assert_eq!(parse_host_port("[::]:8080->80/tcp"), Some(8080));
    }

    #[test]
    fn test_extract_label() {
        let labels = "com.docker.compose.project=navaris,com.supabase.cli.project=navaris";
        assert_eq!(
            extract_label(labels, "com.docker.compose.project"),
            Some("navaris".to_string())
        );
        assert_eq!(extract_label(labels, "nonexistent"), None);
        assert_eq!(extract_label("", "com.docker.compose.project"), None);
    }
}
