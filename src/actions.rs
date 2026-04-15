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
