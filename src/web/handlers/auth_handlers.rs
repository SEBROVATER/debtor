use chrono::NaiveDateTime;

use crate::app::state::AppState;
use crate::auth::login_service::{LoginResult, LoginService};
use crate::web::csrf::{CsrfToken, validate_csrf};
use crate::web::error::AppError;
use crate::web::router::RouteMethod;

#[derive(Debug, Clone)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    pub csrf_token: Option<CsrfToken>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoginResponse {
    RedirectToDashboard,
    InvalidCredentials,
    LockedOut { until: NaiveDateTime },
}

pub async fn handle_login(
    state: &AppState,
    request: LoginRequest,
    expected_csrf: Option<CsrfToken>,
    now: NaiveDateTime,
) -> Result<LoginResponse, AppError> {
    validate_csrf(
        RouteMethod::Post,
        request.csrf_token.as_ref(),
        expected_csrf.as_ref(),
    )?;

    let service = LoginService::new(state.db.clone());
    let result = service
        .login(&request.username, &request.password, now)
        .await?;

    Ok(match result {
        LoginResult::Success(_) => LoginResponse::RedirectToDashboard,
        LoginResult::InvalidCredentials => LoginResponse::InvalidCredentials,
        LoginResult::LockedOut { until } => LoginResponse::LockedOut { until },
    })
}

pub async fn handle_logout(
    state: &AppState,
    raw_token: &str,
    now: NaiveDateTime,
) -> Result<bool, AppError> {
    let service = LoginService::new(state.db.clone());
    let revoked = service.logout(raw_token, now).await?;
    Ok(revoked)
}
