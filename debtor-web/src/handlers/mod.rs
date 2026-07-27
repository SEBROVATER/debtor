//! HTTP request handlers.

use askama::Template;
use axum::{
    Form,
    extract::{Path, Query, State},
    response::{Html, IntoResponse, Redirect, Response},
};
use debtor_application::{EqualSpendingCommand, ExactSpendingCommand, RateMode};
use debtor_domain::currency::Currency;
use debtor_domain::model::{Allocation, SpendingType};
use rust_decimal::Decimal;
use serde::Deserialize;
use tower_sessions::Session;
use uuid::Uuid;

use crate::{
    state::AppState,
    templates::{
        DebtsTemplate, GroupRow, GroupTemplate, GroupsTemplate, LoginTemplate, MemberRow,
        ParticipantRow, ParticipantsTemplate, SpendingRow, TransferRow,
    },
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

/// Renders a group's members and spending history.
pub(crate) async fn group_detail(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<i64>,
) -> Response {
    if !authed(&session).await {
        return Redirect::to("/login").into_response();
    }
    let result = tokio::join!(
        state.groups.group(id),
        state.participants.members(id),
        state.spendings.list_spendings(id),
        state.participants.list_participants(false)
    );
    match result {
        (Ok(group), Ok(members), Ok(spendings), Ok(participants)) => {
            let active_ids: std::collections::BTreeSet<_> = members
                .iter()
                .filter(|(participant, member)| !participant.is_archived && member.is_active)
                .map(|(participant, _)| participant.id)
                .collect();
            render(&GroupTemplate {
                name: group.name.to_string(),
                group_id: group.id,
                currency: group.currency.to_string(),
                csrf: csrf(&session).await,
                members: members
                    .into_iter()
                    .filter(|(participant, member)| !participant.is_archived && member.is_active)
                    .map(|(participant, _)| MemberRow {
                        id: participant.id,
                        name: participant.name.to_string(),
                    })
                    .collect(),
                available_participants: participants
                    .into_iter()
                    .filter(|participant| !active_ids.contains(&participant.id))
                    .map(|participant| MemberRow {
                        id: participant.id,
                        name: participant.name.to_string(),
                    })
                    .collect(),
                spendings: spendings
                    .into_iter()
                    .map(|spending| SpendingRow {
                        id: spending.id,
                        description: spending.description.as_str().to_owned(),
                        total: spending.total.to_string(),
                        currency: spending.currency.to_string(),
                        spent_date: spending.spent_date.to_string(),
                    })
                    .collect(),
                archived: group.is_archived,
            })
        }
        _ => response(axum::http::StatusCode::NOT_FOUND, "Group not found."),
    }
}

/// Adds an active participant to a group.
pub(crate) async fn add_member(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<i64>,
    Form(form): Form<MemberForm>,
) -> Response {
    if !authed(&session).await {
        return Redirect::to("/login").into_response();
    }
    if !matches_csrf(&session, &form.csrf).await {
        return response(axum::http::StatusCode::FORBIDDEN, "Invalid form token.");
    }
    match state.participants.add_member(id, form.participant_id).await {
        Ok(()) => Redirect::to(&format!("/groups/{id}")).into_response(),
        Err(_) => response(
            axum::http::StatusCode::CONFLICT,
            "Participant cannot join this group.",
        ),
    }
}

/// Creates a one-payer equal-split spending from the group page.
pub(crate) async fn create_equal_spending(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<i64>,
    Form(form): Form<EqualSpendingForm>,
) -> Response {
    if !authed(&session).await {
        return Redirect::to("/login").into_response();
    }
    if !matches_csrf(&session, &form.csrf).await {
        return response(axum::http::StatusCode::FORBIDDEN, "Invalid form token.");
    }
    let (Ok(total), Ok(currency), Ok(spending_type), Ok(spent_date)) = (
        form.total.parse::<Decimal>(),
        form.currency.parse::<Currency>(),
        form.spending_type.parse::<SpendingType>(),
        chrono::NaiveDate::parse_from_str(&form.spent_date, "%Y-%m-%d"),
    ) else {
        return response(
            axum::http::StatusCode::UNPROCESSABLE_ENTITY,
            "Invalid spending fields.",
        );
    };
    if form.payer_ids.len() != form.payer_amounts.len() {
        return response(
            axum::http::StatusCode::UNPROCESSABLE_ENTITY,
            "Invalid payer allocations.",
        );
    }
    let payers = form
        .payer_ids
        .into_iter()
        .zip(form.payer_amounts)
        .filter_map(|(participant_id, amount)| {
            let amount = amount.trim();
            (!amount.is_empty())
                .then(|| {
                    amount.parse::<Decimal>().ok().map(|amount| Allocation {
                        participant_id,
                        amount,
                    })
                })
                .flatten()
        })
        .filter(|allocation| allocation.amount > Decimal::ZERO)
        .collect();
    let command = EqualSpendingCommand {
        group_id: id,
        description: form.description,
        total,
        currency,
        spending_type,
        spent_date,
        payers,
        share_participant_ids: form.share_ids,
    };
    match state.spendings.create_equal(command).await {
        Ok(_) => Redirect::to(&format!("/groups/{id}")).into_response(),
        Err(error) => response(
            axum::http::StatusCode::UNPROCESSABLE_ENTITY,
            &error.to_string(),
        ),
    }
}

/// Creates a multiple-payer spending with exact positive owed shares.
pub(crate) async fn create_exact_spending(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<i64>,
    Form(form): Form<ExactSpendingForm>,
) -> Response {
    if !authed(&session).await {
        return Redirect::to("/login").into_response();
    }
    if !matches_csrf(&session, &form.csrf).await {
        return response(axum::http::StatusCode::FORBIDDEN, "Invalid form token.");
    }
    let (Ok(total), Ok(currency), Ok(spending_type), Ok(spent_date)) = (
        form.total.parse::<Decimal>(),
        form.currency.parse::<Currency>(),
        form.spending_type.parse::<SpendingType>(),
        chrono::NaiveDate::parse_from_str(&form.spent_date, "%Y-%m-%d"),
    ) else {
        return response(
            axum::http::StatusCode::UNPROCESSABLE_ENTITY,
            "Invalid spending fields.",
        );
    };
    let Some(payers) = allocations_from_form(form.payer_ids, form.payer_amounts) else {
        return response(
            axum::http::StatusCode::UNPROCESSABLE_ENTITY,
            "Invalid payer allocations.",
        );
    };
    let Some(shares) = allocations_from_form(form.share_ids, form.share_amounts) else {
        return response(
            axum::http::StatusCode::UNPROCESSABLE_ENTITY,
            "Invalid share allocations.",
        );
    };
    let command = ExactSpendingCommand {
        group_id: id,
        description: form.description,
        total,
        currency,
        spending_type,
        spent_date,
        payers,
        shares,
    };
    match state.spendings.create_exact(command).await {
        Ok(_) => Redirect::to(&format!("/groups/{id}")).into_response(),
        Err(error) => response(
            axum::http::StatusCode::UNPROCESSABLE_ENTITY,
            &error.to_string(),
        ),
    }
}

/// Renders active or archived participants.
pub(crate) async fn participants(
    State(state): State<AppState>,
    session: Session,
    Query(query): Query<GroupsQuery>,
) -> Response {
    if !authed(&session).await {
        return Redirect::to("/login").into_response();
    }
    let archived = query.archived.unwrap_or(false);
    match state.participants.list_participants(archived).await {
        Ok(items) => render(&ParticipantsTemplate {
            participants: items
                .into_iter()
                .map(|participant| ParticipantRow {
                    id: participant.id,
                    name: participant.name.to_string(),
                    color: participant.color.as_str().to_owned(),
                })
                .collect(),
            csrf: csrf(&session).await,
            archived,
        }),
        Err(_) => response(
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "Unable to load participants.",
        ),
    }
}

/// Creates a participant.
pub(crate) async fn create_participant(
    State(state): State<AppState>,
    session: Session,
    Form(form): Form<ParticipantForm>,
) -> Response {
    if !authed(&session).await {
        return Redirect::to("/login").into_response();
    }
    if !matches_csrf(&session, &form.csrf).await {
        return response(axum::http::StatusCode::FORBIDDEN, "Invalid form token.");
    }
    match state
        .participants
        .create_participant(form.name, form.color)
        .await
    {
        Ok(_) => Redirect::to("/participants").into_response(),
        Err(error) => response(
            axum::http::StatusCode::UNPROCESSABLE_ENTITY,
            &error.to_string(),
        ),
    }
}

/// Archives a participant.
pub(crate) async fn archive_participant(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<i64>,
    Form(form): Form<CsrfForm>,
) -> Response {
    set_participant_archive(state, session, id, true, form).await
}

/// Restores a participant.
pub(crate) async fn restore_participant(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<i64>,
    Form(form): Form<CsrfForm>,
) -> Response {
    set_participant_archive(state, session, id, false, form).await
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

async fn set_participant_archive(
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
    match state.participants.set_archived(id, archived).await {
        Ok(()) => Redirect::to(if archived {
            "/participants"
        } else {
            "/participants?archived=true"
        })
        .into_response(),
        Err(_) => response(
            axum::http::StatusCode::CONFLICT,
            "Participant cannot be changed.",
        ),
    }
}

fn allocations_from_form(ids: Vec<i64>, amounts: Vec<String>) -> Option<Vec<Allocation>> {
    (ids.len() == amounts.len()).then(|| {
        ids.into_iter()
            .zip(amounts)
            .filter_map(|(participant_id, amount)| {
                let amount = amount.trim();
                (!amount.is_empty()).then(|| {
                    amount.parse::<Decimal>().ok().map(|amount| Allocation {
                        participant_id,
                        amount,
                    })
                })?
            })
            .filter(|allocation| allocation.amount > Decimal::ZERO)
            .collect()
    })
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
pub(crate) struct ParticipantForm {
    name: String,
    color: String,
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
#[derive(Deserialize)]
pub(crate) struct EqualSpendingForm {
    description: String,
    total: String,
    currency: String,
    spending_type: String,
    spent_date: String,
    payer_ids: Vec<i64>,
    payer_amounts: Vec<String>,
    share_ids: Vec<i64>,
    csrf: String,
}
#[derive(Deserialize)]
pub(crate) struct MemberForm {
    participant_id: i64,
    csrf: String,
}
#[derive(Deserialize)]
pub(crate) struct ExactSpendingForm {
    description: String,
    total: String,
    currency: String,
    spending_type: String,
    spent_date: String,
    payer_ids: Vec<i64>,
    payer_amounts: Vec<String>,
    share_ids: Vec<i64>,
    share_amounts: Vec<String>,
    csrf: String,
}
