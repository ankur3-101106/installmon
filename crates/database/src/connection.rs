use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use uuid::Uuid;

use installmon_core::capabilities::PlatformCapabilities;
use installmon_core::event::{
    CollectionConfidence, Event, EventType, MonitoringBackendType, ObservationKind,
};
use installmon_core::provenance::{InstallationScope, WorkflowClassification};
use installmon_platform::{database_path, ensure_parent_dir_exists};

use crate::error::DatabaseResult;
use crate::models::{
    ArtifactRecord, EvidenceRecord, InstallationRecord, SessionRecord, SoftwareRecord,
};
use crate::schema::{apply_pragmas, check_integrity, run_migrations};

/// Thread-safe SQLite repository instance.
pub struct Database {
    conn: Mutex<Connection>,
    path: Option<PathBuf>,
}

impl Database {
    /// Open a database file at the given path, creating it and applying migrations if needed.
    pub fn open(path: impl AsRef<Path>) -> DatabaseResult<Self> {
        let p = path.as_ref();
        ensure_parent_dir_exists(p)?;

        let mut conn = Connection::open(p)?;
        apply_pragmas(&conn)?;
        run_migrations(&mut conn)?;

        Ok(Self {
            conn: Mutex::new(conn),
            path: Some(p.to_path_buf()),
        })
    }

    /// Open the standard default database path (system-wide or user XDG directory).
    /// Gracefully falls back to `/tmp/installmon/installmon.db` if the standard path is on a read-only
    /// filesystem or restricted container environment.
    pub fn open_default(is_system: bool) -> DatabaseResult<Self> {
        let path = database_path(is_system);
        match Self::open(&path) {
            Ok(db) => Ok(db),
            Err(e) => {
                let fallback = PathBuf::from("/tmp/installmon/installmon.db");
                if path != fallback {
                    tracing::warn!(
                        "Failed to open default database at {}: {}. Falling back to {}",
                        path.display(),
                        e,
                        fallback.display()
                    );
                    Self::open(&fallback)
                } else {
                    Err(e)
                }
            }
        }
    }

    /// Create an isolated, in-memory SQLite database instance (primarily for tests).
    pub fn in_memory() -> DatabaseResult<Self> {
        let mut conn = Connection::open_in_memory()?;
        apply_pragmas(&conn)?;
        run_migrations(&mut conn)?;

        Ok(Self {
            conn: Mutex::new(conn),
            path: None,
        })
    }

    /// Returns the filesystem path to the database, if not in-memory.
    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    /// Perform a full SQLite integrity verification.
    pub fn verify_integrity(&self) -> DatabaseResult<()> {
        let conn = self.conn.lock().unwrap();
        check_integrity(&conn)
    }

    // ==========================================
    // Platform Capabilities Persistence
    // ==========================================

    pub fn insert_capabilities(&self, caps: &PlatformCapabilities) -> DatabaseResult<Uuid> {
        let conn = self.conn.lock().unwrap();
        let id = Uuid::new_v4();
        let raw_json = serde_json::to_string(caps)?;

        conn.execute(
            "INSERT INTO monitoring_capabilities (
                id, probed_at, kernel_release, architecture, distribution,
                init_system, environment, active_process_backend,
                active_filesystem_backend, raw_json
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                id.to_string(),
                caps.probed_at.to_rfc3339(),
                caps.kernel_release,
                caps.architecture,
                caps.distribution_name,
                caps.init_system.to_string(),
                caps.environment.to_string(),
                caps.active_process_backend,
                caps.active_filesystem_backend,
                raw_json,
            ],
        )?;

        Ok(id)
    }

    pub fn get_latest_capabilities(&self) -> DatabaseResult<Option<PlatformCapabilities>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT raw_json FROM monitoring_capabilities ORDER BY probed_at DESC LIMIT 1;",
        )?;

        let mut rows = stmt.query([])?;
        if let Some(row) = rows.next()? {
            let json_str: String = row.get(0)?;
            let caps: PlatformCapabilities = serde_json::from_str(&json_str)?;
            Ok(Some(caps))
        } else {
            Ok(None)
        }
    }

    // ==========================================
    // Events Persistence (Transactional & Batched)
    // ==========================================

    pub fn insert_event(&self, event: &Event) -> DatabaseResult<()> {
        self.insert_events_batch(std::slice::from_ref(event))
    }

    pub fn insert_events_batch(&self, events: &[Event]) -> DatabaseResult<()> {
        if events.is_empty() {
            return Ok(());
        }

        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;

        {
            let mut stmt = tx.prepare_cached(
                "INSERT INTO events (
                    id, session_id, timestamp, event_type, pid, ppid, uid, euid,
                    executable_path, arguments_json, working_directory,
                    target_path, destination_path, backend, confidence,
                    observation_kind, metadata_json
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
            )?;

            for ev in events {
                let args_json = serde_json::to_string(&ev.arguments)?;
                let meta_json = serde_json::to_string(&ev.metadata)?;

                stmt.execute(params![
                    ev.id.to_string(),
                    ev.session_id.map(|s| s.to_string()),
                    ev.timestamp.to_rfc3339(),
                    ev.event_type.to_string(),
                    ev.pid,
                    ev.ppid,
                    ev.uid,
                    ev.euid,
                    ev.executable_path.to_string_lossy().to_string(),
                    args_json,
                    ev.working_directory
                        .as_ref()
                        .map(|p| p.to_string_lossy().to_string()),
                    ev.target_path
                        .as_ref()
                        .map(|p| p.to_string_lossy().to_string()),
                    ev.destination_path
                        .as_ref()
                        .map(|p| p.to_string_lossy().to_string()),
                    ev.backend.to_string(),
                    ev.confidence.to_string(),
                    ev.observation_kind.to_string(),
                    meta_json,
                ])?;
            }
        }

        tx.commit()?;
        Ok(())
    }

    pub fn get_events(
        &self,
        limit: usize,
        offset: usize,
        event_type: Option<&str>,
        session_id: Option<Uuid>,
    ) -> DatabaseResult<Vec<Event>> {
        let conn = self.conn.lock().unwrap();
        let mut query = "SELECT id, session_id, timestamp, event_type, pid, ppid, uid, euid, \
                         executable_path, arguments_json, working_directory, target_path, \
                         destination_path, backend, confidence, observation_kind, metadata_json \
                         FROM events WHERE 1=1"
            .to_string();

        let mut params_vec: Vec<rusqlite::types::Value> = Vec::new();

        if let Some(et) = event_type {
            query.push_str(" AND event_type = ?");
            params_vec.push(et.to_string().into());
        }

        if let Some(sid) = session_id {
            query.push_str(" AND session_id = ?");
            params_vec.push(sid.to_string().into());
        }

        query.push_str(" ORDER BY timestamp DESC LIMIT ? OFFSET ?");
        params_vec.push((limit as i64).into());
        params_vec.push((offset as i64).into());

        let mut stmt = conn.prepare(&query)?;
        let rusqlite_params = rusqlite::params_from_iter(params_vec);

        let rows = stmt.query_map(rusqlite_params, |row| {
            let id_str: String = row.get(0)?;
            let session_id_str: Option<String> = row.get(1)?;
            let ts_str: String = row.get(2)?;
            let et_str: String = row.get(3)?;
            let pid: u32 = row.get(4)?;
            let ppid: Option<u32> = row.get(5)?;
            let uid: u32 = row.get(6)?;
            let euid: u32 = row.get(7)?;
            let exe_str: String = row.get(8)?;
            let args_json: String = row.get(9)?;
            let cwd_str: Option<String> = row.get(10)?;
            let target_str: Option<String> = row.get(11)?;
            let dest_str: Option<String> = row.get(12)?;
            let backend_str: String = row.get(13)?;
            let conf_str: String = row.get(14)?;
            let obs_str: String = row.get(15)?;
            let meta_json: String = row.get(16)?;

            let id = Uuid::parse_str(&id_str).unwrap_or_default();
            let session_id = session_id_str.and_then(|s| Uuid::parse_str(&s).ok());
            let timestamp = DateTime::parse_from_rfc3339(&ts_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            let event_type = match et_str.as_str() {
                "process_exec" => EventType::ProcessExec,
                "process_exit" => EventType::ProcessExit,
                "process_fork" => EventType::ProcessFork,
                "file_create" => EventType::FileCreate,
                "file_modify" => EventType::FileModify,
                "file_delete" => EventType::FileDelete,
                "file_rename" => EventType::FileRename,
                "symlink_create" => EventType::SymlinkCreate,
                "permission_change" => EventType::PermissionChange,
                "download_observed" => EventType::DownloadObserved,
                "privilege_change" => EventType::PrivilegeChange,
                "build_started" => EventType::BuildStarted,
                "build_completed" => EventType::BuildCompleted,
                "installation_started" => EventType::InstallationStarted,
                "installation_completed" => EventType::InstallationCompleted,
                "installation_failed" => EventType::InstallationFailed,
                _ => EventType::ProcessExec,
            };

            let backend = match backend_str.as_str() {
                "ebpf" => MonitoringBackendType::Ebpf,
                "audit" => MonitoringBackendType::Audit,
                "inotify" => MonitoringBackendType::Inotify,
                "fanotify" => MonitoringBackendType::Fanotify,
                "netlink" => MonitoringBackendType::Netlink,
                "manual" => MonitoringBackendType::Manual,
                "synthetic" => MonitoringBackendType::Synthetic,
                _ => MonitoringBackendType::Procfs,
            };

            let confidence = match conf_str.as_str() {
                "low" => CollectionConfidence::Low,
                "medium" => CollectionConfidence::Medium,
                _ => CollectionConfidence::High,
            };

            let observation_kind = if obs_str == "inferred" {
                ObservationKind::Inferred
            } else {
                ObservationKind::Direct
            };

            let arguments: Vec<String> = serde_json::from_str(&args_json).unwrap_or_default();
            let metadata: serde_json::Value = serde_json::from_str(&meta_json).unwrap_or_default();

            Ok(Event {
                id,
                timestamp,
                event_type,
                pid,
                ppid,
                uid,
                euid,
                executable_path: PathBuf::from(exe_str),
                arguments,
                working_directory: cwd_str.map(PathBuf::from),
                target_path: target_str.map(PathBuf::from),
                destination_path: dest_str.map(PathBuf::from),
                backend,
                confidence,
                observation_kind,
                session_id,
                metadata,
            })
        })?;

        let mut results = Vec::new();
        for r in rows {
            results.push(r?);
        }
        Ok(results)
    }

    pub fn count_events(&self) -> DatabaseResult<usize> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM events;", [], |r| r.get(0))?;
        Ok(count as usize)
    }

    pub fn prune_events_older_than(&self, timestamp: DateTime<Utc>) -> DatabaseResult<usize> {
        let conn = self.conn.lock().unwrap();
        let deleted = conn.execute(
            "DELETE FROM events WHERE timestamp < ?1;",
            params![timestamp.to_rfc3339()],
        )?;
        Ok(deleted)
    }

    // ==========================================
    // Sessions Persistence
    // ==========================================

    pub fn insert_session(&self, session: &SessionRecord) -> DatabaseResult<()> {
        let conn = self.conn.lock().unwrap();
        let meta_json = serde_json::to_string(&session.metadata)?;

        conn.execute(
            "INSERT INTO sessions (
                id, started_at, completed_at, status, scope, root_pid,
                initiating_command, classification, confidence,
                installer_identity, metadata_json
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                session.id.to_string(),
                session.started_at.to_rfc3339(),
                session.completed_at.map(|t| t.to_rfc3339()),
                session.status,
                session.scope.to_string(),
                session.root_pid,
                session.initiating_command,
                session.classification.to_string(),
                session.confidence.to_string(),
                session.installer_identity,
                meta_json,
            ],
        )?;

        Ok(())
    }

    pub fn update_session(&self, session: &SessionRecord) -> DatabaseResult<()> {
        let conn = self.conn.lock().unwrap();
        let meta_json = serde_json::to_string(&session.metadata)?;

        conn.execute(
            "UPDATE sessions SET
                completed_at = ?1,
                status = ?2,
                classification = ?3,
                confidence = ?4,
                installer_identity = ?5,
                metadata_json = ?6
            WHERE id = ?7",
            params![
                session.completed_at.map(|t| t.to_rfc3339()),
                session.status,
                session.classification.to_string(),
                session.confidence.to_string(),
                session.installer_identity,
                meta_json,
                session.id.to_string(),
            ],
        )?;

        Ok(())
    }

    pub fn get_session(&self, id: Uuid) -> DatabaseResult<Option<SessionRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, started_at, completed_at, status, scope, root_pid, \
                    initiating_command, classification, confidence, installer_identity, metadata_json \
             FROM sessions WHERE id = ?1",
        )?;

        let mut rows = stmt.query(params![id.to_string()])?;
        if let Some(row) = rows.next()? {
            let session = self.map_session_row(row)?;
            Ok(Some(session))
        } else {
            Ok(None)
        }
    }

    pub fn get_sessions(&self, limit: usize) -> DatabaseResult<Vec<SessionRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, started_at, completed_at, status, scope, root_pid, \
                    initiating_command, classification, confidence, installer_identity, metadata_json \
             FROM sessions ORDER BY started_at DESC LIMIT ?1",
        )?;

        let rows = stmt.query_map(params![limit as i64], |row| self.map_session_row(row))?;

        let mut results = Vec::new();
        for r in rows {
            results.push(r?);
        }
        Ok(results)
    }

    pub fn count_sessions(&self) -> DatabaseResult<usize> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM sessions;", [], |r| r.get(0))?;
        Ok(count as usize)
    }

    fn map_session_row(&self, row: &rusqlite::Row) -> rusqlite::Result<SessionRecord> {
        let id_str: String = row.get(0)?;
        let start_str: String = row.get(1)?;
        let end_str: Option<String> = row.get(2)?;
        let status: String = row.get(3)?;
        let scope_str: String = row.get(4)?;
        let root_pid: Option<u32> = row.get(5)?;
        let initiating_command: Option<String> = row.get(6)?;
        let class_str: String = row.get(7)?;
        let conf_str: String = row.get(8)?;
        let installer: Option<String> = row.get(9)?;
        let meta_json: String = row.get(10)?;

        let id = Uuid::parse_str(&id_str).unwrap_or_default();
        let started_at = DateTime::parse_from_rfc3339(&start_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());
        let completed_at = end_str.and_then(|s| {
            DateTime::parse_from_rfc3339(&s)
                .map(|dt| dt.with_timezone(&Utc))
                .ok()
        });

        let scope = match scope_str.as_str() {
            "system" => InstallationScope::System,
            "hybrid" => InstallationScope::Hybrid,
            _ => InstallationScope::User,
        };

        let classification = match class_str.as_str() {
            "known_workflow" => WorkflowClassification::KnownWorkflow,
            "inferred_workflow" => WorkflowClassification::InferredWorkflow,
            _ => WorkflowClassification::UnknownWorkflow,
        };

        let confidence = match conf_str.as_str() {
            "low" => CollectionConfidence::Low,
            "medium" => CollectionConfidence::Medium,
            _ => CollectionConfidence::High,
        };

        let metadata: serde_json::Value = serde_json::from_str(&meta_json).unwrap_or_default();

        Ok(SessionRecord {
            id,
            started_at,
            completed_at,
            status,
            scope,
            root_pid,
            initiating_command,
            classification,
            confidence,
            installer_identity: installer,
            metadata,
        })
    }

    // ==========================================
    // Software & Installations Persistence
    // ==========================================

    pub fn insert_software(&self, sw: &SoftwareRecord) -> DatabaseResult<()> {
        let conn = self.conn.lock().unwrap();
        let meta_json = serde_json::to_string(&sw.metadata)?;

        conn.execute(
            "INSERT INTO software (
                id, name, display_name, primary_binary_path, scope,
                discovered_at, last_seen_at, metadata_json
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                sw.id.to_string(),
                sw.name,
                sw.display_name,
                sw.primary_binary_path
                    .as_ref()
                    .map(|p| p.to_string_lossy().to_string()),
                sw.scope.to_string(),
                sw.discovered_at.to_rfc3339(),
                sw.last_seen_at.to_rfc3339(),
                meta_json,
            ],
        )?;

        Ok(())
    }

    pub fn get_all_software(&self) -> DatabaseResult<Vec<SoftwareRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, display_name, primary_binary_path, scope, \
                    discovered_at, last_seen_at, metadata_json \
             FROM software ORDER BY name ASC",
        )?;

        let rows = stmt.query_map([], |row| {
            let id_str: String = row.get(0)?;
            let name: String = row.get(1)?;
            let display_name: Option<String> = row.get(2)?;
            let path_str: Option<String> = row.get(3)?;
            let scope_str: String = row.get(4)?;
            let disc_str: String = row.get(5)?;
            let last_str: String = row.get(6)?;
            let meta_json: String = row.get(7)?;

            let id = Uuid::parse_str(&id_str).unwrap_or_default();
            let scope = match scope_str.as_str() {
                "system" => InstallationScope::System,
                "hybrid" => InstallationScope::Hybrid,
                _ => InstallationScope::User,
            };
            let discovered_at = DateTime::parse_from_rfc3339(&disc_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());
            let last_seen_at = DateTime::parse_from_rfc3339(&last_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());
            let metadata = serde_json::from_str(&meta_json).unwrap_or_default();

            Ok(SoftwareRecord {
                id,
                name,
                display_name,
                primary_binary_path: path_str.map(PathBuf::from),
                scope,
                discovered_at,
                last_seen_at,
                metadata,
            })
        })?;

        let mut results = Vec::new();
        for r in rows {
            results.push(r?);
        }
        Ok(results)
    }

    pub fn get_software_by_name(&self, name: &str) -> DatabaseResult<Option<SoftwareRecord>> {
        let list = self.get_all_software()?;
        Ok(list
            .into_iter()
            .find(|s| s.name == name || s.display_name.as_deref() == Some(name)))
    }

    pub fn count_software(&self) -> DatabaseResult<usize> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM software;", [], |r| r.get(0))?;
        Ok(count as usize)
    }

    pub fn insert_installation(&self, inst: &InstallationRecord) -> DatabaseResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO installations (
                id, software_id, session_id, installed_at, version,
                install_prefix, installer, classification, confidence, scope
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                inst.id.to_string(),
                inst.software_id.to_string(),
                inst.session_id.to_string(),
                inst.installed_at.to_rfc3339(),
                inst.version,
                inst.install_prefix
                    .as_ref()
                    .map(|p| p.to_string_lossy().to_string()),
                inst.installer,
                inst.classification.to_string(),
                inst.confidence.to_string(),
                inst.scope.to_string(),
            ],
        )?;
        Ok(())
    }

    pub fn get_installations_for_software(
        &self,
        software_id: Uuid,
    ) -> DatabaseResult<Vec<InstallationRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, software_id, session_id, installed_at, version, \
                    install_prefix, installer, classification, confidence, scope \
             FROM installations WHERE software_id = ?1 ORDER BY installed_at DESC",
        )?;

        let rows = stmt.query_map(params![software_id.to_string()], |row| {
            let id_str: String = row.get(0)?;
            let sw_id_str: String = row.get(1)?;
            let session_id_str: String = row.get(2)?;
            let inst_str: String = row.get(3)?;
            let version: Option<String> = row.get(4)?;
            let prefix_str: Option<String> = row.get(5)?;
            let installer: Option<String> = row.get(6)?;
            let class_str: String = row.get(7)?;
            let conf_str: String = row.get(8)?;
            let scope_str: String = row.get(9)?;

            let id = Uuid::parse_str(&id_str).unwrap_or_default();
            let software_id = Uuid::parse_str(&sw_id_str).unwrap_or_default();
            let session_id = Uuid::parse_str(&session_id_str).unwrap_or_default();
            let installed_at = DateTime::parse_from_rfc3339(&inst_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            let classification = match class_str.as_str() {
                "known_workflow" => WorkflowClassification::KnownWorkflow,
                "inferred_workflow" => WorkflowClassification::InferredWorkflow,
                _ => WorkflowClassification::UnknownWorkflow,
            };

            let confidence = match conf_str.as_str() {
                "low" => CollectionConfidence::Low,
                "medium" => CollectionConfidence::Medium,
                _ => CollectionConfidence::High,
            };

            let scope = match scope_str.as_str() {
                "system" => InstallationScope::System,
                "hybrid" => InstallationScope::Hybrid,
                _ => InstallationScope::User,
            };

            Ok(InstallationRecord {
                id,
                software_id,
                session_id,
                installed_at,
                version,
                install_prefix: prefix_str.map(PathBuf::from),
                installer,
                classification,
                confidence,
                scope,
            })
        })?;

        let mut results = Vec::new();
        for r in rows {
            results.push(r?);
        }
        Ok(results)
    }

    pub fn insert_artifact(&self, art: &ArtifactRecord) -> DatabaseResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO artifacts (
                id, installation_id, session_id, path, file_type,
                sha256, permissions, size_bytes, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                art.id.to_string(),
                art.installation_id.map(|i| i.to_string()),
                art.session_id.to_string(),
                art.path.to_string_lossy().to_string(),
                art.file_type,
                art.sha256,
                art.permissions,
                art.size_bytes.map(|s| s as i64),
                art.created_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn get_artifacts_for_session(
        &self,
        session_id: Uuid,
    ) -> DatabaseResult<Vec<ArtifactRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, installation_id, session_id, path, file_type, \
                    sha256, permissions, size_bytes, created_at \
             FROM artifacts WHERE session_id = ?1 ORDER BY path ASC",
        )?;

        let rows = stmt.query_map(params![session_id.to_string()], |row| {
            let id_str: String = row.get(0)?;
            let inst_id_str: Option<String> = row.get(1)?;
            let sess_id_str: String = row.get(2)?;
            let path_str: String = row.get(3)?;
            let file_type: String = row.get(4)?;
            let sha256: Option<String> = row.get(5)?;
            let permissions: Option<u32> = row.get(6)?;
            let size_bytes: Option<i64> = row.get(7)?;
            let created_str: String = row.get(8)?;

            let id = Uuid::parse_str(&id_str).unwrap_or_default();
            let installation_id = inst_id_str.and_then(|s| Uuid::parse_str(&s).ok());
            let session_id = Uuid::parse_str(&sess_id_str).unwrap_or_default();
            let created_at = DateTime::parse_from_rfc3339(&created_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            Ok(ArtifactRecord {
                id,
                installation_id,
                session_id,
                path: PathBuf::from(path_str),
                file_type,
                sha256,
                permissions,
                size_bytes: size_bytes.map(|s| s as u64),
                created_at,
            })
        })?;

        let mut results = Vec::new();
        for r in rows {
            results.push(r?);
        }
        Ok(results)
    }

    pub fn insert_evidence(&self, ev: &EvidenceRecord) -> DatabaseResult<()> {
        let conn = self.conn.lock().unwrap();
        let raw_json = serde_json::to_string(&ev.raw_data)?;
        conn.execute(
            "INSERT INTO evidence (
                id, session_id, timestamp, category, description,
                raw_data_json, confidence
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                ev.id.to_string(),
                ev.session_id.to_string(),
                ev.timestamp.to_rfc3339(),
                ev.category,
                ev.description,
                raw_json,
                ev.confidence.to_string(),
            ],
        )?;
        Ok(())
    }

    pub fn get_evidence_for_session(
        &self,
        session_id: Uuid,
    ) -> DatabaseResult<Vec<EvidenceRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, session_id, timestamp, category, description, raw_data_json, confidence \
             FROM evidence WHERE session_id = ?1 ORDER BY timestamp ASC",
        )?;

        let rows = stmt.query_map(params![session_id.to_string()], |row| {
            let id_str: String = row.get(0)?;
            let sess_str: String = row.get(1)?;
            let ts_str: String = row.get(2)?;
            let category: String = row.get(3)?;
            let description: String = row.get(4)?;
            let raw_json: String = row.get(5)?;
            let conf_str: String = row.get(6)?;

            let id = Uuid::parse_str(&id_str).unwrap_or_default();
            let session_id = Uuid::parse_str(&sess_str).unwrap_or_default();
            let timestamp = DateTime::parse_from_rfc3339(&ts_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());
            let confidence = match conf_str.as_str() {
                "low" => CollectionConfidence::Low,
                "medium" => CollectionConfidence::Medium,
                _ => CollectionConfidence::High,
            };
            let raw_data = serde_json::from_str(&raw_json).unwrap_or_default();

            Ok(EvidenceRecord {
                id,
                session_id,
                timestamp,
                category,
                description,
                raw_data,
                confidence,
            })
        })?;

        let mut results = Vec::new();
        for r in rows {
            results.push(r?);
        }
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_in_memory_database_lifecycle() {
        let db = Database::in_memory().expect("Failed to initialize in-memory DB");
        db.verify_integrity().expect("Integrity check failed");

        let event = Event::builder(EventType::ProcessExec, "/usr/bin/git")
            .pid(5000)
            .arguments(&[
                "git".into(),
                "clone".into(),
                "https://example.com/repo.git".into(),
            ])
            .build();

        db.insert_event(&event).expect("Failed to insert event");
        assert_eq!(db.count_events().unwrap(), 1);

        let retrieved = db.get_events(10, 0, None, None).unwrap();
        assert_eq!(retrieved.len(), 1);
        assert_eq!(retrieved[0].executable_path, PathBuf::from("/usr/bin/git"));
        assert_eq!(retrieved[0].pid, 5000);
    }

    #[test]
    fn test_session_and_evidence_persistence() {
        let db = Database::in_memory().unwrap();
        let session_id = Uuid::new_v4();

        let session = SessionRecord {
            id: session_id,
            started_at: Utc::now(),
            completed_at: None,
            status: "active".to_string(),
            scope: InstallationScope::User,
            root_pid: Some(1234),
            initiating_command: Some("cargo install ripgrep".to_string()),
            classification: WorkflowClassification::KnownWorkflow,
            confidence: CollectionConfidence::High,
            installer_identity: Some("cargo".to_string()),
            metadata: serde_json::json!({"crate": "ripgrep"}),
        };

        db.insert_session(&session).unwrap();
        assert_eq!(db.count_sessions().unwrap(), 1);

        let fetched = db
            .get_session(session_id)
            .unwrap()
            .expect("Session not found");
        assert_eq!(
            fetched.initiating_command.as_deref(),
            Some("cargo install ripgrep")
        );
        assert_eq!(fetched.scope, InstallationScope::User);

        let evidence = EvidenceRecord {
            id: Uuid::new_v4(),
            session_id,
            timestamp: Utc::now(),
            category: "command_execution".to_string(),
            description: "Cargo installation invoked".to_string(),
            raw_data: serde_json::json!({"args": ["install", "ripgrep"]}),
            confidence: CollectionConfidence::High,
        };

        db.insert_evidence(&evidence).unwrap();
        let ev_list = db.get_evidence_for_session(session_id).unwrap();
        assert_eq!(ev_list.len(), 1);
        assert_eq!(ev_list[0].category, "command_execution");
    }
}
