# Development & Contributing Guide: InstallMon

[InstallMon](../../README.md) > [Documentation Hub](../README.md) > Development Guide

Thank you for your interest in contributing to InstallMon! This guide details the local development setup, code formatting standards, linting requirements, test suites, and release packaging procedures.

---

## Toolchain & Environment Requirements

- **Language**: Rust (Stable channel).
- **Minimum Supported Rust Version (MSRV)**: **1.80.0** (due to `std::sync::LazyLock`).
- **Platform**: Linux (x86_64 or AArch64). WSL2 is fully supported for local development.
- **Dependencies**: SQLite3 development headers are not strictly required because `rusqlite` is configured with the `bundled` feature.

```bash
# Check Rust compiler version
rustc --version
# Output must be >= 1.80.0
```

---

## Workspace Structure

InstallMon is organized as a unified Cargo workspace across 5 modular crates:

```text
installmon/
├── Cargo.toml                  # Workspace manifest (MSRV 1.80.0, license, repo URL)
├── crates/
│   ├── core/                   # Event models, provenance types, sensitive argument sanitizer
│   ├── platform/               # Capability detector, OS detection, path resolution, service manager
│   ├── database/               # SQLite connection, WAL pragmas, schema migrations (v1)
│   ├── tui/                    # Ratatui terminal dashboard, 6 views, keyboard navigation
│   └── cli/                    # CLI entrypoint (installmon), subcommands, daemon loop
├── docs/                       # Technical documentation and architecture guides
└── scripts/
    ├── install.sh              # Unified interactive installation and lifecycle assistant
    └── build-release.sh        # Multi-architecture release packaging script
```

---

## Development Workflow

### 1. Verification & Compilation
```bash
# Type check all crates in workspace
cargo check --workspace

# Build unoptimized debug binaries
cargo build --workspace

# Build optimized release binaries
cargo build --workspace --release
```

### 2. Code Formatting Standards
InstallMon enforces standard Rust formatting across all files:
```bash
# Verify formatting (fails CI if unformatted)
cargo fmt --all -- --check

# Auto-format all code
cargo fmt --all
```

### 3. Clippy Lints
All code must compile with **zero warnings** under strict Clippy checks:
```bash
cargo clippy --workspace --all-targets -- -D warnings
```

### 4. Running Test Suites
InstallMon maintains unit tests and integration tests covering the sanitizer, platform detection, database migrations, and event persistence:
```bash
# Run all workspace unit and integration tests
cargo test --workspace --verbose

# Run tests for a specific crate
cargo test -p installmon-core
cargo test -p installmon-database
cargo test -p installmon-platform
```

---

## Local Smoke Testing

Before submitting changes, test the release binary directly:

```bash
# Compile release binary
cargo build --workspace --release

# Run smoke test commands against a temporary database
./target/release/installmon --version
./target/release/installmon --help
./target/release/installmon --database /tmp/dev-test.db status
./target/release/installmon --database /tmp/dev-test.db capabilities
./target/release/installmon --database /tmp/dev-test.db scan
./target/release/installmon --database /tmp/dev-test.db audit

# Clean up
rm -f /tmp/dev-test.db*
```

---

## Release Packaging Script

The repository includes a standalone packaging script in `scripts/build-release.sh`:

```bash
# Package for host architecture
./scripts/build-release.sh

# Or specify target triple and release tag
./scripts/build-release.sh x86_64-unknown-linux-musl v0.1.0
```

The script will:
1. Compile the release binary using `cargo` (or `cross` for cross-compilation).
2. Strip debug symbols using `strip` or `llvm-strip`.
3. Package `installmon`, `README.md`, and `LICENSE` into a versioned `.tar.gz` archive.
4. Generate corresponding `.sha256` checksums.

---

## Continuous Integration (CI)

Every pull request and push to `main` triggers automated CI via [.github/workflows/ci.yml](../../.github/workflows/ci.yml):
1. **Formatting & Lints**: Runs `cargo fmt --check` and `cargo clippy -- -D warnings`.
2. **Unit & Integration Tests**: Runs `cargo test --workspace`.
3. **Build & Smoke Tests**: Compiles `--release` binary and executes CLI smoke commands.

Ensure all local checks pass before pushing to GitHub.

---

## Conventional Commit Standards

Please format commit messages according to conventional commit guidelines:
- `feat:` New features or capabilities.
- `fix:` Bug fixes and corrections.
- `docs:` Documentation additions or updates.
- `ci:` GitHub Actions workflow updates.
- `chore:` Dependency or metadata maintenance.

---

## Related Documentation & Next Steps

- 🏗️ [Architecture Overview](../architecture/overview.md) — Internal subsystems and component interactions.
- 🗄️ [Database Schema Specification](../database/schema.md) — Schema migrations and entity models.
- 🗺️ [Project Roadmap](../roadmap.md) — Active features and planned future contributions.

---

[← Back to Documentation Hub](../README.md) | [← Back to Main README](../../README.md)
