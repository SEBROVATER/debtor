use crate::app::state::AppState;
use crate::debts::debt_summary_service::{DebtSummary, DebtSummaryError, DebtSummaryService};
use crate::web::error::AppError;
use chrono::Utc;

pub async fn handle_debt_summary(
    state: &AppState,
    group_id: &str,
) -> Result<DebtSummary, AppError> {
    let service = if let Some(provider) = state.exchange_provider.clone() {
        DebtSummaryService::with_provider(state.db.clone(), provider)
    } else {
        DebtSummaryService::new(state.db.clone())
    };
    let now = Utc::now().naive_utc();
    service
        .summarize_group(group_id, now)
        .await
        .map_err(map_debt_error)
}

fn map_debt_error(error: DebtSummaryError) -> AppError {
    match error {
        DebtSummaryError::GroupNotFound => AppError::NotFound,
        DebtSummaryError::ConversionBlocked => AppError::ConversionBlocked,
        DebtSummaryError::Database(err) => AppError::Database(err),
    }
}
