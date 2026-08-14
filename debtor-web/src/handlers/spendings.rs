use std::collections::HashMap;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use chrono::NaiveDate;
use debtor_application::{
    PayerInput, ShareInput, SpendingCursor, SpendingInput, SpendingPageDirection,
};
use debtor_domain::model::Spending;
use tower_sessions::Session;

use super::{
    ExpenseForm,
    auth::{authenticated_shell, csrf, require_auth},
    groups::require_writable_group,
    response::{error_response, map_error, render},
    spending_views::{build_group_template, map_group_template_error, named_allocations},
};
use crate::{
    forms::{CsrfValidatedForm, parse_expense_form},
    state::AppState,
    templates::{ConfirmTemplate, SpendingDetailTemplate},
};

pub(crate) async fn create_spending(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<i64>,
    form: CsrfValidatedForm,
) -> Response {
    save_spending(state, session, id, None, form).await
}

pub(crate) async fn update_spending(
    State(state): State<AppState>,
    session: Session,
    Path((group_id, spending_id)): Path<(i64, i64)>,
    form: CsrfValidatedForm,
) -> Response {
    save_spending(state, session, group_id, Some(spending_id), form).await
}

pub(crate) async fn edit_spending_form(
    State(state): State<AppState>,
    session: Session,
    Path((group_id, spending_id)): Path<(i64, i64)>,
) -> Response {
    if let Err(response) = require_auth(&session).await {
        return response;
    }
    if let Err(response) = require_writable_group(&state, group_id).await {
        return response;
    }
    let spending = match state.spendings.spending(group_id, spending_id).await {
        Ok(value) => value,
        Err(error) => return map_error(error),
    };
    match build_group_template(
        &state,
        &session,
        group_id,
        None,
        Some(&spending),
        None,
        None,
        None,
    )
    .await
    {
        Ok(template) => render(&template),
        Err(error) => map_group_template_error(error),
    }
}

pub(crate) async fn spending_detail(
    State(state): State<AppState>,
    session: Session,
    Path((group_id, spending_id)): Path<(i64, i64)>,
) -> Response {
    if let Err(response) = require_auth(&session).await {
        return response;
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
    let names = members
        .into_iter()
        .map(|(participant, _)| (participant.id, participant.name.to_string()))
        .collect();
    let shell = match authenticated_shell(&state, &session).await {
        Ok(shell) => shell,
        Err(response) => return response,
    };
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
        csrf: match csrf(&session).await {
            Ok(token) => token,
            Err(response) => return response,
        },
        shell,
    })
}

pub(crate) async fn delete_spending_form(
    State(state): State<AppState>,
    session: Session,
    Path((group_id, spending_id)): Path<(i64, i64)>,
) -> Response {
    if let Err(response) = require_auth(&session).await {
        return response;
    }
    if let Err(response) = require_writable_group(&state, group_id).await {
        return response;
    }
    let spending = match state.spendings.spending(group_id, spending_id).await {
        Ok(value) => value,
        Err(error) => return map_error(error),
    };
    let shell = match authenticated_shell(&state, &session).await {
        Ok(shell) => shell,
        Err(response) => return response,
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
        csrf: match csrf(&session).await {
            Ok(token) => token,
            Err(response) => return response,
        },
        shell,
    })
}

pub(crate) async fn delete_spending(
    State(state): State<AppState>,
    session: Session,
    Path((group_id, spending_id)): Path<(i64, i64)>,
    form: CsrfValidatedForm,
) -> Response {
    if let Err(response) = require_auth(&session).await {
        return response;
    }
    if let Err(response) = require_writable_group(&state, group_id).await {
        return response;
    }
    let Some(session_id) = session.id() else {
        return super::response::session_error();
    };
    if let Err(response) = form
        .reserve_and_dispatch(&state.submission_tokens, session_id)
        .await
    {
        return response;
    }
    match state.spendings.delete(group_id, spending_id).await {
        Ok(()) => Redirect::to(&format!("/groups/{group_id}")).into_response(),
        Err(error) => map_error(error),
    }
}

async fn save_spending(
    state: AppState,
    session: Session,
    group_id: i64,
    spending_id: Option<i64>,
    form: CsrfValidatedForm,
) -> Response {
    if let Err(response) = require_auth(&session).await {
        return response;
    }
    let csrf_form = form;
    let form = csrf_form.ordered();
    if let Err(response) = require_writable_group(&state, group_id).await {
        return response;
    }
    let editing = if let Some(id) = spending_id {
        match state.spendings.spending(group_id, id).await {
            Ok(spending) => Some(spending),
            Err(error) => return map_error(error),
        }
    } else {
        None
    };
    let form = match parse_expense_form(form) {
        Ok(value) => value,
        Err(error) => return error_response(error.status, error.message),
    };
    let input = match parse_expense(group_id, &form) {
        Ok(value) => value,
        Err(message) => {
            return form_error(
                &state,
                &session,
                group_id,
                None,
                editing.as_ref(),
                message,
                &form,
            )
            .await;
        }
    };
    let Some(session_id) = session.id() else {
        return super::response::session_error();
    };
    if let Err(response) = csrf_form
        .reserve_and_dispatch(&state.submission_tokens, session_id)
        .await
    {
        return response;
    }
    let result = if let Some(id) = spending_id {
        state.spendings.update_input(id, input).await
    } else {
        state.spendings.create_input(input).await
    };
    match result {
        Ok(_) => Redirect::to(&format!("/groups/{group_id}")).into_response(),
        Err(debtor_application::ApplicationError::Validation(error)) => {
            form_error(
                &state,
                &session,
                group_id,
                None,
                editing.as_ref(),
                error.to_string(),
                &form,
            )
            .await
        }
        Err(error) => map_error(error),
    }
}

async fn form_error(
    state: &AppState,
    session: &Session,
    group_id: i64,
    cursor: Option<SpendingCursor>,
    editing: Option<&Spending>,
    message: String,
    form: &ExpenseForm,
) -> Response {
    match build_group_template(
        state,
        session,
        group_id,
        cursor,
        editing,
        Some(message),
        Some(form),
        None,
    )
    .await
    {
        Ok(template) => (StatusCode::UNPROCESSABLE_ENTITY, render(&template)).into_response(),
        Err(error) => map_group_template_error(error),
    }
}

fn parse_expense(group_id: i64, form: &ExpenseForm) -> Result<SpendingInput, String> {
    let payers = match form.payer_mode.as_str() {
        "single" => PayerInput::Single(form.single_payer_id.unwrap_or_default()),
        "multiple" => PayerInput::Exact(raw_allocations(&form.extra, "payer_")),
        _ => return Err("Choose how many people paid.".into()),
    };
    let shares = match form.split_mode.as_str() {
        "equal" => ShareInput::Equal(raw_ids(&form.extra, "share_")),
        "exact" => ShareInput::Exact(raw_allocations(&form.extra, "exact_")),
        _ => return Err("Choose how the expense is split.".into()),
    };
    Ok(SpendingInput {
        group_id,
        description: form.description.clone(),
        total: form.total.clone(),
        currency: form.currency.clone(),
        spending_type: form.spending_type.clone(),
        spent_date: form.spent_date.clone(),
        payers,
        shares,
    })
}

fn raw_ids(fields: &HashMap<String, String>, prefix: &str) -> Vec<i64> {
    let mut ids = fields
        .keys()
        .filter_map(|key| key.strip_prefix(prefix).and_then(|id| id.parse().ok()))
        .collect::<Vec<_>>();
    ids.sort_unstable();
    ids
}

fn raw_allocations(fields: &HashMap<String, String>, prefix: &str) -> Vec<(i64, String)> {
    let mut values = fields
        .iter()
        .filter_map(|(key, value)| {
            key.strip_prefix(prefix)
                .and_then(|id| id.parse().ok())
                .map(|id| (id, value.clone()))
        })
        .filter(|(_, value)| !value.trim().is_empty())
        .collect::<Vec<_>>();
    values.sort_unstable_by_key(|(id, _)| *id);
    values
}

pub(super) fn parse_cursor(raw: Option<&str>) -> Result<Option<SpendingCursor>, &'static str> {
    let Some(raw) = raw else {
        return Ok(None);
    };
    let mut fields = raw.split(':');
    let direction = match fields.next() {
        Some("older") => SpendingPageDirection::Older,
        Some("newer") => SpendingPageDirection::Newer,
        _ => return Err("Invalid spending history cursor."),
    };
    let spent_date = fields
        .next()
        .and_then(|value| NaiveDate::parse_from_str(value, "%Y-%m-%d").ok())
        .ok_or("Invalid spending history cursor.")?;
    let id = fields
        .next()
        .and_then(|value| value.parse::<i64>().ok())
        .filter(|value| *value > 0)
        .ok_or("Invalid spending history cursor.")?;
    if fields.next().is_some() {
        return Err("Invalid spending history cursor.");
    }
    Ok(Some(SpendingCursor {
        direction,
        spent_date,
        id,
    }))
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::{SpendingPageDirection, parse_cursor};

    #[test]
    fn cursor_parser_accepts_only_strict_direction_date_and_positive_id() {
        let cursor = parse_cursor(Some("older:2026-01-02:7"))
            .expect("valid cursor")
            .expect("cursor value");
        assert_eq!(cursor.direction, SpendingPageDirection::Older);
        assert_eq!(cursor.id, 7);
        assert!(parse_cursor(Some("sideways:2026-01-02:7")).is_err());
        assert!(parse_cursor(Some("older:2026-01-02:0")).is_err());
        assert!(parse_cursor(Some("older:2026-01-02:7:extra")).is_err());
    }
}
