use anyhow::Result;
use uuid::Uuid;

use installmon_database::Database;

pub fn execute(
    db: &Database,
    is_json: bool,
    limit: usize,
    offset: usize,
    event_type: Option<String>,
    session_id: Option<Uuid>,
) -> Result<()> {
    let events = db.get_events(limit, offset, event_type.as_deref(), session_id)?;

    if is_json {
        println!("{}", serde_json::to_string_pretty(&events)?);
    } else {
        println!(
            "=== Recorded Installation Lifecycle Events (limit: {}, offset: {}) ===",
            limit, offset
        );
        if events.is_empty() {
            println!("No matching events recorded.");
            return Ok(());
        }

        println!(
            "{:<24} {:<18} {:<7} {:<6} {:<10} {:<32} TARGET PATH",
            "TIMESTAMP", "EVENT TYPE", "PID", "EUID", "BACKEND", "EXECUTABLE"
        );
        println!("{}", "-".repeat(110));

        for ev in &events {
            let ts = ev.timestamp.format("%Y-%m-%d %H:%M:%S").to_string();
            let exe = ev
                .executable_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| ev.executable_path.to_string_lossy().to_string());
            let target = ev
                .target_path
                .as_ref()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| "-".to_string());

            println!(
                "{:<24} {:<18} {:<7} {:<6} {:<10} {:<32} {}",
                ts, ev.event_type, ev.pid, ev.euid, ev.backend, exe, target
            );
        }
    }

    Ok(())
}
