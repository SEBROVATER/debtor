//! HTTP request handlers.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use askama::Template;
use axum::{
    Form,
    extract::{ConnectInfo, Path, Query, State},
    http::{HeaderMap, HeaderValue},
    response::{Html, IntoResponse, Redirect, Response},
};
use chrono::NaiveDate;
use debtor_application::{EqualSpendingCommand, ExactSpendingCommand, LoginAdmission, RateMode};
use debtor_domain::{
    currency::Currency,
    model::{Allocation, Spending, SpendingType},
};
use rust_decimal::Decimal;
use serde::Deserialize;
use tower_sessions::Session;
use uuid::Uuid;

use crate::{
    state::AppState,
    templates::{
        AllocationRow, ConfirmTemplate, DebtsTemplate, ErrorTemplate, ExpenseFormView,
        GroupEditTemplate, GroupRow, GroupTemplate, GroupsTemplate, LoginTemplate, MemberRow,
        ParticipantEditTemplate, ParticipantRow, ParticipantsTemplate, RateRow, SelectOption,
        SpendingDetailTemplate, SpendingRow, TransferRow,
    },
};

const AUTH: &str = "authenticated";
const CSRF: &str = "csrf";

pub(crate) async fn health() -> &'static str {
    "ok"
}

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
    Form(form): Form<LoginForm>,
) -> Response {
    if !matches_csrf(&session, &form.csrf).await {
        return error_response(axum::http::StatusCode::FORBIDDEN, "Invalid form token.");
    }
    let client = match state.proxy.resolve(peer, &headers) {
        Ok(ip) => ip,
        Err(message) => return error_response(axum::http::StatusCode::BAD_REQUEST, &message),
    };
    if let LoginAdmission::RetryAfter(seconds) = state.limiter.reserve(client).await {
        let mut response = error_response(
            axum::http::StatusCode::TOO_MANY_REQUESTS,
            "Too many login attempts. Try again later.",
        );
        response.headers_mut().insert(
            "retry-after",
            HeaderValue::from_str(&seconds.to_string()).unwrap_or(HeaderValue::from_static("300")),
        );
        return response;
    }
    match state.password.verify(&form.password).await {
        Ok(true) => {
            if session.cycle_id().await.is_err() || session.insert(AUTH, true).await.is_err() {
                return error_response(
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "Session error.",
                );
            }
            let _ = session.insert(CSRF, Uuid::new_v4().to_string()).await;
            state.limiter.reset(client).await;
            Redirect::to("/groups").into_response()
        }
        Ok(false) | Err(_) => login_page(&session, Some("Invalid password.")).await,
    }
}

pub(crate) async fn logout(session: Session, Form(form): Form<CsrfForm>) -> Response {
    if !matches_csrf(&session, &form.csrf).await {
        return error_response(axum::http::StatusCode::FORBIDDEN, "Invalid form token.");
    }
    match session.flush().await {
        Ok(()) => Redirect::to("/login").into_response(),
        Err(_) => error_response(
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "Session error.",
        ),
    }
}

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
                .map(|g| GroupRow {
                    id: g.id,
                    name: g.name.to_string(),
                    currency: g.currency.to_string(),
                })
                .collect(),
            csrf: csrf(&session).await,
            archived,
        }),
        Err(_) => error_response(
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "Unable to load groups.",
        ),
    }
}

pub(crate) async fn create_group(
    State(state): State<AppState>,
    session: Session,
    Form(form): Form<GroupForm>,
) -> Response {
    if !authed(&session).await {
        return Redirect::to("/login").into_response();
    }
    if !matches_csrf(&session, &form.csrf).await {
        return error_response(axum::http::StatusCode::FORBIDDEN, "Invalid form token.");
    }
    let Ok(currency) = form.currency.parse::<Currency>() else {
        return error_response(
            axum::http::StatusCode::UNPROCESSABLE_ENTITY,
            "Invalid currency.",
        );
    };
    match state.groups.create_group(form.name, currency).await {
        Ok(_) => Redirect::to("/groups").into_response(),
        Err(error) => error_response(
            axum::http::StatusCode::UNPROCESSABLE_ENTITY,
            &error.to_string(),
        ),
    }
}

pub(crate) async fn group_detail(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<i64>,
) -> Response {
    if !authed(&session).await {
        return Redirect::to("/login").into_response();
    }
    match build_group_template(&state, &session, id, None, None).await {
        Ok(template) => render(&template),
        Err(error) => map_error(error),
    }
}

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
        return error_response(axum::http::StatusCode::FORBIDDEN, "Invalid form token.");
    }
    match state.participants.add_member(id, form.participant_id).await {
        Ok(()) => Redirect::to(&format!("/groups/{id}")).into_response(),
        Err(error) => map_error(error),
    }
}

pub(crate) async fn create_group_participant(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<i64>,
    Form(form): Form<ParticipantForm>,
) -> Response {
    if !authed(&session).await {
        return Redirect::to("/login").into_response();
    }
    if !matches_csrf(&session, &form.csrf).await {
        return error_response(axum::http::StatusCode::FORBIDDEN, "Invalid form token.");
    }
    match state
        .participants
        .create_group_participant(id, form.name, form.color)
        .await
    {
        Ok(_) => Redirect::to(&format!("/groups/{id}")).into_response(),
        Err(error) => map_error(error),
    }
}

pub(crate) async fn deactivate_member(
    State(state): State<AppState>,
    session: Session,
    Path((group_id, participant_id)): Path<(i64, i64)>,
    Form(form): Form<CsrfForm>,
) -> Response {
    if !authed(&session).await {
        return Redirect::to("/login").into_response();
    }
    if !matches_csrf(&session, &form.csrf).await {
        return error_response(axum::http::StatusCode::FORBIDDEN, "Invalid form token.");
    }
    match state
        .participants
        .deactivate_member(group_id, participant_id)
        .await
    {
        Ok(()) => Redirect::to(&format!("/groups/{group_id}")).into_response(),
        Err(error) => map_error(error),
    }
}

pub(crate) async fn create_spending(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<i64>,
    Form(form): Form<ExpenseForm>,
) -> Response {
    save_spending(state, session, id, None, form).await
}

pub(crate) async fn edit_spending_form(
    State(state): State<AppState>,
    session: Session,
    Path((group_id, spending_id)): Path<(i64, i64)>,
) -> Response {
    if !authed(&session).await {
        return Redirect::to("/login").into_response();
    }
    let spending = match state.spendings.spending(group_id, spending_id).await {
        Ok(value) => value,
        Err(error) => return map_error(error),
    };
    match build_group_template(&state, &session, group_id, Some(&spending), None).await {
        Ok(template) => render(&template),
        Err(error) => map_error(error),
    }
}

pub(crate) async fn update_spending(
    State(state): State<AppState>,
    session: Session,
    Path((group_id, spending_id)): Path<(i64, i64)>,
    Form(form): Form<ExpenseForm>,
) -> Response {
    save_spending(state, session, group_id, Some(spending_id), form).await
}

pub(crate) async fn spending_detail(
    State(state): State<AppState>,
    session: Session,
    Path((group_id, spending_id)): Path<(i64, i64)>,
) -> Response {
    if !authed(&session).await {
        return Redirect::to("/login").into_response();
    }
    let group = match state.groups.group(group_id).await {
        Ok(value) => value,
        Err(error) => return map_error(error),
    };
    let spending = match state.spendings.spending(group_id, spending_id).await {
        Ok(value) => value,
        Err(error) => return map_error(error),
    };
    let members = match state.participants.members(group_id).await {
        Ok(value) => value,
        Err(error) => return map_error(error),
    };
    let names: BTreeMap<_, _> = members
        .into_iter()
        .map(|(p, _)| (p.id, p.name.to_string()))
        .collect();
    render(&SpendingDetailTemplate {
        group_id,
        spending_id,
        archived: group.is_archived,
        description: spending.description.as_str().to_owned(),
        total: spending.total.to_string(),
        currency: spending.currency.to_string(),
        spending_type: spending.spending_type.to_string(),
        spent_date: spending.spent_date.to_string(),
        payers: named_allocations(&spending.payers, &names),
        shares: named_allocations(&spending.shares, &names),
        csrf: csrf(&session).await,
    })
}

pub(crate) async fn delete_spending_form(
    State(state): State<AppState>,
    session: Session,
    Path((group_id, spending_id)): Path<(i64, i64)>,
) -> Response {
    if !authed(&session).await {
        return Redirect::to("/login").into_response();
    }
    let spending = match state.spendings.spending(group_id, spending_id).await {
        Ok(value) => value,
        Err(error) => return map_error(error),
    };
    render(&ConfirmTemplate {
        heading: "Delete expense".into(),
        message: format!(
            "Delete '{}' ({} {})?",
            spending.description.as_str(),
            spending.total,
            spending.currency
        ),
        action: format!("/groups/{group_id}/spendings/{spending_id}/delete"),
        cancel: format!("/groups/{group_id}/spendings/{spending_id}"),
        csrf: csrf(&session).await,
    })
}

pub(crate) async fn delete_spending(
    State(state): State<AppState>,
    session: Session,
    Path((group_id, spending_id)): Path<(i64, i64)>,
    Form(form): Form<CsrfForm>,
) -> Response {
    if !authed(&session).await {
        return Redirect::to("/login").into_response();
    }
    if !matches_csrf(&session, &form.csrf).await {
        return error_response(axum::http::StatusCode::FORBIDDEN, "Invalid form token.");
    }
    match state.spendings.delete(group_id, spending_id).await {
        Ok(()) => Redirect::to(&format!("/groups/{group_id}")).into_response(),
        Err(error) => map_error(error),
    }
}

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
                .map(|p| ParticipantRow {
                    id: p.id,
                    name: p.name.to_string(),
                    color: p.color.as_str().to_owned(),
                })
                .collect(),
            csrf: csrf(&session).await,
            archived,
        }),
        Err(_) => error_response(
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "Unable to load participants.",
        ),
    }
}

pub(crate) async fn create_participant(
    State(state): State<AppState>,
    session: Session,
    Form(form): Form<ParticipantForm>,
) -> Response {
    if !authed(&session).await {
        return Redirect::to("/login").into_response();
    }
    if !matches_csrf(&session, &form.csrf).await {
        return error_response(axum::http::StatusCode::FORBIDDEN, "Invalid form token.");
    }
    match state
        .participants
        .create_participant(form.name, form.color)
        .await
    {
        Ok(_) => Redirect::to("/participants").into_response(),
        Err(error) => map_error(error),
    }
}

pub(crate) async fn participant_edit_form(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<i64>,
) -> Response {
    if !authed(&session).await {
        return Redirect::to("/login").into_response();
    }
    match state.participants.participant(id).await {
        Ok(p) if !p.is_archived => render(&ParticipantEditTemplate {
            id,
            name: p.name.to_string(),
            color: p.color.as_str().to_owned(),
            csrf: csrf(&session).await,
            error: None,
        }),
        Ok(_) => error_response(
            axum::http::StatusCode::CONFLICT,
            "Archived participants must be restored before editing.",
        ),
        Err(error) => map_error(error),
    }
}

pub(crate) async fn update_participant(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<i64>,
    Form(form): Form<ParticipantForm>,
) -> Response {
    if !authed(&session).await {
        return Redirect::to("/login").into_response();
    }
    if !matches_csrf(&session, &form.csrf).await {
        return error_response(axum::http::StatusCode::FORBIDDEN, "Invalid form token.");
    }
    match state
        .participants
        .update_participant(id, form.name, form.color)
        .await
    {
        Ok(_) => Redirect::to("/participants").into_response(),
        Err(error) => map_error(error),
    }
}

pub(crate) async fn archive_participant(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<i64>,
    Form(form): Form<CsrfForm>,
) -> Response {
    set_participant_archive(state, session, id, true, form).await
}
pub(crate) async fn restore_participant(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<i64>,
    Form(form): Form<CsrfForm>,
) -> Response {
    set_participant_archive(state, session, id, false, form).await
}

pub(crate) async fn debts(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<i64>,
    Query(query): Query<DebtQuery>,
) -> Response {
    if !authed(&session).await {
        return Redirect::to("/login").into_response();
    }
    let mode = match query.rate_mode.as_deref() {
        None | Some("historical") => RateMode::Historical,
        Some("current") => RateMode::Current,
        Some(_) => {
            return error_response(axum::http::StatusCode::BAD_REQUEST, "Unknown rate mode.");
        }
    };
    let result = match state.debts.calculate(id, mode).await {
        Ok(value) => value,
        Err(error) => return map_error(error),
    };
    let members = match state.participants.members(id).await {
        Ok(value) => value,
        Err(error) => return map_error(error),
    };
    let names: BTreeMap<_, _> = members
        .into_iter()
        .map(|(p, _)| (p.id, p.name.to_string()))
        .collect();
    let warning = result
        .rates
        .iter()
        .any(|r| r.is_stale || r.is_provisional)
        .then(|| "Some conversions use stale or provisional rates.".to_string());
    render(&DebtsTemplate {
        currency: result.currency.to_string(),
        transfers: result
            .transfers
            .into_iter()
            .map(|t| TransferRow {
                from: names
                    .get(&t.from_participant_id)
                    .cloned()
                    .unwrap_or_else(|| format!("Participant {}", t.from_participant_id)),
                to: names
                    .get(&t.to_participant_id)
                    .cloned()
                    .unwrap_or_else(|| format!("Participant {}", t.to_participant_id)),
                amount: t.amount.to_string(),
            })
            .collect(),
        mode: if mode == RateMode::Current {
            "current".into()
        } else {
            "historical".into()
        },
        warning,
        calculated_at: result.calculated_at.to_rfc3339(),
        rates: result
            .rates
            .into_iter()
            .map(|r| RateRow {
                base: r.base.to_string(),
                quote: r.quote.to_string(),
                requested_date: r.requested_date.to_string(),
                effective_date: r.effective_date.to_string(),
                rate: r.rate.to_string(),
                stale: r.is_stale,
                provisional: r.is_provisional,
            })
            .collect(),
    })
}

pub(crate) async fn archive_group(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<i64>,
    Form(form): Form<CsrfForm>,
) -> Response {
    archive(state, session, id, true, form).await
}
pub(crate) async fn restore_group(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<i64>,
    Form(form): Form<CsrfForm>,
) -> Response {
    archive(state, session, id, false, form).await
}

pub(crate) async fn group_edit_form(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<i64>,
) -> Response {
    if !authed(&session).await {
        return Redirect::to("/login").into_response();
    }
    match state.groups.group(id).await {
        Ok(group) if !group.is_archived => render(&GroupEditTemplate {
            id,
            name: group.name.to_string(),
            currency: group.currency.to_string(),
            currencies: Currency::ALL
                .iter()
                .map(|c| SelectOption {
                    value: c.to_string(),
                    label: c.to_string(),
                    selected: *c == group.currency,
                })
                .collect(),
            csrf: csrf(&session).await,
            error: None,
        }),
        Ok(_) => error_response(
            axum::http::StatusCode::CONFLICT,
            "Archived groups are read-only.",
        ),
        Err(error) => map_error(error),
    }
}

pub(crate) async fn update_group(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<i64>,
    Form(form): Form<GroupForm>,
) -> Response {
    if !authed(&session).await {
        return Redirect::to("/login").into_response();
    }
    if !matches_csrf(&session, &form.csrf).await {
        return error_response(axum::http::StatusCode::FORBIDDEN, "Invalid form token.");
    }
    let Ok(currency) = form.currency.parse::<Currency>() else {
        return error_response(
            axum::http::StatusCode::UNPROCESSABLE_ENTITY,
            "Invalid currency.",
        );
    };
    match state.groups.update_group(id, form.name, currency).await {
        Ok(_) => Redirect::to(&format!("/groups/{id}")).into_response(),
        Err(error) => map_error(error),
    }
}

pub(crate) async fn delete_group_form(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<i64>,
) -> Response {
    if !authed(&session).await {
        return Redirect::to("/login").into_response();
    }
    match state.groups.group(id).await {
        Ok(group) if !group.is_archived => render(&ConfirmTemplate {
            heading: "Delete empty group".into(),
            message: "This permanently deletes the group only if it has no expenses.".into(),
            action: format!("/groups/{id}/delete"),
            cancel: format!("/groups/{id}"),
            csrf: csrf(&session).await,
        }),
        Ok(_) => error_response(
            axum::http::StatusCode::CONFLICT,
            "Archived groups cannot be deleted.",
        ),
        Err(error) => map_error(error),
    }
}

pub(crate) async fn delete_group(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<i64>,
    Form(form): Form<CsrfForm>,
) -> Response {
    if !authed(&session).await {
        return Redirect::to("/login").into_response();
    }
    if !matches_csrf(&session, &form.csrf).await {
        return error_response(axum::http::StatusCode::FORBIDDEN, "Invalid form token.");
    }
    match state.groups.delete_empty(id).await {
        Ok(()) => Redirect::to("/groups").into_response(),
        Err(error) => map_error(error),
    }
}

async fn save_spending(
    state: AppState,
    session: Session,
    group_id: i64,
    spending_id: Option<i64>,
    form: ExpenseForm,
) -> Response {
    if !authed(&session).await {
        return Redirect::to("/login").into_response();
    }
    if !matches_csrf(&session, &form.csrf).await {
        return error_response(axum::http::StatusCode::FORBIDDEN, "Invalid form token.");
    }
    let mut member_ids = match state.participants.members(group_id).await {
        Ok(members) => members
            .into_iter()
            .filter(|(p, m)| m.is_active && !p.is_archived)
            .map(|(p, _)| p.id)
            .collect::<Vec<_>>(),
        Err(error) => return map_error(error),
    };
    if let Some(id) = spending_id {
        match state.spendings.spending(group_id, id).await {
            Ok(spending) => member_ids.extend(
                spending
                    .payers
                    .iter()
                    .chain(&spending.shares)
                    .map(|allocation| allocation.participant_id),
            ),
            Err(error) => return map_error(error),
        }
        member_ids.sort_unstable();
        member_ids.dedup();
    }
    let parsed = match parse_expense(group_id, &member_ids, form) {
        Ok(value) => value,
        Err(message) => {
            return match build_group_template(&state, &session, group_id, None, Some(message)).await
            {
                Ok(template) => (
                    axum::http::StatusCode::UNPROCESSABLE_ENTITY,
                    render(&template),
                )
                    .into_response(),
                Err(error) => map_error(error),
            };
        }
    };
    let result = match parsed {
        ParsedExpense::Equal(command) => {
            if let Some(id) = spending_id {
                state.spendings.update_equal(id, command).await
            } else {
                state.spendings.create_equal(command).await
            }
        }
        ParsedExpense::Exact(command) => {
            if let Some(id) = spending_id {
                state.spendings.update_exact(id, command).await
            } else {
                state.spendings.create_exact(command).await
            }
        }
    };
    match result {
        Ok(_) => Redirect::to(&format!("/groups/{group_id}")).into_response(),
        Err(error) => map_error(error),
    }
}

enum ParsedExpense {
    Equal(EqualSpendingCommand),
    Exact(ExactSpendingCommand),
}

fn parse_expense(
    group_id: i64,
    member_ids: &[i64],
    form: ExpenseForm,
) -> Result<ParsedExpense, String> {
    let total = form
        .total
        .parse::<Decimal>()
        .map_err(|_| "Total must be a valid amount.".to_string())?;
    let currency = form
        .currency
        .parse::<Currency>()
        .map_err(|_| "Currency is invalid.".to_string())?;
    let spending_type = form
        .spending_type
        .parse::<SpendingType>()
        .map_err(|_| "Category is invalid.".to_string())?;
    let spent_date = NaiveDate::parse_from_str(&form.spent_date, "%Y-%m-%d")
        .map_err(|_| "Date must be a valid ISO date.".to_string())?;
    let payers = match form.payer_mode.as_str() {
        "single" => vec![Allocation {
            participant_id: form
                .single_payer_id
                .filter(|id| *id > 0)
                .ok_or("Choose who paid.")?,
            amount: total,
        }],
        "multiple" => strict_allocations(member_ids, &form.extra, "payer_", "paid amounts")?,
        _ => return Err("Choose how many people paid.".into()),
    };
    let share_ids = member_ids
        .iter()
        .copied()
        .filter(|id| form.extra.contains_key(&format!("share_{id}")))
        .collect();
    match form.split_mode.as_str() {
        "equal" => Ok(ParsedExpense::Equal(EqualSpendingCommand {
            group_id,
            description: form.description,
            total,
            currency,
            spending_type,
            spent_date,
            payers,
            share_participant_ids: share_ids,
        })),
        "exact" => Ok(ParsedExpense::Exact(ExactSpendingCommand {
            group_id,
            description: form.description,
            total,
            currency,
            spending_type,
            spent_date,
            payers,
            shares: strict_allocations(member_ids, &form.extra, "exact_", "owed amounts")?,
        })),
        _ => Err("Choose how the expense is split.".into()),
    }
}

fn strict_allocations(
    ids: &[i64],
    fields: &HashMap<String, String>,
    prefix: &str,
    field: &str,
) -> Result<Vec<Allocation>, String> {
    let mut seen = BTreeSet::new();
    let mut values = Vec::new();
    for id in ids {
        let raw = fields
            .get(&format!("{prefix}{id}"))
            .map_or("", String::as_str);
        let raw = raw.trim();
        if raw.is_empty() {
            continue;
        }
        if !seen.insert(*id) {
            return Err(format!("A participant appears twice in {field}."));
        }
        let amount = raw
            .parse::<Decimal>()
            .map_err(|_| format!("Every {field} entry must be a valid amount."))?;
        if amount <= Decimal::ZERO {
            return Err(format!("Every {field} entry must be positive."));
        }
        values.push(Allocation {
            participant_id: *id,
            amount,
        });
    }
    Ok(values)
}

async fn build_group_template(
    state: &AppState,
    session: &Session,
    id: i64,
    editing: Option<&Spending>,
    error: Option<String>,
) -> Result<GroupTemplate, debtor_application::ApplicationError> {
    let group = state.groups.group(id).await?;
    let members = state.participants.members(id).await?;
    let all_participants = state.participants.list_participants(false).await?;
    let active_ids: BTreeSet<_> = members
        .iter()
        .filter(|(_, m)| m.is_active)
        .map(|(p, _)| p.id)
        .collect();
    let inactive_ids: BTreeSet<_> = members
        .iter()
        .filter(|(_, m)| !m.is_active)
        .map(|(p, _)| p.id)
        .collect();
    let active_members: Vec<MemberRow> = members
        .iter()
        .filter(|(p, m)| m.is_active && !p.is_archived)
        .map(|(p, _)| member_row(p, true, false))
        .collect();
    let inactive_members: Vec<MemberRow> = members
        .iter()
        .filter(|(p, m)| !m.is_active && !p.is_archived)
        .map(|(p, _)| member_row(p, false, false))
        .collect();
    let available: Vec<MemberRow> = all_participants
        .into_iter()
        .filter(|p| !active_ids.contains(&p.id) && !inactive_ids.contains(&p.id))
        .map(|p| member_row(&p, false, false))
        .collect();
    let spendings = state.spendings.list_spendings(id).await?;
    let mut form_members = active_members.clone();
    if let Some(spending) = editing {
        let historical_ids: BTreeSet<_> = spending
            .payers
            .iter()
            .chain(&spending.shares)
            .map(|allocation| allocation.participant_id)
            .collect();
        for (participant, membership) in &members {
            if historical_ids.contains(&participant.id)
                && !form_members
                    .iter()
                    .any(|member| member.id == participant.id)
            {
                form_members.push(member_row(participant, membership.is_active, false));
            }
        }
    }
    let mut expense = expense_view(state, group.currency, &form_members, editing);
    if editing.is_none() {
        expense.action = format!("/groups/{id}/spendings");
    }
    Ok(GroupTemplate {
        name: group.name.to_string(),
        group_id: id,
        currency: group.currency.to_string(),
        csrf: csrf(session).await,
        members: active_members,
        inactive_members,
        available_participants: available,
        spendings: spendings
            .into_iter()
            .map(|s| SpendingRow {
                id: s.id,
                description: s.description.as_str().to_owned(),
                total: s.total.to_string(),
                currency: s.currency.to_string(),
                spent_date: s.spent_date.to_string(),
            })
            .collect(),
        archived: group.is_archived,
        error,
        create_name: String::new(),
        create_color: "#16697A".into(),
        expense,
    })
}

fn member_row(p: &debtor_domain::model::Participant, active: bool, selected: bool) -> MemberRow {
    MemberRow {
        id: p.id,
        name: p.name.to_string(),
        color: p.color.as_str().to_owned(),
        active,
        archived: p.is_archived,
        selected,
        amount: String::new(),
    }
}

fn expense_view(
    state: &AppState,
    currency: Currency,
    members: &[MemberRow],
    spending: Option<&Spending>,
) -> ExpenseFormView {
    let today = state.clock.now().date_naive().to_string();
    let mut view = ExpenseFormView {
        action: String::new(),
        heading: if spending.is_some() {
            "Edit expense".into()
        } else {
            "Add expense".into()
        },
        submit_label: if spending.is_some() {
            "Save expense".into()
        } else {
            "Add expense".into()
        },
        description: String::new(),
        total: String::new(),
        currency: currency.to_string(),
        currencies: Currency::ALL
            .iter()
            .map(|c| SelectOption {
                value: c.to_string(),
                label: c.to_string(),
                selected: *c == currency,
            })
            .collect(),
        spending_type: "other".into(),
        categories: SpendingType::ALL
            .iter()
            .map(|c| SelectOption {
                value: c.code().into(),
                label: c.code().to_string(),
                selected: *c == SpendingType::Other,
            })
            .collect(),
        spent_date: today,
        payer_mode: "single".into(),
        split_mode: "equal".into(),
        single_payer_id: if members.len() == 1 { members[0].id } else { 0 },
        payer_rows: members.to_vec(),
        share_rows: members
            .iter()
            .map(|m| {
                let mut row = m.clone();
                row.selected = true;
                row
            })
            .collect(),
        exact_rows: members.to_vec(),
        error: None,
    };
    if let Some(s) = spending {
        s.description.as_str().clone_into(&mut view.description);
        view.total = s.total.to_string();
        view.currency = s.currency.to_string();
        view.spending_type = s.spending_type.to_string();
        view.spent_date = s.spent_date.to_string();
        view.action = format!("/groups/{}/spendings/{}", s.group_id, s.id);
        if s.payers.len() == 1 && s.payers[0].amount == s.total {
            view.single_payer_id = s.payers[0].participant_id;
        } else {
            view.payer_mode = "multiple".into();
        }
        if s.shares.iter().all(|a| a.amount > Decimal::ZERO) {
            let ids: Vec<_> = s.shares.iter().map(|a| a.participant_id).collect();
            if let Ok(equal) =
                debtor_domain::expenses::splitting::equal_split(s.total, s.currency, &ids)
            {
                if equal == s.shares {
                    view.split_mode = "equal".into();
                } else {
                    view.split_mode = "exact".into();
                }
            } else {
                view.split_mode = "exact".into();
            }
        }
        let payer_map: BTreeMap<_, _> = s
            .payers
            .iter()
            .map(|a| (a.participant_id, a.amount.to_string()))
            .collect();
        view.payer_rows
            .iter_mut()
            .for_each(|m| m.amount = payer_map.get(&m.id).cloned().unwrap_or_default());
        let share_map: BTreeSet<_> = s.shares.iter().map(|a| a.participant_id).collect();
        view.share_rows
            .iter_mut()
            .for_each(|m| m.selected = share_map.contains(&m.id));
        let exact_map: BTreeMap<_, _> = s
            .shares
            .iter()
            .map(|a| (a.participant_id, a.amount.to_string()))
            .collect();
        view.exact_rows
            .iter_mut()
            .for_each(|m| m.amount = exact_map.get(&m.id).cloned().unwrap_or_default());
    }
    view
}

fn named_allocations(items: &[Allocation], names: &BTreeMap<i64, String>) -> Vec<AllocationRow> {
    items
        .iter()
        .map(|a| AllocationRow {
            participant: names
                .get(&a.participant_id)
                .cloned()
                .unwrap_or_else(|| format!("Participant {}", a.participant_id)),
            amount: a.amount.to_string(),
        })
        .collect()
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
        return error_response(axum::http::StatusCode::FORBIDDEN, "Invalid form token.");
    }
    match state.groups.set_archived(id, archived).await {
        Ok(()) => Redirect::to("/groups").into_response(),
        Err(error) => map_error(error),
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
        return error_response(axum::http::StatusCode::FORBIDDEN, "Invalid form token.");
    }
    match state.participants.set_archived(id, archived).await {
        Ok(()) => Redirect::to("/participants").into_response(),
        Err(error) => map_error(error),
    }
}

async fn login_page(session: &Session, error: Option<&str>) -> Response {
    let token = csrf(session).await;
    render(&LoginTemplate {
        error,
        csrf: &token,
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
    if let Ok(Some(value)) = session.get::<String>(CSRF).await {
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
fn render(template: &impl Template) -> Response {
    template.render().map_or_else(
        |_| {
            error_response(
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "Template error.",
            )
        },
        |value| Html(value).into_response(),
    )
}
fn error_response(status: axum::http::StatusCode, message: &str) -> Response {
    let template = ErrorTemplate { message };
    (status, render(&template)).into_response()
}
fn map_error(error: debtor_application::ApplicationError) -> Response {
    match error {
        debtor_application::ApplicationError::NotFound => {
            error_response(axum::http::StatusCode::NOT_FOUND, "Resource not found.")
        }
        debtor_application::ApplicationError::Conflict => error_response(
            axum::http::StatusCode::CONFLICT,
            "This operation conflicts with preserved history.",
        ),
        debtor_application::ApplicationError::Validation(error) => error_response(
            axum::http::StatusCode::UNPROCESSABLE_ENTITY,
            &error.to_string(),
        ),
        debtor_application::ApplicationError::Unavailable(error) => {
            error_response(axum::http::StatusCode::SERVICE_UNAVAILABLE, &error)
        }
        debtor_application::ApplicationError::Storage(_) => error_response(
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "Storage error.",
        ),
    }
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
pub(crate) struct MemberForm {
    participant_id: i64,
    csrf: String,
}
#[derive(Deserialize)]
pub(crate) struct ExpenseForm {
    description: String,
    total: String,
    currency: String,
    spending_type: String,
    spent_date: String,
    payer_mode: String,
    single_payer_id: Option<i64>,
    split_mode: String,
    csrf: String,
    #[serde(flatten)]
    extra: HashMap<String, String>,
}
