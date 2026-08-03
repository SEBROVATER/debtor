//! Balance calculation across spendings.

use std::collections::BTreeMap;

use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;

use crate::currency::Currency;
use crate::model::{EntityId, Spending};

/// Adds source-currency nets from a spending after multiplying by its rate.
pub fn add_converted_spending(
    balances: &mut BTreeMap<EntityId, Decimal>,
    spending: &Spending,
    rate: Decimal,
) {
    for (participant_id, net) in spending.source_nets() {
        *balances.entry(participant_id).or_default() += net * rate;
    }
}

/// Quantizes zero-sum balances to the target currency's minor units.
///
/// Each balance is first truncated toward zero. Any residual minor units are
/// then assigned one at a time in largest-remainder order. Positive residuals
/// use descending signed fractional remainders; negative residuals use
/// ascending signed fractional remainders. Participant IDs break equal
/// remainder ties, and the resulting balances sum to exactly zero.
///
/// # Panics
///
/// This function does not panic for valid `BTreeMap` inputs.
pub fn quantize_balances(balances: &mut BTreeMap<EntityId, Decimal>, currency: Currency) {
    let unit = Decimal::new(1, currency.minor_unit_scale());
    let mut remainders = Vec::with_capacity(balances.len());
    let mut quantized = BTreeMap::new();

    for (&participant_id, &amount) in balances.iter() {
        let truncated = amount.trunc_with_scale(currency.minor_unit_scale());
        remainders.push((participant_id, amount - truncated));
        quantized.insert(participant_id, truncated);
    }

    let residual = -quantized.values().copied().sum::<Decimal>();
    let residual_units = (residual / unit).to_i128().unwrap_or_default();
    if residual_units == 0 || remainders.is_empty() {
        *balances = quantized;
        return;
    }

    if residual_units.is_positive() {
        remainders.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    } else {
        remainders.sort_by(|left, right| left.1.cmp(&right.1).then_with(|| left.0.cmp(&right.0)));
    }

    let adjustment = if residual_units.is_positive() {
        unit
    } else {
        -unit
    };
    let units = residual_units.unsigned_abs() as usize;
    for index in 0..units {
        let participant_id = remainders[index % remainders.len()].0;
        if let Some(value) = quantized.get_mut(&participant_id) {
            *value += adjustment;
        }
    }

    *balances = quantized;
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use rust_decimal::Decimal;

    use super::quantize_balances;
    use crate::currency::Currency;

    fn sum(balances: &BTreeMap<i64, Decimal>) -> Decimal {
        balances.values().copied().sum()
    }

    #[test]
    fn distributes_multiple_positive_residual_units_by_largest_remainder() {
        let mut balances = BTreeMap::from([
            (1, Decimal::new(1_009, 3)),
            (2, Decimal::new(2_009, 3)),
            (3, Decimal::new(3_009, 3)),
            (4, Decimal::new(-6_027, 3)),
        ]);

        quantize_balances(&mut balances, Currency::Usd);

        assert_eq!(
            balances,
            BTreeMap::from([
                (1, Decimal::new(101, 2)),
                (2, Decimal::new(201, 2)),
                (3, Decimal::new(300, 2)),
                (4, Decimal::new(-602, 2)),
            ])
        );
        assert_eq!(sum(&balances), Decimal::ZERO);
    }

    #[test]
    fn distributes_multiple_negative_residual_units_by_signed_remainder() {
        let mut balances = BTreeMap::from([
            (1, Decimal::new(-1_009, 3)),
            (2, Decimal::new(-2_009, 3)),
            (3, Decimal::new(-3_009, 3)),
            (4, Decimal::new(6_027, 3)),
        ]);

        quantize_balances(&mut balances, Currency::Usd);

        assert_eq!(
            balances,
            BTreeMap::from([
                (1, Decimal::new(-101, 2)),
                (2, Decimal::new(-201, 2)),
                (3, Decimal::new(-300, 2)),
                (4, Decimal::new(602, 2)),
            ])
        );
        assert_eq!(sum(&balances), Decimal::ZERO);
    }

    #[test]
    fn breaks_equal_remainder_ties_by_ascending_participant_id() {
        let mut balances = BTreeMap::from([
            (9, Decimal::new(1_005, 3)),
            (2, Decimal::new(2_005, 3)),
            (4, Decimal::new(-3_010, 3)),
        ]);

        quantize_balances(&mut balances, Currency::Usd);

        assert_eq!(balances[&2], Decimal::new(201, 2));
        assert_eq!(balances[&9], Decimal::new(100, 2));
        assert_eq!(balances[&4], Decimal::new(-301, 2));
        assert_eq!(sum(&balances), Decimal::ZERO);
    }

    #[test]
    fn uses_zero_minor_units_for_jpy() {
        let mut balances = BTreeMap::from([
            (1, Decimal::new(16, 1)),
            (2, Decimal::new(26, 1)),
            (3, Decimal::new(36, 1)),
            (4, Decimal::new(-78, 1)),
        ]);

        quantize_balances(&mut balances, Currency::Jpy);

        assert_eq!(
            balances,
            BTreeMap::from([
                (1, Decimal::new(2, 0)),
                (2, Decimal::new(2, 0)),
                (3, Decimal::new(3, 0)),
                (4, Decimal::new(-7, 0)),
            ])
        );
        assert_eq!(sum(&balances), Decimal::ZERO);
    }

    #[test]
    fn uses_three_minor_units_for_omr() {
        let mut balances = BTreeMap::from([
            (1, Decimal::new(10_006, 4)),
            (2, Decimal::new(20_006, 4)),
            (3, Decimal::new(30_006, 4)),
            (4, Decimal::new(-60_018, 4)),
        ]);

        quantize_balances(&mut balances, Currency::Omr);

        assert_eq!(
            balances,
            BTreeMap::from([
                (1, Decimal::new(1_001, 3)),
                (2, Decimal::new(2_000, 3)),
                (3, Decimal::new(3_000, 3)),
                (4, Decimal::new(-6_001, 3)),
            ])
        );
        assert_eq!(sum(&balances), Decimal::ZERO);
    }
}
