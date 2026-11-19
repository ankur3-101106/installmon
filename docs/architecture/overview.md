# Architecture Overview: InstallMon

## 1. System Vision

InstallMon observes how software enters, gets installed, and changes within a Linux environment. Modern Linux hosts accumulate software through many disconnected vectors:
- Traditional package managers (`apt`, `pacman`, `dnf`, `apk`)
- Language package ecosystems (`cargo`, `pip`, `npm`, `uv`, `go install`, `gem`)
- Direct binary downloads (`curl ... | sh`, `wget`, direct executable placement)
- Source code compilation (`make`, `cmake`, `ninja`, `meson`)
- Unpacked archives (`tar -xzf`, `unzip`, `7z`)
- System & user binary directories (`/usr/local/bin`, `~/.local/bin`, `/opt`)

InstallMon observes this execution and filesystem mutation flow continuously and non-invasively, providing provenance tracking without requiring invasive kernel patches or wrapping package managers.

```mermaid
flowchart TD
    subgraph S1["Host Monitoring Backends"]
        direction TB
        B1["procfs Poller (Active)<br/>• Periodic /proc scans<br/>• Installer classification"]
        B2["inotify Monitor (Active)<br/>• Watches bin & config dirs<br/>• File creations & modifications"]
        B3["Kernel BTF / eBPF (Probed)<br/>• Readiness detection<br/>• Planned tracepoints"]
        B4["Linux Audit / fanotify (Probed)<br/>• System probe detection<br/>• Planned event streams"]
    end

    subgraph S2["Normalization & Security Ingest"]
        direction TB
        N1["Normalized Event Model<br/>(installmon-core)"]
        N2["Credential & Argument Sanitizer<br/>• Redacts passwords, tokens, API keys<br/>• Scrubbed URL credentials & headers"]
    end

    subgraph S3["Correlation & Session Engine"]
        direction TB
        C1["Session Lifecycle Manager<br/>• Classifies known vs inferred workflows<br/>• Maps process roots & EUID scope"]
        C2["Binary Discovery Engine<br/>• Automatic PATH executable registration"]
    end

    subgraph S4["Local Storage Layer"]
        direction TB
        D1[("SQLite Database (WAL Mode)<br/>• Schema migrations (v1)<br/>• PRAGMA integrity_check<br/>• Foreign keys & indexes")]
    end

    subgraph S5["User Interfaces"]
        direction TB
        UI1["CLI: installmon<br/>• status, capabilities, scan, list<br/>• inspect, history, sessions, events<br/>• audit, export, daemon, service"]
        UI2["TUI: Ratatui Interface<br/>• Dashboard, Software, Sessions<br/>• Events, Audit, Service views"]
    end

    B1 & B2 --> N1
    N1 --> N2
    N2 --> C1 & C2
    C1 & C2 --> D1
    D1 --> UI1 & UI2
```

## 2. Component Structure

### `installmon-core`
- **Normalized Event Model**: Strongly-typed structures for process execution, lifecycle mutations, downloads, and installation states.
- **Sensitive Argument Sanitizer**: Regex and token-aware scrubber ensuring passwords, tokens, API keys, HTTP authorization headers, and private keys are never persisted or logged.
- **Provenance & Evidence**: Models confidence levels (`High`, `Medium`, `Low`), scopes (`User`, `System`, `Hybrid`), and evidence chains.

### `installmon-platform`
- **Capability Detection**: Probes host kernel, distribution, libc, CPU architecture, virtualization (WSL2, Docker, LXC, Podman), and available kernel facilities (`/sys/kernel/btf/vmlinux`, `/proc/sys/fs/inotify`, `/proc/net/netlink`).
- **Path Resolution**: Provides standard XDG user paths and `/var/lib/installmon` system paths with automatic `/tmp` fallback for restricted container environments.
- **Service Management**: Controls background daemon lifecycle across systemd user units, systemd system units, and standalone detached background processes.

### `installmon-database`
- **SQLite Engine**: Embedded, zero-configuration local persistence in WAL mode with transactional batch commits.
- **Schema Migrations**: Versioned database schemas for events, sessions, processes, software, installations, artifacts, sources, evidence, and capabilities.
- **Integrity Verification**: Built-in verification via `PRAGMA integrity_check`.

### `installmon-tui`
- **Terminal User Interface**: Built on Ratatui 0.29 and Crossterm 0.28.
- **Navigation & Inspection**: 6 interactive views: Dashboard, Software, Sessions, Events, Audit, and Service.
- **Centralized Control**: Setup, toggle, restart, and inspect the background daemon directly from the terminal.

### `installmon-cli`
- **Binary**: `installmon`.
- **Command Set**: `tui`, `status`, `capabilities`, `list`, `scan`, `inspect`, `history`, `sessions`, `events`, `audit`, `export`, `daemon`, and `service`.
- **Machine Readability**: Standardized `--json` output across commands.
