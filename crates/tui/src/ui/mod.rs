pub mod audit;
pub mod dashboard;
pub mod events;
pub mod help;
pub mod sessions;
pub mod software;

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame,
};

use crate::app::{App, CurrentTab};

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header & Tabs
            Constraint::Min(10),   // Content
            Constraint::Length(2), // Footer Status & Keys
        ])
        .split(frame.area());

    render_header(frame, app, chunks[0]);

    match app.current_tab {
        CurrentTab::Dashboard => dashboard::render(frame, app, chunks[1]),
        CurrentTab::Software => software::render(frame, app, chunks[1]),
        CurrentTab::Sessions => sessions::render(frame, app, chunks[1]),
        CurrentTab::Events => events::render(frame, app, chunks[1]),
        CurrentTab::Audit => audit::render(frame, app, chunks[1]),
    }

    render_footer(frame, app, chunks[2]);

    if app.showing_help {
        help::render(frame);
    }
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(22), // Title
            Constraint::Min(40),    // Tabs
            Constraint::Length(26), // Mode & Time
        ])
        .split(area);

    // Title
    let title_p = Paragraph::new(Line::from(vec![
        Span::styled(
            "⚡ InstallMon ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("v0.1.0", Style::default().fg(Color::DarkGray)),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    frame.render_widget(title_p, chunks[0]);

    // Tabs
    let titles: Vec<Line> = CurrentTab::titles()
        .iter()
        .map(|t| Line::from(Span::raw(*t)))
        .collect();

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .select(app.current_tab as usize)
        .style(Style::default().fg(Color::Gray))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
                .bg(Color::DarkGray),
        );
    frame.render_widget(tabs, chunks[1]);

    // Mode
    let mode_str = if app.capabilities.is_root {
        "SYSTEM (ROOT)"
    } else {
        "USER (UNPRIV)"
    };
    let mode_color = if app.capabilities.is_root {
        Color::Red
    } else {
        Color::Green
    };
    let time_str = chrono::Utc::now().format("%H:%M:%S UTC").to_string();

    let mode_p = Paragraph::new(Line::from(vec![
        Span::styled(
            mode_str,
            Style::default().fg(mode_color).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled(time_str, Style::default().fg(Color::White)),
    ]))
    .alignment(Alignment::Right)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    frame.render_widget(mode_p, chunks[2]);
}

fn render_footer(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(60), // Keybindings
            Constraint::Percentage(40), // Toast notification
        ])
        .split(area);

    let keys = Line::from(vec![
        Span::styled(
            "[1..5/Tab]",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Switch  "),
        Span::styled(
            "[↑/↓/j/k]",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Navigate  "),
        Span::styled(
            "[s]",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Scan  "),
        Span::styled(
            "[r]",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Refresh  "),
        Span::styled(
            "[?]",
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Help  "),
        Span::styled(
            "[q]",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Quit"),
    ]);
    frame.render_widget(Paragraph::new(keys), chunks[0]);

    if let Some((ref msg, _)) = app.status_message {
        let msg_p = Paragraph::new(Line::from(vec![
            Span::styled("● ", Style::default().fg(Color::Green)),
            Span::styled(msg, Style::default().fg(Color::White)),
        ]))
        .alignment(Alignment::Right);
        frame.render_widget(msg_p, chunks[1]);
    }
}
