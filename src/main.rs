mod actions;
mod app;
mod config;
mod detect;
mod git;
mod process;
mod scanner;
mod types;
mod ui;

use app::App;
use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use std::time::{Duration, Instant};

#[derive(Parser)]
#[command(
    name = "portwatch",
    about = "A terminal UI for monitoring and managing local web server ports",
    version
)]
struct Cli {
    /// Refresh interval in seconds
    #[arg(short, long, default_value_t = 3)]
    interval: u64,
}

/// Check whether --interval was explicitly passed on the command line.
fn cli_interval_was_provided() -> bool {
    std::env::args().any(|a| a == "--interval" || a == "-i" || a.starts_with("--interval="))
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let cfg = config::load().unwrap_or_default();

    // CLI --interval overrides config file when explicitly provided
    let refresh_secs = if cli_interval_was_provided() {
        cli.interval
    } else {
        cfg.refresh_interval
    };
    let terminal_setting = cfg.terminal.clone();

    // Set up panic hook to restore terminal on crash
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        original_hook(panic_info);
    }));

    // Initialize terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run the app
    let result = run_app(&mut terminal, refresh_secs, &terminal_setting);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    refresh_secs: u64,
    terminal_setting: &str,
) -> Result<()> {
    let mut app = App::new();
    let refresh_interval = Duration::from_secs(refresh_secs);
    let mut last_refresh = Instant::now();
    let mut needs_redraw = true;

    loop {
        // Check for scan results (non-blocking)
        if app.poll_results() {
            needs_redraw = true;
        }
        if app.clear_stale_status() {
            needs_redraw = true;
        }

        // Only redraw when state changed
        if needs_redraw {
            terminal.draw(|f| ui::draw(f, &app))?;
            needs_redraw = false;
        }

        // Wait for input — sleep up to 200ms so we stay idle when nothing happens,
        // but wake up fast enough to pick up scan results and status clears
        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                needs_redraw = true;
                if app.filter_active {
                    handle_filter_key(&mut app, key.code);
                } else if app.confirm_kill {
                    handle_kill_confirm(&mut app, key.code);
                } else if app.show_help {
                    match key.code {
                        KeyCode::Char('?') | KeyCode::Esc | KeyCode::Char('q') => {
                            app.show_help = false;
                        }
                        _ => {}
                    }
                } else {
                    handle_key(&mut app, key.code, key.modifiers, terminal_setting);
                }
            }
        }

        // Trigger periodic background refresh
        if last_refresh.elapsed() >= refresh_interval {
            app.request_refresh();
            last_refresh = Instant::now();
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

fn handle_filter_key(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Enter | KeyCode::Esc => app.close_filter(),
        KeyCode::Backspace => app.delete_filter_char(),
        KeyCode::Char(c) => app.update_filter(c),
        _ => {}
    }
}

fn handle_key(app: &mut App, code: KeyCode, modifiers: KeyModifiers, terminal_setting: &str) {
    match code {
        KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
        KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => app.should_quit = true,
        KeyCode::Char('j') | KeyCode::Down => app.select_next(),
        KeyCode::Char('k') | KeyCode::Up => app.select_prev(),
        KeyCode::Char('g') => app.select_first(),
        KeyCode::Char('G') => app.select_last(),
        KeyCode::Char('?') => app.show_help = true,
        KeyCode::Char('/') => app.toggle_filter(),
        KeyCode::Char('s') => app.cycle_sort(),
        KeyCode::Char('S') => app.toggle_sort_direction(),
        KeyCode::Char('r') => {
            app.request_refresh();
            app.set_status("Refreshing...".to_string());
        }
        KeyCode::Char('x') => {
            if app.selected_entry().is_some() {
                app.confirm_kill = true;
            }
        }
        KeyCode::Char('b') => {
            if let Some(entry) = app.selected_entry() {
                let entry = entry.clone();
                match actions::open_in_browser(&entry) {
                    Ok(()) => app.set_status(format!("Opened localhost:{} in browser", entry.port)),
                    Err(e) => app.set_status(format!("Error: {e}")),
                }
            }
        }
        KeyCode::Char('f') => {
            if let Some(entry) = app.selected_entry() {
                let entry = entry.clone();
                match actions::open_folder(&entry, terminal_setting) {
                    Ok(()) => app.set_status("Opened folder".to_string()),
                    Err(e) => app.set_status(format!("Error: {e}")),
                }
            }
        }
        KeyCode::Char('c') => {
            if let Some(entry) = app.selected_entry() {
                let entry = entry.clone();
                match actions::copy_url_to_clipboard(&entry) {
                    Ok(()) => app.set_status(format!("Copied http://localhost:{}", entry.port)),
                    Err(e) => app.set_status(format!("Error: {e}")),
                }
            }
        }
        _ => {}
    }
}

fn handle_kill_confirm(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            if let Some(entry) = app.selected_entry() {
                let entry = entry.clone();
                match actions::kill_process(&entry) {
                    Ok(()) => {
                        app.set_status(format!(
                            "Killed {} (PID {}) on port {}",
                            entry.process_name, entry.pid, entry.port
                        ));
                        app.request_refresh();
                    }
                    Err(e) => app.set_status(format!("Kill failed: {e}")),
                }
            }
            app.confirm_kill = false;
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            app.confirm_kill = false;
        }
        _ => {}
    }
}
