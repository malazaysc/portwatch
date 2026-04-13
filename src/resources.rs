use crate::types::PortEntry;
use sysinfo::{Pid, ProcessesToUpdate, System};
use std::time::Duration;

/// Collect CPU and memory usage for each PortEntry by PID.
///
/// CPU measurement requires two process refreshes with a delay between them
/// so that sysinfo can compute a delta. The 200ms sleep is acceptable here
/// because this runs on the background scanner thread.
pub fn collect_resources(entries: &mut [PortEntry]) {
    if entries.is_empty() {
        return;
    }

    let pids: Vec<Pid> = entries
        .iter()
        .map(|e| Pid::from_u32(e.pid))
        .collect();

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
}
