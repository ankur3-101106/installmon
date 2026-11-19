# InstallMon

[![CI](https://github.com/ankur3-101106/installmon/actions/workflows/ci.yml/badge.svg)](https://github.com/ankur3-101106/installmon/actions/workflows/ci.yml)
[![License: GPL-3.0-only](https://img.shields.io/badge/License-GPL--3.0--only-blue.svg)](LICENSE)
[![MSRV](https://img.shields.io/badge/MSRV-1.80.0%2B-orange.svg)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/Platform-Linux-brightgreen.svg)](docs/compatibility/matrix.md)
[![Architecture](https://img.shields.io/badge/Architecture-x86__64%20%7C%20aarch64-blueviolet.svg)](docs/compatibility/matrix.md)

> **Linux Software Installation Monitoring and Provenance Tracking.**

InstallMon observes how software enters, gets installed, and changes within a Linux host. It records installation workflows, reconstructs execution history, identifies installed software binaries, and maintains provenance records without wrapping package managers or modifying system execution flows.

---

## The Problem It Solves

Modern Linux systems accumulate software through disparate mechanisms—system package managers (`apt`, `pacman`, `dnf`), language ecosystem installers (`cargo`, `pip`, `npm`, `uv`, `go install`), source builds (`make`, `cmake`), ad-hoc scripts (`curl ... | bash`), and unpacked archives (`tar -xzf`).

Most distributions only track software installed through their native package manager. Software installed via other vectors remains invisible to system audits, creating configuration drift, blind spots, and untracked binaries. **InstallMon closes this gap** by continuously monitoring installation activity and maintaining an offline, queryable provenance catalog.

---

## Key Highlights

- **Filesystem & Process Ingestion**: Multi-directory `inotify` observation on executable paths (`/usr/local/bin`, `~/.local/bin`, `~/.cargo/bin`) paired with periodic `/proc` process polling.
- **Ingest-Time Secret Sanitization**: Built-in credential scrubber that redacts passwords, tokens, API keys, HTTP Authorization headers, URL credentials, and private keys before records touch disk.
- **Session Correlation & Provenance**: Automatically links initiating commands, process ancestry (PID/PPID/UID/EUID), and filesystem mutations into structured installation sessions.
- **Interactive Terminal UI (TUI)**: Rich [Ratatui](https://github.com/ratatui/ratatui) interface with 6 tabs (Dashboard, Software, Sessions, Events, Audit, Service) and direct background daemon management.
- **12-Subcommand CLI with JSON Output**: Formatted terminal tables or machine-readable JSON (`--json`) across all inspection, querying, and export commands.
- **Zero-Config Embedded SQLite (WAL Mode)**: Thread-safe local persistence with Write-Ahead Logging, automated schema migrations (`v1`), and integrity auditing.
- **Universal Static Binaries**: Self-contained `musl` binaries for `x86_64` and `aarch64` with zero dynamic library dependencies.

---

## How It Works

```mermaid
flowchart TD
    subgraph S1["1. Host Monitoring Backends"]
        direction TB
        B1["procfs Poller (Active)<br/>• Scans /proc every 800ms<br/>• Classifies installer toolchains"]
        B2["inotify Monitor (Active)<br/>• Watched bin & config directories<br/>• File creations, modifications & attributes"]
        B3["Kernel BTF / eBPF (Probed)<br/>• Host capability detection<br/>• Planned tracepoint engine"]
        B4["Linux Audit / fanotify (Probed)<br/>• Host capability detection<br/>• Planned mount interception"]
    end

    subgraph S2["2. Normalization & Security Ingest"]
        direction TB
        N1["Normalized Event Model<br/>(installmon-core)"]
        N2["Credential & Argument Sanitizer<br/>• Redacts passwords, tokens, API keys<br/>• Scrubs URL credentials & auth headers<br/>• Replaces private key blocks"]
    end

    subgraph S3["3. Correlation & Session Engine"]
        direction TB
        C1["Session Lifecycle Manager<br/>• Maps process roots & EUID scope<br/>• Classifies workflow patterns"]
        C2["Binary Discovery Engine<br/>• Automatic PATH executable registration"]
    end

    subgraph S4["4. Local Persistence Layer"]
        direction TB
        D1[("SQLite Database (WAL Mode)<br/>• Pragmas: WAL, foreign keys, busy timeout<br/>• Versioned schema migrations (v1)<br/>• Performance indexes on timestamp & session")]
    end

    subgraph S5["5. User Interfaces"]
        direction TB
        UI1["CLI: installmon<br/>• Formatted terminal tables<br/>• Machine-readable JSON (--json)"]
        UI2["Interactive TUI: Ratatui Dashboard<br/>• 6 views: Dashboard, Software, Sessions, Events, Audit, Service<br/>• Centralized daemon lifecycle control"]
    end

    B1 & B2 --> N1
    N1 --> N2
    N2 --> C1 & C2
    C1 & C2 --> D1
    D1 --> UI1 & UI2
```

---

## Quick Start

### 1. Install via Unified Lifecycle Assistant (Recommended)

One command installs or updates InstallMon, verifies SHA-256 checksums, and optionally configures the continuous background daemon:

```bash
curl -fsSL https://raw.githubusercontent.com/ankur3-101106/installmon/main/install.sh | bash
```

### 2. Take It for a Spin

```bash
# Check host environment and database metrics
installmon status

# Discover existing executables across PATH
installmon scan

# Launch the interactive terminal dashboard
installmon
```

---

## 📚 Documentation Hub

Detailed technical guides, command specifications, and architectural documentation are maintained in the [`docs/`](docs/) directory:

| Guide | Description |
| :--- | :--- |
| [📦 **Installation & Lifecycle Guide**](docs/installation/guide.md) | Single-command installer, prebuilt archives, checksums, building from source, systemd setup, and uninstallation. |
| [💻 **CLI Command Reference**](docs/cli/reference.md) | Complete reference for all 12 subcommands, global options, arguments, syntax, and JSON output formats. |
| [🖥️ **Terminal User Interface (TUI) Guide**](docs/tui/guide.md) | Ratatui terminal dashboard walkthrough, 6 views, keyboard navigation, and centralized service control. |
| [🏗️ **Architecture Overview**](docs/architecture/overview.md) | Ingestion pipeline breakdown, event normalization, session correlation, and Cargo workspace structure. |
| [🗄️ **Database Schema Specification**](docs/database/schema.md) | Entity-relationship diagrams, SQLite WAL configuration, pragmas, schema migrations, and maintenance. |
| [🔒 **Security & Argument Sanitization**](docs/security/sanitization.md) | Least privilege design, credential scrubbing rules, regex filters, offline-first policy, and audit boundaries. |
| [🐧 **Platform Compatibility Matrix**](docs/compatibility/matrix.md) | Active vs probed capabilities (eBPF, fanotify, audit, procfs, inotify), Linux distros, and container environments. |
| [🛠️ **Troubleshooting & FAQ**](docs/troubleshooting/faq.md) | Diagnosing permissions, SQLite database locks, terminal truecolor issues, systemd lingering, and common questions. |
| [🚀 **Development & Contributing**](docs/development/contributing.md) | Rust toolchain requirements, formatting, Clippy lints, running test suites, release packaging, and CI workflows. |
| [🗺️ **Project Roadmap**](docs/roadmap.md) | Implemented capabilities vs planned future enhancements (kernel eBPF tracepoints, netlink audit streaming). |

---

## CLI Commands at a Glance

| Command | Summary | Reference |
| :--- | :--- | :--- |
| `installmon` / `installmon tui` | Launch the interactive visual dashboard | [Docs](docs/tui/guide.md) |
| `installmon status` | Display database metrics and backend health | [Docs](docs/cli/reference.md#installmon-status) |
| `installmon capabilities` | Probe kernel BTF, inotify, fanotify, audit, libc, and init system | [Docs](docs/cli/reference.md#installmon-capabilities) |
| `installmon list` | List all discovered software packages | [Docs](docs/cli/reference.md#installmon-list) |
| `installmon scan [DIRS...]` | Scan PATH and target directories for executables | [Docs](docs/cli/reference.md#installmon-scan) |
| `installmon inspect <NAME>` | View installation provenance, artifacts, and evidence | [Docs](docs/cli/reference.md#installmon-inspect) |
| `installmon history` | View chronological timeline of installation sessions | [Docs](docs/cli/reference.md#installmon-history) |
| `installmon sessions` | List tracked installation sessions in table format | [Docs](docs/cli/reference.md#installmon-sessions) |
| `installmon events` | Query observed lifecycle events with filters and pagination | [Docs](docs/cli/reference.md#installmon-events) |
| `installmon audit` | Verify SQLite database integrity and audit disk binaries | [Docs](docs/cli/reference.md#installmon-audit) |
| `installmon export` | Export tracking records to JSON or JSON-Lines (`jsonl`) | [Docs](docs/cli/reference.md#installmon-export) |
| `installmon daemon` | Run continuous monitoring daemon in foreground | [Docs](docs/cli/reference.md#installmon-daemon) |
| `installmon service <ACTION>` | Manage background service lifecycle (setup, start, stop, enable) | [Docs](docs/cli/reference.md#installmon-service) |

---

## Development & Verification

Ensure you have Rust **1.80.0+** installed:

```bash
# Verify formatting, lints, and test suites
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace

# Compile release binary
cargo build --workspace --release
```

See the [Development & Contributing Guide](docs/development/contributing.md) for complete workflows.

---

## License

InstallMon is free software licensed under the **GNU General Public License Version 3 (GPL-3.0-only)**. See the [LICENSE](LICENSE) file for complete details.
