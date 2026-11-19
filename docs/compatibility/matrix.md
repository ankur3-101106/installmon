# Platform Compatibility Matrix: InstallMon

[InstallMon](../../README.md) > [Documentation Hub](../README.md) > Compatibility Matrix

InstallMon detects the host platform at runtime and probes monitoring backend availability. This document details verified architectures, kernel requirements, and active vs probed capabilities.

---

## 1. Supported Platforms & Architectures

| Architecture | Target Triple | Binary Format | Verification Status |
| :--- | :--- | :--- | :--- |
| **x86_64** (Intel / AMD 64-bit) | `x86_64-unknown-linux-gnu` | Dynamically linked ELF (glibc) | Verified & Tested |
| **x86_64** (Universal Static) | `x86_64-unknown-linux-musl` | Statically linked ELF (musl) | Verified & Tested |
| **AArch64** (ARM 64-bit) | `aarch64-unknown-linux-gnu` | Dynamically linked ELF (glibc) | Verified & Tested |
| **AArch64** (ARM 64-bit Static) | `aarch64-unknown-linux-musl` | Statically linked ELF (musl) | Verified & Tested |

---

## 2. Active vs Detected Capabilities

InstallMon strictly distinguishes between **implemented, active features** and **detected host capabilities**:

| Backend Subsystem | Role | Implementation Status | Notes |
| :--- | :--- | :--- | :--- |
| **procfs (`/proc`)** | Process Activity | **Implemented & Active** | Periodic polling (800ms) of `/proc` for installer commands, package managers, and builds. |
| **inotify** | Filesystem Mutations | **Implemented & Active** | Event-driven directory watching on configured binary and config paths (`IN_CREATE`, `IN_MOVED_TO`, etc.). |
| **eBPF / BTF** | Process Activity | Probed (Detected) | Probes `/sys/kernel/btf/vmlinux`; kernel tracepoint engine planned for future release. |
| **Linux Audit Subsystem** | Process Activity | Probed (Detected) | Probes netlink socket availability; netlink audit event streaming planned. |
| **fanotify** | System-wide Filesystem | Probed (Detected) | Syscall availability verified via `fanotify_init`; mount interception planned. |

---

## 3. Environment Compatibility

| Environment | Active Process Backend | Active Filesystem Backend | Status |
| :--- | :--- | :--- | :--- |
| **Linux (Bare-Metal Root)** | procfs polling | inotify | Supported (system scope) |
| **Linux (Bare-Metal User)** | procfs polling | inotify | Supported (unprivileged user scope) |
| **WSL2 (Windows Subsystem for Linux)** | procfs polling | inotify | Tested & Supported |
| **Docker / Podman Containers** | procfs polling | inotify | Supported (requires `/proc` access) |
| **glibc (Debian, Ubuntu, Fedora, Arch)** | procfs polling | inotify | Supported |
| **musl (Alpine, Void Linux)** | procfs polling | inotify | Supported via static target |

---

## 4. System Requirements

- **Operating System**: Linux kernel 4.4 or higher (5.x / 6.x recommended).
- **Rust Toolchain**: Rust 1.80.0 or higher (MSRV) for building from source (utilizes `std::sync::LazyLock`).
- **C Library**: GNU C Library (`glibc` 2.17+) or Musl libc.
- **Kernel Interfaces**: `/proc` filesystem mounted and readable; `/proc/sys/fs/inotify` support enabled.
- **Privileges**:
  - **Unprivileged (User Mode)**: Fully functional for user directories (`~/.local/bin`, `~/.cargo/bin`), user systemd services, and user SQLite database.
  - **Privileged (Root / sudo)**: Required for system directories (`/usr/bin`, `/usr/local/bin`, `/opt`), system-wide systemd units, and `/var/lib/installmon`.

---

## Related Documentation & Next Steps

- 📦 [Installation & Lifecycle Guide](../installation/guide.md) — Prebuilt archives and installation commands.
- 🏗️ [Architecture Overview](../architecture/overview.md) — How backends interact with event normalization.
- 🛠️ [Troubleshooting & FAQ](../troubleshooting/faq.md) — Diagnosing missing capabilities or permissions.

---

[← Back to Documentation Hub](../README.md) | [← Back to Main README](../../README.md)
