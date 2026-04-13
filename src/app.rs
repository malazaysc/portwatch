use crate::detect;
use crate::git;
use crate::process;
use crate::resources;
use crate::scanner;
use crate::types::{DetectionSource, PortEntry, TechInfo};
use std::sync::mpsc;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortColumn {
    Port,
    Process,
    Tech,
    Uptime,
    Cpu,
    Memory,
}

impl SortColumn {
    pub fn next(self) -> Self {
        match self {
            SortColumn::Port => SortColumn::Process,
            SortColumn::Process => SortColumn::Tech,
            SortColumn::Tech => SortColumn::Uptime,
            SortColumn::Uptime => SortColumn::Cpu,
            SortColumn::Cpu => SortColumn::Memory,
            SortColumn::Memory => SortColumn::Port,
        }
    }

    #[allow(dead_code)]
    pub fn label(self) -> &'static str {
        match self {
            SortColumn::Port => "PORT",
            SortColumn::Process => "PROCESS",
            SortColumn::Tech => "TECH",
            SortColumn::Uptime => "UPTIME",
            SortColumn::Cpu => "CPU%",
            SortColumn::Memory => "MEM",
        }
    }
}

/// A group of ports sharing the same project (git repo, Docker compose project, etc.).
pub struct PortGroup {
    pub name: String,
    pub entries: Vec<usize>, // indices into App::ports
}

pub struct App {
    pub all_ports: Vec<PortEntry>,
    pub ports: Vec<PortEntry>,
    pub groups: Vec<PortGroup>,
    pub selected: usize,
    pub should_quit: bool,
    pub show_help: bool,
    pub confirm_kill: bool,
    pub status_message: Option<(String, Instant)>,
    pub scanning: bool,
    pub filter_text: String,
    pub filter_active: bool,
    pub sort_column: SortColumn,
    pub sort_ascending: bool,
    rx: mpsc::Receiver<ScanResult>,
    scan_trigger: mpsc::Sender<()>,
}

enum ScanResult {
    Data(Vec<PortEntry>),
    Error(String),
}

impl App {
    pub fn new() -> Self {
        let (data_tx, data_rx) = mpsc::channel();
        let (trigger_tx, trigger_rx) = mpsc::channel::<()>();

        std::thread::spawn(move || {
            let scanner = scanner::create_scanner();
            loop {
                if trigger_rx.recv().is_err() {
                    break;
                }
                while trigger_rx.try_recv().is_ok() {}

                let result = match scanner.scan() {
                    Ok(mut entries) => {
                        #[cfg(target_os = "macos")]
                        process::macos::batch_resolve(&mut entries);
                        #[cfg(target_os = "linux")]
                        process::linux::batch_resolve(&mut entries);
                        for entry in &mut entries {
                            entry.tech = detect::detect_tech(entry);
                        }
                        git::batch_detect(&mut entries);

                        // Enrich Docker ports with container info
                        let docker_ports = detect::docker::detect_docker_ports();
                        for entry in &mut entries {
                            if let Some(info) = docker_ports.get(&entry.port) {
                                entry.docker_info = Some(info.clone());
                                // Show project name if available, otherwise container name
                                let label = if let Some(proj) = &info.project {
                                    format!("Docker ({proj})")
                                } else {
                                    format!("Docker ({})", info.container_name)
                                };
                                entry.tech = Some(TechInfo {
                                    name: label,
                                    source: DetectionSource::CommandLine,
                                });
                            }
                        }

                        // Collect per-process CPU and memory usage
                        resources::collect_resources(&mut entries);

                        ScanResult::Data(entries)
                    }
                    Err(e) => ScanResult::Error(format!("{e}")),
                };

                if data_tx.send(result).is_err() {
                    break;
                }
            }
        });

        let app = Self {
            all_ports: Vec::new(),
            ports: Vec::new(),
            groups: Vec::new(),
            selected: 0,
            should_quit: false,
            show_help: false,
            confirm_kill: false,
            status_message: None,
            scanning: false,
            filter_text: String::new(),
            filter_active: false,
            sort_column: SortColumn::Port,
            sort_ascending: true,
            rx: data_rx,
            scan_trigger: trigger_tx,
        };

        let _ = app.scan_trigger.send(());
        app
    }

    pub fn request_refresh(&mut self) {
        if !self.scanning {
            self.scanning = true;
            let _ = self.scan_trigger.send(());
        }
    }

    pub fn poll_results(&mut self) -> bool {
        match self.rx.try_recv() {
            Ok(ScanResult::Data(entries)) => {
                self.all_ports = entries;
                self.scanning = false;
                self.apply_filter_and_sort();
                true
            }
            Ok(ScanResult::Error(e)) => {
                self.scanning = false;
                self.set_status(format!("Scan error: {e}"));
                true
            }
            Err(mpsc::TryRecvError::Empty) => false,
            Err(mpsc::TryRecvError::Disconnected) => {
                self.set_status("Scanner thread died".to_string());
                true
            }
        }
    }

    pub fn selected_entry(&self) -> Option<&PortEntry> {
        self.ports.get(self.selected)
    }

    pub fn select_next(&mut self) {
        if !self.ports.is_empty() {
            self.selected = (self.selected + 1).min(self.ports.len() - 1);
        }
    }

    pub fn select_prev(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    pub fn select_first(&mut self) {
        self.selected = 0;
    }

    pub fn select_last(&mut self) {
        if !self.ports.is_empty() {
            self.selected = self.ports.len() - 1;
        }
    }

    pub fn set_status(&mut self, msg: String) {
        self.status_message = Some((msg, Instant::now()));
    }

    pub fn clear_stale_status(&mut self) -> bool {
        if let Some((_, time)) = &self.status_message {
            if time.elapsed() > std::time::Duration::from_secs(3) {
                self.status_message = None;
                return true;
            }
        }
        false
    }

    // --- Filter methods ---

    pub fn toggle_filter(&mut self) {
        self.filter_active = true;
    }

    pub fn update_filter(&mut self, c: char) {
        self.filter_text.push(c);
        self.apply_filter_and_sort();
    }

    pub fn delete_filter_char(&mut self) {
        self.filter_text.pop();
        self.apply_filter_and_sort();
    }

    pub fn close_filter(&mut self) {
        self.filter_active = false;
    }

    #[allow(dead_code)]
    pub fn clear_filter(&mut self) {
        self.filter_text.clear();
        self.filter_active = false;
        self.apply_filter_and_sort();
    }

    // --- Sort methods ---

    pub fn cycle_sort(&mut self) {
        self.sort_column = self.sort_column.next();
        self.apply_filter_and_sort();
    }

    pub fn toggle_sort_direction(&mut self) {
        self.sort_ascending = !self.sort_ascending;
        self.apply_filter_and_sort();
    }

    // --- Internal ---

    fn apply_filter_and_sort(&mut self) {
        let filter = self.filter_text.to_lowercase();

        // Filter
        let mut filtered: Vec<PortEntry> = if filter.is_empty() {
            self.all_ports.clone()
        } else {
            self.all_ports
                .iter()
                .filter(|entry| {
                    let port_str = entry.port.to_string();
                    let process = entry.process_name.to_lowercase();
                    let tech = entry
                        .tech
                        .as_ref()
                        .map(|t| t.name.to_lowercase())
                        .unwrap_or_default();
                    let dir = entry
                        .working_dir
                        .as_ref()
                        .map(|d| d.display().to_string().to_lowercase())
                        .unwrap_or_default();

                    port_str.contains(&filter)
                        || process.contains(&filter)
                        || tech.contains(&filter)
                        || dir.contains(&filter)
                })
                .cloned()
                .collect()
        };

        // Sort
        let ascending = self.sort_ascending;
        filtered.sort_by(|a, b| {
            let cmp = match self.sort_column {
                SortColumn::Port => a.port.cmp(&b.port),
                SortColumn::Process => a
                    .process_name
                    .to_lowercase()
                    .cmp(&b.process_name.to_lowercase()),
                SortColumn::Tech => {
                    let a_tech = a
                        .tech
                        .as_ref()
                        .map(|t| t.name.to_lowercase())
                        .unwrap_or_default();
                    let b_tech = b
                        .tech
                        .as_ref()
                        .map(|t| t.name.to_lowercase())
                        .unwrap_or_default();
                    a_tech.cmp(&b_tech)
                }
                SortColumn::Uptime => {
                    let a_up = a.uptime.unwrap_or_default();
                    let b_up = b.uptime.unwrap_or_default();
                    a_up.cmp(&b_up)
                }
                SortColumn::Cpu => {
                    let a_cpu = a.cpu_usage.unwrap_or(0.0);
                    let b_cpu = b.cpu_usage.unwrap_or(0.0);
                    a_cpu.partial_cmp(&b_cpu).unwrap_or(std::cmp::Ordering::Equal)
                }
                SortColumn::Memory => {
                    let a_mem = a.memory_mb.unwrap_or(0.0);
                    let b_mem = b.memory_mb.unwrap_or(0.0);
                    a_mem.partial_cmp(&b_mem).unwrap_or(std::cmp::Ordering::Equal)
                }
            };
            if ascending { cmp } else { cmp.reverse() }
        });

        self.ports = filtered;

        // Build groups by project key
        self.groups = self.build_groups();

        // Clamp selection
        if self.ports.is_empty() {
            self.selected = 0;
        } else if self.selected >= self.ports.len() {
            self.selected = self.ports.len() - 1;
        }
    }

    fn build_groups(&self) -> Vec<PortGroup> {
        let mut groups: Vec<PortGroup> = Vec::new();

        for (i, entry) in self.ports.iter().enumerate() {
            let key = Self::project_key(entry);
            if let Some(group) = groups.iter_mut().find(|g| g.name == key) {
                group.entries.push(i);
            } else {
                groups.push(PortGroup {
                    name: key,
                    entries: vec![i],
                });
            }
        }

        groups
    }

    /// Determine the project grouping key for a port entry.
    fn project_key(entry: &PortEntry) -> String {
        // 1. Git repo root -- use the last path component as the display name
        if let Some(git) = &entry.git_info {
            if let Some(name) = git.repo_root.file_name() {
                return name.to_string_lossy().to_string();
            }
            return git.repo_root.display().to_string();
        }

        // 2. Docker compose project name
        if let Some(docker) = &entry.docker_info {
            if let Some(project) = &docker.project {
                return format!("Docker ({project})");
            }
            return format!("Docker ({})", docker.container_name);
        }

        // 3. Working directory -- use its last component
        if let Some(dir) = &entry.working_dir {
            let s = dir.display().to_string();
            if s != "/" {
                if let Some(name) = dir.file_name() {
                    return name.to_string_lossy().to_string();
                }
            }
        }

        // 4. Fallback
        "System".to_string()
    }
}
