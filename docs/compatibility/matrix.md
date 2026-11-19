# Platform Compatibility Matrix

InstallMon detects the host platform at runtime and probes monitoring backend availability.

## Active vs Detected Capabilities

| Backend Subsystem | Role | Implementation Status | Notes |
| :--- | :--- | :--- | :--- |
| **procfs (`/proc`)** | Process Activity | **Implemented & Active** | Periodic polling (800ms) of `/proc` for installer commands, package managers, and builds |
| **inotify** | Filesystem Mutations | **Implemented & Active** | Event-driven watching on configured binary and config directories (`IN_CREATE`, `IN_MOVED_TO`, etc.) |
| **eBPF / BTF** | Process Activity | Probed (Detected) | Probes `/sys/kernel/btf/vmlinux`; kernel tracing engine planned for future release |
| **Linux Audit Subsystem** | Process Activity | Probed (Detected) | Probes netlink socket availability; netlink audit event streaming planned |
| **fanotify** | System-wide Filesystem | Probed (Detected) | Syscall availability verified via `fanotify_init`; mount interception planned |

## Environment Support

| Environment | Active Process Backend | Active Filesystem Backend | Status |
| :--- | :--- | :--- | :--- |
| **Linux (Bare-Metal Root)** | procfs polling | inotify | Supported (system scope) |
| **Linux (Bare-Metal User)** | procfs polling | inotify | Supported (unprivileged user scope) |
| **WSL2 (Windows Subsystem for Linux)** | procfs polling | inotify | Tested & Supported |
| **Docker / Podman Containers** | procfs polling | inotify | Supported (requires `/proc` access) |
| **glibc (Debian, Ubuntu, Fedora, Arch)** | procfs polling | inotify | Supported |
| **musl (Alpine, Void Linux)** | procfs polling | inotify | Supported via static target |
