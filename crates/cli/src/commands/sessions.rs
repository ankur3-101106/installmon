use anyhow::Result;
use installmon_database::Database;

pub fn execute(db: &Database, is_json: bool, limit: usize) -> Result<()> {
    let sessions = db.get_sessions(limit)?;

    if is_json {
        println!("{}", serde_json::to_string_pretty(&sessions)?);
    } else {
        println!("=== Tracked Installation Sessions (limit: {}) ===", limit);
        if sessions.is_empty() {
            println!("No installation sessions recorded yet.");
            return Ok(());
        }

        println!(
            "{:<36} {:<20} {:<10} {:<8} {:<18} {:<8} INITIATING COMMAND",
            "SESSION ID", "STARTED AT", "STATUS", "SCOPE", "CLASSIFICATION", "CONF"
        );
        println!("{}", "-".repeat(120));

        for s in &sessions {
            let ts = s.started_at.format("%Y-%m-%d %H:%M:%S").to_string();
            let cmd = s.initiating_command.as_deref().unwrap_or("-");

            println!(
                "{:<36} {:<20} {:<10} {:<8} {:<18} {:<8} {}",
                s.id, ts, s.status, s.scope, s.classification, s.confidence, cmd
            );
        }
    }

    Ok(())
}
