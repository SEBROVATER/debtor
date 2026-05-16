use chrono::{Duration, NaiveDateTime};
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::app::config::AppConfig;

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

#[derive(Debug, Clone)]
pub struct SessionRow {
    pub id: String,
    pub user_id: i64,
    pub token_hash: String,
    pub created_at: NaiveDateTime,
    pub last_seen_at: NaiveDateTime,
    pub expires_at: NaiveDateTime,
    pub revoked_at: Option<NaiveDateTime>,
}

#[derive(Clone)]
pub struct SessionRepo {
    pool: SqlitePool,
}

impl SessionRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create_session(
        &self,
        user_id: i64,
        now: NaiveDateTime,
    ) -> Result<SessionToken, sqlx::Error> {
        let raw = Uuid::new_v4().to_string();
        let hash = hash_token(&raw);
        let expires_at = now + Duration::days(30);
        let session_id = Uuid::new_v4().to_string();

        sqlx::query!(
            "INSERT INTO sessions (id, user_id, token_hash, created_at, last_seen_at, expires_at, revoked_at)
             VALUES (?, ?, ?, ?, ?, ?, NULL)",
            session_id,
            user_id,
            hash,
            now,
            now,
            expires_at
        )
        .execute(&self.pool)
        .await?;

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
    ) -> Result<Option<SessionRow>, sqlx::Error> {
        let hash = hash_token(raw_token);
        let row = sqlx::query_as!(
            SessionRow,
            r#"SELECT id, user_id,
               token_hash,
               created_at as "created_at: NaiveDateTime",
               last_seen_at as "last_seen_at: NaiveDateTime",
               expires_at as "expires_at: NaiveDateTime",
               revoked_at as "revoked_at: Option<NaiveDateTime>"
               FROM sessions
               WHERE token_hash = ? AND revoked_at IS NULL"#,
            hash
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.filter(|s| s.expires_at > now))
    }

    pub async fn touch_session(
        &self,
        raw_token: &str,
        now: NaiveDateTime,
    ) -> Result<Option<SessionRow>, sqlx::Error> {
        let session = self.find_active_session(raw_token, now).await?;
        let Some(session) = session else {
            return Ok(None);
        };

        let new_expires_at = now + Duration::days(30);
        sqlx::query!(
            "UPDATE sessions SET last_seen_at = ?, expires_at = ? WHERE id = ?",
            now,
            new_expires_at,
            session.id
        )
        .execute(&self.pool)
        .await?;

        Ok(Some(SessionRow {
            last_seen_at: now,
            expires_at: new_expires_at,
            ..session
        }))
    }

    pub async fn revoke_session(
        &self,
        raw_token: &str,
        now: NaiveDateTime,
    ) -> Result<bool, sqlx::Error> {
        let hash = hash_token(raw_token);
        Ok(sqlx::query!(
            "UPDATE sessions SET revoked_at = ? WHERE token_hash = ? AND revoked_at IS NULL",
            now,
            hash
        )
        .execute(&self.pool)
        .await?
        .rows_affected()
            > 0)
    }
}

fn hash_token(raw: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(raw.as_bytes());
    let result = hasher.finalize();
    result.iter().map(|byte| format!("{byte:02x}")).collect()
}
