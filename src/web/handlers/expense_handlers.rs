use chrono::{NaiveDate, NaiveDateTime};
use rust_decimal::Decimal;

use crate::app::state::AppState;
use crate::debts::debt_summary_service::DebtSummary;
use crate::debts::debt_summary_service::DebtSummaryService;
use crate::expenses::expense_service::{
    CreateExpense, ExpenseError, ExpenseService, UpdateExpense,
};
use crate::expenses::share_splitter::ShareInput;
use crate::web::error::AppError;

#[derive(Debug, Clone)]
pub struct CreateExpenseRequest {
    pub group_id: String,
    pub payer_member_id: String,
    pub amount: Decimal,
    pub currency: String,
    pub expense_date: NaiveDate,
    pub note: Option<String>,
    pub shares: Vec<ShareInput>,
}

#[derive(Debug, Clone)]
pub struct UpdateExpenseRequest {
    pub payer_member_id: String,
    pub amount: Decimal,
    pub currency: String,
    pub expense_date: NaiveDate,
    pub note: Option<String>,
    pub shares: Vec<ShareInput>,
}

pub async fn handle_create_expense(
    state: &AppState,
    request: CreateExpenseRequest,
    now: NaiveDateTime,
) -> Result<crate::expenses::expense_service::ExpenseWithShares, AppError> {
    let service = ExpenseService::new(state.db.clone());
    service
        .create_expense(
            CreateExpense {
                group_id: request.group_id,
                payer_member_id: request.payer_member_id,
                amount: request.amount,
                currency: request.currency,
                expense_date: request.expense_date,
                note: request.note,
                shares: request.shares,
            },
            now,
        )
        .await
        .map_err(map_expense_error)
}

pub async fn handle_update_expense(
    state: &AppState,
    expense_id: &str,
    request: UpdateExpenseRequest,
    now: NaiveDateTime,
) -> Result<crate::expenses::expense_service::ExpenseWithShares, AppError> {
    let service = ExpenseService::new(state.db.clone());
    service
        .update_expense(
            expense_id,
            UpdateExpense {
                payer_member_id: request.payer_member_id,
                amount: request.amount,
                currency: request.currency,
                expense_date: request.expense_date,
                note: request.note,
                shares: request.shares,
            },
            now,
        )
        .await
        .map_err(map_expense_error)
}

pub async fn handle_delete_expense(state: &AppState, expense_id: &str) -> Result<bool, AppError> {
    let service = ExpenseService::new(state.db.clone());
    service
        .delete_expense(expense_id)
        .await
        .map_err(map_expense_error)
}

pub async fn handle_list_expenses(
    state: &AppState,
    group_id: &str,
) -> Result<Vec<crate::db::entities::expenses::Model>, AppError> {
    let service = ExpenseService::new(state.db.clone());
    service
        .list_expenses(group_id)
        .await
        .map_err(map_expense_error)
}

pub async fn refresh_debt_summary(
    state: &AppState,
    group_id: &str,
) -> Result<DebtSummary, AppError> {
    let service = if let Some(provider) = state.exchange_provider.clone() {
        DebtSummaryService::with_provider(state.db.clone(), provider)
    } else {
        DebtSummaryService::new(state.db.clone())
    };
    let now = chrono::Utc::now().naive_utc();
    service
        .summarize_group(group_id, now)
        .await
        .map_err(|err| match err {
            crate::debts::debt_summary_service::DebtSummaryError::GroupNotFound => {
                AppError::NotFound
            }
            crate::debts::debt_summary_service::DebtSummaryError::ConversionBlocked => {
                AppError::ConversionBlocked
            }
            crate::debts::debt_summary_service::DebtSummaryError::Database(db) => {
                AppError::Database(db)
            }
        })
}

fn map_expense_error(error: ExpenseError) -> AppError {
    match error {
        ExpenseError::Validation(msg) => AppError::Validation(msg),
        ExpenseError::NotFound => AppError::NotFound,
        ExpenseError::Database(err) => AppError::Database(err),
        ExpenseError::ShareSplit(err) => AppError::Validation(err.to_string()),
    }
}
