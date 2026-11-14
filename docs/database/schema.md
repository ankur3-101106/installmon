# Database Schema Specification

InstallMon uses SQLite with Write-Ahead Logging (`PRAGMA journal_mode = WAL`) and foreign key enforcement (`PRAGMA foreign_keys = ON`).

## Entities

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

    EVIDENCE {
        string id PK
        string session_id FK
        string timestamp
        string category
        string description
        string raw_data_json
        string confidence
    }
```

## Performance & Durability Indexes
- `idx_events_timestamp` on `events(timestamp)`
- `idx_events_session` on `events(session_id)`
- `idx_events_type` on `events(event_type)`
- `idx_events_target` on `events(target_path)`
- `idx_processes_pid` on `processes(pid)`
- `idx_processes_session` on `processes(session_id)`
- `idx_artifacts_path` on `artifacts(path)`
- `idx_software_name` on `software(name)`
- `idx_sessions_status` on `sessions(status)`
