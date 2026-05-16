use chrono::NaiveDateTime;
use sqlx::SqlitePool;

use crate::auth::auth_state_repo::AuthStateRepo;
use crate::auth::password::verify_password;
use crate::auth::session_repo::{SessionRepo, SessionToken};

#[derive(Debug, Clone)]
pub struct AdminUserRow {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoginResult {
    Success(SessionToken),
    InvalidCredentials,
    LockedOut { until: NaiveDateTime },
}

#[derive(Clone)]
pub struct LoginService {
    pool: SqlitePool,
}

impl LoginService {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn login(
        &self,
        username: &str,
        password: &str,
        now: NaiveDateTime,
    ) -> Result<LoginResult, sqlx::Error> {
        let auth_repo = AuthStateRepo::new(self.pool.clone());
        let mut state = auth_repo.load_or_create(now).await?;

        if AuthStateRepo::is_locked_out(&state, now) {
            return Ok(LoginResult::LockedOut {
                until: state.lockout_until.unwrap(),
            });
        }

        let admin = sqlx::query_as!(
            AdminUserRow,
            r#"SELECT id, username, password_hash,
               created_at as "created_at: NaiveDateTime",
               updated_at as "updated_at: NaiveDateTime"
               FROM admin_users WHERE username = ?"#,
            username
        )
        .fetch_optional(&self.pool)
        .await?;

        let Some(admin) = admin else {
            state = auth_repo.record_failure(state, now).await?;
            if AuthStateRepo::is_locked_out(&state, now) {
                return Ok(LoginResult::LockedOut {
                    until: state.lockout_until.unwrap(),
                });
            }
            return Ok(LoginResult::InvalidCredentials);
        };

        if verify_password(&admin.password_hash, password).is_err() {
            state = auth_repo.record_failure(state, now).await?;
            if AuthStateRepo::is_locked_out(&state, now) {
                return Ok(LoginResult::LockedOut {
                    until: state.lockout_until.unwrap(),
                });
            }
            return Ok(LoginResult::InvalidCredentials);
        }

        let _ = auth_repo.reset_failures(state, now).await?;
        let session_repo = SessionRepo::new(self.pool.clone());
        let session = session_repo.create_session(admin.id, now).await?;

        Ok(LoginResult::Success(session))
    }

    pub async fn logout(&self, raw_token: &str, now: NaiveDateTime) -> Result<bool, sqlx::Error> {
        let session_repo = SessionRepo::new(self.pool.clone());
        session_repo.revoke_session(raw_token, now).await
    }
}
