# Project Roadmap: InstallMon

[InstallMon](../README.md) > [Documentation Hub](README.md) > Project Roadmap

This document outlines the evolutionary direction of InstallMon, clearly separating **implemented and stable capabilities** from **planned future enhancements**.

---

## 🟢 Implemented & Stable (Current Release: v0.1.x)

The following components are implemented, fully tested, and actively utilized:

- [x] **Normalized Event Schema**: Strongly typed event models (`process_exec`, `file_create`, `file_modify`, `file_delete`, `file_rename`, build/installation phases).
- [x] **Ingest-Time Argument Sanitizer**: Regex and token-aware credential scrubber that redacts flags (`--password`, `--token`, `--api-key`), inline secrets (`--password=SECRET`), HTTP Authorization headers, URL credentials, and PEM private keys.
- [x] **Inotify Directory Monitoring**: Active filesystem mutation observation on system and user binary/configuration paths.
- [x] **Procfs Process Polling**: Periodic 800ms `/proc` scanning to observe installer commands (`apt`, `cargo`, `pip`, `make`, `curl`, `git clone`, `tar`).
- [x] **Session Correlation & Classification**: Automatic grouping of processes into installation sessions with workflow pattern classification (`distribution_package`, `language_ecosystem`, `direct_download`, `source_build`, `script_execution`).
- [x] **Embedded SQLite Persistence**: Zero-configuration WAL mode database with versioned schema migrations (`schema_migrations`), foreign key constraints, and 5000ms busy timeout handling.
- [x] **12-Subcommand CLI**: Full suite of querying, scanning, auditing, and export commands with `--json` output across all commands.
- [x] **Interactive Ratatui TUI**: Single-pane-of-glass terminal interface featuring 6 tabs: Dashboard, Software, Sessions, Events, Audit, and Service.
- [x] **Autonomous Daemon Service**: Background monitoring supporting `systemd --user`, systemd system units, and standalone detached process mode.
- [x] **Multi-Architecture Release Pipeline**: Automated GitHub Actions CI/CD building static `musl` binaries for `x86_64` and `aarch64` with SHA-256 checksums.

---

## 🟡 Planned Future Enhancements

The following capabilities are actively planned or currently probed at runtime:

### 1. Kernel eBPF / BTF Tracepoint Engine
- **Current State**: Probed (the platform detector inspects `/sys/kernel/btf/vmlinux`).
- **Goal**: Ingest `sched_process_exec` and `sched_process_exit` tracepoints via an embedded eBPF probe.
- **Benefit**: Eliminates the 800ms polling gap, capturing short-lived transient compiler commands in real-time with sub-millisecond precision.

### 2. Linux Audit Netlink Event Streaming
- **Current State**: Probed (the platform detector probes netlink socket availability).
- **Goal**: Stream kernel audit events (`kauditd`) over Netlink sockets as an alternative to eBPF on older kernels lacking BTF metadata.
- **Benefit**: Provides zero-polling process auditing on enterprise kernels (RHEL 7/8, Debian 10).

### 3. fanotify System-Wide Mount Interception
- **Current State**: Probed (the platform detector validates `fanotify_init` syscall readiness).
- **Goal**: Intercept filesystem modifications across entire mount points (e.g. `/usr`, `/opt`) when running with `CAP_SYS_ADMIN` privileges.
- **Benefit**: Replaces manual directory inotify watches with system-wide filesystem interception.

### 4. Recursive Inotify Directory Tree Tracking
- **Current State**: Configured root directory watching.
- **Goal**: Automatically register child subdirectories dynamically when new directories are created inside watched trees.

### 5. Automated Binary SHA-256 Checksumming
- **Current State**: Binary paths, sizes, and timestamps are recorded.
- **Goal**: Asynchronously compute cryptographic SHA-256 hashes of newly registered executable binaries in the background for integrity auditing.

---

## Contributing to the Roadmap

Interested in implementing one of these planned enhancements? Please review:
- 🏗️ [Architecture Overview](architecture/overview.md) — To understand subsystem interfaces.
- 🚀 [Development & Contributing Guide](development/contributing.md) — For development workflow instructions.

---

[← Back to Documentation Hub](README.md) | [← Back to Main README](../README.md)
