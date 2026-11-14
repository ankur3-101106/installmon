use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let left_rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(cols[0]);

    let right_rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(cols[1]);

    // 1. Host & Platform Info
    let caps = &app.capabilities;
    let platform_lines = vec![
        Line::from(vec![
            Span::styled(
                "Operating System   : ",
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                format!("{} ({})", caps.distribution_name, caps.distribution_id),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "Kernel Release     : ",
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                format!("{} [{}]", caps.kernel_release, caps.architecture),
                Style::default().fg(Color::Cyan),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "Host Environment   : ",
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                caps.environment.to_string(),
                Style::default().fg(Color::Yellow),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "Init System        : ",
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                caps.init_system.to_string(),
                Style::default().fg(Color::Magenta),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "Standard C Library : ",
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(caps.libc.to_string(), Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled(
                "Privilege Scope    : ",
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                if caps.is_root {
                    "System Root (CAP_SYS_ADMIN)"
                } else {
                    "Unprivileged User"
                },
                Style::default().fg(if caps.is_root {
                    Color::Red
                } else {
                    Color::Green
                }),
            ),
        ]),
    ];

    let platform_block = Paragraph::new(platform_lines).block(
        Block::default()
            .title(" Host & Platform Environment ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue)),
    );
    frame.render_widget(platform_block, left_rows[0]);

    // 2. Active Monitoring Backends
    let active_backend_lines = vec![
        Line::from(vec![
            Span::styled(
                "Process Monitoring Backend : ",
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                format!("[ {} ]", caps.active_process_backend.to_uppercase()),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "Filesystem Backend         : ",
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                format!("[ {} ]", caps.active_filesystem_backend.to_uppercase()),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Strategy: ", Style::default().fg(Color::DarkGray)),
            Span::raw("Continuous event-driven capture with dynamic graceful degradation."),
        ]),
    ];

    let backend_block = Paragraph::new(active_backend_lines).block(
        Block::default()
            .title(" Active Monitoring Negotiated ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );
    frame.render_widget(backend_block, left_rows[1]);

    // 3. Database & Observability Counters
    let db_path = app
        .db
        .path()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "in-memory".to_string());
    let counter_lines = vec![
        Line::from(vec![
            Span::styled(
                "Tracked Software   : ",
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                format!("{} packages", app.software_list.len()),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "Tracked Sessions   : ",
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                format!("{} sessions", app.sessions_list.len()),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "Lifecycle Events   : ",
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                format!("{} observed", app.events_list.len()),
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "Database Path      : ",
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(db_path, Style::default().fg(Color::White)),
        ]),
    ];

    let counter_block = Paragraph::new(counter_lines).block(
        Block::default()
            .title(" Observability Statistics ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green)),
    );
    frame.render_widget(counter_block, right_rows[0]);

    // 4. Subsystems Capability Matrix
    let mut matrix_lines = Vec::new();
    for b in &caps.backends {
        let (status_color, status_icon) = match b.status.to_string().as_str() {
            "available" => (Color::Green, "✓"),
            "requires_privilege" => (Color::Yellow, "!"),
            "partially_available" => (Color::Cyan, "~"),
            _ => (Color::Red, "✗"),
        };

        matrix_lines.push(Line::from(vec![
            Span::styled(
                format!(" {} ", status_icon),
                Style::default()
                    .fg(status_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{:<10}", b.name),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("[{:<18}] ", b.status),
                Style::default().fg(status_color),
            ),
            Span::styled(&b.description, Style::default().fg(Color::DarkGray)),
        ]));
    }

    let matrix_block = Paragraph::new(matrix_lines).block(
        Block::default()
            .title(" Kernel & Subsystem Capabilities Matrix ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow)),
    );
    frame.render_widget(matrix_block, right_rows[1]);
}
