use std::collections::BTreeSet;
use std::str::FromStr;
use std::sync::atomic::{AtomicU64, Ordering};

use async_trait::async_trait;
use debtor_application::{
    ApplicationError, GroupReader, SpendingCursor, SpendingDetail, SpendingEligibilityReader,
    SpendingHistoryPage, SpendingHistoryRow, SpendingPage, SpendingPageDirection, SpendingReader,
    SpendingRepository, SpendingSummary, StorageReason,
};
use debtor_domain::currency::Currency;
use debtor_domain::model::{Allocation, Description, EntityId, Spending, SpendingType};
use debtor_domain::money::format_decimal;

use super::decoding::{
    DbAllocation, DbParticipant, DbSpending, DbSpendingHistory, DbSpendingSummary, allocation,
    canonical_decimal, invalid, participant, spending_summary,
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

async fn participant_in_group(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    group_id: EntityId,
    participant_id: EntityId,
) -> Result<debtor_domain::model::Participant, ApplicationError> {
    sqlx::query_as!(
        DbParticipant,
        "SELECT id, name, color, is_archived FROM participants WHERE id = ? AND group_id = ?",
        participant_id,
        group_id
    )
    .fetch_optional(&mut **tx)
    .await
    .map_err(storage)?
    .ok_or_else(invalid)
    .and_then(participant)
}

async fn named_allocations(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    group_id: EntityId,
    allocations: Vec<Allocation>,
) -> Result<Vec<(debtor_domain::model::Participant, Allocation)>, ApplicationError> {
    let mut result = Vec::with_capacity(allocations.len());
    for allocation in allocations {
        let participant = participant_in_group(tx, group_id, allocation.participant_id).await?;
        result.push((participant, allocation));
    }
    Ok(result)
}

async fn load_detail(
    pool: &sqlx::SqlitePool,
    group_id: EntityId,
    id: EntityId,
) -> Result<SpendingDetail, ApplicationError> {
    let mut tx = pool.begin().await.map_err(storage)?;
    let group_row = sqlx::query_as!(
        super::decoding::DbGroup,
        "SELECT id, name, currency, is_archived FROM groups WHERE id = ?",
        group_id
    )
    .fetch_optional(&mut *tx)
    .await
    .map_err(storage)?
    .ok_or(ApplicationError::NotFound)?;
    let group = super::decoding::group(group_row)?;
    let row = sqlx::query_as!(
        DbSpending,
        "SELECT description, total_amount, currency, spending_type, spent_date FROM spendings WHERE id = ? AND group_id = ?",
        id,
        group_id
    )
    .fetch_optional(&mut *tx)
    .await
    .map_err(storage)?
    .ok_or(ApplicationError::NotFound)?;
    let payers = payer_rows(&mut tx, id).await?;
    let shares = share_rows(&mut tx, id).await?;
    let spending = Spending {
        id,
        group_id,
        description: debtor_domain::model::Description::new(row.description)
            .map_err(|_| invalid())?,
        total: canonical_decimal(&row.total_amount)?,
        currency: Currency::from_str(&row.currency).map_err(|_| invalid())?,
        spending_type: SpendingType::from_str(&row.spending_type).map_err(|_| invalid())?,
        spent_date: chrono::NaiveDate::parse_from_str(&row.spent_date, "%Y-%m-%d")
            .map_err(|_| invalid())?,
        payers: payers.clone(),
        shares: shares.clone(),
    };
    spending.validate().map_err(|_| invalid())?;
    let named_payers = named_allocations(&mut tx, group_id, payers).await?;
    let named_shares = named_allocations(&mut tx, group_id, shares).await?;
    tx.commit().await.map_err(storage)?;
    Ok(SpendingDetail {
        group,
        spending,
        payers: named_payers,
        shares: named_shares,
    })
}

fn history_summary(
    group_id: EntityId,
    row: DbSpendingHistory,
) -> Result<
    (
        SpendingSummary,
        debtor_domain::model::Participant,
        rust_decimal::Decimal,
    ),
    ApplicationError,
> {
    let payer_id = row.payer_id.ok_or_else(invalid)?;
    let payer_amount = canonical_decimal(&row.payer_amount.ok_or_else(invalid)?)?;
    let summary = spending_summary(
        group_id,
        DbSpendingSummary {
            id: row.id,
            description: row.description,
            total_amount: row.total_amount,
            currency: row.currency,
            spending_type: row.spending_type,
            spent_date: row.spent_date,
        },
    )?;
    let payer = participant(DbParticipant {
        id: payer_id,
        name: row.payer_name.ok_or_else(invalid)?,
        color: row.payer_color.ok_or_else(invalid)?,
        is_archived: row.payer_archived.ok_or_else(invalid)?,
    })?;
    Ok((summary, payer, payer_amount))
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
    generation: &AtomicU64,
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
    generation.fetch_add(1, Ordering::Release);
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

    async fn spending_detail(
        &self,
        group_id: EntityId,
        spending_id: EntityId,
    ) -> Result<SpendingDetail, ApplicationError> {
        load_detail(&self.pool, group_id, spending_id).await
    }

    async fn spending_history_page(
        &self,
        group_id: EntityId,
        cursor: Option<SpendingCursor>,
    ) -> Result<SpendingHistoryPage, ApplicationError> {
        let mut tx = self.pool.begin().await.map_err(storage)?;
        let group = sqlx::query_as!(
            super::decoding::DbGroup,
            "SELECT id, name, currency, is_archived FROM groups WHERE id = ?",
            group_id
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(storage)?
        .ok_or(ApplicationError::NotFound)
        .and_then(super::decoding::group)?;
        let (mut rows, direction) = match cursor {
            None => (sqlx::query_as!(DbSpendingHistory, "SELECT s.id, s.description, s.total_amount, s.currency, s.spending_type, s.spent_date, p.id AS payer_id, sp.paid_amount AS payer_amount, p.name AS payer_name, p.color AS payer_color, p.is_archived AS payer_archived FROM spendings s LEFT JOIN spending_payers sp ON sp.spending_id = s.id LEFT JOIN participants p ON p.id = sp.participant_id AND p.group_id = ? WHERE s.group_id = ? ORDER BY s.spent_date DESC, s.id DESC LIMIT 26", group_id, group_id).fetch_all(&mut *tx).await.map_err(storage)?, None),
            Some(cursor) if cursor.direction == SpendingPageDirection::Older => (sqlx::query_as!(DbSpendingHistory, "SELECT s.id, s.description, s.total_amount, s.currency, s.spending_type, s.spent_date, p.id AS payer_id, sp.paid_amount AS payer_amount, p.name AS payer_name, p.color AS payer_color, p.is_archived AS payer_archived FROM spendings s LEFT JOIN spending_payers sp ON sp.spending_id = s.id LEFT JOIN participants p ON p.id = sp.participant_id AND p.group_id = ? WHERE s.group_id = ? AND (s.spent_date < ? OR (s.spent_date = ? AND s.id < ?)) ORDER BY s.spent_date DESC, s.id DESC LIMIT 26", group_id, group_id, cursor.spent_date.to_string(), cursor.spent_date.to_string(), cursor.id).fetch_all(&mut *tx).await.map_err(storage)?, Some(SpendingPageDirection::Older)),
            Some(cursor) => (sqlx::query_as!(DbSpendingHistory, "SELECT s.id, s.description, s.total_amount, s.currency, s.spending_type, s.spent_date, p.id AS payer_id, sp.paid_amount AS payer_amount, p.name AS payer_name, p.color AS payer_color, p.is_archived AS payer_archived FROM spendings s LEFT JOIN spending_payers sp ON sp.spending_id = s.id LEFT JOIN participants p ON p.id = sp.participant_id AND p.group_id = ? WHERE s.group_id = ? AND (s.spent_date > ? OR (s.spent_date = ? AND s.id > ?)) ORDER BY s.spent_date ASC, s.id ASC LIMIT 26", group_id, group_id, cursor.spent_date.to_string(), cursor.spent_date.to_string(), cursor.id).fetch_all(&mut *tx).await.map_err(storage)?, Some(SpendingPageDirection::Newer)),
        };
        let has_more = rows.len() > 25;
        rows.truncate(25);
        if direction == Some(SpendingPageDirection::Newer) {
            rows.reverse();
        }
        let mut items = Vec::with_capacity(rows.len());
        for row in rows {
            let (spending_summary, payer, payer_amount) = history_summary(group_id, row)?;
            let payers = payer_rows(&mut tx, spending_summary.id).await?;
            let shares = share_rows(&mut tx, spending_summary.id).await?;
            let spending = Spending {
                id: spending_summary.id,
                group_id,
                description: spending_summary.description.clone(),
                total: spending_summary.total,
                currency: spending_summary.currency,
                spending_type: spending_summary.spending_type,
                spent_date: spending_summary.spent_date,
                payers,
                shares: shares.clone(),
            };
            spending.validate().map_err(|_| invalid())?;
            let named_shares = named_allocations(&mut tx, group_id, shares).await?;
            items.push(SpendingHistoryRow {
                spending: spending_summary,
                payer,
                payer_amount,
                shares: named_shares,
            });
        }
        let older = matches!(direction, Some(SpendingPageDirection::Newer)) || has_more;
        let newer = matches!(direction, Some(SpendingPageDirection::Older))
            || (matches!(direction, Some(SpendingPageDirection::Newer)) && has_more);
        let older = older
            .then(|| {
                items.last().map(|item| SpendingCursor {
                    direction: SpendingPageDirection::Older,
                    spent_date: item.spending.spent_date,
                    id: item.spending.id,
                })
            })
            .flatten();
        let newer = newer
            .then(|| {
                items.first().map(|item| SpendingCursor {
                    direction: SpendingPageDirection::Newer,
                    spent_date: item.spending.spent_date,
                    id: item.spending.id,
                })
            })
            .flatten();
        tx.commit().await.map_err(storage)?;
        Ok(SpendingHistoryPage {
            group,
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
        save_spending(&self.pool, &self.generation, spending, false).await
    }
    async fn update_spending(&self, spending: Spending) -> Result<Spending, ApplicationError> {
        let _write_guard = self.write_guard().await?;
        save_spending(&self.pool, &self.generation, spending, true).await
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
        self.committed();
        Ok(())
    }
}
