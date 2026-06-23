use crate::types::{DetectionSource, TechInfo};

pub fn detect(port: u16) -> Option<TechInfo> {
    let name = match port {
        80 | 443 => "HTTP Server",
        3000 => "Node.js (likely)",
        3001 => "Node.js (likely)",
        4200 => "Angular (likely)",
        4321 => "Astro (likely)",
        5000 => "Flask (likely)",
        5173 | 5174 => "Vite (likely)",
        8000 => "Python (likely)",
        8080 => "HTTP Server",
        8888 => "Jupyter (likely)",
        9000 => "PHP (likely)",
        // Databases & infrastructure
        3306 => "MySQL",
        5432 => "PostgreSQL",
        6379 => "Redis",
        27017 => "MongoDB",
        9200 => "Elasticsearch",
        2181 => "ZooKeeper",
        9092 => "Kafka",
        8500 => "Consul",
        _ => return None,
    };

    Some(TechInfo {
        name: name.to_string(),
        source: DetectionSource::PortHeuristic,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_ports_resolve() {
        assert_eq!(detect(5432).unwrap().name, "PostgreSQL");
        assert_eq!(detect(6379).unwrap().name, "Redis");
        assert_eq!(detect(3000).unwrap().name, "Node.js (likely)");
        assert_eq!(detect(5173).unwrap().name, "Vite (likely)");
    }

    #[test]
    fn known_ports_use_heuristic_source() {
        assert!(matches!(
            detect(5432).unwrap().source,
            DetectionSource::PortHeuristic
        ));
    }

    #[test]
    fn unknown_port_returns_none() {
        assert!(detect(12345).is_none());
    }
}
