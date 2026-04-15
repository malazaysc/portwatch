use crate::app::App;
use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let port_count = app.ports.len();

    let total_cpu: f32 = app.ports.iter().filter_map(|e| e.cpu_usage).sum();

    let total_mem_mb: f64 = app.ports.iter().filter_map(|e| e.memory_mb).sum();
    let mem_str = if total_mem_mb >= 1024.0 {
        format!("{:.1} GB", total_mem_mb / 1024.0)
    } else {
        format!("{:.0} MB", total_mem_mb)
    };

    let net = &app.network_stats;
    let rx_str = format_bytes_rate(net.rx_bytes_per_sec);
    let tx_str = format_bytes_rate(net.tx_bytes_per_sec);

    let dim = Style::default().fg(Color::DarkGray);
    let val = Style::default().fg(Color::White);

    let line = Line::from(vec![
        Span::styled(" Ports: ", dim),
        Span::styled(port_count.to_string(), val),
        Span::styled("  \u{2502}  CPU: ", dim),
        Span::styled(format!("{:.1}%", total_cpu), val),
        Span::styled("  \u{2502}  MEM: ", dim),
        Span::styled(mem_str, val),
        Span::styled("  \u{2502}  NET: ", dim),
        Span::styled(
            format!("\u{2193}{rx_str}"),
            Style::default().fg(Color::Green),
        ),
        Span::styled(" ", dim),
        Span::styled(
            format!("\u{2191}{tx_str}"),
            Style::default().fg(Color::Cyan),
        ),
    ]);

    let paragraph = Paragraph::new(line).alignment(Alignment::Left);
    frame.render_widget(paragraph, area);
}

fn format_bytes_rate(bytes_per_sec: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * 1024;
    const GB: u64 = 1024 * 1024 * 1024;

    if bytes_per_sec >= GB {
        format!("{:.1} GB/s", bytes_per_sec as f64 / GB as f64)
    } else if bytes_per_sec >= MB {
        format!("{:.1} MB/s", bytes_per_sec as f64 / MB as f64)
    } else if bytes_per_sec >= KB {
        format!("{:.1} KB/s", bytes_per_sec as f64 / KB as f64)
    } else {
        format!("{bytes_per_sec} B/s")
    }
}
