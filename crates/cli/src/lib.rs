use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use uuid::Uuid;

use installmon_database::Database;
use installmon_platform::database_path;

pub mod commands;
pub mod logging;

#[derive(Parser, Debug)]
#[command(
    name = "installmon",
    version = "0.1.0",
    about = "InstallMon - Linux Software Installation Monitoring & Auditing",
    long_about = "Lightweight, Linux-native software installation monitoring and auditing system.\n\
                  Tracks how software enters, gets installed, and changes within a Linux environment."
)]
pub struct Cli {
    /// Emit machine-readable JSON output
    #[arg(long, global = true)]
    pub json: bool,

    /// Explicit path to SQLite database file
    #[arg(short = 'd', long = "database", global = true)]
    pub database: Option<PathBuf>,

    /// Operate in system-wide mode (/var/lib/installmon) instead of user XDG mode
    #[arg(long, global = true)]
    pub system: bool,

    /// Increase logging verbosity (-v, -vv)
    #[arg(short = 'v', long = "verbose", action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,

    /// Silence log output
    #[arg(short = 'q', long = "quiet", global = true)]
    pub quiet: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Show service status, database health, and active monitoring scope
    Status,

    /// Inspect kernel, distribution, and monitoring capability details
    Capabilities,

    /// List tracked software installations
    List,

    /// Scan system PATH and directories to discover existing software binaries
    Scan {
        /// Optional explicit directories to scan instead of standard PATH
        #[arg(value_name = "DIRECTORIES")]
        directories: Vec<PathBuf>,
    },

    /// Inspect detailed provenance, evidence, and artifacts for a software package
    Inspect {
        /// Software name or UUID to inspect
        #[arg(value_name = "SOFTWARE")]
        software: String,
    },

    /// Show timeline of recorded installation sessions
    History {
        /// Maximum number of history entries to return
        #[arg(short, long, default_value_t = 20)]
        limit: usize,
    },

    /// List installation sessions
    Sessions {
        /// Maximum number of sessions to display
        #[arg(short, long, default_value_t = 20)]
        limit: usize,
    },

    /// Query raw observed lifecycle events
    Events {
        /// Maximum number of events to display
        #[arg(short, long, default_value_t = 50)]
        limit: usize,

        /// Offset index for pagination
        #[arg(long, default_value_t = 0)]
        offset: usize,

        /// Filter by specific event type (e.g. process_exec, file_create)
        #[arg(short = 't', long = "type")]
        event_type: Option<String>,

        /// Filter by installation session ID
        #[arg(short = 's', long = "session")]
        session: Option<Uuid>,
    },

    /// Perform database integrity check and audit tracked software for anomalies
    Audit,

    /// Export database records to JSON or JSON-Lines format
    Export {
        /// Export format: 'json' or 'jsonl'
        #[arg(short = 'f', long, default_value = "json")]
        format: String,

        /// Data entity to export: 'all', 'events', 'sessions', or 'software'
        #[arg(short = 'e', long, default_value = "all")]
        entity: String,

        /// Output file path (defaults to stdout)
        #[arg(short = 'o', long)]
        out: Option<PathBuf>,
    },
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();

    logging::init_logging(cli.verbose, cli.quiet, cli.json);

    let is_root = unsafe { libc::geteuid() == 0 };
    let is_system = cli.system || is_root;

    let db = match cli.database {
        Some(ref p) => Database::open(p)?,
        None => Database::open_default(is_system)?,
    };
    let db_path = db
        .path()
        .map(PathBuf::from)
        .unwrap_or_else(|| database_path(is_system));

    match cli.command {
        Commands::Status => {
            commands::status::execute(&db, cli.json, &db_path)?;
        }
        Commands::Capabilities => {
            commands::capabilities::execute(&db, cli.json)?;
        }
        Commands::List => {
            commands::software::execute(&db, cli.json)?;
        }
        Commands::Scan { directories } => {
            commands::scan::execute(&db, cli.json, directories)?;
        }
        Commands::Inspect { software } => {
            commands::inspect::execute(&db, cli.json, &software)?;
        }
        Commands::History { limit } => {
            commands::history::execute(&db, cli.json, limit)?;
        }
        Commands::Sessions { limit } => {
            commands::sessions::execute(&db, cli.json, limit)?;
        }
        Commands::Events {
            limit,
            offset,
            event_type,
            session,
        } => {
            commands::events::execute(&db, cli.json, limit, offset, event_type, session)?;
        }
        Commands::Audit => {
            commands::audit::execute(&db, cli.json)?;
        }
        Commands::Export {
            format,
            entity,
            out,
        } => {
            commands::export::execute(&db, &format, &entity, out)?;
        }
    }

    Ok(())
}
