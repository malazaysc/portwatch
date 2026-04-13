use crate::types::{DetectionSource, TechInfo};

/// Detect specific frameworks from command line patterns.
/// Returns None for generic runtimes (node, python, etc.) — those are handled
/// by detect_runtime() as a lower-priority fallback.
pub fn detect(command_line: &str) -> Option<TechInfo> {
    let cmd = command_line.to_lowercase();

    let name = if cmd.contains("next dev") || cmd.contains("next start") || cmd.contains("next-server") {
        "Next.js"
    } else if cmd.contains("vite") && !cmd.contains("invite") {
        "Vite"
    } else if cmd.contains("nuxt") {
        "Nuxt"
    } else if cmd.contains("remix") {
        "Remix"
    } else if cmd.contains("astro") && (cmd.contains("dev") || cmd.contains("preview")) {
        "Astro"
    } else if cmd.contains("webpack") && cmd.contains("serve") {
        "Webpack"
    } else if cmd.contains("angular") || cmd.contains("ng serve") {
        "Angular"
    } else if cmd.contains("manage.py") && cmd.contains("runserver") {
        "Django"
    } else if cmd.contains("flask run") || cmd.contains("flask") && cmd.contains("--app") {
        "Flask"
    } else if cmd.contains("uvicorn") {
        "FastAPI"
    } else if cmd.contains("gunicorn") {
        "Gunicorn"
    } else if cmd.contains("rails") && (cmd.contains("server") || cmd.contains("s")) {
        "Rails"
    } else if cmd.contains("puma") {
        "Puma (Ruby)"
    } else if cmd.contains("sinatra") {
        "Sinatra"
    } else if cmd.contains("php artisan serve") {
        "Laravel"
    } else if cmd.contains("hugo server") || cmd.contains("hugo serve") {
        "Hugo"
    } else if cmd.contains("jekyll serve") {
        "Jekyll"
    } else if cmd.contains("cargo run") || cmd.contains("target/debug/") || cmd.contains("target/release/") {
        "Rust"
    } else if cmd.contains("go run") {
        "Go"
    } else if cmd.contains("deno") {
        "Deno"
    } else if cmd.contains("bun") && !cmd.contains("bundle") {
        "Bun"
    } else if cmd.contains("tsx") || cmd.contains("ts-node") {
        "TypeScript"
    } else if cmd.contains("nodemon") {
        "Node.js (nodemon)"
    } else {
        return None;
    };

    Some(TechInfo {
        name: name.to_string(),
        source: DetectionSource::CommandLine,
    })
}

/// Detect IDE internals, system services, and generic runtimes.
/// Runs as a lower-priority fallback so specific detections aren't shadowed.
pub fn detect_runtime(command_line: &str) -> Option<TechInfo> {
    let cmd = command_line.to_lowercase();

    // IDE/editor internals — extract workspace name if available
    if let Some(name) = detect_ide(command_line) {
        return Some(TechInfo {
            name,
            source: DetectionSource::CommandLine,
        });
    }

    // System services
    let name = if cmd.contains("com.docker") {
        "Docker"
    } else if cmd.contains("postgres") {
        "PostgreSQL"
    } else if cmd.contains("redis-server") {
        "Redis"
    } else if cmd.contains("mongod") {
        "MongoDB"
    } else if cmd.contains("controlcenter") || cmd.contains("coreaudio") {
        "macOS System"
    } else if cmd.contains("rapportd") {
        "macOS Rapport"
    // Generic runtimes
    } else if cmd.contains("node") {
        "Node.js"
    } else if cmd.contains("python") || cmd.contains("python3") {
        "Python"
    } else if cmd.contains("ruby") {
        "Ruby"
    } else if cmd.contains("java") || cmd.contains("spring") {
        "Java"
    } else if cmd.contains("dotnet") {
        ".NET"
    } else {
        return None;
    };

    Some(TechInfo {
        name: name.to_string(),
        source: DetectionSource::CommandLine,
    })
}

/// Extract workspace/project info from IDE command lines.
/// e.g. "Cursor Helper (Plugin): extension-host (user) navaris [2-5]" → "Cursor (navaris)"
fn detect_ide(command_line: &str) -> Option<String> {
    // Cursor: "Cursor Helper (Plugin): extension-host (user) WORKSPACE [N-N]"
    if command_line.contains("Cursor") {
        if let Some(workspace) = extract_cursor_workspace(command_line) {
            return Some(format!("Cursor ({workspace})"));
        }
        return Some("Cursor (internal)".to_string());
    }

    // VS Code
    if command_line.contains("Code Helper") || command_line.contains("code-server") {
        return Some("VS Code (internal)".to_string());
    }

    // Zed
    if command_line.contains("zed") {
        return Some("Zed (internal)".to_string());
    }

    // Postman
    if command_line.contains("Postman") {
        return Some("Postman".to_string());
    }

    None
}

fn extract_cursor_workspace(cmd: &str) -> Option<String> {
    // Pattern: "extension-host (user) WORKSPACE [N-N]" or similar
    // The workspace name sits between "(user) " and the trailing " [" or end
    if let Some(idx) = cmd.find("(user) ") {
        let after = &cmd[idx + 7..]; // skip "(user) "
        let workspace = if let Some(bracket) = after.find(" [") {
            &after[..bracket]
        } else {
            after.trim()
        };
        if !workspace.is_empty() {
            return Some(workspace.to_string());
        }
    }
    None
}
