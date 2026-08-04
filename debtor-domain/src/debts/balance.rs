//! Balance calculation across spendings.

use std::collections::BTreeMap;

use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;

use crate::currency::Currency;
use crate::debts::CalculationError;
use crate::model::{EntityId, Spending};

/// Adds source-currency nets from a spending after multiplying by its rate.
///
/// # Errors
///
/// Returns a calculation error when source-net or converted-balance arithmetic
/// overflows the Decimal representation.
pub fn add_converted_spending(
    balances: &mut BTreeMap<EntityId, Decimal>,
    spending: &Spending,
    rate: Decimal,
) -> Result<(), CalculationError> {
    for (participant_id, net) in spending
        .source_nets()
        .map_err(|_| CalculationError::ArithmeticOverflow)?
    {
        let converted = net
            .checked_mul(rate)
            .ok_or(CalculationError::ArithmeticOverflow)?;
        let current = balances.entry(participant_id).or_default();
        *current = current
            .checked_add(converted)
            .ok_or(CalculationError::ArithmeticOverflow)?;
    }
    Ok(())
}

/// Quantizes zero-sum balances to the target currency's minor units.
///
/// Each balance is first truncated toward zero. Any residual minor units are
/// then assigned one at a time in largest-remainder order. Positive residuals
/// use descending signed fractional remainders; negative residuals use
/// ascending signed fractional remainders. Participant IDs break equal
/// remainder ties, and the resulting balances sum to exactly zero.
///
/// # Errors
///
/// Returns a calculation error for non-zero-sum input, overflow, a non-integral
/// residual, or an impossible settlement adjustment.
pub fn quantize_balances(
    balances: &mut BTreeMap<EntityId, Decimal>,
    currency: Currency,
) -> Result<(), CalculationError> {
    let unit = Decimal::new(1, currency.minor_unit_scale());
    let mut remainders = Vec::with_capacity(balances.len());
    let mut quantized = BTreeMap::new();
    let mut original_sum = Decimal::ZERO;

    for (&participant_id, &amount) in balances.iter() {
        original_sum = original_sum
            .checked_add(amount)
            .ok_or(CalculationError::ArithmeticOverflow)?;
        let truncated = amount.trunc_with_scale(currency.minor_unit_scale());
        remainders.push((
            participant_id,
            amount
                .checked_sub(truncated)
                .ok_or(CalculationError::ArithmeticOverflow)?,
        ));
        quantized.insert(participant_id, truncated);
    }
    if !original_sum.is_zero() {
        return Err(CalculationError::NonZeroSum);
    }

    let quantized_sum = quantized.values().try_fold(Decimal::ZERO, |sum, value| {
        sum.checked_add(*value)
            .ok_or(CalculationError::ArithmeticOverflow)
    })?;
    let residual = Decimal::ZERO
        .checked_sub(quantized_sum)
        .ok_or(CalculationError::ArithmeticOverflow)?;
    let residual_units_decimal = residual
        .checked_div(unit)
        .ok_or(CalculationError::ArithmeticOverflow)?;
    let residual_units = residual_units_decimal
        .to_i128()
        .ok_or(CalculationError::ArithmeticOverflow)?;
    let reconstructed_residual = unit
        .checked_mul(Decimal::from_i128_with_scale(residual_units, 0))
        .ok_or(CalculationError::ArithmeticOverflow)?;
    if reconstructed_residual != residual {
        return Err(CalculationError::NonIntegralResidual);
    }
    if residual_units == 0 || remainders.is_empty() {
        *balances = quantized;
        return Ok(());
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
    let units = usize::try_from(residual_units.unsigned_abs())
        .map_err(|_| CalculationError::ArithmeticOverflow)?;
    if units > remainders.len() {
        return Err(CalculationError::SettlementInvariant);
    }
    for index in 0..units {
        let participant_id = remainders[index % remainders.len()].0;
        if let Some(value) = quantized.get_mut(&participant_id) {
            *value = value
                .checked_add(adjustment)
                .ok_or(CalculationError::ArithmeticOverflow)?;
        }
    }

    let final_sum = quantized.values().try_fold(Decimal::ZERO, |sum, value| {
        sum.checked_add(*value)
            .ok_or(CalculationError::ArithmeticOverflow)
    })?;
    if !final_sum.is_zero() {
        return Err(CalculationError::SettlementInvariant);
    }
    *balances = quantized;
    Ok(())
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use std::collections::BTreeMap;

    use rust_decimal::Decimal;

    use super::{add_converted_spending, quantize_balances};
    use crate::currency::Currency;
    use crate::model::{Allocation, Description, Spending, SpendingType};

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

        quantize_balances(&mut balances, Currency::Usd).unwrap();

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

        quantize_balances(&mut balances, Currency::Usd).unwrap();

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

        quantize_balances(&mut balances, Currency::Usd).unwrap();

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

        quantize_balances(&mut balances, Currency::Jpy).unwrap();

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

        quantize_balances(&mut balances, Currency::Omr).unwrap();

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

    #[test]
    fn rejects_non_zero_balance_inputs() {
        let mut balances = BTreeMap::from([(1, Decimal::ONE)]);

        assert_eq!(
            quantize_balances(&mut balances, Currency::Usd),
            Err(super::CalculationError::NonZeroSum)
        );
    }

    #[test]
    fn rejects_quantization_sum_overflow() {
        let mut balances = BTreeMap::from([(1, Decimal::MAX), (2, Decimal::MAX)]);

        assert_eq!(
            quantize_balances(&mut balances, Currency::Usd),
            Err(super::CalculationError::ArithmeticOverflow)
        );
    }

    #[test]
    fn rejects_conversion_overflow() {
        let spending = Spending {
            id: 1,
            group_id: 1,
            description: Description::new("Overflow").unwrap(),
            total: Decimal::MAX,
            currency: Currency::Usd,
            spending_type: SpendingType::Other,
            spent_date: chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            payers: vec![Allocation {
                participant_id: 1,
                amount: Decimal::MAX,
            }],
            shares: vec![Allocation {
                participant_id: 2,
                amount: Decimal::ONE,
            }],
        };
        let mut balances = BTreeMap::new();

        assert_eq!(
            add_converted_spending(&mut balances, &spending, Decimal::TWO),
            Err(super::CalculationError::ArithmeticOverflow)
        );
    }
}
