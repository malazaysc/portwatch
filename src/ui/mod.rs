mod action_bar;
mod detail_view;
mod port_list;
mod status_bar;

use crate::app::App;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};

pub fn draw(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(8),     // port list
            Constraint::Length(1),  // status bar
            Constraint::Length(10), // detail view
            Constraint::Length(3),  // action bar
        ])
        .split(frame.area());

    port_list::draw(frame, app, chunks[0]);
    status_bar::draw(frame, app, chunks[1]);
    detail_view::draw(frame, app, chunks[2]);
    action_bar::draw(frame, app, chunks[3]);

    if app.show_help {
        draw_help_popup(frame);
    }

    if app.confirm_kill {
        draw_kill_confirm(frame, app);
    }
}

fn draw_help_popup(frame: &mut Frame) {
    use ratatui::style::{Color, Style};
    use ratatui::text::Line;
    use ratatui::widgets::{Block, Borders, Clear, Paragraph};

    let area = centered_rect(50, 60, frame.area());
    frame.render_widget(Clear, area);

    let help_text = vec![
        Line::from("Navigation"),
        Line::from("  \u{2191}/\u{2193}          Move up/down"),
        Line::from("  \u{2190}            Collapse group"),
        Line::from("  \u{2192}            Expand group"),
        Line::from("  Home/End     Go to first/last"),
        Line::from(""),
        Line::from("Actions"),
        Line::from("  x            Kill process"),
        Line::from("  b            Open in browser"),
        Line::from("  c            Copy URL"),
        Line::from("  d            Copy directory path"),
        Line::from("  r            Refresh"),
        Line::from(""),
        Line::from("Filter & Sort"),
        Line::from("  /            Search/filter ports"),
        Line::from("  s            Cycle sort column"),
        Line::from("  S            Toggle sort direction"),
        Line::from(""),
        Line::from("General"),
        Line::from("  ?            Toggle help"),
        Line::from("  q / Esc      Quit"),
    ];

    let paragraph = Paragraph::new(help_text)
        .block(
            Block::default()
                .title(" Help ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .style(Style::default().fg(Color::White));

    frame.render_widget(paragraph, area);
}

fn draw_kill_confirm(frame: &mut Frame, app: &App) {
    use ratatui::style::{Color, Style};
    use ratatui::text::Line;
    use ratatui::widgets::{Block, Borders, Clear, Paragraph};

    let area = centered_rect(40, 20, frame.area());
    frame.render_widget(Clear, area);

    let entry = app.selected_entry();
    let msg = if let Some(e) = entry {
        format!(
            "Kill {} (PID {}) on port {}?",
            e.process_name, e.pid, e.port
        )
    } else {
        "No process selected".to_string()
    };

    let text = vec![
        Line::from(""),
        Line::from(msg),
        Line::from(""),
        Line::from("  [y] Yes    [n/Esc] Cancel"),
    ];

    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .title(" Confirm Kill ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Red)),
        )
        .style(Style::default().fg(Color::White));

    frame.render_widget(paragraph, area);
}

fn centered_rect(
    percent_x: u16,
    percent_y: u16,
    area: ratatui::layout::Rect,
) -> ratatui::layout::Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
