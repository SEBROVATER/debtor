use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use debtor_domain::currency::Currency;
use debtor_domain::debts::{
    CalculationError, Transfer, add_converted_spending, quantize_balances, simplify,
};
use debtor_domain::model::{EntityId, Group, Spending};
use futures::stream::{self, StreamExt};
use rust_decimal::Decimal;

use crate::{ApplicationError, CalculationReason, UnavailableReason};

/// Rate selection requested by the debts view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RateMode {
    /// Use each spending's historical date.
    Historical,
    /// Use the current UTC date.
    Current,
}

/// Exchange-rate information retained for transparent calculations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RateQuote {
    /// Requested source currency.
    pub base: Currency,
    /// Target currency.
    pub quote: Currency,
    /// Date requested by the calculation mode.
    pub requested_date: NaiveDate,
    /// Date returned by the provider.
    pub effective_date: NaiveDate,
    /// Exact rate.
    pub rate: Decimal,
    /// Whether an old cached result was used.
    pub is_stale: bool,
    /// Whether a future historical date used a current fallback.
    pub is_provisional: bool,
}

/// Source of wall-clock time, injected for deterministic tests.
pub trait Clock: Send + Sync {
    /// Returns the current UTC instant.
    fn now(&self) -> DateTime<Utc>;
}

/// Production UTC clock.
pub struct UtcClock;

impl Clock for UtcClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

/// Loads a rate for a currency pair and requested date.
#[async_trait]
pub trait ExchangeRateProvider: Send + Sync {
    /// Returns a rate quote, including fallback metadata.
    async fn rate(
        &self,
        base: Currency,
        quote: Currency,
        requested_date: NaiveDate,
        today: NaiveDate,
    ) -> Result<RateQuote, ApplicationError>;
}

/// Immutable ledger input for one debt calculation.
#[derive(Debug, Clone)]
pub struct LedgerSnapshot {
    /// Group identity and settlement currency captured with the spendings.
    pub group: Group,
    /// Complete spending aggregates from the same database snapshot.
    pub spendings: Vec<Spending>,
}

/// Reads one transactionally consistent ledger snapshot.
#[async_trait]
pub trait LedgerSnapshotReader: Send + Sync {
    /// Loads the group and all complete spendings from one read snapshot.
    async fn ledger_snapshot(&self, group_id: EntityId)
    -> Result<LedgerSnapshot, ApplicationError>;
}

/// Result of a debt calculation.
#[derive(Debug, Clone)]
pub struct DebtResult {
    /// Group currency.
    pub currency: Currency,
    /// Transfers which settle the rounded balances.
    pub transfers: Vec<Transfer>,
    /// Final target-currency balances by participant.
    pub balances: BTreeMap<EntityId, Decimal>,
    /// Unique provider quotes used.
    pub rates: Vec<RateQuote>,
    /// UTC instant at which this calculation was performed.
    pub calculated_at: DateTime<Utc>,
}

/// Inbound debt operations.
#[async_trait]
pub trait DebtUseCases: Send + Sync {
    /// Calculates advisory transfers for all group history.
    async fn calculate(
        &self,
        group_id: EntityId,
        mode: RateMode,
    ) -> Result<DebtResult, ApplicationError>;
}

/// Debt workflow implementation.
pub struct DebtService {
    snapshot_reader: Arc<dyn LedgerSnapshotReader>,
    rates: Arc<dyn ExchangeRateProvider>,
    clock: Arc<dyn Clock>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RateContext {
    base: Currency,
    quote: Currency,
    requested_date: NaiveDate,
    today: NaiveDate,
}

impl Ord for RateContext {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.base
            .code()
            .cmp(other.base.code())
            .then_with(|| self.quote.code().cmp(other.quote.code()))
            .then_with(|| self.requested_date.cmp(&other.requested_date))
            .then_with(|| self.today.cmp(&other.today))
    }
}

impl PartialOrd for RateContext {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl DebtService {
    /// Creates a service with injected dependencies.
    pub fn new(
        snapshot_reader: Arc<dyn LedgerSnapshotReader>,
        rates: Arc<dyn ExchangeRateProvider>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            snapshot_reader,
            rates,
            clock,
        }
    }
}

fn calculation_error(error: CalculationError) -> ApplicationError {
    let reason = match error {
        CalculationError::ArithmeticOverflow | CalculationError::NonIntegralResidual => {
            CalculationReason::ArithmeticOverflow
        }
        CalculationError::NonZeroSum => CalculationReason::NonZeroSum,
        CalculationError::UnsettledBalances | CalculationError::SettlementInvariant => {
            CalculationReason::SettlementInvariant
        }
    };
    ApplicationError::Calculation(reason)
}

#[async_trait]
impl DebtUseCases for DebtService {
    async fn calculate(
        &self,
        group_id: EntityId,
        mode: RateMode,
    ) -> Result<DebtResult, ApplicationError> {
        let calculated_at = self.clock.now();
        let today = calculated_at.date_naive();
        let snapshot = self.snapshot_reader.ledger_snapshot(group_id).await?;
        let group = snapshot.group;
        let spendings = snapshot.spendings;
        let mut context_set = BTreeSet::new();
        let mut contexts = Vec::new();
        for spending in &spendings {
            let requested_date = match mode {
                RateMode::Historical => spending.spent_date,
                RateMode::Current => today,
            };
            let context = RateContext {
                base: spending.currency,
                quote: group.currency,
                requested_date,
                today,
            };
            if context_set.insert(context) {
                contexts.push(context);
            }
        }
        contexts.sort_unstable();

        let mut fetched = stream::iter(contexts.iter().copied().map(|context| async move {
            let quote = self
                .rates
                .rate(
                    context.base,
                    context.quote,
                    context.requested_date,
                    context.today,
                )
                .await?;
            if quote.base != context.base
                || quote.quote != context.quote
                || quote.requested_date != context.requested_date
            {
                return Err(ApplicationError::Unavailable(
                    UnavailableReason::ExchangeRates,
                ));
            }
            Ok::<_, ApplicationError>((context, quote))
        }))
        .buffer_unordered(4);
        let mut quotes = BTreeMap::new();
        while let Some(result) = fetched.next().await {
            let (context, quote) = result?;
            quotes.insert(context, quote);
        }

        let rates = contexts
            .iter()
            .map(|context| {
                quotes
                    .get(context)
                    .cloned()
                    .ok_or(ApplicationError::Calculation(
                        CalculationReason::SettlementInvariant,
                    ))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut balances = BTreeMap::new();
        for spending in &spendings {
            let requested_date = match mode {
                RateMode::Historical => spending.spent_date,
                RateMode::Current => today,
            };
            let context = RateContext {
                base: spending.currency,
                quote: group.currency,
                requested_date,
                today,
            };
            let quote = quotes.get(&context).ok_or(ApplicationError::Calculation(
                CalculationReason::SettlementInvariant,
            ))?;
            add_converted_spending(&mut balances, spending, quote.rate)
                .map_err(calculation_error)?;
        }
        quantize_balances(&mut balances, group.currency).map_err(calculation_error)?;
        let transfers = simplify(&balances).map_err(calculation_error)?;
        Ok(DebtResult {
            currency: group.currency,
            transfers,
            balances,
            rates,
            calculated_at,
        })
    }
}
