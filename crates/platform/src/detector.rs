use chrono::Utc;
use nix::sys::utsname::uname;
use std::fs;
use std::path::Path;

use installmon_core::capabilities::{
    BackendCapability, CapabilityStatus, EnvironmentKind, InitSystemKind, LibcKind,
    PlatformCapabilities,
};

/// Collect a comprehensive snapshot of platform capabilities, operating system identity,
/// init system, container/virtualization status, and available monitoring backends.
pub fn detect_platform_capabilities() -> PlatformCapabilities {
    let uts = uname().expect("Failed to retrieve uname");
    let kernel_release = uts.release().to_string_lossy().to_string();
    let kernel_version = uts.version().to_string_lossy().to_string();
    let architecture = uts.machine().to_string_lossy().to_string();

    let (distro_id, distro_name, distro_version) = detect_os_release();
    let init_system = detect_init_system();
    let environment = detect_environment(&kernel_release, &kernel_version);
    let libc_kind = detect_libc();
    let is_root = unsafe { libc::geteuid() == 0 };

    let btf_supported = Path::new("/sys/kernel/btf/vmlinux").exists();
    let inotify_supported = Path::new("/proc/sys/fs/inotify").exists();
    let procfs_supported = Path::new("/proc/self/status").exists();
    let audit_supported = Path::new("/proc/net/netlink").exists();

    // Fanotify requires root/CAP_SYS_ADMIN on most kernels, but let's probe availability
    let fanotify_supported = probe_fanotify_support(is_root);

    let mut backends = Vec::new();

    // 1. eBPF Process Monitoring Backend
    backends.push(BackendCapability {
        name: "eBPF".to_string(),
        status: if btf_supported && is_root {
            CapabilityStatus::Available
        } else if btf_supported && !is_root {
            CapabilityStatus::RequiresPrivilege
        } else {
            CapabilityStatus::Unavailable
        },
        description: "Kernel tracepoint/kprobe process monitoring via eBPF/BTF".to_string(),
        details: if btf_supported {
            Some("BTF vmlinux detected in /sys/kernel/btf/vmlinux".to_string())
        } else {
            Some("Kernel BTF metadata not found in /sys/kernel/btf".to_string())
        },
    });

    // 2. Linux Audit Backend
    backends.push(BackendCapability {
        name: "audit".to_string(),
        status: if audit_supported && is_root {
            CapabilityStatus::Available
        } else if audit_supported && !is_root {
            CapabilityStatus::RequiresPrivilege
        } else {
            CapabilityStatus::Unavailable
        },
        description: "Linux Audit subsystem via netlink sockets".to_string(),
        details: Some("Auditing system activity and process executions".to_string()),
    });

    // 3. Procfs Process Monitoring Backend (Universal Fallback)
    backends.push(BackendCapability {
        name: "procfs".to_string(),
        status: if procfs_supported {
            CapabilityStatus::Available
        } else {
            CapabilityStatus::Unavailable
        },
        description: "Process inspection and polling via /proc filesystem (unprivileged fallback)"
            .to_string(),
        details: Some("Lightweight polling / fallback process tracking".to_string()),
    });

    // 4. Fanotify Filesystem Backend
    backends.push(BackendCapability {
        name: "fanotify".to_string(),
        status: if fanotify_supported {
            CapabilityStatus::Available
        } else if is_root {
            CapabilityStatus::PartiallyAvailable
        } else {
            CapabilityStatus::RequiresPrivilege
        },
        description: "High-performance system-wide filesystem activity monitoring".to_string(),
        details: if is_root {
            Some("Privileged filesystem notifications enabled".to_string())
        } else {
            Some("Requires CAP_SYS_ADMIN for system-wide fanotify".to_string())
        },
    });

    // 5. Inotify Filesystem Backend
    backends.push(BackendCapability {
        name: "inotify".to_string(),
        status: if inotify_supported {
            CapabilityStatus::Available
        } else {
            CapabilityStatus::Unavailable
        },
        description: "Directory and file mutation monitoring via Linux inotify".to_string(),
        details: Some(
            "Supports user-level and system directory trees without root requirement".to_string(),
        ),
    });

    // Determine active backends by negotiation
    let active_process_backend = if btf_supported && is_root {
        "ebpf".to_string()
    } else if audit_supported && is_root {
        "audit".to_string()
    } else {
        "procfs".to_string()
    };

    let active_filesystem_backend = if fanotify_supported && is_root {
        "fanotify".to_string()
    } else if inotify_supported {
        "inotify".to_string()
    } else {
        "none".to_string()
    };

    PlatformCapabilities {
        probed_at: Utc::now(),
        kernel_release,
        kernel_version,
        architecture,
        distribution_id: distro_id,
        distribution_name: distro_name,
        distribution_version: distro_version,
        init_system,
        environment,
        libc: libc_kind,
        is_root,
        btf_supported,
        inotify_supported,
        fanotify_supported,
        audit_supported,
        procfs_supported,
        active_process_backend,
        active_filesystem_backend,
        backends,
    }
}

/// Detect OS distribution identity by parsing standard `/etc/os-release` or `/usr/lib/os-release`.
fn detect_os_release() -> (String, String, Option<String>) {
    let paths = ["/etc/os-release", "/usr/lib/os-release"];
    for p in paths {
        if let Ok(content) = fs::read_to_string(p) {
            let mut id = "unknown".to_string();
            let mut name = "Linux".to_string();
            let mut version = None;

            for line in content.lines() {
                let line = line.trim();
                if let Some(val) = line.strip_prefix("ID=") {
                    id = val.trim_matches('"').to_string();
                } else if let Some(val) = line.strip_prefix("NAME=") {
                    name = val.trim_matches('"').to_string();
                } else if let Some(val) = line.strip_prefix("VERSION_ID=") {
                    version = Some(val.trim_matches('"').to_string());
                }
            }

            return (id, name, version);
        }
    }

    ("unknown".to_string(), "Linux".to_string(), None)
}

/// Detect the active init system by checking runtime indicators and PID 1.
fn detect_init_system() -> InitSystemKind {
    if Path::new("/run/systemd/system").exists() {
        return InitSystemKind::Systemd;
    }
    if Path::new("/run/openrc").exists() {
        return InitSystemKind::OpenRC;
    }
    if Path::new("/run/runit").exists() || Path::new("/etc/runit").exists() {
        return InitSystemKind::Runit;
    }

    // Inspect /proc/1/comm
    if let Ok(comm) = fs::read_to_string("/proc/1/comm") {
        let comm = comm.trim();
        match comm {
            "systemd" => return InitSystemKind::Systemd,
            "init" => return InitSystemKind::SysVInit,
            "openrc-init" => return InitSystemKind::OpenRC,
            "runit" => return InitSystemKind::Runit,
            "s6-svscan" => return InitSystemKind::S6,
            _ => {}
        }
    }

    InitSystemKind::Unknown
}

/// Detect virtualization or container environment.
fn detect_environment(kernel_release: &str, kernel_version: &str) -> EnvironmentKind {
    if std::env::var("WSL_DISTRO_NAME").is_ok()
        || kernel_release.to_lowercase().contains("microsoft")
        || kernel_version.to_lowercase().contains("microsoft")
    {
        return EnvironmentKind::WSL2;
    }

    if Path::new("/.dockerenv").exists() {
        return EnvironmentKind::Docker;
    }

    if Path::new("/run/.containerenv").exists() {
        return EnvironmentKind::Podman;
    }

    if let Ok(cgroups) = fs::read_to_string("/proc/1/cgroup") {
        if cgroups.contains("docker") {
            return EnvironmentKind::Docker;
        }
        if cgroups.contains("libpod") {
            return EnvironmentKind::Podman;
        }
        if cgroups.contains("lxc") {
            return EnvironmentKind::LXC;
        }
    }

    if let Ok(environ) = fs::read_to_string("/proc/1/environ") {
        if environ.contains("container=lxc") {
            return EnvironmentKind::LXC;
        }
    }

    EnvironmentKind::BareMetal
}

/// Detect the active standard C library implementation.
fn detect_libc() -> LibcKind {
    if let Ok(maps) = fs::read_to_string("/proc/self/maps") {
        if maps.contains("musl") {
            return LibcKind::Musl;
        }
        if maps.contains("libc.so") || maps.contains("libc-") {
            return LibcKind::Glibc;
        }
    }
    LibcKind::Unknown
}

/// Probes whether fanotify group creation is supported in this context.
fn probe_fanotify_support(is_root: bool) -> bool {
    if !is_root {
        return false;
    }
    // Probe fanotify syscall existence
    #[cfg(target_os = "linux")]
    {
        unsafe {
            // FAN_CLASS_NOTIF = 0x00000000, O_RDONLY = 0
            let fd = libc::syscall(libc::SYS_fanotify_init, 0, 0);
            if fd >= 0 {
                libc::close(fd as i32);
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_platform_capabilities() {
        let caps = detect_platform_capabilities();
        assert!(!caps.kernel_release.is_empty());
        assert!(!caps.architecture.is_empty());
        assert!(!caps.distribution_name.is_empty());
        assert!(!caps.backends.is_empty());
    }

    #[test]
    fn test_detect_os_release() {
        let (id, name, _) = detect_os_release();
        assert!(!id.is_empty());
        assert!(!name.is_empty());
    }
}
