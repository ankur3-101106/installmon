# Installation & Lifecycle Guide: InstallMon

[InstallMon](../../README.md) > [Documentation Hub](../README.md) > Installation Guide

InstallMon is distributed as universal, standalone Linux executables with zero external dependencies, pre-compiled for both **x86_64** (Intel/AMD) and **AArch64** (ARM 64-bit).

---

## 1. Unified Interactive Lifecycle Assistant (Recommended)

InstallMon provides a single interactive script that handles **installation, updates, system diagnostics, and uninstallation**:

- Detects CPU architecture (`x86_64`, `aarch64`) and environment (Bare-metal, WSL2, Docker).
- Detects host distribution and C library (`glibc` or `musl`).
- Prompts for installation scope:
  - **User Mode (`~/.local/bin`)**: Recommended, requires no root or `sudo` permissions.
  - **System Mode (`/usr/local/bin`)**: System-wide installation for all users.
- Prompts to configure the continuous background daemon.
- Downloads the appropriate static binary from GitHub Releases and verifies its cryptographic SHA-256 checksum.

### Run with a Single Command:
```bash
curl -fsSL https://raw.githubusercontent.com/ankur3-101106/installmon/main/install.sh | bash
```

### Run from Local Repository Clone:
```bash
./install.sh
```

---

## 2. Downloading Prebuilt Release Archives

Official pre-built archives and checksums are published on the [GitHub Releases](https://github.com/ankur3-101106/installmon/releases) page.

### For x86_64 (Universal Statically Linked musl - Recommended):
```bash
VERSION="v0.1.0"

# 1. Download binary archive and checksums
curl -LO "https://github.com/ankur3-101106/installmon/releases/download/${VERSION}/installmon-${VERSION}-x86_64-unknown-linux-musl.tar.gz"
curl -LO "https://github.com/ankur3-101106/installmon/releases/download/${VERSION}/SHA256SUMS"

# 2. Verify SHA-256 checksum
sha256sum --ignore-missing --check SHA256SUMS

# 3. Unpack archive
tar -xzf "installmon-${VERSION}-x86_64-unknown-linux-musl.tar.gz"

# 4. Place binary in PATH
# For user mode (no sudo):
mkdir -p ~/.local/bin && mv installmon ~/.local/bin/

# Or for system mode:
sudo install -m 0755 installmon /usr/local/bin/installmon
```

### For AArch64 (ARM 64-bit / Raspberry Pi 4/5 / AWS Graviton):
```bash
VERSION="v0.1.0"

curl -LO "https://github.com/ankur3-101106/installmon/releases/download/${VERSION}/installmon-${VERSION}-aarch64-unknown-linux-musl.tar.gz"
curl -LO "https://github.com/ankur3-101106/installmon/releases/download/${VERSION}/SHA256SUMS"

sha256sum --ignore-missing --check SHA256SUMS

tar -xzf "installmon-${VERSION}-aarch64-unknown-linux-musl.tar.gz"
sudo install -m 0755 installmon /usr/local/bin/installmon
```

---

## 3. Building From Source

InstallMon requires Rust **1.80.0** or higher (MSRV) due to modern standard library features (such as `std::sync::LazyLock`).

```bash
# 1. Clone repository
git clone https://github.com/ankur3-101106/installmon.git
cd installmon

# 2. Compile release binary
cargo build --workspace --release

# 3. Verify compiled binary
./target/release/installmon --version

# 4. Install to PATH
mkdir -p ~/.local/bin
cp target/release/installmon ~/.local/bin/
```

### Installing via Cargo:
Install directly from source into `~/.cargo/bin`:
```bash
cargo install --path crates/cli
```

---

## 4. Running as a User vs System Service

InstallMon can run as an unprivileged user daemon, a system-wide systemd service, or a standalone detached background process:

| Mode | Database Path | Watched Binary Dirs | Unit File Path | Permissions Required |
| :--- | :--- | :--- | :--- | :--- |
| **User Mode** | `~/.local/share/installmon/installmon.db` | `~/.local/bin`, `~/.cargo/bin`, etc. | `~/.config/systemd/user/installmon.service` | Unprivileged (No root) |
| **System Mode** | `/var/lib/installmon/installmon.db` | `/usr/bin`, `/usr/local/bin`, `/opt`, etc. | `/etc/systemd/system/installmon.service` | Privileged (`root` / `sudo`) |
| **Standalone Mode** | Configurable via `--database` | Configurable | None (Detached process) | Dependent on database path |

### Setting Up User Service:
```bash
# Sets up user unit and enables auto-start on login
installmon service setup

# Verify status
installmon service status
```

### Setting Up System Service:
```bash
# Sets up system unit (requires root)
sudo installmon --system service setup

# Verify status
sudo installmon --system service status
```

### Running Standalone (WSL2, Containers, Non-systemd):
```bash
# Run in foreground:
installmon daemon

# Or launch detached from TUI:
installmon tui
# Navigate to Tab 6 (Service) and press 'b'
```

---

## 5. Uninstallation & Data Cleanup

### Using Unified Script:
Run the script and select option `2` (`Uninstall`):
```bash
curl -fsSL https://raw.githubusercontent.com/ankur3-101106/installmon/main/install.sh | bash
```

The uninstaller will:
1. Stop running background services and systemd units.
2. Remove systemd service configuration files.
3. Remove the `installmon` executable binary.
4. Prompt whether you wish to preserve or completely purge historical tracking databases (`~/.local/share/installmon`).

### Manual Removal:
```bash
# Stop and remove user service
systemctl --user stop installmon 2>/dev/null || true
rm -f ~/.config/systemd/user/installmon.service
systemctl --user daemon-reload 2>/dev/null || true

# Remove binary
rm -f ~/.local/bin/installmon /usr/local/bin/installmon

# (Optional) Delete database and tracking records
rm -rf ~/.local/share/installmon
```

---

## Related Documentation & Next Steps

- 💻 [CLI Command Reference](../cli/reference.md) — All available subcommands and flags.
- 🐧 [Platform Compatibility Matrix](../compatibility/matrix.md) — Linux kernel, distribution, and container compatibility.
- 🛠️ [Troubleshooting & FAQ](../troubleshooting/faq.md) — Service lingering, database locks, and common permission issues.

---

[← Back to Documentation Hub](../README.md) | [← Back to Main README](../../README.md)
