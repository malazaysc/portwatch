pub mod docker;
mod command_line;
mod npm_package;
mod port_hints;
mod project_files;

use crate::types::{PortEntry, TechInfo};

pub fn detect_tech(entry: &PortEntry) -> Option<TechInfo> {
    // Priority order:
    // 1. Specific framework patterns in command line (next dev, vite, django, etc.)
    // 2. npm package.json from node_modules paths in command line
    // 3. Known non-server apps (browsers, IDEs, system services) — before project
    //    files so Chrome's cwd in a Rust project doesn't get labeled "Axum"
    // 4. Project files in working directory (package.json, Cargo.toml, etc.)
    // 5. Generic runtime fallback (node, python, ruby, java)
    // 6. Port-based heuristics
    command_line::detect(&entry.command_line)
        .or_else(|| npm_package::detect(&entry.command_line, entry.working_dir.as_deref()))
        .or_else(|| command_line::detect_app(&entry.command_line))
        .or_else(|| {
            entry
                .working_dir
                .as_ref()
                .and_then(|dir| project_files::detect(dir))
        })
        .or_else(|| command_line::detect_runtime(&entry.command_line))
        .or_else(|| port_hints::detect(entry.port))
}
