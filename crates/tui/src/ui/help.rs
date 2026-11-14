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
                "  Tab / 1..5       ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(
                "Switch between navigation views (Dashboard, Software, Sessions, Events, Audit)",
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "  ↑ / ↓ / j / k    ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Scroll / navigate items in tables"),
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
                "  ? / h            ",
                Style::default()
                    .fg(Color::Yellow)
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
