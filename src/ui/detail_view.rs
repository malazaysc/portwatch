use crate::app::App;
use crate::types::{BindAddress, format_uptime};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Details ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue));

    let content = if let Some(entry) = app.selected_entry() {
        let mut lines = Vec::new();

        // Title line
        lines.push(Line::from(vec![
            Span::styled(
                format!(":{}", entry.port),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" — "),
            Span::styled(&entry.process_name, Style::default().fg(Color::Cyan)),
            Span::raw(format!(" (PID {})", entry.pid)),
        ]));

        // Bind address
        let (bind_label, bind_color) = match &entry.bind_address {
            BindAddress::Local => ("127.0.0.1 (local only)", Color::Green),
            BindAddress::Exposed => ("0.0.0.0 (exposed to network!)", Color::Red),
            BindAddress::Specific(ip) => (ip.as_str(), Color::Yellow),
        };
        lines.push(Line::from(vec![
            Span::styled("  Bind:  ", Style::default().fg(Color::DarkGray)),
            Span::styled(bind_label, Style::default().fg(bind_color)),
        ]));

        // Technology
        if let Some(tech) = &entry.tech {
            lines.push(Line::from(vec![
                Span::styled("  Tech:  ", Style::default().fg(Color::DarkGray)),
                Span::styled(&tech.name, Style::default().fg(Color::White)),
                Span::styled(
                    format!(" (via {})", tech.source),
                    Style::default().fg(Color::DarkGray),
                ),
            ]));
        }

        // Docker container info
        if let Some(docker) = &entry.docker_info {
            if let Some(project) = &docker.project {
                lines.push(Line::from(vec![
                    Span::styled("  Project:   ", Style::default().fg(Color::DarkGray)),
                    Span::styled(project, Style::default().fg(Color::Magenta)),
                ]));
            }
            lines.push(Line::from(vec![
                Span::styled("  Container: ", Style::default().fg(Color::DarkGray)),
                Span::styled(&docker.container_name, Style::default().fg(Color::Cyan)),
            ]));
            lines.push(Line::from(vec![
                Span::styled("  Image:     ", Style::default().fg(Color::DarkGray)),
                Span::styled(&docker.image, Style::default().fg(Color::White)),
            ]));
        }

        // Working directory
        if let Some(dir) = &entry.working_dir {
            let dir_str = shorten_path(dir);
            let mut dir_parts = vec![
                Span::styled("  Dir:   ", Style::default().fg(Color::DarkGray)),
                Span::styled(dir_str, Style::default().fg(Color::White)),
            ];

            if let Some(git) = &entry.git_info {
                dir_parts.push(Span::styled(
                    format!(
                        " ({}{}) ",
                        if git.is_worktree { "worktree: " } else { "" },
                        git.branch
                    ),
                    Style::default().fg(Color::Magenta),
                ));
            }

            lines.push(Line::from(dir_parts));
        }

        // Command line
        if !entry.command_line.is_empty() {
            let max_len = (area.width as usize).saturating_sub(15);
            let cmd = if max_len > 3 && entry.command_line.len() > max_len {
                // Truncate at a char boundary
                let truncated: String = entry.command_line.chars().take(max_len - 3).collect();
                format!("{truncated}...")
            } else {
                entry.command_line.clone()
            };
            lines.push(Line::from(vec![
                Span::styled("  Cmd:   ", Style::default().fg(Color::DarkGray)),
                Span::styled(cmd, Style::default().fg(Color::DarkGray)),
            ]));
        }

        // User
        if !entry.is_own {
            lines.push(Line::from(vec![
                Span::styled("  User:  ", Style::default().fg(Color::DarkGray)),
                Span::styled(&entry.user, Style::default().fg(Color::Yellow)),
                Span::styled(
                    " (not yours — actions may fail)",
                    Style::default().fg(Color::DarkGray),
                ),
            ]));
        }

        // Uptime
        if let Some(uptime) = &entry.uptime {
            lines.push(Line::from(vec![
                Span::styled("  Up:    ", Style::default().fg(Color::DarkGray)),
                Span::styled(format_uptime(uptime), Style::default().fg(Color::White)),
            ]));
        }

        // CPU usage
        if let Some(cpu) = entry.cpu_usage {
            lines.push(Line::from(vec![
                Span::styled("  CPU:   ", Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{:.1}%", cpu), Style::default().fg(Color::White)),
            ]));
        }

        // Memory usage
        if let Some(mem_mb) = entry.memory_mb {
            let mem_str = if mem_mb >= 1024.0 {
                format!("{:.1} GB", mem_mb / 1024.0)
            } else {
                format!("{:.1} MB", mem_mb)
            };
            lines.push(Line::from(vec![
                Span::styled("  Mem:   ", Style::default().fg(Color::DarkGray)),
                Span::styled(mem_str, Style::default().fg(Color::White)),
            ]));
        }

        lines
    } else {
        vec![Line::from(Span::styled(
            "  No port selected",
            Style::default().fg(Color::DarkGray),
        ))]
    };

    let paragraph = Paragraph::new(content).block(block);
    frame.render_widget(paragraph, area);
}

fn shorten_path(path: &std::path::Path) -> String {
    if let Some(home) = dirs::home_dir()
        && let Ok(stripped) = path.strip_prefix(&home)
    {
        return format!("~/{}", stripped.display());
    }
    path.display().to_string()
}
