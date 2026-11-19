# InstallMon

[![License: GPL-3.0-only](https://img.shields.io/badge/License-GPL--3.0--only-blue.svg)](LICENSE)
[![MSRV](https://img.shields.io/badge/MSRV-1.80.0%2B-orange.svg)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/Platform-Linux-brightgreen.svg)]()
[![Architecture](https://img.shields.io/badge/Architecture-x86__64%20%7C%20aarch64-blueviolet.svg)]()

> **Linux Software Installation Monitoring and Provenance Tracking.**

InstallMon observes how software enters, gets installed, and changes within a Linux host. It records installation workflows, reconstructs execution history, identifies installed software binaries, and maintains provenance records without wrapping package managers or modifying system execution flows.

---

## Table of Contents

- [A. Project Introduction](#a-project-introduction)
- [B. Features](#b-features)
- [C. How It Works](#c-how-it-works)
- [D. Architecture](#d-architecture)
- [E. Requirements](#e-requirements)
- [F. Installation](#f-installation)
- [G. Quick Start](#g-quick-start)
- [H. CLI Reference](#h-cli-reference)
- [I. TUI Documentation](#i-tui-documentation)
- [J. Database and Data Storage](#j-database-and-data-storage)
- [K. Security and Privacy](#k-security-and-privacy)
- [L. Limitations](#l-limitations)
- [M. Troubleshooting](#m-troubleshooting)
- [N. Development](#n-development)
- [O. Roadmap](#o-roadmap)
- [P. License and Acknowledgements](#p-license-and-acknowledgements)

---

## A. Project Introduction

Modern Linux systems accumulate software through many disparate mechanisms:

- System package managers (`apt`, `pacman`, `dnf`, `apk`).
- Language ecosystem installers (`cargo`, `pip`, `npm`, `uv`, `go install`, `gem`).
- Direct binary downloads (`curl ... | sh`, `wget`, direct executable placement).
- Source compilation (`make`, `cmake`, `ninja`, `meson`).
- Unpacked archives (`tar -xzf`, `unzip`, `7z`).
- Ad-hoc scripts executed in user or system directories (`~/.local/bin`, `/usr/local/bin`).

Most distributions track only software installed through their native package manager. Software installed via language package managers, compiled from source, or downloaded directly remains invisible to system audits, creating configuration drift, blind spots, and untracked binaries.

### What InstallMon Solves

InstallMon continuously observes system installation activity and maintains an offline, queryable provenance catalog:

- **Observes**: Monitors filesystem mutations in binary and configuration directories and identifies installer processes in `/proc`.
- **Sanitizes**: Scrubs sensitive tokens, passwords, private keys, and authorization headers before records touch disk.
- **Correlates**: Links initiating commands, process ancestry (PID/PPID/UID/EUID), and filesystem changes to installation sessions.
- **Audits**: Identifies missing executables, anomalous installations, and unclassified deployment workflows.

### What InstallMon Is NOT

- **Not a package manager**: It does not replace or wrap `apt`, `pacman`, `dnf`, or `cargo`.
- **Not an installer or dependency solver**: It does not download or compile software on its own.
- **Not an enforcement or blocking tool**: It operates non-invasively as a read-only observer.

---

## B. Features

InstallMon strictly distinguishes between **implemented, active features** and **detected host capabilities**:

### Monitoring (Implemented & Active)
- **Continuous Background Daemon**: Background observation service (`installmon daemon`) tracking filesystem mutations and process activity.
- **Inotify Directory Monitoring**: Real-time event notifications (`IN_CREATE`, `IN_MOVED_TO`, `IN_MODIFY`, `IN_ATTRIB`) on configured directories.
- **Process Activity Polling**: Periodic scanning of `/proc` (800ms interval) to identify active installer executions.

### Software Discovery (Implemented & Active)
- **PATH & Directory Scanner**: Fast binary discovery (`installmon scan`) across system and user executable paths (`/usr/local/bin`, `~/.local/bin`, `~/.cargo/bin`).
- **Automatic Registration**: The background daemon automatically detects new executable files placed in watched `bin` directories and indexes them in the database.

### Installation History & Provenance (Implemented & Active)
- **Session Tracking**: Groups related installation events into sessions with start/completion timestamps, root PID, and user scope (`User`, `System`, `Hybrid`).
- **Workflow Classification**: Categorizes installations into known patterns (`distribution_package`, `language_ecosystem`, `direct_download`, `source_build`, `git_repository`, `archive_extraction`, `binary_deployment`, `script_execution`) or inferred/unknown patterns.
- **Evidence Storage**: Preserves supporting evidence items linked to specific installation sessions.

### Filesystem & Process Tracking (Implemented & Active)
- **Normalized Lifecycle Events**: Strongly typed event schema covering `process_exec`, `file_create`, `file_modify`, `file_delete`, `file_rename`, and build/installation lifecycles.
- **Process Context**: Captures PID, PPID, UID, EUID, working directory, executable path, and sanitized arguments.

### Database & Persistence (Implemented & Active)
- **Embedded SQLite Engine**: Zero-configuration, local persistence utilizing Write-Ahead Logging (`WAL` mode).
- **Automated Schema Migrations**: Versioned database migration management (`schema_migrations` table).
- **Database Integrity**: Automated verification using `PRAGMA integrity_check`.
- **Flexible Data Paths**: Supports standard XDG user paths and system-wide `/var/lib/installmon` paths with fallback for restricted container environments.

### CLI & Machine Readability (Implemented & Active)
- **Comprehensive Command Set**: 12 dedicated subcommands (`status`, `capabilities`, `list`, `scan`, `inspect`, `history`, `sessions`, `events`, `audit`, `export`, `daemon`, `service`).
- **Structured JSON Output**: Every inspection and querying command supports `--json` for automation and script integration.

### Terminal User Interface (Implemented & Active)
- **Interactive TUI**: Built with Ratatui and Crossterm, featuring 6 dedicated views (Dashboard, Software, Sessions, Events, Audit, and Service).
- **Centralized Service Management**: Start, stop, restart, enable, disable, and monitor background service status directly within the terminal.

### Security & Sanitization (Implemented & Active)
- **Ingest-Time Redaction**: Built-in regex scrubber that redacts flags (`--password`, `--token`, `--api-key`), inline secrets (`--password=SECRET`), HTTP Authorization headers, embedded URL credentials, and PEM private keys.

### Detected Capabilities (Probed, Not Yet Active in Monitoring Loop)
The platform detector (`crates/platform/src/detector.rs`) inspects host facilities, but the following are **not yet actively used for event ingestion**:
- *eBPF / BTF*: Probed via `/sys/kernel/btf/vmlinux`. (Tracepoint/kprobe engine planned).
- *Linux Audit Subsystem*: Probed via `/proc/net/netlink`. (Netlink socket streaming planned).
- *fanotify*: Probed via `fanotify_init` syscall. (System-wide mount interception planned).

---

## C. How It Works

InstallMon operates through an ingest, sanitize, correlate, and persist pipeline:

```mermaid
flowchart TD
    subgraph S1["Host Monitoring Backends"]
        direction TB
        B1["procfs Poller (Active)<br/>• Periodic /proc scans (800ms)<br/>• Commandline & metadata parsing"]
        B2["inotify Monitor (Active)<br/>• Watched bin & config directories<br/>• File creations, modifications & attributes"]
        B3["Kernel BTF / eBPF (Probed)<br/>• Detected host capability<br/>• Planned tracepoint engine"]
        B4["Linux Audit / fanotify (Probed)<br/>• Detected host capability<br/>• Planned event streams"]
    end

    subgraph S2["Normalization & Security Hygiene"]
        direction TB
        N1["Normalized Event Model<br/>(installmon-core)"]
        N2["Credential & Argument Sanitizer<br/>• Redacts passwords, tokens, API keys<br/>• Scrubs URL credentials & auth headers<br/>• Replaces private key blocks"]
    end

    subgraph S3["Correlation & Classification Engine"]
        direction TB
        C1["Session Manager<br/>• Maps process roots & EUID scope<br/>• Classifies installer toolchains"]
        C2["Software Discovery<br/>• Automatic binary indexing<br/>• PATH discovery integration"]
    end

    subgraph S4["Persistence Layer"]
        direction TB
        D1[("SQLite Database (WAL Mode)<br/>• Pragmas: WAL, foreign keys, busy timeout<br/>• Versioned schema migrations (v1)<br/>• Indexes: timestamp, session, target, name")]
    end

    subgraph S5["User Interfaces"]
        direction TB
        UI1["CLI (installmon)<br/>• Formatted terminal tables<br/>• Machine-readable JSON (--json)"]
        UI2["Interactive TUI (Ratatui)<br/>• 6 views: Dashboard, Software, Sessions, Events, Audit, Service<br/>• Centralized daemon lifecycle control"]
    end

    B1 & B2 --> N1
    N1 --> N2
    N2 --> C1 & C2
    C1 & C2 --> D1
    D1 --> UI1 & UI2
```

### Event Collection & Normalization Flow
1. **Detection**:
   - The daemon monitors specified directories (`/usr/local/bin`, `~/.local/bin`, `~/.cargo/bin`, `/opt`, etc.) via non-blocking Linux `inotify`.
   - The daemon polls `/proc` every 800ms to detect newly spawned processes.
2. **Classification**:
   - Commands are inspected against known installer signatures (`apt`, `cargo`, `pip`, `make`, `curl`, `git clone`, `tar`).
   - If an installer is detected, an active `SessionRecord` is initialized.
3. **Sanitization**:
   - Command arguments pass through `sanitize_arguments` before being recorded into event structures.
4. **Persistence**:
   - Events and sessions are committed into the local SQLite database inside WAL-mode transactions.
5. **Presentation**:
   - The CLI and TUI read directly from the shared SQLite database and daemon heartbeat file.

---

## D. Architecture

The repository is structured as a modular Cargo workspace:

```text
installmon/
├── Cargo.toml                  # Workspace root manifest
├── LICENSE                     # GNU General Public License v3.0
├── README.md                   # Project documentation
├── .github/
│   └── workflows/
│       ├── ci.yml              # Continuous integration (fmt, clippy, test, build)
│       └── release.yml         # Multi-arch release build and packaging pipeline
├── crates/
│   ├── core/                   # Normalized event schema, provenance models, sanitizer
│   ├── platform/               # Capability detector, path resolution, service manager
│   ├── database/               # SQLite connection, schema migrations, models
│   ├── tui/                    # Ratatui terminal user interface and view renderers
│   └── cli/                    # CLI binary (installmon), subcommands, background daemon
├── docs/
│   ├── architecture/           # Architecture overview and design diagrams
│   ├── compatibility/          # Platform and backend compatibility matrix
│   ├── database/               # Database entity-relationship and index specifications
│   └── security/               # Sensitive argument sanitization rules
└── scripts/
    └── build-release.sh        # Reproducible packaging and release build script
```

### Workspace Crates

| Crate | Responsibility | Primary Dependencies |
| :--- | :--- | :--- |
| **`installmon-core`** | Defines normalized event models (`Event`, `EventType`), provenance structures (`InstallationProvenance`, `WorkflowClassification`), and the credential argument sanitizer (`sanitize_arguments`). | `serde`, `serde_json`, `chrono`, `uuid`, `thiserror`, `regex` |
| **`installmon-platform`** | Probes host OS, libc, init system, virtualization (WSL2, Docker, LXC), and kernel monitoring interfaces. Manages systemd units and daemon heartbeats. | `installmon-core`, `serde`, `chrono`, `uuid`, `nix`, `libc`, `anyhow` |
| **`installmon-database`** | Thread-safe SQLite repository operating in WAL mode. Manages versioned migrations, transactional batch writes, and entity querying. | `installmon-core`, `installmon-platform`, `rusqlite` (bundled), `tracing` |
| **`installmon-tui`** | Terminal User Interface with 6 tabs, modal overlays, keyboard navigation, and centralized background service control. | `installmon-database`, `ratatui`, `crossterm`, `chrono`, `anyhow` |
| **`installmon-cli`** | Binary entrypoint (`installmon`). Implements CLI commands, JSON output formatting, and the continuous background daemon monitoring loop. | All workspace crates, `clap` (derive), `tracing`, `tracing-subscriber` |

---

## E. Requirements

### Supported Platforms & Architectures

| Architecture | Target Triple | Binary Format | Verification Status |
| :--- | :--- | :--- | :--- |
| **x86_64** (Intel / AMD 64-bit) | `x86_64-unknown-linux-gnu` | Dynamically linked ELF (glibc) | Verified & Tested |
| **x86_64** (Universal Static) | `x86_64-unknown-linux-musl` | Statically linked ELF (musl) | Verified & Tested |
| **AArch64** (ARM 64-bit) | `aarch64-unknown-linux-gnu` | Dynamically linked ELF (glibc) | Verified & Tested |
| **AArch64** (ARM 64-bit Static) | `aarch64-unknown-linux-musl` | Statically linked ELF (musl) | Verified & Tested |

### System Requirements
- **Operating System**: Linux kernel 4.4 or higher (5.x / 6.x recommended).
- **Rust Toolchain**: Rust 1.80.0 or higher (MSRV) for building from source (utilizes `std::sync::LazyLock`).
- **C Library**: GNU C Library (`glibc` 2.17+) or Musl libc.
- **Kernel Interfaces**: `/proc` filesystem mounted and readable; `/proc/sys/fs/inotify` support enabled.
- **Privileges**:
  - **Unprivileged (User Mode)**: Fully functional for user directories (`~/.local/bin`, `~/.cargo/bin`), user systemd services, and user SQLite database.
  - **Privileged (Root / sudo)**: Required for system directories (`/usr/bin`, `/usr/local/bin`, `/opt`), system-wide systemd units, and `/var/lib/installmon`.

---

## F. Installation

### 1. Automated Installation Script (Recommended)

InstallMon includes an automated installation script that:
- Detects the CPU architecture (`x86_64` or `aarch64`).
- Detects the Linux distribution and C library (`glibc` or `musl`).
- Downloads the matching release archive and verifies its SHA-256 checksum.
- Installs the binary into your PATH (`~/.local/bin` for users or `/usr/local/bin` with `sudo`).
- Optionally sets up and enables the background monitoring service.
- Provides a clean uninstallation method.

#### One-Line Installation (User Mode):
```bash
curl -fsSL https://raw.githubusercontent.com/ankur3-101106/installmon/main/scripts/install.sh | bash
```

#### System-Wide Installation (Root / sudo):
```bash
curl -fsSL https://raw.githubusercontent.com/ankur3-101106/installmon/main/scripts/install.sh | sudo bash -s -- --system
```

#### Script Options & Uninstallation:
```bash
# Install specific version and auto-enable background service:
bash scripts/install.sh --version v0.1.0 --service

# Clean uninstallation (preserves database):
bash scripts/install.sh --uninstall

# Complete purge (removes binaries, systemd units, and databases):
bash scripts/install.sh --uninstall --purge
```

### 2. Downloading Prebuilt Release Archives

Official release archives are published on the [GitHub Releases](https://github.com/ankur3-101106/installmon/releases) page.

#### For x86_64 (Universal Static Binary - Recommended):
```bash
# Set release version
VERSION="v0.1.0"

# Download archive and checksum
curl -LO "https://github.com/ankur3-101106/installmon/releases/download/${VERSION}/installmon-${VERSION}-x86_64-unknown-linux-musl.tar.gz"
curl -LO "https://github.com/ankur3-101106/installmon/releases/download/${VERSION}/SHA256SUMS"

# Verify checksum
sha256sum --ignore-missing --check SHA256SUMS

# Extract and install
tar -xzf "installmon-${VERSION}-x86_64-unknown-linux-musl.tar.gz"
sudo install -m 0755 installmon /usr/local/bin/installmon
```

#### For AArch64 (ARM 64-bit / Raspberry Pi 4/5 / AWS Graviton):
```bash
VERSION="v0.1.0"
curl -LO "https://github.com/ankur3-101106/installmon/releases/download/${VERSION}/installmon-${VERSION}-aarch64-unknown-linux-musl.tar.gz"
curl -LO "https://github.com/ankur3-101106/installmon/releases/download/${VERSION}/SHA256SUMS"

sha256sum --ignore-missing --check SHA256SUMS

tar -xzf "installmon-${VERSION}-aarch64-unknown-linux-musl.tar.gz"
sudo install -m 0755 installmon /usr/local/bin/installmon
```

### 3. Building from Source

Ensure a compatible Rust toolchain (1.80.0+) is installed:

```bash
# Clone the repository
git clone https://github.com/ankur3-101106/installmon.git
cd installmon

# Build release binary
cargo build --workspace --release

# The compiled binary is located at target/release/installmon
./target/release/installmon --version

# Optional: Install to system or user PATH
sudo install -m 0755 target/release/installmon /usr/local/bin/installmon
```

### 4. Installing via Cargo

Install directly into `~/.cargo/bin`:

```bash
cargo install --path crates/cli
```

### 5. Running as a User vs System Service

- **User Service (`systemd --user`)**:
  ```bash
  # Sets up ~/.config/systemd/user/installmon.service and starts monitoring user paths
  installmon service setup
  ```
- **System Service (`/etc/systemd/system`)**:
  ```bash
  # Sets up system service and monitors system-wide directories
  sudo installmon --system service setup
  ```
- **Standalone Process (Non-systemd hosts, containers, WSL2)**:
  ```bash
  # Runs daemon in foreground
  installmon daemon

  # Or spawn detached process via TUI (press 'b' in Service tab)
  ```

---

## G. Quick Start

### 1. Check System & Service Status
```bash
installmon status
```
Output:
```text
=== InstallMon System Status ===
Execution Scope  : User (Unprivileged)
Database Path    : /home/user/.local/share/installmon/installmon.db
Host Distribution: Debian GNU/Linux (debian)
Kernel Release   : 6.8.0-45-generic [x86_64]
Environment      : bare_metal
Init System      : systemd
Active Backends  : Process: [procfs] | Filesystem: [inotify]
--------------------------------
Observed Events  : 42
Tracked Sessions : 3
Tracked Software : 18
```

### 2. View Platform Capabilities
Inspect kernel facilities and active monitoring backends:
```bash
installmon capabilities
```

### 3. Run a Discovery Scan
Discover existing executables across PATH directories:
```bash
installmon scan
```

### 4. List Tracked Software
```bash
installmon list
```

### 5. Inspect Software Provenance
View recorded installations, artifacts, and evidence for a specific tool:
```bash
installmon inspect ripgrep
```

### 6. View Installation History & Events
```bash
# View recent installation sessions
installmon history --limit 10

# View raw observed lifecycle events
installmon events --limit 20
```

### 7. Run System Integrity Audit
Verify SQLite database integrity and audit for missing binaries:
```bash
installmon audit
```

### 8. Export Data to JSON / JSON-Lines
```bash
# Export all records as pretty JSON
installmon export --entity all --format json --out backup.json

# Export events as JSON-Lines for ingestion into SIEM / log pipelines
installmon export --entity events --format jsonl
```

### 9. Manage Background Service
```bash
# Install and start systemd unit
installmon service setup

# Check daemon heartbeat and uptime
installmon service status

# Stop background service
installmon service stop
```

---

## H. CLI Reference

### Global Options

These options apply across all commands:

| Option | Flag | Description |
| :--- | :--- | :--- |
| `--json` | Flag | Output machine-readable JSON instead of formatted text tables. |
| `--tui` | Flag | Launch interactive Terminal User Interface (TUI). |
| `-d, --database <PATH>` | Option | Explicit path to SQLite database file. |
| `--system` | Flag | Operate in system-wide mode (`/var/lib/installmon`) instead of user XDG mode. |
| `-v, --verbose` | Flag (Count) | Increase log verbosity (`-v`, `-vv`). |
| `-q, --quiet` | Flag | Silence log output. |
| `-h, --help` | Flag | Print command help information. |
| `-V, --version` | Flag | Print version information. |

---

### Commands

#### `installmon tui`
- **Purpose**: Launch the interactive Terminal User Interface.
- **Syntax**: `installmon tui` (or invoke `installmon` without arguments).
- **Options**: Inherits global options.
- **Example**: `installmon tui`

#### `installmon status`
- **Purpose**: Display database health, record counts, active scope, and platform identity.
- **Syntax**: `installmon status [--json]`
- **Options**: `--json`
- **Example**: `installmon status --json`
- **Behavior**: Queries SQLite database metrics and probes host environment.

#### `installmon capabilities`
- **Purpose**: Probe kernel BTF, inotify, fanotify, audit subsystem, libc, and init system.
- **Syntax**: `installmon capabilities [--json]`
- **Options**: `--json`
- **Example**: `installmon capabilities`
- **Behavior**: Runs platform detection and records probe results in `monitoring_capabilities`.

#### `installmon list`
- **Purpose**: Display all tracked software packages stored in the database.
- **Syntax**: `installmon list [--json]`
- **Options**: `--json`
- **Example**: `installmon list`
- **Behavior**: Lists software name, scope, discovery timestamp, and primary binary path.

#### `installmon scan`
- **Purpose**: Scan PATH and specified directories to discover executable binaries on disk.
- **Syntax**: `installmon scan [DIRECTORIES...] [--json]`
- **Arguments**: `[DIRECTORIES...]` (Optional: explicit directories to scan instead of default PATH).
- **Options**: `--json`
- **Example**: `installmon scan /opt/bin /usr/local/bin`
- **Behavior**: Identifies executable files, inserts new software records, and skips existing entries.

#### `installmon inspect`
- **Purpose**: Inspect detailed installation sessions, artifacts, and evidence for a software package.
- **Syntax**: `installmon inspect <SOFTWARE> [--json]`
- **Arguments**: `<SOFTWARE>` (Required: software name or UUID).
- **Options**: `--json`
- **Example**: `installmon inspect git`
- **Behavior**: Performs case-insensitive lookup and displays linked installations and artifacts.

#### `installmon history`
- **Purpose**: Show a timeline of recorded installation sessions.
- **Syntax**: `installmon history [--limit <N>] [--json]`
- **Options**:
  - `-l, --limit <LIMIT>`: Number of sessions to display (default: `20`).
  - `--json`: Machine-readable JSON output.
- **Example**: `installmon history --limit 5`

#### `installmon sessions`
- **Purpose**: List tracked installation sessions in table format.
- **Syntax**: `installmon sessions [--limit <N>] [--json]`
- **Options**:
  - `-l, --limit <LIMIT>`: Maximum sessions to display (default: `20`).
  - `--json`: Machine-readable JSON output.
- **Example**: `installmon sessions`

#### `installmon events`
- **Purpose**: Query raw observed process execution and filesystem lifecycle events.
- **Syntax**: `installmon events [OPTIONS]`
- **Options**:
  - `-l, --limit <LIMIT>`: Maximum events to display (default: `50`).
  - `--offset <OFFSET>`: Pagination offset (default: `0`).
  - `-t, --type <TYPE>`: Filter by event type (`process_exec`, `file_create`, `file_modify`, etc.).
  - `-s, --session <UUID>`: Filter by specific session UUID.
  - `--json`: Machine-readable JSON output.
- **Example**: `installmon events --type process_exec --limit 25`

#### `installmon audit`
- **Purpose**: Run database integrity checks and audit tracked software against the physical disk.
- **Syntax**: `installmon audit [--json]`
- **Options**: `--json`
- **Example**: `installmon audit`
- **Behavior**: Runs `PRAGMA integrity_check`, checks if binary paths exist on disk, and flags unknown workflow classifications.

#### `installmon export`
- **Purpose**: Export tracking records to JSON or JSON-Lines format.
- **Syntax**: `installmon export [OPTIONS]`
- **Options**:
  - `-f, --format <FORMAT>`: Export format: `json` or `jsonl` (default: `json`).
  - `-e, --entity <ENTITY>`: Entity to export: `all`, `events`, `sessions`, `software` (default: `all`).
  - `-o, --out <PATH>`: Destination file path (defaults to stdout).
- **Example**: `installmon export -e software -f jsonl -o software.jsonl`

#### `installmon daemon`
- **Purpose**: Run the continuous monitoring daemon in the foreground.
- **Syntax**: `installmon daemon`
- **Behavior**: Sets up signal handlers (SIGINT/SIGTERM), initializes inotify watches, polls `/proc`, and updates `daemon.json` heartbeat every 2 seconds.

#### `installmon service`
- **Purpose**: Manage continuous background daemon service lifecycle.
- **Syntax**: `installmon service <ACTION>`
- **Subcommands**:
  - `status`: Show service state (`Running`, `Stopped`, `NotInstalled`), PID, uptime, and live daemon heartbeat.
  - `setup` (alias: `install`): Install systemd unit file, reload daemon, enable auto-start, and start service.
  - `start`: Start the background service.
  - `stop`: Stop the background service.
  - `restart`: Restart the background service.
  - `enable`: Enable systemd auto-start on boot/login.
  - `disable`: Disable systemd auto-start.
  - `uninstall`: Stop service and remove systemd unit file.
- **Example**: `installmon service status`

---

## I. TUI Documentation

The InstallMon TUI is an interactive, single-pane-of-glass terminal interface built with Ratatui:

```bash
installmon tui
# or simply invoke without subcommands:
installmon
```

### Views & Tabs

| Tab Number | View | Purpose |
| :---: | :--- | :--- |
| **`1`** | **Dashboard** | High-level system overview, host architecture, database counts, negotiated monitoring backends, and recent activity logs. |
| **`2`** | **Software** | Table of tracked software packages. Displays name, scope, discovery timestamp, and primary binary path on disk. |
| **`3`** | **Sessions** | Table of recorded installation sessions with classification, initiating command, execution scope, and status. |
| **`4`** | **Events** | Real-time log of normalized lifecycle events (`process_exec`, `file_create`, etc.), displaying PID, EUID, executable, and target paths. |
| **`5`** | **Audit** | Live audit report showing SQLite integrity check status, missing binary alerts, and unclassified workflow warnings. |
| **`6`** | **Service** | Interactive service management panel showing daemon heartbeat, process PID, recorded event counters, uptime, and control buttons. |

### Navigation & Keybindings

| Key | Context | Action |
| :--- | :--- | :--- |
| `1` .. `6` | Global | Switch directly to Tab 1 through 6. |
| `Tab` / `l` / `→` | Global | Switch to next tab. |
| `BackTab` / `h` / `←` | Global | Switch to previous tab. |
| `j` / `↓` | Software / Sessions / Events | Move selection down in the table. |
| `k` / `↑` | Software / Sessions / Events | Move selection up in the table. |
| `s` | Global | Trigger a binary discovery scan on PATH. |
| `r` | Global | Refresh database records and daemon heartbeat. |
| `I` / `i` | Global | **Setup Service**: Install systemd unit, enable, and start background daemon. |
| `S` | Global | **Toggle Service**: Start or stop background service. |
| `R` | Global | **Restart Service**: Restart background daemon. |
| `e` | Service Tab | **Toggle Auto-Start**: Enable or disable boot/login auto-start. |
| `b` | Service Tab | **Spawn Standalone**: Launch detached background process (works without systemd). |
| `u` | Service Tab | **Uninstall Unit**: Stop daemon and remove systemd unit file. |
| `?` | Global | Toggle Help modal overlay. |
| `q` / `Ctrl+C` | Global | Exit TUI. *(Note: The background daemon continues running uninterrupted).* |

---

## J. Database and Data Storage

InstallMon uses an embedded SQLite database. No separate database server process, user configuration, or network port is required.

### Database File Locations

| Scope | Default Database Path | Overriding Environment Variable | Fallback Path |
| :--- | :--- | :--- | :--- |
| **User Mode** | `~/.local/share/installmon/installmon.db` | `XDG_DATA_HOME/installmon/installmon.db` | `/tmp/installmon/installmon.db` |
| **System Mode** | `/var/lib/installmon/installmon.db` | Explicit `--database <PATH>` | `/tmp/installmon/installmon.db` |

### SQLite Configuration & WAL Mode

Every database connection applies high-performance pragmas upon opening:

```sql
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA foreign_keys = ON;
PRAGMA busy_timeout = 5000;
PRAGMA temp_store = MEMORY;
```

- **Write-Ahead Logging (WAL)**: Allows concurrent readers while the background daemon writes events, avoiding database locking errors.
- **Foreign Keys**: Enforces relational consistency across sessions, installations, artifacts, and evidence.
- **Busy Timeout**: Sets a 5000ms retry timeout for concurrent operations.

### Schema Migrations

The database manages schema evolution automatically via the `schema_migrations` table:

```sql
CREATE TABLE schema_migrations (
    version INTEGER PRIMARY KEY,
    applied_at TEXT NOT NULL,
    description TEXT NOT NULL
);
```

- **Migration v1**: Initializes core tables: `monitoring_capabilities`, `sessions`, `processes`, `events`, `software`, `installations`, `artifacts`, `sources`, and `evidence`.
- **Indexes**: Includes performance indexes on `events(timestamp)`, `events(session_id)`, `events(event_type)`, `events(target_path)`, `processes(pid)`, `artifacts(path)`, `software(name)`, and `sessions(status)`.

### Integrity & Maintenance

- Verification: Run `installmon audit` to execute `PRAGMA integrity_check`.
- Export: Use `installmon export -f jsonl` to backup data or ingest into external SIEM platforms.

---

## K. Security and Privacy

### Principle of Least Privilege

- **Unprivileged by Default**: InstallMon does not require root permissions. Standard users can monitor user installations (`~/.local/bin`, `~/.cargo/bin`) and manage user systemd units without `sudo`.
- **Read-Only Observability**: InstallMon observes and logs events; it never executes arbitrary user commands or alters installation workflows.

### Sensitive Credential Sanitization

Command-line invocations often contain passwords, tokens, or secret keys (e.g. `curl --token SECRET`, `git clone https://user:pass@host`). InstallMon sanitizes all arguments before persistence:

```text
Raw Ingest:
  curl -u admin:super_secret -H "Authorization: Bearer eyJhbGci..." --token=xyz123 https://user:secretpass@host/repo.git

Sanitized in Database:
  curl -u [REDACTED] -H "Authorization: Bearer [REDACTED]" --token=[REDACTED] https://user:[REDACTED]@host/repo.git
```

#### Redaction Rules:
1. **Separate Flag Values**: Arguments following flags like `--password`, `-p`, `--token`, `--api-key`, `--auth`, `--secret`, `--access-key`, `--private-key`, and `-u` / `--user` are replaced with `[REDACTED]`.
2. **Inline Flag Arguments**: Arguments formatted as `--password=SECRET` or `--token=SECRET` are rewritten to `--password=[REDACTED]`.
3. **URL Embedded Credentials**: URLs like `https://user:pass@host` are transformed to `https://user:[REDACTED]@host`.
4. **Authorization Headers**: HTTP headers formatted as `Authorization: Bearer <token>` or `Authorization: Basic <base64>` are scrubbed to `Authorization: Bearer [REDACTED]`.
5. **Private Key Blocks**: Any text block matching `-----BEGIN ... PRIVATE KEY-----` is replaced with `[REDACTED_PRIVATE_KEY]`.

### Network Behavior & Privacy

- **Offline-First**: InstallMon makes **zero outbound network connections**. It contains no analytics, telemetry, or remote crash reporting.
- **Local Storage**: All data remains entirely within your local SQLite database file.

### Limitations of Sanitization

> [!WARNING]
> While regex sanitization catches standard options and patterns, custom positional arguments or non-standard flag names cannot be 100% guaranteed to be detected. Avoid passing plaintext credentials directly on command lines.

---

## L. Limitations

Technical transparency is fundamental to InstallMon. The following limitations apply to the current release:

1. **Procfs Polling Interval**: Process activity monitoring operates via periodic `/proc` scanning at 800ms intervals. Extremely transient processes (executing and exiting within < 800ms) may not be detected.
2. **Inotify Directory Scope**: Linux `inotify` does not watch directory hierarchies recursively. Watched directories are explicitly registered (`/usr/local/bin`, `~/.local/bin`, `~/.cargo/bin`, etc.). Arbitrary directories outside configured paths are not automatically observed until scanned.
3. **Probed vs Active Backends**: Kernel eBPF/BTF tracing, Linux Audit netlink streaming, and fanotify mount interception are currently probed for platform capability reporting, but the active daemon monitoring loop uses `inotify` and `/proc`.
4. **Unprivileged Scope**: In user mode, InstallMon cannot observe actions executed by other users, privileged system modifications in `/usr/bin`, or manage system-wide systemd units.
5. **Static Binary Targets**: Standard release builds on GNU Linux (`x86_64-unknown-linux-gnu`) dynamically link against glibc. For a 100% static binary with zero dynamic library dependencies, use the `musl` release target (`x86_64-unknown-linux-musl`).

---

## M. Troubleshooting

### 1. `Permission Denied` when running daemon or database
- **Symptom**: `Failed to open database at /var/lib/installmon/installmon.db: Permission denied`.
- **Cause**: Attempting to access system database without root privileges.
- **Resolution**: Run without `--system` to use user XDG storage (`~/.local/share/installmon`), or run with `sudo` if system monitoring is intended.

### 2. `Database is locked` or initialization error
- **Symptom**: `database is locked (5)`.
- **Cause**: Another process holds an exclusive lock or SQLite WAL mode was interrupted.
- **Resolution**: InstallMon automatically applies `PRAGMA busy_timeout = 5000;`. Ensure no hanging background process holds the file:
  ```bash
  installmon service status
  # Stop duplicate processes if necessary:
  killall installmon
  ```

### 3. TUI terminal rendering glitches
- **Symptom**: Characters misplaced or colors distorted.
- **Resolution**: Ensure your terminal emulator supports 256 colors or truecolor and matches `TERM=xterm-256color`. You can also resize the terminal window to trigger a redraw.

### 4. Background daemon not starting under systemd
- **Symptom**: `installmon service setup` fails or reports inactive.
- **Resolution**:
  - For user service: Verify lingering is enabled if needed: `loginctl enable-linger $USER`.
  - Check service logs: `journalctl --user -u installmon -e` or check daemon log file at `~/.local/share/installmon/daemon.log`.
  - Fallback: Press `b` in the TUI Service tab to spawn a standalone detached daemon without systemd.

---

## N. Development

### Repository Setup & Toolchain

InstallMon requires Rust 1.80.0+ (stable).

```bash
# Clone the repository
git clone https://github.com/ankur3-101106/installmon.git
cd installmon

# Verify toolchain version
rustc --version
```

### Building & Formatting

```bash
# Check compilation across all crates
cargo check --workspace

# Verify code formatting
cargo fmt --all -- --check

# Format code
cargo fmt --all

# Run Clippy lints
cargo clippy --workspace --all-targets -- -D warnings
```

### Running Tests

```bash
# Run all workspace unit and integration tests
cargo test --workspace --verbose

# Run specific crate tests
cargo test -p installmon-core
cargo test -p installmon-database
```

### Packaging Release Artifacts

The packaging script verifies and packages versioned tarballs with SHA-256 checksums:

```bash
# Build and package for host architecture
./scripts/build-release.sh

# Or specify target triple and version
./scripts/build-release.sh x86_64-unknown-linux-gnu v0.1.0
```

### Contribution Guidelines
1. Ensure all code compiles cleanly with zero Clippy warnings (`cargo clippy --workspace --all-targets -- -D warnings`).
2. Run `cargo fmt --all -- --check` prior to committing.
3. Ensure all tests pass (`cargo test --workspace`).
4. Avoid adding heavy external dependencies without justification.
5. Create focused commits adhering to conventional commit guidelines (`docs:`, `ci:`, `chore:`, `feat:`, `fix:`).

---

## O. Roadmap

### Implemented & Stable (Current Release)
- [x] Normalized event model with strongly-typed lifecycle event types.
- [x] Ingest-time sensitive credential and argument sanitization.
- [x] Multi-directory inotify filesystem monitoring loop.
- [x] Periodic `/proc` process activity observation and classifier.
- [x] SQLite WAL-mode repository with versioned migrations and transactional batch ingestion.
- [x] 12-command CLI with standardized `--json` output across commands.
- [x] Ratatui Terminal User Interface with 6 views and service management.
- [x] systemd user/system service management and standalone daemon fallback.
- [x] Multi-architecture CI and verified release automation with SHA-256 checksums.

### Planned Future Enhancements
- [ ] **Kernel eBPF / BTF Monitoring Engine**: Ingest process exec/fork events in real-time via tracepoints to eliminate the 800ms polling gap.
- [ ] **Linux Audit Subsystem Integration**: Stream process events over netlink sockets as an alternative to eBPF.
- [ ] **fanotify Filesystem Interception**: System-wide mount-level monitoring when running with root privileges.
- [ ] **Recursive Directory Watches**: Dynamic directory tree expansion for deep installation paths.
- [ ] **Binary Checksum Ingestion**: Automated background SHA-256 hashing of newly registered binaries.

---

## P. License and Acknowledgements

### License

InstallMon is free software: you can redistribute it and/or modify it under the terms of the **GNU General Public License Version 3 (GPL-3.0-only)** as published by the Free Software Foundation. See the [LICENSE](LICENSE) file for complete details.

### Acknowledgements

InstallMon builds upon the excellent work of the open-source Rust systems community:
- [ratatui](https://github.com/ratatui/ratatui) & [crossterm](https://github.com/crossterm-rs/crossterm) for terminal user interfaces.
- [rusqlite](https://github.com/rusqlite/rusqlite) for embedded SQLite bindings.
- [clap](https://github.com/clap-rs/clap) for command-line parsing.
- [nix](https://github.com/nix-rust/nix) & [libc](https://github.com/rust-lang/libc) for native Linux systems programming interfaces.
