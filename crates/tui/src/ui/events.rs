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
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    // Top: Events Table
    let header_cells = [
        "Timestamp",
        "Type",
        "PID",
        "EUID",
        "Executable",
        "Target Path",
        "Backend",
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
        .events_list
        .iter()
        .enumerate()
        .map(|(idx, ev)| {
            let is_selected = idx == app.selected_event;
            let row_style = if is_selected {
                Style::default()
                    .bg(Color::DarkGray)
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let type_color = match ev.event_type.to_string().as_str() {
                "process_exec" => Color::Cyan,
                "process_exit" => Color::DarkGray,
                "file_create" => Color::Green,
                "file_modify" => Color::Yellow,
                "file_delete" => Color::Red,
                "download_observed" => Color::Magenta,
                _ => Color::White,
            };

            let exe = ev
                .executable_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| ev.executable_path.to_string_lossy().to_string());
            let target = ev
                .target_path
                .as_ref()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| "-".to_string());

            let cells = vec![
                Cell::from(format!(
                    "{} {}",
                    if is_selected { "▶" } else { " " },
                    ev.timestamp.format("%H:%M:%S")
                )),
                Cell::from(ev.event_type.to_string()).style(Style::default().fg(type_color)),
                Cell::from(ev.pid.to_string()),
                Cell::from(ev.euid.to_string()),
                Cell::from(exe),
                Cell::from(target),
                Cell::from(ev.backend.to_string()),
            ];
            Row::new(cells).style(row_style).height(1)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(14),
            Constraint::Length(18),
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Length(22),
            Constraint::Min(25),
            Constraint::Length(12),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .title(format!(
                " Observed Lifecycle Events ({}) ",
                app.events_list.len()
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Magenta)),
    );
    frame.render_widget(table, chunks[0]);

    // Bottom: Event Detail View
    if let Some(ev) = app.events_list.get(app.selected_event) {
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[1]);

        let info_lines = vec![
            Line::from(vec![
                Span::styled("Event ID      : ", Style::default().fg(Color::DarkGray)),
                Span::styled(ev.id.to_string(), Style::default().fg(Color::Cyan)),
            ]),
            Line::from(vec![
                Span::styled("Full Path     : ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    ev.executable_path.display().to_string(),
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::from(vec![
                Span::styled("Working Dir   : ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    ev.working_directory
                        .as_ref()
                        .map(|p| p.display().to_string())
                        .unwrap_or_else(|| "none".to_string()),
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::from(vec![
                Span::styled("Confidence    : ", Style::default().fg(Color::DarkGray)),
                Span::styled(ev.confidence.to_string(), Style::default().fg(Color::Green)),
                Span::raw(" | Observation: "),
                Span::styled(
                    ev.observation_kind.to_string(),
                    Style::default().fg(Color::Yellow),
                ),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "Sanitized Command Arguments:",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                if ev.arguments.is_empty() {
                    "<none>".to_string()
                } else {
                    format!("$ {}", ev.arguments.join(" "))
                },
                Style::default().fg(Color::Cyan),
            )),
        ];

        let info_p = Paragraph::new(info_lines).block(
            Block::default()
                .title(" Event Details ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
        frame.render_widget(info_p, cols[0]);

        // Metadata
        let meta_str = serde_json::to_string_pretty(&ev.metadata).unwrap_or_default();
        let meta_p = Paragraph::new(meta_str).block(
            Block::default()
                .title(" Event Metadata ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
        frame.render_widget(meta_p, cols[1]);
    } else {
        let empty = Paragraph::new("No event selected or no events observed yet.").block(
            Block::default()
                .title(" Event Details ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
        frame.render_widget(empty, chunks[1]);
    }
}
