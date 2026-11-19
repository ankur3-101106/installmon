# Database Schema Specification: InstallMon

[InstallMon](../../README.md) > [Documentation Hub](../README.md) > Database Schema

InstallMon utilizes an embedded SQLite engine for zero-configuration, thread-safe local data storage. It operates in Write-Ahead Logging (`WAL`) mode to support concurrent reading and writing without lock contention.

---

## 1. Entity-Relationship Diagram

```mermaid
erDiagram
    MONITORING_CAPABILITIES {
        string id PK
        string probed_at
        string kernel_release
        string architecture
        string distribution
        string init_system
        string environment
        string active_process_backend
        string active_filesystem_backend
        string raw_json
    }

    SESSIONS ||--o{ PROCESSES : contains
    SESSIONS ||--o{ EVENTS : tracks
    SESSIONS ||--o{ INSTALLATIONS : records
    SESSIONS ||--o{ ARTIFACTS : produces
    SESSIONS ||--o{ SOURCES : references
    SESSIONS ||--o{ EVIDENCE : preserves

    SOFTWARE ||--o{ INSTALLATIONS : has

    SESSIONS {
        string id PK
        string started_at
        string completed_at
        string status
        string scope
        int root_pid
        string initiating_command
        string classification
        string confidence
        string installer_identity
        string metadata_json
    }

    SOFTWARE {
        string id PK
        string name
        string display_name
        string primary_binary_path
        string scope
        string discovered_at
        string last_seen_at
        string metadata_json
    }

    EVENTS {
        string id PK
        string session_id FK
        string timestamp
        string event_type
        int pid
        int ppid
        int uid
        int euid
        string executable_path
        string arguments_json
        string working_directory
        string target_path
        string destination_path
        string backend
        string confidence
        string observation_kind
        string metadata_json
    }

    INSTALLATIONS {
        string id PK
        string software_id FK
        string session_id FK
        string installed_at
        string version
        string install_prefix
        string installer
        string classification
        string confidence
        string scope
    }

    ARTIFACTS {
        string id PK
        string installation_id FK
        string session_id FK
        string path
        string file_type
        string sha256
        int permissions
        int size_bytes
        string created_at
    }

    PROCESSES {
        string id PK
        string session_id FK
        int pid
        int ppid
        string executable_path
        string command_line
        string started_at
        string ended_at
        int exit_code
    }

    SOURCES {
        string id PK
        string session_id FK
        string source_type
        string uri
        string reference
    }

    EVIDENCE {
        string id PK
        string session_id FK
        string evidence_type
        string description
        string payload_json
        string captured_at
    }
```

---

## 2. Storage Locations & Paths

InstallMon dynamically resolves its storage paths based on execution scope:

| Scope | Default Database Path | Environment Variable Override | Fallback Path |
| :--- | :--- | :--- | :--- |
| **User Mode** | `~/.local/share/installmon/installmon.db` | `XDG_DATA_HOME/installmon/installmon.db` | `/tmp/installmon/installmon.db` |
| **System Mode** | `/var/lib/installmon/installmon.db` | Explicit `--database <PATH>` | `/tmp/installmon/installmon.db` |

---

## 3. High-Performance SQLite Pragmas

Upon opening every database connection, InstallMon applies the following pragmas:

```sql
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA foreign_keys = ON;
PRAGMA busy_timeout = 5000;
PRAGMA temp_store = MEMORY;
```

- **Write-Ahead Logging (WAL)**: Ensures read operations never block write operations and write operations never block reads.
- **Foreign Keys**: Enforces relational consistency across sessions, installations, artifacts, and evidence.
- **Busy Timeout**: Sets a 5000ms retry timeout to eliminate transient locking errors during heavy write activity.

---

## 4. Automated Schema Migrations

The database manages schema evolution automatically via the `schema_migrations` table:

```sql
CREATE TABLE schema_migrations (
    version INTEGER PRIMARY KEY,
    applied_at TEXT NOT NULL,
    description TEXT NOT NULL
);
```

- **Migration v1**: Initializes core tables: `monitoring_capabilities`, `sessions`, `processes`, `events`, `software`, `installations`, `artifacts`, `sources`, and `evidence`.
- **Indexes**: Includes performance indexes on `events(timestamp)`, `events(session_id)`, `events(event_type)`, `events(target_path)`, `processes(pid)`, `artifacts(path)`, `software(name)`, and `sessions(status)`.

---

## 5. Maintenance & Integrity Checks

- **Run Integrity Audit**:
  ```bash
  installmon audit
  ```
  Runs `PRAGMA integrity_check` and verifies tracked binary files against the filesystem.
- **Backup & SIEM Export**:
  ```bash
  # Export all records as pretty JSON
  installmon export --entity all --format json -o backup.json

  # Export event stream as JSON-Lines for external log ingestion
  installmon export --entity events --format jsonl
  ```

---

## Related Documentation & Next Steps

- 🏗️ [Architecture Overview](../architecture/overview.md) — System design and event persistence pipeline.
- 🔒 [Security & Sanitization](../security/sanitization.md) — How arguments are scrubbed before reaching SQLite.
- 💻 [CLI Command Reference](../cli/reference.md) — Documentation for `installmon audit` and `installmon export`.

---

[← Back to Documentation Hub](../README.md) | [← Back to Main README](../../README.md)
