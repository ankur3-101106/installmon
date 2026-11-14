use thiserror::Error;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Migration error: {0}")]
    Migration(String),

    #[error("Database integrity check failed: {0}")]
    IntegrityCheckFailed(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Record not found: {0}")]
    NotFound(String),
}

pub type DatabaseResult<T> = Result<T, DatabaseError>;
