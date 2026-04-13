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
            anyhow::bail!("Permission denied — PID {} is owned by another user", entry.pid);
        }
        anyhow::bail!(
            "Failed to kill PID {}: {}",
            entry.pid,
            stderr.trim()
        );
    }
    Ok(())
}

pub fn open_in_browser(entry: &PortEntry) -> Result<()> {
    let url = format!("http://localhost:{}", entry.port);
    Command::new("open")
        .arg(&url)
        .spawn()
        .context("Failed to open browser")?;
    Ok(())
}

pub fn open_folder(entry: &PortEntry) -> Result<()> {
    let dir = entry
        .working_dir
        .as_ref()
        .context("No working directory known for this process")?;

    // Open in Finder as a reliable fallback; terminal tab opening is terminal-specific
    Command::new("open")
        .arg(dir)
        .spawn()
        .context("Failed to open folder")?;
    Ok(())
}

pub fn copy_url_to_clipboard(entry: &PortEntry) -> Result<()> {
    let url = format!("http://localhost:{}", entry.port);
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
    Ok(())
}
