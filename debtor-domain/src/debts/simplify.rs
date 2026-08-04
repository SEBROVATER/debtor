//! Deterministic settlement transfer generation.

use std::collections::BTreeMap;

use rust_decimal::Decimal;

use crate::debts::CalculationError;
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
///
/// # Errors
///
/// Returns a calculation error when balances are not zero-sum, checked decimal
/// arithmetic overflows, or the generated transfers cannot settle every balance.
pub fn simplify(balances: &BTreeMap<EntityId, Decimal>) -> Result<Vec<Transfer>, CalculationError> {
    let total = balances.values().try_fold(Decimal::ZERO, |sum, amount| {
        sum.checked_add(*amount)
            .ok_or(CalculationError::ArithmeticOverflow)
    })?;
    if !total.is_zero() {
        return Err(CalculationError::NonZeroSum);
    }
    let mut debtors: Vec<_> = balances
        .iter()
        .filter(|(_, amount)| **amount < Decimal::ZERO)
        .map(|(id, amount)| {
            Decimal::ZERO
                .checked_sub(*amount)
                .map(|value| (*id, value))
                .ok_or(CalculationError::ArithmeticOverflow)
        })
        .collect::<Result<_, _>>()?;
    let mut creditors: Vec<_> = balances
        .iter()
        .filter(|(_, amount)| **amount > Decimal::ZERO)
        .map(|(id, amount)| (*id, *amount))
        .collect();
    debtors.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    creditors.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    let (mut debtor, mut creditor) = (0, 0);
    let mut transfers = Vec::new();
    let participant_count = debtors
        .len()
        .checked_add(creditors.len())
        .ok_or(CalculationError::ArithmeticOverflow)?;
    while debtor < debtors.len() && creditor < creditors.len() {
        let amount = debtors[debtor].1.min(creditors[creditor].1);
        if amount <= Decimal::ZERO || transfers.len() >= participant_count.saturating_sub(1) {
            return Err(CalculationError::SettlementInvariant);
        }
        transfers.push(Transfer {
            from_participant_id: debtors[debtor].0,
            to_participant_id: creditors[creditor].0,
            amount,
        });
        debtors[debtor].1 = debtors[debtor]
            .1
            .checked_sub(amount)
            .ok_or(CalculationError::ArithmeticOverflow)?;
        creditors[creditor].1 = creditors[creditor]
            .1
            .checked_sub(amount)
            .ok_or(CalculationError::ArithmeticOverflow)?;
        if debtors[debtor].1.is_zero() {
            debtor += 1;
        }
        if creditors[creditor].1.is_zero() {
            creditor += 1;
        }
    }
    if debtors.iter().any(|(_, amount)| !amount.is_zero())
        || creditors.iter().any(|(_, amount)| !amount.is_zero())
    {
        return Err(CalculationError::UnsettledBalances);
    }
    if transfers
        .iter()
        .any(|transfer| transfer.amount <= Decimal::ZERO)
    {
        return Err(CalculationError::SettlementInvariant);
    }
    Ok(transfers)
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use rust_decimal::Decimal;

    use super::simplify;

    #[test]
    fn creates_deterministic_positive_transfers() {
        let balances = BTreeMap::from([
            (1, Decimal::new(-5, 0)),
            (2, Decimal::new(-3, 0)),
            (3, Decimal::new(8, 0)),
        ]);
        let transfers = simplify(&balances).unwrap();
        assert_eq!(transfers.len(), 2);
        assert_eq!(transfers[0].from_participant_id, 1);
        assert_eq!(transfers[0].to_participant_id, 3);
        assert!(
            transfers
                .iter()
                .all(|transfer| transfer.amount > Decimal::ZERO)
        );
    }

    #[test]
    fn rejects_non_zero_sum_balances() {
        let balances = BTreeMap::from([(1, Decimal::new(-5, 0)), (2, Decimal::new(3, 0))]);

        assert_eq!(
            simplify(&balances),
            Err(super::CalculationError::NonZeroSum)
        );
    }

    #[test]
    fn settles_every_balance_with_at_most_n_minus_one_transfers() {
        let balances = BTreeMap::from([
            (1, Decimal::new(-7, 0)),
            (2, Decimal::new(-2, 0)),
            (3, Decimal::new(4, 0)),
            (4, Decimal::new(5, 0)),
        ]);
        let transfers = simplify(&balances).unwrap();

        assert!(transfers.len() < balances.len());
        assert!(
            transfers
                .iter()
                .all(|transfer| transfer.from_participant_id != transfer.to_participant_id)
        );
        let pairs = transfers
            .iter()
            .map(|transfer| (transfer.from_participant_id, transfer.to_participant_id))
            .collect::<BTreeSet<_>>();
        assert_eq!(pairs.len(), transfers.len());
        let mut net = BTreeMap::new();
        for transfer in transfers {
            *net.entry(transfer.from_participant_id)
                .or_insert(Decimal::ZERO) += transfer.amount;
            *net.entry(transfer.to_participant_id)
                .or_insert(Decimal::ZERO) -= transfer.amount;
        }
        let expected = balances
            .iter()
            .map(|(participant_id, balance)| (*participant_id, -*balance))
            .collect();
        assert_eq!(net, expected);
    }
}
