use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use rust_decimal::RoundingStrategy;

use crate::debts::balance_calculator::{ExpenseShareSummary, MemberShare, compute_balances};
use crate::debts::simplify::{DebtTransfer, minimal_transfers};
use crate::exchange_rates::rate_repo::RateRepo;
use crate::exchange_rates::rate_service::{RateError, RateService};
use crate::expenses::expense_repo::{ExpenseRepo, ExpenseRow};
use crate::expenses::share_repo::ExpenseShareRow;
use crate::groups::group_repo::{GroupRepo, GroupRow};
use sqlx::SqlitePool;
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct DebtSummary {
    pub transfers: Vec<DebtTransfer>,
    pub no_outstanding: bool,
    pub rates_stale_warning: bool,
    pub conversion_blocked_no_cache: bool,
}

#[derive(Debug, Error)]
pub enum DebtSummaryError {
    #[error("group not found")]
    GroupNotFound,
    #[error("conversion blocked")]
    ConversionBlocked,
    #[error(transparent)]
    Database(#[from] sqlx::Error),
}

#[derive(Clone)]
pub struct DebtSummaryService {
    expense_repo: ExpenseRepo,
    group_repo: GroupRepo,
    rate_service: Option<RateService>,
}

impl DebtSummaryService {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            expense_repo: ExpenseRepo::new(pool.clone()),
            group_repo: GroupRepo::new(pool),
            rate_service: None,
        }
    }

    pub fn with_rate_service(pool: SqlitePool, rate_service: RateService) -> Self {
        Self {
            expense_repo: ExpenseRepo::new(pool.clone()),
            group_repo: GroupRepo::new(pool),
            rate_service: Some(rate_service),
        }
    }

    pub fn with_provider(
        pool: SqlitePool,
        provider: std::sync::Arc<dyn crate::exchange_rates::rate_service::ExchangeProvider>,
    ) -> Self {
        let repo = RateRepo::new(pool.clone());
        let rate_service = RateService::new(repo, provider);
        Self::with_rate_service(pool, rate_service)
    }

    pub async fn summarize_group(
        &self,
        group_id: &str,
        now: NaiveDateTime,
    ) -> Result<DebtSummary, DebtSummaryError> {
        let group = self
            .group_repo
            .find(group_id)
            .await?
            .ok_or(DebtSummaryError::GroupNotFound)?;

        let expenses_with_shares = self
            .expense_repo
            .list_with_shares_by_group(group_id)
            .await?;
        let (summaries, rates_stale_warning) = self
            .collect_expense_summaries(&expenses_with_shares, &group, now)
            .await?;
        let balances = compute_balances(&summaries);

        let mut normalized: Vec<(String, Decimal)> = balances
            .into_iter()
            .filter(|(_, amount)| *amount != Decimal::ZERO)
            .collect();
        normalized.sort_by(|a, b| a.0.cmp(&b.0));

        let mut transfers = minimal_transfers(&normalized);
        for transfer in &mut transfers {
            transfer.amount = transfer
                .amount
                .round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero);
        }

        Ok(DebtSummary {
            no_outstanding: transfers.is_empty(),
            transfers,
            rates_stale_warning,
            conversion_blocked_no_cache: false,
        })
    }

    async fn collect_expense_summaries(
        &self,
        expenses_with_shares: &[(ExpenseRow, Vec<ExpenseShareRow>)],
        group: &GroupRow,
        now: NaiveDateTime,
    ) -> Result<(Vec<ExpenseShareSummary>, bool), DebtSummaryError> {
        let mut summaries = Vec::with_capacity(expenses_with_shares.len());
        let mut stale_warning = false;

        for (expense, shares) in expenses_with_shares {
            let rate = if expense.currency == group.target_currency {
                Decimal::ONE
            } else {
                let Some(rate_service) = &self.rate_service else {
                    return Err(DebtSummaryError::ConversionBlocked);
                };

                match rate_service
                    .get_rate(&expense.currency, &group.target_currency, now)
                    .await
                {
                    Ok(lookup) => {
                        if lookup.stale {
                            stale_warning = true;
                        }
                        lookup.rate
                    }
                    Err(RateError::MissingRate) => return Err(DebtSummaryError::ConversionBlocked),
                    Err(RateError::ProviderFailed(_)) => {
                        return Err(DebtSummaryError::ConversionBlocked);
                    }
                    Err(RateError::Database(err)) => return Err(DebtSummaryError::Database(err)),
                }
            };

            summaries.push(ExpenseShareSummary {
                payer_member_id: expense.payer_member_id.clone(),
                shares: shares
                    .iter()
                    .cloned()
                    .map(|share| map_share(share, rate))
                    .collect::<Vec<_>>(),
            });
        }

        Ok((summaries, stale_warning))
    }
}

fn map_share(share: ExpenseShareRow, rate: Decimal) -> MemberShare {
    MemberShare {
        member_id: share.member_id,
        amount: share.computed_amount * rate,
    }
}
