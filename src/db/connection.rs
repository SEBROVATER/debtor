use sea_orm::{
    ConnectOptions, ConnectionTrait, Database, DatabaseConnection, DbBackend, DbErr, Statement,
};

pub async fn connect_sqlite(database_url: &str) -> Result<DatabaseConnection, DbErr> {
    let mut options = ConnectOptions::new(database_url.to_string());
    options.sqlx_logging(false);

    let conn = Database::connect(options).await?;
    conn.execute(Statement::from_string(
        DbBackend::Sqlite,
        "PRAGMA journal_mode = WAL;",
    ))
    .await?;
    conn.execute(Statement::from_string(
        DbBackend::Sqlite,
        "PRAGMA foreign_keys = ON;",
    ))
    .await?;

    Ok(conn)
}
