# InstallMon

[![License](https://img.shields.io/badge/license-GPL--3.0-blue.svg)](LICENSE)
[![Language](https://img.shields.io/badge/rust-1.99%2B-orange.svg)](https://www.rust-lang.org/)
[![Status](https://img.shields.io/badge/version-0.1.0-green.svg)]()
[![Platform](https://img.shields.io/badge/platform-Linux%20(Universal)-brightgreen.svg)]()

> Lightweight, Linux-native software installation monitoring and auditing system written entirely in Rust.

**InstallMon** observes how software enters, gets installed, and changes within a Linux environment. It tracks installation workflows, reconstructs their execution history, identifies installed software, and maintains installation provenance.

---

## Core Principle

> *"Track how software enters and changes within a Linux environment, independently of its distribution, package manager, installation method, or CPU architecture."*

### What InstallMon Is NOT:
- **Not** a package manager or wrapper around pacman, apt, dnf, or flatpak.
- **Not** an installer or dependency resolver.
- **Not** a tool that modifies or deletes software automatically.

Its purpose is **software installation observability, lifecycle auditing, and provenance tracking**.

---

## Features & Highlights

- **Interactive Ratatui TUI**:
  - Full-featured terminal interface with 5 primary views: Dashboard, Software, Sessions, Events, and Audit.
  - Live inspection of software packages, disk existence verification, process ancestry, and preserved evidence.
- **Universal Linux Compatibility**:
  - Available as completely static, standalone binaries with zero dynamic library dependencies.
  - Compatible out-of-the-box on Arch, Debian, Ubuntu, Fedora, Alpine (musl), Void, openSUSE, RHEL, and embedded Linux.
- **Detection Priority**:
  1. *Priority 1*: Installation method monitoring (process ancestry, scripts, downloads, builds, deployments).
  2. *Priority 2*: Filesystem correlation (creations, symlinks, mutations, permissions).
  3. *Priority 3*: Binary discovery fallback (PATH scanning, AppImages, untracked executables).
- **Capability-Aware & Graceful Degradation**:
  - Dynamically probes kernel BTF/eBPF, Linux Audit, fanotify, inotify, procfs fallback, and container/WSL environments.
  - Automatically negotiates the most optimal active monitoring backends.
- **Security & Privacy First**:
  - Built-in credential & token sanitization ensures passwords, API keys, Authorization headers, and private keys are never written to disk or logs.
  - Zero arbitrary command execution; read-only observability.
- **Robust Local Persistence**:
  - Embedded SQLite storage operating in WAL mode with transactional batch persistence and automatic schema migrations.
  - Offline-first: requires zero network connectivity.

---

## Workspace Structure

The project is structured as a modular Cargo workspace:

```text
installmon/
├── Cargo.toml
├── LICENSE
├── README.md
├── .cargo/config.toml  # Static linkage configuration
├── .github/workflows/
│   └── release.yml     # Multi-arch automated release matrix pipeline
├── crates/
│   ├── core/           # Normalized event model, provenance, and argument sanitizer
│   ├── platform/       # Kernel capability, init system, and environment detection
│   ├── database/       # SQLite persistence, schema migrations, and transactional storage
│   ├── tui/            # Ratatui & Crossterm terminal user interface
│   └── cli/            # Command-line interface binary (installmon)
├── docs/
│   ├── architecture/   # Architecture overview and design specifications
│   ├── database/       # Database schema and index documentation
│   ├── security/       # Argument sanitization and least-privilege security model
│   └── compatibility/  # Linux kernel, libc, and init system compatibility matrix
├── scripts/
│   └── build-release.sh # Release build and packaging script
└── tests/              # Multi-stage integration and regression tests
```

---

## Interactive TUI Mode

Launch the full interactive Terminal User Interface:

```bash
installmon tui
# or simply invoke without arguments:
installmon
```

### TUI Keybindings

| Keybinding | Action |
| :--- | :--- |
| `1` .. `5` / `Tab` | Switch between Dashboard, Software, Sessions, Events, Audit |
| `↑` / `↓` / `j` / `k` | Navigate items in tables and lists |
| `s` | Trigger system PATH discovery scan |
| `r` | Refresh database state and capabilities |
| `?` | Toggle Help modal overlay |
| `q` / `Esc` / `Ctrl+C` | Quit InstallMon TUI |

---

## Multi-Architecture Compatibility Matrix

Static release binaries are built for all major Linux CPU architectures:

| Architecture | Target Triple | Target Platforms |
| :--- | :--- | :--- |
| **x86_64** | `x86_64-unknown-linux-musl` | Intel / AMD 64-bit PCs, Servers, VMs |
| **AArch64** | `aarch64-unknown-linux-musl` | 64-bit ARM, Raspberry Pi 4/5, AWS Graviton, Apple Silicon Linux |
| **ARMv7** | `armv7-unknown-linux-musleabihf` | 32-bit ARM Hard-Float, Raspberry Pi 2/3 |
| **RISC-V** | `riscv64gc-unknown-linux-gnu` | 64-bit RISC-V SBCs and development boards |
| **PowerPC** | `powerpc64le-unknown-linux-gnu` | 64-bit Little Endian POWER8/9/10 |
| **s390x** | `s390x-unknown-linux-gnu` | IBM System z Linux Mainframes |

---

## Getting Started

### Prerequisites

- Linux (x86_64, AArch64, RISC-V, etc.)
- Rust 1.80+ (tested on Rust 1.99+)
- Standard Linux toolchain

### Building from Source

```bash
cargo build --release
```

The release binary will be placed at `target/release/installmon`.

### Running Tests

```bash
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

---

## CLI Usage

Inspect system status:
```bash
installmon status
# or for JSON output:
installmon status --json
```

Probe host and kernel capabilities:
```bash
installmon capabilities
```

Perform binary discovery scan across standard directories:
```bash
installmon scan
```

List tracked software installations:
```bash
installmon list
```

Inspect detailed provenance of a software package:
```bash
installmon inspect <software-name>
```

Audit system integrity and check for anomalies:
```bash
installmon audit
```

Export tracking records to JSON or JSON-Lines:
```bash
installmon export --entity all --format json
```
