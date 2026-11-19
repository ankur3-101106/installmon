# Security & Argument Sanitization: InstallMon

[InstallMon](../../README.md) > [Documentation Hub](../README.md) > Security & Sanitization

Security and privacy are core architectural requirements for InstallMon. When monitoring process execution and filesystem modifications on a Linux machine, commands frequently contain sensitive secrets (such as automated scripts, developer API tokens, or package manager flags). Persisting raw command strings could create credential leaks.

---

## 1. The Core Principle

> **"Never persist plaintext passwords, tokens, private keys, or raw authorization credentials to disk."**

InstallMon operates an **ingest-time redaction filter** within `installmon-core`. Before any process execution event is committed to the SQLite database or exposed to the user interface, arguments pass through `sanitize_arguments`.

---

## 2. Redaction Demonstration

```text
[Raw Command Ingestion from /proc]:
  curl -u deployer:super_secret_password \
       -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..." \
       --token=ghp_ABC123XYZSecretToken \
       https://user:gitlab_token_123@gitlab.com/internal/repo.git

[Sanitized Event Persisted to SQLite Database]:
  curl -u [REDACTED] \
       -H "Authorization: Bearer [REDACTED]" \
       --token=[REDACTED] \
       https://user:[REDACTED]@gitlab.com/internal/repo.git
```

---

## 3. Scrubbing Rules & Regex Filters

InstallMon applies 5 distinct redaction passes across argument vectors:

| Target Pattern | Examples Detected | Redacted Output |
| :--- | :--- | :--- |
| **Separate Flag Values** | `--password secret`, `-p 12345`, `--token abc`, `--api-key key`, `--auth creds`, `--secret val`, `--private-key path`, `-u user:pass` | `--password [REDACTED]`, `--token [REDACTED]` |
| **Inline Flag Arguments** | `--password=secret`, `--token=abc`, `--secret=xyz`, `--api-key=123` | `--password=[REDACTED]`, `--token=[REDACTED]` |
| **Embedded URL Credentials** | `https://user:password@host/repo.git`, `http://token@domain.com` | `https://user:[REDACTED]@host/repo.git` |
| **HTTP Authorization Headers** | `Authorization: Bearer <token>`, `Authorization: Basic <base64>` | `Authorization: Bearer [REDACTED]` |
| **Private Key Blocks** | `-----BEGIN RSA PRIVATE KEY-----...` | `[REDACTED_PRIVATE_KEY]` |

Because sanitization preserves command names, flags, and targets while scrubbing secret values, auditability and process correlation remain intact without compromising security.

---

## 4. Principle of Least Privilege

- **Unprivileged by Default**: InstallMon does not require root permissions. Standard users can monitor user-level installations (`~/.local/bin`, `~/.cargo/bin`) and manage user systemd units without `sudo`.
- **Read-Only Observability**: InstallMon observes and logs events; it never executes arbitrary user commands or alters installation workflows.
- **Scope Isolation**: In user mode, InstallMon cannot observe actions executed by other users, privileged system modifications in `/usr/bin`, or manage system-wide systemd units.

---

## 5. Offline-First Network Guarantee

- **Zero Outbound Telemetry**: InstallMon contains **zero** analytics, crash reporting, telemetry, or remote telemetry pings.
- **Local Storage**: All data remains entirely within your local SQLite database file.
- **Offline Operation**: InstallMon functions identically in isolated, air-gapped, or firewall-restricted environments.

---

## 6. Technical Limitations & Security Assumptions

> [!WARNING]
> While regex sanitization catches standard options and patterns, custom positional arguments or non-standard flag names cannot be 100% guaranteed to be detected. Avoid passing plaintext credentials directly on command lines.

---

## Related Documentation & Next Steps

- 🏗️ [Architecture Overview](../architecture/overview.md) — How the sanitization filter fits into event ingestion.
- 🗄️ [Database Schema Specification](../database/schema.md) — SQLite schema and argument storage formats.
- 💻 [CLI Command Reference](../cli/reference.md) — Inspecting events with `installmon events`.

---

[← Back to Documentation Hub](../README.md) | [← Back to Main README](../../README.md)
