use crate::types::{DetectionSource, TechInfo};

/// Detect specific frameworks from command line patterns.
/// Returns None for generic runtimes (node, python, etc.) — those are handled
/// by detect_runtime() as a lower-priority fallback.
pub fn detect(command_line: &str) -> Option<TechInfo> {
    let cmd = command_line.to_lowercase();

    let name =
        if cmd.contains("next dev") || cmd.contains("next start") || cmd.contains("next-server") {
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
        } else if cmd.contains("rails server") || cmd.contains("rails s ") {
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
        } else if cmd.contains("cargo run")
            || cmd.contains("target/debug/")
            || cmd.contains("target/release/")
        {
            "Rust"
        } else if cmd.contains("go run") {
            "Go"
        } else if cmd.contains("deno") {
            "Deno"
        } else if cmd
            .split(|c: char| c.is_whitespace() || c == '/')
            .any(|t| t == "bun" || t == "bunx")
        {
            // Match the `bun` runtime by exact path/arg token, not a bare substring.
            // A substring check mislabels `/home/ubuntu/...` as "Bun" and needs a
            // brittle `bundle` exclusion; an exact-token check avoids both.
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

/// Detect known non-server apps: browsers, IDEs, system services.
/// These should be identified BEFORE project file scanning, because their cwd
/// may happen to be a project directory (e.g. Chrome cwd in a Rust project).
pub fn detect_app(command_line: &str) -> Option<TechInfo> {
    let cmd = command_line.to_lowercase();

    // IDE/editor internals
    if let Some(name) = detect_ide(command_line) {
        return Some(TechInfo {
            name,
            source: DetectionSource::CommandLine,
        });
    }

    // Browsers
    let name = if cmd.contains("google chrome") {
        "Chrome (debug port)"
    } else if cmd.contains("firefox") {
        "Firefox (debug port)"
    } else if cmd.contains("safari") && !cmd.contains("safariplatform") {
        "Safari (debug port)"
    } else if cmd.contains("brave") {
        "Brave (debug port)"
    } else if cmd.contains("arc") && cmd.contains("browser") {
        "Arc (debug port)"
    // System services
    } else if cmd.contains("com.docker") {
        "Docker"
    } else if cmd.contains("controlcenter") || cmd.contains("coreaudio") {
        "macOS System"
    } else if cmd.contains("rapportd") {
        "macOS Rapport"
    } else {
        return None;
    };

    Some(TechInfo {
        name: name.to_string(),
        source: DetectionSource::CommandLine,
    })
}

/// Detect generic runtimes and databases as a last resort before port heuristics.
pub fn detect_runtime(command_line: &str) -> Option<TechInfo> {
    let cmd = command_line.to_lowercase();

    let name = if cmd.contains("postgres") {
        "PostgreSQL"
    } else if cmd.contains("redis-server") {
        "Redis"
    } else if cmd.contains("mongod") {
        "MongoDB"
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

#[cfg(test)]
mod tests {
    use super::*;

    fn name_of(info: Option<TechInfo>) -> Option<String> {
        info.map(|t| t.name)
    }

    #[test]
    fn detects_frameworks_from_command_line() {
        assert_eq!(name_of(detect("node next dev")).as_deref(), Some("Next.js"));
        assert_eq!(
            name_of(detect("/path/.bin/vite --host")).as_deref(),
            Some("Vite")
        );
        assert_eq!(
            name_of(detect("python manage.py runserver")).as_deref(),
            Some("Django")
        );
        assert_eq!(
            name_of(detect("uvicorn app:app")).as_deref(),
            Some("FastAPI")
        );
        assert_eq!(
            name_of(detect("/app/target/release/server")).as_deref(),
            Some("Rust")
        );
    }

    #[test]
    fn astro_requires_dev_or_preview() {
        // Bare "astro" (e.g. a path) should not match; needs a dev/preview verb.
        assert_eq!(name_of(detect("/usr/lib/astro-utils")), None);
        assert_eq!(name_of(detect("astro dev")).as_deref(), Some("Astro"));
    }

    #[test]
    fn vite_does_not_match_invite() {
        assert_eq!(name_of(detect("node ./invite-service.js")), None);
    }

    #[test]
    fn bun_matches_runtime_by_exact_token() {
        assert_eq!(name_of(detect("bun run dev")).as_deref(), Some("Bun"));
        assert_eq!(
            name_of(detect("bunx create-next-app")).as_deref(),
            Some("Bun")
        );
        assert_eq!(
            name_of(detect("/home/me/.bun/bin/bun start")).as_deref(),
            Some("Bun")
        );
        // A script literally named `bundle` run by bun is still bun.
        assert_eq!(
            name_of(detect("/home/me/.bun/bin/bun run bundle")).as_deref(),
            Some("Bun")
        );
    }

    #[test]
    fn bun_does_not_match_substrings() {
        // Regression: a path containing "ubuntu" must not be mislabeled "Bun".
        assert_eq!(name_of(detect("/home/ubuntu/app/myserver")), None);
        // A process whose name merely contains "bun" is not the bun runtime.
        assert_eq!(name_of(detect("/usr/bin/bunny-server")), None);
        // "bundle" (Ruby) must not trip the bun branch.
        assert_eq!(name_of(detect("ruby /usr/bin/bundle install")), None);
    }

    #[test]
    fn generic_runtime_returns_none_in_framework_detect() {
        // Bare runtimes are handled by detect_runtime, not detect().
        assert_eq!(name_of(detect("node server.js")), None);
        assert_eq!(name_of(detect("python3 app.py")), None);
    }

    #[test]
    fn detect_runtime_identifies_bare_runtimes() {
        assert_eq!(
            name_of(detect_runtime("node server.js")).as_deref(),
            Some("Node.js")
        );
        assert_eq!(
            name_of(detect_runtime("/usr/bin/postgres -D /data")).as_deref(),
            Some("PostgreSQL")
        );
        assert_eq!(name_of(detect_runtime("./mystery-binary")), None);
    }

    #[test]
    fn detect_app_identifies_browsers_and_services() {
        assert_eq!(
            name_of(detect_app("Google Chrome Helper --type=renderer")).as_deref(),
            Some("Chrome (debug port)")
        );
        assert_eq!(
            name_of(detect_app("com.docker.backend")).as_deref(),
            Some("Docker")
        );
        assert_eq!(name_of(detect_app("some-random-daemon")), None);
    }

    #[test]
    fn detect_app_extracts_cursor_workspace() {
        let cmd = "Cursor Helper (Plugin): extension-host (user) navaris [2-5]";
        assert_eq!(
            name_of(detect_app(cmd)).as_deref(),
            Some("Cursor (navaris)")
        );
        // Cursor process with no parseable workspace falls back to "internal".
        assert_eq!(
            name_of(detect_app("Cursor Helper (Renderer)")).as_deref(),
            Some("Cursor (internal)")
        );
    }

    #[test]
    fn extract_cursor_workspace_handles_trailing_bracket_and_eol() {
        assert_eq!(
            extract_cursor_workspace("x (user) navaris [2-5]").as_deref(),
            Some("navaris")
        );
        assert_eq!(
            extract_cursor_workspace("x (user) my-app").as_deref(),
            Some("my-app")
        );
        assert_eq!(extract_cursor_workspace("no marker here"), None);
    }
}
