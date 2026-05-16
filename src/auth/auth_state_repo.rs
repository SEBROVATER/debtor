use chrono::{Duration, NaiveDateTime};
use sqlx::SqlitePool;

const AUTH_STATE_ID: i64 = 1;
const LOCKOUT_THRESHOLD: i64 = 5;
const LOCKOUT_MINUTES: i64 = 15;

#[derive(Debug, Clone)]
pub struct AuthStateRow {
    pub id: i64,
    pub failed_attempt_count: i64,
    pub lockout_until: Option<NaiveDateTime>,
    pub last_failed_at: Option<NaiveDateTime>,
    pub updated_at: NaiveDateTime,
}

#[derive(Clone)]
pub struct AuthStateRepo {
    pool: SqlitePool,
}

impl AuthStateRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn load_or_create(&self, now: NaiveDateTime) -> Result<AuthStateRow, sqlx::Error> {
        if let Some(row) = sqlx::query_as!(
            AuthStateRow,
            r#"SELECT id, failed_attempt_count,
               lockout_until as "lockout_until: Option<NaiveDateTime>",
               last_failed_at as "last_failed_at: Option<NaiveDateTime>",
               updated_at as "updated_at: NaiveDateTime"
               FROM auth_state WHERE id = ?"#,
            AUTH_STATE_ID
        )
        .fetch_optional(&self.pool)
        .await?
        {
            return Ok(row);
        }

        sqlx::query!(
            "INSERT INTO auth_state (id, failed_attempt_count, lockout_until, last_failed_at, updated_at)
             VALUES (?, 0, NULL, NULL, ?)",
            AUTH_STATE_ID,
            now
        )
        .execute(&self.pool)
        .await?;

        Ok(AuthStateRow {
            id: AUTH_STATE_ID,
            failed_attempt_count: 0,
            lockout_until: None,
            last_failed_at: None,
            updated_at: now,
        })
    }

    pub fn is_locked_out(state: &AuthStateRow, now: NaiveDateTime) -> bool {
        state
            .lockout_until
            .map(|until| until > now)
            .unwrap_or(false)
    }

    pub async fn record_failure(
        &self,
        mut state: AuthStateRow,
        now: NaiveDateTime,
    ) -> Result<AuthStateRow, sqlx::Error> {
        state.failed_attempt_count += 1;
        state.last_failed_at = Some(now);

        if state.failed_attempt_count >= LOCKOUT_THRESHOLD {
            state.lockout_until = Some(now + Duration::minutes(LOCKOUT_MINUTES));
        }

        sqlx::query!(
            "UPDATE auth_state
             SET failed_attempt_count = ?, lockout_until = ?, last_failed_at = ?, updated_at = ?
             WHERE id = ?",
            state.failed_attempt_count,
            state.lockout_until,
            state.last_failed_at,
            now,
            state.id
        )
        .execute(&self.pool)
        .await?;

        state.updated_at = now;
        Ok(state)
    }

    pub async fn reset_failures(
        &self,
        state: AuthStateRow,
        now: NaiveDateTime,
    ) -> Result<AuthStateRow, sqlx::Error> {
        sqlx::query!(
            "UPDATE auth_state
             SET failed_attempt_count = 0, lockout_until = NULL, last_failed_at = NULL, updated_at = ?
             WHERE id = ?",
            now,
            state.id
        )
        .execute(&self.pool)
        .await?;

        Ok(AuthStateRow {
            id: state.id,
            failed_attempt_count: 0,
            lockout_until: None,
            last_failed_at: None,
            updated_at: now,
        })
    }
}
