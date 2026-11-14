# InstallMon

[![License](https://img.shields.io/badge/license-GPL--3.0-blue.svg)](LICENSE)
[![Language](https://img.shields.io/badge/rust-1.99%2B-orange.svg)](https://www.rust-lang.org/)
[![Status](https://img.shields.io/badge/version-0.1.0--dev-green.svg)]()

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

## Architectural Highlights

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
├── crates/
│   ├── core/           # Normalized event model, provenance, and argument sanitizer
│   ├── platform/       # Kernel capability, init system, and environment detection
│   ├── database/       # SQLite persistence, schema migrations, and transactional storage
│   └── cli/            # Command-line interface binary (installmon)
├── docs/
│   ├── architecture/   # Architecture overview and design specifications
│   ├── database/       # Database schema and index documentation
│   ├── security/       # Argument sanitization and least-privilege security model
│   └── compatibility/  # Linux kernel, libc, and init system compatibility matrix
└── tests/              # Multi-stage integration and regression tests
```

---

## Getting Started

### Prerequisites

- Linux (x86_64, AArch64, RISC-V, etc.)
- Rust 1.80+ (tested on Rust 1.99+)
- Standard Linux toolchain

### Building the Workspace

```bash
cargo build --release
```

The `installmon` binary will be produced in `target/release/installmon`.

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
cargo run --bin installmon -- status
# or for JSON output:
cargo run --bin installmon -- status --json
```

Probe host and kernel capabilities:
```bash
cargo run --bin installmon -- capabilities
```

Perform binary discovery scan across standard directories:
```bash
cargo run --bin installmon -- scan
```

List tracked software installations:
```bash
cargo run --bin installmon -- list
```

Inspect detailed provenance of a software package:
```bash
cargo run --bin installmon -- inspect <software-name>
```

Audit system integrity and check for anomalies:
```bash
cargo run --bin installmon -- audit
```

Export tracking records to JSON or JSON-Lines:
```bash
cargo run --bin installmon -- export --entity all --format json
```
