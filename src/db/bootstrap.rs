use chrono::Utc;
use sqlx::SqlitePool;

use crate::app::config::AppConfig;
use crate::db::connection::connect_sqlite;

pub async fn initialize_database(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    let pool = connect_sqlite(database_url).await?;
    sqlx::migrate!().run(&pool).await?;
    Ok(pool)
}

pub async fn bootstrap_admin_user(
    pool: &SqlitePool,
    config: &AppConfig,
) -> Result<bool, sqlx::Error> {
    let existing: Option<(i64,)> =
        sqlx::query_as("SELECT id FROM admin_users WHERE username = ?")
            .bind(&config.admin_username)
            .fetch_optional(pool)
            .await?;

    if existing.is_some() {
        return Ok(false);
    }

    let Some(ref password_hash) = config.admin_password_hash else {
        return Ok(false);
    };

    let now = Utc::now().naive_utc();
    sqlx::query(
        "INSERT INTO admin_users (id, username, password_hash, created_at, updated_at)
         VALUES (1, ?, ?, ?, ?)",
    )
    .bind(&config.admin_username)
    .bind(password_hash)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;

    Ok(true)
}
