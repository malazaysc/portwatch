use crate::app::App;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue));

    let mut spans = vec![Span::raw("  ")];

    if let Some((msg, _)) = &app.status_message {
        spans.push(Span::styled(msg, Style::default().fg(Color::Yellow)));
    } else {
        spans.extend([
            key_hint("x", "kill"),
            Span::raw("  "),
            key_hint("b", "browser"),
            Span::raw("  "),
            key_hint("f", "folder"),
            Span::raw("  "),
            key_hint("c", "copy url"),
            Span::raw("  "),
            key_hint("r", "refresh"),
            Span::raw("  "),
            key_hint("?", "help"),
            Span::raw("  "),
            key_hint("q", "quit"),
        ]);
    }

    if app.scanning {
        spans.push(Span::raw("  "));
        spans.push(Span::styled("scanning...", Style::default().fg(Color::DarkGray)));
    }

    let content = Line::from(spans);

    let paragraph = Paragraph::new(content).block(block);
    frame.render_widget(paragraph, area);
}

fn key_hint<'a>(key: &'a str, label: &'a str) -> Span<'a> {
    // We return just the combined text as a single span for simplicity
    // A more complex version could use two spans with different styles
    Span::styled(
        format!("[{key}] {label}"),
        Style::default().fg(Color::DarkGray),
    )
}
