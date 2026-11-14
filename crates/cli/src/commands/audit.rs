use anyhow::Result;
use serde_json::json;

use installmon_database::Database;

pub fn execute(db: &Database, is_json: bool) -> Result<()> {
    let mut warnings = Vec::new();

    // 1. Database Integrity Verification
    let integrity_status = match db.verify_integrity() {
        Ok(_) => "Passed (PRAGMA integrity_check ok)".to_string(),
        Err(e) => {
            let err_msg = format!("Database corruption or error: {}", e);
            warnings.push(err_msg.clone());
            err_msg
        }
    };

    // 2. Audit Tracked Software Executables on Disk
    let software = db.get_all_software().unwrap_or_default();
    let mut missing_binaries = 0;
    for sw in &software {
        if let Some(ref path) = sw.primary_binary_path {
            if !path.exists() {
                missing_binaries += 1;
                warnings.push(format!(
                    "Binary missing on disk: {} ({})",
                    sw.name,
                    path.display()
                ));
            }
        }
    }

    // 3. Audit Sessions for Interrupted / Orphaned states
    let sessions = db.get_sessions(100).unwrap_or_default();
    let mut active_or_interrupted = 0;
    let mut unclassified_sessions = 0;
    for s in &sessions {
        if s.status == "active" || s.status == "interrupted" {
            active_or_interrupted += 1;
        }
        if s.classification.to_string() == "unknown_workflow" {
            unclassified_sessions += 1;
            warnings.push(format!(
                "Session {} has unknown workflow classification (manual review recommended)",
                s.id
            ));
        }
    }

    let is_clean = warnings.is_empty();

    if is_json {
        let out = json!({
            "audit_passed": is_clean,
            "database_integrity": integrity_status,
            "metrics": {
                "total_software_audited": software.len(),
                "missing_binaries": missing_binaries,
                "unclassified_sessions": unclassified_sessions,
                "active_or_interrupted_sessions": active_or_interrupted,
            },
            "warnings": warnings,
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        println!("=== InstallMon System & Software Audit Report ===");
        println!("Database Integrity  : {}", integrity_status);
        println!("Software Audited    : {}", software.len());
        println!("Missing Binaries    : {}", missing_binaries);
        println!("Unclassified Flows  : {}", unclassified_sessions);
        println!("Pending Sessions    : {}", active_or_interrupted);
        println!("------------------------------------------------");

        if is_clean {
            println!("Status: ALL AUDIT CHECKS PASSED. No anomalies detected.");
        } else {
            println!(
                "Status: AUDIT NOTICES / ANOMALIES FOUND ({}):",
                warnings.len()
            );
            for w in &warnings {
                println!("  [!] {}", w);
            }
        }
    }

    Ok(())
}
