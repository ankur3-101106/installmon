# CLI Command Reference: InstallMon

[InstallMon](../../README.md) > [Documentation Hub](../README.md) > CLI Command Reference

`installmon` provides a command-line interface with 12 specialized subcommands for monitoring, querying, auditing, and managing software installation provenance. Every querying command supports structured `--json` output for automation and pipeline integration.

---

## Global Options

The following flags and options are available across all subcommands:

| Option | Flag | Description |
| :--- | :--- | :--- |
| `--json` | Flag | Output machine-readable JSON instead of formatted terminal tables. |
| `--tui` | Flag | Launch the interactive Terminal User Interface (TUI). |
| `-d, --database <PATH>` | Option | Explicit path to SQLite database file. Overrides XDG default. |
| `--system` | Flag | Operate in system-wide mode (`/var/lib/installmon`) instead of user mode. |
| `-v, --verbose` | Flag (Count) | Increase log verbosity (`-v` for debug, `-vv` for trace). |
| `-q, --quiet` | Flag | Silence operational logs. |
| `-h, --help` | Flag | Print command help information. |
| `-V, --version` | Flag | Print version information. |

---

## Command Quick Index

| Command | Primary Role | Output Formats |
| :--- | :--- | :--- |
| [`installmon tui`](#installmon-tui) | Launch interactive Ratatui terminal dashboard | Terminal UI |
| [`installmon status`](#installmon-status) | Show host identity, DB metrics, and backend status | Table / JSON |
| [`installmon capabilities`](#installmon-capabilities) | Probe kernel BTF, inotify, fanotify, audit, libc, and init system | Table / JSON |
| [`installmon list`](#installmon-list) | Display all discovered software packages | Table / JSON |
| [`installmon scan`](#installmon-scan) | Scan PATH and target directories for executables | Table / JSON |
| [`installmon inspect`](#installmon-inspect) | View installations, artifacts, and evidence for a software item | Table / JSON |
| [`installmon history`](#installmon-history) | View chronologically ordered installation sessions | Table / JSON |
| [`installmon sessions`](#installmon-sessions) | List tracked installation sessions in table format | Table / JSON |
| [`installmon events`](#installmon-events) | Query observed lifecycle events with filtering and pagination | Table / JSON |
| [`installmon audit`](#installmon-audit) | Run PRAGMA integrity checks and software disk verification | Table / JSON |
| [`installmon export`](#installmon-export) | Export database entities to JSON or JSON-Lines | File / stdout |
| [`installmon daemon`](#installmon-daemon) | Run continuous background monitoring daemon in foreground | Log stream |
| [`installmon service`](#installmon-service) | Manage systemd background service lifecycle | Status / Table |

---

## Detailed Command Specifications

### `installmon tui`
- **Purpose**: Launch the interactive Terminal User Interface.
- **Syntax**: `installmon tui` (or simply run `installmon` without arguments).
- **Description**: Opens the single-pane-of-glass dashboard featuring 6 views: Dashboard, Software, Sessions, Events, Audit, and Service.
- **Related Guide**: See the complete [Terminal UI (TUI) Guide](../tui/guide.md).

---

### `installmon status`
- **Purpose**: Display database health, record counts, active scope, and platform identity.
- **Syntax**: `installmon status [--json]`
- **Example**:
  ```bash
  installmon status
  ```
- **Example Output**:
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
- <details><summary><b>View JSON Output (--json)</b></summary>

  ```json
  {
    "scope": "User",
    "database_path": "/home/user/.local/share/installmon/installmon.db",
    "host_distribution": "Debian GNU/Linux",
    "kernel_release": "6.8.0-45-generic",
    "architecture": "x86_64",
    "environment": "bare_metal",
    "active_backends": {
      "process": "procfs",
      "filesystem": "inotify"
    },
    "counts": {
      "events": 42,
      "sessions": 3,
      "software": 18
    }
  }
  ```
  </details>

---

### `installmon capabilities`
- **Purpose**: Inspect host platform and probe kernel monitoring capabilities.
- **Syntax**: `installmon capabilities [--json]`
- **Description**: Probes kernel BTF (`/sys/kernel/btf/vmlinux`), inotify, fanotify (`fanotify_init`), netlink audit socket, C library, and init system.
- **Related Guide**: See the [Platform Compatibility Matrix](../compatibility/matrix.md).
- **Example**:
  ```bash
  installmon capabilities
  ```

---

### `installmon list`
- **Purpose**: Display all tracked software packages stored in the database.
- **Syntax**: `installmon list [--json]`
- **Description**: Queries the `software` table and outputs software name, scope, discovery date, and primary binary path.
- **Example**:
  ```bash
  installmon list
  ```

---

### `installmon scan`
- **Purpose**: Scan PATH and specified directories to discover executable binaries on disk.
- **Syntax**: `installmon scan [DIRECTORIES...] [--json]`
- **Arguments**:
  - `[DIRECTORIES...]`: Optional directory list. If omitted, scans user/system PATH directories (`/usr/local/bin`, `~/.local/bin`, `~/.cargo/bin`, etc.).
- **Example**:
  ```bash
  installmon scan /opt/bin /usr/local/bin
  ```

---

### `installmon inspect`
- **Purpose**: Inspect detailed installation sessions, artifacts, and evidence for a software package.
- **Syntax**: `installmon inspect <SOFTWARE> [--json]`
- **Arguments**:
  - `<SOFTWARE>`: Software name or UUID (case-insensitive lookup).
- **Example**:
  ```bash
  installmon inspect ripgrep
  ```
- **Description**: Displays the provenance chain, linked installation records, package classifications, created artifact files, and supporting evidence.

---

### `installmon history`
- **Purpose**: Show a timeline of recorded installation sessions.
- **Syntax**: `installmon history [--limit <N>] [--json]`
- **Options**:
  - `-l, --limit <LIMIT>`: Number of sessions to display (default: `20`).
- **Example**:
  ```bash
  installmon history --limit 5
  ```

---

### `installmon sessions`
- **Purpose**: List tracked installation sessions in table format.
- **Syntax**: `installmon sessions [--limit <N>] [--json]`
- **Options**:
  - `-l, --limit <LIMIT>`: Maximum sessions to display (default: `20`).
- **Example**:
  ```bash
  installmon sessions
  ```

---

### `installmon events`
- **Purpose**: Query raw observed process execution and filesystem lifecycle events.
- **Syntax**: `installmon events [OPTIONS]`
- **Options**:
  - `-l, --limit <LIMIT>`: Maximum events to return (default: `50`).
  - `--offset <OFFSET>`: Pagination offset (default: `0`).
  - `-t, --type <TYPE>`: Filter by event type (`process_exec`, `file_create`, `file_modify`, `file_delete`, `file_rename`).
  - `-s, --session <UUID>`: Filter by specific session UUID.
  - `--json`: Machine-readable JSON output.
- **Example**:
  ```bash
  installmon events --type process_exec --limit 10
  ```
- **Security Note**: All command arguments shown in events are sanitized. See [Security & Sanitization](../security/sanitization.md).

---

### `installmon audit`
- **Purpose**: Run database integrity checks and audit tracked software against physical disk state.
- **Syntax**: `installmon audit [--json]`
- **Behavior**:
  1. Runs SQLite `PRAGMA integrity_check`.
  2. Verifies whether tracked primary binaries exist and remain executable.
  3. Identifies unclassified or pending workflows.
- **Example**:
  ```bash
  installmon audit
  ```
- **Output**:
  ```text
  === InstallMon System & Software Audit Report ===
  Database Integrity  : Passed (PRAGMA integrity_check ok)
  Software Audited    : 18
  Missing Binaries    : 0
  Unclassified Flows  : 0
  Pending Sessions    : 0
  ------------------------------------------------
  Status: ALL AUDIT CHECKS PASSED. No anomalies detected.
  ```

---

### `installmon export`
- **Purpose**: Export tracking records to JSON or JSON-Lines format.
- **Syntax**: `installmon export [OPTIONS]`
- **Options**:
  - `-f, --format <FORMAT>`: Output format: `json` (pretty JSON array) or `jsonl` (line-delimited JSON). Default: `json`.
  - `-e, --entity <ENTITY>`: Entity type: `all`, `events`, `sessions`, `software`. Default: `all`.
  - `-o, --out <PATH>`: Output file destination (default: writes to stdout).
- **Examples**:
  ```bash
  # Export software catalog as pretty JSON
  installmon export --entity software --format json -o software.json

  # Stream events as JSONL for SIEM ingestion (Splunk, Elastic, Loki)
  installmon export --entity events --format jsonl
  ```

---

### `installmon daemon`
- **Purpose**: Run the continuous monitoring daemon in the foreground.
- **Syntax**: `installmon daemon`
- **Behavior**:
  - Initializes inotify watches on target executable and configuration directories.
  - Starts periodic 800ms `/proc` polling for active installer commands.
  - Updates heartbeat state in `daemon.json` every 2 seconds.
  - Gracefully handles `SIGINT` and `SIGTERM` signals.

---

### `installmon service`
- **Purpose**: Manage continuous background daemon service lifecycle via systemd or standalone process mode.
- **Syntax**: `installmon service <ACTION>`
- **Subcommands**:
  - `status`: Show service state (`Running`, `Stopped`, `NotInstalled`), PID, uptime, and daemon heartbeat.
  - `setup` (alias: `install`): Install systemd unit file, reload daemon, enable auto-start, and start service.
  - `start`: Start the background service.
  - `stop`: Stop the background service.
  - `restart`: Restart the background service.
  - `enable`: Enable systemd auto-start on boot/login.
  - `disable`: Disable systemd auto-start.
  - `uninstall`: Stop service and remove systemd unit file.
- **Examples**:
  ```bash
  # Check service state
  installmon service status

  # Set up and enable background daemon for current user
  installmon service setup

  # Set up system-wide monitoring daemon (requires root)
  sudo installmon --system service setup
  ```
- **Related Guide**: See [Installation & Service Setup Guide](../installation/guide.md#running-as-a-user-vs-system-service).

---

## Related Documentation & Next Steps

- 🖥️ [Terminal User Interface (TUI) Guide](../tui/guide.md) — Explore the interactive visual dashboard.
- 🗄️ [Database Schema Specification](../database/schema.md) — Understand tables, indexes, and SQLite WAL configuration.
- 🔒 [Security & Sanitization](../security/sanitization.md) — Review ingest-time credential scrubbing rules.
- 🛠️ [Troubleshooting & FAQ](../troubleshooting/faq.md) — Common command issues and resolutions.

---

[← Back to Documentation Hub](../README.md) | [← Back to Main README](../../README.md)
