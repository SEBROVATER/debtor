use chrono::{NaiveDate, NaiveDateTime};
use rust_decimal::Decimal;
use sqlx::SqlitePool;
use thiserror::Error;

use crate::expenses::expense_repo::{ExpenseRepo, ExpenseRow};
use crate::expenses::share_repo::{ExpenseShareRow, ShareRepo};
use crate::expenses::share_splitter::{ShareInput, ShareSplitError, normalize_shares};
use crate::groups::group_repo::GroupRepo;
use crate::groups::member_repo::MemberRepo;

#[derive(Debug, Error)]
pub enum ExpenseError {
    #[error("expense not found")]
    NotFound,
    #[error("validation error: {0}")]
    Validation(String),
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error(transparent)]
    ShareSplit(#[from] ShareSplitError),
}

#[derive(Debug, Clone)]
pub struct CreateExpense {
    pub group_id: String,
    pub payer_member_id: String,
    pub amount: Decimal,
    pub currency: String,
    pub expense_date: NaiveDate,
    pub note: Option<String>,
    pub shares: Vec<ShareInput>,
}

#[derive(Debug, Clone)]
pub struct UpdateExpense {
    pub payer_member_id: String,
    pub amount: Decimal,
    pub currency: String,
    pub expense_date: NaiveDate,
    pub note: Option<String>,
    pub shares: Vec<ShareInput>,
}

#[derive(Debug, Clone)]
pub struct ExpenseWithShares {
    pub expense: ExpenseRow,
    pub shares: Vec<ExpenseShareRow>,
}

#[derive(Clone)]
pub struct ExpenseService {
    expense_repo: ExpenseRepo,
    share_repo: ShareRepo,
    group_repo: GroupRepo,
    member_repo: MemberRepo,
}

impl ExpenseService {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            expense_repo: ExpenseRepo::new(pool.clone()),
            share_repo: ShareRepo::new(pool.clone()),
            group_repo: GroupRepo::new(pool.clone()),
            member_repo: MemberRepo::new(pool),
        }
    }

    pub async fn create_expense(
        &self,
        request: CreateExpense,
        now: NaiveDateTime,
    ) -> Result<ExpenseWithShares, ExpenseError> {
        self.ensure_group_exists(&request.group_id).await?;
        self.ensure_member_active(&request.group_id, &request.payer_member_id)
            .await?;
        self.ensure_share_members(&request.group_id, &request.shares)
            .await?;

        let currency = normalize_currency(&request.currency)?;
        let allocations = normalize_shares(request.amount, request.shares)?;

        let expense = self
            .expense_repo
            .create(
                uuid::Uuid::new_v4().to_string(),
                request.group_id,
                request.payer_member_id,
                request.amount,
                currency,
                request.expense_date,
                request.note,
                now,
            )
            .await?;

        let shares = self
            .share_repo
            .replace_shares(&expense.id, allocations, now)
            .await?;

        Ok(ExpenseWithShares { expense, shares })
    }

    pub async fn update_expense(
        &self,
        expense_id: &str,
        request: UpdateExpense,
        now: NaiveDateTime,
    ) -> Result<ExpenseWithShares, ExpenseError> {
        let existing = self.expense_repo.find(expense_id).await?;
        let Some(existing) = existing else {
            return Err(ExpenseError::NotFound);
        };

        self.ensure_group_exists(&existing.group_id).await?;
        self.ensure_member_active(&existing.group_id, &request.payer_member_id)
            .await?;
        self.ensure_share_members(&existing.group_id, &request.shares)
            .await?;

        let currency = normalize_currency(&request.currency)?;
        let allocations = normalize_shares(request.amount, request.shares)?;

        let updated = self
            .expense_repo
            .update(
                expense_id,
                request.payer_member_id,
                request.amount,
                currency,
                request.expense_date,
                request.note,
                now,
            )
            .await?;

        let Some(updated) = updated else {
            return Err(ExpenseError::NotFound);
        };

        let shares = self
            .share_repo
            .replace_shares(&updated.id, allocations, now)
            .await?;

        Ok(ExpenseWithShares {
            expense: updated,
            shares,
        })
    }

    pub async fn delete_expense(&self, expense_id: &str) -> Result<bool, ExpenseError> {
        Ok(self.expense_repo.delete(expense_id).await?)
    }

    pub async fn list_expenses(
        &self,
        group_id: &str,
    ) -> Result<Vec<ExpenseRow>, ExpenseError> {
        Ok(self.expense_repo.list_by_group(group_id).await?)
    }

    async fn ensure_group_exists(&self, group_id: &str) -> Result<(), ExpenseError> {
        let exists = self.group_repo.find(group_id).await?;
        if exists.is_none() {
            return Err(ExpenseError::Validation("group not found".to_string()));
        }
        Ok(())
    }

    async fn ensure_member_active(
        &self,
        group_id: &str,
        member_id: &str,
    ) -> Result<(), ExpenseError> {
        let members = self.member_repo.list(group_id, false).await?;
        if members.iter().any(|m| m.id == member_id) {
            return Ok(());
        }
        Err(ExpenseError::Validation(
            "payer must be active group member".to_string(),
        ))
    }

    async fn ensure_share_members(
        &self,
        group_id: &str,
        shares: &[ShareInput],
    ) -> Result<(), ExpenseError> {
        let members = self.member_repo.list(group_id, false).await?;
        for share in shares {
            if !members.iter().any(|m| m.id == share.member_id) {
                return Err(ExpenseError::Validation(
                    "share members must be active group members".to_string(),
                ));
            }
        }
        Ok(())
    }
}

fn normalize_currency(input: &str) -> Result<String, ExpenseError> {
    let trimmed = input.trim().to_ascii_uppercase();
    if trimmed.len() != 3 || !trimmed.chars().all(|c| c.is_ascii_alphabetic()) {
        return Err(ExpenseError::Validation(
            "currency must be ISO-4217 code".to_string(),
        ));
    }
    Ok(trimmed)
}
