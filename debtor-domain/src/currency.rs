//! Supported currencies as ISO 4217 codes.

use std::fmt;
use std::str::FromStr;

/// A supported currency.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Currency {
    /// US Dollar
    Usd,
    /// Euro
    Eur,
    /// Russian Ruble
    Rub,
    /// Kyrgyz Som
    Kgs,
    /// Turkish Lira
    Try,
    /// Kazakh Tenge
    Kzt,
    /// Uzbekistan Som
    Uzs,
    /// Chinese Yuan
    Cny,
    /// South Korean Won
    Krw,
    /// Japanese Yen
    Jpy,
    /// Omani Rial
    Omr,
    /// Tajikistani Somoni
    Tjs,
}

impl Currency {
    /// All supported currency variants.
    pub const ALL: [Self; 12] = [
        Self::Usd,
        Self::Eur,
        Self::Rub,
        Self::Kgs,
        Self::Try,
        Self::Kzt,
        Self::Uzs,
        Self::Cny,
        Self::Krw,
        Self::Jpy,
        Self::Omr,
        Self::Tjs,
    ];

    /// Returns the ISO 4217 code for this currency.
    pub const fn code(self) -> &'static str {
        match self {
            Self::Usd => "USD",
            Self::Eur => "EUR",
            Self::Rub => "RUB",
            Self::Kgs => "KGS",
            Self::Try => "TRY",
            Self::Kzt => "KZT",
            Self::Uzs => "UZS",
            Self::Cny => "CNY",
            Self::Krw => "KRW",
            Self::Jpy => "JPY",
            Self::Omr => "OMR",
            Self::Tjs => "TJS",
        }
    }
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.code())
    }
}

/// Error returned when parsing an unknown currency code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseCurrencyError(String);

impl fmt::Display for ParseCurrencyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown currency code: {}", self.0)
    }
}

impl std::error::Error for ParseCurrencyError {}

impl FromStr for Currency {
    type Err = ParseCurrencyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "USD" => Ok(Self::Usd),
            "EUR" => Ok(Self::Eur),
            "RUB" => Ok(Self::Rub),
            "KGS" => Ok(Self::Kgs),
            "TRY" => Ok(Self::Try),
            "KZT" => Ok(Self::Kzt),
            "UZS" => Ok(Self::Uzs),
            "CNY" => Ok(Self::Cny),
            "KRW" => Ok(Self::Krw),
            "JPY" => Ok(Self::Jpy),
            "OMR" => Ok(Self::Omr),
            "TJS" => Ok(Self::Tjs),
            _ => Err(ParseCurrencyError(s.to_owned())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_returns_iso_code() {
        assert_eq!(Currency::Usd.to_string(), "USD");
        assert_eq!(Currency::Eur.to_string(), "EUR");
        assert_eq!(Currency::Try.to_string(), "TRY");
    }

    #[test]
    fn from_str_parses_known_codes() {
        assert_eq!("USD".parse::<Currency>(), Ok(Currency::Usd));
        assert_eq!("EUR".parse::<Currency>(), Ok(Currency::Eur));
        assert_eq!("KGS".parse::<Currency>(), Ok(Currency::Kgs));
    }

    #[test]
    fn from_str_rejects_unknown_codes() {
        assert!("XYZ".parse::<Currency>().is_err());
        assert!("".parse::<Currency>().is_err());
        assert!("usd".parse::<Currency>().is_err());
    }

    #[test]
    fn all_variants_count() {
        assert_eq!(Currency::ALL.len(), 12);
    }

    #[test]
    fn roundtrip_display_from_str() {
        for c in Currency::ALL {
            assert_eq!(c.to_string().parse::<Currency>(), Ok(c));
        }
    }
}
