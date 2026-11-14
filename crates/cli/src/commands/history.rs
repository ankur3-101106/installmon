use anyhow::Result;
use installmon_database::Database;
use serde_json::json;

pub fn execute(db: &Database, is_json: bool, limit: usize) -> Result<()> {
    let sessions = db.get_sessions(limit)?;

    if is_json {
        let history: Vec<_> = sessions
            .iter()
            .map(|s| {
                json!({
                    "session_id": s.id,
                    "timestamp": s.started_at,
                    "status": s.status,
                    "scope": s.scope.to_string(),
                    "classification": s.classification.to_string(),
                    "confidence": s.confidence.to_string(),
                    "command": s.initiating_command
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&history)?);
    } else {
        println!("=== Installation History Timeline (limit: {}) ===", limit);
        if sessions.is_empty() {
            println!("No installation history found.");
            return Ok(());
        }

        for (idx, s) in sessions.iter().enumerate() {
            let ts = s.started_at.format("%Y-%m-%d %H:%M:%S UTC");
            let cmd = s
                .initiating_command
                .as_deref()
                .unwrap_or("<unknown command>");
            let completed = s
                .completed_at
                .map(|c| format!(" (completed: {})", c.format("%H:%M:%S")))
                .unwrap_or_default();

            println!(
                "[{}] {} -> [{}] scope: {}{}",
                idx + 1,
                ts,
                s.status.to_uppercase(),
                s.scope,
                completed
            );
            println!("    Command       : {}", cmd);
            println!(
                "    Classification: {} [{}]",
                s.classification, s.confidence
            );
            println!("    Session ID    : {}", s.id);
            println!();
        }
    }

    Ok(())
}
