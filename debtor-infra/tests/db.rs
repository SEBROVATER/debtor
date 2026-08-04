//! Integration tests for the supported `SQLite` connection and write contract.

#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use debtor_application::{
    ApplicationError, GroupRepository, ParticipantRepository, SpendingRepository, StorageReason,
};
use debtor_domain::currency::Currency;
use debtor_domain::model::{Allocation, Color, Description, Name, Spending, SpendingType};
use debtor_infra::db::connect;
use debtor_infra::db::repos::SqliteLedgerStore;
use sqlx::migrate::Migrator;
use sqlx::sqlite::SqlitePool;

static DATABASE_COUNTER: AtomicUsize = AtomicUsize::new(0);

async fn migrated_database() -> (PathBuf, SqlitePool) {
    let id = DATABASE_COUNTER.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!("debtor-slice7-{}-{id}.db", std::process::id()));
    let pool = connect(&database_url(&path))
        .await
        .expect("connect database");
    Migrator::new(std::path::Path::new("../migrations"))
        .await
        .expect("load migrations")
        .run(&pool)
        .await
        .expect("run migrations");
    (path, pool)
}

fn database_url(path: &Path) -> String {
    format!("sqlite://{}", path.display())
}

fn remove_database(path: &Path) {
    let _ = std::fs::remove_file(path);
    let _ = std::fs::remove_file(format!("{}-wal", path.display()));
    let _ = std::fs::remove_file(format!("{}-shm", path.display()));
}

#[tokio::test]
async fn file_database_uses_wal_full_and_persists_after_reopen() {
    let (path, pool) = migrated_database().await;
    let journal: String = sqlx::query_scalar("PRAGMA journal_mode")
        .fetch_one(&pool)
        .await
        .expect("journal mode");
    let synchronous: i64 = sqlx::query_scalar("PRAGMA synchronous")
        .fetch_one(&pool)
        .await
        .expect("synchronous mode");
    let busy_timeout: i64 = sqlx::query_scalar("PRAGMA busy_timeout")
        .fetch_one(&pool)
        .await
        .expect("busy timeout");
    assert_eq!(journal.to_ascii_lowercase(), "wal");
    assert_eq!(synchronous, 2, "SQLite FULL synchronous mode");
    assert_eq!(busy_timeout, 5_000);

    sqlx::query("INSERT INTO groups (name, currency) VALUES ('Persisted', 'USD')")
        .execute(&pool)
        .await
        .expect("insert persisted group");
    drop(pool);

    let reopened = connect(&database_url(&path))
        .await
        .expect("reopen database");
    let name: String = sqlx::query_scalar("SELECT name FROM groups WHERE id = 1")
        .fetch_one(&reopened)
        .await
        .expect("persisted group");
    assert_eq!(name, "Persisted");
    drop(reopened);
    remove_database(&path);
}

#[tokio::test]
async fn concurrent_mutations_through_one_store_are_serialized() {
    let (path, pool) = migrated_database().await;
    let store = Arc::new(SqliteLedgerStore::new(pool.clone()));
    let first = store.clone();
    let second = store.clone();
    let first_task = tokio::spawn(async move {
        first
            .create_group(Name::new("First").expect("name"), Currency::Usd)
            .await
    });
    let second_task = tokio::spawn(async move {
        second
            .create_group(Name::new("Second").expect("name"), Currency::Eur)
            .await
    });
    assert!(first_task.await.expect("first task").is_ok());
    assert!(second_task.await.expect("second task").is_ok());

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM groups")
        .fetch_one(&pool)
        .await
        .expect("group count");
    assert_eq!(count, 2);
    drop(store);
    drop(pool);
    remove_database(&path);
}

#[tokio::test]
async fn external_sqlite_lock_maps_to_contention_without_partial_write() {
    let (path, pool) = migrated_database().await;
    let store = SqliteLedgerStore::new(pool.clone());
    store
        .create_group(Name::new("Existing").expect("name"), Currency::Usd)
        .await
        .expect("existing group");

    let locker = connect(&database_url(&path)).await.expect("lock pool");
    let mut connection = locker.acquire().await.expect("lock connection");
    sqlx::query("BEGIN IMMEDIATE")
        .execute(&mut *connection)
        .await
        .expect("begin immediate lock");

    let result = store
        .create_group(Name::new("Blocked").expect("name"), Currency::Usd)
        .await;
    assert!(matches!(
        result,
        Err(ApplicationError::Storage(StorageReason::Contention))
    ));
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM groups")
        .fetch_one(&pool)
        .await
        .expect("group count");
    assert_eq!(count, 1);

    drop(connection);
    drop(locker);
    drop(pool);
    remove_database(&path);
}

#[tokio::test]
async fn spending_eligibility_failure_rolls_back_without_parent_or_allocations() {
    let (path, pool) = migrated_database().await;
    let store = SqliteLedgerStore::new(pool.clone());
    let group = store
        .create_group(Name::new("Trip").expect("name"), Currency::Usd)
        .await
        .expect("group");
    let participant = store
        .create_participant(
            Name::new("Ada").expect("name"),
            Color::new("#123456").expect("color"),
        )
        .await
        .expect("participant");

    let spending = Spending {
        id: 0,
        group_id: group.id,
        description: Description::new("Dinner").expect("description"),
        total: rust_decimal::Decimal::ONE,
        currency: Currency::Usd,
        spending_type: SpendingType::Food,
        spent_date: chrono::NaiveDate::from_ymd_opt(2026, 1, 1).expect("date"),
        payers: vec![Allocation {
            participant_id: participant.id,
            amount: rust_decimal::Decimal::ONE,
        }],
        shares: vec![Allocation {
            participant_id: participant.id,
            amount: rust_decimal::Decimal::ONE,
        }],
    };
    let result = store.create_spending(spending).await;
    assert!(matches!(result, Err(ApplicationError::Conflict)));
    for table in ["spendings", "spending_payers", "spending_shares"] {
        let count: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {table}"))
            .fetch_one(&pool)
            .await
            .expect("row count");
        assert_eq!(count, 0, "partial row in {table}");
    }

    drop(pool);
    remove_database(&path);
}
