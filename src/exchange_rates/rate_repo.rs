use chrono::{NaiveDate, NaiveDateTime};
use rust_decimal::Decimal;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::exchange_rates::rate_service::RateQuote;

#[derive(Debug, Clone)]
pub struct ExchangeRateRow {
    pub id: String,
    pub from_currency: String,
    pub to_currency: String,
    pub rate: Decimal,
    pub fetched_at: NaiveDateTime,
    pub rate_date: NaiveDate,
    pub provider: String,
}

#[derive(Clone)]
pub struct RateRepo {
    pool: SqlitePool,
}

impl RateRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn find_rate_on_date(
        &self,
        from_currency: &str,
        to_currency: &str,
        rate_date: NaiveDate,
    ) -> Result<Option<ExchangeRateRow>, sqlx::Error> {
        sqlx::query_as!(
            ExchangeRateRow,
            r#"SELECT id, from_currency, to_currency,
               rate as "rate: Decimal",
               fetched_at as "fetched_at: NaiveDateTime",
               rate_date as "rate_date: NaiveDate",
               provider
               FROM exchange_rates
               WHERE from_currency = ? AND to_currency = ? AND rate_date = ?"#,
            from_currency,
            to_currency,
            rate_date
        )
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn find_latest_rate(
        &self,
        from_currency: &str,
        to_currency: &str,
    ) -> Result<Option<ExchangeRateRow>, sqlx::Error> {
        sqlx::query_as!(
            ExchangeRateRow,
            r#"SELECT id, from_currency, to_currency,
               rate as "rate: Decimal",
               fetched_at as "fetched_at: NaiveDateTime",
               rate_date as "rate_date: NaiveDate",
               provider
               FROM exchange_rates
               WHERE from_currency = ? AND to_currency = ?
               ORDER BY rate_date DESC, fetched_at DESC
               LIMIT 1"#,
            from_currency,
            to_currency
        )
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn upsert_rate(&self, quote: RateQuote) -> Result<ExchangeRateRow, sqlx::Error> {
        if let Some(existing) = self
            .find_rate_on_date(&quote.from_currency, &quote.to_currency, quote.rate_date)
            .await?
        {
            sqlx::query!(
                "UPDATE exchange_rates SET rate = ?, fetched_at = ?, provider = ? WHERE id = ?",
                quote.rate,
                quote.fetched_at,
                quote.provider,
                existing.id
            )
            .execute(&self.pool)
            .await?;

            return self
                .find_rate_on_date(&quote.from_currency, &quote.to_currency, quote.rate_date)
                .await?
                .ok_or_else(|| {
                    sqlx::Error::RowNotFound
                });
        }

        let id = Uuid::new_v4().to_string();
        sqlx::query!(
            "INSERT INTO exchange_rates (id, from_currency, to_currency, rate, fetched_at, rate_date, provider)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
            id,
            quote.from_currency,
            quote.to_currency,
            quote.rate,
            quote.fetched_at,
            quote.rate_date,
            quote.provider
        )
        .execute(&self.pool)
        .await?;

        self.find_rate_on_date(&quote.from_currency, &quote.to_currency, quote.rate_date)
            .await?
            .ok_or(sqlx::Error::RowNotFound)
    }

    pub async fn insert_manual(
        &self,
        from_currency: &str,
        to_currency: &str,
        rate: Decimal,
        fetched_at: NaiveDateTime,
        rate_date: NaiveDate,
        provider: &str,
    ) -> Result<ExchangeRateRow, sqlx::Error> {
        let id = Uuid::new_v4().to_string();
        sqlx::query!(
            "INSERT INTO exchange_rates (id, from_currency, to_currency, rate, fetched_at, rate_date, provider)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
            id,
            from_currency,
            to_currency,
            rate,
            fetched_at,
            rate_date,
            provider
        )
        .execute(&self.pool)
        .await?;

        self.find_rate_on_date(from_currency, to_currency, rate_date)
            .await?
            .ok_or(sqlx::Error::RowNotFound)
    }
}
