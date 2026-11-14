use chrono::Utc;
use std::path::PathBuf;
use uuid::Uuid;

use installmon_core::capabilities::PlatformCapabilities;
use installmon_core::event::{CollectionConfidence, Event, EventType, MonitoringBackendType};
use installmon_core::provenance::{InstallationScope, WorkflowClassification};
use installmon_core::sanitize::sanitize_arguments;
use installmon_database::{Database, SessionRecord, SoftwareRecord};
use installmon_platform::detect_platform_capabilities;

#[test]
fn test_sensitive_argument_sanitization() {
    let raw_args = vec![
        "curl".into(),
        "-H".into(),
        "Authorization: Bearer secret_jwt_token_here".into(),
        "-u".into(),
        "admin:p@ssw0rd123".into(),
        "--token=xyz987654321".into(),
        "--password".into(),
        "my_database_password".into(),
        "https://download.example.com/binary".into(),
    ];

    let sanitized = sanitize_arguments(&raw_args);

    assert_eq!(sanitized[0], "curl");
    assert_eq!(sanitized[1], "-H");
    assert_eq!(sanitized[2], "Authorization: Bearer [REDACTED]");
    assert_eq!(sanitized[3], "-u");
    assert_eq!(sanitized[4], "[REDACTED]");
    assert_eq!(sanitized[5], "--token=[REDACTED]");
    assert_eq!(sanitized[6], "--password");
    assert_eq!(sanitized[7], "[REDACTED]");
    assert_eq!(sanitized[8], "https://download.example.com/binary");
}

#[test]
fn test_database_batched_event_persistence_and_queries() {
    let db = Database::in_memory().expect("In-memory DB init failed");
    db.verify_integrity().expect("Integrity check failed");

    let session_id = Uuid::new_v4();

    let session = SessionRecord {
        id: session_id,
        started_at: Utc::now(),
        completed_at: None,
        status: "active".to_string(),
        scope: InstallationScope::User,
        root_pid: Some(4200),
        initiating_command: Some("curl -sSL https://get.example.com | sh".to_string()),
        classification: WorkflowClassification::InferredWorkflow,
        confidence: CollectionConfidence::Medium,
        installer_identity: Some("sh".to_string()),
        metadata: serde_json::json!({"test": true}),
    };
    db.insert_session(&session)
        .expect("Failed to insert session");

    let mut events = Vec::new();
    for i in 0..10 {
        let ev = Event::builder(EventType::ProcessExec, format!("/bin/step-{}", i))
            .pid(4200 + i)
            .ppid(Some(4200))
            .session_id(Some(session_id))
            .backend(MonitoringBackendType::Procfs)
            .arguments(&[
                format!("step-{}", i),
                "--token".to_string(),
                "secret".to_string(),
            ])
            .build();
        events.push(ev);
    }

    db.insert_events_batch(&events)
        .expect("Failed to batch insert events");

    assert_eq!(db.count_events().unwrap(), 10);

    let retrieved = db
        .get_events(5, 0, None, Some(session_id))
        .expect("Failed to query events");
    assert_eq!(retrieved.len(), 5);
    assert_eq!(retrieved[0].session_id, Some(session_id));
    assert_eq!(retrieved[0].arguments[2], "[REDACTED]");
}

#[test]
fn test_platform_capabilities_detection_and_persistence() {
    let db = Database::in_memory().expect("In-memory DB init failed");
    let caps: PlatformCapabilities = detect_platform_capabilities();

    assert!(!caps.kernel_release.is_empty());
    assert!(!caps.architecture.is_empty());
    assert!(!caps.distribution_name.is_empty());
    assert!(!caps.active_process_backend.is_empty());
    assert!(!caps.active_filesystem_backend.is_empty());

    let record_id = db
        .insert_capabilities(&caps)
        .expect("Failed to persist capabilities");
    assert!(!record_id.is_nil());

    let latest = db
        .get_latest_capabilities()
        .expect("Failed to get latest caps")
        .expect("Missing caps record");
    assert_eq!(latest.kernel_release, caps.kernel_release);
    assert_eq!(latest.architecture, caps.architecture);
}

#[test]
fn test_software_and_evidence_lifecycle() {
    let db = Database::in_memory().expect("In-memory DB init failed");

    let sw = SoftwareRecord {
        id: Uuid::new_v4(),
        name: "test-tool".to_string(),
        display_name: Some("Test Tool".to_string()),
        primary_binary_path: Some(PathBuf::from("/usr/local/bin/test-tool")),
        scope: InstallationScope::System,
        discovered_at: Utc::now(),
        last_seen_at: Utc::now(),
        metadata: serde_json::json!({"custom": "attr"}),
    };

    db.insert_software(&sw).expect("Insert software failed");
    assert_eq!(db.count_software().unwrap(), 1);

    let found = db
        .get_software_by_name("test-tool")
        .unwrap()
        .expect("Software not found");
    assert_eq!(found.id, sw.id);
    assert_eq!(found.scope, InstallationScope::System);
}
