# InstallMon Documentation Hub

[![License: GPL-3.0-only](https://img.shields.io/badge/License-GPL--3.0--only-blue.svg)](../LICENSE)
[![Platform: Linux](https://img.shields.io/badge/Platform-Linux-brightgreen.svg)]()
[![Rust: 1.80.0+](https://img.shields.io/badge/Rust-1.80.0%2B-orange.svg)](https://www.rust-lang.org/)

Welcome to the **InstallMon Documentation Hub**. This directory provides technical references, user guides, architecture specifications, and operational manuals for InstallMon.

---

## 🧭 Guided Reading Paths

Choose the path that best matches your immediate goal:

```mermaid
flowchart TD
    Start["Where do you want to start?"]
    Start --> Path1["🚀 Getting Started"]
    Start --> Path2["🔧 Operator / SysAdmin"]
    Start --> Path3["🏗️ Architecture & Security"]
    Start --> Path4["💻 Developer / Contributor"]

    Path1 --> D_Inst["Installation Guide<br/>(docs/installation/guide.md)"]
    D_Inst --> D_CLI["CLI Reference<br/>(docs/cli/reference.md)"]
    D_CLI --> D_TUI["TUI Guide<br/>(docs/tui/guide.md)"]

    Path2 --> D_Daemon["Service Management & systemd<br/>(docs/installation/guide.md#service-setup)"]
    D_Daemon --> D_Compat["Compatibility Matrix<br/>(docs/compatibility/matrix.md)"]
    D_Compat --> D_Trouble["Troubleshooting & FAQ<br/>(docs/troubleshooting/faq.md)"]

    Path3 --> D_Arch["Architecture Overview<br/>(docs/architecture/overview.md)"]
    D_Arch --> D_DB["Database Schema & WAL<br/>(docs/database/schema.md)"]
    D_DB --> D_Sec["Security & Sanitization<br/>(docs/security/sanitization.md)"]

    Path4 --> D_Dev["Contributing Guide<br/>(docs/development/contributing.md)"]
    D_Dev --> D_Road["Project Roadmap<br/>(docs/roadmap.md)"]
```

---

## 📚 Complete Documentation Index

| Section | Document | Description | Target Audience |
| :--- | :--- | :--- | :--- |
| **Getting Started** | [📦 Installation & Lifecycle](installation/guide.md) | Single-command installer, prebuilt archives, checksums, user vs systemd daemon setup, and uninstallation. | All Users |
| **Interfaces** | [💻 CLI Command Reference](cli/reference.md) | Syntax, options, examples, and JSON output formats for all 12 CLI subcommands. | Users, Automation Scripts |
| **Interfaces** | [🖥️ Terminal UI (TUI) Guide](tui/guide.md) | Ratatui terminal dashboard, 6 views, keyboard shortcuts, and centralized service management. | Interactive Users |
| **System Design** | [🏗️ Architecture Overview](architecture/overview.md) | Pipeline breakdown, event normalization, session correlation, and Cargo workspace crate roles. | Architects, Developers |
| **Storage** | [🗄️ Database & Schema](database/schema.md) | Embedded SQLite in WAL mode, ER diagrams, versioned migrations, retention, and maintenance. | Operators, Developers |
| **Security** | [🔒 Security & Sanitization](security/sanitization.md) | Least privilege design, credential scrubbing rules, regex filters, and audit boundaries. | Security Engineers |
| **Compatibility**| [🐧 Compatibility Matrix](compatibility/matrix.md) | Active vs probed capabilities (eBPF, fanotify, audit, procfs, inotify), distro and libc support. | SysAdmins, Integrators |
| **Operations** | [🛠️ Troubleshooting & FAQ](troubleshooting/faq.md) | Solutions for permission issues, locked databases, terminal glitches, and daemon lifecycle. | Operators, Users |
| **Development**| [🚀 Development & Contributing](development/contributing.md) | Toolchain requirements, cargo build/test/clippy workflows, smoke testing, and CI/CD pipelines. | Contributors |
| **Evolution** | [🗺️ Project Roadmap](roadmap.md) | Currently implemented features vs planned future capabilities (eBPF, netlink audit, fanotify). | Community |

---

## ⚡ Quick Common Tasks

<details>
<summary><b>How do I install InstallMon with one command?</b></summary>

Run the interactive lifecycle assistant:
```bash
curl -fsSL https://raw.githubusercontent.com/ankur3-101106/installmon/main/install.sh | bash
```
See the full [Installation Guide](installation/guide.md).
</details>

<details>
<summary><b>How do I launch the interactive terminal dashboard?</b></summary>

Simply run:
```bash
installmon
# or explicitly:
installmon tui
```
See the full [TUI Guide](tui/guide.md).
</details>

<details>
<summary><b>How do I export records to JSON or SIEM log pipelines?</b></summary>

Export events as JSON-Lines:
```bash
installmon export --entity events --format jsonl -o events.jsonl
```
See the full [CLI Command Reference](cli/reference.md#installmon-export).
</details>

<details>
<summary><b>How does InstallMon protect sensitive credentials?</b></summary>

All process arguments are scrubbed at ingestion time. Passwords, API tokens, Authorization headers, and private keys are redacted to `[REDACTED]` before records touch SQLite.
See the full [Security & Sanitization Guide](security/sanitization.md).
</details>

---

[← Back to Main README](../README.md)
