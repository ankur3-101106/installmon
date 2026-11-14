use anyhow::Result;
use chrono::Utc;
use serde_json::json;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use installmon_core::provenance::InstallationScope;
use installmon_database::{Database, SoftwareRecord};

pub fn execute(db: &Database, is_json: bool, paths: Vec<PathBuf>) -> Result<()> {
    let scan_paths = if paths.is_empty() {
        default_discovery_paths()
    } else {
        paths
    };

    let mut discovered_count = 0;
    let mut scanned_directories = 0;

    for dir in &scan_paths {
        if !dir.exists() || !dir.is_dir() {
            continue;
        }

        scanned_directories += 1;
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if is_executable(&path) {
                    let file_name = path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();

                    if file_name.is_empty() {
                        continue;
                    }

                    // Check if already present in database
                    if db.get_software_by_name(&file_name)?.is_none() {
                        let is_system = path.starts_with("/usr") || path.starts_with("/opt");
                        let scope = if is_system {
                            InstallationScope::System
                        } else {
                            InstallationScope::User
                        };

                        let rec = SoftwareRecord {
                            id: Uuid::new_v4(),
                            name: file_name.clone(),
                            display_name: Some(file_name),
                            primary_binary_path: Some(path.clone()),
                            scope,
                            discovered_at: Utc::now(),
                            last_seen_at: Utc::now(),
                            metadata: json!({
                                "discovery_method": "path_scan",
                                "source_path": path.to_string_lossy()
                            }),
                        };

                        let _ = db.insert_software(&rec);
                        discovered_count += 1;
                    }
                }
            }
        }
    }

    if is_json {
        let out = json!({
            "scanned_directories": scanned_directories,
            "newly_discovered": discovered_count,
            "total_tracked": db.count_software().unwrap_or(0),
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        println!("=== Binary Discovery Scan Completed ===");
        println!("Scanned Directories : {}", scanned_directories);
        println!("Newly Discovered    : {}", discovered_count);
        println!("Total Tracked       : {}", db.count_software().unwrap_or(0));
    }

    Ok(())
}

fn default_discovery_paths() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Ok(path_var) = env::var("PATH") {
        for p in env::split_paths(&path_var) {
            // Prioritize local / user / custom prefixes first to keep scan light
            if p.starts_with("/usr/local")
                || p.to_string_lossy().contains(".local")
                || p.to_string_lossy().contains(".cargo")
            {
                dirs.push(p);
            }
        }
    }

    if dirs.is_empty() {
        dirs.push(PathBuf::from("/usr/local/bin"));
        if let Ok(home) = env::var("HOME") {
            let home_path = PathBuf::from(home);
            dirs.push(home_path.join(".local/bin"));
            dirs.push(home_path.join(".cargo/bin"));
        }
    }

    dirs
}

fn is_executable(path: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = fs::metadata(path) {
            return metadata.is_file() && (metadata.permissions().mode() & 0o111 != 0);
        }
    }
    false
}
