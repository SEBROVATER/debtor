use std::collections::BTreeMap;
use std::str::FromStr;
use std::time::Duration;

use async_trait::async_trait;
use debtor_application::{
    ApplicationError, DatabaseReadiness, LedgerSnapshot, LedgerSnapshotReader, StorageReason,
};
use debtor_domain::currency::Currency;
use debtor_domain::model::{
    Allocation, Description, EntityId, GroupMember, Spending, SpendingType,
};

use super::decoding::{
    DbGroup, DbGroupMember, DbSnapshotSpending, DbSpendingAllocation, canonical_decimal, group,
    invalid,
};
use super::{SqliteLedgerStore, storage};

const READINESS_TIMEOUT: Duration = Duration::from_secs(1);

async fn ledger_snapshot(
    pool: &sqlx::SqlitePool,
    group_id: EntityId,
) -> Result<LedgerSnapshot, ApplicationError> {
    let mut tx = pool.begin().await.map_err(storage)?;
    let group = sqlx::query_as!(
        DbGroup,
        "SELECT id, name, currency, is_archived FROM groups WHERE id = ?",
        group_id
    )
    .fetch_optional(&mut *tx)
    .await
    .map_err(storage)?
    .ok_or(ApplicationError::NotFound)
    .and_then(group)?;
    let members = sqlx::query_as!(DbGroupMember, "SELECT p.id AS \"id!: i64\", p.name, p.color, p.is_archived, gm.is_active AS \"is_active!: i64\" FROM group_members gm JOIN participants p ON p.id = gm.participant_id WHERE gm.group_id = ? ORDER BY p.name, p.id", group_id)
        .fetch_all(&mut *tx).await.map_err(storage)?;
    let participants = members
        .into_iter()
        .map(|row| {
            let participant = super::decoding::participant(super::decoding::DbParticipant {
                id: row.id,
                name: row.name,
                color: row.color,
                is_archived: row.is_archived,
            })?;
            Ok((
                participant,
                GroupMember {
                    group_id,
                    participant_id: row.id,
                    is_active: super::decoding::decoded_bool(row.is_active)?,
                },
            ))
        })
        .collect::<Result<Vec<_>, ApplicationError>>()?;
    let parents = sqlx::query_as!(DbSnapshotSpending, "SELECT id AS \"id!: i64\", description, total_amount, currency, spending_type, spent_date FROM spendings WHERE group_id = ? ORDER BY spent_date DESC, id DESC", group_id)
        .fetch_all(&mut *tx).await.map_err(storage)?;
    let payer_rows = sqlx::query_as!(DbSpendingAllocation, "SELECT sp.id AS \"spending_id!: i64\", p.participant_id, p.paid_amount AS amount FROM spending_payers p JOIN spendings sp ON sp.id = p.spending_id WHERE sp.group_id = ? ORDER BY sp.id, p.participant_id", group_id)
        .fetch_all(&mut *tx).await.map_err(storage)?;
    let share_rows = sqlx::query_as!(DbSpendingAllocation, "SELECT sp.id AS \"spending_id!: i64\", s.participant_id, s.share_amount AS amount FROM spending_shares s JOIN spendings sp ON sp.id = s.spending_id WHERE sp.group_id = ? ORDER BY sp.id, s.participant_id", group_id)
        .fetch_all(&mut *tx).await.map_err(storage)?;
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
            description: Description::new(row.description).map_err(|_| invalid())?,
            total: canonical_decimal(&row.total_amount)?,
            currency: Currency::from_str(&row.currency).map_err(|_| invalid())?,
            spending_type: SpendingType::from_str(&row.spending_type).map_err(|_| invalid())?,
            spent_date: chrono::NaiveDate::parse_from_str(&row.spent_date, "%Y-%m-%d")
                .map_err(|_| invalid())?,
            payers: payers.remove(&row.id).unwrap_or_default(),
            shares: shares.remove(&row.id).unwrap_or_default(),
        };
        spending.validate().map_err(|_| invalid())?;
        spendings.push(spending);
    }
    tx.commit().await.map_err(storage)?;
    Ok(LedgerSnapshot {
        group,
        spendings,
        participants,
    })
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

#[async_trait]
impl LedgerSnapshotReader for SqliteLedgerStore {
    async fn ledger_snapshot(
        &self,
        group_id: EntityId,
    ) -> Result<LedgerSnapshot, ApplicationError> {
        ledger_snapshot(&self.pool, group_id).await
    }
}
