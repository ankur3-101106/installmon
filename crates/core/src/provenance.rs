use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

use crate::event::CollectionConfidence;

/// Classification state for observed installation activity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowClassification {
    /// Deterministically identified through known toolchains / patterns.
    KnownWorkflow,
    /// Correlated through heuristics and process-filesystem relationships.
    InferredWorkflow,
    /// Unrecognized installation or manual deployment requiring preservation.
    UnknownWorkflow,
}

impl std::fmt::Display for WorkflowClassification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::KnownWorkflow => write!(f, "known_workflow"),
            Self::InferredWorkflow => write!(f, "inferred_workflow"),
            Self::UnknownWorkflow => write!(f, "unknown_workflow"),
        }
    }
}

/// The privilege scope under which an installation was performed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstallationScope {
    /// User-level installation (e.g. ~/.local/bin, ~/.cargo/bin, ~/bin).
    User,
    /// System-wide privileged installation (e.g. /usr/local/bin, /opt, /etc).
    System,
    /// Hybrid or transitional privilege scope.
    Hybrid,
}

impl std::fmt::Display for InstallationScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::User => write!(f, "user"),
            Self::System => write!(f, "system"),
            Self::Hybrid => write!(f, "hybrid"),
        }
    }
}

/// Individual item of preserved supporting evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceItem {
    pub id: Uuid,
    pub session_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub category: String,
    pub description: String,
    pub raw_data: serde_json::Value,
    pub confidence: CollectionConfidence,
}

impl EvidenceItem {
    pub fn new(
        session_id: Uuid,
        category: impl Into<String>,
        description: impl Into<String>,
        raw_data: serde_json::Value,
        confidence: CollectionConfidence,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            session_id,
            timestamp: Utc::now(),
            category: category.into(),
            description: description.into(),
            raw_data,
            confidence,
        }
    }
}

/// Installation provenance record capturing verified or inferred origins.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstallationProvenance {
    pub id: Uuid,
    pub session_id: Uuid,
    pub software_id: Option<Uuid>,
    pub classification: WorkflowClassification,
    pub confidence: CollectionConfidence,
    pub scope: InstallationScope,

    /// Verified or inferred source download URL.
    pub download_url: Option<String>,
    /// Git repository URL.
    pub repository_url: Option<String>,
    /// Git commit SHA or tag.
    pub git_commit: Option<String>,
    /// Software release or version string.
    pub release_version: Option<String>,
    /// Tool or package manager responsible (e.g. cargo, pip, curl+tar, make install).
    pub installer_identity: Option<String>,
    /// Classification of command: "curl_to_bin", "git_clone_and_make", "cargo_install", etc.
    pub command_classification: Option<String>,
    /// Build system used (e.g. make, cmake, meson, cargo).
    pub build_system: Option<String>,
    /// Primary installation prefix or destination.
    pub installation_destination: Option<PathBuf>,

    /// Preserved supporting evidence items.
    pub evidence: Vec<EvidenceItem>,
}

impl InstallationProvenance {
    pub fn new(
        session_id: Uuid,
        classification: WorkflowClassification,
        confidence: CollectionConfidence,
        scope: InstallationScope,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            session_id,
            software_id: None,
            classification,
            confidence,
            scope,
            download_url: None,
            repository_url: None,
            git_commit: None,
            release_version: None,
            installer_identity: None,
            command_classification: None,
            build_system: None,
            installation_destination: None,
            evidence: Vec::new(),
        }
    }

    pub fn add_evidence(&mut self, evidence: EvidenceItem) {
        self.evidence.push(evidence);
    }
}
