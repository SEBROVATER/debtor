use chrono::{NaiveDate, NaiveDateTime};
use rust_decimal::Decimal;
use sqlx::SqlitePool;

use crate::expenses::share_repo::ExpenseShareRow;

#[derive(Debug, Clone)]
pub struct ExpenseRow {
    pub id: String,
    pub group_id: String,
    pub payer_member_id: String,
    pub amount: Decimal,
    pub currency: String,
    pub note: Option<String>,
    pub expense_date: NaiveDate,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Clone)]
pub struct ExpenseRepo {
    pool: SqlitePool,
}

impl ExpenseRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        id: String,
        group_id: String,
        payer_member_id: String,
        amount: Decimal,
        currency: String,
        expense_date: NaiveDate,
        note: Option<String>,
        now: NaiveDateTime,
    ) -> Result<ExpenseRow, sqlx::Error> {
        sqlx::query!(
            "INSERT INTO expenses (id, group_id, payer_member_id, amount, currency, note, expense_date, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            id,
            group_id,
            payer_member_id,
            amount,
            currency,
            note,
            expense_date,
            now,
            now
        )
        .execute(&self.pool)
        .await?;

        Ok(ExpenseRow {
            id,
            group_id,
            payer_member_id,
            amount,
            currency,
            note,
            expense_date,
            created_at: now,
            updated_at: now,
        })
    }

    pub async fn find(&self, expense_id: &str) -> Result<Option<ExpenseRow>, sqlx::Error> {
        sqlx::query_as!(
            ExpenseRow,
            r#"SELECT id, group_id, payer_member_id,
               amount as "amount: Decimal", currency, note,
               expense_date as "expense_date: NaiveDate",
               created_at as "created_at: NaiveDateTime",
               updated_at as "updated_at: NaiveDateTime"
               FROM expenses WHERE id = ?"#,
            expense_id
        )
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn list_by_group(&self, group_id: &str) -> Result<Vec<ExpenseRow>, sqlx::Error> {
        sqlx::query_as!(
            ExpenseRow,
            r#"SELECT id, group_id, payer_member_id,
               amount as "amount: Decimal", currency, note,
               expense_date as "expense_date: NaiveDate",
               created_at as "created_at: NaiveDateTime",
               updated_at as "updated_at: NaiveDateTime"
               FROM expenses WHERE group_id = ?"#,
            group_id
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn list_with_shares_by_group(
        &self,
        group_id: &str,
    ) -> Result<Vec<(ExpenseRow, Vec<ExpenseShareRow>)>, sqlx::Error> {
        let expenses = self.list_by_group(group_id).await?;
        let mut result = Vec::with_capacity(expenses.len());

        for expense in expenses {
            let shares = sqlx::query_as!(
                ExpenseShareRow,
                r#"SELECT id, expense_id, member_id, share_mode,
                   share_value as "share_value: Decimal",
                   computed_amount as "computed_amount: Decimal",
                   created_at as "created_at: NaiveDateTime",
                   updated_at as "updated_at: NaiveDateTime"
                   FROM expense_shares WHERE expense_id = ?"#,
                expense.id
            )
            .fetch_all(&self.pool)
            .await?;

            result.push((expense, shares));
        }

        Ok(result)
    }

    pub async fn update(
        &self,
        expense_id: &str,
        payer_member_id: String,
        amount: Decimal,
        currency: String,
        expense_date: NaiveDate,
        note: Option<String>,
        now: NaiveDateTime,
    ) -> Result<Option<ExpenseRow>, sqlx::Error> {
        let Some(_existing) = self.find(expense_id).await? else {
            return Ok(None);
        };

        sqlx::query!(
            "UPDATE expenses SET payer_member_id = ?, amount = ?, currency = ?, expense_date = ?, note = ?, updated_at = ?
             WHERE id = ?",
            payer_member_id,
            amount,
            currency,
            expense_date,
            note,
            now,
            expense_id
        )
        .execute(&self.pool)
        .await?;

        self.find(expense_id).await
    }

    pub async fn delete(&self, expense_id: &str) -> Result<bool, sqlx::Error> {
        Ok(
            sqlx::query!("DELETE FROM expenses WHERE id = ?", expense_id)
                .execute(&self.pool)
                .await?
                .rows_affected()
                > 0,
        )
    }
}
