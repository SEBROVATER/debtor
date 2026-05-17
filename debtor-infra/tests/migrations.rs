//! Integration tests for database migrations.
//!
//! Verifies schema creation, table structure, constraints, default values, and reversibility.

use sqlx::Row;
use sqlx::migrate::Migrator;
use sqlx::sqlite::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions, SqliteRow};
use std::path::Path;
use std::str::FromStr;

const TABLES: &[&str] = &[
    "groups",
    "participants",
    "group_members",
    "spendings",
    "spending_payers",
    "spending_shares",
];

const INDEXES: &[&str] = &[
    "idx_group_members_group",
    "idx_group_members_participant",
    "idx_spendings_group",
    "idx_spendings_spent_date",
    "idx_spendings_type",
    "idx_spendings_group_date",
];

async fn pool_with_pragma() -> SqlitePool {
    let opts = SqliteConnectOptions::from_str("sqlite::memory:")
        .expect("valid connection string")
        .pragma("foreign_keys", "on");
    SqlitePoolOptions::new()
        .connect_with(opts)
        .await
        .expect("pool creation")
}

async fn all_tables_exist(pool: &SqlitePool) -> bool {
    let mut remaining = TABLES.to_vec();
    let rows: Vec<SqliteRow> = sqlx::query(
        "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' AND name NOT LIKE '_sqlx_%'",
    )
    .fetch_all(pool)
    .await
    .expect("query sqlite_master");
    for row in &rows {
        let name: &str = row.get("name");
        remaining.retain(|t| t != &name);
    }
    remaining.is_empty()
}

async fn all_indexes_exist(pool: &SqlitePool) -> bool {
    let mut remaining = INDEXES.to_vec();
    let rows: Vec<SqliteRow> = sqlx::query(
        "SELECT name FROM sqlite_master WHERE type='index' AND name NOT LIKE 'sqlite_%'",
    )
    .fetch_all(pool)
    .await
    .expect("query sqlite_master indexes");
    for row in &rows {
        let name: &str = row.get("name");
        remaining.retain(|i| i != &name);
    }
    remaining.is_empty()
}

#[sqlx::test(migrations = "../migrations")]
async fn all_migrations_apply_successfully(_pool: SqlitePool) {
    let pool = pool_with_pragma().await;
    let migrator = Migrator::new(Path::new("../migrations"))
        .await
        .expect("load migrator");
    migrator.run(&pool).await.expect("migrations apply");
}

#[sqlx::test(migrations = "../migrations")]
async fn all_expected_tables_exist(pool: SqlitePool) {
    let rows: Vec<SqliteRow> = sqlx::query(
        "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' AND name NOT LIKE '_sqlx_%'",
    )
    .fetch_all(&pool)
    .await
    .expect("query sqlite_master");

    let mut remaining = TABLES.to_vec();
    for row in &rows {
        let name: &str = row.get("name");
        remaining.retain(|t| t != &name);
    }
    assert!(remaining.is_empty(), "missing tables: {remaining:?}");
}

#[sqlx::test(migrations = "../migrations")]
async fn all_expected_indexes_exist(pool: SqlitePool) {
    let rows: Vec<SqliteRow> = sqlx::query(
        "SELECT name FROM sqlite_master WHERE type='index' AND name NOT LIKE 'sqlite_%'",
    )
    .fetch_all(&pool)
    .await
    .expect("query sqlite_master indexes");

    let mut remaining = INDEXES.to_vec();
    for row in &rows {
        let name: &str = row.get("name");
        remaining.retain(|i| i != &name);
    }
    assert!(remaining.is_empty(), "missing indexes: {remaining:?}");
}

#[sqlx::test(migrations = "../migrations")]
async fn groups_table_has_expected_columns(pool: SqlitePool) {
    let rows: Vec<SqliteRow> = sqlx::query("PRAGMA table_info(groups)")
        .fetch_all(&pool)
        .await
        .expect("PRAGMA table_info");

    let columns: Vec<(&str, &str, bool)> = rows
        .iter()
        .map(|r| {
            let name: &str = r.get("name");
            let _type: &str = r.get("type");
            let notnull: i32 = r.get("notnull");
            (name, _type, notnull == 1)
        })
        .collect();

    let expected = [
        ("id", "INTEGER", false),
        ("name", "TEXT", true),
        ("currency", "TEXT", true),
        ("created_at", "TEXT", true),
        ("updated_at", "TEXT", true),
    ];

    assert_eq!(columns.len(), expected.len(), "column count mismatch");
    for (actual, exp) in columns.iter().zip(expected.iter()) {
        assert_eq!(actual.0, exp.0, "column name mismatch");
        assert_eq!(actual.2, exp.2, "NOT NULL mismatch for column {}", exp.0);
    }
}

#[sqlx::test(migrations = "../migrations")]
async fn participants_table_has_expected_columns(pool: SqlitePool) {
    let rows: Vec<SqliteRow> = sqlx::query("PRAGMA table_info(participants)")
        .fetch_all(&pool)
        .await
        .expect("PRAGMA table_info");

    let columns: Vec<&str> = rows.iter().map(|r| r.get("name")).collect();

    let expected = ["id", "name", "color", "created_at", "updated_at"];
    assert_eq!(columns, expected);
}

#[sqlx::test(migrations = "../migrations")]
async fn group_members_has_composite_primary_key(pool: SqlitePool) {
    let columns: Vec<SqliteRow> = sqlx::query("PRAGMA table_info(group_members)")
        .fetch_all(&pool)
        .await
        .expect("PRAGMA table_info");

    let col_names: Vec<&str> = columns.iter().map(|r| r.get("name")).collect();
    assert_eq!(
        col_names,
        vec!["group_id", "participant_id", "is_active", "joined_at"]
    );

    let pk_name: String =
        sqlx::query_scalar("SELECT name FROM pragma_index_list('group_members') WHERE origin='pk'")
            .fetch_one(&pool)
            .await
            .expect("pk index name");

    let pk_columns: Vec<SqliteRow> = sqlx::query(&*format!("PRAGMA index_info({pk_name})"))
        .fetch_all(&pool)
        .await
        .expect("PRAGMA index_info");

    let pk_col_names: Vec<String> = pk_columns
        .iter()
        .map(|r| r.get::<String, _>("name"))
        .collect();
    assert_eq!(pk_col_names, vec!["group_id", "participant_id"]);
}

#[sqlx::test(migrations = "../migrations")]
async fn spendings_table_has_expected_columns(pool: SqlitePool) {
    let rows: Vec<SqliteRow> = sqlx::query("PRAGMA table_info(spendings)")
        .fetch_all(&pool)
        .await
        .expect("PRAGMA table_info");

    let col_names: Vec<&str> = rows.iter().map(|r| r.get("name")).collect();

    let expected = [
        "id",
        "group_id",
        "description",
        "total_amount",
        "currency",
        "spending_type",
        "spent_date",
        "created_at",
        "updated_at",
    ];
    assert_eq!(col_names, expected);
}

#[sqlx::test(migrations = "../migrations")]
async fn spending_payers_has_correct_structure(pool: SqlitePool) {
    let rows: Vec<SqliteRow> = sqlx::query("PRAGMA table_info(spending_payers)")
        .fetch_all(&pool)
        .await
        .expect("PRAGMA table_info");

    let col_names: Vec<&str> = rows.iter().map(|r| r.get("name")).collect();
    assert_eq!(
        col_names,
        vec!["spending_id", "participant_id", "paid_amount"]
    );

    let fk_list: Vec<SqliteRow> = sqlx::query("PRAGMA foreign_key_list(spending_payers)")
        .fetch_all(&pool)
        .await
        .expect("PRAGMA foreign_key_list");

    let tables: Vec<String> = fk_list.iter().map(|r| r.get("table")).collect();
    assert!(tables.contains(&"spendings".to_string()));
    assert!(tables.contains(&"participants".to_string()));
}

#[sqlx::test(migrations = "../migrations")]
async fn spending_shares_has_correct_structure(pool: SqlitePool) {
    let rows: Vec<SqliteRow> = sqlx::query("PRAGMA table_info(spending_shares)")
        .fetch_all(&pool)
        .await
        .expect("PRAGMA table_info");

    let col_names: Vec<&str> = rows.iter().map(|r| r.get("name")).collect();
    assert_eq!(
        col_names,
        vec!["spending_id", "participant_id", "share_amount"]
    );

    let fk_list: Vec<SqliteRow> = sqlx::query("PRAGMA foreign_key_list(spending_shares)")
        .fetch_all(&pool)
        .await
        .expect("PRAGMA foreign_key_list");

    let tables: Vec<String> = fk_list.iter().map(|r| r.get("table")).collect();
    assert!(tables.contains(&"spendings".to_string()));
    assert!(tables.contains(&"participants".to_string()));
}

#[sqlx::test(migrations = "../migrations")]
async fn foreign_keys_are_enforced(pool: SqlitePool) {
    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&pool)
        .await
        .expect("enable FK");

    let result = sqlx::query(
        "INSERT INTO spendings (group_id, description, total_amount, currency, spent_date) VALUES (9999, 'test', '10.00', 'USD', '2026-01-01')",
    )
    .execute(&pool)
    .await;

    assert!(
        result.is_err(),
        "expected FK violation for non-existent group_id"
    );
}

#[sqlx::test(migrations = "../migrations")]
async fn group_members_cascade_on_group_delete(pool: SqlitePool) {
    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&pool)
        .await
        .expect("enable FK");

    sqlx::query("INSERT INTO groups (name, currency) VALUES ('Test Group', 'USD')")
        .execute(&pool)
        .await
        .expect("insert group");

    sqlx::query("INSERT INTO participants (name, color) VALUES ('Alice', '#FF0000')")
        .execute(&pool)
        .await
        .expect("insert participant");

    sqlx::query("INSERT INTO group_members (group_id, participant_id) VALUES (1, 1)")
        .execute(&pool)
        .await
        .expect("insert group_member");

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM group_members")
        .fetch_one(&pool)
        .await
        .expect("count before delete");
    assert_eq!(count, 1);

    sqlx::query("DELETE FROM groups WHERE id = 1")
        .execute(&pool)
        .await
        .expect("delete group");

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM group_members")
        .fetch_one(&pool)
        .await
        .expect("count after delete");
    assert_eq!(count, 0);
}

#[sqlx::test(migrations = "../migrations")]
async fn spendings_cascade_on_group_delete(pool: SqlitePool) {
    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&pool)
        .await
        .expect("enable FK");

    sqlx::query("INSERT INTO groups (name, currency) VALUES ('G', 'USD')")
        .execute(&pool)
        .await
        .expect("insert group");

    sqlx::query("INSERT INTO spendings (group_id, description, total_amount, currency, spent_date) VALUES (1, 'Dinner', '50.00', 'USD', '2026-01-01')")
        .execute(&pool)
        .await
        .expect("insert spending");

    sqlx::query("DELETE FROM groups WHERE id = 1")
        .execute(&pool)
        .await
        .expect("delete group");

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM spendings")
        .fetch_one(&pool)
        .await
        .expect("count after delete");
    assert_eq!(count, 0);
}

#[sqlx::test(migrations = "../migrations")]
async fn spending_payers_cascade_on_spending_delete(pool: SqlitePool) {
    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&pool)
        .await
        .expect("enable FK");

    sqlx::query("INSERT INTO groups (name, currency) VALUES ('G', 'USD')")
        .execute(&pool)
        .await
        .expect("insert group");

    sqlx::query("INSERT INTO participants (name, color) VALUES ('Alice', '#FF0000')")
        .execute(&pool)
        .await
        .expect("insert participant");

    sqlx::query("INSERT INTO spendings (group_id, description, total_amount, currency, spent_date) VALUES (1, 'Dinner', '50.00', 'USD', '2026-01-01')")
        .execute(&pool)
        .await
        .expect("insert spending");

    sqlx::query("INSERT INTO spending_payers (spending_id, participant_id, paid_amount) VALUES (1, 1, '50.00')")
        .execute(&pool)
        .await
        .expect("insert payer");

    sqlx::query("DELETE FROM spendings WHERE id = 1")
        .execute(&pool)
        .await
        .expect("delete spending");

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM spending_payers")
        .fetch_one(&pool)
        .await
        .expect("count after delete");
    assert_eq!(count, 0);
}

#[sqlx::test(migrations = "../migrations")]
async fn spending_shares_cascade_on_spending_delete(pool: SqlitePool) {
    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&pool)
        .await
        .expect("enable FK");

    sqlx::query("INSERT INTO groups (name, currency) VALUES ('G', 'USD')")
        .execute(&pool)
        .await
        .expect("insert group");

    sqlx::query("INSERT INTO participants (name, color) VALUES ('Alice', '#FF0000')")
        .execute(&pool)
        .await
        .expect("insert participant");

    sqlx::query("INSERT INTO spendings (group_id, description, total_amount, currency, spent_date) VALUES (1, 'Dinner', '50.00', 'USD', '2026-01-01')")
        .execute(&pool)
        .await
        .expect("insert spending");

    sqlx::query("INSERT INTO spending_shares (spending_id, participant_id, share_amount) VALUES (1, 1, '50.00')")
        .execute(&pool)
        .await
        .expect("insert share");

    sqlx::query("DELETE FROM spendings WHERE id = 1")
        .execute(&pool)
        .await
        .expect("delete spending");

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM spending_shares")
        .fetch_one(&pool)
        .await
        .expect("count after delete");
    assert_eq!(count, 0);
}

#[sqlx::test(migrations = "../migrations")]
async fn group_members_prevents_duplicates(pool: SqlitePool) {
    sqlx::query("INSERT INTO groups (name, currency) VALUES ('G', 'USD')")
        .execute(&pool)
        .await
        .expect("insert group");

    sqlx::query("INSERT INTO participants (name, color) VALUES ('Alice', '#FF0000')")
        .execute(&pool)
        .await
        .expect("insert participant");

    sqlx::query("INSERT INTO group_members (group_id, participant_id) VALUES (1, 1)")
        .execute(&pool)
        .await
        .expect("first insert");

    let result = sqlx::query("INSERT INTO group_members (group_id, participant_id) VALUES (1, 1)")
        .execute(&pool)
        .await;

    assert!(result.is_err(), "expected duplicate composite PK violation");
}

#[sqlx::test(migrations = "../migrations")]
async fn groups_created_at_defaults_to_now(pool: SqlitePool) {
    sqlx::query("INSERT INTO groups (name, currency) VALUES ('Test', 'USD')")
        .execute(&pool)
        .await
        .expect("insert group");

    let row: SqliteRow = sqlx::query("SELECT created_at, updated_at FROM groups WHERE id = 1")
        .fetch_one(&pool)
        .await
        .expect("fetch group");

    let created_at: &str = row.get("created_at");
    let updated_at: &str = row.get("updated_at");

    assert!(!created_at.is_empty(), "created_at should not be empty");
    assert!(!updated_at.is_empty(), "updated_at should not be empty");

    chrono::NaiveDateTime::parse_from_str(created_at, "%Y-%m-%d %H:%M:%S")
        .expect("created_at should be valid ISO 8601 datetime");
    chrono::NaiveDateTime::parse_from_str(updated_at, "%Y-%m-%d %H:%M:%S")
        .expect("updated_at should be valid ISO 8601 datetime");
}

#[sqlx::test(migrations = "../migrations")]
async fn spendings_spending_type_defaults_to_other(pool: SqlitePool) {
    sqlx::query("INSERT INTO groups (name, currency) VALUES ('G', 'USD')")
        .execute(&pool)
        .await
        .expect("insert group");

    sqlx::query("INSERT INTO spendings (group_id, description, total_amount, currency, spent_date) VALUES (1, 'Test', '10.00', 'USD', '2026-01-01')")
        .execute(&pool)
        .await
        .expect("insert spending without spending_type");

    let spending_type: String =
        sqlx::query_scalar("SELECT spending_type FROM spendings WHERE id = 1")
            .fetch_one(&pool)
            .await
            .expect("fetch spending_type");

    assert_eq!(spending_type, "other");
}

#[sqlx::test(migrations = "../migrations")]
async fn group_members_is_active_defaults_to_true(pool: SqlitePool) {
    sqlx::query("INSERT INTO groups (name, currency) VALUES ('G', 'USD')")
        .execute(&pool)
        .await
        .expect("insert group");

    sqlx::query("INSERT INTO participants (name, color) VALUES ('Alice', '#FF0000')")
        .execute(&pool)
        .await
        .expect("insert participant");

    sqlx::query("INSERT INTO group_members (group_id, participant_id) VALUES (1, 1)")
        .execute(&pool)
        .await
        .expect("insert group_member without is_active");

    let is_active: i32 = sqlx::query_scalar(
        "SELECT is_active FROM group_members WHERE group_id = 1 AND participant_id = 1",
    )
    .fetch_one(&pool)
    .await
    .expect("fetch is_active");

    assert_eq!(is_active, 1);
}

#[tokio::test]
async fn migrations_can_be_reverted() {
    let pool = pool_with_pragma().await;
    let migrator = Migrator::new(Path::new("../migrations"))
        .await
        .expect("load migrator");

    migrator.run(&pool).await.expect("apply migrations");
    assert!(
        all_tables_exist(&pool).await,
        "tables should exist after up"
    );
    assert!(
        all_indexes_exist(&pool).await,
        "indexes should exist after up"
    );

    migrator
        .undo(&pool, 0)
        .await
        .expect("revert all migrations");

    assert!(
        !all_tables_exist(&pool).await,
        "tables should not exist after revert"
    );
}
