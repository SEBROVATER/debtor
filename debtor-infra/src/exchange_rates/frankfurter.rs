use std::collections::HashMap;
use std::sync::RwLock;

use async_trait::async_trait;
use chrono::NaiveDate;
use debtor_application::{ApplicationError, ExchangeRateProvider, RateQuote, UnavailableReason};
use debtor_domain::currency::Currency;
use rust_decimal::Decimal;
use serde::Deserialize;

const DEFAULT_BASE_URL: &str = "https://api.frankfurter.dev/v2";
type CacheKey = (Currency, Currency, NaiveDate, NaiveDate);

#[derive(Debug, Deserialize)]
struct RateResponse {
    date: NaiveDate,
    rate: Decimal,
}

/// Frankfurter v2 exchange provider with dated process-local caching.
pub struct FrankfurterClient {
    http: reqwest::Client,
    base_url: String,
    stable_cache: RwLock<HashMap<CacheKey, RateQuote>>,
    refreshable_cache: RwLock<HashMap<CacheKey, RateQuote>>,
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
            stable_cache: RwLock::new(HashMap::new()),
            refreshable_cache: RwLock::new(HashMap::new()),
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
        let original_requested_date = requested_date;
        let fetch_date = requested_date.min(today);
        if base == quote {
            return Ok(RateQuote {
                base,
                quote,
                requested_date: original_requested_date,
                effective_date: fetch_date,
                rate: Decimal::ONE,
                is_stale: false,
                is_provisional: original_requested_date > today,
            });
        }
        let key = (base, quote, original_requested_date, fetch_date);
        let cache = self.cache_for(original_requested_date, today);
        let cached = cache
            .read()
            .map_err(|_| ApplicationError::Unavailable(UnavailableReason::ExchangeRates))?
            .get(&key)
            .cloned();
        if let Some(value) = cached {
            return Ok(value);
        }
        let url = format!(
            "{}/rate/{}/{}?date={fetch_date}",
            self.base_url,
            base.code(),
            quote.code()
        );
        let response = match self.http.get(url).send().await {
            Ok(response) if response.status().is_success() => response,
            Ok(_) | Err(_) => return self.stale_or_error(key, today),
        };
        let payload: RateResponse = match response.json().await {
            Ok(payload) => payload,
            Err(_) => return self.stale_or_error(key, today),
        };
        if payload.rate <= Decimal::ZERO {
            return self.stale_or_error(key, today);
        }
        let value = RateQuote {
            base,
            quote,
            requested_date: original_requested_date,
            effective_date: payload.date,
            rate: payload.rate,
            is_stale: false,
            is_provisional: original_requested_date > today,
        };
        cache
            .write()
            .map_err(|_| ApplicationError::Unavailable(UnavailableReason::ExchangeRates))?
            .insert(key, value.clone());
        Ok(value)
    }
}

impl FrankfurterClient {
    fn stale_or_error(
        &self,
        key: CacheKey,
        today: NaiveDate,
    ) -> Result<RateQuote, ApplicationError> {
        let (base, quote, requested_date, fetch_date) = key;
        if requested_date < today {
            return Err(ApplicationError::Unavailable(
                UnavailableReason::ExchangeRates,
            ));
        }
        let stale = self.refreshable_cache.read().ok().and_then(|cache| {
            cache
                .iter()
                .filter(
                    |((cached_base, cached_quote, cached_requested, cached_fetch), _)| {
                        *cached_base == base
                            && *cached_quote == quote
                            && *cached_fetch < fetch_date
                            && if requested_date == today {
                                *cached_requested == *cached_fetch
                            } else {
                                *cached_requested == requested_date
                            }
                    },
                )
                .max_by_key(|((_, _, _, cached_fetch), _)| *cached_fetch)
                .map(|(_, quote)| quote.clone())
        });
        stale
            .map(|mut quote| {
                quote.requested_date = requested_date;
                quote.is_stale = true;
                quote.is_provisional = requested_date > today;
                quote
            })
            .ok_or(ApplicationError::Unavailable(
                UnavailableReason::ExchangeRates,
            ))
    }

    fn cache_for(
        &self,
        requested_date: NaiveDate,
        today: NaiveDate,
    ) -> &RwLock<HashMap<CacheKey, RateQuote>> {
        if requested_date < today {
            &self.stable_cache
        } else {
            &self.refreshable_cache
        }
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

    #[tokio::test]
    async fn current_rollover_failure_uses_latest_prior_current_quote() {
        let client = FrankfurterClient::with_base_url("http://127.0.0.1:1");
        let previous = date(2026, 1, 1);
        let today = date(2026, 1, 2);
        insert_refreshable(
            &client,
            Currency::Usd,
            Currency::Eur,
            previous,
            previous,
            previous,
        );

        let quote = client
            .rate(Currency::Usd, Currency::Eur, today, today)
            .await
            .expect("prior current quote is a fallback");

        assert_eq!(quote.requested_date, today);
        assert_eq!(quote.effective_date, previous);
        assert!(quote.is_stale);
        assert!(!quote.is_provisional);
    }

    #[tokio::test]
    async fn future_rollover_failure_uses_only_the_same_future_context() {
        let client = FrankfurterClient::with_base_url("http://127.0.0.1:1");
        let previous = date(2026, 1, 1);
        let today = date(2026, 1, 2);
        let requested = date(2026, 1, 10);
        insert_refreshable(
            &client,
            Currency::Usd,
            Currency::Eur,
            requested,
            previous,
            previous,
        );
        insert_refreshable(
            &client,
            Currency::Usd,
            Currency::Eur,
            date(2026, 1, 11),
            previous,
            previous,
        );

        let quote = client
            .rate(Currency::Usd, Currency::Eur, requested, today)
            .await
            .expect("matching future quote is a fallback");

        assert_eq!(quote.requested_date, requested);
        assert_eq!(quote.effective_date, previous);
        assert!(quote.is_stale);
        assert!(quote.is_provisional);
    }

    #[tokio::test]
    async fn failures_never_fall_back_across_rate_contexts() {
        let client = FrankfurterClient::with_base_url("http://127.0.0.1:1");
        let previous = date(2026, 1, 1);
        let today = date(2026, 1, 2);
        insert_refreshable(
            &client,
            Currency::Usd,
            Currency::Eur,
            date(2026, 1, 10),
            previous,
            previous,
        );

        assert!(
            client
                .rate(Currency::Usd, Currency::Eur, today, today)
                .await
                .is_err()
        );
        assert!(
            client
                .rate(Currency::Usd, Currency::Eur, date(2026, 1, 11), today)
                .await
                .is_err()
        );
        assert!(
            client
                .rate(Currency::Usd, Currency::Eur, previous, today)
                .await
                .is_err()
        );
    }

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).expect("valid constant date")
    }

    fn insert_refreshable(
        client: &FrankfurterClient,
        base: Currency,
        quote: Currency,
        requested_date: NaiveDate,
        fetch_date: NaiveDate,
        effective_date: NaiveDate,
    ) {
        client
            .refreshable_cache
            .write()
            .expect("cache lock")
            .insert(
                (base, quote, requested_date, fetch_date),
                RateQuote {
                    base,
                    quote,
                    requested_date,
                    effective_date,
                    rate: Decimal::ONE,
                    is_stale: false,
                    is_provisional: requested_date > fetch_date,
                },
            );
    }
}
