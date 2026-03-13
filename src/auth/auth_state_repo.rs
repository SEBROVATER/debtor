use chrono::{Duration, NaiveDateTime};
use sea_orm::{ActiveModelTrait, DatabaseConnection, DbErr, EntityTrait, Set};

use crate::db::entities::auth_state;

const AUTH_STATE_ID: i32 = 1;
const LOCKOUT_THRESHOLD: i32 = 5;
const LOCKOUT_MINUTES: i64 = 15;

#[derive(Clone)]
pub struct AuthStateRepo {
    conn: DatabaseConnection,
}

impl AuthStateRepo {
    pub fn new(conn: DatabaseConnection) -> Self {
        Self { conn }
    }

    pub async fn load_or_create(&self, now: NaiveDateTime) -> Result<auth_state::Model, DbErr> {
        if let Some(model) = auth_state::Entity::find_by_id(AUTH_STATE_ID)
            .one(&self.conn)
            .await?
        {
            return Ok(model);
        }

        let model = auth_state::ActiveModel {
            id: Set(AUTH_STATE_ID),
            failed_attempt_count: Set(0),
            lockout_until: Set(None),
            last_failed_at: Set(None),
            updated_at: Set(now),
        }
        .insert(&self.conn)
        .await?;

        Ok(model)
    }

    pub fn is_locked_out(state: &auth_state::Model, now: NaiveDateTime) -> bool {
        state
            .lockout_until
            .map(|until| until > now)
            .unwrap_or(false)
    }

    pub async fn record_failure(
        &self,
        mut state: auth_state::Model,
        now: NaiveDateTime,
    ) -> Result<auth_state::Model, DbErr> {
        state.failed_attempt_count += 1;
        state.last_failed_at = Some(now);

        if state.failed_attempt_count >= LOCKOUT_THRESHOLD {
            state.lockout_until = Some(now + Duration::minutes(LOCKOUT_MINUTES));
        }

        let updated = auth_state::ActiveModel {
            id: Set(state.id),
            failed_attempt_count: Set(state.failed_attempt_count),
            lockout_until: Set(state.lockout_until),
            last_failed_at: Set(state.last_failed_at),
            updated_at: Set(now),
        }
        .update(&self.conn)
        .await?;

        Ok(updated)
    }

    pub async fn reset_failures(
        &self,
        state: auth_state::Model,
        now: NaiveDateTime,
    ) -> Result<auth_state::Model, DbErr> {
        let updated = auth_state::ActiveModel {
            id: Set(state.id),
            failed_attempt_count: Set(0),
            lockout_until: Set(None),
            last_failed_at: Set(None),
            updated_at: Set(now),
        }
        .update(&self.conn)
        .await?;

        Ok(updated)
    }
}
