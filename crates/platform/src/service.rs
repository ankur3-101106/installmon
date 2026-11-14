use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::paths::{config_dir, data_dir, ensure_parent_dir_exists};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceState {
    Running,
    Stopped,
    NotInstalled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceType {
    SystemdUser,
    SystemdSystem,
    Standalone,
}

impl std::fmt::Display for ServiceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SystemdUser => write!(f, "systemd (User Service)"),
            Self::SystemdSystem => write!(f, "systemd (System Service)"),
            Self::Standalone => write!(f, "Standalone Background Daemon"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonHeartbeat {
    pub pid: u32,
    pub started_at: DateTime<Utc>,
    pub last_heartbeat: DateTime<Utc>,
    pub is_system: bool,
    pub events_recorded: u64,
    pub sessions_recorded: u64,
    pub active_backends: Vec<String>,
    pub monitored_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceStatusInfo {
    pub state: ServiceState,
    pub service_type: ServiceType,
    pub pid: Option<u32>,
    pub uptime_seconds: Option<u64>,
    pub is_installed: bool,
    pub is_enabled: bool,
    pub unit_path: PathBuf,
    pub daemon_log_path: PathBuf,
    pub heartbeat: Option<DaemonHeartbeat>,
    pub details: String,
}

/// Path to the JSON heartbeat file written by the running daemon.
pub fn daemon_info_path(is_system: bool) -> PathBuf {
    data_dir(is_system).join("daemon.json")
}

/// Path to the daemon log file for background execution.
pub fn daemon_log_path(is_system: bool) -> PathBuf {
    data_dir(is_system).join("daemon.log")
}

/// Path to the systemd unit file.
pub fn unit_file_path(is_system: bool) -> PathBuf {
    if is_system {
        PathBuf::from("/etc/systemd/system/installmon.service")
    } else if let Ok(home) = env::var("HOME") {
        PathBuf::from(home)
            .join(".config")
            .join("systemd")
            .join("user")
            .join("installmon.service")
    } else {
        config_dir(false).join("installmon.service")
    }
}

/// Checks whether a process with `pid` is currently alive and corresponds to `installmon`.
pub fn is_process_alive(pid: u32) -> bool {
    let proc_path = PathBuf::from(format!("/proc/{}", pid));
    if !proc_path.exists() {
        return false;
    }
    if let Ok(comm) = fs::read_to_string(proc_path.join("comm")) {
        let comm = comm.trim();
        return comm.contains("installmon") || comm.contains("footprint");
    }
    // Fallback: directory exists
    true
}

/// Reads the daemon heartbeat file if present and valid.
pub fn read_daemon_heartbeat(is_system: bool) -> Option<DaemonHeartbeat> {
    let path = daemon_info_path(is_system);
    if !path.exists() {
        return None;
    }
    let data = fs::read_to_string(&path).ok()?;
    serde_json::from_str::<DaemonHeartbeat>(&data).ok()
}

/// Writes the daemon heartbeat file.
pub fn write_daemon_heartbeat(is_system: bool, info: &DaemonHeartbeat) -> Result<()> {
    let path = daemon_info_path(is_system);
    ensure_parent_dir_exists(&path)?;
    let content = serde_json::to_string_pretty(info)?;
    fs::write(&path, content)?;
    Ok(())
}

/// Removes the daemon heartbeat file on clean shutdown.
pub fn remove_daemon_heartbeat(is_system: bool) {
    let path = daemon_info_path(is_system);
    let _ = fs::remove_file(path);
}

/// Detects whether `systemctl` is available on the system.
pub fn has_systemctl() -> bool {
    Command::new("systemctl")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Returns the systemd unit file content.
pub fn generate_unit_file(is_system: bool, binary_path: &Path) -> String {
    let mode_flag = if is_system { "--system " } else { "" };
    let target = if is_system {
        "multi-user.target"
    } else {
        "default.target"
    };

    format!(
        "[Unit]\n\
         Description=InstallMon - Continuous Linux Software Installation Monitoring Daemon\n\
         Documentation=https://github.com/ankur3-101106/installmon\n\
         After=network.target\n\
         \n\
         [Service]\n\
         Type=simple\n\
         ExecStart={} {}daemon\n\
         Restart=on-failure\n\
         RestartSec=3s\n\
         Environment=RUST_LOG=info\n\
         \n\
         [Install]\n\
         WantedBy={}\n",
        binary_path.display(),
        mode_flag,
        target
    )
}

/// Query full status of the background service.
pub fn get_service_status(is_system: bool) -> ServiceStatusInfo {
    let unit_path = unit_file_path(is_system);
    let log_path = daemon_log_path(is_system);
    let is_systemd = has_systemctl();
    let service_type = if is_system {
        ServiceType::SystemdSystem
    } else if is_systemd {
        ServiceType::SystemdUser
    } else {
        ServiceType::Standalone
    };

    let unit_exists = unit_path.exists();
    let heartbeat = read_daemon_heartbeat(is_system);

    // Check systemd status if unit exists and systemctl is available
    let mut systemd_active = false;
    let mut systemd_enabled = false;
    let mut systemd_pid: Option<u32> = None;

    if is_systemd && unit_exists {
        let mut cmd = Command::new("systemctl");
        if !is_system {
            cmd.arg("--user");
        }
        cmd.args(["is-active", "installmon"]);
        if let Ok(output) = cmd.output() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            systemd_active = stdout.trim() == "active";
        }

        let mut cmd_en = Command::new("systemctl");
        if !is_system {
            cmd_en.arg("--user");
        }
        cmd_en.args(["is-enabled", "installmon"]);
        if let Ok(output) = cmd_en.output() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            systemd_enabled = stdout.trim() == "enabled";
        }

        if systemd_active {
            let mut cmd_pid = Command::new("systemctl");
            if !is_system {
                cmd_pid.arg("--user");
            }
            cmd_pid.args(["show", "installmon", "--property=MainPID"]);
            if let Ok(output) = cmd_pid.output() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if let Some(pid_str) = stdout.trim().strip_prefix("MainPID=") {
                    if let Ok(p) = pid_str.parse::<u32>() {
                        if p > 0 {
                            systemd_pid = Some(p);
                        }
                    }
                }
            }
        }
    }

    // Determine running state and PID
    let (state, pid, uptime_seconds, details) = if systemd_active {
        let pid = systemd_pid.or_else(|| heartbeat.as_ref().map(|h| h.pid));
        let uptime = heartbeat
            .as_ref()
            .map(|h| (Utc::now() - h.started_at).num_seconds().max(0) as u64);
        (
            ServiceState::Running,
            pid,
            uptime,
            format!("Running under systemd (PID: {:?})", pid.unwrap_or(0)),
        )
    } else if let Some(ref hb) = heartbeat {
        if is_process_alive(hb.pid) {
            let uptime = (Utc::now() - hb.started_at).num_seconds().max(0) as u64;
            (
                ServiceState::Running,
                Some(hb.pid),
                Some(uptime),
                format!("Running as standalone background daemon (PID: {})", hb.pid),
            )
        } else {
            (
                if unit_exists {
                    ServiceState::Stopped
                } else {
                    ServiceState::NotInstalled
                },
                None,
                None,
                "Process terminated or dead heartbeat detected".to_string(),
            )
        }
    } else if unit_exists {
        (
            ServiceState::Stopped,
            None,
            None,
            "Service unit installed but not currently running".to_string(),
        )
    } else {
        (
            ServiceState::NotInstalled,
            None,
            None,
            "Service not installed. Press [I] to set up and start.".to_string(),
        )
    };

    ServiceStatusInfo {
        state,
        service_type,
        pid,
        uptime_seconds,
        is_installed: unit_exists,
        is_enabled: systemd_enabled,
        unit_path,
        daemon_log_path: log_path,
        heartbeat,
        details,
    }
}

/// Automatically install the service unit file, reload systemd, enable, and start it.
pub fn install_and_start_service(is_system: bool) -> Result<String> {
    let current_exe =
        env::current_exe().context("Failed to determine path to current executable")?;
    let unit_path = unit_file_path(is_system);
    ensure_parent_dir_exists(&unit_path)?;

    let unit_content = generate_unit_file(is_system, &current_exe);
    fs::write(&unit_path, unit_content)
        .with_context(|| format!("Failed to write service unit to {}", unit_path.display()))?;

    if has_systemctl() {
        // systemctl daemon-reload
        let mut reload_cmd = Command::new("systemctl");
        if !is_system {
            reload_cmd.arg("--user");
        }
        reload_cmd.arg("daemon-reload");
        let _ = reload_cmd.status();

        // systemctl enable installmon
        let mut enable_cmd = Command::new("systemctl");
        if !is_system {
            enable_cmd.arg("--user");
        }
        enable_cmd.args(["enable", "installmon"]);
        let _ = enable_cmd.status();

        // systemctl start installmon
        let mut start_cmd = Command::new("systemctl");
        if !is_system {
            start_cmd.arg("--user");
        }
        start_cmd.args(["start", "installmon"]);
        let output = start_cmd
            .output()
            .context("Failed to execute systemctl start")?;
        if output.status.success() {
            return Ok(format!(
                "Successfully installed and started InstallMon service via systemd ({})",
                unit_path.display()
            ));
        }
    }

    // Fallback: spawn standalone daemon detached
    let pid = spawn_standalone_daemon(is_system)?;
    Ok(format!(
        "Installed unit at {} and launched background daemon (PID: {})",
        unit_path.display(),
        pid
    ))
}

/// Start the background service.
pub fn start_service(is_system: bool) -> Result<String> {
    if has_systemctl() && unit_file_path(is_system).exists() {
        let mut cmd = Command::new("systemctl");
        if !is_system {
            cmd.arg("--user");
        }
        cmd.args(["start", "installmon"]);
        let output = cmd.output().context("Failed to run systemctl start")?;
        if output.status.success() {
            return Ok("Service started successfully via systemctl.".to_string());
        }
    }

    let pid = spawn_standalone_daemon(is_system)?;
    Ok(format!(
        "Started standalone background daemon (PID: {})",
        pid
    ))
}

/// Stop the background service.
pub fn stop_service(is_system: bool) -> Result<String> {
    if has_systemctl() && unit_file_path(is_system).exists() {
        let mut cmd = Command::new("systemctl");
        if !is_system {
            cmd.arg("--user");
        }
        cmd.args(["stop", "installmon"]);
        let _ = cmd.status();
    }

    // Also stop standalone daemon if running
    if let Some(hb) = read_daemon_heartbeat(is_system) {
        if is_process_alive(hb.pid) {
            unsafe {
                libc::kill(hb.pid as i32, libc::SIGTERM);
            }
            std::thread::sleep(std::time::Duration::from_millis(200));
        }
        remove_daemon_heartbeat(is_system);
    }

    Ok("Service stopped successfully.".to_string())
}

/// Restart the background service.
pub fn restart_service(is_system: bool) -> Result<String> {
    let _ = stop_service(is_system);
    std::thread::sleep(std::time::Duration::from_millis(300));
    start_service(is_system)
}

/// Enable systemd auto-start on boot / login.
pub fn enable_service(is_system: bool) -> Result<String> {
    if !has_systemctl() {
        bail!("systemctl is not available on this platform");
    }
    let mut cmd = Command::new("systemctl");
    if !is_system {
        cmd.arg("--user");
    }
    cmd.args(["enable", "installmon"]);
    let output = cmd.output().context("Failed to run systemctl enable")?;
    if output.status.success() {
        Ok("Service enabled for automatic startup.".to_string())
    } else {
        bail!(
            "Failed to enable service: {}",
            String::from_utf8_lossy(&output.stderr)
        )
    }
}

/// Disable systemd auto-start on boot / login.
pub fn disable_service(is_system: bool) -> Result<String> {
    if !has_systemctl() {
        bail!("systemctl is not available on this platform");
    }
    let mut cmd = Command::new("systemctl");
    if !is_system {
        cmd.arg("--user");
    }
    cmd.args(["disable", "installmon"]);
    let output = cmd.output().context("Failed to run systemctl disable")?;
    if output.status.success() {
        Ok("Service disabled from automatic startup.".to_string())
    } else {
        bail!(
            "Failed to disable service: {}",
            String::from_utf8_lossy(&output.stderr)
        )
    }
}

/// Cleanly uninstall the service unit and stop any running instance.
pub fn uninstall_service(is_system: bool) -> Result<String> {
    let _ = stop_service(is_system);
    let unit_path = unit_file_path(is_system);
    if unit_path.exists() {
        let _ = fs::remove_file(&unit_path);
    }

    if has_systemctl() {
        let mut cmd = Command::new("systemctl");
        if !is_system {
            cmd.arg("--user");
        }
        cmd.arg("daemon-reload");
        let _ = cmd.status();
    }

    remove_daemon_heartbeat(is_system);
    Ok("Service unit uninstalled and stopped.".to_string())
}

/// Spawn the daemon as a detached background process (works universally without systemd).
pub fn spawn_standalone_daemon(is_system: bool) -> Result<u32> {
    let current_exe = env::current_exe().context("Failed to determine current executable path")?;
    let log_path = daemon_log_path(is_system);
    ensure_parent_dir_exists(&log_path)?;

    let log_file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .with_context(|| format!("Failed to open log file at {}", log_path.display()))?;

    let log_file_err = log_file.try_clone()?;

    let mut cmd = Command::new(&current_exe);
    if is_system {
        cmd.arg("--system");
    }
    cmd.arg("daemon");
    cmd.stdout(log_file);
    cmd.stderr(log_file_err);

    let child = cmd
        .spawn()
        .context("Failed to spawn background daemon process")?;
    let pid = child.id();

    Ok(pid)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unit_generation() {
        let fake_bin = Path::new("/usr/local/bin/installmon");
        let unit_user = generate_unit_file(false, fake_bin);
        assert!(unit_user.contains("ExecStart=/usr/local/bin/installmon daemon"));
        assert!(unit_user.contains("WantedBy=default.target"));

        let unit_sys = generate_unit_file(true, fake_bin);
        assert!(unit_sys.contains("ExecStart=/usr/local/bin/installmon --system daemon"));
        assert!(unit_sys.contains("WantedBy=multi-user.target"));
    }

    #[test]
    fn test_daemon_heartbeat_serde() {
        let hb = DaemonHeartbeat {
            pid: 1234,
            started_at: Utc::now(),
            last_heartbeat: Utc::now(),
            is_system: false,
            events_recorded: 42,
            sessions_recorded: 3,
            active_backends: vec!["inotify".to_string(), "procfs".to_string()],
            monitored_paths: vec!["/usr/local/bin".to_string()],
        };
        let serialized = serde_json::to_string(&hb).expect("Failed to serialize");
        let deserialized: DaemonHeartbeat =
            serde_json::from_str(&serialized).expect("Failed to deserialize");
        assert_eq!(deserialized.pid, 1234);
        assert_eq!(deserialized.events_recorded, 42);
        assert_eq!(deserialized.sessions_recorded, 3);
    }
}
