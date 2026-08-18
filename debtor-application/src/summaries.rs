use std::collections::BTreeMap;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{Datelike, NaiveDate};
use debtor_domain::currency::Currency;
use debtor_domain::model::{EntityId, Participant};
use rust_decimal::Decimal;

use crate::{ApplicationError, Clock, LedgerSnapshotReader};

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

/// Inbound monthly source-summary operations.
#[async_trait]
pub trait SummaryUseCases: Send + Sync {
    /// Calculates exact paid totals for the current UTC calendar month.
    async fn source_summary(&self, group_id: EntityId) -> Result<SourceSummary, ApplicationError>;
}

/// Source-summary workflow implementation.
pub struct SummaryService {
    snapshots: Arc<dyn LedgerSnapshotReader>,
    clock: Arc<dyn Clock>,
}

impl SummaryService {
    /// Creates a source-summary service with injected readers and clock.
    pub fn new(snapshots: Arc<dyn LedgerSnapshotReader>, clock: Arc<dyn Clock>) -> Self {
        Self { snapshots, clock }
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
        let (month, next_month) = month_bounds(today)?;
        let snapshot = self.snapshots.ledger_snapshot(group_id).await?;
        let identities = snapshot
            .participants
            .into_iter()
            .map(|(participant, membership)| (participant.id, (participant, membership)))
            .collect::<BTreeMap<_, _>>();
        let mut grouped = BTreeMap::<String, (Currency, BTreeMap<EntityId, Decimal>)>::new();

        for spending in snapshot.spendings {
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
                            .map(|(participant, _)| participant.clone())
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
}
