use anyhow::Result;
use installmon_database::Database;

pub fn execute(db: &Database, is_json: bool) -> Result<()> {
    let software = db.get_all_software()?;

    if is_json {
        println!("{}", serde_json::to_string_pretty(&software)?);
    } else {
        println!("=== Tracked Software Installations ===");
        if software.is_empty() {
            println!("No software tracked in database. Run 'installmon scan' to discover existing installations.");
            return Ok(());
        }

        println!(
            "{:<24} {:<8} {:<20} {:<36} PRIMARY BINARY",
            "NAME", "SCOPE", "DISCOVERED AT", "SOFTWARE ID"
        );
        println!("{}", "-".repeat(110));

        for sw in &software {
            let disc = sw.discovered_at.format("%Y-%m-%d %H:%M:%S").to_string();
            let bin = sw
                .primary_binary_path
                .as_ref()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| "-".to_string());

            println!(
                "{:<24} {:<8} {:<20} {:<36} {}",
                sw.name, sw.scope, disc, sw.id, bin
            );
        }
    }

    Ok(())
}
