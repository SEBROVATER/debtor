use std::collections::BTreeSet;
use std::str::FromStr;

use async_trait::async_trait;
use debtor_application::{
    ApplicationError, GroupReader, SpendingCursor, SpendingEligibilityReader, SpendingPage,
    SpendingPageDirection, SpendingReader, SpendingRepository, StorageReason,
};
use debtor_domain::currency::Currency;
use debtor_domain::model::{Allocation, Description, EntityId, Spending, SpendingType};
use debtor_domain::money::format_decimal;

use super::decoding::{
    DbAllocation, DbSpending, DbSpendingSummary, allocation, canonical_decimal, invalid,
    spending_summary,
};
use super::{SqliteLedgerStore, group_write_failure, group_write_failure_in_transaction, storage};

async fn payer_rows(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    spending_id: EntityId,
) -> Result<Vec<Allocation>, ApplicationError> {
    sqlx::query_as!(DbAllocation, "SELECT participant_id, paid_amount AS amount FROM spending_payers WHERE spending_id = ? ORDER BY participant_id", spending_id)
        .fetch_all(&mut **tx).await.map_err(storage)?.into_iter().map(allocation).collect()
}

async fn share_rows(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    spending_id: EntityId,
) -> Result<Vec<Allocation>, ApplicationError> {
    sqlx::query_as!(DbAllocation, "SELECT participant_id, share_amount AS amount FROM spending_shares WHERE spending_id = ? ORDER BY participant_id", spending_id)
        .fetch_all(&mut **tx).await.map_err(storage)?.into_iter().map(allocation).collect()
}

async fn load_spending(
    pool: &sqlx::SqlitePool,
    group_id: EntityId,
    id: EntityId,
) -> Result<Spending, ApplicationError> {
    let mut tx = pool.begin().await.map_err(storage)?;
    let row = sqlx::query_as!(DbSpending, "SELECT description, total_amount, currency, spending_type, spent_date FROM spendings WHERE id = ? AND group_id = ?", id, group_id)
        .fetch_optional(&mut *tx).await.map_err(storage)?.ok_or(ApplicationError::NotFound)?;
    let spending = Spending {
        id,
        group_id,
        description: Description::new(row.description).map_err(|_| invalid())?,
        total: canonical_decimal(&row.total_amount)?,
        currency: Currency::from_str(&row.currency).map_err(|_| invalid())?,
        spending_type: SpendingType::from_str(&row.spending_type).map_err(|_| invalid())?,
        spent_date: chrono::NaiveDate::parse_from_str(&row.spent_date, "%Y-%m-%d")
            .map_err(|_| invalid())?,
        payers: payer_rows(&mut tx, id).await?,
        shares: share_rows(&mut tx, id).await?,
    };
    spending.validate().map_err(|_| invalid())?;
    tx.commit().await.map_err(storage)?;
    Ok(spending)
}

async fn insert_payer(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    spending_id: EntityId,
    allocation: &Allocation,
) -> Result<(), ApplicationError> {
    sqlx::query!(
        "INSERT INTO spending_payers (spending_id, participant_id, paid_amount) VALUES (?, ?, ?)",
        spending_id,
        allocation.participant_id,
        format_decimal(allocation.amount)
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
    sqlx::query!(
        "INSERT INTO spending_shares (spending_id, participant_id, share_amount) VALUES (?, ?, ?)",
        spending_id,
        allocation.participant_id,
        format_decimal(allocation.amount)
    )
    .execute(&mut **tx)
    .await
    .map_err(storage)?;
    Ok(())
}

async fn save_spending(
    pool: &sqlx::SqlitePool,
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
            .fetch_one(&mut *tx).await.map_err(storage)?;
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
        let result = sqlx::query!("UPDATE spendings SET description = ?, total_amount = ?, currency = ?, spending_type = ?, spent_date = ?, updated_at = datetime('now') WHERE id = ? AND group_id = ? AND EXISTS (SELECT 1 FROM groups WHERE id = ? AND is_archived = 0)", description, total_amount, currency, spending_type, spent_date, spending.id, spending.group_id, spending.group_id).execute(&mut *tx).await.map_err(storage)?;
        if result.rows_affected() == 0 {
            return Err(group_write_failure_in_transaction(
                &mut tx,
                spending.group_id,
                ApplicationError::NotFound,
            )
            .await);
        }
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
        let result = sqlx::query!("INSERT INTO spendings (group_id, description, total_amount, currency, spending_type, spent_date) SELECT ?, ?, ?, ?, ?, ? WHERE EXISTS (SELECT 1 FROM groups WHERE id = ? AND is_archived = 0)", spending.group_id, description, total_amount, currency, spending_type, spent_date, spending.group_id).execute(&mut *tx).await.map_err(storage)?;
        if result.rows_affected() == 0 {
            return Err(group_write_failure_in_transaction(
                &mut tx,
                spending.group_id,
                ApplicationError::Storage(StorageReason::Unexpected),
            )
            .await);
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
    let mut committed = spending;
    committed.id = id;
    Ok(committed)
}

#[async_trait]
impl SpendingEligibilityReader for SqliteLedgerStore {
    async fn eligible_participant_ids(
        &self,
        group_id: EntityId,
    ) -> Result<BTreeSet<EntityId>, ApplicationError> {
        self.group(group_id).await?;
        sqlx::query_scalar!("SELECT gm.participant_id AS \"participant_id!: i64\" FROM group_members gm JOIN participants p ON p.id = gm.participant_id WHERE gm.group_id = ? AND gm.is_active = 1 AND p.is_archived = 0 ORDER BY gm.participant_id", group_id)
            .fetch_all(&self.pool).await.map_err(storage).map(|ids| ids.into_iter().collect())
    }
}

#[async_trait]
impl SpendingReader for SqliteLedgerStore {
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
            None => (sqlx::query_as!(DbSpendingSummary, "SELECT id, description, total_amount, currency, spending_type, spent_date FROM spendings WHERE group_id = ? ORDER BY spent_date DESC, id DESC LIMIT 26", group_id).fetch_all(&self.pool).await.map_err(storage)?, None),
            Some(cursor) if cursor.direction == SpendingPageDirection::Older => (sqlx::query_as!(DbSpendingSummary, "SELECT id, description, total_amount, currency, spending_type, spent_date FROM spendings WHERE group_id = ? AND (spent_date < ? OR (spent_date = ? AND id < ?)) ORDER BY spent_date DESC, id DESC LIMIT 26", group_id, cursor.spent_date.to_string(), cursor.spent_date.to_string(), cursor.id).fetch_all(&self.pool).await.map_err(storage)?, Some(SpendingPageDirection::Older)),
            Some(cursor) => (sqlx::query_as!(DbSpendingSummary, "SELECT id, description, total_amount, currency, spending_type, spent_date FROM spendings WHERE group_id = ? AND (spent_date > ? OR (spent_date = ? AND id > ?)) ORDER BY spent_date ASC, id ASC LIMIT 26", group_id, cursor.spent_date.to_string(), cursor.spent_date.to_string(), cursor.id).fetch_all(&self.pool).await.map_err(storage)?, Some(SpendingPageDirection::Newer)),
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
        let older = matches!(direction, Some(SpendingPageDirection::Newer)) || has_more;
        let newer = matches!(direction, Some(SpendingPageDirection::Older))
            || (matches!(direction, Some(SpendingPageDirection::Newer)) && has_more);
        let older = older
            .then(|| {
                items.last().map(|item| SpendingCursor {
                    direction: SpendingPageDirection::Older,
                    spent_date: item.spent_date,
                    id: item.id,
                })
            })
            .flatten();
        let newer = newer
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
        let result = sqlx::query!("DELETE FROM spendings WHERE id = ? AND group_id = ? AND EXISTS (SELECT 1 FROM groups WHERE id = ? AND is_archived = 0)", spending_id, group_id, group_id).execute(&self.pool).await.map_err(storage)?;
        if result.rows_affected() == 0 {
            return Err(
                group_write_failure(&self.pool, group_id, ApplicationError::NotFound).await,
            );
        }
        Ok(())
    }
}
