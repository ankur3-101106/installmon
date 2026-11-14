use anyhow::Result;
use chrono::Utc;
use crossterm::event::KeyCode;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;
use uuid::Uuid;

use installmon_core::capabilities::PlatformCapabilities;
use installmon_core::event::Event;
use installmon_core::provenance::InstallationScope;
use installmon_database::{Database, SessionRecord, SoftwareRecord};
use installmon_platform::detect_platform_capabilities;
use installmon_platform::service::{
    disable_service, enable_service, get_service_status, install_and_start_service,
    restart_service, spawn_standalone_daemon, start_service, stop_service, uninstall_service,
    ServiceState, ServiceStatusInfo,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurrentTab {
    Dashboard = 0,
    Software = 1,
    Sessions = 2,
    Events = 3,
    Audit = 4,
    Service = 5,
}

impl CurrentTab {
    pub fn next(self) -> Self {
        match self {
            Self::Dashboard => Self::Software,
            Self::Software => Self::Sessions,
            Self::Sessions => Self::Events,
            Self::Events => Self::Audit,
            Self::Audit => Self::Service,
            Self::Service => Self::Dashboard,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::Dashboard => Self::Service,
            Self::Software => Self::Dashboard,
            Self::Sessions => Self::Software,
            Self::Events => Self::Sessions,
            Self::Audit => Self::Events,
            Self::Service => Self::Audit,
        }
    }

    pub fn titles() -> [&'static str; 6] {
        [
            "[1] Dashboard",
            "[2] Software",
            "[3] Sessions",
            "[4] Events",
            "[5] Audit",
            "[6] Service",
        ]
    }
}

pub struct App {
    pub db: Database,
    pub current_tab: CurrentTab,
    pub showing_help: bool,

    // Data
    pub capabilities: PlatformCapabilities,
    pub service_status: ServiceStatusInfo,
    pub software_list: Vec<SoftwareRecord>,
    pub sessions_list: Vec<SessionRecord>,
    pub events_list: Vec<Event>,
    pub audit_warnings: Vec<String>,
    pub integrity_status: String,

    // Selections
    pub selected_software: usize,
    pub selected_session: usize,
    pub selected_event: usize,

    // Notification toast
    pub status_message: Option<(String, Instant)>,
}

impl App {
    pub fn new(db: Database) -> Self {
        let caps = detect_platform_capabilities();
        let service_status = get_service_status(caps.is_root);
        Self {
            db,
            current_tab: CurrentTab::Dashboard,
            showing_help: false,
            capabilities: caps,
            service_status,
            software_list: Vec::new(),
            sessions_list: Vec::new(),
            events_list: Vec::new(),
            audit_warnings: Vec::new(),
            integrity_status: "Pending".to_string(),
            selected_software: 0,
            selected_session: 0,
            selected_event: 0,
            status_message: Some((
                "InstallMon TUI ready. Press '?' for help, '6' for Service.".to_string(),
                Instant::now(),
            )),
        }
    }

    pub fn refresh(&mut self) -> Result<()> {
        self.capabilities = detect_platform_capabilities();
        self.service_status = get_service_status(self.capabilities.is_root);
        self.software_list = self.db.get_all_software().unwrap_or_default();
        self.sessions_list = self.db.get_sessions(100).unwrap_or_default();
        self.events_list = self.db.get_events(100, 0, None, None).unwrap_or_default();

        // Audit
        let mut warnings = Vec::new();
        self.integrity_status = match self.db.verify_integrity() {
            Ok(_) => "Passed (PRAGMA integrity_check ok)".to_string(),
            Err(e) => {
                let msg = format!("Integrity error: {}", e);
                warnings.push(msg.clone());
                msg
            }
        };

        for sw in &self.software_list {
            if let Some(ref path) = sw.primary_binary_path {
                if !path.exists() {
                    warnings.push(format!(
                        "Missing binary on disk: {} ({})",
                        sw.name,
                        path.display()
                    ));
                }
            }
        }

        for s in &self.sessions_list {
            if s.classification.to_string() == "unknown_workflow" {
                warnings.push(format!(
                    "Session {} has unknown workflow classification",
                    s.id
                ));
            }
        }

        self.audit_warnings = warnings;

        // Bound selections
        if !self.software_list.is_empty() && self.selected_software >= self.software_list.len() {
            self.selected_software = self.software_list.len() - 1;
        }
        if !self.sessions_list.is_empty() && self.selected_session >= self.sessions_list.len() {
            self.selected_session = self.sessions_list.len() - 1;
        }
        if !self.events_list.is_empty() && self.selected_event >= self.events_list.len() {
            self.selected_event = self.events_list.len() - 1;
        }

        Ok(())
    }

    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status_message = Some((msg.into(), Instant::now()));
    }

    pub fn trigger_scan(&mut self) -> Result<usize> {
        let mut count = 0;
        let mut paths = Vec::new();

        if let Ok(path_var) = env::var("PATH") {
            for p in env::split_paths(&path_var) {
                if p.starts_with("/usr/local")
                    || p.to_string_lossy().contains(".local")
                    || p.to_string_lossy().contains(".cargo")
                {
                    paths.push(p);
                }
            }
        }

        if paths.is_empty() {
            paths.push(PathBuf::from("/usr/local/bin"));
            if let Ok(home) = env::var("HOME") {
                let home_p = PathBuf::from(home);
                paths.push(home_p.join(".local/bin"));
                paths.push(home_p.join(".cargo/bin"));
            }
        }

        for dir in paths {
            if !dir.exists() || !dir.is_dir() {
                continue;
            }
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if is_executable(&path) {
                        let name = path
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_default();
                        if !name.is_empty() && self.db.get_software_by_name(&name)?.is_none() {
                            let is_system = path.starts_with("/usr") || path.starts_with("/opt");
                            let rec = SoftwareRecord {
                                id: Uuid::new_v4(),
                                name: name.clone(),
                                display_name: Some(name),
                                primary_binary_path: Some(path.clone()),
                                scope: if is_system {
                                    InstallationScope::System
                                } else {
                                    InstallationScope::User
                                },
                                discovered_at: Utc::now(),
                                last_seen_at: Utc::now(),
                                metadata: serde_json::json!({
                                    "discovery_method": "tui_scan",
                                    "source_path": path.to_string_lossy()
                                }),
                            };
                            let _ = self.db.insert_software(&rec);
                            count += 1;
                        }
                    }
                }
            }
        }

        self.refresh()?;
        Ok(count)
    }

    pub fn setup_service(&mut self) {
        let is_root = self.capabilities.is_root;
        match install_and_start_service(is_root) {
            Ok(msg) => self.set_status(format!("✓ {}", msg)),
            Err(e) => self.set_status(format!("✗ Setup failed: {}", e)),
        }
        let _ = self.refresh();
    }

    pub fn toggle_service(&mut self) {
        let is_root = self.capabilities.is_root;
        if self.service_status.state == ServiceState::Running {
            match stop_service(is_root) {
                Ok(msg) => self.set_status(format!("✓ {}", msg)),
                Err(e) => self.set_status(format!("✗ Stop failed: {}", e)),
            }
        } else {
            match start_service(is_root) {
                Ok(msg) => self.set_status(format!("✓ {}", msg)),
                Err(e) => self.set_status(format!("✗ Start failed: {}", e)),
            }
        }
        let _ = self.refresh();
    }

    pub fn restart_service(&mut self) {
        let is_root = self.capabilities.is_root;
        match restart_service(is_root) {
            Ok(msg) => self.set_status(format!("✓ {}", msg)),
            Err(e) => self.set_status(format!("✗ Restart failed: {}", e)),
        }
        let _ = self.refresh();
    }

    pub fn toggle_autostart(&mut self) {
        let is_root = self.capabilities.is_root;
        if self.service_status.is_enabled {
            match disable_service(is_root) {
                Ok(msg) => self.set_status(format!("✓ {}", msg)),
                Err(e) => self.set_status(format!("✗ Disable failed: {}", e)),
            }
        } else {
            match enable_service(is_root) {
                Ok(msg) => self.set_status(format!("✓ {}", msg)),
                Err(e) => self.set_status(format!("✗ Enable failed: {}", e)),
            }
        }
        let _ = self.refresh();
    }

    pub fn spawn_standalone(&mut self) {
        let is_root = self.capabilities.is_root;
        match spawn_standalone_daemon(is_root) {
            Ok(pid) => self.set_status(format!(
                "✓ Standalone background daemon spawned (PID: {})",
                pid
            )),
            Err(e) => self.set_status(format!("✗ Spawn failed: {}", e)),
        }
        let _ = self.refresh();
    }

    pub fn uninstall_service(&mut self) {
        let is_root = self.capabilities.is_root;
        match uninstall_service(is_root) {
            Ok(msg) => self.set_status(format!("✓ {}", msg)),
            Err(e) => self.set_status(format!("✗ Uninstall failed: {}", e)),
        }
        let _ = self.refresh();
    }

    pub fn handle_key(&mut self, code: KeyCode) {
        if self.showing_help {
            if matches!(
                code,
                KeyCode::Char('?') | KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q')
            ) {
                self.showing_help = false;
            }
            return;
        }

        match code {
            KeyCode::Char('?') => {
                self.showing_help = true;
            }
            KeyCode::Tab | KeyCode::Right | KeyCode::Char('l') => {
                self.current_tab = self.current_tab.next();
            }
            KeyCode::BackTab | KeyCode::Left | KeyCode::Char('h') => {
                self.current_tab = self.current_tab.prev();
            }
            KeyCode::Char('1') => self.current_tab = CurrentTab::Dashboard,
            KeyCode::Char('2') => self.current_tab = CurrentTab::Software,
            KeyCode::Char('3') => self.current_tab = CurrentTab::Sessions,
            KeyCode::Char('4') => self.current_tab = CurrentTab::Events,
            KeyCode::Char('5') => self.current_tab = CurrentTab::Audit,
            KeyCode::Char('6') => self.current_tab = CurrentTab::Service,

            // Centralized Service Controls
            KeyCode::Char('I') | KeyCode::Char('i') => {
                self.setup_service();
            }
            KeyCode::Char('S') => {
                self.toggle_service();
            }
            KeyCode::Char('R') => {
                self.restart_service();
            }
            KeyCode::Char('e') if self.current_tab == CurrentTab::Service => {
                self.toggle_autostart();
            }
            KeyCode::Char('b') if self.current_tab == CurrentTab::Service => {
                self.spawn_standalone();
            }
            KeyCode::Char('u') if self.current_tab == CurrentTab::Service => {
                self.uninstall_service();
            }

            KeyCode::Char('r') => {
                let _ = self.refresh();
                self.set_status("Refreshed database and service state.");
            }
            KeyCode::Char('s') => {
                self.set_status("Scanning PATH for binaries...");
                match self.trigger_scan() {
                    Ok(n) => {
                        self.set_status(format!("Scan completed: {} new binaries discovered.", n))
                    }
                    Err(e) => self.set_status(format!("Scan failed: {}", e)),
                }
            }

            KeyCode::Down | KeyCode::Char('j') => match self.current_tab {
                CurrentTab::Software
                    if !self.software_list.is_empty()
                        && self.selected_software + 1 < self.software_list.len() =>
                {
                    self.selected_software += 1;
                }
                CurrentTab::Sessions
                    if !self.sessions_list.is_empty()
                        && self.selected_session + 1 < self.sessions_list.len() =>
                {
                    self.selected_session += 1;
                }
                CurrentTab::Events
                    if !self.events_list.is_empty()
                        && self.selected_event + 1 < self.events_list.len() =>
                {
                    self.selected_event += 1;
                }
                _ => {}
            },

            KeyCode::Up | KeyCode::Char('k') => match self.current_tab {
                CurrentTab::Software if self.selected_software > 0 => {
                    self.selected_software -= 1;
                }
                CurrentTab::Sessions if self.selected_session > 0 => {
                    self.selected_session -= 1;
                }
                CurrentTab::Events if self.selected_event > 0 => {
                    self.selected_event -= 1;
                }
                _ => {}
            },

            _ => {}
        }
    }
}

fn is_executable(path: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = fs::metadata(path) {
            return metadata.is_file() && (metadata.permissions().mode() & 0o111 != 0);
        }
    }
    false
}
