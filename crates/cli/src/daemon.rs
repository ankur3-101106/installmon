use anyhow::Result;
use chrono::Utc;
use nix::sys::inotify::{AddWatchFlags, InitFlags, Inotify};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};
use uuid::Uuid;

use installmon_core::event::{
    CollectionConfidence, Event, EventType, MonitoringBackendType, ObservationKind,
};
use installmon_core::provenance::{InstallationScope, WorkflowClassification};
use installmon_core::sanitize::sanitize_arguments;
use installmon_database::{Database, SessionRecord, SoftwareRecord};
use installmon_platform::detect_platform_capabilities;
use installmon_platform::service::{
    remove_daemon_heartbeat, write_daemon_heartbeat, DaemonHeartbeat,
};

static RUNNING: AtomicBool = AtomicBool::new(true);

extern "C" fn handle_signal(_: libc::c_int) {
    RUNNING.store(false, Ordering::SeqCst);
}

/// Sets up standard Unix signal handling for graceful shutdown.
fn setup_signals() {
    unsafe {
        libc::signal(libc::SIGINT, handle_signal as *const () as usize);
        libc::signal(libc::SIGTERM, handle_signal as *const () as usize);
    }
}

/// Continuous background daemon monitoring loop.
pub fn run_daemon(is_system: bool, db: &Database) -> Result<()> {
    setup_signals();
    tracing::info!(
        "Starting InstallMon continuous background daemon (scope: {})",
        if is_system { "system" } else { "user" }
    );

    let pid = std::process::id();
    let started_at = Utc::now();
    let _caps = detect_platform_capabilities();

    // Determine watched directories
    let mut watched_dirs = Vec::new();
    if is_system {
        for d in &[
            "/usr/local/bin",
            "/usr/bin",
            "/opt",
            "/etc",
            "/usr/local/lib",
        ] {
            let p = PathBuf::from(d);
            if p.exists() && p.is_dir() {
                watched_dirs.push(p);
            }
        }
    } else {
        if let Ok(home) = std::env::var("HOME") {
            let home_p = PathBuf::from(home);
            for sub in &[".local/bin", ".cargo/bin", ".local/share", ".config"] {
                let p = home_p.join(sub);
                if p.exists() && p.is_dir() {
                    watched_dirs.push(p);
                }
            }
        }
        let usr_local = PathBuf::from("/usr/local/bin");
        if usr_local.exists() && usr_local.is_dir() && !watched_dirs.contains(&usr_local) {
            watched_dirs.push(usr_local);
        }
    }

    // Initialize Inotify instance
    let inotify_opt = Inotify::init(InitFlags::IN_NONBLOCK | InitFlags::IN_CLOEXEC).ok();
    let mut watch_map = HashMap::new();

    if let Some(ref inotify) = inotify_opt {
        for dir in &watched_dirs {
            let watch_flags = AddWatchFlags::IN_CREATE
                | AddWatchFlags::IN_MOVED_TO
                | AddWatchFlags::IN_MODIFY
                | AddWatchFlags::IN_ATTRIB;
            if let Ok(wd) = inotify.add_watch(dir, watch_flags) {
                watch_map.insert(wd, dir.clone());
                tracing::debug!("Watching filesystem directory: {}", dir.display());
            }
        }
    }

    let mut active_backends = vec!["procfs".to_string()];
    if inotify_opt.is_some() {
        active_backends.push("inotify".to_string());
    }

    let monitored_paths_str: Vec<String> = watched_dirs
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    // Initial heartbeat
    let mut events_recorded: u64 = 0;
    let mut sessions_recorded: u64 = 0;

    let heartbeat = DaemonHeartbeat {
        pid,
        started_at,
        last_heartbeat: Utc::now(),
        is_system,
        events_recorded,
        sessions_recorded,
        active_backends: active_backends.clone(),
        monitored_paths: monitored_paths_str.clone(),
    };
    let _ = write_daemon_heartbeat(is_system, &heartbeat);

    let mut seen_pids: HashSet<u32> = HashSet::new();
    // Pre-populate with our own PID and existing system PIDs so we only catch new ones
    seen_pids.insert(pid);
    if let Ok(entries) = fs::read_dir("/proc") {
        for entry in entries.flatten() {
            if let Ok(p) = entry.file_name().to_string_lossy().parse::<u32>() {
                seen_pids.insert(p);
            }
        }
    }

    let mut last_heartbeat_time = Instant::now();

    tracing::info!("InstallMon daemon initialized. Entering continuous observation loop.");

    while RUNNING.load(Ordering::SeqCst) {
        // 1. Filesystem events via inotify
        if let Some(ref inotify) = inotify_opt {
            if let Ok(events) = inotify.read_events() {
                for ev in events {
                    if let Some(dir) = watch_map.get(&ev.wd) {
                        if let Some(name) = ev.name {
                            let filename = name.to_string_lossy();
                            let target_path = dir.join(&*filename);

                            if target_path.exists() {
                                let is_exec = is_executable(&target_path);
                                let event_type = if is_exec {
                                    EventType::FileCreate
                                } else {
                                    EventType::FileModify
                                };

                                let event = Event {
                                    id: Uuid::new_v4(),
                                    timestamp: Utc::now(),
                                    event_type,
                                    pid,
                                    ppid: None,
                                    uid: unsafe { libc::getuid() },
                                    euid: unsafe { libc::geteuid() },
                                    executable_path: target_path.clone(),
                                    arguments: Vec::new(),
                                    working_directory: Some(dir.clone()),
                                    target_path: Some(target_path.clone()),
                                    destination_path: None,
                                    backend: MonitoringBackendType::Inotify,
                                    confidence: CollectionConfidence::High,
                                    observation_kind: ObservationKind::Direct,
                                    session_id: None,
                                    metadata: serde_json::json!({
                                        "monitored_dir": dir.to_string_lossy(),
                                        "is_executable": is_exec,
                                    }),
                                };

                                if let Err(e) = db.insert_event(&event) {
                                    tracing::error!("Failed to record inotify event: {}", e);
                                } else {
                                    events_recorded += 1;
                                }

                                // Auto-register new executable software in bin directories
                                if is_exec
                                    && (dir.ends_with("bin")
                                        || dir.to_string_lossy().contains("bin"))
                                {
                                    let sw_name = filename.to_string();
                                    if let Ok(None) = db.get_software_by_name(&sw_name) {
                                        let sw = SoftwareRecord {
                                            id: Uuid::new_v4(),
                                            name: sw_name.clone(),
                                            display_name: Some(sw_name),
                                            primary_binary_path: Some(target_path),
                                            scope: if is_system {
                                                InstallationScope::System
                                            } else {
                                                InstallationScope::User
                                            },
                                            discovered_at: Utc::now(),
                                            last_seen_at: Utc::now(),
                                            metadata: serde_json::json!({ "source": "daemon_inotify_watch" }),
                                        };
                                        let _ = db.insert_software(&sw);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // 2. Process inspection via /proc
        if let Ok(entries) = fs::read_dir("/proc") {
            for entry in entries.flatten() {
                let name = entry.file_name();
                if let Ok(proc_pid) = name.to_string_lossy().parse::<u32>() {
                    if seen_pids.contains(&proc_pid) {
                        continue;
                    }
                    seen_pids.insert(proc_pid);

                    // Inspect this new process
                    if let Some(proc) = inspect_process(proc_pid) {
                        if let Some((classification, category)) =
                            classify_installer_comm(&proc.comm, &proc.cmdline)
                        {
                            tracing::info!(
                                "Detected installer process [PID: {}]: {} ({:?})",
                                proc_pid,
                                proc.comm,
                                classification
                            );

                            let sanitized_args = sanitize_arguments(&proc.cmdline);
                            let exe_path = PathBuf::from(
                                proc.cmdline.first().cloned().unwrap_or(proc.comm.clone()),
                            );

                            // Create an installation session
                            let session_id = Uuid::new_v4();
                            let session = SessionRecord {
                                id: session_id,
                                started_at: Utc::now(),
                                completed_at: None,
                                status: "active".to_string(),
                                scope: if proc.euid == 0 {
                                    InstallationScope::System
                                } else {
                                    InstallationScope::User
                                },
                                root_pid: Some(proc_pid),
                                initiating_command: Some(sanitized_args.join(" ")),
                                classification,
                                confidence: CollectionConfidence::High,
                                installer_identity: Some(proc.comm.clone()),
                                metadata: serde_json::json!({
                                    "ppid": proc.ppid,
                                    "cwd": proc.cwd.as_ref().map(|p| p.to_string_lossy().to_string()),
                                    "category": category,
                                    "observed_by": "daemon_procfs",
                                }),
                            };
                            if let Ok(()) = db.insert_session(&session) {
                                sessions_recorded += 1;
                            }

                            // Record ProcessExec event
                            let event = Event {
                                id: Uuid::new_v4(),
                                timestamp: Utc::now(),
                                event_type: EventType::ProcessExec,
                                pid: proc_pid,
                                ppid: proc.ppid,
                                uid: proc.uid,
                                euid: proc.euid,
                                executable_path: exe_path,
                                arguments: sanitized_args,
                                working_directory: proc.cwd,
                                target_path: None,
                                destination_path: None,
                                backend: MonitoringBackendType::Procfs,
                                confidence: CollectionConfidence::High,
                                observation_kind: ObservationKind::Direct,
                                session_id: Some(session_id),
                                metadata: serde_json::json!({
                                    "comm": proc.comm,
                                    "category": category,
                                    "classification": classification.to_string(),
                                }),
                            };

                            if let Ok(()) = db.insert_event(&event) {
                                events_recorded += 1;
                            }
                        }
                    }
                }
            }
        }

        // Keep seen_pids set bounded
        if seen_pids.len() > 10_000 {
            seen_pids.retain(|p| Path::new(&format!("/proc/{}", p)).exists());
        }

        // 3. Heartbeat update every 2 seconds
        if last_heartbeat_time.elapsed() >= Duration::from_secs(2) {
            let hb = DaemonHeartbeat {
                pid,
                started_at,
                last_heartbeat: Utc::now(),
                is_system,
                events_recorded,
                sessions_recorded,
                active_backends: active_backends.clone(),
                monitored_paths: monitored_paths_str.clone(),
            };
            let _ = write_daemon_heartbeat(is_system, &hb);
            last_heartbeat_time = Instant::now();
        }

        thread::sleep(Duration::from_millis(800));
    }

    tracing::info!("InstallMon daemon stopping gracefully.");
    remove_daemon_heartbeat(is_system);
    Ok(())
}

/// Checks if a file is an executable binary or script.
fn is_executable(path: &Path) -> bool {
    if let Ok(meta) = path.metadata() {
        meta.is_file() && (meta.permissions().mode() & 0o111 != 0)
    } else {
        false
    }
}

struct ParsedProcess {
    comm: String,
    cmdline: Vec<String>,
    ppid: Option<u32>,
    cwd: Option<PathBuf>,
    uid: u32,
    euid: u32,
}

/// Reads process metadata from `/proc/[pid]`.
fn inspect_process(pid: u32) -> Option<ParsedProcess> {
    let proc_dir = PathBuf::from(format!("/proc/{}", pid));
    if !proc_dir.exists() {
        return None;
    }

    let comm = fs::read_to_string(proc_dir.join("comm"))
        .ok()?
        .trim()
        .to_string();

    let cmdline_bytes = fs::read(proc_dir.join("cmdline")).ok()?;
    let cmdline: Vec<String> = cmdline_bytes
        .split(|&b| b == 0)
        .filter(|chunk| !chunk.is_empty())
        .map(|chunk| String::from_utf8_lossy(chunk).to_string())
        .collect();

    let mut ppid = None;
    let mut uid = 1000;
    let mut euid = 1000;

    if let Ok(status) = fs::read_to_string(proc_dir.join("status")) {
        for line in status.lines() {
            if let Some(rest) = line.strip_prefix("PPid:\t") {
                ppid = rest.trim().parse::<u32>().ok();
            } else if let Some(rest) = line.strip_prefix("Uid:\t") {
                let parts: Vec<&str> = rest.split_whitespace().collect();
                if let Some(u) = parts.first().and_then(|s| s.parse::<u32>().ok()) {
                    uid = u;
                }
                if let Some(eu) = parts.get(1).and_then(|s| s.parse::<u32>().ok()) {
                    euid = eu;
                }
            }
        }
    }

    let cwd = fs::read_link(proc_dir.join("cwd")).ok();

    Some(ParsedProcess {
        comm,
        cmdline,
        ppid,
        cwd,
        uid,
        euid,
    })
}

/// Classifies whether a process is an installation workflow based on comm and cmdline.
fn classify_installer_comm(
    comm: &str,
    cmdline: &[String],
) -> Option<(WorkflowClassification, &'static str)> {
    let comm_lower = comm.to_lowercase();
    let full_cmd = cmdline.join(" ").to_lowercase();

    // 1. Direct Downloads
    if matches!(comm_lower.as_str(), "curl" | "wget" | "aria2c") {
        return Some((WorkflowClassification::KnownWorkflow, "direct_download"));
    }

    // 2. Package Managers
    if matches!(
        comm_lower.as_str(),
        "apt" | "apt-get" | "dpkg" | "pacman" | "dnf" | "yum" | "zypper" | "apk"
    ) {
        return Some((
            WorkflowClassification::KnownWorkflow,
            "distribution_package",
        ));
    }

    // 3. Language Package Managers
    if matches!(
        comm_lower.as_str(),
        "cargo" | "pip" | "pip3" | "uv" | "npm" | "pnpm" | "yarn" | "gem"
    ) || (comm_lower == "go" && full_cmd.contains("go install"))
    {
        return Some((WorkflowClassification::KnownWorkflow, "language_ecosystem"));
    }

    // 4. Build Systems
    if matches!(
        comm_lower.as_str(),
        "make" | "ninja" | "cmake" | "meson" | "gcc" | "g++" | "clang" | "rustc"
    ) {
        return Some((WorkflowClassification::KnownWorkflow, "source_build"));
    }

    // 5. Git Clones
    if comm_lower == "git" && (full_cmd.contains("clone") || full_cmd.contains("pull")) {
        return Some((WorkflowClassification::KnownWorkflow, "git_repository"));
    }

    // 6. Archive Extraction
    if matches!(
        comm_lower.as_str(),
        "tar" | "unzip" | "7z" | "xz" | "gzip" | "bzip2"
    ) {
        return Some((WorkflowClassification::KnownWorkflow, "archive_extraction"));
    }

    // 7. Scripts Executed for Installation
    if matches!(comm_lower.as_str(), "sh" | "bash" | "zsh")
        && (full_cmd.contains("install")
            || full_cmd.contains("setup")
            || full_cmd.contains("configure"))
    {
        return Some((WorkflowClassification::InferredWorkflow, "script_execution"));
    }

    // 8. Executable Installation / Deployment
    if matches!(comm_lower.as_str(), "install" | "cp")
        && (full_cmd.contains("/usr/local") || full_cmd.contains(".local/bin"))
    {
        return Some((
            WorkflowClassification::InferredWorkflow,
            "binary_deployment",
        ));
    }

    None
}
