use crate::types::PortEntry;
use anyhow::{Context, Result};
use std::process::Command;

pub fn kill_process(entry: &PortEntry) -> Result<()> {
    let output = Command::new("kill")
        .arg(entry.pid.to_string())
        .output()
        .context("Failed to run kill command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("Operation not permitted") || stderr.contains("Not permitted") {
            anyhow::bail!(
                "Permission denied — PID {} is owned by another user",
                entry.pid
            );
        }
        anyhow::bail!("Failed to kill PID {}: {}", entry.pid, stderr.trim());
    }
    Ok(())
}

pub fn open_in_browser(entry: &PortEntry) -> Result<()> {
    let url = format!("http://localhost:{}", entry.port);

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(&url)
            .spawn()
            .context("Failed to open browser")?;
    }
    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(&url)
            .spawn()
            .context("Failed to open browser — is xdg-open installed?")?;
    }

    Ok(())
}

pub fn open_folder(entry: &PortEntry, terminal: &str) -> Result<()> {
    let dir = entry
        .working_dir
        .as_ref()
        .context("No working directory known for this process")?;

    #[cfg(target_os = "macos")]
    let dir_str = dir.to_string_lossy();

    match terminal {
        #[cfg(target_os = "macos")]
        "iterm2" => {
            let script = format!(
                r#"tell application "iTerm2"
    activate
    tell current window
        create tab with default profile
        tell current session
            write text "cd {}"
        end tell
    end tell
end tell"#,
                shell_escape(&dir_str)
            );
            Command::new("osascript")
                .arg("-e")
                .arg(&script)
                .spawn()
                .context("Failed to open iTerm2 tab")?;
        }
        #[cfg(target_os = "macos")]
        "terminal" => {
            let script = format!(
                r#"tell application "Terminal"
    activate
    do script "cd {}"
end tell"#,
                shell_escape(&dir_str)
            );
            Command::new("osascript")
                .arg("-e")
                .arg(&script)
                .spawn()
                .context("Failed to open Terminal.app tab")?;
        }
        "wezterm" => {
            Command::new("wezterm")
                .args(["cli", "spawn", "--cwd"])
                .arg(dir)
                .spawn()
                .context("Failed to open wezterm")?;
        }
        "alacritty" => {
            Command::new("alacritty")
                .arg("--working-directory")
                .arg(dir)
                .spawn()
                .context("Failed to open alacritty")?;
        }
        "kitty" => {
            Command::new("kitty")
                .arg("--directory")
                .arg(dir)
                .spawn()
                .context("Failed to open kitty")?;
        }
        _ => {
            // Default: open in file manager
            #[cfg(target_os = "macos")]
            {
                // "finder" or any unknown value — open in Finder
                Command::new("open")
                    .arg(dir)
                    .spawn()
                    .context("Failed to open folder")?;
            }
            #[cfg(target_os = "linux")]
            {
                // Open in default file manager via xdg-open
                Command::new("xdg-open")
                    .arg(dir)
                    .spawn()
                    .context("Failed to open folder — is xdg-open installed?")?;
            }
        }
    }
    Ok(())
}

/// Escape a string for use inside AppleScript double-quoted strings.
#[cfg(target_os = "macos")]
fn shell_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

pub fn copy_url_to_clipboard(entry: &PortEntry) -> Result<()> {
    let url = format!("http://localhost:{}", entry.port);

    #[cfg(target_os = "macos")]
    {
        let mut child = Command::new("pbcopy")
            .stdin(std::process::Stdio::piped())
            .spawn()
            .context("Failed to run pbcopy")?;

        use std::io::Write;
        if let Some(ref mut stdin) = child.stdin {
            stdin
                .write_all(url.as_bytes())
                .context("Failed to write to pbcopy")?;
        }
        child.wait().context("pbcopy failed")?;
    }

    #[cfg(target_os = "linux")]
    {
        // Try xclip first, fall back to xsel
        let result = Command::new("xclip")
            .args(["-selection", "clipboard"])
            .stdin(std::process::Stdio::piped())
            .spawn();

        let mut child = match result {
            Ok(child) => child,
            Err(_) => Command::new("xsel")
                .arg("--clipboard")
                .stdin(std::process::Stdio::piped())
                .spawn()
                .context("Failed to copy to clipboard — install xclip or xsel")?,
        };

        use std::io::Write;
        if let Some(ref mut stdin) = child.stdin {
            stdin
                .write_all(url.as_bytes())
                .context("Failed to write to clipboard")?;
        }
        child.wait().context("Clipboard command failed")?;
    }

    Ok(())
}
