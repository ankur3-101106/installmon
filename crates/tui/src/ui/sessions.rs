use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

use crate::app::App;

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(area);

    // Top: Sessions Table
    let header_cells = [
        "Session ID",
        "Started At",
        "Status",
        "Scope",
        "Classification",
        "Confidence",
        "Initiating Command",
    ]
    .iter()
    .map(|h| {
        Cell::from(*h).style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
    });
    let header = Row::new(header_cells).height(1).bottom_margin(1);

    let rows: Vec<Row> = app
        .sessions_list
        .iter()
        .enumerate()
        .map(|(idx, s)| {
            let is_selected = idx == app.selected_session;
            let row_style = if is_selected {
                Style::default()
                    .bg(Color::DarkGray)
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let status_color = match s.status.as_str() {
                "completed" => Color::Green,
                "active" => Color::Yellow,
                "failed" => Color::Red,
                _ => Color::DarkGray,
            };

            let cmd_str = s.initiating_command.as_deref().unwrap_or("-");

            let cells = vec![
                Cell::from(format!(
                    "{} {}",
                    if is_selected { "▶" } else { " " },
                    &s.id.to_string()[..8]
                )),
                Cell::from(s.started_at.format("%Y-%m-%d %H:%M:%S").to_string()),
                Cell::from(s.status.to_uppercase()).style(Style::default().fg(status_color)),
                Cell::from(s.scope.to_string()),
                Cell::from(s.classification.to_string()),
                Cell::from(s.confidence.to_string()),
                Cell::from(cmd_str),
            ];
            Row::new(cells).style(row_style).height(1)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(12),
            Constraint::Length(20),
            Constraint::Length(12),
            Constraint::Length(10),
            Constraint::Length(18),
            Constraint::Length(12),
            Constraint::Min(25),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .title(format!(
                " Installation Sessions ({}) ",
                app.sessions_list.len()
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );
    frame.render_widget(table, chunks[0]);

    // Bottom: Session Details Drawer
    if let Some(session) = app.sessions_list.get(app.selected_session) {
        let artifacts = app
            .db
            .get_artifacts_for_session(session.id)
            .unwrap_or_default();
        let evidence = app
            .db
            .get_evidence_for_session(session.id)
            .unwrap_or_default();

        let detail_cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
            .split(chunks[1]);

        let meta_lines = vec![
            Line::from(vec![
                Span::styled(
                    "Full Session ID    : ",
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(session.id.to_string(), Style::default().fg(Color::Cyan)),
            ]),
            Line::from(vec![
                Span::styled(
                    "Initiating Command : ",
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    session.initiating_command.as_deref().unwrap_or("<unknown>"),
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    "Classification     : ",
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    session.classification.to_string(),
                    Style::default().fg(Color::Yellow),
                ),
                Span::raw(" | Confidence: "),
                Span::styled(
                    session.confidence.to_string(),
                    Style::default().fg(Color::Green),
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    "Installer Identity : ",
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    session
                        .installer_identity
                        .as_deref()
                        .unwrap_or("<inferred>"),
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    "Artifacts Recorded : ",
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    format!("{} files", artifacts.len()),
                    Style::default().fg(Color::Cyan),
                ),
                Span::raw(" | Evidence: "),
                Span::styled(
                    format!("{} items", evidence.len()),
                    Style::default().fg(Color::Magenta),
                ),
            ]),
        ];

        let meta_p = Paragraph::new(meta_lines).block(
            Block::default()
                .title(" Session Overview ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
        frame.render_widget(meta_p, detail_cols[0]);

        // Evidence and artifacts preview
        let mut right_lines = Vec::new();
        right_lines.push(Line::from(Span::styled(
            "Preserved Supporting Evidence:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )));
        if evidence.is_empty() {
            right_lines.push(Line::from(Span::styled(
                "  No evidence items recorded for this session.",
                Style::default().fg(Color::DarkGray),
            )));
        } else {
            for ev in &evidence {
                right_lines.push(Line::from(vec![
                    Span::styled("  • ", Style::default().fg(Color::Magenta)),
                    Span::styled(
                        format!("[{}] ", ev.category),
                        Style::default().fg(Color::Cyan),
                    ),
                    Span::styled(&ev.description, Style::default().fg(Color::White)),
                ]));
            }
        }

        right_lines.push(Line::from(""));
        right_lines.push(Line::from(Span::styled(
            "Filesystem Artifacts:",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )));
        if artifacts.is_empty() {
            right_lines.push(Line::from(Span::styled(
                "  No filesystem artifacts recorded for this session.",
                Style::default().fg(Color::DarkGray),
            )));
        } else {
            for art in &artifacts {
                right_lines.push(Line::from(vec![
                    Span::styled("  • ", Style::default().fg(Color::Green)),
                    Span::styled(
                        art.path.display().to_string(),
                        Style::default().fg(Color::White),
                    ),
                    Span::styled(
                        format!(" ({})", art.file_type),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]));
            }
        }

        let right_p = Paragraph::new(right_lines).block(
            Block::default()
                .title(" Evidence & Artifacts ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
        frame.render_widget(right_p, detail_cols[1]);
    } else {
        let empty = Paragraph::new("No session selected or no sessions recorded yet.").block(
            Block::default()
                .title(" Session Details ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
        frame.render_widget(empty, chunks[1]);
    }
}
