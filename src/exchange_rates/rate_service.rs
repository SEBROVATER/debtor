use chrono::{NaiveDate, NaiveDateTime};
use rust_decimal::Decimal;
use std::sync::Arc;
use thiserror::Error;

use crate::exchange_rates::rate_repo::RateRepo;

#[derive(Debug, Clone)]
pub struct RateQuote {
    pub from_currency: String,
    pub to_currency: String,
    pub rate: Decimal,
    pub fetched_at: NaiveDateTime,
    pub rate_date: NaiveDate,
    pub provider: String,
}

#[derive(Debug, Clone)]
pub struct RateLookup {
    pub rate: Decimal,
    pub rate_date: NaiveDate,
    pub stale: bool,
}

#[derive(Debug, Error)]
pub enum RateError {
    #[error("provider failed: {0}")]
    ProviderFailed(String),
    #[error(transparent)]
    Database(#[from] sea_orm::DbErr),
    #[error("missing rate")]
    MissingRate,
}

#[async_trait::async_trait]
pub trait ExchangeProvider: Send + Sync {
    async fn fetch_rates(&self, from: &str, to: &[String]) -> Result<Vec<RateQuote>, RateError>;
}

#[derive(Clone)]
pub struct RateService {
    repo: RateRepo,
    provider: Arc<dyn ExchangeProvider>,
}

impl RateService {
    pub fn new(repo: RateRepo, provider: Arc<dyn ExchangeProvider>) -> Self {
        Self { repo, provider }
    }

    pub async fn get_rate(
        &self,
        from_currency: &str,
        to_currency: &str,
        now: NaiveDateTime,
    ) -> Result<RateLookup, RateError> {
        let today = now.date();
        if let Some(rate) = self
            .repo
            .find_rate_on_date(from_currency, to_currency, today)
            .await?
        {
            return Ok(RateLookup {
                rate: rate.rate,
                rate_date: rate.rate_date,
                stale: false,
            });
        }

        let fetch_result = self
            .provider
            .fetch_rates(from_currency, &[to_currency.to_string()])
            .await;

        if let Ok(quotes) = fetch_result {
            for quote in quotes {
                let _ = self.repo.upsert_rate(quote.clone()).await?;
            }
            if let Some(rate) = self
                .repo
                .find_rate_on_date(from_currency, to_currency, today)
                .await?
            {
                return Ok(RateLookup {
                    rate: rate.rate,
                    rate_date: rate.rate_date,
                    stale: false,
                });
            }
        }

        if let Some(latest) = self
            .repo
            .find_latest_rate(from_currency, to_currency)
            .await?
        {
            return Ok(RateLookup {
                rate: latest.rate,
                rate_date: latest.rate_date,
                stale: true,
            });
        }

        Err(RateError::MissingRate)
    }
}
