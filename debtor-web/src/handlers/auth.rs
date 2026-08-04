use axum::{
    extract::{ConnectInfo, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Redirect, Response},
};
use debtor_application::AuthenticationAttempt;
use tower_sessions::Session;

use super::response::error_response;
use crate::{forms::CsrfValidatedForm, session, state::AppState, templates::LoginTemplate};

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
    form: CsrfValidatedForm,
) -> Response {
    let fields = match form.0.required_fields(&["csrf", "password"]) {
        Ok(fields) => fields,
        Err(message) => return error_response(StatusCode::BAD_REQUEST, message),
    };
    let password = fields
        .iter()
        .find(|(key, _)| *key == "password")
        .map_or("", |(_, value)| *value);
    let client = match state.proxy.resolve(peer, &headers) {
        Ok(ip) => ip,
        Err(message) => return error_response(StatusCode::BAD_REQUEST, &message),
    };
    match state.authentication.attempt(client, password).await {
        Ok(AuthenticationAttempt::RetryAfter(seconds)) => {
            tracing::warn!(
                target: "debtor.auth",
                event = "login_rate_limit_rejected",
                category = "attempt_window",
                count = 1_u64,
            );
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

pub(crate) async fn logout(session: Session, form: CsrfValidatedForm) -> Response {
    let _fields = match form.0.required_fields(&["csrf"]) {
        Ok(fields) => fields,
        Err(message) => return error_response(StatusCode::BAD_REQUEST, message),
    };
    match session::flush(&session).await {
        Ok(()) => Redirect::to("/login").into_response(),
        Err(_) => super::response::session_error(),
    }
}

async fn login_page(session: &Session, error: Option<&str>) -> Response {
    let new_session = session.id().is_none();
    let token = match csrf(session).await {
        Ok(token) => token,
        Err(response) => return response,
    };
    if new_session && session.save().await.is_err() {
        return super::response::session_unavailable();
    }
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

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use std::{collections::HashMap, sync::Arc};

    use time::{Duration, OffsetDateTime};
    use tower_sessions::{
        Session, SessionStore,
        session::{Id, Record},
    };

    use super::login_form;
    use crate::{
        session,
        session_store::{ANONYMOUS_CAPACITY, ReapingMemoryStore},
    };

    #[tokio::test]
    async fn login_page_saves_new_csrf_session_and_reports_capacity_as_503() {
        let store = ReapingMemoryStore::default();
        for _ in 0..ANONYMOUS_CAPACITY {
            let mut record = Record {
                id: Id::default(),
                data: HashMap::default(),
                expiry_date: OffsetDateTime::now_utc() + Duration::hours(1),
            };
            store.create(&mut record).await.expect("fill capacity");
        }
        let session = Session::new(None, Arc::new(store), Some(session::anonymous_expiry()));

        let response = login_form(session).await;
        assert_eq!(
            response.status(),
            axum::http::StatusCode::SERVICE_UNAVAILABLE
        );
    }
}
