use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub fn render(frame: &mut Frame) {
    let area = centered_rect(60, 50, frame.area());
    frame.render_widget(Clear, area);

    let help_text = vec![
        Line::from(vec![Span::styled(
            "InstallMon TUI Keyboard Shortcuts",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "  Tab / 1..6       ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Switch tabs (Dashboard, Software, Sessions, Events, Audit, Service)"),
        ]),
        Line::from(vec![
            Span::styled(
                "  I / i            ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Setup & start background systemd/user monitoring service"),
        ]),
        Line::from(vec![
            Span::styled(
                "  S                ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Toggle background service (Start / Stop daemon)"),
        ]),
        Line::from(vec![
            Span::styled(
                "  R                ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Restart background service daemon"),
        ]),
        Line::from(vec![
            Span::styled(
                "  e                ",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Toggle service auto-start on system boot / login"),
        ]),
        Line::from(vec![
            Span::styled(
                "  b                ",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Launch detached background daemon (standalone fallback)"),
        ]),
        Line::from(vec![
            Span::styled(
                "  s                ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Scan system PATH and discover new executable binaries"),
        ]),
        Line::from(vec![
            Span::styled(
                "  r                ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Refresh database state and active capabilities"),
        ]),
        Line::from(vec![
            Span::styled(
                "  ↑ / ↓ / j / k    ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Scroll / navigate items in tables"),
        ]),
        Line::from(vec![
            Span::styled(
                "  ?                ",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Toggle this Help modal"),
        ]),
        Line::from(vec![
            Span::styled(
                "  Esc / Enter / q  ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Close Help modal"),
        ]),
        Line::from(vec![
            Span::styled(
                "  Ctrl+C / q       ",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::raw("Quit InstallMon TUI"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Press Esc or Enter to close.",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let block = Paragraph::new(help_text).alignment(Alignment::Left).block(
        Block::default()
            .title(" Help & Keybindings ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow)),
    );
    frame.render_widget(block, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
