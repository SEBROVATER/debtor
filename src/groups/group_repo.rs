use chrono::NaiveDateTime;
use sea_orm::{ActiveModelTrait, DatabaseConnection, DbErr, EntityTrait, Set};

use crate::db::entities::groups;

#[derive(Clone)]
pub struct GroupRepo {
    conn: DatabaseConnection,
}

impl GroupRepo {
    pub fn new(conn: DatabaseConnection) -> Self {
        Self { conn }
    }

    pub async fn create(
        &self,
        id: String,
        name: String,
        target_currency: String,
        now: NaiveDateTime,
    ) -> Result<groups::Model, DbErr> {
        let model = groups::ActiveModel {
            id: Set(id.clone()),
            name: Set(name.clone()),
            target_currency: Set(target_currency.clone()),
            created_at: Set(now),
            updated_at: Set(now),
        };
        groups::Entity::insert(model).exec(&self.conn).await?;
        Ok(groups::Model {
            id,
            name,
            target_currency,
            created_at: now,
            updated_at: now,
        })
    }

    pub async fn list(&self) -> Result<Vec<groups::Model>, DbErr> {
        groups::Entity::find().all(&self.conn).await
    }

    pub async fn find(&self, id: &str) -> Result<Option<groups::Model>, DbErr> {
        groups::Entity::find_by_id(id.to_string())
            .one(&self.conn)
            .await
    }

    pub async fn update(
        &self,
        id: &str,
        name: Option<String>,
        target_currency: Option<String>,
        now: NaiveDateTime,
    ) -> Result<Option<groups::Model>, DbErr> {
        let Some(existing) = self.find(id).await? else {
            return Ok(None);
        };

        let updated = groups::ActiveModel {
            id: Set(existing.id),
            name: name.map(Set).unwrap_or_else(|| Set(existing.name)),
            target_currency: target_currency
                .map(Set)
                .unwrap_or_else(|| Set(existing.target_currency)),
            updated_at: Set(now),
            ..Default::default()
        }
        .update(&self.conn)
        .await?;

        Ok(Some(updated))
    }

    pub async fn delete(&self, id: &str) -> Result<bool, DbErr> {
        let result = groups::Entity::delete_by_id(id.to_string())
            .exec(&self.conn)
            .await?;
        Ok(result.rows_affected > 0)
    }
}
