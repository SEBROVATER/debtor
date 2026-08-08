//! Application use cases and mockable ports for debtor.

mod authentication;
mod debts;
mod errors;
mod groups;
mod participants;
mod readiness;
mod spendings;

pub use authentication::*;
pub use debts::*;
pub use errors::*;
pub use groups::*;
pub use participants::*;
pub use readiness::*;
pub use spendings::*;

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use std::collections::BTreeSet;
    use std::sync::Arc;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    use async_trait::async_trait;
    use chrono::{DateTime, Datelike, NaiveDate, TimeZone, Utc};
    use debtor_domain::currency::Currency;
    use debtor_domain::model::{
        Allocation, Description, EntityId, Group, Name, Spending, SpendingType, ValidationError,
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

    struct SpendingFake {
        listed_groups: Mutex<Vec<EntityId>>,
        read_requests: Mutex<Vec<(EntityId, EntityId)>>,
        created: Mutex<Vec<Spending>>,
        updated: Mutex<Vec<Spending>>,
        fail_update: bool,
    }

    #[async_trait]
    impl SpendingReader for SpendingFake {
        async fn spendings(&self, group_id: EntityId) -> Result<Vec<Spending>, ApplicationError> {
            self.listed_groups.lock().unwrap().push(group_id);
            Ok(Vec::new())
        }
        async fn spending(
            &self,
            group_id: EntityId,
            spending_id: EntityId,
        ) -> Result<Spending, ApplicationError> {
            self.read_requests
                .lock()
                .unwrap()
                .push((group_id, spending_id));
            Err(ApplicationError::NotFound)
        }

        async fn spending_page(
            &self,
            _: EntityId,
            _: Option<SpendingCursor>,
        ) -> Result<SpendingPage, ApplicationError> {
            Ok(SpendingPage {
                items: Vec::new(),
                older: None,
                newer: None,
            })
        }
    }

    #[async_trait]
    impl SpendingRepository for SpendingFake {
        async fn create_spending(&self, spending: Spending) -> Result<Spending, ApplicationError> {
            self.created.lock().unwrap().push(spending.clone());
            Ok(spending)
        }
        async fn update_spending(&self, spending: Spending) -> Result<Spending, ApplicationError> {
            if self.fail_update {
                return Err(ApplicationError::Conflict);
            }
            self.updated.lock().unwrap().push(spending.clone());
            Ok(spending)
        }
        async fn delete_spending(&self, _: EntityId, _: EntityId) -> Result<(), ApplicationError> {
            Ok(())
        }
    }

    #[async_trait]
    impl SpendingEligibilityReader for SpendingFake {
        async fn eligible_participant_ids(
            &self,
            _: EntityId,
        ) -> Result<BTreeSet<EntityId>, ApplicationError> {
            Ok([PARTICIPANT_ONE, PARTICIPANT_TWO].into_iter().collect())
        }
    }

    fn equal_input() -> SpendingInput {
        SpendingInput {
            group_id: GROUP_ID,
            description: "  Dinner  ".into(),
            total: "10.01".into(),
            currency: "USD".into(),
            spending_type: "food".into(),
            spent_date: date(5).to_string(),
            payers: PayerInput::Single(PARTICIPANT_ONE),
            shares: ShareInput::Equal(vec![PARTICIPANT_TWO, PARTICIPANT_ONE]),
        }
    }

    fn exact_input() -> SpendingInput {
        SpendingInput {
            group_id: GROUP_ID,
            description: "Taxi".into(),
            total: "10.00".into(),
            currency: "USD".into(),
            spending_type: "transport".into(),
            spent_date: date(6).to_string(),
            payers: PayerInput::Exact(vec![
                (PARTICIPANT_ONE, "4.00".into()),
                (PARTICIPANT_TWO, "6.00".into()),
            ]),
            shares: ShareInput::Exact(vec![
                (PARTICIPANT_ONE, "4.00".into()),
                (PARTICIPANT_TWO, "6.00".into()),
            ]),
        }
    }

    #[tokio::test]
    async fn spending_service_parses_raw_input_for_create_update_and_scopes_reads() {
        let fake = Arc::new(SpendingFake {
            listed_groups: Mutex::new(Vec::new()),
            read_requests: Mutex::new(Vec::new()),
            created: Mutex::new(Vec::new()),
            updated: Mutex::new(Vec::new()),
            fail_update: false,
        });
        let service = SpendingService::new(fake.clone(), fake.clone(), fake.clone());

        service.create_input(equal_input()).await.unwrap();
        service.create_input(exact_input()).await.unwrap();
        service.update_input(99, equal_input()).await.unwrap();
        service.update_input(100, exact_input()).await.unwrap();
        assert!(matches!(
            service.spending(GROUP_ID, 77).await,
            Err(ApplicationError::NotFound)
        ));
        service.list_spendings(GROUP_ID).await.unwrap();

        let created = fake.created.lock().unwrap();
        assert_eq!(created[0].description.as_str(), "Dinner");
        assert_eq!(
            created[0].shares,
            vec![
                allocation(PARTICIPANT_ONE, 501),
                allocation(PARTICIPANT_TWO, 500)
            ]
        );
        assert_eq!(
            created[1].shares,
            vec![
                allocation(PARTICIPANT_ONE, 400),
                allocation(PARTICIPANT_TWO, 600)
            ]
        );
        let updated = fake.updated.lock().unwrap();
        assert_eq!(updated[0].id, 99);
        assert_eq!(updated[1].id, 100);
        assert_eq!(*fake.read_requests.lock().unwrap(), vec![(GROUP_ID, 77)]);
        assert_eq!(*fake.listed_groups.lock().unwrap(), vec![GROUP_ID]);
    }

    #[tokio::test]
    async fn spending_input_covers_all_payer_and_share_modes_and_validates_selection() {
        let fake = Arc::new(SpendingFake {
            listed_groups: Mutex::new(Vec::new()),
            read_requests: Mutex::new(Vec::new()),
            created: Mutex::new(Vec::new()),
            updated: Mutex::new(Vec::new()),
            fail_update: false,
        });
        let service = SpendingService::new(fake.clone(), fake.clone(), fake);

        let mut equal_multiple_payers = equal_input();
        equal_multiple_payers.payers = PayerInput::Exact(vec![(PARTICIPANT_ONE, "10.01".into())]);
        service.create_input(equal_multiple_payers).await.unwrap();

        let mut exact_single_payer = exact_input();
        exact_single_payer.payers = PayerInput::Single(PARTICIPANT_ONE);
        service.create_input(exact_single_payer).await.unwrap();

        let mut duplicate_payers = equal_input();
        duplicate_payers.payers = PayerInput::Exact(vec![
            (PARTICIPANT_ONE, "5.00".into()),
            (PARTICIPANT_ONE, "5.01".into()),
        ]);
        assert!(matches!(
            service.create_input(duplicate_payers).await,
            Err(ApplicationError::Validation(
                ValidationError::DuplicateParticipant { .. }
            ))
        ));

        let mut empty_shares = exact_input();
        empty_shares.shares = ShareInput::Exact(Vec::new());
        assert!(matches!(
            service.create_input(empty_shares).await,
            Err(ApplicationError::Validation(
                ValidationError::EmptyAllocations { field: "share" }
            ))
        ));

        let mut invalid_id = equal_input();
        invalid_id.shares = ShareInput::Equal(vec![-1]);
        assert!(matches!(
            service.create_input(invalid_id).await,
            Err(ApplicationError::Validation(
                ValidationError::InvalidParticipantId
            ))
        ));
    }

    #[tokio::test]
    async fn spending_service_propagates_repository_errors() {
        let fake = Arc::new(SpendingFake {
            listed_groups: Mutex::new(Vec::new()),
            read_requests: Mutex::new(Vec::new()),
            created: Mutex::new(Vec::new()),
            updated: Mutex::new(Vec::new()),
            fail_update: true,
        });
        let error = SpendingService::new(fake.clone(), fake.clone(), fake)
            .update_input(9, exact_input())
            .await
            .unwrap_err();
        assert!(matches!(error, ApplicationError::Conflict));
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
            Ok(LedgerSnapshot {
                group: group(group_id),
                spendings: self.0.clone(),
            })
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
}
