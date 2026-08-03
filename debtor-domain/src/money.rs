//! Canonical exact monetary text conversion.

use rust_decimal::Decimal;
use thiserror::Error;

/// An error parsing a persisted monetary value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum DecimalTextError {
    /// The value is not a valid exact decimal.
    #[error("invalid decimal value")]
    Invalid,
    /// The value is valid but not in canonical persisted form.
    #[error("decimal value is not canonical")]
    NonCanonical,
}

/// Formats a decimal as canonical plain text.
///
/// Canonical values have no leading plus sign, redundant leading or trailing
/// zeroes, or exponent notation. Zero is always represented as `0`.
#[must_use]
pub fn format_decimal(value: Decimal) -> String {
    if value.is_zero() {
        String::from("0")
    } else {
        value.normalize().to_string()
    }
}

/// Parses canonical persisted monetary text without rounding.
///
/// The parser deliberately rejects values such as `01`, `1.0`, `+1`, and
/// `1e0`; persisted values must already be canonical rather than normalized
/// as a side effect of reading them.
///
/// # Errors
///
/// Returns an error when the text is invalid or not already canonical.
pub fn parse_decimal(value: &str) -> Result<Decimal, DecimalTextError> {
    let parsed = Decimal::from_str_exact(value).map_err(|_| DecimalTextError::Invalid)?;
    if format_decimal(parsed) == value {
        Ok(parsed)
    } else {
        Err(DecimalTextError::NonCanonical)
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use rust_decimal::Decimal;

    use super::{DecimalTextError, format_decimal, parse_decimal};

    #[test]
    fn formats_normalized_plain_decimal_text() {
        assert_eq!(format_decimal(Decimal::new(1_200, 2)), "12");
        assert_eq!(format_decimal(Decimal::new(-1_200, 2)), "-12");
        assert_eq!(format_decimal(Decimal::new(1_230, 2)), "12.3");
        assert_eq!(format_decimal(Decimal::new(0, 4)), "0");
    }

    #[test]
    fn parses_canonical_values_without_rounding() {
        for text in ["0", "1", "-12.3", "1234567890123456789012345678"] {
            let value = parse_decimal(text).expect("canonical decimal");
            assert_eq!(format_decimal(value), text);
        }
    }

    #[test]
    fn rejects_noncanonical_values() {
        for text in ["01", "-0", "+1", "1.0", "0.00"] {
            assert_eq!(
                parse_decimal(text),
                Err(DecimalTextError::NonCanonical),
                "{text}"
            );
        }
    }

    #[test]
    fn rejects_invalid_values() {
        for text in ["", ".", "1.2.3", "1e0", " 1", "not-a-number"] {
            assert_eq!(
                parse_decimal(text),
                Err(DecimalTextError::Invalid),
                "{text}"
            );
        }
    }
}
