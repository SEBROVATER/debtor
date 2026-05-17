//! Repository and service trait definitions.
//!
//! These traits are implemented by `debtor-infra` and consumed by
//! `debtor-web` handlers via `Arc<dyn Trait>`.

use crate::currency::Currency;
use rust_decimal::Decimal;
use thiserror::Error;

/// Errors that can occur when fetching exchange rates.
#[derive(Debug, Error)]
pub enum ExchangeRateError {
    /// Network or API communication failure.
    #[error("failed to fetch exchange rate from provider: {0}")]
    FetchFailed(String),
    /// The requested currency code is not supported.
    #[error("unsupported currency: {0}")]
    UnsupportedCurrency(String),
}

/// Provider of currency exchange rates.
///
/// Implementations may cache rates. If `base == quote`, implementations
/// should return `Decimal::ONE` without calling any external service.
#[allow(async_fn_in_trait)]
pub trait ExchangeRateProvider: Send + Sync {
    /// Fetch the exchange rate from `base` to `quote`.
    ///
    /// Returns how many units of `quote` equal one unit of `base`.
    async fn get_rate(&self, base: Currency, quote: Currency)
    -> Result<Decimal, ExchangeRateError>;
}
