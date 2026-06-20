use crate::types::PortEntry;
use anyhow::{Context, Result};
use std::process::Command;

pub fn kill_process(entry: &PortEntry) -> Result<()> {
    // Guard against PID reuse: the PID was captured by a scan up to a few seconds
    // ago. If that process has since exited and the OS recycled the PID, we'd be
    // signaling an unrelated process. Re-check the live process name and refuse if
    // it no longer matches what we're showing. (If we can't determine the current
    // name, fall through and let kill proceed.)
    if let Some(current) = current_process_name(entry.pid)
        && !names_match(&current, &entry.process_name)
    {
        anyhow::bail!(
            "PID {} is now '{}', not '{}' — it was likely recycled. Refresh and try again.",
            entry.pid,
            current,
            entry.process_name
        );
    }

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

    // Use output() rather than spawn(): `open`/`xdg-open` launch the browser and
    // return immediately, so this doesn't block, and output() reaps the child so
    // we don't leak zombies across a long session. Capturing stdout/stderr also
    // keeps stray output from corrupting the TUI.
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(&url)
            .output()
            .context("Failed to open browser")?;
    }
    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(&url)
            .output()
            .context("Failed to open browser — is xdg-open installed?")?;
    }

    Ok(())
}

/// Look up the current name of a live process by PID, used to detect PID reuse
/// before killing. Returns None if the process is gone or can't be queried.
fn current_process_name(pid: u32) -> Option<String> {
    let output = Command::new("ps")
        .args(["-p", &pid.to_string(), "-o", "comm="])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if name.is_empty() { None } else { Some(name) }
}

/// Compare a process name from `ps -o comm=` (often a full path) against the
/// name captured by the scanner. Matches on basename, case-insensitively, and
/// tolerates truncation differences between the two sources.
fn names_match(current: &str, recorded: &str) -> bool {
    let basename = |s: &str| -> String {
        s.rsplit(['/', '\\'])
            .next()
            .unwrap_or(s)
            .trim()
            .to_lowercase()
    };
    let a = basename(current);
    let b = basename(recorded);
    if a.is_empty() || b.is_empty() {
        return true; // can't meaningfully compare — don't block the kill
    }
    a == b || a.starts_with(&b) || b.starts_with(&a)
}

pub fn copy_url_to_clipboard(entry: &PortEntry) -> Result<()> {
    let url = format!("http://localhost:{}", entry.port);
    copy_to_clipboard(&url)
}

pub fn copy_dir_to_clipboard(entry: &PortEntry) -> Result<()> {
    let dir = entry
        .working_dir
        .as_ref()
        .context("No working directory known for this process")?;
    copy_to_clipboard(&dir.to_string_lossy())
}

fn copy_to_clipboard(text: &str) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        let mut child = Command::new("pbcopy")
            .stdin(std::process::Stdio::piped())
            .spawn()
            .context("Failed to run pbcopy")?;

        use std::io::Write;
        if let Some(ref mut stdin) = child.stdin {
            stdin
                .write_all(text.as_bytes())
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
                .write_all(text.as_bytes())
                .context("Failed to write to clipboard")?;
        }
        child.wait().context("Clipboard command failed")?;
    }

    Ok(())
}
