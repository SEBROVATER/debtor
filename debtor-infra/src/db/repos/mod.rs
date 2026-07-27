//! `SQLite` implementations of application persistence ports.

#![allow(clippy::needless_pass_by_value)]

use std::str::FromStr;

use async_trait::async_trait;
use debtor_application::{ApplicationError, LedgerStore};
use debtor_domain::currency::Currency;
use debtor_domain::model::{
    Allocation, Color, Description, EntityId, Group, GroupMember, Name, Participant, Spending,
    SpendingType,
};
use rust_decimal::Decimal;
use sqlx::{Row, SqlitePool};

/// SQLite-backed ledger persistence adapter.
pub struct SqliteLedgerStore {
    pool: SqlitePool,
}

impl SqliteLedgerStore {
    /// Creates an adapter from a configured pool.
    pub const fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    async fn group_mutable(&self, id: EntityId) -> Result<(), ApplicationError> {
        match sqlx::query_scalar::<_, i64>("SELECT is_archived FROM groups WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(storage)?
        {
            Some(0) => Ok(()),
            Some(_) => Err(ApplicationError::Conflict),
            None => Err(ApplicationError::NotFound),
        }
    }
}

#[async_trait]
impl LedgerStore for SqliteLedgerStore {
    async fn list_groups(&self, archived: bool) -> Result<Vec<Group>, ApplicationError> {
        sqlx::query("SELECT id, name, currency, is_archived FROM groups WHERE is_archived = ? ORDER BY name, id").bind(i64::from(archived)).fetch_all(&self.pool).await.map_err(storage)?.into_iter().map(group).collect()
    }
    async fn group(&self, id: EntityId) -> Result<Group, ApplicationError> {
        sqlx::query("SELECT id, name, currency, is_archived FROM groups WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(storage)?
            .ok_or(ApplicationError::NotFound)
            .and_then(group)
    }
    async fn create_group(
        &self,
        name: Name,
        currency: Currency,
    ) -> Result<Group, ApplicationError> {
        let id = sqlx::query("INSERT INTO groups (name, currency) VALUES (?, ?)")
            .bind(name.as_str())
            .bind(currency.code())
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
        self.group_mutable(id).await?;
        sqlx::query(
            "UPDATE groups SET name = ?, currency = ?, updated_at = datetime('now') WHERE id = ?",
        )
        .bind(name.as_str())
        .bind(currency.code())
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(storage)?;
        self.group(id).await
    }
    async fn set_group_archived(
        &self,
        id: EntityId,
        archived: bool,
    ) -> Result<(), ApplicationError> {
        changed(
            sqlx::query(
                "UPDATE groups SET is_archived = ?, updated_at = datetime('now') WHERE id = ?",
            )
            .bind(i64::from(archived))
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(storage)?,
        )
    }
    async fn delete_empty_group(&self, id: EntityId) -> Result<(), ApplicationError> {
        let count =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM spendings WHERE group_id = ?")
                .bind(id)
                .fetch_one(&self.pool)
                .await
                .map_err(storage)?;
        if count > 0 {
            return Err(ApplicationError::Conflict);
        }
        changed(
            sqlx::query("DELETE FROM groups WHERE id = ?")
                .bind(id)
                .execute(&self.pool)
                .await
                .map_err(storage)?,
        )
    }
    async fn list_participants(
        &self,
        archived: bool,
    ) -> Result<Vec<Participant>, ApplicationError> {
        sqlx::query("SELECT id, name, color, is_archived FROM participants WHERE is_archived = ? ORDER BY name, id").bind(i64::from(archived)).fetch_all(&self.pool).await.map_err(storage)?.into_iter().map(participant).collect()
    }
    async fn create_participant(
        &self,
        name: Name,
        color: Color,
    ) -> Result<Participant, ApplicationError> {
        let id = sqlx::query("INSERT INTO participants (name, color) VALUES (?, ?)")
            .bind(name.as_str())
            .bind(color.as_str())
            .execute(&self.pool)
            .await
            .map_err(storage)?
            .last_insert_rowid();
        participant_by_id(&self.pool, id).await
    }
    async fn update_participant(
        &self,
        id: EntityId,
        name: Name,
        color: Color,
    ) -> Result<Participant, ApplicationError> {
        changed(sqlx::query("UPDATE participants SET name = ?, color = ?, updated_at = datetime('now') WHERE id = ?").bind(name.as_str()).bind(color.as_str()).bind(id).execute(&self.pool).await.map_err(storage)?)?;
        participant_by_id(&self.pool, id).await
    }
    async fn set_participant_archived(
        &self,
        id: EntityId,
        archived: bool,
    ) -> Result<(), ApplicationError> {
        changed(sqlx::query("UPDATE participants SET is_archived = ?, updated_at = datetime('now') WHERE id = ?").bind(i64::from(archived)).bind(id).execute(&self.pool).await.map_err(storage)?)
    }
    async fn group_members(
        &self,
        group_id: EntityId,
    ) -> Result<Vec<(Participant, GroupMember)>, ApplicationError> {
        self.group(group_id).await?;
        sqlx::query("SELECT p.id, p.name, p.color, p.is_archived, gm.is_active FROM group_members gm JOIN participants p ON p.id = gm.participant_id WHERE gm.group_id = ? ORDER BY p.name, p.id").bind(group_id).fetch_all(&self.pool).await.map_err(storage)?.into_iter().map(|row| { let id = row.get("id"); let active = row.get::<i64, _>("is_active") != 0; Ok((participant(row)?, GroupMember { group_id, participant_id: id, is_active: active })) }).collect()
    }
    async fn add_member(
        &self,
        group_id: EntityId,
        participant_id: EntityId,
    ) -> Result<(), ApplicationError> {
        self.group_mutable(group_id).await?;
        match sqlx::query_scalar::<_, i64>("SELECT is_archived FROM participants WHERE id = ?")
            .bind(participant_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(storage)?
        {
            Some(0) => {}
            Some(_) => return Err(ApplicationError::Conflict),
            None => return Err(ApplicationError::NotFound),
        }
        sqlx::query("INSERT INTO group_members (group_id, participant_id) VALUES (?, ?) ON CONFLICT(group_id, participant_id) DO UPDATE SET is_active = 1").bind(group_id).bind(participant_id).execute(&self.pool).await.map_err(storage)?;
        Ok(())
    }
    async fn set_member_active(
        &self,
        group_id: EntityId,
        participant_id: EntityId,
        active: bool,
    ) -> Result<(), ApplicationError> {
        self.group_mutable(group_id).await?;
        changed(
            sqlx::query(
                "UPDATE group_members SET is_active = ? WHERE group_id = ? AND participant_id = ?",
            )
            .bind(i64::from(active))
            .bind(group_id)
            .bind(participant_id)
            .execute(&self.pool)
            .await
            .map_err(storage)?,
        )
    }
    async fn delete_unused_member(
        &self,
        group_id: EntityId,
        participant_id: EntityId,
    ) -> Result<(), ApplicationError> {
        self.group_mutable(group_id).await?;
        let used = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM spendings s WHERE s.group_id = ? AND (EXISTS (SELECT 1 FROM spending_payers p WHERE p.spending_id = s.id AND p.participant_id = ?) OR EXISTS (SELECT 1 FROM spending_shares sh WHERE sh.spending_id = s.id AND sh.participant_id = ?))").bind(group_id).bind(participant_id).bind(participant_id).fetch_one(&self.pool).await.map_err(storage)?;
        if used > 0 {
            return Err(ApplicationError::Conflict);
        }
        changed(
            sqlx::query("DELETE FROM group_members WHERE group_id = ? AND participant_id = ?")
                .bind(group_id)
                .bind(participant_id)
                .execute(&self.pool)
                .await
                .map_err(storage)?,
        )
    }
    async fn spendings(&self, group_id: EntityId) -> Result<Vec<Spending>, ApplicationError> {
        self.group(group_id).await?;
        let ids = sqlx::query_scalar::<_, i64>(
            "SELECT id FROM spendings WHERE group_id = ? ORDER BY spent_date DESC, id DESC",
        )
        .bind(group_id)
        .fetch_all(&self.pool)
        .await
        .map_err(storage)?;
        let mut output = Vec::new();
        for id in ids {
            output.push(load_spending(&self.pool, group_id, id).await?);
        }
        Ok(output)
    }
    async fn spending(
        &self,
        group_id: EntityId,
        spending_id: EntityId,
    ) -> Result<Spending, ApplicationError> {
        load_spending(&self.pool, group_id, spending_id).await
    }
    async fn create_spending(&self, spending: Spending) -> Result<Spending, ApplicationError> {
        save_spending(&self.pool, spending, false).await
    }
    async fn update_spending(&self, spending: Spending) -> Result<Spending, ApplicationError> {
        save_spending(&self.pool, spending, true).await
    }
    async fn delete_spending(
        &self,
        group_id: EntityId,
        spending_id: EntityId,
    ) -> Result<(), ApplicationError> {
        self.group_mutable(group_id).await?;
        changed(
            sqlx::query("DELETE FROM spendings WHERE id = ? AND group_id = ?")
                .bind(spending_id)
                .bind(group_id)
                .execute(&self.pool)
                .await
                .map_err(storage)?,
        )
    }
}

fn storage(error: sqlx::Error) -> ApplicationError {
    ApplicationError::Storage(error.to_string())
}
fn changed(result: sqlx::sqlite::SqliteQueryResult) -> Result<(), ApplicationError> {
    if result.rows_affected() == 0 {
        Err(ApplicationError::NotFound)
    } else {
        Ok(())
    }
}
fn group(row: sqlx::sqlite::SqliteRow) -> Result<Group, ApplicationError> {
    Ok(Group {
        id: row.get("id"),
        name: Name::new(row.get::<String, _>("name"))?,
        currency: Currency::from_str(&row.get::<String, _>("currency"))
            .map_err(|_| ApplicationError::Storage("invalid persisted currency".into()))?,
        is_archived: row.get::<i64, _>("is_archived") != 0,
    })
}
fn participant(row: sqlx::sqlite::SqliteRow) -> Result<Participant, ApplicationError> {
    Ok(Participant {
        id: row.get("id"),
        name: Name::new(row.get::<String, _>("name"))?,
        color: Color::new(row.get::<String, _>("color"))?,
        is_archived: row.get::<i64, _>("is_archived") != 0,
    })
}
async fn participant_by_id(
    pool: &SqlitePool,
    id: EntityId,
) -> Result<Participant, ApplicationError> {
    sqlx::query("SELECT id, name, color, is_archived FROM participants WHERE id = ?")
        .bind(id)
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
    let row = sqlx::query("SELECT id, description, total_amount, currency, spending_type, spent_date FROM spendings WHERE id = ? AND group_id = ?").bind(id).bind(group_id).fetch_optional(pool).await.map_err(storage)?.ok_or(ApplicationError::NotFound)?;
    let currency = Currency::from_str(&row.get::<String, _>("currency"))
        .map_err(|_| ApplicationError::Storage("invalid currency".into()))?;
    let spending = Spending {
        id,
        group_id,
        description: Description::new(row.get::<String, _>("description"))?,
        total: Decimal::from_str(&row.get::<String, _>("total_amount"))
            .map_err(|e| ApplicationError::Storage(e.to_string()))?,
        currency,
        spending_type: SpendingType::from_str(&row.get::<String, _>("spending_type"))
            .map_err(|_| ApplicationError::Storage("invalid type".into()))?,
        spent_date: chrono::NaiveDate::parse_from_str(
            &row.get::<String, _>("spent_date"),
            "%Y-%m-%d",
        )
        .map_err(|e| ApplicationError::Storage(e.to_string()))?,
        payers: allocation_rows(pool, "spending_payers", "paid_amount", id).await?,
        shares: allocation_rows(pool, "spending_shares", "share_amount", id).await?,
    };
    spending.validate()?;
    Ok(spending)
}
async fn allocation_rows(
    pool: &SqlitePool,
    table: &str,
    amount_column: &str,
    id: EntityId,
) -> Result<Vec<Allocation>, ApplicationError> {
    let statement = format!(
        "SELECT participant_id, {amount_column} FROM {table} WHERE spending_id = ? ORDER BY participant_id"
    );
    sqlx::query(&statement)
        .bind(id)
        .fetch_all(pool)
        .await
        .map_err(storage)?
        .into_iter()
        .map(|r| {
            Ok(Allocation {
                participant_id: r.get("participant_id"),
                amount: Decimal::from_str(&r.get::<String, _>(amount_column))
                    .map_err(|e| ApplicationError::Storage(e.to_string()))?,
            })
        })
        .collect()
}

async fn save_spending(
    pool: &SqlitePool,
    spending: Spending,
    update: bool,
) -> Result<Spending, ApplicationError> {
    spending.validate()?;
    let mut tx = pool.begin().await.map_err(storage)?;
    match sqlx::query_scalar::<_, i64>("SELECT is_archived FROM groups WHERE id = ?")
        .bind(spending.group_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(storage)?
    {
        Some(0) => {}
        Some(_) => return Err(ApplicationError::Conflict),
        None => return Err(ApplicationError::NotFound),
    }
    for item in spending.payers.iter().chain(&spending.shares) {
        let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM group_members gm JOIN participants p ON p.id = gm.participant_id WHERE gm.group_id = ? AND gm.participant_id = ? AND gm.is_active = 1 AND p.is_archived = 0").bind(spending.group_id).bind(item.participant_id).fetch_one(&mut *tx).await.map_err(storage)?;
        if count != 1 {
            return Err(ApplicationError::Conflict);
        }
    }
    let id = if update {
        changed(sqlx::query("UPDATE spendings SET description = ?, total_amount = ?, currency = ?, spending_type = ?, spent_date = ?, updated_at = datetime('now') WHERE id = ? AND group_id = ?").bind(spending.description.as_str()).bind(spending.total.to_string()).bind(spending.currency.code()).bind(spending.spending_type.code()).bind(spending.spent_date.to_string()).bind(spending.id).bind(spending.group_id).execute(&mut *tx).await.map_err(storage)?)?;
        sqlx::query("DELETE FROM spending_payers WHERE spending_id = ?")
            .bind(spending.id)
            .execute(&mut *tx)
            .await
            .map_err(storage)?;
        sqlx::query("DELETE FROM spending_shares WHERE spending_id = ?")
            .bind(spending.id)
            .execute(&mut *tx)
            .await
            .map_err(storage)?;
        spending.id
    } else {
        sqlx::query("INSERT INTO spendings (group_id, description, total_amount, currency, spending_type, spent_date) VALUES (?, ?, ?, ?, ?, ?)").bind(spending.group_id).bind(spending.description.as_str()).bind(spending.total.to_string()).bind(spending.currency.code()).bind(spending.spending_type.code()).bind(spending.spent_date.to_string()).execute(&mut *tx).await.map_err(storage)?.last_insert_rowid()
    };
    for (table, column, items) in [
        ("spending_payers", "paid_amount", &spending.payers),
        ("spending_shares", "share_amount", &spending.shares),
    ] {
        let statement =
            format!("INSERT INTO {table} (spending_id, participant_id, {column}) VALUES (?, ?, ?)");
        for item in items {
            sqlx::query(&statement)
                .bind(id)
                .bind(item.participant_id)
                .bind(item.amount.to_string())
                .execute(&mut *tx)
                .await
                .map_err(storage)?;
        }
    }
    tx.commit().await.map_err(storage)?;
    load_spending(pool, spending.group_id, id).await
}
