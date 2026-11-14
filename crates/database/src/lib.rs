//! InstallMon Database - SQLite persistence layer for installation events, sessions,
//! artifacts, capabilities, and provenance evidence.

pub mod connection;
pub mod error;
pub mod models;
pub mod schema;

pub use connection::Database;
pub use error::{DatabaseError, DatabaseResult};
pub use models::*;
pub use schema::CURRENT_SCHEMA_VERSION;
