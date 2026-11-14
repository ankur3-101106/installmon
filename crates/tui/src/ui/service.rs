use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;
use installmon_platform::service::ServiceState;

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7), // Service Status Banner & Overview
            Constraint::Min(10),   // Controls & Monitoring Scope
            Constraint::Length(8), // Live Daemon Heartbeat & Activity
        ])
        .split(area);

    let status = &app.service_status;

    // 1. Service Status Banner
    let (state_color, state_text) = match status.state {
        ServiceState::Running => (
            Color::Green,
            format!(
                "● ACTIVE & RUNNING (PID: {}, Uptime: {})",
                status
                    .pid
                    .map(|p| p.to_string())
                    .unwrap_or_else(|| "Unknown".to_string()),
                status
                    .uptime_seconds
                    .map(|u| format!("{}s", u))
                    .unwrap_or_else(|| "N/A".to_string())
            ),
        ),
        ServiceState::Stopped => (
            Color::Yellow,
            "○ STOPPED (Service installed but inactive)".to_string(),
        ),
        ServiceState::NotInstalled => (
            Color::Red,
            "! NOT INSTALLED (Press [I] to set up systemd unit & start)".to_string(),
        ),
    };

    let status_lines = vec![
        Line::from(vec![
            Span::styled(
                "Service Status     : ",
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                state_text,
                Style::default()
                    .fg(state_color)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "Service Type       : ",
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                status.service_type.to_string(),
                Style::default().fg(Color::Cyan),
            ),
            Span::raw(" | "),
            Span::styled(
                "Auto-Start on Boot : ",
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                if status.is_enabled {
                    "Enabled"
                } else {
                    "Disabled"
                },
                Style::default().fg(if status.is_enabled {
                    Color::Green
                } else {
                    Color::Yellow
                }),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "Unit File Location : ",
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                status.unit_path.display().to_string(),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "Daemon Log Output  : ",
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                status.daemon_log_path.display().to_string(),
                Style::default().fg(Color::DarkGray),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "Current Details    : ",
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(&status.details, Style::default().fg(Color::White)),
        ]),
    ];

    let status_block = Paragraph::new(status_lines).block(
        Block::default()
            .title(" Background Service Status & Lifecycle ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(state_color)),
    );
    frame.render_widget(status_block, chunks[0]);

    // 2. Middle Row: Scope on Left, Interactive Controls on Right
    let mid_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(chunks[1]);

    // Monitoring Scope
    let monitored_paths = if let Some(ref hb) = status.heartbeat {
        if !hb.monitored_paths.is_empty() {
            hb.monitored_paths.join("\n  • ")
        } else {
            "Standard binary directories (/usr/local/bin, ~/.local/bin)".to_string()
        }
    } else {
        "/usr/local/bin, ~/.local/bin, ~/.cargo/bin, /opt".to_string()
    };

    let scope_lines = vec![
        Line::from(vec![
            Span::styled(
                "Active Backends    : ",
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                format!(
                    "[{}] Inotify Filesystem | [{}] Procfs Process Polling",
                    "✓", "✓"
                ),
                Style::default().fg(Color::Green),
            ),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Monitored Directories (Inotify Watch):",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![Span::styled(
            format!("  • {}", monitored_paths),
            Style::default().fg(Color::White),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Recognized Installers:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled(
                "  • Package Managers : ",
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                "apt, dpkg, pacman, dnf, yum, zypper, apk",
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "  • Language Tooling : ",
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                "cargo, pip, uv, npm, pnpm, yarn, go, gem",
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "  • Build Systems    : ",
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                "make, ninja, cmake, meson, gcc, clang, rustc",
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "  • Downloads & Git  : ",
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                "curl, wget, aria2, git clone",
                Style::default().fg(Color::White),
            ),
        ]),
    ];

    let scope_block = Paragraph::new(scope_lines).block(
        Block::default()
            .title(" Continuous Observation Scope ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue)),
    );
    frame.render_widget(scope_block, mid_cols[0]);

    // Centralized Service Actions (Directly from TUI!)
    let action_lines = vec![
        Line::from(vec![Span::styled(
            "Centralized Service Management:",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![Span::styled(
            "Everything can be set up and controlled right from this TUI.",
            Style::default().fg(Color::DarkGray),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                " [I] ",
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::styled(
                "Setup & Start Service",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![Span::styled(
            "     Installs systemd user/system unit, reloads, and starts daemon.",
            Style::default().fg(Color::DarkGray),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                " [S] ",
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::styled(
                "Start / Stop Service",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" (Toggle running state)"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                " [R] ",
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::styled(
                "Restart Background Service",
                Style::default().fg(Color::Cyan),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                " [E] ",
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::styled(
                "Toggle Auto-Start on Boot / Login",
                Style::default().fg(Color::Magenta),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                " [B] ",
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::styled("Run Standalone Daemon", Style::default().fg(Color::White)),
            Span::styled(
                " (Non-systemd fallback)",
                Style::default().fg(Color::DarkGray),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                " [U] ",
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Red)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::styled("Uninstall Service Unit", Style::default().fg(Color::Red)),
        ]),
    ];

    let action_block = Paragraph::new(action_lines).block(
        Block::default()
            .title(" Centralized Controls (Hotkeys) ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green)),
    );
    frame.render_widget(action_block, mid_cols[1]);

    // 3. Bottom Row: Live Daemon Heartbeat & Activity
    let (hb_events, hb_sessions, hb_time) = if let Some(ref hb) = status.heartbeat {
        (
            hb.events_recorded.to_string(),
            hb.sessions_recorded.to_string(),
            hb.last_heartbeat.format("%H:%M:%S UTC").to_string(),
        )
    } else {
        (
            "0".to_string(),
            "0".to_string(),
            "No active heartbeat".to_string(),
        )
    };

    let heartbeat_lines = vec![
        Line::from(vec![
            Span::styled("Daemon Heartbeat    : ", Style::default().fg(Color::DarkGray)),
            Span::styled(hb_time, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::raw(" | "),
            Span::styled("Events Recorded Continuously: ", Style::default().fg(Color::DarkGray)),
            Span::styled(hb_events, Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::raw(" | "),
            Span::styled("Sessions Captured: ", Style::default().fg(Color::DarkGray)),
            Span::styled(hb_sessions, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Autonomous Mode     : ", Style::default().fg(Color::DarkGray)),
            Span::styled("The service runs continuously in the background independent of this TUI.", Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::styled("Single-Pane-Of-Glass : ", Style::default().fg(Color::DarkGray)),
            Span::styled("You can launch this TUI at any time to review live data, audit changes, or reconfigure the service.", Style::default().fg(Color::DarkGray)),
        ]),
    ];

    let heartbeat_block = Paragraph::new(heartbeat_lines).block(
        Block::default()
            .title(" Autonomous Observation & Heartbeat ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );
    frame.render_widget(heartbeat_block, chunks[2]);
}
