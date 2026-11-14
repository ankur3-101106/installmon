use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(8), Constraint::Min(10)])
        .split(area);

    let is_clean = app.audit_warnings.is_empty();

    let status_color = if is_clean {
        Color::Green
    } else {
        Color::Yellow
    };
    let status_headline = if is_clean {
        "ALL AUDIT CHECKS PASSED — NO ANOMALIES DETECTED"
    } else {
        "AUDIT NOTICES / SYSTEM ANOMALIES FOUND"
    };

    let summary_lines = vec![
        Line::from(vec![
            Span::styled(
                "Overall Health Status : ",
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                status_headline,
                Style::default()
                    .fg(status_color)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "Database Integrity    : ",
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(&app.integrity_status, Style::default().fg(Color::Green)),
        ]),
        Line::from(vec![
            Span::styled(
                "Audited Software      : ",
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                format!("{} entries checked", app.software_list.len()),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "Audited Sessions      : ",
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                format!("{} sessions inspected", app.sessions_list.len()),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "Total Audit Notices   : ",
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                format!("{} notices", app.audit_warnings.len()),
                Style::default()
                    .fg(if is_clean {
                        Color::Green
                    } else {
                        Color::Yellow
                    })
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
    ];

    let summary_block = Paragraph::new(summary_lines).block(
        Block::default()
            .title(" System Integrity & Health Summary ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(status_color)),
    );
    frame.render_widget(summary_block, rows[0]);

    // Warnings list
    let mut notice_lines = Vec::new();
    if is_clean {
        notice_lines.push(Line::from(""));
        notice_lines.push(Line::from(Span::styled(
            "  ✓ Database integrity check verified via PRAGMA integrity_check.",
            Style::default().fg(Color::Green),
        )));
        notice_lines.push(Line::from(Span::styled(
            "  ✓ All tracked software binaries are present and accessible on disk.",
            Style::default().fg(Color::Green),
        )));
        notice_lines.push(Line::from(Span::styled(
            "  ✓ No orphaned or suspicious installation workflows discovered.",
            Style::default().fg(Color::Green),
        )));
        notice_lines.push(Line::from(Span::styled(
            "  ✓ Sensitive arguments and credential filters operational.",
            Style::default().fg(Color::Green),
        )));
    } else {
        for (i, w) in app.audit_warnings.iter().enumerate() {
            notice_lines.push(Line::from(vec![
                Span::styled(
                    format!("  [{:02}] ", i + 1),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "! ",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::styled(w, Style::default().fg(Color::White)),
            ]));
        }
    }

    let notices_block = Paragraph::new(notice_lines).block(
        Block::default()
            .title(" Audit Notices & Anomalies ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    frame.render_widget(notices_block, rows[1]);
}
