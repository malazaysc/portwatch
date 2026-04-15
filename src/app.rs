use crate::detect;
use crate::git;
use crate::process;
use crate::resources;
use crate::scanner;
use crate::types::{DetectionSource, NetworkStats, PortEntry, TechInfo};
use std::collections::HashMap;
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

/// A display row in the port list — either a group header or a port entry.
#[derive(Clone)]
pub enum DisplayRow {
    GroupHeader {
        name: String,
        count: usize,
        collapsed: bool,
    },
    Port(usize), // index into App::ports
}

pub struct App {
    pub all_ports: Vec<PortEntry>,
    pub ports: Vec<PortEntry>,
    pub display_rows: Vec<DisplayRow>,
    pub selected: usize, // index into display_rows
    pub should_quit: bool,
    pub show_help: bool,
    pub confirm_kill: bool,
    pub status_message: Option<(String, Instant)>,
    pub scanning: bool,
    pub filter_text: String,
    pub filter_active: bool,
    pub sort_column: SortColumn,
    pub sort_ascending: bool,
    pub network_stats: NetworkStats,
    prev_net: HashMap<u32, (u64, u64)>,
    prev_net_time: Option<Instant>,
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

                        // Collect per-process CPU, memory, and network I/O
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
            display_rows: Vec::new(),
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
            network_stats: NetworkStats::default(),
            prev_net: HashMap::new(),
            prev_net_time: None,
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
            Ok(ScanResult::Data(mut entries)) => {
                let now = Instant::now();
                self.compute_net_rates(&mut entries, now);
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
        if let Some(DisplayRow::Port(idx)) = self.display_rows.get(self.selected) {
            self.ports.get(*idx)
        } else {
            None
        }
    }

    pub fn select_next(&mut self) {
        if self.selected + 1 < self.display_rows.len() {
            self.selected += 1;
            self.auto_expand_if_needed();
        }
    }

    pub fn select_prev(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
            self.auto_expand_if_needed();
        }
    }

    pub fn select_first(&mut self) {
        self.selected = 0;
        self.auto_expand_if_needed();
    }

    pub fn select_last(&mut self) {
        if !self.display_rows.is_empty() {
            self.selected = self.display_rows.len() - 1;
            self.auto_expand_if_needed();
        }
    }

    /// If cursor lands on a collapsed group header, expand it automatically.
    fn auto_expand_if_needed(&mut self) {
        if let Some(DisplayRow::GroupHeader { collapsed, .. }) =
            self.display_rows.get(self.selected)
            && *collapsed
        {
            self.expand_group();
        }
    }

    /// Expand the group at cursor
    pub fn expand_group(&mut self) {
        if let Some(DisplayRow::GroupHeader {
            name, collapsed, ..
        }) = self.display_rows.get(self.selected)
            && *collapsed
        {
            let name = name.clone();
            for row in &mut self.display_rows {
                if let DisplayRow::GroupHeader {
                    name: n,
                    collapsed: c,
                    ..
                } = row
                    && *n == name
                {
                    *c = false;
                    break;
                }
            }
            self.rebuild_display_rows();
        }
    }

    /// Collapse the group at cursor (or the group containing the current port)
    pub fn collapse_group(&mut self) {
        let group_name = match self.display_rows.get(self.selected) {
            Some(DisplayRow::GroupHeader { name, .. }) => name.clone(),
            Some(DisplayRow::Port(idx)) => Self::project_key(&self.ports[*idx]),
            None => return,
        };

        for row in &mut self.display_rows {
            if let DisplayRow::GroupHeader {
                name, collapsed, ..
            } = row
                && *name == group_name
            {
                *collapsed = true;
                break;
            }
        }
        self.rebuild_display_rows();

        // Move cursor to the group header
        for (i, row) in self.display_rows.iter().enumerate() {
            if let DisplayRow::GroupHeader { name, .. } = row
                && *name == group_name
            {
                self.selected = i;
                return;
            }
        }
    }

    pub fn set_status(&mut self, msg: String) {
        self.status_message = Some((msg, Instant::now()));
    }

    pub fn clear_stale_status(&mut self) -> bool {
        if let Some((_, time)) = &self.status_message
            && time.elapsed() > std::time::Duration::from_secs(3)
        {
            self.status_message = None;
            return true;
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

    /// Compute per-process network rates from cumulative byte deltas,
    /// and update the aggregate NetworkStats for the status bar.
    fn compute_net_rates(&mut self, entries: &mut [PortEntry], now: Instant) {
        let mut total_rx_rate: u64 = 0;
        let mut total_tx_rate: u64 = 0;

        if let Some(prev_time) = self.prev_net_time {
            let elapsed = now.duration_since(prev_time).as_secs_f64();
            if elapsed > 0.0 {
                for entry in entries.iter_mut() {
                    if let (Some(rx), Some(tx)) = (entry.net_rx_bytes, entry.net_tx_bytes) {
                        if let Some(&(prev_rx, prev_tx)) = self.prev_net.get(&entry.pid) {
                            let rx_rate = rx.saturating_sub(prev_rx) as f64 / elapsed;
                            let tx_rate = tx.saturating_sub(prev_tx) as f64 / elapsed;
                            entry.net_rx_rate = Some(rx_rate as u64);
                            entry.net_tx_rate = Some(tx_rate as u64);
                            total_rx_rate += rx_rate as u64;
                            total_tx_rate += tx_rate as u64;
                        }
                    }
                }
            }
        }

        // Store current readings for next delta
        self.prev_net.clear();
        for entry in entries.iter() {
            if let (Some(rx), Some(tx)) = (entry.net_rx_bytes, entry.net_tx_bytes) {
                self.prev_net.insert(entry.pid, (rx, tx));
            }
        }
        self.prev_net_time = Some(now);

        self.network_stats = NetworkStats {
            rx_bytes_per_sec: total_rx_rate,
            tx_bytes_per_sec: total_tx_rate,
        };
    }

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
                    a_cpu
                        .partial_cmp(&b_cpu)
                        .unwrap_or(std::cmp::Ordering::Equal)
                }
                SortColumn::Memory => {
                    let a_mem = a.memory_mb.unwrap_or(0.0);
                    let b_mem = b.memory_mb.unwrap_or(0.0);
                    a_mem
                        .partial_cmp(&b_mem)
                        .unwrap_or(std::cmp::Ordering::Equal)
                }
            };
            if ascending { cmp } else { cmp.reverse() }
        });

        self.ports = filtered;

        // Preserve collapse state from current display_rows
        let collapsed_groups: Vec<String> = self
            .display_rows
            .iter()
            .filter_map(|row| match row {
                DisplayRow::GroupHeader {
                    name, collapsed, ..
                } if *collapsed => Some(name.clone()),
                _ => None,
            })
            .collect();

        // Build groups
        let mut group_order: Vec<String> = Vec::new();
        let mut group_entries: std::collections::HashMap<String, Vec<usize>> =
            std::collections::HashMap::new();

        for (i, entry) in self.ports.iter().enumerate() {
            let key = Self::project_key(entry);
            if !group_order.contains(&key) {
                group_order.push(key.clone());
            }
            group_entries.entry(key).or_default().push(i);
        }

        // Build display rows
        let show_groups = group_order.len() > 1;
        let mut rows = Vec::new();

        for group_name in &group_order {
            let entries = &group_entries[group_name];
            let collapsed = collapsed_groups.contains(group_name);

            if show_groups {
                rows.push(DisplayRow::GroupHeader {
                    name: group_name.clone(),
                    count: entries.len(),
                    collapsed,
                });
            }

            if !collapsed || !show_groups {
                for &idx in entries {
                    rows.push(DisplayRow::Port(idx));
                }
            }
        }

        self.display_rows = rows;

        // Clamp selection to a valid port row
        if self.display_rows.is_empty() {
            self.selected = 0;
        } else if self.selected >= self.display_rows.len() {
            self.selected = self.display_rows.len() - 1;
        }
    }

    fn rebuild_display_rows(&mut self) {
        // Collect current collapse state
        let mut collapsed_map: std::collections::HashMap<String, bool> =
            std::collections::HashMap::new();
        for row in &self.display_rows {
            if let DisplayRow::GroupHeader {
                name, collapsed, ..
            } = row
            {
                collapsed_map.insert(name.clone(), *collapsed);
            }
        }

        // Rebuild groups
        let mut group_order: Vec<String> = Vec::new();
        let mut group_entries: std::collections::HashMap<String, Vec<usize>> =
            std::collections::HashMap::new();

        for (i, entry) in self.ports.iter().enumerate() {
            let key = Self::project_key(entry);
            if !group_order.contains(&key) {
                group_order.push(key.clone());
            }
            group_entries.entry(key).or_default().push(i);
        }

        let show_groups = group_order.len() > 1;
        let mut rows = Vec::new();

        for group_name in &group_order {
            let entries = &group_entries[group_name];
            let collapsed = collapsed_map.get(group_name).copied().unwrap_or(false);

            if show_groups {
                rows.push(DisplayRow::GroupHeader {
                    name: group_name.clone(),
                    count: entries.len(),
                    collapsed,
                });
            }

            if !collapsed || !show_groups {
                for &idx in entries {
                    rows.push(DisplayRow::Port(idx));
                }
            }
        }

        self.display_rows = rows;

        if self.selected >= self.display_rows.len() && !self.display_rows.is_empty() {
            self.selected = self.display_rows.len() - 1;
        }
    }

    /// Determine the project grouping key for a port entry.
    fn project_key(entry: &PortEntry) -> String {
        // 1. Git repo root
        if let Some(git) = &entry.git_info {
            if let Some(name) = git.repo_root.file_name() {
                return name.to_string_lossy().to_string();
            }
            return git.repo_root.display().to_string();
        }

        // 2. Docker compose project
        if let Some(docker) = &entry.docker_info {
            if let Some(project) = &docker.project {
                return project.clone();
            }
            return format!("Docker ({})", docker.container_name);
        }

        // 3. IDE/app with workspace name in tech label — e.g. "Cursor (navaris)" → "navaris"
        if let Some(tech) = &entry.tech
            && let Some(project) = extract_parens_project(&tech.name)
        {
            return project;
        }

        // 4. Working directory
        if let Some(dir) = &entry.working_dir {
            let s = dir.display().to_string();
            if s != "/"
                && let Some(name) = dir.file_name()
            {
                return name.to_string_lossy().to_string();
            }
        }

        // 5. Group by process name for known apps (Postman, Zed, etc.)
        let name = &entry.process_name;
        if matches!(
            name.as_str(),
            "Postman" | "zed" | "Google" | "ControlCe" | "rapportd"
        ) {
            return entry
                .tech
                .as_ref()
                .map(|t| t.name.clone())
                .unwrap_or_else(|| name.clone());
        }

        "System".to_string()
    }
}

/// Extract project name from parentheses: "Cursor (navaris)" → "navaris"
fn extract_parens_project(label: &str) -> Option<String> {
    let start = label.find('(')?;
    let end = label.find(')')?;
    if end > start + 1 {
        let project = &label[start + 1..end];
        // Skip generic labels
        if !matches!(project, "internal" | "debug port" | "likely") {
            return Some(project.to_string());
        }
    }
    None
}
