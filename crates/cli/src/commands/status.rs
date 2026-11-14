use anyhow::Result;
use serde_json::json;
use std::path::PathBuf;

use installmon_database::Database;
use installmon_platform::detect_platform_capabilities;

pub fn execute(db: &Database, is_json: bool, db_path: &PathBuf) -> Result<()> {
    let caps = detect_platform_capabilities();
    let events_count = db.count_events().unwrap_or(0);
    let sessions_count = db.count_sessions().unwrap_or(0);
    let software_count = db.count_software().unwrap_or(0);
    let is_root = caps.is_root;

    if is_json {
        let out = json!({
            "status": "operational",
            "scope": if is_root { "system" } else { "user" },
            "is_root": is_root,
            "database_path": db_path,
            "counts": {
                "events": events_count,
                "sessions": sessions_count,
                "software": software_count
            },
            "platform": {
                "os": caps.distribution_name,
                "kernel": caps.kernel_release,
                "arch": caps.architecture,
                "init": caps.init_system.to_string(),
                "environment": caps.environment.to_string(),
                "active_process_backend": caps.active_process_backend,
                "active_filesystem_backend": caps.active_filesystem_backend
            }
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        println!("=== InstallMon System Status ===");
        println!(
            "Execution Scope  : {}",
            if is_root {
                "System (Privileged)"
            } else {
                "User (Unprivileged)"
            }
        );
        println!("Database Path    : {}", db_path.display());
        println!(
            "Host Distribution: {} ({})",
            caps.distribution_name, caps.distribution_id
        );
        println!(
            "Kernel Release   : {} [{}]",
            caps.kernel_release, caps.architecture
        );
        println!("Environment      : {}", caps.environment);
        println!("Init System      : {}", caps.init_system);
        println!(
            "Active Backends  : Process: [{}] | Filesystem: [{}]",
            caps.active_process_backend, caps.active_filesystem_backend
        );
        println!("--------------------------------");
        println!("Observed Events  : {}", events_count);
        println!("Tracked Sessions : {}", sessions_count);
        println!("Tracked Software : {}", software_count);
    }

    Ok(())
}
