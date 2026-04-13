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
