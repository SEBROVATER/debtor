use axum::{
    extract::{ConnectInfo, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Redirect, Response},
};
use debtor_application::AuthenticationAttempt;
use tower_sessions::Session;

use super::response::error_response;
use crate::{forms::OrderedForm, session, state::AppState, templates::LoginTemplate};

pub(crate) async fn root(session: Session) -> Response {
    match require_auth(&session).await {
        Ok(()) => Redirect::to("/groups").into_response(),
        Err(response) => response,
    }
}

pub(crate) async fn login_form(session: Session) -> Response {
    login_page(&session, None).await
}

pub(crate) async fn login(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<std::net::SocketAddr>,
    headers: HeaderMap,
    session: Session,
    form: OrderedForm,
) -> Response {
    let fields = match form.required_fields(&["csrf", "password"]) {
        Ok(fields) => fields,
        Err(message) => return error_response(StatusCode::BAD_REQUEST, message),
    };
    let csrf_value = fields
        .iter()
        .find(|(key, _)| *key == "csrf")
        .map_or("", |(_, value)| *value);
    let password = fields
        .iter()
        .find(|(key, _)| *key == "password")
        .map_or("", |(_, value)| *value);
    if let Err(response) = require_csrf(&session, csrf_value).await {
        return response;
    }
    let client = match state.proxy.resolve(peer, &headers) {
        Ok(ip) => ip,
        Err(message) => return error_response(StatusCode::BAD_REQUEST, &message),
    };
    match state.authentication.attempt(client, password).await {
        Ok(AuthenticationAttempt::RetryAfter(seconds)) => {
            let mut response = error_response(
                StatusCode::TOO_MANY_REQUESTS,
                "Too many login attempts. Try again later.",
            );
            response.headers_mut().insert(
                "retry-after",
                HeaderValue::from_str(&seconds.to_string())
                    .unwrap_or(HeaderValue::from_static("300")),
            );
            response
        }
        Ok(AuthenticationAttempt::Authenticated) => {
            if session::establish(&session).await.is_err() {
                let _ = session::flush(&session).await;
                return super::response::session_error();
            }
            state.authentication.complete_login(client).await;
            Redirect::to("/groups").into_response()
        }
        Ok(AuthenticationAttempt::InvalidPassword) => {
            login_page(&session, Some("Invalid password.")).await
        }
        Err(_) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Authentication service unavailable.",
        ),
    }
}

pub(crate) async fn logout(session: Session, form: OrderedForm) -> Response {
    let fields = match form.required_fields(&["csrf"]) {
        Ok(fields) => fields,
        Err(message) => return error_response(StatusCode::BAD_REQUEST, message),
    };
    if let Err(response) = require_csrf(&session, fields[0].1).await {
        return response;
    }
    match session::flush(&session).await {
        Ok(()) => Redirect::to("/login").into_response(),
        Err(_) => super::response::session_error(),
    }
}

async fn login_page(session: &Session, error: Option<&str>) -> Response {
    let token = match csrf(session).await {
        Ok(token) => token,
        Err(response) => return response,
    };
    super::response::render(&LoginTemplate {
        error,
        csrf: &token,
    })
}

pub(super) async fn require_auth(session: &Session) -> Result<(), Response> {
    match session::authenticated(session).await {
        Ok(true) => Ok(()),
        Ok(false) => Err(Redirect::to("/login").into_response()),
        Err(_) => Err(super::response::session_error()),
    }
}

pub(super) async fn csrf(session: &Session) -> Result<String, Response> {
    session::csrf_token(session)
        .await
        .map_err(|_| super::response::session_error())
}

pub(super) async fn require_csrf(session: &Session, supplied: &str) -> Result<(), Response> {
    match session::matches_csrf(session, supplied).await {
        Ok(true) => Ok(()),
        Ok(false) => Err(error_response(StatusCode::FORBIDDEN, "Invalid form token.")),
        Err(_) => Err(super::response::session_error()),
    }
}
