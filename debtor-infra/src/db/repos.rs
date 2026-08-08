//! `SQLite` implementations of application persistence ports.

#![allow(clippy::needless_pass_by_value)]

use std::collections::{BTreeMap, BTreeSet};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use debtor_application::{
    ApplicationError, DatabaseReadiness, GroupReader, GroupRepository, LedgerSnapshot,
    LedgerSnapshotReader, ParticipantRepository, SpendingCursor, SpendingEligibilityReader,
    SpendingPage, SpendingPageDirection, SpendingReader, SpendingRepository, SpendingSummary,
    StorageReason,
};
use debtor_domain::currency::Currency;
use debtor_domain::model::{
    Allocation, Color, Description, EntityId, Group, GroupMember, Name, Participant, Spending,
    SpendingType, validate_amount,
};
use debtor_domain::money::{format_decimal, parse_decimal};
use sqlx::SqlitePool;
use tokio::sync::Mutex;

const WRITE_GATE_TIMEOUT: Duration = Duration::from_secs(5);
const READINESS_TIMEOUT: Duration = Duration::from_secs(1);

/// SQLite-backed ledger persistence adapter.
pub struct SqliteLedgerStore {
    pool: SqlitePool,
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

struct DbGroup {
    id: i64,
    name: String,
    currency: String,
    is_archived: i64,
}

struct DbParticipant {
    id: i64,
    name: String,
    color: String,
    is_archived: i64,
}

struct DbGroupMember {
    id: i64,
    name: String,
    color: String,
    is_archived: i64,
    is_active: i64,
}

struct DbSpending {
    description: String,
    total_amount: String,
    currency: String,
    spending_type: String,
    spent_date: String,
}

struct DbSpendingSummary {
    id: i64,
    description: String,
    total_amount: String,
    currency: String,
    spending_type: String,
    spent_date: String,
}

struct DbSnapshotSpending {
    id: i64,
    description: String,
    total_amount: String,
    currency: String,
    spending_type: String,
    spent_date: String,
}

struct DbAllocation {
    participant_id: i64,
    amount: String,
}

struct DbSpendingAllocation {
    spending_id: i64,
    participant_id: i64,
    amount: String,
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

    async fn group_mutable(&self, id: EntityId) -> Result<(), ApplicationError> {
        match sqlx::query_scalar!("SELECT is_archived FROM groups WHERE id = ?", id)
            .fetch_optional(&self.pool)
            .await
            .map_err(storage)?
        {
            Some(0) => Ok(()),
            Some(_) => Err(ApplicationError::Conflict),
            None => Err(ApplicationError::NotFound),
        }
    }

    async fn participant_mutable(&self, id: EntityId) -> Result<(), ApplicationError> {
        match sqlx::query_scalar!("SELECT is_archived FROM participants WHERE id = ?", id)
            .fetch_optional(&self.pool)
            .await
            .map_err(storage)?
        {
            Some(0) => Ok(()),
            Some(_) => Err(ApplicationError::Conflict),
            None => Err(ApplicationError::NotFound),
        }
    }

    async fn ledger_snapshot_impl(
        &self,
        group_id: EntityId,
    ) -> Result<LedgerSnapshot, ApplicationError> {
        let mut tx = self.pool.begin().await.map_err(storage)?;
        let group = sqlx::query_as!(
            DbGroup,
            "SELECT id, name, currency, is_archived FROM groups WHERE id = ?",
            group_id
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(storage)?
        .ok_or(ApplicationError::NotFound)
        .and_then(|row| {
            group(row).map_err(|_| ApplicationError::Storage(StorageReason::InvalidData))
        })?;

        let parents = sqlx::query_as!(
            DbSnapshotSpending,
            "SELECT id AS \"id!: i64\", description, total_amount, currency, spending_type, spent_date FROM spendings WHERE group_id = ? ORDER BY spent_date DESC, id DESC",
            group_id
        )
        .fetch_all(&mut *tx)
        .await
        .map_err(storage)?;
        let payer_rows = sqlx::query_as!(
            DbSpendingAllocation,
            "SELECT sp.id AS \"spending_id!: i64\", p.participant_id, p.paid_amount AS amount FROM spending_payers p JOIN spendings sp ON sp.id = p.spending_id WHERE sp.group_id = ? ORDER BY sp.id, p.participant_id",
            group_id
        )
        .fetch_all(&mut *tx)
        .await
        .map_err(storage)?;
        let share_rows = sqlx::query_as!(
            DbSpendingAllocation,
            "SELECT sp.id AS \"spending_id!: i64\", s.participant_id, s.share_amount AS amount FROM spending_shares s JOIN spendings sp ON sp.id = s.spending_id WHERE sp.group_id = ? ORDER BY sp.id, s.participant_id",
            group_id
        )
        .fetch_all(&mut *tx)
        .await
        .map_err(storage)?;

        let mut payers = BTreeMap::<EntityId, Vec<Allocation>>::new();
        for row in payer_rows {
            payers.entry(row.spending_id).or_default().push(Allocation {
                participant_id: row.participant_id,
                amount: canonical_decimal(&row.amount)?,
            });
        }
        let mut shares = BTreeMap::<EntityId, Vec<Allocation>>::new();
        for row in share_rows {
            shares.entry(row.spending_id).or_default().push(Allocation {
                participant_id: row.participant_id,
                amount: canonical_decimal(&row.amount)?,
            });
        }

        let mut spendings = Vec::with_capacity(parents.len());
        for row in parents {
            let spending = Spending {
                id: row.id,
                group_id,
                description: Description::new(row.description)
                    .map_err(|_| ApplicationError::Storage(StorageReason::InvalidData))?,
                total: canonical_decimal(&row.total_amount)?,
                currency: Currency::from_str(&row.currency)
                    .map_err(|_| ApplicationError::Storage(StorageReason::InvalidData))?,
                spending_type: SpendingType::from_str(&row.spending_type)
                    .map_err(|_| ApplicationError::Storage(StorageReason::InvalidData))?,
                spent_date: chrono::NaiveDate::parse_from_str(&row.spent_date, "%Y-%m-%d")
                    .map_err(|_| ApplicationError::Storage(StorageReason::InvalidData))?,
                payers: payers.remove(&row.id).unwrap_or_default(),
                shares: shares.remove(&row.id).unwrap_or_default(),
            };
            spending
                .validate()
                .map_err(|_| ApplicationError::Storage(StorageReason::InvalidData))?;
            spendings.push(spending);
        }
        tx.commit().await.map_err(storage)?;
        Ok(LedgerSnapshot { group, spendings })
    }
}

#[async_trait]
impl DatabaseReadiness for SqliteLedgerStore {
    async fn check(&self) -> Result<(), ApplicationError> {
        match tokio::time::timeout(READINESS_TIMEOUT, async {
            let mut connection = self.pool.acquire().await.map_err(storage)?;
            let _: i64 = sqlx::query_scalar!("SELECT 1 AS value")
                .fetch_one(&mut *connection)
                .await
                .map_err(storage)?;
            Ok(())
        })
        .await
        {
            Ok(result) => result,
            Err(_) => Err(ApplicationError::Storage(StorageReason::Contention)),
        }
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

#[async_trait]
impl GroupReader for SqliteLedgerStore {
    async fn list_groups(&self, archived: bool) -> Result<Vec<Group>, ApplicationError> {
        let archived = i64::from(archived);
        sqlx::query_as!(DbGroup, "SELECT id, name, currency, is_archived FROM groups WHERE is_archived = ? ORDER BY name, id", archived)
            .fetch_all(&self.pool)
            .await
            .map_err(storage)?
            .into_iter()
            .map(group)
            .collect()
    }

    async fn group(&self, id: EntityId) -> Result<Group, ApplicationError> {
        sqlx::query_as!(
            DbGroup,
            "SELECT id, name, currency, is_archived FROM groups WHERE id = ?",
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(storage)?
        .ok_or(ApplicationError::NotFound)
        .and_then(group)
    }
}

#[async_trait]
impl GroupRepository for SqliteLedgerStore {
    async fn create_group(
        &self,
        name: Name,
        currency: Currency,
    ) -> Result<Group, ApplicationError> {
        let _write_guard = self.write_guard().await?;
        let name = name.as_str();
        let currency = currency.code();
        let id = sqlx::query!(
            "INSERT INTO groups (name, currency) VALUES (?, ?)",
            name,
            currency
        )
        .execute(&self.pool)
        .await
        .map_err(storage)?
        .last_insert_rowid();
        self.group(id).await
    }

    async fn update_group(
        &self,
        id: EntityId,
        name: Name,
        currency: Currency,
    ) -> Result<Group, ApplicationError> {
        let _write_guard = self.write_guard().await?;
        let name = name.as_str();
        let currency = currency.code();
        changed(sqlx::query!("UPDATE groups SET name = ?, currency = ?, updated_at = datetime('now') WHERE id = ? AND is_archived = 0", name, currency, id)
            .execute(&self.pool)
            .await
            .map_err(storage)?)?;
        self.group(id).await
    }

    async fn set_group_archived(
        &self,
        id: EntityId,
        archived: bool,
    ) -> Result<(), ApplicationError> {
        let _write_guard = self.write_guard().await?;
        let archived = i64::from(archived);
        changed(
            sqlx::query!(
                "UPDATE groups SET is_archived = ?, updated_at = datetime('now') WHERE id = ?",
                archived,
                id
            )
            .execute(&self.pool)
            .await
            .map_err(storage)?,
        )
    }

    async fn delete_empty_group(&self, id: EntityId) -> Result<(), ApplicationError> {
        let _write_guard = self.write_guard().await?;
        let result = sqlx::query!("DELETE FROM groups WHERE id = ? AND is_archived = 0 AND NOT EXISTS (SELECT 1 FROM spendings WHERE group_id = ?)", id, id)
            .execute(&self.pool)
            .await
            .map_err(storage)?;
        if result.rows_affected() == 0 {
            if self.group(id).await.is_err() {
                return Err(ApplicationError::NotFound);
            }
            return Err(ApplicationError::Conflict);
        }
        Ok(())
    }
}

#[async_trait]
impl ParticipantRepository for SqliteLedgerStore {
    async fn list_participants(
        &self,
        archived: bool,
    ) -> Result<Vec<Participant>, ApplicationError> {
        let archived = i64::from(archived);
        sqlx::query_as!(DbParticipant, "SELECT id, name, color, is_archived FROM participants WHERE is_archived = ? ORDER BY name, id", archived)
            .fetch_all(&self.pool)
            .await
            .map_err(storage)?
            .into_iter()
            .map(participant)
            .collect()
    }

    async fn create_participant(
        &self,
        name: Name,
        color: Color,
    ) -> Result<Participant, ApplicationError> {
        let _write_guard = self.write_guard().await?;
        let name = name.as_str();
        let color = color.as_str();
        let id = sqlx::query!(
            "INSERT INTO participants (name, color) VALUES (?, ?)",
            name,
            color
        )
        .execute(&self.pool)
        .await
        .map_err(storage)?
        .last_insert_rowid();
        participant_by_id(&self.pool, id).await
    }

    async fn participant(&self, id: EntityId) -> Result<Participant, ApplicationError> {
        participant_by_id(&self.pool, id).await
    }

    async fn create_group_participant(
        &self,
        group_id: EntityId,
        name: Name,
        color: Color,
    ) -> Result<Participant, ApplicationError> {
        let _write_guard = self.write_guard().await?;
        let mut tx = self.pool.begin().await.map_err(storage)?;
        let name = name.as_str();
        let color = color.as_str();
        let id = sqlx::query!(
            "INSERT INTO participants (name, color) VALUES (?, ?)",
            name,
            color
        )
        .execute(&mut *tx)
        .await
        .map_err(storage)?
        .last_insert_rowid();
        let membership = sqlx::query!(
            "INSERT INTO group_members (group_id, participant_id) SELECT ?, ? WHERE EXISTS (SELECT 1 FROM groups WHERE id = ? AND is_archived = 0)",
            group_id,
            id,
            group_id
        )
        .execute(&mut *tx)
        .await
        .map_err(storage)?;
        if membership.rows_affected() == 0 {
            group_mutable_in_transaction(&mut tx, group_id).await?;
            return Err(ApplicationError::Storage(StorageReason::Unexpected));
        }
        tx.commit().await.map_err(storage)?;
        participant_by_id(&self.pool, id).await
    }

    async fn update_participant(
        &self,
        id: EntityId,
        name: Name,
        color: Color,
    ) -> Result<Participant, ApplicationError> {
        let _write_guard = self.write_guard().await?;
        let name = name.as_str();
        let color = color.as_str();
        let result = sqlx::query!("UPDATE participants SET name = ?, color = ?, updated_at = datetime('now') WHERE id = ? AND is_archived = 0", name, color, id)
            .execute(&self.pool)
            .await
            .map_err(storage)?;
        if result.rows_affected() == 0 {
            return match participant_by_id(&self.pool, id).await? {
                Participant {
                    is_archived: true, ..
                } => Err(ApplicationError::Conflict),
                _ => Err(ApplicationError::NotFound),
            };
        }
        participant_by_id(&self.pool, id).await
    }

    async fn set_participant_archived(
        &self,
        id: EntityId,
        archived: bool,
    ) -> Result<(), ApplicationError> {
        let _write_guard = self.write_guard().await?;
        let archived = i64::from(archived);
        changed(sqlx::query!("UPDATE participants SET is_archived = ?, updated_at = datetime('now') WHERE id = ?", archived, id)
            .execute(&self.pool)
            .await
            .map_err(storage)?)
    }

    async fn group_members(
        &self,
        group_id: EntityId,
    ) -> Result<Vec<(Participant, GroupMember)>, ApplicationError> {
        self.group(group_id).await?;
        sqlx::query_as!(DbGroupMember, "SELECT p.id AS \"id!: i64\", p.name, p.color, p.is_archived, gm.is_active AS \"is_active!: i64\" FROM group_members gm JOIN participants p ON p.id = gm.participant_id WHERE gm.group_id = ? ORDER BY p.name, p.id", group_id)
            .fetch_all(&self.pool)
            .await
            .map_err(storage)?
            .into_iter()
            .map(|row| {
                let participant_id = row.id;
                Ok((
                    participant(DbParticipant {
                        id: participant_id,
                        name: row.name,
                        color: row.color,
                        is_archived: row.is_archived,
                    })?,
                    GroupMember {
                        group_id,
                        participant_id,
                        is_active: decoded_bool(row.is_active)?,
                    },
                ))
            })
            .collect()
    }

    async fn add_member(
        &self,
        group_id: EntityId,
        participant_id: EntityId,
    ) -> Result<(), ApplicationError> {
        let _write_guard = self.write_guard().await?;
        let result = sqlx::query!("INSERT INTO group_members (group_id, participant_id) SELECT ?, ? WHERE EXISTS (SELECT 1 FROM groups WHERE id = ? AND is_archived = 0) AND EXISTS (SELECT 1 FROM participants WHERE id = ? AND is_archived = 0) ON CONFLICT(group_id, participant_id) DO UPDATE SET is_active = 1", group_id, participant_id, group_id, participant_id)
            .execute(&self.pool)
            .await
            .map_err(storage)?;
        if result.rows_affected() == 0 {
            self.group_mutable(group_id).await?;
            self.participant_mutable(participant_id).await?;
            return Err(ApplicationError::Storage(StorageReason::Unexpected));
        }
        Ok(())
    }

    async fn set_member_active(
        &self,
        group_id: EntityId,
        participant_id: EntityId,
        active: bool,
    ) -> Result<(), ApplicationError> {
        let _write_guard = self.write_guard().await?;
        let active = i64::from(active);
        changed(sqlx::query!("UPDATE group_members SET is_active = ? WHERE group_id = ? AND participant_id = ? AND EXISTS (SELECT 1 FROM groups WHERE id = ? AND is_archived = 0)", active, group_id, participant_id, group_id)
            .execute(&self.pool)
            .await
            .map_err(storage)?)
    }
}

#[async_trait]
impl SpendingEligibilityReader for SqliteLedgerStore {
    async fn eligible_participant_ids(
        &self,
        group_id: EntityId,
    ) -> Result<BTreeSet<EntityId>, ApplicationError> {
        self.group(group_id).await?;
        sqlx::query_scalar!(
            "SELECT gm.participant_id AS \"participant_id!: i64\" FROM group_members gm JOIN participants p ON p.id = gm.participant_id WHERE gm.group_id = ? AND gm.is_active = 1 AND p.is_archived = 0 ORDER BY gm.participant_id",
            group_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(storage)
        .map(|ids| ids.into_iter().collect())
    }
}

#[async_trait]
impl SpendingReader for SqliteLedgerStore {
    async fn spendings(&self, group_id: EntityId) -> Result<Vec<Spending>, ApplicationError> {
        Ok(self.ledger_snapshot_impl(group_id).await?.spendings)
    }

    async fn spending(
        &self,
        group_id: EntityId,
        spending_id: EntityId,
    ) -> Result<Spending, ApplicationError> {
        load_spending(&self.pool, group_id, spending_id).await
    }

    async fn spending_page(
        &self,
        group_id: EntityId,
        cursor: Option<SpendingCursor>,
    ) -> Result<SpendingPage, ApplicationError> {
        let (mut rows, direction) = match cursor {
            None => (
                sqlx::query_as!(DbSpendingSummary, "SELECT id, description, total_amount, currency, spending_type, spent_date FROM spendings WHERE group_id = ? ORDER BY spent_date DESC, id DESC LIMIT 26", group_id)
                    .fetch_all(&self.pool)
                    .await
                    .map_err(storage)?,
                None,
            ),
            Some(cursor) if cursor.direction == SpendingPageDirection::Older => (
                sqlx::query_as!(DbSpendingSummary, "SELECT id, description, total_amount, currency, spending_type, spent_date FROM spendings WHERE group_id = ? AND (spent_date < ? OR (spent_date = ? AND id < ?)) ORDER BY spent_date DESC, id DESC LIMIT 26", group_id, cursor.spent_date.to_string(), cursor.spent_date.to_string(), cursor.id)
                    .fetch_all(&self.pool)
                    .await
                    .map_err(storage)?,
                Some(SpendingPageDirection::Older),
            ),
            Some(cursor) => (
                sqlx::query_as!(DbSpendingSummary, "SELECT id, description, total_amount, currency, spending_type, spent_date FROM spendings WHERE group_id = ? AND (spent_date > ? OR (spent_date = ? AND id > ?)) ORDER BY spent_date ASC, id ASC LIMIT 26", group_id, cursor.spent_date.to_string(), cursor.spent_date.to_string(), cursor.id)
                    .fetch_all(&self.pool)
                    .await
                    .map_err(storage)?,
                Some(SpendingPageDirection::Newer),
            ),
        };
        let has_more = rows.len() > 25;
        rows.truncate(25);
        if direction == Some(SpendingPageDirection::Newer) {
            rows.reverse();
        }
        let items = rows
            .into_iter()
            .map(|row| spending_summary(group_id, row))
            .collect::<Result<Vec<_>, _>>()?;
        let older = has_more
            .then(|| {
                items.last().map(|item| SpendingCursor {
                    direction: SpendingPageDirection::Older,
                    spent_date: item.spent_date,
                    id: item.id,
                })
            })
            .flatten();
        let newer = direction
            .is_some()
            .then(|| {
                items.first().map(|item| SpendingCursor {
                    direction: SpendingPageDirection::Newer,
                    spent_date: item.spent_date,
                    id: item.id,
                })
            })
            .flatten();
        Ok(SpendingPage {
            items,
            older,
            newer,
        })
    }
}

#[async_trait]
impl LedgerSnapshotReader for SqliteLedgerStore {
    async fn ledger_snapshot(
        &self,
        group_id: EntityId,
    ) -> Result<LedgerSnapshot, ApplicationError> {
        self.ledger_snapshot_impl(group_id).await
    }
}

#[async_trait]
impl SpendingRepository for SqliteLedgerStore {
    async fn create_spending(&self, spending: Spending) -> Result<Spending, ApplicationError> {
        let _write_guard = self.write_guard().await?;
        save_spending(&self.pool, spending, false).await
    }

    async fn update_spending(&self, spending: Spending) -> Result<Spending, ApplicationError> {
        let _write_guard = self.write_guard().await?;
        save_spending(&self.pool, spending, true).await
    }

    async fn delete_spending(
        &self,
        group_id: EntityId,
        spending_id: EntityId,
    ) -> Result<(), ApplicationError> {
        let _write_guard = self.write_guard().await?;
        let result = sqlx::query!(
            "DELETE FROM spendings WHERE id = ? AND group_id = ? AND EXISTS (SELECT 1 FROM groups WHERE id = ? AND is_archived = 0)",
            spending_id,
            group_id,
            group_id
        )
        .execute(&self.pool)
        .await
        .map_err(storage)?;
        if result.rows_affected() == 0 {
            self.group_mutable(group_id).await?;
            return Err(ApplicationError::NotFound);
        }
        Ok(())
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

fn group(row: DbGroup) -> Result<Group, ApplicationError> {
    Ok(Group {
        id: row.id,
        name: Name::new(row.name)
            .map_err(|_| ApplicationError::Storage(StorageReason::InvalidData))?,
        currency: Currency::from_str(&row.currency)
            .map_err(|_| ApplicationError::Storage(StorageReason::InvalidData))?,
        is_archived: decoded_bool(row.is_archived)?,
    })
}

fn participant(row: DbParticipant) -> Result<Participant, ApplicationError> {
    Ok(Participant {
        id: row.id,
        name: Name::new(row.name)
            .map_err(|_| ApplicationError::Storage(StorageReason::InvalidData))?,
        color: Color::new(row.color)
            .map_err(|_| ApplicationError::Storage(StorageReason::InvalidData))?,
        is_archived: decoded_bool(row.is_archived)?,
    })
}

fn decoded_bool(value: i64) -> Result<bool, ApplicationError> {
    match value {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(ApplicationError::Storage(StorageReason::InvalidData)),
    }
}

async fn participant_by_id(
    pool: &SqlitePool,
    id: EntityId,
) -> Result<Participant, ApplicationError> {
    sqlx::query_as!(
        DbParticipant,
        "SELECT id, name, color, is_archived FROM participants WHERE id = ?",
        id
    )
    .fetch_optional(pool)
    .await
    .map_err(storage)?
    .ok_or(ApplicationError::NotFound)
    .and_then(participant)
}

async fn load_spending(
    pool: &SqlitePool,
    group_id: EntityId,
    id: EntityId,
) -> Result<Spending, ApplicationError> {
    let mut tx = pool.begin().await.map_err(storage)?;
    let row = sqlx::query_as!(DbSpending, "SELECT description, total_amount, currency, spending_type, spent_date FROM spendings WHERE id = ? AND group_id = ?", id, group_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(storage)?
        .ok_or(ApplicationError::NotFound)?;
    let currency = Currency::from_str(&row.currency)
        .map_err(|_| ApplicationError::Storage(StorageReason::InvalidData))?;
    let spending = Spending {
        id,
        group_id,
        description: Description::new(row.description)
            .map_err(|_| ApplicationError::Storage(StorageReason::InvalidData))?,
        total: canonical_decimal(&row.total_amount)?,
        currency,
        spending_type: SpendingType::from_str(&row.spending_type)
            .map_err(|_| ApplicationError::Storage(StorageReason::InvalidData))?,
        spent_date: chrono::NaiveDate::parse_from_str(&row.spent_date, "%Y-%m-%d")
            .map_err(|_| ApplicationError::Storage(StorageReason::InvalidData))?,
        payers: payer_rows(&mut tx, id).await?,
        shares: share_rows(&mut tx, id).await?,
    };
    spending
        .validate()
        .map_err(|_| ApplicationError::Storage(StorageReason::InvalidData))?;
    tx.commit().await.map_err(storage)?;
    Ok(spending)
}

fn canonical_decimal(value: &str) -> Result<rust_decimal::Decimal, ApplicationError> {
    parse_decimal(value).map_err(|_| ApplicationError::Storage(StorageReason::InvalidData))
}

fn spending_summary(
    group_id: EntityId,
    row: DbSpendingSummary,
) -> Result<SpendingSummary, ApplicationError> {
    let currency = Currency::from_str(&row.currency)
        .map_err(|_| ApplicationError::Storage(StorageReason::InvalidData))?;
    let total = canonical_decimal(&row.total_amount)?;
    validate_amount(total, currency, "total")
        .map_err(|_| ApplicationError::Storage(StorageReason::InvalidData))?;
    SpendingType::from_str(&row.spending_type)
        .map_err(|_| ApplicationError::Storage(StorageReason::InvalidData))?;
    let spent_date = chrono::NaiveDate::parse_from_str(&row.spent_date, "%Y-%m-%d")
        .map_err(|_| ApplicationError::Storage(StorageReason::InvalidData))?;
    let earliest = chrono::NaiveDate::from_ymd_opt(2025, 1, 1)
        .ok_or(ApplicationError::Storage(StorageReason::InvalidData))?;
    if spent_date < earliest {
        return Err(ApplicationError::Storage(StorageReason::InvalidData));
    }
    Ok(SpendingSummary {
        id: row.id,
        group_id,
        description: Description::new(row.description)
            .map_err(|_| ApplicationError::Storage(StorageReason::InvalidData))?,
        total,
        currency,
        spent_date,
    })
}

fn allocation(row: DbAllocation) -> Result<Allocation, ApplicationError> {
    Ok(Allocation {
        participant_id: row.participant_id,
        amount: canonical_decimal(&row.amount)?,
    })
}

async fn payer_rows(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    spending_id: EntityId,
) -> Result<Vec<Allocation>, ApplicationError> {
    sqlx::query_as!(DbAllocation, "SELECT participant_id, paid_amount AS amount FROM spending_payers WHERE spending_id = ? ORDER BY participant_id", spending_id)
        .fetch_all(&mut **tx)
        .await
        .map_err(storage)?
        .into_iter()
        .map(allocation)
        .collect()
}

async fn share_rows(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    spending_id: EntityId,
) -> Result<Vec<Allocation>, ApplicationError> {
    sqlx::query_as!(DbAllocation, "SELECT participant_id, share_amount AS amount FROM spending_shares WHERE spending_id = ? ORDER BY participant_id", spending_id)
        .fetch_all(&mut **tx)
        .await
        .map_err(storage)?
        .into_iter()
        .map(allocation)
        .collect()
}

async fn insert_payer(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    spending_id: EntityId,
    allocation: &Allocation,
) -> Result<(), ApplicationError> {
    let amount = format_decimal(allocation.amount);
    sqlx::query!(
        "INSERT INTO spending_payers (spending_id, participant_id, paid_amount) VALUES (?, ?, ?)",
        spending_id,
        allocation.participant_id,
        amount
    )
    .execute(&mut **tx)
    .await
    .map_err(storage)?;
    Ok(())
}

async fn insert_share(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    spending_id: EntityId,
    allocation: &Allocation,
) -> Result<(), ApplicationError> {
    let amount = format_decimal(allocation.amount);
    sqlx::query!(
        "INSERT INTO spending_shares (spending_id, participant_id, share_amount) VALUES (?, ?, ?)",
        spending_id,
        allocation.participant_id,
        amount
    )
    .execute(&mut **tx)
    .await
    .map_err(storage)?;
    Ok(())
}

async fn save_spending(
    pool: &SqlitePool,
    spending: Spending,
    update: bool,
) -> Result<Spending, ApplicationError> {
    spending.validate()?;
    let mut tx = pool.begin().await.map_err(storage)?;
    let original_payers = if update {
        sqlx::query_scalar!(
            "SELECT participant_id FROM spending_payers WHERE spending_id = ?",
            spending.id
        )
        .fetch_all(&mut *tx)
        .await
        .map_err(storage)?
        .into_iter()
        .collect::<BTreeSet<_>>()
    } else {
        BTreeSet::new()
    };
    let original_shares = if update {
        sqlx::query_scalar!(
            "SELECT participant_id FROM spending_shares WHERE spending_id = ?",
            spending.id
        )
        .fetch_all(&mut *tx)
        .await
        .map_err(storage)?
        .into_iter()
        .collect::<BTreeSet<_>>()
    } else {
        BTreeSet::new()
    };
    for (item, original) in spending
        .payers
        .iter()
        .zip(std::iter::repeat(&original_payers))
        .chain(
            spending
                .shares
                .iter()
                .zip(std::iter::repeat(&original_shares)),
        )
    {
        let count = sqlx::query_scalar!("SELECT COUNT(*) AS \"count!: i64\" FROM group_members gm JOIN participants p ON p.id = gm.participant_id WHERE gm.group_id = ? AND gm.participant_id = ? AND gm.is_active = 1 AND p.is_archived = 0", spending.group_id, item.participant_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(storage)?;
        if count != 1 && !original.contains(&item.participant_id) {
            return Err(ApplicationError::Conflict);
        }
    }
    let description = spending.description.as_str();
    let total_amount = format_decimal(spending.total);
    let currency = spending.currency.code();
    let spending_type = spending.spending_type.code();
    let spent_date = spending.spent_date.to_string();
    let id = if update {
        changed(sqlx::query!("UPDATE spendings SET description = ?, total_amount = ?, currency = ?, spending_type = ?, spent_date = ?, updated_at = datetime('now') WHERE id = ? AND group_id = ? AND EXISTS (SELECT 1 FROM groups WHERE id = ? AND is_archived = 0)", description, total_amount, currency, spending_type, spent_date, spending.id, spending.group_id, spending.group_id)
            .execute(&mut *tx)
            .await
            .map_err(storage)?)?;
        sqlx::query!(
            "DELETE FROM spending_payers WHERE spending_id = ?",
            spending.id
        )
        .execute(&mut *tx)
        .await
        .map_err(storage)?;
        sqlx::query!(
            "DELETE FROM spending_shares WHERE spending_id = ?",
            spending.id
        )
        .execute(&mut *tx)
        .await
        .map_err(storage)?;
        spending.id
    } else {
        let result = sqlx::query!("INSERT INTO spendings (group_id, description, total_amount, currency, spending_type, spent_date) SELECT ?, ?, ?, ?, ?, ? WHERE EXISTS (SELECT 1 FROM groups WHERE id = ? AND is_archived = 0)", spending.group_id, description, total_amount, currency, spending_type, spent_date, spending.group_id)
            .execute(&mut *tx)
            .await
            .map_err(storage)?;
        if result.rows_affected() == 0 {
            group_mutable_in_transaction(&mut tx, spending.group_id).await?;
            return Err(ApplicationError::Storage(StorageReason::Unexpected));
        }
        result.last_insert_rowid()
    };
    for allocation in &spending.payers {
        insert_payer(&mut tx, id, allocation).await?;
    }
    for allocation in &spending.shares {
        insert_share(&mut tx, id, allocation).await?;
    }
    tx.commit().await.map_err(storage)?;
    load_spending(pool, spending.group_id, id).await
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

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
    async fn readiness_accepts_a_healthy_pool() {
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .expect("test pool");
        let store = SqliteLedgerRuntime::new(pool).store();

        assert!(store.check().await.is_ok());
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
        let store = SqliteLedgerRuntime::new(pool).store();

        assert!(matches!(
            store.check().await,
            Err(ApplicationError::Storage(StorageReason::Contention))
        ));
    }
}
