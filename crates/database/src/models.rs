use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

use installmon_core::capabilities::PlatformCapabilities;
use installmon_core::event::CollectionConfidence;
use installmon_core::provenance::{InstallationScope, WorkflowClassification};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionRecord {
    pub id: Uuid,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub status: String,
    pub scope: InstallationScope,
    pub root_pid: Option<u32>,
    pub initiating_command: Option<String>,
    pub classification: WorkflowClassification,
    pub confidence: CollectionConfidence,
    pub installer_identity: Option<String>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcessRecord {
    pub id: Uuid,
    pub session_id: Option<Uuid>,
    pub pid: u32,
    pub ppid: Option<u32>,
    pub uid: u32,
    pub euid: u32,
    pub executable_path: PathBuf,
    pub arguments: Vec<String>,
    pub working_directory: Option<PathBuf>,
    pub started_at: DateTime<Utc>,
    pub exited_at: Option<DateTime<Utc>>,
    pub exit_code: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SoftwareRecord {
    pub id: Uuid,
    pub name: String,
    pub display_name: Option<String>,
    pub primary_binary_path: Option<PathBuf>,
    pub scope: InstallationScope,
    pub discovered_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstallationRecord {
    pub id: Uuid,
    pub software_id: Uuid,
    pub session_id: Uuid,
    pub installed_at: DateTime<Utc>,
    pub version: Option<String>,
    pub install_prefix: Option<PathBuf>,
    pub installer: Option<String>,
    pub classification: WorkflowClassification,
    pub confidence: CollectionConfidence,
    pub scope: InstallationScope,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactRecord {
    pub id: Uuid,
    pub installation_id: Option<Uuid>,
    pub session_id: Uuid,
    pub path: PathBuf,
    pub file_type: String,
    pub sha256: Option<String>,
    pub permissions: Option<u32>,
    pub size_bytes: Option<u64>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceRecord {
    pub id: Uuid,
    pub session_id: Uuid,
    pub source_type: String,
    pub url: Option<String>,
    pub git_commit: Option<String>,
    pub git_branch: Option<String>,
    pub checksum: Option<String>,
    pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceRecord {
    pub id: Uuid,
    pub session_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub category: String,
    pub description: String,
    pub raw_data: serde_json::Value,
    pub confidence: CollectionConfidence,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilitiesRecord {
    pub id: Uuid,
    pub probed_at: DateTime<Utc>,
    pub kernel_release: String,
    pub architecture: String,
    pub distribution: String,
    pub init_system: String,
    pub environment: String,
    pub active_process_backend: String,
    pub active_filesystem_backend: String,
    pub capabilities: PlatformCapabilities,
}
