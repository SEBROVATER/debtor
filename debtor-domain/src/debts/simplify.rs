//! Deterministic settlement transfer generation.

use std::collections::BTreeMap;

use rust_decimal::Decimal;

use crate::model::EntityId;

/// A positive transfer from a debtor to a creditor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transfer {
    /// Participant who pays.
    pub from_participant_id: EntityId,
    /// Participant who receives payment.
    pub to_participant_id: EntityId,
    /// Positive target-currency amount.
    pub amount: Decimal,
}

/// Produces a deterministic, complete settlement for zero-sum balances.
pub fn simplify(balances: &BTreeMap<EntityId, Decimal>) -> Vec<Transfer> {
    let mut debtors: Vec<_> = balances
        .iter()
        .filter(|(_, amount)| **amount < Decimal::ZERO)
        .map(|(id, amount)| (*id, -*amount))
        .collect();
    let mut creditors: Vec<_> = balances
        .iter()
        .filter(|(_, amount)| **amount > Decimal::ZERO)
        .map(|(id, amount)| (*id, *amount))
        .collect();
    debtors.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    creditors.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    let (mut debtor, mut creditor) = (0, 0);
    let mut transfers = Vec::new();
    while debtor < debtors.len() && creditor < creditors.len() {
        let amount = debtors[debtor].1.min(creditors[creditor].1);
        transfers.push(Transfer {
            from_participant_id: debtors[debtor].0,
            to_participant_id: creditors[creditor].0,
            amount,
        });
        debtors[debtor].1 -= amount;
        creditors[creditor].1 -= amount;
        if debtors[debtor].1.is_zero() {
            debtor += 1;
        }
        if creditors[creditor].1.is_zero() {
            creditor += 1;
        }
    }
    transfers
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use rust_decimal::Decimal;

    use super::simplify;

    #[test]
    fn creates_deterministic_positive_transfers() {
        let balances = BTreeMap::from([
            (1, Decimal::new(-5, 0)),
            (2, Decimal::new(-3, 0)),
            (3, Decimal::new(8, 0)),
        ]);
        let transfers = simplify(&balances);
        assert_eq!(transfers.len(), 2);
        assert_eq!(transfers[0].from_participant_id, 1);
        assert_eq!(transfers[0].to_participant_id, 3);
        assert!(
            transfers
                .iter()
                .all(|transfer| transfer.amount > Decimal::ZERO)
        );
    }
}
