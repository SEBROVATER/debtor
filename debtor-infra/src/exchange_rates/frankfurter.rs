use std::collections::HashMap;
use std::str::FromStr;
use std::sync::RwLock;

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::Deserialize;

use debtor_domain::currency::Currency;
use debtor_domain::traits::{ExchangeRateError, ExchangeRateProvider};

const DEFAULT_BASE_URL: &str = "https://api.frankfurter.dev/v2";

#[derive(Debug, Deserialize)]
struct RateResponse {
    rate: serde_json::Value,
}

/// Exchange rate client backed by the Frankfurter API.
///
/// Caches rates lazily per currency pair per day. On the first request for
/// a pair each day, the rate is fetched from the API and stored. Subsequent
/// requests for the same pair on the same day return the cached value.
/// No background tasks or periodic refresh cycles are used.
pub struct FrankfurterClient {
    http: reqwest::Client,
    base_url: String,
    cache: RwLock<HashMap<(Currency, Currency), (NaiveDate, Decimal)>>,
}

impl FrankfurterClient {
    /// Creates a new client pointing at the live Frankfurter API.
    pub fn new() -> Self {
        Self::with_base_url(DEFAULT_BASE_URL)
    }

    /// Creates a new client pointing at a custom base URL.
    ///
    /// Useful for testing with a local mock server.
    pub fn with_base_url(base_url: &str) -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url: base_url.trim_end_matches('/').to_owned(),
            cache: RwLock::new(HashMap::new()),
        }
    }

    fn today() -> NaiveDate {
        chrono::Utc::now().date_naive()
    }
}

impl Default for FrankfurterClient {
    fn default() -> Self {
        Self::new()
    }
}

impl ExchangeRateProvider for FrankfurterClient {
    async fn get_rate(
        &self,
        base: Currency,
        quote: Currency,
    ) -> Result<Decimal, ExchangeRateError> {
        if base == quote {
            return Ok(Decimal::ONE);
        }

        {
            let cache = self
                .cache
                .read()
                .map_err(|e| ExchangeRateError::FetchFailed(e.to_string()))?;
            if let Some((date, rate)) = cache.get(&(base, quote)) {
                if *date == Self::today() {
                    return Ok(*rate);
                }
            }
        }

        let url = format!("{}/rate/{}/{}", self.base_url, base.code(), quote.code());

        let response = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(|e| ExchangeRateError::FetchFailed(e.to_string()))?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(ExchangeRateError::UnsupportedCurrency(format!(
                "{}/{}",
                base.code(),
                quote.code()
            )));
        }

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ExchangeRateError::FetchFailed(format!(
                "Frankfurter API returned {status}: {body}"
            )));
        }

        let rate_response: RateResponse = response
            .json()
            .await
            .map_err(|e| ExchangeRateError::FetchFailed(format!("invalid JSON: {e}")))?;

        let rate_str = rate_response
            .rate
            .as_str()
            .ok_or_else(|| ExchangeRateError::FetchFailed("rate field is not a string".into()))?;

        let rate = Decimal::from_str(rate_str)
            .map_err(|e| ExchangeRateError::FetchFailed(format!("invalid rate value: {e}")))?;

        {
            let mut cache = self
                .cache
                .write()
                .map_err(|e| ExchangeRateError::FetchFailed(e.to_string()))?;
            cache.insert((base, quote), (Self::today(), rate));
        }

        Ok(rate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use debtor_domain::currency::Currency;

    #[tokio::test]
    async fn same_currency_returns_one() {
        let client = FrankfurterClient::with_base_url("http://127.0.0.1:1");
        assert_eq!(
            client.get_rate(Currency::Eur, Currency::Eur).await.unwrap(),
            Decimal::ONE
        );
    }

    #[tokio::test]
    async fn cache_hit_avoids_api_call() {
        let cache = RwLock::new(HashMap::new());
        cache.write().unwrap().insert(
            (Currency::Eur, Currency::Usd),
            (FrankfurterClient::today(), Decimal::new(108, 2)),
        );

        let client = FrankfurterClient {
            http: reqwest::Client::new(),
            base_url: "http://127.0.0.1:1".to_owned(),
            cache,
        };

        let rate = client.get_rate(Currency::Eur, Currency::Usd).await.unwrap();
        assert_eq!(rate, Decimal::new(108, 2));
    }

    #[tokio::test]
    async fn stale_cache_triggers_refetch() {
        let stale_date = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
        let cache = RwLock::new(HashMap::new());
        cache.write().unwrap().insert(
            (Currency::Eur, Currency::Usd),
            (stale_date, Decimal::new(100, 2)),
        );

        let client = FrankfurterClient {
            http: reqwest::Client::new(),
            base_url: "http://127.0.0.1:1".to_owned(),
            cache,
        };

        let result = client.get_rate(Currency::Eur, Currency::Usd).await;
        assert!(result.is_err());
    }
}
