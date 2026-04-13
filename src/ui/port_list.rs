use crate::app::{App, DisplayRow, SortColumn};
use crate::types::{BindAddress, format_uptime};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Row, Table, TableState};

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let is_extra_wide = area.width >= 120;
    let is_wide = area.width >= 100;
    let is_medium = area.width >= 70;

    let header_cells = build_header(app, is_extra_wide, is_wide, is_medium);
    let header = Row::new(header_cells)
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .height(1);

    let num_cols = {
        let mut n = 3;
        if is_medium { n += 2; }
        if is_extra_wide { n += 2; }
        if is_wide { n += 1; }
        n
    };

    let mut rows: Vec<Row> = Vec::new();

    for (i, display_row) in app.display_rows.iter().enumerate() {
        match display_row {
            DisplayRow::GroupHeader { name, count, collapsed } => {
                let arrow = if *collapsed { "\u{25b6}" } else { "\u{25bc}" };
                let header_text = format!(
                    "{arrow} {name} ({count} {})",
                    if *count == 1 { "port" } else { "ports" }
                );

                let style = if i == app.selected {
                    Style::default()
                        .fg(Color::Cyan)
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                };

                let mut cells: Vec<ratatui::text::Text> = vec![
                    ratatui::text::Text::styled(header_text, style),
                ];
                for _ in 1..num_cols {
                    cells.push(ratatui::text::Text::raw(""));
                }
                rows.push(Row::new(cells).style(style));
            }
            DisplayRow::Port(idx) => {
                let entry = &app.ports[*idx];
                let style = if !entry.is_own {
                    Style::default().fg(Color::DarkGray)
                } else {
                    Style::default()
                };

                let exposure_indicator = match &entry.bind_address {
                    BindAddress::Local => Span::styled("\u{25cf}", Style::default().fg(Color::Green)),
                    BindAddress::Exposed => Span::styled("\u{25cf}", Style::default().fg(Color::Red)),
                    BindAddress::Specific(_) => Span::styled("\u{25cf}", Style::default().fg(Color::Yellow)),
                };

                let port_cell = Line::from(vec![
                    exposure_indicator,
                    Span::raw(" "),
                    Span::raw(entry.port.to_string()),
                ]);

                let tech = entry
                    .tech
                    .as_ref()
                    .map(|t| t.name.as_str())
                    .unwrap_or("\u{2014}");

                let tech_style = tech_color(tech);

                let dir_display = entry
                    .working_dir
                    .as_ref()
                    .map(|d| shorten_path(d))
                    .unwrap_or_else(|| "\u{2014}".to_string());

                let uptime = entry
                    .uptime
                    .as_ref()
                    .map(|u| format_uptime(u))
                    .unwrap_or_else(|| "\u{2014}".to_string());

                let mut cells = vec![
                    ratatui::text::Text::from(port_cell),
                    ratatui::text::Text::styled(entry.process_name.clone(), Style::default()),
                    ratatui::text::Text::styled(tech.to_string(), tech_style),
                ];

                if is_medium {
                    cells.push(ratatui::text::Text::raw(dir_display));
                    cells.push(ratatui::text::Text::styled(
                        uptime,
                        Style::default().fg(Color::DarkGray),
                    ));
                }

                if is_extra_wide {
                    let cpu_str = entry
                        .cpu_usage
                        .map(|c| format!("{:.1}%", c))
                        .unwrap_or_else(|| "\u{2014}".to_string());
                    cells.push(ratatui::text::Text::styled(
                        cpu_str,
                        Style::default().fg(Color::DarkGray),
                    ));

                    let mem_str = match entry.memory_mb {
                        Some(mb) if mb >= 1024.0 => format!("{:.1}G", mb / 1024.0),
                        Some(mb) => format!("{:.1}M", mb),
                        None => "\u{2014}".to_string(),
                    };
                    cells.push(ratatui::text::Text::styled(
                        mem_str,
                        Style::default().fg(Color::DarkGray),
                    ));
                }

                if is_wide {
                    cells.push(ratatui::text::Text::raw(entry.pid.to_string()));
                }

                rows.push(Row::new(cells).style(style));
            }
        }
    }

    let title = build_title(app);
    let widths = build_widths(is_extra_wide, is_wide, is_medium, area.width);

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue)),
        )
        .row_highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

    let mut state = TableState::default();
    state.select(Some(app.selected));
    frame.render_stateful_widget(table, area, &mut state);
}

fn build_title(app: &App) -> String {
    if app.filter_active || !app.filter_text.is_empty() {
        format!(
            " Ports ({}/{}) filter: {} ",
            app.ports.len(),
            app.all_ports.len(),
            app.filter_text
        )
    } else {
        format!(" Ports ({}) ", app.ports.len())
    }
}

fn build_header(app: &App, is_extra_wide: bool, is_wide: bool, is_medium: bool) -> Vec<String> {
    let arrow = if app.sort_ascending {
        " \u{25b2}"
    } else {
        " \u{25bc}"
    };

    let decorate = |col: SortColumn, label: &str| -> String {
        if app.sort_column == col {
            format!("{label}{arrow}")
        } else {
            label.to_string()
        }
    };

    let mut h = vec![
        decorate(SortColumn::Port, "PORT"),
        decorate(SortColumn::Process, "PROCESS"),
        decorate(SortColumn::Tech, "TECH"),
    ];
    if is_medium {
        h.push("DIRECTORY".to_string());
        h.push(decorate(SortColumn::Uptime, "UPTIME"));
    }
    if is_extra_wide {
        h.push(decorate(SortColumn::Cpu, "CPU%"));
        h.push(decorate(SortColumn::Memory, "MEM"));
    }
    if is_wide {
        h.push("PID".to_string());
    }
    h
}

fn build_widths(is_extra_wide: bool, is_wide: bool, is_medium: bool, _total: u16) -> Vec<ratatui::layout::Constraint> {
    use ratatui::layout::Constraint;
    let mut w = vec![
        Constraint::Length(10),
        Constraint::Length(14),
        Constraint::Length(16),
    ];
    if is_medium {
        w.push(Constraint::Min(20));
        w.push(Constraint::Length(10));
    }
    if is_extra_wide {
        w.push(Constraint::Length(8));
        w.push(Constraint::Length(8));
    }
    if is_wide {
        w.push(Constraint::Length(8));
    }
    w
}

fn tech_color(tech: &str) -> Style {
    let color = match tech {
        t if t.contains("Next") || t.contains("React") => Color::Cyan,
        t if t.contains("Vite") || t.contains("Vue") || t.contains("Nuxt") => Color::Green,
        t if t.contains("Angular") => Color::Red,
        t if t.contains("Svelte") => Color::LightRed,
        t if t.contains("Node") || t.contains("Express") || t.contains("Fastify") => Color::LightGreen,
        t if t.contains("Python") || t.contains("Django") || t.contains("Flask") || t.contains("FastAPI") => Color::Yellow,
        t if t.contains("Rust") || t.contains("Axum") || t.contains("Actix") => Color::LightRed,
        t if t.contains("Go") || t.contains("Gin") => Color::Cyan,
        t if t.contains("Ruby") || t.contains("Rails") => Color::Red,
        t if t.contains("Java") || t.contains("Spring") => Color::LightYellow,
        t if t.contains("PHP") || t.contains("Laravel") => Color::Magenta,
        t if t.contains("Deno") => Color::LightCyan,
        t if t.contains("Bun") => Color::LightYellow,
        t if t.contains("PostgreSQL") || t.contains("MySQL") || t.contains("Redis") || t.contains("MongoDB") => Color::Magenta,
        "\u{2014}" => Color::DarkGray,
        _ => Color::White,
    };
    Style::default().fg(color)
}

fn shorten_path(path: &std::path::Path) -> String {
    if let Some(home) = dirs::home_dir() {
        if let Ok(stripped) = path.strip_prefix(&home) {
            return format!("~/{}", stripped.display());
        }
    }
    path.display().to_string()
}
