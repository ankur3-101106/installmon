# Architecture Overview: InstallMon

[InstallMon](../../README.md) > [Documentation Hub](../README.md) > Architecture Overview

InstallMon observes how software enters, gets installed, and changes within a Linux host. It operates non-invasively through an ingest, sanitize, correlate, and persist pipeline without wrapping package managers or modifying system execution flows.

---

## 1. System Pipeline

```mermaid
flowchart TD
    subgraph S1["1. Host Monitoring Backends"]
        direction TB
        B1["procfs Poller (Active)<br/>• Scans /proc every 800ms<br/>• Classifies installer toolchains"]
        B2["inotify Monitor (Active)<br/>• Watched bin & config directories<br/>• File creations, modifications & attributes"]
        B3["Kernel BTF / eBPF (Probed)<br/>• Host capability detection<br/>• Planned tracepoint engine"]
        B4["Linux Audit / fanotify (Probed)<br/>• Host capability detection<br/>• Planned mount interception"]
    end

    subgraph S2["2. Normalization & Security Hygiene"]
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

## 2. Event Flow & Normalization

1. **Ingestion**:
   - The daemon monitors specified binary directories (`/usr/local/bin`, `~/.local/bin`, `~/.cargo/bin`, etc.) via non-blocking Linux `inotify`.
   - The daemon polls `/proc` every 800ms to detect newly spawned processes.
2. **Classification**:
   - Commands are inspected against known installer signatures (`apt`, `cargo`, `pip`, `make`, `curl`, `git clone`, `tar`).
   - If an installer is detected, an active `SessionRecord` is initialized.
3. **Sanitization**:
   - Command arguments pass through `sanitize_arguments` before being recorded into event structures. See [Security & Sanitization](../security/sanitization.md).
4. **Persistence**:
   - Events and sessions are committed into the local SQLite database inside WAL-mode transactions. See [Database Schema Specification](../database/schema.md).
5. **Presentation**:
   - The CLI and TUI read directly from the shared SQLite database and daemon heartbeat file. See [CLI Reference](../cli/reference.md) and [TUI Guide](../tui/guide.md).

---

## 3. Cargo Workspace Crate Architecture

The repository is organized as a modular Cargo workspace across 5 crates:

```text
installmon/
├── crates/
│   ├── core/                   # Normalized event schema, provenance models, sanitizer
│   ├── platform/               # Capability detector, path resolution, service manager
│   ├── database/               # SQLite connection, schema migrations, models
│   ├── tui/                    # Ratatui terminal user interface and view renderers
│   └── cli/                    # CLI binary (installmon), subcommands, background daemon
```

| Crate | Responsibility | Primary Dependencies |
| :--- | :--- | :--- |
| **`installmon-core`** | Defines normalized event models (`Event`, `EventType`), provenance structures (`InstallationProvenance`, `WorkflowClassification`), and the credential argument sanitizer (`sanitize_arguments`). | `serde`, `serde_json`, `chrono`, `uuid`, `thiserror`, `regex` |
| **`installmon-platform`** | Probes host OS, libc, init system, virtualization (WSL2, Docker, LXC), and kernel monitoring interfaces. Manages systemd units and daemon heartbeats. | `installmon-core`, `serde`, `chrono`, `uuid`, `nix`, `libc`, `anyhow` |
| **`installmon-database`** | Thread-safe SQLite repository operating in WAL mode. Manages versioned migrations, transactional batch writes, and entity querying. | `installmon-core`, `installmon-platform`, `rusqlite` (bundled), `tracing` |
| **`installmon-tui`** | Terminal User Interface with 6 tabs, modal overlays, keyboard navigation, and centralized background service control. | `installmon-database`, `ratatui`, `crossterm`, `chrono`, `anyhow` |
| **`installmon-cli`** | Binary entrypoint (`installmon`). Implements CLI commands, JSON output formatting, and the continuous background daemon monitoring loop. | All workspace crates, `clap` (derive), `tracing`, `tracing-subscriber` |

---

## Related Documentation & Next Steps

- 🗄️ [Database Schema Specification](../database/schema.md) — Table schemas, relations, and WAL performance tuning.
- 🔒 [Security & Sanitization](../security/sanitization.md) — Ingest-time secret redaction rules.
- 🐧 [Platform Compatibility Matrix](../compatibility/matrix.md) — Active vs probed kernel capabilities.
- 🗺️ [Project Roadmap](../roadmap.md) — Future architecture plans (eBPF, netlink audit, fanotify).

---

[← Back to Documentation Hub](../README.md) | [← Back to Main README](../../README.md)
