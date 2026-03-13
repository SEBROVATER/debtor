use std::collections::HashSet;

use debtor::db::bootstrap::initialize_database;
use sea_orm::{ConnectionTrait, DbBackend, Statement};
use tempfile::tempdir;

#[tokio::test]
async fn bootstrap_runs_migrations_and_creates_tables() {
    let dir = tempdir().expect("tempdir");
    let db_path = dir.path().join("test.db");
    let db_path_str = db_path.to_string_lossy().replace("\\", "/");
    let database_url = format!("sqlite://{}?mode=rwc", db_path_str);

    let conn = initialize_database(&database_url)
        .await
        .expect("bootstrap should succeed");

    let rows = conn
        .query_all(Statement::from_string(
            DbBackend::Sqlite,
            "SELECT name FROM sqlite_master WHERE type='table'",
        ))
        .await
        .expect("query sqlite_master");

    let mut names = HashSet::new();
    for row in rows {
        let name: String = row.try_get::<String>("", "name").expect("table name");
        names.insert(name);
    }

    let required = [
        "admin_users",
        "auth_state",
        "sessions",
        "groups",
        "members",
        "expenses",
        "expense_shares",
        "exchange_rates",
    ];

    for table in required {
        assert!(names.contains(table), "missing table {table}");
    }

    assert!(
        names.contains("seaql_migrations"),
        "missing migration table"
    );
}
