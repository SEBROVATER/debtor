//! HTTP request handlers.

use askama::Template;
use axum::{
    Form,
    extract::{Path, Query, State},
    response::{Html, IntoResponse, Redirect, Response},
};
use debtor_application::RateMode;
use debtor_domain::currency::Currency;
use serde::Deserialize;
use tower_sessions::Session;
use uuid::Uuid;

use crate::{
    state::AppState,
    templates::{DebtsTemplate, GroupRow, GroupsTemplate, LoginTemplate, TransferRow},
};

const AUTH: &str = "authenticated";
const CSRF: &str = "csrf";

/// Returns a liveness response without application data.
pub(crate) async fn health() -> &'static str {
    "ok"
}

/// Redirects to the appropriate landing page.
pub(crate) async fn root(session: Session) -> Response {
    if authed(&session).await {
        Redirect::to("/groups").into_response()
    } else {
        Redirect::to("/login").into_response()
    }
}

/// Renders the password gate.
pub(crate) async fn login_form(session: Session) -> Response {
    login_page(&session, None).await
}

/// Verifies the configured password and starts a session.
pub(crate) async fn login(
    State(state): State<AppState>,
    session: Session,
    Form(form): Form<LoginForm>,
) -> Response {
    if !matches_csrf(&session, &form.csrf).await {
        return response(axum::http::StatusCode::FORBIDDEN, "Invalid form token.");
    }
    match state.password.verify(&form.password).await {
        Ok(true) => match session.insert(AUTH, true).await {
            Ok(()) => Redirect::to("/groups").into_response(),
            Err(_) => response(
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "Session error.",
            ),
        },
        Ok(false) | Err(_) => login_page(&session, Some("Invalid password.")).await,
    }
}

/// Clears the current session.
pub(crate) async fn logout(session: Session, Form(form): Form<CsrfForm>) -> Response {
    if !matches_csrf(&session, &form.csrf).await {
        return response(axum::http::StatusCode::FORBIDDEN, "Invalid form token.");
    }
    match session.flush().await {
        Ok(()) => Redirect::to("/login").into_response(),
        Err(_) => response(
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "Session error.",
        ),
    }
}

/// Renders active or archived groups.
pub(crate) async fn groups(
    State(state): State<AppState>,
    session: Session,
    Query(query): Query<GroupsQuery>,
) -> Response {
    if !authed(&session).await {
        return Redirect::to("/login").into_response();
    }
    let archived = query.archived.unwrap_or(false);
    match state.groups.list_groups(archived).await {
        Ok(items) => render(&GroupsTemplate {
            groups: items
                .into_iter()
                .map(|group| GroupRow {
                    id: group.id,
                    name: group.name.to_string(),
                    currency: group.currency.to_string(),
                })
                .collect(),
            csrf: csrf(&session).await,
            archived,
        }),
        Err(_) => response(
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "Unable to load groups.",
        ),
    }
}

/// Creates a group.
pub(crate) async fn create_group(
    State(state): State<AppState>,
    session: Session,
    Form(form): Form<GroupForm>,
) -> Response {
    if !authed(&session).await {
        return Redirect::to("/login").into_response();
    }
    if !matches_csrf(&session, &form.csrf).await {
        return response(axum::http::StatusCode::FORBIDDEN, "Invalid form token.");
    }
    let Ok(currency) = form.currency.parse::<Currency>() else {
        return response(
            axum::http::StatusCode::UNPROCESSABLE_ENTITY,
            "Invalid currency.",
        );
    };
    match state.groups.create_group(form.name, currency).await {
        Ok(_) => Redirect::to("/groups").into_response(),
        Err(error) => response(
            axum::http::StatusCode::UNPROCESSABLE_ENTITY,
            &error.to_string(),
        ),
    }
}

/// Archives a group.
pub(crate) async fn archive_group(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<i64>,
    Form(form): Form<CsrfForm>,
) -> Response {
    archive(state, session, id, true, form).await
}

/// Restores a group.
pub(crate) async fn restore_group(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<i64>,
    Form(form): Form<CsrfForm>,
) -> Response {
    archive(state, session, id, false, form).await
}

/// Renders advisory transfers for all group history.
pub(crate) async fn debts(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<i64>,
    Query(query): Query<DebtQuery>,
) -> Response {
    if !authed(&session).await {
        return Redirect::to("/login").into_response();
    }
    let mode = if query.rate_mode.as_deref() == Some("current") {
        RateMode::Current
    } else {
        RateMode::Historical
    };
    match state.debts.calculate(id, mode).await {
        Ok(result) => render(&DebtsTemplate {
            currency: result.currency.to_string(),
            transfers: result
                .transfers
                .into_iter()
                .map(|transfer| TransferRow {
                    from: transfer.from_participant_id,
                    to: transfer.to_participant_id,
                    amount: transfer.amount.to_string(),
                })
                .collect(),
            mode: if mode == RateMode::Current {
                "current".into()
            } else {
                "historical".into()
            },
            warning: result
                .rates
                .iter()
                .any(|rate| rate.is_stale || rate.is_provisional)
                .then(|| "Some conversions use stale or provisional rates.".into()),
        }),
        Err(debtor_application::ApplicationError::Unavailable(_)) => response(
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            "Exchange rates are temporarily unavailable.",
        ),
        Err(_) => response(axum::http::StatusCode::NOT_FOUND, "Group not found."),
    }
}

async fn archive(
    state: AppState,
    session: Session,
    id: i64,
    archived: bool,
    form: CsrfForm,
) -> Response {
    if !authed(&session).await {
        return Redirect::to("/login").into_response();
    }
    if !matches_csrf(&session, &form.csrf).await {
        return response(axum::http::StatusCode::FORBIDDEN, "Invalid form token.");
    }
    match state.groups.set_archived(id, archived).await {
        Ok(()) => Redirect::to(if archived {
            "/groups"
        } else {
            "/groups?archived=true"
        })
        .into_response(),
        Err(_) => response(axum::http::StatusCode::CONFLICT, "Group cannot be changed."),
    }
}

async fn authed(session: &Session) -> bool {
    session
        .get::<bool>(AUTH)
        .await
        .ok()
        .flatten()
        .unwrap_or(false)
}
async fn csrf(session: &Session) -> String {
    if let Ok(Some(value)) = session.get(CSRF).await {
        value
    } else {
        let value = Uuid::new_v4().to_string();
        let _ = session.insert(CSRF, value.clone()).await;
        value
    }
}
async fn matches_csrf(session: &Session, supplied: &str) -> bool {
    session
        .get::<String>(CSRF)
        .await
        .ok()
        .flatten()
        .is_some_and(|value| value == supplied)
}
async fn login_page(session: &Session, error: Option<&str>) -> Response {
    let token = csrf(session).await;
    render(&LoginTemplate {
        error,
        csrf: &token,
    })
}
fn render(template: &impl Template) -> Response {
    template.render().map_or_else(
        |_| {
            response(
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "Template error.",
            )
        },
        |value| Html(value).into_response(),
    )
}
fn response(status: axum::http::StatusCode, message: &str) -> Response {
    (
        status,
        Html(format!(
            "<!doctype html><html><body><main><p>{message}</p></main></body></html>"
        )),
    )
        .into_response()
}

#[derive(Deserialize)]
pub(crate) struct LoginForm {
    password: String,
    csrf: String,
}
#[derive(Deserialize)]
pub(crate) struct CsrfForm {
    csrf: String,
}
#[derive(Deserialize)]
pub(crate) struct GroupForm {
    name: String,
    currency: String,
    csrf: String,
}
#[derive(Deserialize)]
pub(crate) struct GroupsQuery {
    archived: Option<bool>,
}
#[derive(Deserialize)]
pub(crate) struct DebtQuery {
    rate_mode: Option<String>,
}
