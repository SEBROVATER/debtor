//! Database layer — `SQLx` connection pool and repository implementations.

pub mod repos;

use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

/// Opens a `SQLite` pool with foreign keys enabled on every connection.
///
/// # Errors
///
/// Returns a `SQLx` error when parsing or connecting the configured URL fails.
pub async fn connect(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    let options: SqliteConnectOptions = database_url.parse()?;
    SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options.foreign_keys(true))
        .await
}
