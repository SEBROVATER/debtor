use chrono::{Duration, NaiveDateTime};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, Set,
};

use crate::app::config::AppConfig;
use crate::db::entities::sessions;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SameSitePolicy {
    Lax,
    Strict,
    None,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionCookiePolicy {
    pub http_only: bool,
    pub same_site: SameSitePolicy,
    pub secure: bool,
    pub path: &'static str,
    pub max_age_days: i64,
}

impl SessionCookiePolicy {
    pub fn from_config(config: &AppConfig) -> Self {
        Self {
            http_only: true,
            same_site: SameSitePolicy::Lax,
            secure: config.secure_cookie,
            path: "/",
            max_age_days: 30,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionToken {
    pub raw: String,
    pub hash: String,
    pub expires_at: NaiveDateTime,
}

#[derive(Clone)]
pub struct SessionRepo {
    conn: DatabaseConnection,
}

impl SessionRepo {
    pub fn new(conn: DatabaseConnection) -> Self {
        Self { conn }
    }

    pub async fn create_session(
        &self,
        user_id: i32,
        now: NaiveDateTime,
    ) -> Result<SessionToken, DbErr> {
        let raw = Uuid::new_v4().to_string();
        let hash = hash_token(&raw);
        let expires_at = now + Duration::days(30);

        let model = sessions::ActiveModel {
            id: Set(Uuid::new_v4().to_string()),
            user_id: Set(user_id),
            token_hash: Set(hash.clone()),
            created_at: Set(now),
            last_seen_at: Set(now),
            expires_at: Set(expires_at),
            revoked_at: Set(None),
        };
        sessions::Entity::insert(model).exec(&self.conn).await?;

        Ok(SessionToken {
            raw,
            hash,
            expires_at,
        })
    }

    pub async fn find_active_session(
        &self,
        raw_token: &str,
        now: NaiveDateTime,
    ) -> Result<Option<sessions::Model>, DbErr> {
        let hash = hash_token(raw_token);
        let session = sessions::Entity::find()
            .filter(sessions::Column::TokenHash.eq(hash))
            .filter(sessions::Column::RevokedAt.is_null())
            .one(&self.conn)
            .await?;

        Ok(session.filter(|s| s.expires_at > now))
    }

    pub async fn touch_session(
        &self,
        raw_token: &str,
        now: NaiveDateTime,
    ) -> Result<Option<sessions::Model>, DbErr> {
        let session = self.find_active_session(raw_token, now).await?;
        let Some(session) = session else {
            return Ok(None);
        };

        let new_expires_at = now + Duration::days(30);
        let updated = sessions::ActiveModel {
            id: Set(session.id.clone()),
            last_seen_at: Set(now),
            expires_at: Set(new_expires_at),
            ..Default::default()
        }
        .update(&self.conn)
        .await?;

        Ok(Some(updated))
    }

    pub async fn revoke_session(&self, raw_token: &str, now: NaiveDateTime) -> Result<bool, DbErr> {
        let hash = hash_token(raw_token);
        let session = sessions::Entity::find()
            .filter(sessions::Column::TokenHash.eq(hash))
            .one(&self.conn)
            .await?;

        let Some(session) = session else {
            return Ok(false);
        };

        let _ = sessions::ActiveModel {
            id: Set(session.id),
            revoked_at: Set(Some(now)),
            ..Default::default()
        }
        .update(&self.conn)
        .await?;

        Ok(true)
    }
}

fn hash_token(raw: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(raw.as_bytes());
    let result = hasher.finalize();
    result.iter().map(|byte| format!("{byte:02x}")).collect()
}
