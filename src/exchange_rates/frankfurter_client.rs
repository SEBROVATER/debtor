use chrono::{NaiveDate, NaiveDateTime};
use reqwest::Client;
use rust_decimal::Decimal;
use serde::Deserialize;
use std::collections::HashMap;

use crate::exchange_rates::rate_service::{ExchangeProvider, RateError, RateQuote};

#[derive(Clone)]
pub struct FrankfurterClient {
    client: Client,
    base_url: String,
}

impl FrankfurterClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            base_url: "https://api.frankfurter.app".to_string(),
        }
    }

    pub fn with_base_url(base_url: &str) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }

    pub fn parse_response(
        body: &str,
        fetched_at: NaiveDateTime,
    ) -> Result<Vec<RateQuote>, RateError> {
        let response: FrankfurterResponse = serde_json::from_str(body)
            .map_err(|err| RateError::ProviderFailed(err.to_string()))?;
        response.to_quotes(fetched_at)
    }

    async fn fetch_raw(&self, from: &str, to: &[String]) -> Result<String, RateError> {
        let url = format!("{}/latest", self.base_url);
        let response = self
            .client
            .get(url)
            .query(&[("from", from), ("to", &to.join(","))])
            .send()
            .await
            .map_err(|err| RateError::ProviderFailed(err.to_string()))?;

        let status = response.status();
        if !status.is_success() {
            return Err(RateError::ProviderFailed(format!(
                "status {}",
                status.as_u16()
            )));
        }

        response
            .text()
            .await
            .map_err(|err| RateError::ProviderFailed(err.to_string()))
    }
}

#[async_trait::async_trait]
impl ExchangeProvider for FrankfurterClient {
    async fn fetch_rates(&self, from: &str, to: &[String]) -> Result<Vec<RateQuote>, RateError> {
        let body = self.fetch_raw(from, to).await?;
        let fetched_at = chrono::Utc::now().naive_utc();
        Self::parse_response(&body, fetched_at)
    }
}

#[derive(Debug, Deserialize)]
struct FrankfurterResponse {
    base: String,
    date: String,
    rates: HashMap<String, Decimal>,
}

impl FrankfurterResponse {
    fn to_quotes(self, fetched_at: NaiveDateTime) -> Result<Vec<RateQuote>, RateError> {
        let rate_date = NaiveDate::parse_from_str(&self.date, "%Y-%m-%d")
            .map_err(|err| RateError::ProviderFailed(err.to_string()))?;

        let mut quotes = Vec::new();
        for (to_currency, rate) in self.rates {
            quotes.push(RateQuote {
                from_currency: self.base.clone(),
                to_currency,
                rate,
                fetched_at,
                rate_date,
                provider: "frankfurter".to_string(),
            });
        }
        Ok(quotes)
    }
}
