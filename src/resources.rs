use crate::types::PortEntry;
use std::collections::HashMap;
use std::time::Duration;
use sysinfo::{Pid, ProcessesToUpdate, System};

/// Collect CPU and memory usage for each PortEntry by PID,
/// plus per-process network I/O (cumulative bytes via nettop on macOS).
///
/// CPU measurement requires two process refreshes with a delay between them
/// so that sysinfo can compute a delta. The 200ms sleep is acceptable here
/// because this runs on the background scanner thread.
pub fn collect_resources(entries: &mut [PortEntry]) {
    if entries.is_empty() {
        return;
    }

    let pids: Vec<Pid> = entries.iter().map(|e| Pid::from_u32(e.pid)).collect();

    let mut system = System::new();

    // First refresh — seeds the baseline for CPU calculation
    system.refresh_processes(ProcessesToUpdate::Some(&pids), true);

    // Wait so sysinfo can measure a CPU time delta
    std::thread::sleep(Duration::from_millis(200));

    // Second refresh — now cpu_usage() returns meaningful values
    system.refresh_processes(ProcessesToUpdate::Some(&pids), true);

    for entry in entries.iter_mut() {
        let pid = Pid::from_u32(entry.pid);
        if let Some(process) = system.process(pid) {
            entry.cpu_usage = Some(process.cpu_usage());
            let mem_bytes = process.memory();
            entry.memory_mb = Some(mem_bytes as f64 / (1024.0 * 1024.0));
        }
    }

    // Collect per-process network I/O
    let net_map = collect_nettop();
    for entry in entries.iter_mut() {
        if let Some(&(rx, tx)) = net_map.get(&entry.pid) {
            entry.net_rx_bytes = Some(rx);
            entry.net_tx_bytes = Some(tx);
        }
    }
}

/// Parse nettop output to get cumulative network bytes per PID.
/// Returns HashMap<pid, (bytes_in, bytes_out)>.
fn collect_nettop() -> HashMap<u32, (u64, u64)> {
    let mut map: HashMap<u32, (u64, u64)> = HashMap::new();

    let Ok(output) = std::process::Command::new("nettop")
        .args(["-P", "-J", "bytes_in,bytes_out", "-l", "1", "-n", "-x"])
        .output()
    else {
        return map;
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines().skip(1) {
        // Format: "timestamp process_name.pid    bytes_in    bytes_out"
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() < 4 {
            continue;
        }

        // fields[0] is the timestamp, fields[1] is process_name.pid
        let Some(dot_pos) = fields[1].rfind('.') else {
            continue;
        };
        let Ok(pid) = fields[1][dot_pos + 1..].parse::<u32>() else {
            continue;
        };

        let rx: u64 = fields[2].parse().unwrap_or(0);
        let tx: u64 = fields[3].parse().unwrap_or(0);

        // nettop may list multiple entries per PID (multiple connections) — sum them
        let entry = map.entry(pid).or_insert((0, 0));
        entry.0 += rx;
        entry.1 += tx;
    }

    map
}
