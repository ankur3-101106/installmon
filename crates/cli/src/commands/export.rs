use anyhow::Result;
use serde_json::json;
use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;

use installmon_database::Database;

pub fn execute(db: &Database, format: &str, entity: &str, out_path: Option<PathBuf>) -> Result<()> {
    let mut writer: Box<dyn Write> = match out_path {
        Some(path) => Box::new(File::create(path)?),
        None => Box::new(io::stdout()),
    };

    match entity {
        "events" => {
            let events = db.get_events(10000, 0, None, None)?;
            if format == "jsonl" {
                for ev in &events {
                    writeln!(writer, "{}", serde_json::to_string(ev)?)?;
                }
            } else {
                writeln!(writer, "{}", serde_json::to_string_pretty(&events)?)?;
            }
        }
        "sessions" => {
            let sessions = db.get_sessions(1000)?;
            if format == "jsonl" {
                for s in &sessions {
                    writeln!(writer, "{}", serde_json::to_string(s)?)?;
                }
            } else {
                writeln!(writer, "{}", serde_json::to_string_pretty(&sessions)?)?;
            }
        }
        "software" => {
            let sw = db.get_all_software()?;
            if format == "jsonl" {
                for item in &sw {
                    writeln!(writer, "{}", serde_json::to_string(item)?)?;
                }
            } else {
                writeln!(writer, "{}", serde_json::to_string_pretty(&sw)?)?;
            }
        }
        _ => {
            let events = db.get_events(5000, 0, None, None)?;
            let sessions = db.get_sessions(500)?;
            let sw = db.get_all_software()?;

            let full = json!({
                "exported_at": chrono::Utc::now(),
                "software": sw,
                "sessions": sessions,
                "events": events
            });

            writeln!(writer, "{}", serde_json::to_string_pretty(&full)?)?;
        }
    }

    Ok(())
}
