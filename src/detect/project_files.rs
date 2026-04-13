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
    let path = dir.join("package.json");
    let content = std::fs::read_to_string(path).ok()?;

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
            return Some(make(name, "package.json"));
        }
    }

    // Generic Node.js if package.json exists but no framework matched
    Some(make("Node.js", "package.json"))
}

fn check_cargo_toml(dir: &Path) -> Option<TechInfo> {
    let path = dir.join("Cargo.toml");
    let content = std::fs::read_to_string(path).ok()?;

    let checks: &[(&str, &str)] = &[
        ("axum", "Axum (Rust)"),
        ("actix-web", "Actix (Rust)"),
        ("rocket", "Rocket (Rust)"),
        ("warp", "Warp (Rust)"),
    ];

    for (pattern, name) in checks {
        if content.contains(pattern) {
            return Some(make(name, "Cargo.toml"));
        }
    }

    Some(make("Rust", "Cargo.toml"))
}

fn check_python(dir: &Path) -> Option<TechInfo> {
    // Check pyproject.toml first
    if let Ok(content) = std::fs::read_to_string(dir.join("pyproject.toml")) {
        if content.contains("django") {
            return Some(make("Django", "pyproject.toml"));
        }
        if content.contains("flask") {
            return Some(make("Flask", "pyproject.toml"));
        }
        if content.contains("fastapi") {
            return Some(make("FastAPI", "pyproject.toml"));
        }
        return Some(make("Python", "pyproject.toml"));
    }

    // Check requirements.txt
    if let Ok(content) = std::fs::read_to_string(dir.join("requirements.txt")) {
        if content.contains("django") || content.contains("Django") {
            return Some(make("Django", "requirements.txt"));
        }
        if content.contains("flask") || content.contains("Flask") {
            return Some(make("Flask", "requirements.txt"));
        }
        if content.contains("fastapi") || content.contains("FastAPI") {
            return Some(make("FastAPI", "requirements.txt"));
        }
        return Some(make("Python", "requirements.txt"));
    }

    // Check manage.py
    if dir.join("manage.py").exists() {
        return Some(make("Django", "manage.py"));
    }

    None
}

fn make(name: &str, source_file: &str) -> TechInfo {
    let _ = source_file; // used for debugging context, could be stored later
    TechInfo {
        name: name.to_string(),
        source: DetectionSource::ProjectFile,
    }
}
