# Terminal User Interface (TUI) Guide: InstallMon

[InstallMon](../../README.md) > [Documentation Hub](../README.md) > Terminal UI Guide

InstallMon includes a built-in terminal dashboard developed with [Ratatui](https://github.com/ratatui/ratatui) and [Crossterm](https://github.com/crossterm-rs/crossterm). It provides a responsive, single-pane-of-glass overview of host software installations, system sessions, raw lifecycle events, database integrity, and background service controls.

---

## Launching the TUI

Run `installmon` without arguments, or invoke the explicit `tui` subcommand:

```bash
installmon
# or explicitly:
installmon tui
```

To run against a custom database or in system-wide mode:
```bash
# Explicit database file
installmon --database /path/to/custom.db tui

# System-wide mode (/var/lib/installmon)
sudo installmon --system tui
```

---

## Available Views & Tabs

The interface is organized into 6 tabs, accessible via keys `1` through `6` or `Tab` / `BackTab`:

```text
┌────────────────────────────────────────────────────────────────────────┐
│ InstallMon │ 1:Dashboard │ 2:Software │ 3:Sessions │ 4:Events │ 5:Audit │ 6:Service │
├────────────────────────────────────────────────────────────────────────┤
│                                                                        │
│                       [ Active View Content ]                          │
│                                                                        │
├────────────────────────────────────────────────────────────────────────┤
│ [1..6] Tabs | [j/k] Scroll | [s] Scan | [r] Refresh | [?] Help | [q] Exit│
└────────────────────────────────────────────────────────────────────────┘
```

### 1. Dashboard View (`Tab 1`)
- **Purpose**: System-level health and metrics snapshot.
- **Components**:
  - **Host Information**: Operating system, kernel release, CPU architecture, virtualization environment (WSL2, Docker, Bare Metal), and init system.
  - **Negotiated Backends**: Indicates active process (`procfs`) and filesystem (`inotify`) monitoring engines.
  - **Database Counters**: Live tally of observed events, tracked installation sessions, and discovered software packages.
  - **Recent Activity Feed**: Chronological log of recent lifecycle observations.

### 2. Software View (`Tab 2`)
- **Purpose**: Interactive software catalog.
- **Columns**: `Name`, `Scope` (`User` / `System`), `Discovered At`, and `Primary Binary Path`.
- **Navigation**: Use `j` / `↓` and `k` / `↑` to scroll through tracked applications.

### 3. Sessions View (`Tab 3`)
- **Purpose**: Inspection of recorded installation workflows.
- **Columns**: `Session ID`, `Classification` (`distribution_package`, `language_ecosystem`, `direct_download`, `source_build`, `script_execution`), `Root PID`, `Scope`, and `Status`.
- **Details**: Highlights the initiating command line and active user context.

### 4. Events View (`Tab 4`)
- **Purpose**: Low-level stream of normalized lifecycle events.
- **Columns**: `Timestamp`, `Event Type` (`process_exec`, `file_create`, `file_modify`, `file_delete`, `file_rename`), `PID`, `EUID`, and `Executable / Target Path`.
- **Privacy Assurance**: All displayed arguments are scrubbed at ingestion time. See [Security & Sanitization](../security/sanitization.md).

### 5. Audit View (`Tab 5`)
- **Purpose**: Live host integrity verification.
- **Components**:
  - **SQLite Integrity**: Real-time status of `PRAGMA integrity_check`.
  - **Missing Binary Warnings**: Flags software entries whose registered binaries have been deleted or moved.
  - **Workflow Anomaly Checks**: Reports sessions classified as `unknown` or unfinished.

### 6. Service View (`Tab 6`)
- **Purpose**: Interactive daemon operations panel.
- **Components**:
  - **Daemon State**: `Running`, `Stopped`, or `Not Installed`.
  - **Heartbeat & Telemetry**: Daemon PID, uptime counter, and recent event counters read from `daemon.json`.
  - **Direct Controls**: Trigger setup, start, stop, restart, auto-start toggle, or spawn detached standalone processes with one keystroke.

---

## Keyboard Navigation & Keybindings Reference

| Key | Scope | Action | Description |
| :---: | :---: | :--- | :--- |
| `1` .. `6` | Global | Switch Tab | Jump directly to Tab 1 (`Dashboard`) through Tab 6 (`Service`). |
| `Tab` / `l` / `→` | Global | Next Tab | Cycle forward to the next tab. |
| `BackTab` / `h` / `←` | Global | Previous Tab | Cycle backward to the previous tab. |
| `j` / `↓` | Tables | Move Down | Select the next row in Software, Sessions, or Events. |
| `k` / `↑` | Tables | Move Up | Select the previous row in Software, Sessions, or Events. |
| `s` | Global | PATH Scan | Trigger an on-demand executable discovery scan across PATH. |
| `r` | Global | Refresh | Manually reload database records and daemon heartbeat. |
| `I` / `i` | Global | Setup Service | Install systemd unit, enable boot auto-start, and start daemon. |
| `S` | Global | Toggle Service | Start active daemon if stopped, or stop if running. |
| `R` | Global | Restart Service | Restart background monitoring service. |
| `e` | Service Tab | Toggle Auto-Start | Enable or disable systemd auto-start on boot/login. |
| `b` | Service Tab | Spawn Detached | Spawn background daemon process (standalone mode without systemd). |
| `u` | Service Tab | Uninstall Unit | Stop service and remove systemd unit file. |
| `?` | Global | Help Overlay | Display interactive keybinding reference modal. |
| `q` / `Ctrl+C` | Global | Exit TUI | Quit TUI. **The background daemon continues running uninterrupted.** |

---

## Managing Background Monitoring from the TUI

You do not need to drop to the shell to control the InstallMon daemon:

1. **First-Time Setup**: Press `I` anywhere in the TUI. InstallMon generates the systemd service file, reloads systemd, enables auto-start, and launches the daemon.
2. **Checking Daemon Health**: Navigate to Tab `6` (`Service`). The panel displays the live heartbeat, uptime, and event throughput.
3. **Running in Containers / Non-systemd Environments**: Press `b` in the Service tab to spawn a detached background process that continues monitoring even after closing the terminal.

---

## Related Documentation & Next Steps

- 💻 [CLI Command Reference](../cli/reference.md) — Command-line equivalents for all TUI operations.
- 📦 [Installation & Service Setup Guide](../installation/guide.md) — Systemd unit paths and manual service configuration.
- 🛠️ [Troubleshooting & FAQ](../troubleshooting/faq.md) — Terminal truecolor and rendering issue resolutions.

---

[← Back to Documentation Hub](../README.md) | [← Back to Main README](../../README.md)
