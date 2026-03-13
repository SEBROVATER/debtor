use chrono::{NaiveDate, NaiveDateTime};
use rust_decimal::Decimal;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, Set,
};

use crate::db::entities::{expense_shares, expenses};

#[derive(Clone)]
pub struct ExpenseRepo {
    conn: DatabaseConnection,
}

impl ExpenseRepo {
    pub fn new(conn: DatabaseConnection) -> Self {
        Self { conn }
    }

    pub async fn create(
        &self,
        id: String,
        group_id: String,
        payer_member_id: String,
        amount: Decimal,
        currency: String,
        expense_date: NaiveDate,
        note: Option<String>,
        now: NaiveDateTime,
    ) -> Result<expenses::Model, DbErr> {
        let model = expenses::ActiveModel {
            id: Set(id.clone()),
            group_id: Set(group_id.clone()),
            payer_member_id: Set(payer_member_id.clone()),
            amount: Set(amount),
            currency: Set(currency.clone()),
            note: Set(note.clone()),
            expense_date: Set(expense_date),
            created_at: Set(now),
            updated_at: Set(now),
        };
        expenses::Entity::insert(model).exec(&self.conn).await?;
        Ok(expenses::Model {
            id,
            group_id,
            payer_member_id,
            amount,
            currency,
            note,
            expense_date,
            created_at: now,
            updated_at: now,
        })
    }

    pub async fn find(&self, expense_id: &str) -> Result<Option<expenses::Model>, DbErr> {
        expenses::Entity::find_by_id(expense_id.to_string())
            .one(&self.conn)
            .await
    }

    pub async fn list_by_group(&self, group_id: &str) -> Result<Vec<expenses::Model>, DbErr> {
        expenses::Entity::find()
            .filter(expenses::Column::GroupId.eq(group_id))
            .all(&self.conn)
            .await
    }

    pub async fn list_with_shares_by_group(
        &self,
        group_id: &str,
    ) -> Result<Vec<(expenses::Model, Vec<expense_shares::Model>)>, DbErr> {
        expenses::Entity::find()
            .filter(expenses::Column::GroupId.eq(group_id))
            .find_with_related(expense_shares::Entity)
            .all(&self.conn)
            .await
    }

    pub async fn update(
        &self,
        expense_id: &str,
        payer_member_id: String,
        amount: Decimal,
        currency: String,
        expense_date: NaiveDate,
        note: Option<String>,
        now: NaiveDateTime,
    ) -> Result<Option<expenses::Model>, DbErr> {
        let Some(existing) = self.find(expense_id).await? else {
            return Ok(None);
        };

        let updated = expenses::ActiveModel {
            id: Set(existing.id),
            payer_member_id: Set(payer_member_id),
            amount: Set(amount),
            currency: Set(currency),
            note: Set(note),
            expense_date: Set(expense_date),
            updated_at: Set(now),
            ..Default::default()
        }
        .update(&self.conn)
        .await?;

        Ok(Some(updated))
    }

    pub async fn delete(&self, expense_id: &str) -> Result<bool, DbErr> {
        let result = expenses::Entity::delete_by_id(expense_id.to_string())
            .exec(&self.conn)
            .await?;
        Ok(result.rows_affected > 0)
    }
}
