use crate::detect;
use crate::git;
use crate::process;
use crate::scanner;
use crate::types::PortEntry;
use std::sync::mpsc;
use std::time::Instant;

pub struct App {
    pub ports: Vec<PortEntry>,
    pub selected: usize,
    pub should_quit: bool,
    pub show_help: bool,
    pub confirm_kill: bool,
    pub status_message: Option<(String, Instant)>,
    pub scanning: bool,
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
                        process::macos::batch_resolve(&mut entries);
                        for entry in &mut entries {
                            entry.tech = detect::detect_tech(entry);
                        }
                        git::batch_detect(&mut entries);
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
            ports: Vec::new(),
            selected: 0,
            should_quit: false,
            show_help: false,
            confirm_kill: false,
            status_message: None,
            scanning: false,
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
                self.ports = entries;
                self.scanning = false;
                if self.selected >= self.ports.len() && !self.ports.is_empty() {
                    self.selected = self.ports.len() - 1;
                }
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
}
