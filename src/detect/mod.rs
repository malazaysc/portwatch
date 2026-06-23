mod command_line;
pub mod docker;
mod npm_package;
mod port_hints;
mod project_files;

use crate::types::{PortEntry, TechInfo};
use std::path::Path;

/// Upper bound on the size of a project-metadata file we'll read during tech
/// detection (package.json, Cargo.toml, requirements.txt, ...). These files are
/// normally a few KB; the cap bounds memory use on the synchronous scan path.
const MAX_METADATA_FILE_BYTES: u64 = 1024 * 1024; // 1 MiB

/// Read a small text file for detection, refusing anything that isn't a regular
/// file within the size cap.
///
/// Detection paths are influenced by untrusted local processes: a port owner
/// chooses its own working directory and command-line, which is where we look
/// for these files. A hostile (or merely broken) process could point us at a
/// special file or symlink — e.g. `/dev/zero`, a FIFO, or a multi-gigabyte file
/// — and an unbounded `read_to_string` would hang or exhaust memory. The
/// `is_file()` check (which follows symlinks) rejects devices/FIFOs, the length
/// check rejects oversized regular files, and the bounded read caps the result
/// even if the file grows between the stat and the read.
pub(crate) fn read_metadata_file(path: &Path) -> Option<String> {
    use std::io::Read;

    let meta = std::fs::metadata(path).ok()?;
    if !meta.is_file() || meta.len() > MAX_METADATA_FILE_BYTES {
        return None;
    }

    let mut buf = String::new();
    std::fs::File::open(path)
        .ok()?
        .take(MAX_METADATA_FILE_BYTES)
        .read_to_string(&mut buf)
        .ok()?;
    Some(buf)
}

pub fn detect_tech(entry: &PortEntry) -> Option<TechInfo> {
    // Priority order:
    // 1. Specific framework patterns in command line (next dev, vite, django, etc.)
    // 2. npm package.json from node_modules paths in command line
    // 3. Known non-server apps (browsers, IDEs, system services) — before project
    //    files so Chrome's cwd in a Rust project doesn't get labeled "Axum"
    // 4. Project files in working directory (package.json, Cargo.toml, etc.)
    // 5. Generic runtime fallback (node, python, ruby, java)
    // 6. Port-based heuristics
    command_line::detect(&entry.command_line)
        .or_else(|| npm_package::detect(&entry.command_line, entry.working_dir.as_deref()))
        .or_else(|| command_line::detect_app(&entry.command_line))
        .or_else(|| {
            entry
                .working_dir
                .as_ref()
                .and_then(|dir| project_files::detect(dir))
        })
        .or_else(|| command_line::detect_runtime(&entry.command_line))
        .or_else(|| port_hints::detect(entry.port))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::sync::atomic::{AtomicU32, Ordering};

    /// Unique temp path per call, avoiding collisions across parallel tests
    /// without depending on the `tempfile` crate.
    fn temp_path(tag: &str) -> std::path::PathBuf {
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "portwatch_test_{}_{}_{}",
            std::process::id(),
            n,
            tag
        ))
    }

    #[test]
    fn reads_a_small_regular_file() {
        let path = temp_path("small");
        std::fs::File::create(&path)
            .unwrap()
            .write_all(b"hello")
            .unwrap();
        assert_eq!(read_metadata_file(&path).as_deref(), Some("hello"));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn rejects_a_directory() {
        // A directory is not a regular file — must not be read.
        let dir = std::env::temp_dir();
        assert_eq!(read_metadata_file(&dir), None);
    }

    #[test]
    fn rejects_missing_file() {
        assert_eq!(read_metadata_file(&temp_path("missing")), None);
    }

    #[test]
    fn rejects_oversized_file() {
        let path = temp_path("big");
        let mut f = std::fs::File::create(&path).unwrap();
        // One byte over the cap.
        let chunk = vec![b'x'; 64 * 1024];
        let mut written = 0u64;
        while written <= MAX_METADATA_FILE_BYTES {
            f.write_all(&chunk).unwrap();
            written += chunk.len() as u64;
        }
        drop(f);
        assert_eq!(read_metadata_file(&path), None);
        let _ = std::fs::remove_file(&path);
    }
}
