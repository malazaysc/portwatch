use crate::types::{DetectionSource, TechInfo};
use std::path::Path;

pub fn detect(dir: &Path) -> Option<TechInfo> {
    // Check package.json for JS/TS frameworks
    if let Some(tech) = check_package_json(dir) {
        return Some(tech);
    }

    // Check Cargo.toml for Rust frameworks
    if let Some(tech) = check_cargo_toml(dir) {
        return Some(tech);
    }

    // Check Python project files
    if let Some(tech) = check_python(dir) {
        return Some(tech);
    }

    // Check for other project files
    if dir.join("Gemfile").exists() {
        return Some(make("Ruby", "Gemfile"));
    }
    if dir.join("go.mod").exists() {
        return Some(make("Go", "go.mod"));
    }
    if dir.join("composer.json").exists() {
        return Some(make("PHP", "composer.json"));
    }

    None
}

fn check_package_json(dir: &Path) -> Option<TechInfo> {
    let content = super::read_metadata_file(&dir.join("package.json"))?;
    Some(match_package_json(&content))
}

/// Identify a JS/TS framework from package.json contents. Frameworks are checked
/// before generic runtimes; falls back to plain Node.js when nothing matches.
fn match_package_json(content: &str) -> TechInfo {
    // Check deps in priority order (frameworks before runtimes)
    let checks: &[(&str, &str)] = &[
        ("\"next\"", "Next.js"),
        ("\"nuxt\"", "Nuxt"),
        ("\"@remix-run/", "Remix"),
        ("\"astro\"", "Astro"),
        ("\"vite\"", "Vite"),
        ("\"@angular/core\"", "Angular"),
        ("\"svelte\"", "SvelteKit"),
        ("\"fastify\"", "Fastify"),
        ("\"express\"", "Express"),
        ("\"hono\"", "Hono"),
        ("\"koa\"", "Koa"),
        ("\"nest", "NestJS"),
    ];

    for (pattern, name) in checks {
        if content.contains(pattern) {
            return make(name, "package.json");
        }
    }

    // Generic Node.js if package.json exists but no framework matched
    make("Node.js", "package.json")
}

fn check_cargo_toml(dir: &Path) -> Option<TechInfo> {
    let content = super::read_metadata_file(&dir.join("Cargo.toml"))?;
    Some(match_cargo_toml(&content))
}

/// Identify a Rust web framework from Cargo.toml contents; falls back to Rust.
fn match_cargo_toml(content: &str) -> TechInfo {
    let checks: &[(&str, &str)] = &[
        ("axum", "Axum (Rust)"),
        ("actix-web", "Actix (Rust)"),
        ("rocket", "Rocket (Rust)"),
        ("warp", "Warp (Rust)"),
    ];

    for (pattern, name) in checks {
        if content.contains(pattern) {
            return make(name, "Cargo.toml");
        }
    }

    make("Rust", "Cargo.toml")
}

fn check_python(dir: &Path) -> Option<TechInfo> {
    // Check pyproject.toml first
    if let Some(content) = super::read_metadata_file(&dir.join("pyproject.toml")) {
        return Some(match_python_deps(&content, "pyproject.toml"));
    }

    // Check requirements.txt
    if let Some(content) = super::read_metadata_file(&dir.join("requirements.txt")) {
        return Some(match_python_deps(&content, "requirements.txt"));
    }

    // Check manage.py
    if dir.join("manage.py").exists() {
        return Some(make("Django", "manage.py"));
    }

    None
}

/// Identify a Python web framework from dependency-file contents (case
/// insensitive); falls back to plain Python.
fn match_python_deps(content: &str, source_file: &str) -> TechInfo {
    let lower = content.to_lowercase();
    if lower.contains("django") {
        make("Django", source_file)
    } else if lower.contains("flask") {
        make("Flask", source_file)
    } else if lower.contains("fastapi") {
        make("FastAPI", source_file)
    } else {
        make("Python", source_file)
    }
}

fn make(name: &str, source_file: &str) -> TechInfo {
    let _ = source_file; // used for debugging context, could be stored later
    TechInfo {
        name: name.to_string(),
        source: DetectionSource::ProjectFile,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_json_prefers_framework_over_node() {
        let pkg = r#"{ "dependencies": { "next": "14", "react": "18" } }"#;
        assert_eq!(match_package_json(pkg).name, "Next.js");
    }

    #[test]
    fn package_json_priority_orders_next_before_vite() {
        // A project listing both should report the higher-priority framework.
        let pkg = r#"{ "dependencies": { "vite": "5", "next": "14" } }"#;
        assert_eq!(match_package_json(pkg).name, "Next.js");
    }

    #[test]
    fn package_json_falls_back_to_node() {
        let pkg = r#"{ "dependencies": { "lodash": "4" } }"#;
        assert_eq!(match_package_json(pkg).name, "Node.js");
    }

    #[test]
    fn cargo_toml_detects_axum_then_falls_back_to_rust() {
        assert_eq!(
            match_cargo_toml("[dependencies]\naxum = \"0.7\"").name,
            "Axum (Rust)"
        );
        assert_eq!(
            match_cargo_toml("[dependencies]\nserde = \"1\"").name,
            "Rust"
        );
    }

    #[test]
    fn python_deps_are_case_insensitive() {
        assert_eq!(
            match_python_deps("Django==5.0", "requirements.txt").name,
            "Django"
        );
        assert_eq!(match_python_deps("flask", "pyproject.toml").name, "Flask");
        assert_eq!(
            match_python_deps("numpy", "requirements.txt").name,
            "Python"
        );
    }
}
