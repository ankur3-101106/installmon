use anyhow::{anyhow, Result};
use serde_json::json;

use installmon_database::Database;

pub fn execute(db: &Database, is_json: bool, software_query: &str) -> Result<()> {
    let all = db.get_all_software()?;
    let sw = all
        .iter()
        .find(|s| {
            s.name.eq_ignore_ascii_case(software_query)
                || s.id.to_string() == software_query
                || s.display_name
                    .as_deref()
                    .map(|d| d.eq_ignore_ascii_case(software_query))
                    .unwrap_or(false)
        })
        .ok_or_else(|| anyhow!("Software '{}' not found in database.", software_query))?;

    let installations = db.get_installations_for_software(sw.id)?;

    let mut session_details = Vec::new();
    for inst in &installations {
        let session = db.get_session(inst.session_id)?;
        let artifacts = db.get_artifacts_for_session(inst.session_id)?;
        let evidence = db.get_evidence_for_session(inst.session_id)?;

        session_details.push(json!({
            "installation": inst,
            "session": session,
            "artifacts_count": artifacts.len(),
            "artifacts": artifacts,
            "evidence": evidence
        }));
    }

    if is_json {
        let out = json!({
            "software": sw,
            "installations": session_details,
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        println!("=== Software Inspection: {} ===", sw.name);
        println!("ID               : {}", sw.id);
        if let Some(ref disp) = sw.display_name {
            println!("Display Name     : {}", disp);
        }
        println!("Scope            : {}", sw.scope);
        println!("Discovered At    : {}", sw.discovered_at);
        println!("Last Seen At     : {}", sw.last_seen_at);
        if let Some(ref bin) = sw.primary_binary_path {
            println!("Primary Binary   : {}", bin.display());
        }

        println!("\n=== Installation Records ({}) ===", installations.len());
        if installations.is_empty() {
            println!("No installation sessions linked directly to this software entry.");
        } else {
            for (idx, inst) in installations.iter().enumerate() {
                println!("[{}] Installed At : {}", idx + 1, inst.installed_at);
                println!(
                    "    Version      : {}",
                    inst.version.as_deref().unwrap_or("unknown")
                );
                println!(
                    "    Installer    : {}",
                    inst.installer.as_deref().unwrap_or("unknown")
                );
                println!(
                    "    Class        : {} (Confidence: {})",
                    inst.classification, inst.confidence
                );
                println!(
                    "    Prefix       : {}",
                    inst.install_prefix
                        .as_ref()
                        .map(|p| p.display().to_string())
                        .unwrap_or_else(|| "-".to_string())
                );
            }
        }
    }

    Ok(())
}
