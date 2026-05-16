use std::collections::HashSet;

use debtor::db::bootstrap::initialize_database;
use tempfile::tempdir;

#[tokio::test]
async fn bootstrap_runs_migrations_and_creates_tables() {
    let dir = tempdir().expect("tempdir");
    let db_path = dir.path().join("test.db");
    let db_path_str = db_path.to_string_lossy().replace("\\", "/");
    let database_url = format!("sqlite://{}?mode=rwc", db_path_str);

    let pool = initialize_database(&database_url)
        .await
        .expect("bootstrap should succeed");

    let rows = sqlx::query!("SELECT name FROM sqlite_master WHERE type='table'")
        .fetch_all(&pool)
        .await
        .expect("query sqlite_master");

    let names: HashSet<String> = rows.into_iter().map(|r| r.name.unwrap_or_default()).collect();

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
        names.contains("_sqlx_migrations"),
        "missing sqlx migration tracking table"
    );
}
