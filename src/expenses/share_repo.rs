use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::expenses::share_splitter::{ShareAllocation, ShareMode};

#[derive(Debug, Clone)]
pub struct ExpenseShareRow {
    pub id: String,
    pub expense_id: String,
    pub member_id: String,
    pub share_mode: String,
    pub share_value: Decimal,
    pub computed_amount: Decimal,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Clone)]
pub struct ShareRepo {
    pool: SqlitePool,
}

impl ShareRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn list_by_expense(
        &self,
        expense_id: &str,
    ) -> Result<Vec<ExpenseShareRow>, sqlx::Error> {
        sqlx::query_as!(
            ExpenseShareRow,
            r#"SELECT id, expense_id, member_id, share_mode,
               share_value as "share_value: Decimal",
               computed_amount as "computed_amount: Decimal",
               created_at as "created_at: NaiveDateTime",
               updated_at as "updated_at: NaiveDateTime"
               FROM expense_shares WHERE expense_id = ?"#,
            expense_id
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn replace_shares(
        &self,
        expense_id: &str,
        shares: Vec<ShareAllocation>,
        now: NaiveDateTime,
    ) -> Result<Vec<ExpenseShareRow>, sqlx::Error> {
        sqlx::query!("DELETE FROM expense_shares WHERE expense_id = ?", expense_id)
            .execute(&self.pool)
            .await?;

        if shares.is_empty() {
            return Ok(Vec::new());
        }

        for share in shares {
            let id = Uuid::new_v4().to_string();
            let mode = mode_to_string(share.mode);
            sqlx::query!(
                "INSERT INTO expense_shares (id, expense_id, member_id, share_mode, share_value, computed_amount, created_at, updated_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                id,
                expense_id,
                share.member_id,
                mode,
                share.share_value,
                share.computed_amount,
                now,
                now
            )
            .execute(&self.pool)
            .await?;
        }

        self.list_by_expense(expense_id).await
    }
}

fn mode_to_string(mode: ShareMode) -> String {
    match mode {
        ShareMode::Equal => "equal".to_string(),
        ShareMode::Percent => "percent".to_string(),
        ShareMode::Amount => "amount".to_string(),
    }
}
