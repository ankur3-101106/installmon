use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// Get the active data directory for storing databases and logs.
/// In system/root mode, this is `/var/lib/installmon`.
/// In user mode, this defaults to `$XDG_DATA_HOME/installmon` or `~/.local/share/installmon`.
pub fn data_dir(is_system: bool) -> PathBuf {
    if is_system {
        PathBuf::from("/var/lib/installmon")
    } else if let Ok(val) = env::var("XDG_DATA_HOME") {
        PathBuf::from(val).join("installmon")
    } else if let Ok(home) = env::var("HOME") {
        PathBuf::from(home)
            .join(".local")
            .join("share")
            .join("installmon")
    } else {
        PathBuf::from("/tmp/installmon")
    }
}

/// Get the default SQLite database path.
pub fn database_path(is_system: bool) -> PathBuf {
    data_dir(is_system).join("installmon.db")
}

/// Get the active runtime directory for IPC Unix domain sockets.
/// In system/root mode, this is `/run/installmon`.
/// In user mode, this defaults to `$XDG_RUNTIME_DIR/installmon` or `/tmp/installmon-$UID`.
pub fn runtime_dir(is_system: bool) -> PathBuf {
    if is_system {
        PathBuf::from("/run/installmon")
    } else if let Ok(val) = env::var("XDG_RUNTIME_DIR") {
        PathBuf::from(val).join("installmon")
    } else {
        let uid = unsafe { libc::getuid() };
        PathBuf::from(format!("/tmp/installmon-{}", uid))
    }
}

/// Get the default IPC socket path.
pub fn socket_path(is_system: bool) -> PathBuf {
    runtime_dir(is_system).join("installmon.sock")
}

/// Get the configuration directory.
/// In system/root mode: `/etc/installmon`.
/// In user mode: `$XDG_CONFIG_HOME/installmon` or `~/.config/installmon`.
pub fn config_dir(is_system: bool) -> PathBuf {
    if is_system {
        PathBuf::from("/etc/installmon")
    } else if let Ok(val) = env::var("XDG_CONFIG_HOME") {
        PathBuf::from(val).join("installmon")
    } else if let Ok(home) = env::var("HOME") {
        PathBuf::from(home).join(".config").join("installmon")
    } else {
        PathBuf::from("/etc/installmon")
    }
}

/// Ensures the directory for the specified file path exists, creating it if necessary.
pub fn ensure_parent_dir_exists(file_path: &Path) -> std::io::Result<()> {
    if let Some(parent) = file_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_paths() {
        assert_eq!(data_dir(true), PathBuf::from("/var/lib/installmon"));
        assert_eq!(
            database_path(true),
            PathBuf::from("/var/lib/installmon/installmon.db")
        );
        assert_eq!(
            socket_path(true),
            PathBuf::from("/run/installmon/installmon.sock")
        );
        assert_eq!(config_dir(true), PathBuf::from("/etc/installmon"));
    }

    #[test]
    fn test_user_paths() {
        let db = database_path(false);
        assert!(db.to_string_lossy().contains("installmon"));
    }
}
