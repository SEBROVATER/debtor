use chrono::NaiveDateTime;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, Set,
};

use crate::db::entities::members;

#[derive(Clone)]
pub struct MemberRepo {
    conn: DatabaseConnection,
}

impl MemberRepo {
    pub fn new(conn: DatabaseConnection) -> Self {
        Self { conn }
    }

    pub async fn create(
        &self,
        id: String,
        group_id: String,
        display_name: String,
        now: NaiveDateTime,
    ) -> Result<members::Model, DbErr> {
        let model = members::ActiveModel {
            id: Set(id.clone()),
            group_id: Set(group_id.clone()),
            display_name: Set(display_name.clone()),
            is_active: Set(true),
            created_at: Set(now),
            updated_at: Set(now),
            removed_at: Set(None),
        };
        members::Entity::insert(model).exec(&self.conn).await?;
        Ok(members::Model {
            id,
            group_id,
            display_name,
            is_active: true,
            created_at: now,
            updated_at: now,
            removed_at: None,
        })
    }

    pub async fn find(&self, member_id: &str) -> Result<Option<members::Model>, DbErr> {
        members::Entity::find_by_id(member_id.to_string())
            .one(&self.conn)
            .await
    }

    pub async fn list(
        &self,
        group_id: &str,
        include_inactive: bool,
    ) -> Result<Vec<members::Model>, DbErr> {
        let mut query = members::Entity::find().filter(members::Column::GroupId.eq(group_id));
        if !include_inactive {
            query = query.filter(members::Column::IsActive.eq(true));
        }
        query.all(&self.conn).await
    }

    pub async fn update_name(
        &self,
        member_id: &str,
        display_name: String,
        now: NaiveDateTime,
    ) -> Result<Option<members::Model>, DbErr> {
        let Some(existing) = self.find(member_id).await? else {
            return Ok(None);
        };

        let updated = members::ActiveModel {
            id: Set(existing.id),
            display_name: Set(display_name),
            updated_at: Set(now),
            ..Default::default()
        }
        .update(&self.conn)
        .await?;

        Ok(Some(updated))
    }

    pub async fn soft_delete(
        &self,
        member_id: &str,
        now: NaiveDateTime,
    ) -> Result<Option<members::Model>, DbErr> {
        let Some(existing) = self.find(member_id).await? else {
            return Ok(None);
        };

        let updated = members::ActiveModel {
            id: Set(existing.id),
            is_active: Set(false),
            removed_at: Set(Some(now)),
            updated_at: Set(now),
            ..Default::default()
        }
        .update(&self.conn)
        .await?;

        Ok(Some(updated))
    }
}
