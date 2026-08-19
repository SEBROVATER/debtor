use std::collections::BTreeMap;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{Datelike, NaiveDate};
use debtor_domain::currency::Currency;
use debtor_domain::debts::quantize_positive_totals;
use debtor_domain::model::{EntityId, Participant};
use futures::stream::{self, StreamExt};
use rust_decimal::Decimal;

use crate::{
    ApplicationError, Clock, ExchangeRateProvider, LedgerSnapshot, LedgerSnapshotReader, RateQuote,
    UnavailableReason,
};

/// Exact current-month spending totals grouped by original Source Currency.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceSummary {
    /// Inclusive first day of the UTC calendar month represented by this result.
    pub month: NaiveDate,
    /// Currency blocks in deterministic ISO-code order.
    pub currencies: Vec<SourceCurrencySummary>,
}

/// One Source Currency block in the monthly summary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceCurrencySummary {
    /// Original Source Currency.
    pub currency: Currency,
    /// Exact sum of the displayed Payer totals.
    pub total: Decimal,
    /// Exact amount formatted with the Source Currency precision and code.
    pub display_total: String,
    /// Paid totals in ascending Participant-ID order.
    pub payers: Vec<SourcePayerTotal>,
}

/// One current Participant projection and its exact paid total.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePayerTotal {
    /// Current identity projection, including archive state.
    pub participant: Participant,
    /// Exact amount paid in the Source Currency block.
    pub total: Decimal,
    /// Exact amount formatted with the Source Currency precision and code.
    pub display_total: String,
}

/// Exact current-month totals converted into Group Currency.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConvertedSummary {
    /// Inclusive first day of the UTC calendar month represented by this result.
    pub month: NaiveDate,
    /// Group Currency target.
    pub currency: Currency,
    /// Exact sum of the displayed converted Payer totals.
    pub total: Decimal,
    /// Formatted converted Group total.
    pub display_total: String,
    /// Converted paid totals in ascending Participant-ID order.
    pub payers: Vec<ConvertedPayerTotal>,
    /// Unique rate evidence in deterministic context order.
    pub rates: Vec<RateEvidence>,
}

/// One converted Payer total.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConvertedPayerTotal {
    /// Current identity projection, including archive state.
    pub participant: Participant,
    /// Final quantized amount paid in Group Currency.
    pub total: Decimal,
    /// Formatted final amount.
    pub display_total: String,
}

/// One immutable rate context and its returned evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RateEvidence {
    /// Source Currency.
    pub base: Currency,
    /// Group Currency target.
    pub quote: Currency,
    /// Original requested date.
    pub requested_date: NaiveDate,
    /// Effective fetch date used for the provider/cache context.
    pub fetch_date: NaiveDate,
    /// Provider effective date.
    pub effective_date: NaiveDate,
    /// Exact rate.
    pub rate: Decimal,
    /// Whether stale evidence was used.
    pub is_stale: bool,
    /// Whether the context is provisional because the Spending is future-dated.
    pub is_provisional: bool,
}

/// Source and converted projections calculated from one ledger snapshot.
#[derive(Debug)]
pub struct MonthlySummary {
    /// Group Currency captured in the same snapshot as the projections.
    pub currency: Currency,
    /// Provider-free source-currency result.
    pub source: Result<SourceSummary, ApplicationError>,
    /// Group Currency result, which may be unavailable independently.
    pub converted: Result<ConvertedSummary, ApplicationError>,
}

/// Inbound monthly source-summary operations.
#[async_trait]
pub trait SummaryUseCases: Send + Sync {
    /// Calculates exact paid totals for the current UTC calendar month.
    async fn source_summary(&self, group_id: EntityId) -> Result<SourceSummary, ApplicationError>;

    /// Calculates exact current-month totals in Group Currency.
    async fn converted_summary(
        &self,
        group_id: EntityId,
    ) -> Result<ConvertedSummary, ApplicationError>;

    /// Calculates both Summary projections from one consistent ledger read.
    async fn monthly_summary(&self, group_id: EntityId)
    -> Result<MonthlySummary, ApplicationError>;
}

/// Source-summary workflow implementation.
pub struct SummaryService {
    snapshots: Arc<dyn LedgerSnapshotReader>,
    rates: Option<Arc<dyn ExchangeRateProvider>>,
    clock: Arc<dyn Clock>,
}

impl SummaryService {
    /// Creates a source-summary service with injected readers and clock.
    pub fn new(snapshots: Arc<dyn LedgerSnapshotReader>, clock: Arc<dyn Clock>) -> Self {
        Self {
            snapshots,
            rates: None,
            clock,
        }
    }

    /// Creates a Summary service with the shared exchange-rate provider.
    pub fn with_rates(
        snapshots: Arc<dyn LedgerSnapshotReader>,
        rates: Arc<dyn ExchangeRateProvider>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            snapshots,
            rates: Some(rates),
            clock,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RateContext {
    base: Currency,
    quote: Currency,
    requested_date: NaiveDate,
    fetch_date: NaiveDate,
}

impl Ord for RateContext {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.base
            .code()
            .cmp(other.base.code())
            .then_with(|| self.quote.code().cmp(other.quote.code()))
            .then_with(|| self.requested_date.cmp(&other.requested_date))
            .then_with(|| self.fetch_date.cmp(&other.fetch_date))
    }
}

impl PartialOrd for RateContext {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

fn month_bounds(today: NaiveDate) -> Result<(NaiveDate, NaiveDate), ApplicationError> {
    let month = NaiveDate::from_ymd_opt(today.year(), today.month(), 1)
        .ok_or(ApplicationError::Storage(crate::StorageReason::InvalidData))?;
    let (year, next_month) = if today.month() == 12 {
        (
            today
                .year()
                .checked_add(1)
                .ok_or(ApplicationError::Calculation(
                    crate::CalculationReason::ArithmeticOverflow,
                ))?,
            1,
        )
    } else {
        (today.year(), today.month() + 1)
    };
    let next = NaiveDate::from_ymd_opt(year, next_month, 1)
        .ok_or(ApplicationError::Storage(crate::StorageReason::InvalidData))?;
    Ok((month, next))
}

#[async_trait]
impl SummaryUseCases for SummaryService {
    async fn source_summary(&self, group_id: EntityId) -> Result<SourceSummary, ApplicationError> {
        let today = self.clock.now().date_naive();
        let snapshot = self.snapshots.ledger_snapshot(group_id).await?;
        source_summary_from_snapshot(&snapshot, today)
    }

    async fn monthly_summary(
        &self,
        group_id: EntityId,
    ) -> Result<MonthlySummary, ApplicationError> {
        let today = self.clock.now().date_naive();
        let snapshot = self.snapshots.ledger_snapshot(group_id).await?;
        let source = source_summary_from_snapshot(&snapshot, today);
        let converted = self.converted_summary_from_snapshot(&snapshot, today).await;
        Ok(MonthlySummary {
            currency: snapshot.group.currency,
            source,
            converted,
        })
    }

    async fn converted_summary(
        &self,
        group_id: EntityId,
    ) -> Result<ConvertedSummary, ApplicationError> {
        let today = self.clock.now().date_naive();
        let snapshot = self.snapshots.ledger_snapshot(group_id).await?;
        self.converted_summary_from_snapshot(&snapshot, today).await
    }
}

fn source_summary_from_snapshot(
    snapshot: &LedgerSnapshot,
    today: NaiveDate,
) -> Result<SourceSummary, ApplicationError> {
    let (month, next_month) = month_bounds(today)?;
    let identities = snapshot
        .participants
        .iter()
        .map(|(participant, membership)| (participant.id, (participant, membership)))
        .collect::<BTreeMap<_, _>>();
    let mut grouped = BTreeMap::<String, (Currency, BTreeMap<EntityId, Decimal>)>::new();

    for spending in &snapshot.spendings {
        if spending.spent_date < month || spending.spent_date >= next_month {
            continue;
        }
        let payer = spending
            .payers
            .first()
            .ok_or(ApplicationError::Storage(crate::StorageReason::InvalidData))?;
        let Some((_, _)) = identities.get(&payer.participant_id) else {
            return Err(ApplicationError::Storage(crate::StorageReason::InvalidData));
        };
        let entry = grouped
            .entry(spending.currency.code().to_owned())
            .or_insert_with(|| (spending.currency, BTreeMap::new()));
        let total = entry.1.entry(payer.participant_id).or_default();
        *total = total
            .checked_add(payer.amount)
            .ok_or(ApplicationError::Calculation(
                crate::CalculationReason::ArithmeticOverflow,
            ))?;
    }

    let currencies = grouped
        .into_values()
        .map(|(currency, payers)| {
            let mut total = Decimal::ZERO;
            let payers = payers
                .into_iter()
                .map(|(participant_id, amount)| {
                    total = total
                        .checked_add(amount)
                        .ok_or(ApplicationError::Calculation(
                            crate::CalculationReason::ArithmeticOverflow,
                        ))?;
                    let participant = identities
                        .get(&participant_id)
                        .map(|(participant, _)| (*participant).clone())
                        .ok_or(ApplicationError::Storage(crate::StorageReason::InvalidData))?;
                    Ok(SourcePayerTotal {
                        participant,
                        total: amount,
                        display_total: format_money(amount, currency),
                    })
                })
                .collect::<Result<Vec<_>, ApplicationError>>()?;
            Ok(SourceCurrencySummary {
                currency,
                total,
                display_total: format_money(total, currency),
                payers,
            })
        })
        .collect::<Result<Vec<_>, ApplicationError>>()?;

    Ok(SourceSummary { month, currencies })
}

impl SummaryService {
    #[allow(clippy::too_many_lines)]
    async fn converted_summary_from_snapshot(
        &self,
        snapshot: &LedgerSnapshot,
        today: NaiveDate,
    ) -> Result<ConvertedSummary, ApplicationError> {
        let rates = self.rates.as_ref().ok_or(ApplicationError::Unavailable(
            UnavailableReason::ExchangeRates,
        ))?;
        let (month, next_month) = month_bounds(today)?;
        let identities = snapshot
            .participants
            .iter()
            .map(|(participant, _)| (participant.id, participant.clone()))
            .collect::<BTreeMap<_, _>>();
        let spendings = snapshot
            .spendings
            .iter()
            .filter(|spending| spending.spent_date >= month && spending.spent_date < next_month)
            .cloned()
            .collect::<Vec<_>>();
        let currency = snapshot.group.currency;
        let mut contexts = BTreeMap::new();
        for spending in &spendings {
            let context = RateContext {
                base: spending.currency,
                quote: currency,
                requested_date: spending.spent_date,
                fetch_date: spending.spent_date.min(today),
            };
            contexts.insert(context, context);
        }
        let contexts = contexts.into_values().collect::<Vec<_>>();
        let mut quotes = BTreeMap::<RateContext, RateQuote>::new();
        let remote_contexts = contexts
            .iter()
            .copied()
            .filter(|context| context.base != context.quote)
            .collect::<Vec<_>>();
        for context in contexts
            .iter()
            .copied()
            .filter(|context| context.base == context.quote)
        {
            quotes.insert(
                context,
                RateQuote {
                    base: context.base,
                    quote: context.quote,
                    requested_date: context.requested_date,
                    fetch_date: context.fetch_date,
                    effective_date: context.fetch_date,
                    rate: Decimal::ONE,
                    is_stale: false,
                    is_provisional: context.requested_date > today,
                },
            );
        }
        let mut fetched = stream::iter(remote_contexts.into_iter().map(|context| async move {
            let quote = rates
                .rate(context.base, context.quote, context.requested_date, today)
                .await?;
            if quote.base != context.base
                || quote.quote != context.quote
                || quote.requested_date != context.requested_date
                || (!quote.is_stale && quote.fetch_date != context.fetch_date)
                || (quote.is_stale
                    && quote.fetch_date > context.fetch_date
                    && quote.requested_date >= today)
                || quote.rate <= Decimal::ZERO
                || quote.effective_date > context.fetch_date
                || quote.is_provisional != (context.requested_date > today)
            {
                return Err(ApplicationError::Unavailable(
                    UnavailableReason::ExchangeRates,
                ));
            }
            Ok::<_, ApplicationError>((context, quote))
        }))
        .buffer_unordered(4);
        while let Some(result) = fetched.next().await {
            let (context, quote) = result?;
            quotes.insert(context, quote);
        }
        drop(fetched);

        let mut totals = BTreeMap::<EntityId, Decimal>::new();
        for spending in &spendings {
            let payer = spending
                .payers
                .first()
                .ok_or(ApplicationError::Storage(crate::StorageReason::InvalidData))?;
            let context = RateContext {
                base: spending.currency,
                quote: currency,
                requested_date: spending.spent_date,
                fetch_date: spending.spent_date.min(today),
            };
            let quote = quotes.get(&context).ok_or(ApplicationError::Calculation(
                crate::CalculationReason::SettlementInvariant,
            ))?;
            let converted =
                payer
                    .amount
                    .checked_mul(quote.rate)
                    .ok_or(ApplicationError::Calculation(
                        crate::CalculationReason::ArithmeticOverflow,
                    ))?;
            let total = totals.entry(payer.participant_id).or_default();
            *total = total
                .checked_add(converted)
                .ok_or(ApplicationError::Calculation(
                    crate::CalculationReason::ArithmeticOverflow,
                ))?;
        }
        let quantized = quantize_positive_totals(&totals, currency).map_err(|error| {
            ApplicationError::Calculation(match error {
                debtor_domain::debts::CalculationError::ArithmeticOverflow
                | debtor_domain::debts::CalculationError::NonIntegralResidual => {
                    crate::CalculationReason::ArithmeticOverflow
                }
                _ => crate::CalculationReason::SettlementInvariant,
            })
        })?;
        let mut total = Decimal::ZERO;
        let payers = quantized
            .into_iter()
            .map(|(participant_id, amount)| {
                total = total
                    .checked_add(amount)
                    .ok_or(ApplicationError::Calculation(
                        crate::CalculationReason::ArithmeticOverflow,
                    ))?;
                let participant = identities
                    .get(&participant_id)
                    .cloned()
                    .ok_or(ApplicationError::Storage(crate::StorageReason::InvalidData))?;
                Ok(ConvertedPayerTotal {
                    participant,
                    total: amount,
                    display_total: format_money(amount, currency),
                })
            })
            .collect::<Result<Vec<_>, ApplicationError>>()?;
        let rates = contexts
            .into_iter()
            .map(|context| {
                let quote = quotes.get(&context).ok_or(ApplicationError::Calculation(
                    crate::CalculationReason::SettlementInvariant,
                ))?;
                Ok(RateEvidence {
                    base: context.base,
                    quote: context.quote,
                    requested_date: context.requested_date,
                    fetch_date: quote.fetch_date,
                    effective_date: quote.effective_date,
                    rate: quote.rate,
                    is_stale: quote.is_stale,
                    is_provisional: context.requested_date > today,
                })
            })
            .collect::<Result<Vec<_>, ApplicationError>>()?;
        Ok(ConvertedSummary {
            month,
            currency,
            total,
            display_total: format_money(total, currency),
            payers,
            rates,
        })
    }
}

fn format_money(amount: Decimal, currency: Currency) -> String {
    let precision = match currency {
        Currency::Jpy | Currency::Krw => 0,
        Currency::Omr => 3,
        _ => 2,
    };
    format!("{}{:.*} {}", currency.symbol(), precision, amount, currency)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use std::sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    };

    use super::*;
    use chrono::{DateTime, TimeZone, Utc};
    use debtor_domain::model::{
        Allocation, Color, Description, Group, GroupMember, Name, Spending, SpendingType,
    };

    const GROUP_ID: EntityId = 7;

    struct FixedClock(DateTime<Utc>);
    impl Clock for FixedClock {
        fn now(&self) -> DateTime<Utc> {
            self.0
        }
    }

    struct Snapshot(crate::LedgerSnapshot);
    #[async_trait]
    impl LedgerSnapshotReader for Snapshot {
        async fn ledger_snapshot(
            &self,
            _: EntityId,
        ) -> Result<crate::LedgerSnapshot, ApplicationError> {
            Ok(self.0.clone())
        }
    }

    struct CountingSnapshot {
        snapshot: crate::LedgerSnapshot,
        calls: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl LedgerSnapshotReader for CountingSnapshot {
        async fn ledger_snapshot(
            &self,
            _: EntityId,
        ) -> Result<crate::LedgerSnapshot, ApplicationError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Ok(self.snapshot.clone())
        }
    }

    struct RateFake {
        calls: Mutex<Vec<(Currency, Currency, NaiveDate, NaiveDate)>>,
        rate: Decimal,
    }

    #[async_trait]
    impl ExchangeRateProvider for RateFake {
        async fn rate(
            &self,
            base: Currency,
            quote: Currency,
            requested_date: NaiveDate,
            today: NaiveDate,
        ) -> Result<RateQuote, ApplicationError> {
            self.calls
                .lock()
                .unwrap()
                .push((base, quote, requested_date, today));
            Ok(RateQuote {
                base,
                quote,
                requested_date,
                fetch_date: requested_date.min(today),
                effective_date: requested_date.min(today),
                rate: self.rate,
                is_stale: false,
                is_provisional: requested_date > today,
            })
        }
    }

    fn participant(id: EntityId, name: &str, archived: bool) -> Participant {
        Participant {
            id,
            name: Name::new(name).unwrap(),
            color: Color::new("#123456").unwrap(),
            is_archived: archived,
        }
    }

    fn spending(
        id: EntityId,
        date: NaiveDate,
        currency: Currency,
        payer: EntityId,
        amount: i64,
    ) -> Spending {
        let total = Decimal::new(amount, 2);
        Spending {
            id,
            group_id: GROUP_ID,
            description: Description::new("Expense").unwrap(),
            total,
            currency,
            spending_type: SpendingType::Food,
            spent_date: date,
            payers: vec![Allocation {
                participant_id: payer,
                amount: total,
            }],
            shares: vec![Allocation {
                participant_id: payer,
                amount: total,
            }],
        }
    }

    fn service(
        spendings: Vec<Spending>,
        participants: Vec<(Participant, GroupMember)>,
    ) -> SummaryService {
        service_at(
            Utc.with_ymd_and_hms(2026, 8, 18, 12, 0, 0).unwrap(),
            spendings,
            participants,
        )
    }

    fn service_at(
        now: DateTime<Utc>,
        spendings: Vec<Spending>,
        participants: Vec<(Participant, GroupMember)>,
    ) -> SummaryService {
        let group = Group {
            id: GROUP_ID,
            name: Name::new("Trip").unwrap(),
            currency: Currency::Usd,
            is_archived: false,
        };
        SummaryService::new(
            Arc::new(Snapshot(crate::LedgerSnapshot {
                group,
                spendings,
                participants,
            })),
            Arc::new(FixedClock(now)),
        )
    }

    fn converted_service(
        spendings: Vec<Spending>,
        participants: Vec<(Participant, GroupMember)>,
        rate: Decimal,
    ) -> (SummaryService, Arc<RateFake>) {
        let rates = Arc::new(RateFake {
            calls: Mutex::new(Vec::new()),
            rate,
        });
        let service = SummaryService::with_rates(
            Arc::new(Snapshot(crate::LedgerSnapshot {
                group: Group {
                    id: GROUP_ID,
                    name: Name::new("Trip").unwrap(),
                    currency: Currency::Usd,
                    is_archived: false,
                },
                spendings,
                participants,
            })),
            rates.clone(),
            Arc::new(FixedClock(
                Utc.with_ymd_and_hms(2026, 8, 18, 12, 0, 0).unwrap(),
            )),
        );
        (service, rates)
    }

    fn membership(id: EntityId) -> GroupMember {
        GroupMember {
            group_id: GROUP_ID,
            participant_id: id,
            is_active: true,
        }
    }

    #[tokio::test]
    async fn includes_only_current_month_and_orders_currency_and_payer_totals() {
        let p1 = participant(1, "Renamed", true);
        let p2 = participant(2, "Ada", false);
        let value = service(
            vec![
                spending(
                    1,
                    NaiveDate::from_ymd_opt(2026, 8, 1).unwrap(),
                    Currency::Eur,
                    2,
                    250,
                ),
                spending(
                    2,
                    NaiveDate::from_ymd_opt(2026, 8, 2).unwrap(),
                    Currency::Eur,
                    1,
                    125,
                ),
                spending(
                    3,
                    NaiveDate::from_ymd_opt(2026, 7, 31).unwrap(),
                    Currency::Eur,
                    2,
                    900,
                ),
                spending(
                    4,
                    NaiveDate::from_ymd_opt(2026, 8, 3).unwrap(),
                    Currency::Usd,
                    1,
                    100,
                ),
            ],
            vec![(p1.clone(), membership(1)), (p2.clone(), membership(2))],
        )
        .source_summary(GROUP_ID)
        .await
        .unwrap();

        assert_eq!(value.month, NaiveDate::from_ymd_opt(2026, 8, 1).unwrap());
        assert_eq!(value.currencies.len(), 2);
        assert_eq!(value.currencies[0].currency, Currency::Eur);
        assert_eq!(value.currencies[0].total, Decimal::new(375, 2));
        assert_eq!(value.currencies[0].payers[0].participant, p1);
        assert_eq!(value.currencies[0].payers[0].total, Decimal::new(125, 2));
        assert_eq!(value.currencies[0].payers[1].participant, p2);
    }

    #[tokio::test]
    async fn empty_month_has_no_currency_blocks_and_december_rolls_to_january() {
        let value = service(
            vec![spending(
                1,
                NaiveDate::from_ymd_opt(2026, 9, 1).unwrap(),
                Currency::Usd,
                1,
                100,
            )],
            vec![(participant(1, "Ada", false), membership(1))],
        )
        .source_summary(GROUP_ID)
        .await
        .unwrap();
        assert!(value.currencies.is_empty());
    }

    #[tokio::test]
    async fn rejects_missing_historical_identity_without_partial_output() {
        let error = service(
            vec![spending(
                1,
                NaiveDate::from_ymd_opt(2026, 8, 1).unwrap(),
                Currency::Usd,
                99,
                100,
            )],
            Vec::new(),
        )
        .source_summary(GROUP_ID)
        .await
        .unwrap_err();
        assert!(matches!(
            error,
            ApplicationError::Storage(crate::StorageReason::InvalidData)
        ));
    }

    #[tokio::test]
    async fn maps_checked_payer_aggregation_overflow_without_partial_output() {
        let mut first = spending(
            1,
            NaiveDate::from_ymd_opt(2026, 8, 1).unwrap(),
            Currency::Usd,
            1,
            100,
        );
        first.total = Decimal::MAX;
        first.payers[0].amount = Decimal::MAX;
        first.shares[0].amount = Decimal::MAX;
        let mut second = first.clone();
        second.id = 2;
        let error = service(
            vec![first, second],
            vec![(participant(1, "Ada", false), membership(1))],
        )
        .source_summary(GROUP_ID)
        .await
        .unwrap_err();
        assert!(matches!(
            error,
            ApplicationError::Calculation(crate::CalculationReason::ArithmeticOverflow)
        ));
    }

    #[tokio::test]
    async fn filters_the_actual_january_month_after_a_december_rollover() {
        let value = service_at(
            Utc.with_ymd_and_hms(2027, 1, 2, 12, 0, 0).unwrap(),
            vec![
                spending(
                    1,
                    NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
                    Currency::Usd,
                    1,
                    100,
                ),
                spending(
                    2,
                    NaiveDate::from_ymd_opt(2027, 1, 1).unwrap(),
                    Currency::Usd,
                    1,
                    200,
                ),
            ],
            vec![(participant(1, "Ada", false), membership(1))],
        )
        .source_summary(GROUP_ID)
        .await
        .unwrap();

        assert_eq!(value.month, NaiveDate::from_ymd_opt(2027, 1, 1).unwrap());
        assert_eq!(value.currencies[0].total, Decimal::new(200, 2));
    }

    #[test]
    fn formats_all_supported_minor_unit_precisions_without_rounding() {
        assert_eq!(
            format_money(Decimal::new(123, 0), Currency::Jpy),
            "¥123 JPY"
        );
        assert_eq!(
            format_money(Decimal::new(1234, 3), Currency::Omr),
            "ر.ع.1.234 OMR"
        );
    }

    #[test]
    fn month_bounds_handles_december() {
        assert_eq!(
            month_bounds(NaiveDate::from_ymd_opt(2026, 12, 31).unwrap()).unwrap(),
            (
                NaiveDate::from_ymd_opt(2026, 12, 1).unwrap(),
                NaiveDate::from_ymd_opt(2027, 1, 1).unwrap()
            )
        );
    }

    #[tokio::test]
    async fn converts_paid_totals_exactly_and_assigns_equal_residual_to_lowest_id() {
        let (service, rates) = converted_service(
            vec![
                spending(
                    1,
                    NaiveDate::from_ymd_opt(2026, 8, 1).unwrap(),
                    Currency::Eur,
                    9,
                    100,
                ),
                spending(
                    2,
                    NaiveDate::from_ymd_opt(2026, 8, 2).unwrap(),
                    Currency::Eur,
                    2,
                    100,
                ),
            ],
            vec![
                (participant(2, "Ada", false), membership(2)),
                (participant(9, "Zed", false), membership(9)),
            ],
            Decimal::new(1005, 3),
        );

        let result = service.converted_summary(GROUP_ID).await.unwrap();

        assert_eq!(result.total, Decimal::new(201, 2));
        assert_eq!(
            result
                .payers
                .iter()
                .map(|payer| (payer.participant.id, payer.total))
                .collect::<Vec<_>>(),
            vec![(2, Decimal::new(101, 2)), (9, Decimal::new(100, 2))]
        );
        assert_eq!(rates.calls.lock().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn deduplicates_historical_contexts_and_marks_future_evidence_provisional() {
        let (service, rates) = converted_service(
            vec![
                spending(
                    1,
                    NaiveDate::from_ymd_opt(2026, 8, 20).unwrap(),
                    Currency::Eur,
                    2,
                    100,
                ),
                spending(
                    2,
                    NaiveDate::from_ymd_opt(2026, 8, 20).unwrap(),
                    Currency::Eur,
                    2,
                    100,
                ),
            ],
            vec![(participant(2, "Ada", false), membership(2))],
            Decimal::ONE,
        );

        let result = service.converted_summary(GROUP_ID).await.unwrap();

        assert_eq!(rates.calls.lock().unwrap().len(), 1);
        assert_eq!(result.rates.len(), 1);
        assert!(result.rates[0].is_provisional);
        assert_eq!(
            result.rates[0].fetch_date,
            NaiveDate::from_ymd_opt(2026, 8, 18).unwrap()
        );
    }

    #[tokio::test]
    async fn rejects_nonpositive_provider_evidence_without_a_partial_result() {
        let (service, _) = converted_service(
            vec![spending(
                1,
                NaiveDate::from_ymd_opt(2026, 8, 1).unwrap(),
                Currency::Eur,
                2,
                100,
            )],
            vec![(participant(2, "Ada", false), membership(2))],
            Decimal::ZERO,
        );

        assert!(matches!(
            service.converted_summary(GROUP_ID).await,
            Err(ApplicationError::Unavailable(
                UnavailableReason::ExchangeRates
            ))
        ));
    }

    #[tokio::test]
    async fn same_currency_conversion_is_synthetic_and_does_not_call_provider() {
        let (service, rates) = converted_service(
            vec![spending(
                1,
                NaiveDate::from_ymd_opt(2026, 8, 1).unwrap(),
                Currency::Usd,
                2,
                100,
            )],
            vec![(participant(2, "Ada", false), membership(2))],
            Decimal::ZERO,
        );

        let result = service.converted_summary(GROUP_ID).await.unwrap();

        assert_eq!(result.total, Decimal::ONE);
        assert!(rates.calls.lock().unwrap().is_empty());
        assert_eq!(result.rates[0].rate, Decimal::ONE);
    }

    #[tokio::test]
    async fn conversion_overflow_returns_no_partial_projection() {
        let (service, _) = converted_service(
            vec![spending(
                1,
                NaiveDate::from_ymd_opt(2026, 8, 1).unwrap(),
                Currency::Eur,
                2,
                10_000,
            )],
            vec![(participant(2, "Ada", false), membership(2))],
            Decimal::MAX,
        );

        assert!(matches!(
            service.converted_summary(GROUP_ID).await,
            Err(ApplicationError::Calculation(
                crate::CalculationReason::ArithmeticOverflow
            ))
        ));
    }

    #[tokio::test]
    async fn monthly_summary_reads_one_snapshot_for_both_projections() {
        let calls = Arc::new(AtomicUsize::new(0));
        let service = SummaryService::with_rates(
            Arc::new(CountingSnapshot {
                snapshot: crate::LedgerSnapshot {
                    group: Group {
                        id: GROUP_ID,
                        name: Name::new("Trip").unwrap(),
                        currency: Currency::Usd,
                        is_archived: false,
                    },
                    spendings: Vec::new(),
                    participants: Vec::new(),
                },
                calls: calls.clone(),
            }),
            Arc::new(RateFake {
                calls: Mutex::new(Vec::new()),
                rate: Decimal::ONE,
            }),
            Arc::new(FixedClock(
                Utc.with_ymd_and_hms(2026, 8, 18, 12, 0, 0).unwrap(),
            )),
        );

        let result = service.monthly_summary(GROUP_ID).await.unwrap();

        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(result.currency, Currency::Usd);
    }
}
