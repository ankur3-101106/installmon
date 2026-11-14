use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Init system present on the host.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InitSystemKind {
    Systemd,
    OpenRC,
    Runit,
    SysVInit,
    S6,
    ContainerInit,
    Unknown,
}

impl fmt::Display for InitSystemKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Systemd => write!(f, "systemd"),
            Self::OpenRC => write!(f, "openrc"),
            Self::Runit => write!(f, "runit"),
            Self::SysVInit => write!(f, "sysvinit"),
            Self::S6 => write!(f, "s6"),
            Self::ContainerInit => write!(f, "container-init"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

/// Execution environment kind (host, virtualized, containerized).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnvironmentKind {
    BareMetal,
    WSL2,
    Docker,
    Podman,
    LXC,
    KVM,
    UnknownVirtual,
}

impl fmt::Display for EnvironmentKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BareMetal => write!(f, "bare_metal"),
            Self::WSL2 => write!(f, "wsl2"),
            Self::Docker => write!(f, "docker"),
            Self::Podman => write!(f, "podman"),
            Self::LXC => write!(f, "lxc"),
            Self::KVM => write!(f, "kvm"),
            Self::UnknownVirtual => write!(f, "virtual"),
        }
    }
}

/// Standard C Library implementation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LibcKind {
    Glibc,
    Musl,
    Unknown,
}

impl fmt::Display for LibcKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Glibc => write!(f, "glibc"),
            Self::Musl => write!(f, "musl"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

/// Availability status of a monitoring backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityStatus {
    Available,
    PartiallyAvailable,
    Unavailable,
    RequiresPrivilege,
}

impl fmt::Display for CapabilityStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Available => write!(f, "available"),
            Self::PartiallyAvailable => write!(f, "partially_available"),
            Self::Unavailable => write!(f, "unavailable"),
            Self::RequiresPrivilege => write!(f, "requires_privilege"),
        }
    }
}

/// Details of a probed monitoring technology backend.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackendCapability {
    pub name: String,
    pub status: CapabilityStatus,
    pub description: String,
    pub details: Option<String>,
}

/// Full runtime platform inspection snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlatformCapabilities {
    pub probed_at: DateTime<Utc>,
    pub kernel_release: String,
    pub kernel_version: String,
    pub architecture: String,
    pub distribution_id: String,
    pub distribution_name: String,
    pub distribution_version: Option<String>,
    pub init_system: InitSystemKind,
    pub environment: EnvironmentKind,
    pub libc: LibcKind,
    pub is_root: bool,
    pub btf_supported: bool,
    pub inotify_supported: bool,
    pub fanotify_supported: bool,
    pub audit_supported: bool,
    pub procfs_supported: bool,
    pub active_process_backend: String,
    pub active_filesystem_backend: String,
    pub backends: Vec<BackendCapability>,
}
