# Platform Compatibility Matrix

InstallMon detects the host platform at runtime and negotiates monitoring backends dynamically.

| Environment | Process Monitoring Backend | Filesystem Monitoring Backend | Status |
| :--- | :--- | :--- | :--- |
| **Linux (Bare-Metal Root)** | eBPF / BTF (primary) or Linux Audit | fanotify (primary) or inotify | Fully supported |
| **Linux (Bare-Metal User)** | procfs fallback | inotify | Fully supported (unprivileged) |
| **WSL2 (Windows Subsystem for Linux)** | procfs fallback (or eBPF if privileged) | inotify | Tested & Verified |
| **Docker / Podman Container** | procfs fallback | inotify | Tested & Verified |
| **Arch Linux / Debian / Ubuntu / Fedora** | Auto-negotiated | Auto-negotiated | Tested & Verified |
| **glibc & musl (Alpine)** | Auto-negotiated | Auto-negotiated | Supported |

### Graceful Degradation Strategy

1. **Process Activity**:
   - `eBPF` is preferred when kernel BTF metadata (`/sys/kernel/btf/vmlinux`) exists and the process has root / `CAP_SYS_ADMIN`.
   - `Linux Audit` is preferred when the netlink audit subsystem is active.
   - `procfs` fallback inspection is used whenever unprivileged or in restricted container/WSL environments.
2. **Filesystem Activity**:
   - `fanotify` is preferred for system-wide mount interception when running with root privileges.
   - `inotify` is used for user installation directories and unprivileged environments.
