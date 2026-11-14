use crate::error::{DatabaseError, DatabaseResult};
use rusqlite::Connection;
use tracing::info;

pub const CURRENT_SCHEMA_VERSION: i32 = 1;

/// Apply recommended SQLite pragmas for high-throughput, crash-safe operation.
pub fn apply_pragmas(conn: &Connection) -> DatabaseResult<()> {
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;
         PRAGMA foreign_keys = ON;
         PRAGMA busy_timeout = 5000;
         PRAGMA temp_store = MEMORY;",
    )?;
    Ok(())
}

/// Execute database integrity check (PRAGMA integrity_check).
pub fn check_integrity(conn: &Connection) -> DatabaseResult<()> {
    let mut stmt = conn.prepare("PRAGMA integrity_check;")?;
    let mut rows = stmt.query([])?;

    if let Some(row) = rows.next()? {
        let status: String = row.get(0)?;
        if status != "ok" {
            return Err(DatabaseError::IntegrityCheckFailed(status));
        }
    }
    Ok(())
}

/// Run all necessary migrations to bring database up to `CURRENT_SCHEMA_VERSION`.
pub fn run_migrations(conn: &mut Connection) -> DatabaseResult<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL,
            description TEXT NOT NULL
        );",
        [],
    )?;

    let current_version: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_migrations;",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if current_version < 1 {
        info!("Applying database migration v1: Initializing core tables");
        let tx = conn.transaction()?;

        tx.execute_batch(
            "
            -- Platform Capabilities Snapshot
            CREATE TABLE IF NOT EXISTS monitoring_capabilities (
                id TEXT PRIMARY KEY,
                probed_at TEXT NOT NULL,
                kernel_release TEXT NOT NULL,
                architecture TEXT NOT NULL,
                distribution TEXT NOT NULL,
                init_system TEXT NOT NULL,
                environment TEXT NOT NULL,
                active_process_backend TEXT NOT NULL,
                active_filesystem_backend TEXT NOT NULL,
                raw_json TEXT NOT NULL
            );

            -- Installation Sessions
            CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                started_at TEXT NOT NULL,
                completed_at TEXT,
                status TEXT NOT NULL,
                scope TEXT NOT NULL,
                root_pid INTEGER,
                initiating_command TEXT,
                classification TEXT NOT NULL,
                confidence TEXT NOT NULL,
                installer_identity TEXT,
                metadata_json TEXT NOT NULL DEFAULT '{}'
            );

            -- Monitored Processes
            CREATE TABLE IF NOT EXISTS processes (
                id TEXT PRIMARY KEY,
                session_id TEXT REFERENCES sessions(id) ON DELETE SET NULL,
                pid INTEGER NOT NULL,
                ppid INTEGER,
                uid INTEGER NOT NULL,
                euid INTEGER NOT NULL,
                executable_path TEXT NOT NULL,
                arguments_json TEXT NOT NULL DEFAULT '[]',
                working_directory TEXT,
                started_at TEXT NOT NULL,
                exited_at TEXT,
                exit_code INTEGER
            );

            -- Normalized Lifecycle Events
            CREATE TABLE IF NOT EXISTS events (
                id TEXT PRIMARY KEY,
                session_id TEXT REFERENCES sessions(id) ON DELETE SET NULL,
                timestamp TEXT NOT NULL,
                event_type TEXT NOT NULL,
                pid INTEGER NOT NULL,
                ppid INTEGER,
                uid INTEGER NOT NULL,
                euid INTEGER NOT NULL,
                executable_path TEXT NOT NULL,
                arguments_json TEXT NOT NULL DEFAULT '[]',
                working_directory TEXT,
                target_path TEXT,
                destination_path TEXT,
                backend TEXT NOT NULL,
                confidence TEXT NOT NULL,
                observation_kind TEXT NOT NULL,
                metadata_json TEXT NOT NULL DEFAULT '{}'
            );

            -- Software Packages Tracked
            CREATE TABLE IF NOT EXISTS software (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                display_name TEXT,
                primary_binary_path TEXT,
                scope TEXT NOT NULL,
                discovered_at TEXT NOT NULL,
                last_seen_at TEXT NOT NULL,
                metadata_json TEXT NOT NULL DEFAULT '{}'
            );

            -- Specific Software Installations
            CREATE TABLE IF NOT EXISTS installations (
                id TEXT PRIMARY KEY,
                software_id TEXT NOT NULL REFERENCES software(id) ON DELETE CASCADE,
                session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
                installed_at TEXT NOT NULL,
                version TEXT,
                install_prefix TEXT,
                installer TEXT,
                classification TEXT NOT NULL,
                confidence TEXT NOT NULL,
                scope TEXT NOT NULL
            );

            -- Filesystem Artifacts created/modified by an installation
            CREATE TABLE IF NOT EXISTS artifacts (
                id TEXT PRIMARY KEY,
                installation_id TEXT REFERENCES installations(id) ON DELETE CASCADE,
                session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
                path TEXT NOT NULL,
                file_type TEXT NOT NULL,
                sha256 TEXT,
                permissions INTEGER,
                size_bytes INTEGER,
                created_at TEXT NOT NULL
            );

            -- Origin Sources (URLs, git repos, archives)
            CREATE TABLE IF NOT EXISTS sources (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
                source_type TEXT NOT NULL,
                url TEXT,
                git_commit TEXT,
                git_branch TEXT,
                checksum TEXT,
                recorded_at TEXT NOT NULL
            );

            -- Preserved Evidence Records
            CREATE TABLE IF NOT EXISTS evidence (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
                timestamp TEXT NOT NULL,
                category TEXT NOT NULL,
                description TEXT NOT NULL,
                raw_data_json TEXT NOT NULL,
                confidence TEXT NOT NULL
            );

            -- Performance Indexes
            CREATE INDEX IF NOT EXISTS idx_events_timestamp ON events(timestamp);
            CREATE INDEX IF NOT EXISTS idx_events_session ON events(session_id);
            CREATE INDEX IF NOT EXISTS idx_events_type ON events(event_type);
            CREATE INDEX IF NOT EXISTS idx_events_target ON events(target_path);
            CREATE INDEX IF NOT EXISTS idx_processes_pid ON processes(pid);
            CREATE INDEX IF NOT EXISTS idx_processes_session ON processes(session_id);
            CREATE INDEX IF NOT EXISTS idx_artifacts_path ON artifacts(path);
            CREATE INDEX IF NOT EXISTS idx_software_name ON software(name);
            CREATE INDEX IF NOT EXISTS idx_sessions_status ON sessions(status);
            ",
        )?;

        tx.execute(
            "INSERT INTO schema_migrations (version, applied_at, description)
             VALUES (?1, datetime('now'), ?2);",
            (1, "Initial v1 schema with core entities and indexes"),
        )?;

        tx.commit()?;
    }

    Ok(())
}
