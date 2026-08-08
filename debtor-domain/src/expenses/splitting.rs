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

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use rust_decimal::Decimal;

    use super::equal_split;
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
}
