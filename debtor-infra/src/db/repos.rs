//! `SQLite` implementations of application persistence ports.

#![allow(clippy::needless_pass_by_value)]

mod decoding;
mod groups;
mod participants;
mod snapshots;
mod spendings;

use std::sync::Arc;
use std::time::Duration;

use debtor_application::{ApplicationError, StorageReason};
use debtor_domain::model::EntityId;
use sqlx::SqlitePool;
use tokio::sync::Mutex;

const WRITE_GATE_TIMEOUT: Duration = Duration::from_secs(5);

/// SQLite-backed ledger persistence adapter.
pub struct SqliteLedgerStore {
    pub(super) pool: SqlitePool,
    write_gate: Arc<Mutex<()>>,
}

/// Root-owned `SQLite` resources shared by every persistence adapter handle.
pub struct SqliteLedgerRuntime {
    pool: SqlitePool,
    write_gate: Arc<Mutex<()>>,
}

impl SqliteLedgerRuntime {
    /// Creates one process-local runtime around a configured pool.
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            pool,
            write_gate: Arc::new(Mutex::new(())),
        }
    }

    /// Returns a cloneable adapter handle sharing the runtime write gate.
    pub fn store(&self) -> SqliteLedgerStore {
        SqliteLedgerStore {
            pool: self.pool.clone(),
            write_gate: self.write_gate.clone(),
        }
    }

    /// Returns a pool handle for readiness and bounded shutdown.
    pub fn pool(&self) -> SqlitePool {
        self.pool.clone()
    }
}

impl SqliteLedgerStore {
    async fn write_guard(&self) -> Result<tokio::sync::OwnedMutexGuard<()>, ApplicationError> {
        self.write_guard_with_timeout(WRITE_GATE_TIMEOUT).await
    }

    async fn write_guard_with_timeout(
        &self,
        timeout: Duration,
    ) -> Result<tokio::sync::OwnedMutexGuard<()>, ApplicationError> {
        tokio::time::timeout(timeout, self.write_gate.clone().lock_owned())
            .await
            .map_err(|_| ApplicationError::Storage(StorageReason::Contention))
    }
}

async fn group_mutable(pool: &SqlitePool, id: EntityId) -> Result<(), ApplicationError> {
    match sqlx::query_scalar!("SELECT is_archived FROM groups WHERE id = ?", id)
        .fetch_optional(pool)
        .await
        .map_err(storage)?
    {
        Some(0) => Ok(()),
        Some(_) => Err(ApplicationError::Conflict),
        None => Err(ApplicationError::NotFound),
    }
}

// This is only used after a conditional write was rejected, to retain the public error distinction.
async fn group_mutable_in_transaction(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    id: EntityId,
) -> Result<(), ApplicationError> {
    match sqlx::query_scalar!("SELECT is_archived FROM groups WHERE id = ?", id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(storage)?
    {
        Some(0) => Ok(()),
        Some(_) => Err(ApplicationError::Conflict),
        None => Err(ApplicationError::NotFound),
    }
}

fn storage(error: sqlx::Error) -> ApplicationError {
    if is_sqlite_contention(&error) {
        ApplicationError::Storage(StorageReason::Contention)
    } else {
        ApplicationError::Storage(StorageReason::Unexpected)
    }
}

fn is_sqlite_contention(error: &sqlx::Error) -> bool {
    let sqlx::Error::Database(database_error) = error else {
        return false;
    };
    matches!(database_error.code().as_deref(), Some("5" | "6"))
        || database_error.message().contains("database is locked")
}

fn changed(result: sqlx::sqlite::SqliteQueryResult) -> Result<(), ApplicationError> {
    if result.rows_affected() == 0 {
        Err(ApplicationError::NotFound)
    } else {
        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;
    use debtor_application::DatabaseReadiness;

    #[tokio::test]
    async fn write_gate_timeout_is_contention_before_database_work() {
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .expect("test pool");
        let store = SqliteLedgerRuntime::new(pool).store();
        let _held = store.write_gate.clone().lock_owned().await;
        assert!(matches!(
            store
                .write_guard_with_timeout(Duration::from_millis(10))
                .await,
            Err(ApplicationError::Storage(StorageReason::Contention))
        ));
    }

    #[tokio::test]
    async fn cloned_runtime_handles_block_on_the_shared_write_gate() {
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .expect("test pool");
        let runtime = SqliteLedgerRuntime::new(pool);
        let first = runtime.store();
        let second = runtime.store();
        let held = first.write_guard().await.expect("first gate acquisition");
        assert!(matches!(
            second
                .write_guard_with_timeout(Duration::from_millis(10))
                .await,
            Err(ApplicationError::Storage(StorageReason::Contention))
        ));
        drop(held);
        assert!(
            second
                .write_guard_with_timeout(Duration::from_millis(10))
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn readiness_accepts_a_healthy_pool() {
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .expect("test pool");
        assert!(SqliteLedgerRuntime::new(pool).store().check().await.is_ok());
    }

    #[tokio::test]
    async fn readiness_maps_a_closed_pool_to_storage_failure() {
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .expect("test pool");
        let store = SqliteLedgerRuntime::new(pool.clone()).store();
        pool.close().await;
        assert!(matches!(
            store.check().await,
            Err(ApplicationError::Storage(StorageReason::Unexpected))
        ));
    }

    #[tokio::test]
    async fn readiness_times_out_when_the_pool_cannot_acquire() {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .acquire_timeout(Duration::from_secs(30))
            .connect("sqlite::memory:")
            .await
            .expect("test pool");
        let _held = pool.acquire().await.expect("held connection");
        assert!(matches!(
            SqliteLedgerRuntime::new(pool).store().check().await,
            Err(ApplicationError::Storage(StorageReason::Contention))
        ));
    }
}
