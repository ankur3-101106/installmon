use anyhow::Result;
use installmon_platform::service::{
    disable_service, enable_service, get_service_status, install_and_start_service,
    restart_service, start_service, stop_service, uninstall_service,
};

pub fn execute_status(is_system: bool, json: bool) -> Result<()> {
    let status = get_service_status(is_system);

    if json {
        println!("{}", serde_json::to_string_pretty(&status)?);
    } else {
        println!("=== InstallMon Background Service ===");
        println!("State       : {:?}", status.state);
        println!("Type        : {}", status.service_type);
        println!(
            "PID         : {}",
            status
                .pid
                .map(|p| p.to_string())
                .unwrap_or_else(|| "None".to_string())
        );
        println!(
            "Uptime      : {}",
            status
                .uptime_seconds
                .map(|u| format!("{}s", u))
                .unwrap_or_else(|| "N/A".to_string())
        );
        println!("Installed   : {}", status.is_installed);
        println!("Auto-Start  : {}", status.is_enabled);
        println!("Unit File   : {}", status.unit_path.display());
        println!("Log File    : {}", status.daemon_log_path.display());
        println!("Details     : {}", status.details);

        if let Some(ref hb) = status.heartbeat {
            println!("--- Live Daemon Stats ---");
            println!("Events Recorded  : {}", hb.events_recorded);
            println!("Sessions Recorded: {}", hb.sessions_recorded);
            println!("Active Backends  : {}", hb.active_backends.join(", "));
            println!("Monitored Paths  : {}", hb.monitored_paths.join(", "));
            println!("Last Heartbeat   : {}", hb.last_heartbeat.to_rfc3339());
        }
    }

    Ok(())
}

pub fn execute_setup(is_system: bool) -> Result<()> {
    let msg = install_and_start_service(is_system)?;
    println!("{}", msg);
    Ok(())
}

pub fn execute_start(is_system: bool) -> Result<()> {
    let msg = start_service(is_system)?;
    println!("{}", msg);
    Ok(())
}

pub fn execute_stop(is_system: bool) -> Result<()> {
    let msg = stop_service(is_system)?;
    println!("{}", msg);
    Ok(())
}

pub fn execute_restart(is_system: bool) -> Result<()> {
    let msg = restart_service(is_system)?;
    println!("{}", msg);
    Ok(())
}

pub fn execute_enable(is_system: bool) -> Result<()> {
    let msg = enable_service(is_system)?;
    println!("{}", msg);
    Ok(())
}

pub fn execute_disable(is_system: bool) -> Result<()> {
    let msg = disable_service(is_system)?;
    println!("{}", msg);
    Ok(())
}

pub fn execute_uninstall(is_system: bool) -> Result<()> {
    let msg = uninstall_service(is_system)?;
    println!("{}", msg);
    Ok(())
}
