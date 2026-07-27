use std::collections::HashMap;
use std::sync::RwLock;

use async_trait::async_trait;
use chrono::NaiveDate;
use debtor_application::{ApplicationError, ExchangeRateProvider, RateQuote};
use debtor_domain::currency::Currency;
use rust_decimal::Decimal;
use serde::Deserialize;

const DEFAULT_BASE_URL: &str = "https://api.frankfurter.dev/v2";

#[derive(Debug, Deserialize)]
struct RateResponse {
    date: NaiveDate,
    rate: Decimal,
}

/// Frankfurter v2 exchange provider with dated process-local caching.
pub struct FrankfurterClient {
    http: reqwest::Client,
    base_url: String,
    cache: RwLock<HashMap<(Currency, Currency, NaiveDate), RateQuote>>,
}

impl FrankfurterClient {
    /// Creates a client using the public v2 endpoint.
    pub fn new() -> Self {
        Self::with_base_url(DEFAULT_BASE_URL)
    }

    /// Creates a client using a custom endpoint, intended for local tests.
    pub fn with_base_url(base_url: &str) -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url: base_url.trim_end_matches('/').to_owned(),
            cache: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for FrankfurterClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ExchangeRateProvider for FrankfurterClient {
    async fn rate(
        &self,
        base: Currency,
        quote: Currency,
        requested_date: NaiveDate,
        today: NaiveDate,
    ) -> Result<RateQuote, ApplicationError> {
        if base == quote {
            return Ok(RateQuote {
                base,
                quote,
                effective_date: requested_date,
                rate: Decimal::ONE,
                is_stale: false,
                is_provisional: requested_date > today,
            });
        }
        let requested_date = requested_date.min(today);
        let key = (base, quote, requested_date);
        let cached = self
            .cache
            .read()
            .map_err(|error| ApplicationError::Unavailable(error.to_string()))?
            .get(&key)
            .cloned();
        if let Some(value) = cached {
            return Ok(value);
        }
        let url = format!(
            "{}/rate/{}/{}?date={requested_date}",
            self.base_url,
            base.code(),
            quote.code()
        );
        let response = match self.http.get(url).send().await {
            Ok(response) if response.status().is_success() => response,
            Ok(response) => {
                return self
                    .stale_or_error(key, format!("Frankfurter returned {}", response.status()));
            }
            Err(error) => return self.stale_or_error(key, error.to_string()),
        };
        let payload: RateResponse = response.json().await.map_err(|error| {
            ApplicationError::Unavailable(format!("invalid Frankfurter response: {error}"))
        })?;
        if payload.rate <= Decimal::ZERO {
            return Err(ApplicationError::Unavailable(
                "Frankfurter returned a non-positive rate".into(),
            ));
        }
        let value = RateQuote {
            base,
            quote,
            effective_date: payload.date,
            rate: payload.rate,
            is_stale: false,
            is_provisional: requested_date != payload.date
                || requested_date < today && payload.date == today,
        };
        self.cache
            .write()
            .map_err(|error| ApplicationError::Unavailable(error.to_string()))?
            .insert(key, value.clone());
        Ok(value)
    }
}

impl FrankfurterClient {
    fn stale_or_error(
        &self,
        key: (Currency, Currency, NaiveDate),
        message: String,
    ) -> Result<RateQuote, ApplicationError> {
        let stale = self
            .cache
            .read()
            .ok()
            .and_then(|cache| cache.get(&key).cloned());
        stale
            .map(|mut quote| {
                quote.is_stale = true;
                quote
            })
            .ok_or(ApplicationError::Unavailable(message))
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn identity_rate_is_exact_without_network() {
        let client = FrankfurterClient::with_base_url("http://127.0.0.1:1");
        let date = NaiveDate::from_ymd_opt(2026, 1, 1).expect("valid constant date");
        assert_eq!(
            client
                .rate(Currency::Usd, Currency::Usd, date, date)
                .await
                .expect("identity rate")
                .rate,
            Decimal::ONE
        );
    }
}
