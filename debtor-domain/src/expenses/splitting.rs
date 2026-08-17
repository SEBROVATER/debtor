//! Share splitting logic.

use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;

use crate::currency::Currency;
use crate::model::{Allocation, EntityId, ValidationError};

/// Splits a total equally among unique participants.
///
/// Residual minor units go to participants in ascending identifier order.
///
/// # Errors
///
/// Returns an error for invalid totals, empty inputs, or duplicate participants.
pub fn equal_split(
    total: Decimal,
    currency: Currency,
    participant_ids: &[EntityId],
) -> Result<Vec<Allocation>, ValidationError> {
    if participant_ids.is_empty() {
        return Err(ValidationError::EmptyAllocations { field: "share" });
    }
    let mut ids = participant_ids.to_vec();
    ids.sort_unstable();
    for pair in ids.windows(2) {
        if pair[0] == pair[1] {
            return Err(ValidationError::DuplicateParticipant {
                participant_id: pair[0],
            });
        }
    }
    if ids.iter().any(|id| *id <= 0) {
        return Err(ValidationError::InvalidParticipantId);
    }
    crate::model::validate_amount(total, currency, "total")?;
    let unit = Decimal::new(1, currency.minor_unit_scale());
    let units = (total / unit).trunc();
    if units < Decimal::from(ids.len() as u64) {
        return Err(ValidationError::InsufficientMinorUnits {
            recipients: ids.len(),
        });
    }
    let count = Decimal::from(ids.len() as u64);
    let base_units = (units / count).trunc();
    let remainder = units - base_units * count;
    let remainder = remainder
        .to_usize()
        .ok_or(ValidationError::ArithmeticOverflow)?;
    Ok(ids
        .into_iter()
        .enumerate()
        .map(|(index, participant_id)| Allocation {
            participant_id,
            amount: (base_units + Decimal::from(u8::from(index < remainder))) * unit,
        })
        .collect())
}

/// Splits a total according to positive proportional weights.
///
/// Weights are normalized to integer ratios at the maximum submitted scale.
/// Residual minor units are assigned by descending remainder and ascending
/// Participant ID, making the result independent of input or task ordering.
///
/// # Errors
///
/// Returns an error when the total, weights, participants, or checked
/// arithmetic is invalid.
pub fn proportional_split(
    total: Decimal,
    currency: Currency,
    weights: &[(EntityId, Decimal)],
) -> Result<Vec<Allocation>, ValidationError> {
    if weights.is_empty() {
        return Err(ValidationError::EmptyAllocations { field: "share" });
    }
    crate::model::validate_amount(total, currency, "total")?;

    let mut ordered = weights.to_vec();
    ordered.sort_unstable_by_key(|(participant_id, _)| *participant_id);
    for pair in ordered.windows(2) {
        if pair[0].0 == pair[1].0 {
            return Err(ValidationError::DuplicateParticipant {
                participant_id: pair[0].0,
            });
        }
    }
    if ordered
        .iter()
        .any(|(participant_id, _)| *participant_id <= 0)
    {
        return Err(ValidationError::InvalidParticipantId);
    }

    let maximum_scale = ordered
        .iter()
        .map(|(_, weight)| weight.scale())
        .max()
        .unwrap_or_default();
    if maximum_scale > 6 {
        return Err(ValidationError::WeightPrecision);
    }
    let mut integer_weights = Vec::with_capacity(ordered.len());
    let mut total_weight = 0_i128;
    for (participant_id, weight) in &ordered {
        if *weight <= Decimal::ZERO {
            return Err(ValidationError::NonPositive { field: "weight" });
        }
        if *weight > Decimal::new(1_000_000, 0) {
            return Err(ValidationError::WeightTooLarge);
        }
        let integer_weight = weight
            .mantissa()
            .checked_mul(power_of_ten(maximum_scale - weight.scale())?)
            .ok_or(ValidationError::ArithmeticOverflow)?;
        if integer_weight <= 0 {
            return Err(ValidationError::NonPositive { field: "weight" });
        }
        total_weight = total_weight
            .checked_add(integer_weight)
            .ok_or(ValidationError::ArithmeticOverflow)?;
        integer_weights.push((*participant_id, integer_weight));
    }

    let total_minor_units = total
        .checked_mul(scale_factor_decimal(currency)?)
        .ok_or(ValidationError::ArithmeticOverflow)?
        .to_i128()
        .ok_or(ValidationError::ArithmeticOverflow)?;
    let mut allocations = Vec::with_capacity(integer_weights.len());
    let mut allocated_units = 0_i128;
    for (participant_id, integer_weight) in integer_weights {
        let numerator = total_minor_units
            .checked_mul(integer_weight)
            .ok_or(ValidationError::ArithmeticOverflow)?;
        let units = numerator / total_weight;
        let remainder = numerator % total_weight;
        allocated_units = allocated_units
            .checked_add(units)
            .ok_or(ValidationError::ArithmeticOverflow)?;
        allocations.push((participant_id, units, remainder));
    }

    let residual = total_minor_units
        .checked_sub(allocated_units)
        .ok_or(ValidationError::ArithmeticOverflow)?;
    let residual = usize::try_from(residual).map_err(|_| ValidationError::ArithmeticOverflow)?;
    allocations
        .sort_unstable_by(|left, right| right.2.cmp(&left.2).then_with(|| left.0.cmp(&right.0)));
    for (index, allocation) in allocations.iter_mut().enumerate() {
        if index < residual {
            allocation.1 = allocation
                .1
                .checked_add(1)
                .ok_or(ValidationError::ArithmeticOverflow)?;
        }
    }
    allocations.sort_unstable_by_key(|(participant_id, _, _)| *participant_id);

    allocations
        .into_iter()
        .map(|(participant_id, units, _)| {
            if units <= 0 {
                return Err(ValidationError::InsufficientMinorUnits {
                    recipients: weights.len(),
                });
            }
            Ok(Allocation {
                participant_id,
                amount: Decimal::from_i128_with_scale(units, currency.minor_unit_scale()),
            })
        })
        .collect()
}

fn power_of_ten(scale: u32) -> Result<i128, ValidationError> {
    (0..scale).try_fold(1_i128, |value, _| {
        value
            .checked_mul(10)
            .ok_or(ValidationError::ArithmeticOverflow)
    })
}

fn scale_factor_decimal(currency: Currency) -> Result<Decimal, ValidationError> {
    Ok(Decimal::from_i128_with_scale(
        power_of_ten(currency.minor_unit_scale())?,
        0,
    ))
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use rust_decimal::Decimal;

    use super::{equal_split, proportional_split};
    use crate::currency::Currency;
    use crate::model::ValidationError;

    #[test]
    fn distributes_residual_units_by_participant_id() {
        let split =
            equal_split(Decimal::new(10, 2), Currency::Usd, &[9, 2, 4]).expect("valid split");
        assert_eq!(split[0].participant_id, 2);
        assert_eq!(split[0].amount, Decimal::new(4, 2));
        assert_eq!(split[1].amount, Decimal::new(3, 2));
        assert_eq!(split[2].amount, Decimal::new(3, 2));
    }

    #[test]
    fn rejects_a_total_with_fewer_minor_units_than_recipients() {
        let error = equal_split(Decimal::new(2, 2), Currency::Usd, &[1, 2, 3])
            .expect_err("a zero-valued share must be rejected");

        assert_eq!(
            error,
            ValidationError::InsufficientMinorUnits { recipients: 3 }
        );
    }

    #[test]
    fn applies_the_minimum_unit_rule_at_currency_scale() {
        let error = equal_split(Decimal::new(2, 0), Currency::Jpy, &[1, 2, 3])
            .expect_err("a zero-valued yen share must be rejected");

        assert_eq!(
            error,
            ValidationError::InsufficientMinorUnits { recipients: 3 }
        );
    }

    #[test]
    fn proportional_split_assigns_remainders_by_descending_remainder_then_id() {
        let split = proportional_split(
            Decimal::new(1_00, 2),
            Currency::Usd,
            &[
                (9, Decimal::new(1, 0)),
                (2, Decimal::new(1, 0)),
                (4, Decimal::new(1, 0)),
            ],
        )
        .expect("valid proportional split");

        assert_eq!(split[0].participant_id, 2);
        assert_eq!(split[0].amount, Decimal::new(34, 2));
        assert_eq!(split[1].participant_id, 4);
        assert_eq!(split[1].amount, Decimal::new(33, 2));
        assert_eq!(split[2].participant_id, 9);
        assert_eq!(split[2].amount, Decimal::new(33, 2));
    }

    #[test]
    fn proportional_split_normalizes_weights_to_the_maximum_scale() {
        let split = proportional_split(
            Decimal::new(10_000, 2),
            Currency::Usd,
            &[(1, Decimal::new(1, 1)), (2, Decimal::new(25, 2))],
        )
        .expect("valid proportional split");

        assert_eq!(split[0].amount, Decimal::new(2857, 2));
        assert_eq!(split[1].amount, Decimal::new(7143, 2));
    }

    #[test]
    fn proportional_split_rejects_zero_results_and_invalid_weights() {
        assert_eq!(
            proportional_split(
                Decimal::new(2, 2),
                Currency::Usd,
                &[(1, Decimal::ONE), (2, Decimal::ONE), (3, Decimal::ONE)],
            ),
            Err(ValidationError::InsufficientMinorUnits { recipients: 3 })
        );
        assert_eq!(
            proportional_split(Decimal::ONE, Currency::Usd, &[(1, Decimal::ZERO)]),
            Err(ValidationError::NonPositive { field: "weight" })
        );
        assert_eq!(
            proportional_split(
                Decimal::ONE,
                Currency::Usd,
                &[(1, Decimal::new(1_000_001, 0))],
            ),
            Err(ValidationError::WeightTooLarge)
        );
        assert_eq!(
            proportional_split(
                Decimal::ONE,
                Currency::Usd,
                &[(1, Decimal::new(10_000_001, 7))],
            ),
            Err(ValidationError::WeightPrecision)
        );
    }
}
