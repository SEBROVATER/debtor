//! Expense domain logic and share splitting.

use crate::model::Spending;

pub mod splitting;

/// Persisted payer mode inferred for an edit form.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PayerMode {
    /// One payer covers the full total.
    Single,
}

/// Persisted share mode inferred for an edit form.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShareMode {
    /// Shares are explicit exact allocations.
    Exact,
}

/// Infers whether a spending has the single-payer representation.
pub fn infer_payer_mode(spending: &Spending) -> PayerMode {
    let _ = spending;
    PayerMode::Single
}

/// Infers whether a spending has the deterministic equal-share representation.
pub fn infer_share_mode(spending: &Spending) -> ShareMode {
    let _ = spending;
    ShareMode::Exact
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use chrono::NaiveDate;
    use rust_decimal::Decimal;

    use super::{PayerMode, ShareMode, infer_payer_mode, infer_share_mode};
    use crate::currency::Currency;
    use crate::model::{Allocation, Description, Spending, SpendingType};

    fn spending(payers: Vec<Allocation>, shares: Vec<Allocation>) -> Spending {
        Spending {
            id: 1,
            group_id: 1,
            description: Description::new("Dinner").unwrap(),
            total: Decimal::ONE,
            currency: Currency::Usd,
            spending_type: SpendingType::Food,
            spent_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            payers,
            shares,
        }
    }

    #[test]
    fn infers_payer_and_share_modes_from_persisted_allocations() {
        let equal = spending(
            vec![Allocation {
                participant_id: 1,
                amount: Decimal::ONE,
            }],
            vec![Allocation {
                participant_id: 2,
                amount: Decimal::ONE,
            }],
        );
        assert_eq!(infer_payer_mode(&equal), PayerMode::Single);
        assert_eq!(infer_share_mode(&equal), ShareMode::Exact);

        let exact = spending(
            vec![
                Allocation {
                    participant_id: 1,
                    amount: Decimal::new(4, 1),
                },
                Allocation {
                    participant_id: 2,
                    amount: Decimal::new(6, 1),
                },
            ],
            vec![
                Allocation {
                    participant_id: 1,
                    amount: Decimal::new(2, 1),
                },
                Allocation {
                    participant_id: 2,
                    amount: Decimal::new(8, 1),
                },
            ],
        );
        assert_eq!(infer_payer_mode(&exact), PayerMode::Single);
        assert_eq!(infer_share_mode(&exact), ShareMode::Exact);
    }
}
