use chrono::NaiveDateTime;
use sea_orm::{ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter};

use crate::auth::auth_state_repo::AuthStateRepo;
use crate::auth::password::verify_password;
use crate::auth::session_repo::{SessionRepo, SessionToken};
use crate::db::entities::admin_users;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoginResult {
    Success(SessionToken),
    InvalidCredentials,
    LockedOut { until: NaiveDateTime },
}

#[derive(Clone)]
pub struct LoginService {
    conn: DatabaseConnection,
}

impl LoginService {
    pub fn new(conn: DatabaseConnection) -> Self {
        Self { conn }
    }

    pub async fn login(
        &self,
        username: &str,
        password: &str,
        now: NaiveDateTime,
    ) -> Result<LoginResult, DbErr> {
        let auth_repo = AuthStateRepo::new(self.conn.clone());
        let mut state = auth_repo.load_or_create(now).await?;

        if AuthStateRepo::is_locked_out(&state, now) {
            return Ok(LoginResult::LockedOut {
                until: state.lockout_until.unwrap(),
            });
        }

        let admin = admin_users::Entity::find()
            .filter(admin_users::Column::Username.eq(username))
            .one(&self.conn)
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
        let session_repo = SessionRepo::new(self.conn.clone());
        let session = session_repo.create_session(admin.id, now).await?;

        Ok(LoginResult::Success(session))
    }

    pub async fn logout(&self, raw_token: &str, now: NaiveDateTime) -> Result<bool, DbErr> {
        let session_repo = SessionRepo::new(self.conn.clone());
        session_repo.revoke_session(raw_token, now).await
    }
}
