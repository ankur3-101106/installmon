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
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(area);

    // Left: Software Table
    let header_cells = ["Name", "Scope", "Discovered At"].iter().map(|h| {
        Cell::from(*h).style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
    });
    let header = Row::new(header_cells).height(1).bottom_margin(1);

    let rows: Vec<Row> = app
        .software_list
        .iter()
        .enumerate()
        .map(|(idx, sw)| {
            let is_selected = idx == app.selected_software;
            let row_style = if is_selected {
                Style::default()
                    .bg(Color::DarkGray)
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let scope_color = match sw.scope.to_string().as_str() {
                "system" => Color::Red,
                "hybrid" => Color::Yellow,
                _ => Color::Green,
            };

            let cells = vec![
                Cell::from(format!(
                    "{} {}",
                    if is_selected { "▶" } else { " " },
                    sw.name
                )),
                Cell::from(sw.scope.to_string()).style(Style::default().fg(scope_color)),
                Cell::from(sw.discovered_at.format("%Y-%m-%d %H:%M").to_string()),
            ];
            Row::new(cells).style(row_style).height(1)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(45),
            Constraint::Percentage(20),
            Constraint::Percentage(35),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .title(format!(
                " Tracked Software ({}) - Press 's' to Scan PATH ",
                app.software_list.len()
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue)),
    );
    frame.render_widget(table, chunks[0]);

    // Right: Software Details Panel
    if let Some(sw) = app.software_list.get(app.selected_software) {
        let bin_exists = sw
            .primary_binary_path
            .as_ref()
            .map(|p| p.exists())
            .unwrap_or(false);

        let bin_status_color = if bin_exists { Color::Green } else { Color::Red };
        let bin_status_text = if bin_exists {
            "EXISTS ON DISK"
        } else {
            "MISSING ON DISK"
        };

        let mut detail_lines = vec![
            Line::from(vec![
                Span::styled("Software Name  : ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    &sw.name,
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("Software ID    : ", Style::default().fg(Color::DarkGray)),
                Span::styled(sw.id.to_string(), Style::default().fg(Color::Cyan)),
            ]),
            Line::from(vec![
                Span::styled("Privilege Scope: ", Style::default().fg(Color::DarkGray)),
                Span::styled(sw.scope.to_string(), Style::default().fg(Color::Yellow)),
            ]),
            Line::from(vec![
                Span::styled("Discovered At  : ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    sw.discovered_at.to_rfc3339(),
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::from(vec![
                Span::styled("Last Seen At   : ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    sw.last_seen_at.to_rfc3339(),
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Primary Binary : ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    sw.primary_binary_path
                        .as_ref()
                        .map(|p| p.display().to_string())
                        .unwrap_or_else(|| "none".to_string()),
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::from(vec![
                Span::styled("Binary Status  : ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("[ {} ]", bin_status_text),
                    Style::default()
                        .fg(bin_status_color)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "Metadata Attributes:",
                Style::default().fg(Color::Yellow),
            )),
            Line::from(Span::styled(
                serde_json::to_string_pretty(&sw.metadata).unwrap_or_default(),
                Style::default().fg(Color::Gray),
            )),
        ];

        // Linked installations
        if let Ok(installs) = app.db.get_installations_for_software(sw.id) {
            detail_lines.push(Line::from(""));
            detail_lines.push(Line::from(Span::styled(
                format!("Linked Installation Records ({}):", installs.len()),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )));
            for inst in installs {
                detail_lines.push(Line::from(vec![
                    Span::styled("  • ", Style::default().fg(Color::Cyan)),
                    Span::styled(
                        format!(
                            "Installed: {} | Scope: {} | Class: {}",
                            inst.installed_at.format("%Y-%m-%d"),
                            inst.scope,
                            inst.classification
                        ),
                        Style::default().fg(Color::White),
                    ),
                ]));
            }
        }

        let detail_block = Paragraph::new(detail_lines).block(
            Block::default()
                .title(" Package Provenance & Inspection ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        );
        frame.render_widget(detail_block, chunks[1]);
    } else {
        let empty = Paragraph::new(
            "No software selected or database is empty.\nPress 's' to scan system PATH.",
        )
        .block(
            Block::default()
                .title(" Details ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
        frame.render_widget(empty, chunks[1]);
    }
}
