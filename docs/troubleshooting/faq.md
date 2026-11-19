# Troubleshooting & Frequently Asked Questions (FAQ)

[InstallMon](../../README.md) > [Documentation Hub](../README.md) > Troubleshooting & FAQ

This guide provides troubleshooting solutions for common permission, systemd, database locking, and display issues encountered when running InstallMon.

---

## Common Issues & Diagnoses

### 1. `Permission Denied` when opening database
- **Symptom**: `Failed to open database at /var/lib/installmon/installmon.db: Permission denied`.
- **Cause**: The command is attempting to access the system-wide database without root permissions (e.g. running with `--system` or without write access to `/var/lib`).
- **Solution**:
  - Run without `--system` to operate in unprivileged user mode (`~/.local/share/installmon`).
  - Or prepend `sudo` if system-wide monitoring is required:
    ```bash
    sudo installmon --system status
    ```

---

### 2. `Database is locked (5)` or busy timeout
- **Symptom**: `rusqlite::Error: database is locked (code 5)`.
- **Cause**: Another process holds an exclusive lock, or an unclosed transaction exists. InstallMon configures `PRAGMA busy_timeout = 5000;` to handle transient contention automatically, but long-lived locks can cause this error.
- **Solution**:
  1. Inspect running InstallMon processes:
     ```bash
     installmon service status
     # or
     pgrep -fl installmon
     ```
  2. If an orphaned daemon process is hanging, terminate it:
     ```bash
     killall -SIGTERM installmon
     ```
  3. Verify database integrity:
     ```bash
     installmon audit
     ```

---

### 3. TUI terminal rendering glitches or distorted colors
- **Symptom**: Text overlapping, misplaced borders, or missing colors in `installmon tui`.
- **Cause**: Terminal emulator missing 256-color or truecolor support, or incorrect `TERM` variable.
- **Solution**:
  - Ensure your terminal emulator supports truecolor (such as Kitty, Alacritty, WezTerm, iTerm2, modern GNOME Terminal).
  - Check your terminal environment:
    ```bash
    echo $TERM
    ```
    If set to `vt100` or plain `xterm`, export:
    ```bash
    export TERM=xterm-256color
    ```
  - Resize the terminal window slightly to force a Crossterm redraw event.

---

### 4. Background daemon not running after logout
- **Symptom**: The user daemon stops when logging out of an SSH session or closing the graphical desktop.
- **Cause**: By default, `systemd --user` processes are terminated when a user's last session closes unless **user lingering** is enabled.
- **Solution**:
  Enable lingering for your Linux user account:
  ```bash
  loginctl enable-linger $USER
  ```
  Verify lingering status:
  ```bash
  loginctl show-user $USER --property=Linger
  ```

---

### 5. Extremely fast commands (< 800ms) are not caught by procfs
- **Symptom**: Executing `gcc main.c` directly without an installer wrapper completes in 120ms and is not listed in `installmon events`.
- **Cause**: Active process observation currently polls `/proc` on an 800ms loop. Short-lived commands that execute and terminate within < 800ms can evade discovery.
- **Solution**:
  - Filesystem modifications created by fast commands in watched directories (`~/.local/bin`, `/usr/local/bin`) **are still detected via inotify**.
  - Future releases include an **eBPF/BTF tracepoint engine** that will eliminate this polling window by capturing process execution events via kernel tracepoints. See the [Roadmap](../roadmap.md).

---

### 6. Binaries placed in custom folders are not indexed
- **Symptom**: Software extracted into `/opt/my-app/bin` is not showing up in `installmon list`.
- **Cause**: By default, inotify watches system and user PATH directories. Arbitrary custom paths outside PATH are not watched automatically.
- **Solution**:
  Run an on-demand scan pointing directly to your custom path:
  ```bash
  installmon scan /opt/my-app/bin
  ```

---

## Frequently Asked Questions (FAQ)

<details>
<summary><b>Does InstallMon require root privileges to run?</b></summary>

**No.** InstallMon is designed unprivileged-by-default. When launched without `sudo`, it monitors your user environment (`~/.local/bin`, `~/.cargo/bin`), uses user systemd units (`systemd --user`), and persists into `~/.local/share/installmon/installmon.db`. Root is only required if you explicitly choose to monitor system directories like `/usr/bin` via `--system`.
</details>

<details>
<summary><b>Can InstallMon leak my passwords or API keys?</b></summary>

**No.** InstallMon features an ingest-time argument sanitizer that scrubs passwords (`--password`), API keys (`--token`, `--api-key`), URL credentials (`https://user:pass@host`), HTTP Authorization headers (`Bearer eyJ...`), and PEM private keys before records touch the database. See [Security & Sanitization](../security/sanitization.md).
</details>

<details>
<summary><b>Does InstallMon slow down my package manager or compiler?</b></summary>

**No.** InstallMon does not wrap or intercept commands. It operates strictly out-of-band via Linux `inotify` filesystem notifications and asynchronous `/proc` polling.
</details>

<details>
<summary><b>Can I run InstallMon inside Docker or WSL2?</b></summary>

**Yes.** InstallMon has been verified in bare-metal Linux, WSL2, Docker, and Podman containers. In containerized environments where systemd is absent, use standalone daemon mode (`installmon daemon` or press `b` in the TUI Service tab).
</details>

---

## Related Documentation & Next Steps

- 📦 [Installation & Lifecycle Guide](../installation/guide.md) — Service setup and uninstallation instructions.
- 💻 [CLI Command Reference](../cli/reference.md) — Reference for `installmon audit` and `installmon status`.
- 🗄️ [Database Schema Specification](../database/schema.md) — SQLite pragmas, busy timeouts, and WAL mode.

---

[← Back to Documentation Hub](../README.md) | [← Back to Main README](../../README.md)
