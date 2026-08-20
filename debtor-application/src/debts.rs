use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use debtor_domain::currency::Currency;
use debtor_domain::debts::{
    CalculationError, Transfer, add_converted_spending, quantize_balances, simplify,
};
use debtor_domain::model::{EntityId, Group, GroupMember, Participant, Spending};
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
    /// Cache/provider fetch date for the requested context.
    pub fetch_date: NaiveDate,
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
    /// Current Group-owned Participant identities from the same database snapshot.
    pub participants: Vec<(Participant, GroupMember)>,
}

/// Immutable ledger snapshot captured with the `SQLite` mutation generation.
#[derive(Debug, Clone)]
pub struct LedgerCapture {
    /// Complete ledger state used for the calculation.
    pub snapshot: LedgerSnapshot,
    /// Generation owned by the same gate that serializes `SQLite` commits.
    pub generation: u64,
}

/// Reads one transactionally consistent ledger snapshot.
#[async_trait]
pub trait LedgerSnapshotReader: Send + Sync {
    /// Loads the group and all complete spendings from one read snapshot.
    async fn ledger_snapshot(&self, group_id: EntityId)
    -> Result<LedgerSnapshot, ApplicationError>;
    /// Captures a snapshot and mutation generation while the ledger gate is held.
    async fn ledger_capture(&self, group_id: EntityId) -> Result<LedgerCapture, ApplicationError> {
        let _ = group_id;
        Err(ApplicationError::Storage(crate::StorageReason::Unexpected))
    }
}

/// Immutable evidence used for final participant archive admission.
#[derive(Debug, Clone)]
pub struct ArchiveAdmission {
    /// Generation captured with the ledger snapshot.
    pub generation: u64,
    /// UTC date captured before historical quote I/O.
    pub utc_date: NaiveDate,
    /// Historical quote evidence. This is revalidated but never refetched at admission.
    pub quotes: Vec<RateQuote>,
}

/// Result of the archive-specific Historical calculation.
#[derive(Debug, Clone)]
pub struct ArchiveCalculation {
    /// Immutable capture used to derive the result.
    pub capture: LedgerCapture,
    /// Final quantized balances in the group currency.
    pub balances: BTreeMap<EntityId, Decimal>,
    /// Historical quote evidence used for the calculation.
    pub quotes: Vec<RateQuote>,
    /// UTC instant captured for the calculation context.
    pub calculated_at: DateTime<Utc>,
}

/// Result of a debt calculation.
#[derive(Debug, Clone)]
pub struct DebtResult {
    /// Whether the calculated Group is read-only because it is archived.
    pub group_is_archived: bool,
    /// Group currency.
    pub currency: Currency,
    /// Group-owned identities from the calculation snapshot.
    pub participants: Vec<(Participant, GroupMember)>,
    /// Whether the snapshot contained at least one Spending.
    pub has_spendings: bool,
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

/// Historical calculation seam used only by zero-balance archive admission.
#[async_trait]
pub trait ArchiveCalculationUseCases: Send + Sync {
    /// Captures immutable ledger state and produces a complete Historical result.
    async fn calculate_archive(
        &self,
        group_id: EntityId,
    ) -> Result<ArchiveCalculation, ApplicationError>;
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

    #[allow(clippy::too_many_lines)]
    async fn calculate_snapshot(
        &self,
        group_id: EntityId,
        mode: RateMode,
        calculated_at: DateTime<Utc>,
        snapshot: LedgerSnapshot,
    ) -> Result<DebtResult, ApplicationError> {
        let today = calculated_at.date_naive();
        let group = snapshot.group;
        let spendings = snapshot.spendings;
        if group.id != group_id {
            return Err(ApplicationError::Storage(crate::StorageReason::InvalidData));
        }
        let participant_ids = snapshot
            .participants
            .iter()
            .map(|(participant, member)| {
                (
                    participant.id,
                    member.group_id == group.id && member.participant_id == participant.id,
                )
            })
            .collect::<BTreeMap<_, _>>();
        if participant_ids.len() != snapshot.participants.len()
            || participant_ids.values().any(|valid| !valid)
            || spendings.iter().any(|spending| {
                spending.group_id != group.id
                    || spending
                        .payers
                        .iter()
                        .chain(spending.shares.iter())
                        .any(|allocation| {
                            participant_ids.get(&allocation.participant_id) != Some(&true)
                        })
            })
        {
            return Err(ApplicationError::Storage(crate::StorageReason::InvalidData));
        }
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
        let mut quotes = BTreeMap::new();
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
                    fetch_date: context.requested_date.min(context.today),
                    effective_date: context.requested_date.min(context.today),
                    rate: Decimal::ONE,
                    is_stale: false,
                    is_provisional: context.requested_date > context.today,
                },
            );
        }
        let mut fetched = stream::iter(
            contexts
                .iter()
                .copied()
                .filter(|context| context.base != context.quote)
                .map(|context| async move {
                    let quote = self
                        .rates
                        .rate(
                            context.base,
                            context.quote,
                            context.requested_date,
                            context.today,
                        )
                        .await?;
                    if !quote_is_eligible(context, &quote, mode) {
                        return Err(ApplicationError::Unavailable(
                            UnavailableReason::ExchangeRates,
                        ));
                    }
                    Ok::<_, ApplicationError>((context, quote))
                }),
        )
        .buffer_unordered(4);
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
        let mut balances = snapshot
            .participants
            .iter()
            .map(|(participant, _)| (participant.id, Decimal::ZERO))
            .collect::<BTreeMap<_, _>>();
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
            group_is_archived: group.is_archived,
            currency: group.currency,
            participants: snapshot.participants,
            has_spendings: !spendings.is_empty(),
            transfers,
            balances,
            rates,
            calculated_at,
        })
    }
}

fn quote_is_eligible(context: RateContext, quote: &RateQuote, mode: RateMode) -> bool {
    quote.base == context.base
        && quote.quote == context.quote
        && quote.requested_date == context.requested_date
        && (!quote.is_stale && quote.fetch_date == context.requested_date.min(context.today)
            || quote.is_stale && quote.fetch_date <= context.requested_date.min(context.today))
        && !(mode == RateMode::Current
            && quote.is_stale
            && (quote.fetch_date >= context.today
                || quote
                    .fetch_date
                    .checked_add_signed(chrono::Duration::days(7))
                    .is_none_or(|expiry| expiry < context.today)))
        && quote.rate > Decimal::ZERO
        && quote.effective_date <= quote.fetch_date
        && quote.is_provisional == (context.requested_date > context.today)
}

/// Validates immutable Historical evidence at final archive admission without provider I/O.
pub fn archive_admission_is_eligible(admission: &ArchiveAdmission) -> bool {
    admission.quotes.iter().all(|quote| {
        !quote.is_provisional
            && quote_is_eligible(
                RateContext {
                    base: quote.base,
                    quote: quote.quote,
                    requested_date: quote.requested_date,
                    today: admission.utc_date,
                },
                quote,
                RateMode::Historical,
            )
    })
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
        let snapshot = self.snapshot_reader.ledger_snapshot(group_id).await?;
        self.calculate_snapshot(group_id, mode, calculated_at, snapshot)
            .await
    }
}

#[async_trait]
impl ArchiveCalculationUseCases for DebtService {
    async fn calculate_archive(
        &self,
        group_id: EntityId,
    ) -> Result<ArchiveCalculation, ApplicationError> {
        let calculated_at = self.clock.now();
        let capture = self.snapshot_reader.ledger_capture(group_id).await?;
        let result = self
            .calculate_snapshot(
                group_id,
                RateMode::Historical,
                calculated_at,
                capture.snapshot.clone(),
            )
            .await?;
        Ok(ArchiveCalculation {
            capture,
            balances: result.balances,
            quotes: result.rates,
            calculated_at,
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use async_trait::async_trait;
    use chrono::{DateTime, Datelike, NaiveDate, TimeZone, Utc};
    use debtor_domain::model::{
        Allocation, Color, Description, EntityId, Group, GroupMember, Name, Participant, Spending,
        SpendingType,
    };
    use rust_decimal::Decimal;

    use super::*;

    const GROUP_ID: EntityId = 10;
    const PARTICIPANT_ONE: EntityId = 1;
    const PARTICIPANT_TWO: EntityId = 2;

    fn date(day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 7, day).unwrap()
    }

    fn allocation(participant_id: EntityId, amount: i64) -> Allocation {
        Allocation {
            participant_id,
            amount: Decimal::new(amount, 2),
        }
    }

    fn group(id: EntityId) -> Group {
        Group {
            id,
            name: Name::new("Trip").unwrap(),
            currency: Currency::Usd,
            is_archived: false,
        }
    }

    fn participant(id: EntityId) -> (Participant, GroupMember) {
        (
            Participant {
                id,
                name: Name::new(format!("Participant {id}")).unwrap(),
                color: Color::new("#123456").unwrap(),
                is_archived: false,
            },
            GroupMember {
                group_id: GROUP_ID,
                participant_id: id,
                is_active: true,
            },
        )
    }

    struct FixedClock(DateTime<Utc>);
    impl Clock for FixedClock {
        fn now(&self) -> DateTime<Utc> {
            self.0
        }
    }

    struct DebtSnapshot(Vec<Spending>);
    #[async_trait]
    impl LedgerSnapshotReader for DebtSnapshot {
        async fn ledger_snapshot(
            &self,
            group_id: EntityId,
        ) -> Result<LedgerSnapshot, ApplicationError> {
            let participant_ids = self
                .0
                .iter()
                .flat_map(|spending| {
                    spending
                        .payers
                        .iter()
                        .chain(spending.shares.iter())
                        .map(|allocation| allocation.participant_id)
                })
                .collect::<BTreeSet<_>>();
            Ok(LedgerSnapshot {
                group: group(group_id),
                spendings: self.0.clone(),
                participants: participant_ids.into_iter().map(participant).collect(),
            })
        }
    }

    struct ParticipantSnapshot {
        snapshot: LedgerSnapshot,
    }

    #[async_trait]
    impl LedgerSnapshotReader for ParticipantSnapshot {
        async fn ledger_snapshot(&self, _: EntityId) -> Result<LedgerSnapshot, ApplicationError> {
            Ok(self.snapshot.clone())
        }
    }

    struct ObservedSnapshot {
        snapshot: LedgerSnapshot,
        completed: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl LedgerSnapshotReader for ObservedSnapshot {
        async fn ledger_snapshot(&self, _: EntityId) -> Result<LedgerSnapshot, ApplicationError> {
            self.completed.store(1, Ordering::SeqCst);
            Ok(self.snapshot.clone())
        }

        async fn ledger_capture(&self, _: EntityId) -> Result<LedgerCapture, ApplicationError> {
            self.completed.store(1, Ordering::SeqCst);
            Ok(LedgerCapture {
                snapshot: self.snapshot.clone(),
                generation: 41,
            })
        }
    }

    struct SnapshotAwareRate {
        completed: Arc<AtomicUsize>,
        called_before_snapshot: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl ExchangeRateProvider for SnapshotAwareRate {
        async fn rate(
            &self,
            base: Currency,
            quote: Currency,
            requested_date: NaiveDate,
            today: NaiveDate,
        ) -> Result<RateQuote, ApplicationError> {
            if self.completed.load(Ordering::SeqCst) == 0 {
                self.called_before_snapshot.store(1, Ordering::SeqCst);
            }
            Ok(RateQuote {
                base,
                quote,
                requested_date,
                fetch_date: requested_date.min(today),
                effective_date: requested_date,
                rate: Decimal::ONE,
                is_stale: false,
                is_provisional: requested_date > today,
            })
        }
    }

    struct RateFake(Mutex<Vec<(Currency, Currency, NaiveDate, NaiveDate)>>);
    #[async_trait]
    impl ExchangeRateProvider for RateFake {
        async fn rate(
            &self,
            base: Currency,
            quote: Currency,
            requested_date: NaiveDate,
            today: NaiveDate,
        ) -> Result<RateQuote, ApplicationError> {
            self.0
                .lock()
                .unwrap()
                .push((base, quote, requested_date, today));
            Ok(RateQuote {
                base,
                quote,
                requested_date,
                fetch_date: requested_date.min(today),
                effective_date: requested_date,
                rate: Decimal::ONE,
                is_stale: false,
                is_provisional: false,
            })
        }
    }

    struct BoundedRateFake {
        calls: Mutex<Vec<NaiveDate>>,
        active: AtomicUsize,
        maximum: AtomicUsize,
        reverse: bool,
    }

    #[async_trait]
    impl ExchangeRateProvider for BoundedRateFake {
        async fn rate(
            &self,
            base: Currency,
            quote: Currency,
            requested_date: NaiveDate,
            today: NaiveDate,
        ) -> Result<RateQuote, ApplicationError> {
            self.calls.lock().unwrap().push(requested_date);
            let active = self.active.fetch_add(1, Ordering::SeqCst) + 1;
            self.maximum.fetch_max(active, Ordering::SeqCst);
            let delay = if self.reverse {
                10 - requested_date.day()
            } else {
                requested_date.day()
            };
            tokio::time::sleep(Duration::from_millis(u64::from(delay))).await;
            self.active.fetch_sub(1, Ordering::SeqCst);
            Ok(RateQuote {
                base,
                quote,
                requested_date,
                fetch_date: requested_date.min(today),
                effective_date: requested_date,
                rate: Decimal::ONE,
                is_stale: false,
                is_provisional: requested_date > today,
            })
        }
    }

    struct FailingRates;
    #[async_trait]
    impl ExchangeRateProvider for FailingRates {
        async fn rate(
            &self,
            _: Currency,
            _: Currency,
            _: NaiveDate,
            _: NaiveDate,
        ) -> Result<RateQuote, ApplicationError> {
            Err(ApplicationError::Unavailable(
                UnavailableReason::ExchangeRates,
            ))
        }
    }

    struct FixedQuoteRate(RateQuote);

    #[async_trait]
    impl ExchangeRateProvider for FixedQuoteRate {
        async fn rate(
            &self,
            _: Currency,
            _: Currency,
            _: NaiveDate,
            _: NaiveDate,
        ) -> Result<RateQuote, ApplicationError> {
            Ok(self.0.clone())
        }
    }

    #[tokio::test]
    async fn debt_service_uses_fixed_clock_for_current_and_spending_dates_for_historical_rates() {
        let spending = Spending {
            id: 1,
            group_id: GROUP_ID,
            description: Description::new("Lunch").unwrap(),
            total: Decimal::new(100, 2),
            currency: Currency::Eur,
            spending_type: SpendingType::Food,
            spent_date: date(4),
            payers: vec![allocation(PARTICIPANT_ONE, 100)],
            shares: vec![allocation(PARTICIPANT_TWO, 100)],
        };
        let rates = Arc::new(RateFake(Mutex::new(Vec::new())));
        let clock_time = Utc.with_ymd_and_hms(2026, 7, 8, 12, 30, 0).unwrap();
        let service = DebtService::new(
            Arc::new(DebtSnapshot(vec![spending])),
            rates.clone(),
            Arc::new(FixedClock(clock_time)),
        );

        let historical = service
            .calculate(GROUP_ID, RateMode::Historical)
            .await
            .unwrap();
        let current = service
            .calculate(GROUP_ID, RateMode::Current)
            .await
            .unwrap();

        assert_eq!(historical.calculated_at, clock_time);
        assert_eq!(current.calculated_at, clock_time);
        assert_eq!(
            *rates.0.lock().unwrap(),
            vec![
                (Currency::Eur, Currency::Usd, date(4), date(8)),
                (Currency::Eur, Currency::Usd, date(8), date(8)),
            ]
        );
    }

    #[tokio::test]
    async fn current_mode_deduplicates_spendings_to_one_calculation_date_context() {
        let spendings = [4_u32, 5]
            .into_iter()
            .enumerate()
            .map(|(index, day)| Spending {
                id: i64::try_from(index + 1).expect("test ID"),
                group_id: GROUP_ID,
                description: Description::new("Lunch").unwrap(),
                total: Decimal::new(100, 2),
                currency: Currency::Eur,
                spending_type: SpendingType::Food,
                spent_date: date(day),
                payers: vec![allocation(PARTICIPANT_ONE, 100)],
                shares: vec![allocation(PARTICIPANT_TWO, 100)],
            })
            .collect::<Vec<_>>();
        let rates = Arc::new(RateFake(Mutex::new(Vec::new())));
        let service = DebtService::new(
            Arc::new(DebtSnapshot(spendings)),
            rates.clone(),
            Arc::new(FixedClock(
                Utc.with_ymd_and_hms(2026, 7, 8, 12, 30, 0).unwrap(),
            )),
        );

        service
            .calculate(GROUP_ID, RateMode::Current)
            .await
            .unwrap();

        assert_eq!(
            *rates.0.lock().unwrap(),
            vec![(Currency::Eur, Currency::Usd, date(8), date(8))]
        );
    }

    #[tokio::test]
    async fn debt_service_propagates_rate_provider_errors() {
        let spending = Spending {
            id: 1,
            group_id: GROUP_ID,
            description: Description::new("Lunch").unwrap(),
            total: Decimal::new(100, 2),
            currency: Currency::Eur,
            spending_type: SpendingType::Food,
            spent_date: date(4),
            payers: vec![allocation(PARTICIPANT_ONE, 100)],
            shares: vec![allocation(PARTICIPANT_TWO, 100)],
        };
        let service = DebtService::new(
            Arc::new(DebtSnapshot(vec![spending])),
            Arc::new(FailingRates),
            Arc::new(FixedClock(
                Utc.with_ymd_and_hms(2026, 7, 8, 12, 30, 0).unwrap(),
            )),
        );

        let error = service
            .calculate(GROUP_ID, RateMode::Historical)
            .await
            .unwrap_err();

        assert!(matches!(
            error,
            ApplicationError::Unavailable(UnavailableReason::ExchangeRates)
        ));
    }

    #[tokio::test]
    async fn current_mode_accepts_day_seven_stale_evidence_and_rejects_day_eight() {
        let spending = Spending {
            id: 1,
            group_id: GROUP_ID,
            description: Description::new("Lunch").unwrap(),
            total: Decimal::new(100, 2),
            currency: Currency::Eur,
            spending_type: SpendingType::Food,
            spent_date: date(4),
            payers: vec![allocation(PARTICIPANT_ONE, 100)],
            shares: vec![allocation(PARTICIPANT_TWO, 100)],
        };
        let clock = Arc::new(FixedClock(
            Utc.with_ymd_and_hms(2026, 7, 8, 12, 30, 0).unwrap(),
        ));
        let quote = |fetch_date| RateQuote {
            base: Currency::Eur,
            quote: Currency::Usd,
            requested_date: date(8),
            fetch_date,
            effective_date: fetch_date,
            rate: Decimal::ONE,
            is_stale: true,
            is_provisional: false,
        };

        let eligible = DebtService::new(
            Arc::new(DebtSnapshot(vec![spending.clone()])),
            Arc::new(FixedQuoteRate(quote(date(1)))),
            clock.clone(),
        );
        eligible
            .calculate(GROUP_ID, RateMode::Current)
            .await
            .expect("day-seven stale Current quote is eligible");

        let expired = DebtService::new(
            Arc::new(DebtSnapshot(vec![spending.clone()])),
            Arc::new(FixedQuoteRate(quote(
                chrono::NaiveDate::from_ymd_opt(2026, 6, 30).unwrap(),
            ))),
            clock.clone(),
        );
        let error = expired
            .calculate(GROUP_ID, RateMode::Current)
            .await
            .expect_err("day-eight stale Current quote is ineligible");

        assert!(matches!(
            error,
            ApplicationError::Unavailable(UnavailableReason::ExchangeRates)
        ));

        let same_day = DebtService::new(
            Arc::new(DebtSnapshot(vec![spending])),
            Arc::new(FixedQuoteRate(quote(date(8)))),
            clock,
        );
        let error = same_day
            .calculate(GROUP_ID, RateMode::Current)
            .await
            .expect_err("stale Current evidence must be prior");
        assert!(matches!(
            error,
            ApplicationError::Unavailable(UnavailableReason::ExchangeRates)
        ));
    }

    #[tokio::test]
    async fn current_mode_rejects_stale_evidence_with_an_overflowing_expiry() {
        let spending = Spending {
            id: 1,
            group_id: GROUP_ID,
            description: Description::new("Lunch").unwrap(),
            total: Decimal::new(100, 2),
            currency: Currency::Eur,
            spending_type: SpendingType::Food,
            spent_date: date(4),
            payers: vec![allocation(PARTICIPANT_ONE, 100)],
            shares: vec![allocation(PARTICIPANT_TWO, 100)],
        };
        let fetch_date = NaiveDate::MAX.pred_opt().unwrap();
        let quote = RateQuote {
            base: Currency::Eur,
            quote: Currency::Usd,
            requested_date: NaiveDate::MAX,
            fetch_date,
            effective_date: fetch_date,
            rate: Decimal::ONE,
            is_stale: true,
            is_provisional: false,
        };
        let clock =
            DateTime::from_naive_utc_and_offset(NaiveDate::MAX.and_hms_opt(0, 0, 0).unwrap(), Utc);
        let service = DebtService::new(
            Arc::new(DebtSnapshot(vec![spending])),
            Arc::new(FixedQuoteRate(quote)),
            Arc::new(FixedClock(clock)),
        );

        let error = service
            .calculate(GROUP_ID, RateMode::Current)
            .await
            .expect_err("overflowing stale expiry is ineligible");

        assert!(matches!(
            error,
            ApplicationError::Unavailable(UnavailableReason::ExchangeRates)
        ));
    }

    #[tokio::test]
    async fn debt_service_maps_calculation_failures_to_safe_reasons() {
        let spending = Spending {
            id: 1,
            group_id: GROUP_ID,
            description: Description::new("Overflow").unwrap(),
            total: Decimal::MAX,
            currency: Currency::Eur,
            spending_type: SpendingType::Food,
            spent_date: date(4),
            payers: vec![
                Allocation {
                    participant_id: PARTICIPANT_ONE,
                    amount: Decimal::MAX,
                },
                Allocation {
                    participant_id: PARTICIPANT_ONE,
                    amount: Decimal::MAX,
                },
            ],
            shares: vec![Allocation {
                participant_id: PARTICIPANT_TWO,
                amount: Decimal::ONE,
            }],
        };
        let service = DebtService::new(
            Arc::new(DebtSnapshot(vec![spending])),
            Arc::new(RateFake(Mutex::new(Vec::new()))),
            Arc::new(FixedClock(
                Utc.with_ymd_and_hms(2026, 7, 8, 12, 30, 0).unwrap(),
            )),
        );

        let error = service
            .calculate(GROUP_ID, RateMode::Historical)
            .await
            .unwrap_err();

        assert!(matches!(
            error,
            ApplicationError::Calculation(CalculationReason::ArithmeticOverflow)
        ));
    }

    #[tokio::test]
    async fn debt_service_deduplicates_rates_and_bounds_concurrency_deterministically() {
        let spendings = (1..=6)
            .map(|id| Spending {
                id,
                group_id: GROUP_ID,
                description: Description::new("Lunch").unwrap(),
                total: Decimal::ONE,
                currency: Currency::Eur,
                spending_type: SpendingType::Food,
                spent_date: date(u32::try_from(id.min(5)).expect("test day fits")),
                payers: vec![Allocation {
                    participant_id: PARTICIPANT_ONE,
                    amount: Decimal::ONE,
                }],
                shares: vec![Allocation {
                    participant_id: PARTICIPANT_TWO,
                    amount: Decimal::ONE,
                }],
            })
            .collect::<Vec<_>>();
        let clock_time = Utc.with_ymd_and_hms(2026, 7, 8, 12, 30, 0).unwrap();
        let first_rates = Arc::new(BoundedRateFake {
            calls: Mutex::new(Vec::new()),
            active: AtomicUsize::new(0),
            maximum: AtomicUsize::new(0),
            reverse: false,
        });
        let first_service = DebtService::new(
            Arc::new(DebtSnapshot(spendings.clone())),
            first_rates.clone(),
            Arc::new(FixedClock(clock_time)),
        );
        let first = first_service
            .calculate(GROUP_ID, RateMode::Historical)
            .await
            .unwrap();

        let second_rates = Arc::new(BoundedRateFake {
            calls: Mutex::new(Vec::new()),
            active: AtomicUsize::new(0),
            maximum: AtomicUsize::new(0),
            reverse: true,
        });
        let second_service = DebtService::new(
            Arc::new(DebtSnapshot(spendings)),
            second_rates.clone(),
            Arc::new(FixedClock(clock_time)),
        );
        let second = second_service
            .calculate(GROUP_ID, RateMode::Historical)
            .await
            .unwrap();

        let mut first_calls = first_rates.calls.lock().unwrap().clone();
        first_calls.sort_unstable();
        first_calls.dedup();
        assert_eq!(first_calls.len(), 5);
        assert!(first_rates.maximum.load(Ordering::SeqCst) <= 4);
        assert_eq!(
            first
                .rates
                .iter()
                .map(|rate| rate.requested_date)
                .collect::<Vec<_>>(),
            (1..=5).map(date).collect::<Vec<_>>()
        );
        assert_eq!(first.balances, second.balances);
        assert_eq!(first.transfers, second.transfers);
        assert_eq!(first.rates, second.rates);
        assert!(second_rates.maximum.load(Ordering::SeqCst) <= 4);
    }

    #[tokio::test]
    async fn debt_service_fetches_rates_only_after_snapshot_completion() {
        let spending = Spending {
            id: 1,
            group_id: GROUP_ID,
            description: Description::new("Lunch").unwrap(),
            total: Decimal::ONE,
            currency: Currency::Eur,
            spending_type: SpendingType::Food,
            spent_date: date(4),
            payers: vec![allocation(PARTICIPANT_ONE, 1)],
            shares: vec![allocation(PARTICIPANT_TWO, 1)],
        };
        let completed = Arc::new(AtomicUsize::new(0));
        let called_before_snapshot = Arc::new(AtomicUsize::new(0));
        let service = DebtService::new(
            Arc::new(ObservedSnapshot {
                snapshot: LedgerSnapshot {
                    group: group(GROUP_ID),
                    spendings: vec![spending],
                    participants: vec![participant(1), participant(2)],
                },
                completed: completed.clone(),
            }),
            Arc::new(SnapshotAwareRate {
                completed: completed.clone(),
                called_before_snapshot: called_before_snapshot.clone(),
            }),
            Arc::new(FixedClock(
                Utc.with_ymd_and_hms(2026, 7, 8, 12, 30, 0).unwrap(),
            )),
        );

        service
            .calculate(GROUP_ID, RateMode::Historical)
            .await
            .unwrap();

        assert_eq!(completed.load(Ordering::SeqCst), 1);
        assert_eq!(called_before_snapshot.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn archive_calculation_captures_generation_before_provider_work() {
        let spending = Spending {
            id: 1,
            group_id: GROUP_ID,
            description: Description::new("Lunch").unwrap(),
            total: Decimal::ONE,
            currency: Currency::Eur,
            spending_type: SpendingType::Food,
            spent_date: date(4),
            payers: vec![allocation(PARTICIPANT_ONE, 1)],
            shares: vec![allocation(PARTICIPANT_TWO, 1)],
        };
        let completed = Arc::new(AtomicUsize::new(0));
        let called_before_snapshot = Arc::new(AtomicUsize::new(0));
        let service = DebtService::new(
            Arc::new(ObservedSnapshot {
                snapshot: LedgerSnapshot {
                    group: group(GROUP_ID),
                    spendings: vec![spending],
                    participants: vec![participant(1), participant(2)],
                },
                completed: completed.clone(),
            }),
            Arc::new(SnapshotAwareRate {
                completed: completed.clone(),
                called_before_snapshot: called_before_snapshot.clone(),
            }),
            Arc::new(FixedClock(
                Utc.with_ymd_and_hms(2026, 7, 8, 12, 30, 0).unwrap(),
            )),
        );

        let result = service.calculate_archive(GROUP_ID).await.unwrap();

        assert_eq!(result.capture.generation, 41);
        assert_eq!(completed.load(Ordering::SeqCst), 1);
        assert_eq!(called_before_snapshot.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn archive_admission_rejects_ineligible_historical_quote_without_refetching() {
        let admission = ArchiveAdmission {
            generation: 1,
            utc_date: date(8),
            quotes: vec![RateQuote {
                base: Currency::Eur,
                quote: Currency::Usd,
                requested_date: date(4),
                fetch_date: date(5),
                effective_date: date(5),
                rate: Decimal::ONE,
                is_stale: false,
                is_provisional: false,
            }],
        };

        assert!(!archive_admission_is_eligible(&admission));
    }

    #[test]
    fn archive_admission_rejects_provisional_historical_evidence() {
        let admission = ArchiveAdmission {
            generation: 1,
            utc_date: date(8),
            quotes: vec![RateQuote {
                base: Currency::Eur,
                quote: Currency::Usd,
                requested_date: date(9),
                fetch_date: date(8),
                effective_date: date(8),
                rate: Decimal::ONE,
                is_stale: false,
                is_provisional: true,
            }],
        };

        assert!(!archive_admission_is_eligible(&admission));
    }

    #[tokio::test]
    async fn current_mode_includes_archived_inactive_zero_balance_participants_without_activity() {
        let participant = Participant {
            id: 3,
            name: Name::new("Inactive").unwrap(),
            color: Color::new("#123456").unwrap(),
            is_archived: true,
        };
        let service = DebtService::new(
            Arc::new(ParticipantSnapshot {
                snapshot: LedgerSnapshot {
                    group: group(GROUP_ID),
                    spendings: Vec::new(),
                    participants: vec![(
                        participant,
                        GroupMember {
                            group_id: GROUP_ID,
                            participant_id: 3,
                            is_active: false,
                        },
                    )],
                },
            }),
            Arc::new(RateFake(Mutex::new(Vec::new()))),
            Arc::new(FixedClock(
                Utc.with_ymd_and_hms(2026, 7, 8, 12, 30, 0).unwrap(),
            )),
        );

        let result = service
            .calculate(GROUP_ID, RateMode::Current)
            .await
            .unwrap();

        assert_eq!(result.balances, BTreeMap::from([(3, Decimal::ZERO)]));
    }

    #[tokio::test]
    async fn debt_service_synthesizes_same_currency_rates_without_provider_access() {
        let spending = Spending {
            id: 1,
            group_id: GROUP_ID,
            description: Description::new("Lunch").unwrap(),
            total: Decimal::new(100, 2),
            currency: Currency::Usd,
            spending_type: SpendingType::Food,
            spent_date: date(4),
            payers: vec![allocation(PARTICIPANT_ONE, 100)],
            shares: vec![allocation(PARTICIPANT_TWO, 100)],
        };
        let rates = Arc::new(RateFake(Mutex::new(Vec::new())));
        let service = DebtService::new(
            Arc::new(DebtSnapshot(vec![spending])),
            rates.clone(),
            Arc::new(FixedClock(
                Utc.with_ymd_and_hms(2026, 7, 8, 12, 30, 0).unwrap(),
            )),
        );

        let result = service
            .calculate(GROUP_ID, RateMode::Historical)
            .await
            .unwrap();

        assert!(rates.0.lock().unwrap().is_empty());
        assert_eq!(result.rates.len(), 1);
        assert_eq!(result.rates[0].rate, Decimal::ONE);
    }

    #[tokio::test]
    async fn debt_service_derives_the_same_ordered_transfer_in_both_rate_modes() {
        let spending = Spending {
            id: 1,
            group_id: GROUP_ID,
            description: Description::new("Dinner").unwrap(),
            total: Decimal::new(100, 2),
            currency: Currency::Usd,
            spending_type: SpendingType::Food,
            spent_date: date(4),
            payers: vec![allocation(PARTICIPANT_ONE, 100)],
            shares: vec![allocation(PARTICIPANT_TWO, 100)],
        };
        let service = DebtService::new(
            Arc::new(DebtSnapshot(vec![spending])),
            Arc::new(RateFake(Mutex::new(Vec::new()))),
            Arc::new(FixedClock(
                Utc.with_ymd_and_hms(2026, 7, 8, 12, 30, 0).unwrap(),
            )),
        );

        let historical = service
            .calculate(GROUP_ID, RateMode::Historical)
            .await
            .unwrap();
        let current = service
            .calculate(GROUP_ID, RateMode::Current)
            .await
            .unwrap();
        let expected = vec![Transfer {
            from_participant_id: PARTICIPANT_TWO,
            to_participant_id: PARTICIPANT_ONE,
            amount: Decimal::new(100, 2),
        }];

        assert_eq!(historical.transfers, expected);
        assert_eq!(current.transfers, expected);
    }

    #[test]
    fn settlement_invariant_maps_to_the_safe_calculation_reason() {
        assert!(matches!(
            calculation_error(CalculationError::SettlementInvariant),
            ApplicationError::Calculation(CalculationReason::SettlementInvariant)
        ));
    }
}
