use chrono::{NaiveDate, NaiveDateTime};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, QueryOrder,
    Set,
};
use uuid::Uuid;

use crate::db::entities::exchange_rates;
use crate::exchange_rates::rate_service::RateQuote;

#[derive(Clone)]
pub struct RateRepo {
    conn: DatabaseConnection,
}

impl RateRepo {
    pub fn new(conn: DatabaseConnection) -> Self {
        Self { conn }
    }

    pub async fn find_rate_on_date(
        &self,
        from_currency: &str,
        to_currency: &str,
        rate_date: NaiveDate,
    ) -> Result<Option<exchange_rates::Model>, DbErr> {
        exchange_rates::Entity::find()
            .filter(exchange_rates::Column::FromCurrency.eq(from_currency))
            .filter(exchange_rates::Column::ToCurrency.eq(to_currency))
            .filter(exchange_rates::Column::RateDate.eq(rate_date))
            .one(&self.conn)
            .await
    }

    pub async fn find_latest_rate(
        &self,
        from_currency: &str,
        to_currency: &str,
    ) -> Result<Option<exchange_rates::Model>, DbErr> {
        exchange_rates::Entity::find()
            .filter(exchange_rates::Column::FromCurrency.eq(from_currency))
            .filter(exchange_rates::Column::ToCurrency.eq(to_currency))
            .order_by_desc(exchange_rates::Column::RateDate)
            .order_by_desc(exchange_rates::Column::FetchedAt)
            .one(&self.conn)
            .await
    }

    pub async fn upsert_rate(&self, quote: RateQuote) -> Result<exchange_rates::Model, DbErr> {
        if let Some(existing) = self
            .find_rate_on_date(&quote.from_currency, &quote.to_currency, quote.rate_date)
            .await?
        {
            let updated = exchange_rates::ActiveModel {
                id: Set(existing.id),
                rate: Set(quote.rate),
                fetched_at: Set(quote.fetched_at),
                provider: Set(quote.provider),
                ..Default::default()
            }
            .update(&self.conn)
            .await?;
            return Ok(updated);
        }

        let id = Uuid::new_v4().to_string();
        let model = exchange_rates::ActiveModel {
            id: Set(id.clone()),
            from_currency: Set(quote.from_currency),
            to_currency: Set(quote.to_currency),
            rate: Set(quote.rate),
            fetched_at: Set(quote.fetched_at),
            rate_date: Set(quote.rate_date),
            provider: Set(quote.provider),
        };
        exchange_rates::Entity::insert(model)
            .exec(&self.conn)
            .await?;
        exchange_rates::Entity::find_by_id(id)
            .one(&self.conn)
            .await?
            .ok_or_else(|| DbErr::RecordNotFound("inserted exchange rate missing".to_string()))
    }

    pub async fn insert_manual(
        &self,
        from_currency: &str,
        to_currency: &str,
        rate: rust_decimal::Decimal,
        fetched_at: NaiveDateTime,
        rate_date: NaiveDate,
        provider: &str,
    ) -> Result<exchange_rates::Model, DbErr> {
        let id = Uuid::new_v4().to_string();
        let model = exchange_rates::ActiveModel {
            id: Set(id.clone()),
            from_currency: Set(from_currency.to_string()),
            to_currency: Set(to_currency.to_string()),
            rate: Set(rate),
            fetched_at: Set(fetched_at),
            rate_date: Set(rate_date),
            provider: Set(provider.to_string()),
        };
        exchange_rates::Entity::insert(model)
            .exec(&self.conn)
            .await?;
        exchange_rates::Entity::find_by_id(id)
            .one(&self.conn)
            .await?
            .ok_or_else(|| DbErr::RecordNotFound("inserted exchange rate missing".to_string()))
    }
}
