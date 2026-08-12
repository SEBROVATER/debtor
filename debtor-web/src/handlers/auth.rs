use axum::{
    extract::{ConnectInfo, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Redirect, Response},
};
use debtor_application::AuthenticationAttempt;
use tower_sessions::Session;

use super::response::error_response;
use crate::{
    forms::CsrfValidatedForm, session, state::AppState, submission_tokens::ReserveError,
    templates::LoginTemplate,
};

pub(crate) async fn root(session: Session) -> Response {
    match require_auth(&session).await {
        Ok(()) => Redirect::to("/groups").into_response(),
        Err(response) => response,
    }
}

pub(crate) async fn login_form(State(state): State<AppState>, session: Session) -> Response {
    match session::authenticated(&session).await {
        Ok(true) => Redirect::to("/groups").into_response(),
        Ok(false) => login_page(&state, &session, None, true).await,
        Err(_) => super::response::session_error(),
    }
}

#[allow(clippy::too_many_lines)]
pub(crate) async fn login(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<std::net::SocketAddr>,
    headers: HeaderMap,
    session: Session,
    form: CsrfValidatedForm,
) -> Response {
    match session::authenticated(&session).await {
        Ok(true) => return Redirect::to("/groups").into_response(),
        Ok(false) => {}
        Err(_) => return super::response::login_session_error(),
    }
    let ordered = form.ordered();
    let mut csrf_seen = false;
    let mut password = None;
    let mut submission_token = None;
    for (key, value) in &ordered.0 {
        match key.as_str() {
            "csrf" if !csrf_seen => csrf_seen = true,
            "password" if password.is_none() => password = Some(value.as_str()),
            "submission_token" if submission_token.is_none() => {
                submission_token = Some(value.as_str());
            }
            _ => {
                return super::response::login_error_response(
                    StatusCode::BAD_REQUEST,
                    "Malformed form submission.",
                );
            }
        }
    }
    let Some(password) = password else {
        return super::response::login_error_response(
            StatusCode::BAD_REQUEST,
            "Malformed form submission.",
        );
    };
    let Some(submission_token) = submission_token else {
        return super::response::login_token_conflict();
    };
    if !csrf_seen {
        return super::response::login_error_response(
            StatusCode::BAD_REQUEST,
            "Malformed form submission.",
        );
    }
    let client = match state.proxy.resolve(peer, &headers) {
        Ok(ip) => ip,
        Err(message) => {
            return super::response::login_error_response(StatusCode::BAD_REQUEST, &message);
        }
    };
    let Some(session_id) = session.id() else {
        return super::response::login_session_error();
    };
    match state
        .submission_tokens
        .reserve_and_dispatch(session_id, submission_token, || {
            form.dispatch().map_err(|_| ())
        })
        .await
    {
        Ok(()) => {}
        Err(ReserveError::Conflict) => return super::response::login_token_conflict(),
        Err(ReserveError::Deadline) => return super::response::login_timeout(),
    }
    match state.authentication.attempt(client, password).await {
        Ok(AuthenticationAttempt::RetryAfter(seconds)) => {
            tracing::warn!(
                target: "debtor.auth",
                event = "login_rate_limit_rejected",
                category = "attempt_window",
                count = 1_u64,
            );
            let response = login_page(
                &state,
                &session,
                Some("Too many login attempts. Try again later."),
                true,
            )
            .await;
            if response.status().is_server_error() {
                return response;
            }
            let mut response =
                super::response::with_status(response, StatusCode::TOO_MANY_REQUESTS);
            response.headers_mut().insert(
                "retry-after",
                HeaderValue::from_str(&seconds.to_string())
                    .unwrap_or(HeaderValue::from_static("300")),
            );
            response
        }
        Ok(AuthenticationAttempt::Authenticated) => {
            if session::establish(&session).await.is_err() {
                if session::flush(&session).await.is_err() {
                    return super::response::login_session_error();
                }
                let response = login_page(
                    &state,
                    &session,
                    Some("Sign-in is temporarily unavailable. Try again."),
                    true,
                )
                .await;
                return super::response::with_status(response, StatusCode::SERVICE_UNAVAILABLE);
            }
            state.authentication.complete_login(client).await;
            Redirect::to("/groups").into_response()
        }
        Ok(AuthenticationAttempt::InvalidPassword) => {
            login_page(&state, &session, Some("Invalid password."), true).await
        }
        Err(_) => super::response::with_status(
            login_page(
                &state,
                &session,
                Some("Authentication service unavailable."),
                true,
            )
            .await,
            StatusCode::SERVICE_UNAVAILABLE,
        ),
    }
}

pub(crate) async fn logout(session: Session, form: CsrfValidatedForm) -> Response {
    let _fields = match form.ordered().required_fields(&["csrf"]) {
        Ok(fields) => fields,
        Err(message) => return error_response(StatusCode::BAD_REQUEST, message),
    };
    if let Err(response) = form.dispatch() {
        return response;
    }
    match session::flush(&session).await {
        Ok(()) => Redirect::to("/login").into_response(),
        Err(_) => super::response::session_error(),
    }
}

async fn login_page(
    state: &AppState,
    session: &Session,
    error: Option<&str>,
    focus_heading: bool,
) -> Response {
    let Ok(token) = csrf(session).await else {
        return super::response::login_session_error();
    };
    session.set_expiry(Some(session::anonymous_expiry()));
    if session.save().await.is_err() {
        return super::response::login_session_unavailable();
    }
    let Some(session_id) = session.id() else {
        return super::response::login_session_unavailable();
    };
    let Ok(submission_token) = state.submission_tokens.issue(session_id).await else {
        if session::flush(session).await.is_err() {
            return super::response::login_session_error();
        }
        return super::response::login_session_unavailable();
    };
    super::response::render(&LoginTemplate {
        error,
        csrf: &token,
        submission_token: &submission_token,
        focus_heading,
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

    use axum::extract::State;
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

        let test_state = crate::handlers::test_support::state(false);
        for session_id in 0..crate::submission_tokens::ANONYMOUS_CAPACITY {
            test_state
                .app
                .submission_tokens
                .issue(Id(session_id as i128))
                .await
                .expect("fill token capacity");
        }
        let state = test_state.app;
        let response = login_form(State(state), session).await;
        assert_eq!(
            response.status(),
            axum::http::StatusCode::SERVICE_UNAVAILABLE
        );
    }
}
