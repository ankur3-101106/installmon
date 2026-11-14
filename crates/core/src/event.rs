use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::sanitize::sanitize_arguments;

/// Classification of event origin: directly observed by a kernel/monitoring backend,
/// or inferred through workflow correlation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObservationKind {
    /// Directly observed via monitoring interfaces (procfs, fanotify, inotify, eBPF, audit).
    Direct,
    /// Inferred by the correlation or classification engine.
    Inferred,
}

impl std::fmt::Display for ObservationKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Direct => write!(f, "direct"),
            Self::Inferred => write!(f, "inferred"),
        }
    }
}

/// Confidence level in the accuracy or provenance of the collected data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CollectionConfidence {
    Low = 1,
    Medium = 2,
    High = 3,
}

impl std::fmt::Display for CollectionConfidence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "low"),
            Self::Medium => write!(f, "medium"),
            Self::High => write!(f, "high"),
        }
    }
}

/// The underlying Linux monitoring technology responsible for observing the event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MonitoringBackendType {
    Ebpf,
    Audit,
    Procfs,
    Inotify,
    Fanotify,
    Netlink,
    Manual,
    Synthetic,
}

impl std::fmt::Display for MonitoringBackendType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ebpf => write!(f, "ebpf"),
            Self::Audit => write!(f, "audit"),
            Self::Procfs => write!(f, "procfs"),
            Self::Inotify => write!(f, "inotify"),
            Self::Fanotify => write!(f, "fanotify"),
            Self::Netlink => write!(f, "netlink"),
            Self::Manual => write!(f, "manual"),
            Self::Synthetic => write!(f, "synthetic"),
        }
    }
}

/// Normalized event types covering process lifecycle, filesystem changes,
/// download activity, privilege escalation, and installation lifecycle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    ProcessExec,
    ProcessExit,
    ProcessFork,
    FileCreate,
    FileModify,
    FileDelete,
    FileRename,
    SymlinkCreate,
    PermissionChange,
    DownloadObserved,
    PrivilegeChange,
    BuildStarted,
    BuildCompleted,
    InstallationStarted,
    InstallationCompleted,
    InstallationFailed,
}

impl std::fmt::Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ProcessExec => write!(f, "process_exec"),
            Self::ProcessExit => write!(f, "process_exit"),
            Self::ProcessFork => write!(f, "process_fork"),
            Self::FileCreate => write!(f, "file_create"),
            Self::FileModify => write!(f, "file_modify"),
            Self::FileDelete => write!(f, "file_delete"),
            Self::FileRename => write!(f, "file_rename"),
            Self::SymlinkCreate => write!(f, "symlink_create"),
            Self::PermissionChange => write!(f, "permission_change"),
            Self::DownloadObserved => write!(f, "download_observed"),
            Self::PrivilegeChange => write!(f, "privilege_change"),
            Self::BuildStarted => write!(f, "build_started"),
            Self::BuildCompleted => write!(f, "build_completed"),
            Self::InstallationStarted => write!(f, "installation_started"),
            Self::InstallationCompleted => write!(f, "installation_completed"),
            Self::InstallationFailed => write!(f, "installation_failed"),
        }
    }
}

/// The normalized event structure capturing all observed or inferred lifecycle activities.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Event {
    /// Unique identifier for the event.
    pub id: Uuid,
    /// UTC timestamp when the event occurred or was recorded.
    pub timestamp: DateTime<Utc>,
    /// Type of observed or inferred event.
    pub event_type: EventType,
    /// Process ID associated with the event.
    pub pid: u32,
    /// Parent process ID, if known.
    pub ppid: Option<u32>,
    /// Real user ID executing the process.
    pub uid: u32,
    /// Effective user ID executing the process (for privilege tracking).
    pub euid: u32,
    /// Absolute or resolved path to the executable.
    pub executable_path: PathBuf,
    /// Command-line arguments with sensitive credentials scrubbed.
    pub arguments: Vec<String>,
    /// Working directory of the process when the event occurred.
    pub working_directory: Option<PathBuf>,
    /// Target filesystem path, when applicable (file created, modified, deleted).
    pub target_path: Option<PathBuf>,
    /// Destination path for renames or symlink targets.
    pub destination_path: Option<PathBuf>,
    /// Subsystem or backend that collected this event.
    pub backend: MonitoringBackendType,
    /// Confidence level in this observation.
    pub confidence: CollectionConfidence,
    /// Whether this is directly observed or inferred.
    pub observation_kind: ObservationKind,
    /// Associated installation session, if correlated.
    pub session_id: Option<Uuid>,
    /// Extra event-specific metadata (e.g. exit code, permission bits, sha256).
    pub metadata: serde_json::Value,
}

impl Event {
    /// Create a new event builder.
    pub fn builder(event_type: EventType, executable_path: impl Into<PathBuf>) -> EventBuilder {
        EventBuilder::new(event_type, executable_path)
    }

    /// Check if this event was executed with root/elevated privileges.
    pub fn is_privileged(&self) -> bool {
        self.euid == 0
    }

    /// Check if this event represents a process execution.
    pub fn is_exec(&self) -> bool {
        matches!(self.event_type, EventType::ProcessExec)
    }

    /// Check if this event represents a filesystem mutation.
    pub fn is_filesystem_mutation(&self) -> bool {
        matches!(
            self.event_type,
            EventType::FileCreate
                | EventType::FileModify
                | EventType::FileDelete
                | EventType::FileRename
                | EventType::SymlinkCreate
                | EventType::PermissionChange
        )
    }
}

/// Fluent builder for constructing normalized `Event` records with argument sanitization.
pub struct EventBuilder {
    id: Uuid,
    timestamp: DateTime<Utc>,
    event_type: EventType,
    pid: u32,
    ppid: Option<u32>,
    uid: u32,
    euid: u32,
    executable_path: PathBuf,
    arguments: Vec<String>,
    working_directory: Option<PathBuf>,
    target_path: Option<PathBuf>,
    destination_path: Option<PathBuf>,
    backend: MonitoringBackendType,
    confidence: CollectionConfidence,
    observation_kind: ObservationKind,
    session_id: Option<Uuid>,
    metadata: serde_json::Value,
}

impl EventBuilder {
    pub fn new(event_type: EventType, executable_path: impl Into<PathBuf>) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            event_type,
            pid: std::process::id(),
            ppid: None,
            uid: 1000,
            euid: 1000,
            executable_path: executable_path.into(),
            arguments: Vec::new(),
            working_directory: None,
            target_path: None,
            destination_path: None,
            backend: MonitoringBackendType::Procfs,
            confidence: CollectionConfidence::High,
            observation_kind: ObservationKind::Direct,
            session_id: None,
            metadata: serde_json::json!({}),
        }
    }

    pub fn id(mut self, id: Uuid) -> Self {
        self.id = id;
        self
    }

    pub fn timestamp(mut self, timestamp: DateTime<Utc>) -> Self {
        self.timestamp = timestamp;
        self
    }

    pub fn pid(mut self, pid: u32) -> Self {
        self.pid = pid;
        self
    }

    pub fn ppid(mut self, ppid: Option<u32>) -> Self {
        self.ppid = ppid;
        self
    }

    pub fn uids(mut self, uid: u32, euid: u32) -> Self {
        self.uid = uid;
        self.euid = euid;
        self
    }

    pub fn arguments(mut self, args: &[String]) -> Self {
        // Automatically sanitize sensitive arguments
        self.arguments = sanitize_arguments(args);
        self
    }

    pub fn working_directory(mut self, path: impl AsRef<Path>) -> Self {
        self.working_directory = Some(path.as_ref().to_path_buf());
        self
    }

    pub fn target_path(mut self, path: impl AsRef<Path>) -> Self {
        self.target_path = Some(path.as_ref().to_path_buf());
        self
    }

    pub fn destination_path(mut self, path: impl AsRef<Path>) -> Self {
        self.destination_path = Some(path.as_ref().to_path_buf());
        self
    }

    pub fn backend(mut self, backend: MonitoringBackendType) -> Self {
        self.backend = backend;
        self
    }

    pub fn confidence(mut self, confidence: CollectionConfidence) -> Self {
        self.confidence = confidence;
        self
    }

    pub fn observation_kind(mut self, kind: ObservationKind) -> Self {
        self.observation_kind = kind;
        self
    }

    pub fn session_id(mut self, session_id: Option<Uuid>) -> Self {
        self.session_id = session_id;
        self
    }

    pub fn metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn build(self) -> Event {
        Event {
            id: self.id,
            timestamp: self.timestamp,
            event_type: self.event_type,
            pid: self.pid,
            ppid: self.ppid,
            uid: self.uid,
            euid: self.euid,
            executable_path: self.executable_path,
            arguments: self.arguments,
            working_directory: self.working_directory,
            target_path: self.target_path,
            destination_path: self.destination_path,
            backend: self.backend,
            confidence: self.confidence,
            observation_kind: self.observation_kind,
            session_id: self.session_id,
            metadata: self.metadata,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_builder_and_sanitization() {
        let raw_args = vec![
            "curl".into(),
            "-o".into(),
            "/usr/local/bin/mytool".into(),
            "--token".into(),
            "super_secret_token".into(),
            "https://example.com/binary".into(),
        ];

        let event = Event::builder(EventType::ProcessExec, "/usr/bin/curl")
            .pid(1234)
            .ppid(Some(1000))
            .uids(1000, 0)
            .arguments(&raw_args)
            .working_directory("/home/user")
            .backend(MonitoringBackendType::Procfs)
            .build();

        assert_eq!(event.pid, 1234);
        assert_eq!(event.ppid, Some(1000));
        assert!(event.is_privileged());
        assert_eq!(event.arguments[4], "[REDACTED]");
        assert_eq!(event.confidence, CollectionConfidence::High);
        assert_eq!(event.observation_kind, ObservationKind::Direct);
    }
}
