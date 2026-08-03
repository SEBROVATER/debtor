use axum::{
    extract::{ConnectInfo, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Redirect, Response},
};
use debtor_application::LoginAdmission;
use tower_sessions::Session;
use uuid::Uuid;

use super::response::error_response;
use crate::{forms::OrderedForm, state::AppState, templates::LoginTemplate};

const AUTH: &str = "authenticated";
const CSRF: &str = "csrf";

pub(crate) async fn root(session: Session) -> Response {
    if authed(&session).await {
        Redirect::to("/groups").into_response()
    } else {
        Redirect::to("/login").into_response()
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
    if !matches_csrf(&session, csrf_value).await {
        return error_response(StatusCode::FORBIDDEN, "Invalid form token.");
    }
    let client = match state.proxy.resolve(peer, &headers) {
        Ok(ip) => ip,
        Err(message) => return error_response(StatusCode::BAD_REQUEST, &message),
    };
    if let LoginAdmission::RetryAfter(seconds) = state.limiter.reserve(client).await {
        let mut response = error_response(
            StatusCode::TOO_MANY_REQUESTS,
            "Too many login attempts. Try again later.",
        );
        response.headers_mut().insert(
            "retry-after",
            HeaderValue::from_str(&seconds.to_string()).unwrap_or(HeaderValue::from_static("300")),
        );
        return response;
    }
    match state.password.verify(password).await {
        Ok(true) => {
            if session.cycle_id().await.is_err()
                || session
                    .insert(CSRF, Uuid::new_v4().to_string())
                    .await
                    .is_err()
                || session.insert(AUTH, true).await.is_err()
                || session.save().await.is_err()
            {
                let _ = session.flush().await;
                return error_response(StatusCode::INTERNAL_SERVER_ERROR, "Session error.");
            }
            state.limiter.reset(client).await;
            Redirect::to("/groups").into_response()
        }
        Ok(false) => login_page(&session, Some("Invalid password.")).await,
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
    if !matches_csrf(&session, fields[0].1).await {
        return error_response(StatusCode::FORBIDDEN, "Invalid form token.");
    }
    match session.flush().await {
        Ok(()) => Redirect::to("/login").into_response(),
        Err(_) => error_response(StatusCode::INTERNAL_SERVER_ERROR, "Session error."),
    }
}

async fn login_page(session: &Session, error: Option<&str>) -> Response {
    let token = csrf(session).await;
    super::response::render(&LoginTemplate {
        error,
        csrf: &token,
    })
}

pub(super) async fn authed(session: &Session) -> bool {
    session
        .get::<bool>(AUTH)
        .await
        .ok()
        .flatten()
        .unwrap_or(false)
}

pub(super) async fn csrf(session: &Session) -> String {
    if let Ok(Some(value)) = session.get::<String>(CSRF).await {
        value
    } else {
        let value = Uuid::new_v4().to_string();
        let _ = session.insert(CSRF, value.clone()).await;
        value
    }
}

pub(super) async fn matches_csrf(session: &Session, supplied: &str) -> bool {
    session
        .get::<String>(CSRF)
        .await
        .ok()
        .flatten()
        .is_some_and(|value| value == supplied)
}
