use crate::types::GitInfo;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Batch detect git info for multiple directories at once.
/// Deduplicates by directory to avoid redundant git calls.
pub fn batch_detect(entries: &mut [crate::types::PortEntry]) {
    // Collect unique directories
    let mut dirs: Vec<PathBuf> = Vec::new();
    for entry in entries.iter() {
        if let Some(dir) = &entry.working_dir
            && !dirs.contains(dir)
        {
            dirs.push(dir.clone());
        }
    }

    // Detect git info for each unique directory
    let mut cache: HashMap<PathBuf, Option<GitInfo>> = HashMap::new();
    for dir in &dirs {
        cache.insert(dir.clone(), detect_git_info(dir));
    }

    // Apply results
    for entry in entries.iter_mut() {
        if let Some(dir) = &entry.working_dir
            && let Some(info) = cache.get(dir)
        {
            entry.git_info = info.clone();
        }
    }
}

fn detect_git_info(dir: &Path) -> Option<GitInfo> {
    // Combine multiple git queries into fewer calls using stderr suppression
    // First check: is this even a git repo?
    let output = Command::new("git")
        .args([
            "rev-parse",
            "--is-inside-work-tree",
            "--show-toplevel",
            "--git-dir",
            "--git-common-dir",
        ])
        .current_dir(dir)
        .stderr(std::process::Stdio::null())
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    if lines.len() < 4 {
        return None;
    }

    // lines[0] = "true" (is-inside-work-tree)
    // lines[1] = repo root
    // lines[2] = git dir
    // lines[3] = git common dir
    let repo_root = PathBuf::from(lines[1]);
    let is_worktree = lines[2] != lines[3];

    // Get branch name in a second call
    let branch_output = Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir(dir)
        .stderr(std::process::Stdio::null())
        .output()
        .ok()?;

    let branch = String::from_utf8_lossy(&branch_output.stdout)
        .trim()
        .to_string();
    let branch = if branch.is_empty() {
        Command::new("git")
            .args(["rev-parse", "--short", "HEAD"])
            .current_dir(dir)
            .stderr(std::process::Stdio::null())
            .output()
            .ok()
            .map(|o| format!("({})", String::from_utf8_lossy(&o.stdout).trim()))
            .unwrap_or_else(|| "detached".to_string())
    } else {
        branch
    };

    Some(GitInfo {
        branch,
        repo_root,
        is_worktree,
    })
}
