use anyhow::Result;
use installmon_database::Database;
use installmon_platform::detect_platform_capabilities;

pub fn execute(db: &Database, is_json: bool) -> Result<()> {
    let caps = detect_platform_capabilities();

    // Persist latest probed capabilities to database for auditing
    let _ = db.insert_capabilities(&caps);

    if is_json {
        println!("{}", serde_json::to_string_pretty(&caps)?);
    } else {
        println!("=== Host & Kernel Monitoring Capabilities ===");
        println!(
            "Distribution       : {} ({})",
            caps.distribution_name, caps.distribution_id
        );
        if let Some(ref ver) = caps.distribution_version {
            println!("Distro Version     : {}", ver);
        }
        println!("Architecture       : {}", caps.architecture);
        println!("Kernel Release     : {}", caps.kernel_release);
        println!("Virtualization/Env : {}", caps.environment);
        println!("Init System        : {}", caps.init_system);
        println!("C Library (libc)   : {}", caps.libc);
        println!(
            "Privileged (Root)  : {}",
            if caps.is_root { "Yes" } else { "No" }
        );
        println!(
            "BTF Metadata       : {}",
            if caps.btf_supported {
                "Available (/sys/kernel/btf/vmlinux)"
            } else {
                "Not found"
            }
        );
        println!(
            "Inotify Support    : {}",
            if caps.inotify_supported {
                "Available"
            } else {
                "Not available"
            }
        );
        println!(
            "Fanotify Support   : {}",
            if caps.fanotify_supported {
                "Available"
            } else {
                "Unavailable / requires root"
            }
        );
        println!(
            "Procfs Available   : {}",
            if caps.procfs_supported {
                "Available (/proc)"
            } else {
                "Not available"
            }
        );
        println!(
            "Audit Available    : {}",
            if caps.audit_supported {
                "Available"
            } else {
                "Not available"
            }
        );

        println!("\n=== Negotiated Monitoring Backends ===");
        println!(
            "Active Process Monitoring    : [{}]",
            caps.active_process_backend
        );
        println!(
            "Active Filesystem Monitoring : [{}]",
            caps.active_filesystem_backend
        );

        println!("\n=== Backend Subsystem Details ===");
        for backend in &caps.backends {
            println!(
                "  * {:<10} : [{:<18}] - {}",
                backend.name, backend.status, backend.description
            );
            if let Some(ref details) = backend.details {
                println!("    Details: {}", details);
            }
        }
    }

    Ok(())
}
