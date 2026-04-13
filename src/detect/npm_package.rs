use crate::types::{DetectionSource, TechInfo};
use std::path::Path;

/// Try to identify a process by inspecting npm package.json files
/// referenced in the command line. Handles patterns like:
///
/// - node_modules/.pnpm/@playwright+test@1.57.0/node_modules/@playwright/test/cli.js
/// - node_modules/.bin/next
/// - .npm/_npx/.../node_modules/some-package/bin.js
/// - node_modules/express/lib/express.js
pub fn detect(command_line: &str, working_dir: Option<&Path>) -> Option<TechInfo> {
    for arg in command_line.split_whitespace() {
        if !arg.contains("node_modules") {
            continue;
        }

        let path = Path::new(arg);

        // Try absolute path first
        if path.is_absolute() {
            if let Some(info) = find_package_json_up(path) {
                return Some(info);
            }
        }

        // Resolve relative path against working directory
        if let Some(cwd) = working_dir {
            let resolved = cwd.join(path);
            if let Some(info) = find_package_json_up(&resolved) {
                return Some(info);
            }
        }
    }

    None
}

fn find_package_json_up(script_path: &Path) -> Option<TechInfo> {
    // Walk up the directory tree looking for package.json
    // Stop when we hit a node_modules directory (that's the package root)
    let mut dir = if script_path.is_file() || !script_path.exists() {
        script_path.parent()?
    } else {
        script_path
    };

    // Walk up at most 5 levels to find a package.json
    for _ in 0..5 {
        let pkg_path = dir.join("package.json");
        if let Some(info) = read_package_json(&pkg_path) {
            return Some(info);
        }

        // Stop if we've hit the node_modules boundary
        if dir.file_name().map_or(false, |n| n == "node_modules") {
            break;
        }

        dir = dir.parent()?;
    }

    None
}

fn read_package_json(path: &Path) -> Option<TechInfo> {
    let content = std::fs::read_to_string(path).ok()?;

    // Quick JSON parsing — extract "name" and "description" fields
    let name = extract_json_string(&content, "name")?;

    // Skip generic/internal packages
    if name.is_empty() || name == "undefined" {
        return None;
    }

    let description = extract_json_string(&content, "description");

    let display = if let Some(desc) = description {
        if desc.len() <= 50 {
            format!("{name} — {desc}")
        } else {
            name.clone()
        }
    } else {
        name.clone()
    };

    Some(TechInfo {
        name: display,
        source: DetectionSource::CommandLine,
    })
}

/// Extract a simple string value from JSON without pulling in a JSON parser.
/// Handles: "key": "value" with basic escaping.
fn extract_json_string(json: &str, key: &str) -> Option<String> {
    let pattern = format!("\"{}\"", key);
    let idx = json.find(&pattern)?;
    let after_key = &json[idx + pattern.len()..];

    // Skip whitespace and colon
    let after_colon = after_key.trim_start().strip_prefix(':')?;
    let after_ws = after_colon.trim_start();

    // Must start with a quote
    let after_quote = after_ws.strip_prefix('"')?;

    // Find closing quote (handle escaped quotes)
    let mut result = String::new();
    let mut chars = after_quote.chars();
    loop {
        match chars.next()? {
            '\\' => {
                if let Some(c) = chars.next() {
                    result.push(c);
                }
            }
            '"' => break,
            c => result.push(c),
        }
    }

    if result.is_empty() {
        None
    } else {
        Some(result)
    }
}
