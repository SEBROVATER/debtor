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
    spending_views::{
        build_group_template, build_spending_form_template, map_group_template_error,
        named_allocations,
    },
};
use crate::{
    forms::{CsrfValidatedForm, parse_expense_form},
    session,
    state::AppState,
    templates::{ConfirmTemplate, SpendingDetailTemplate},
};

pub(crate) async fn new_spending_form(
    State(state): State<AppState>,
    session: Session,
    Path(group_id): Path<i64>,
) -> Response {
    if let Err(response) = require_auth(&session).await {
        return response;
    }
    if let Err(response) = require_writable_group(&state, group_id).await {
        return response;
    }
    match build_spending_form_template(&state, &session, group_id, None, None, None, false).await {
        Ok(template) => render(&template),
        Err(error) => map_group_template_error(error),
    }
}

pub(crate) async fn preview_spending(
    State(state): State<AppState>,
    session: Session,
    Path(group_id): Path<i64>,
    form: CsrfValidatedForm,
) -> Response {
    if let Err(response) = require_auth(&session).await {
        return response;
    }
    if let Err(response) = require_writable_group(&state, group_id).await {
        return response;
    }
    let ordered = form.ordered();
    let parsed = match parse_expense_form(ordered.clone()) {
        Ok(value) => value,
        Err(error) => return error_response(error.status, error.message),
    };
    let input = match parse_expense(group_id, &parsed) {
        Ok(value) => value,
        Err(message) => {
            return match build_spending_form_template(
                &state,
                &session,
                group_id,
                Some(&parsed),
                None,
                Some(message),
                false,
            )
            .await
            {
                Ok(template) => {
                    (StatusCode::UNPROCESSABLE_ENTITY, render(&template)).into_response()
                }
                Err(error) => map_group_template_error(error),
            };
        }
    };
    match state.spendings.preview_input(input).await {
        Ok(preview) => {
            let review_fields = ordered
                .0
                .iter()
                .filter(|(key, _)| key != "csrf" && key != "submission_token")
                .cloned()
                .collect();
            if session::set_spending_preview(&session, group_id, review_fields)
                .await
                .is_err()
            {
                return super::response::session_error();
            }
            let mut template = match build_spending_form_template(
                &state,
                &session,
                group_id,
                Some(&parsed),
                None,
                Some("Preview ready. Review the exact Shares before approving.".into()),
                true,
            )
            .await
            {
                Ok(template) => template,
                Err(error) => return map_group_template_error(error),
            };
            for row in &mut template.expense.share_rows {
                row.derived_amount = preview
                    .shares
                    .iter()
                    .find(|allocation| allocation.participant_id == row.id)
                    .map_or_else(String::new, |allocation| allocation.amount.to_string());
            }
            render(&template)
        }
        Err(debtor_application::ApplicationError::Validation(error)) => {
            match build_spending_form_template(
                &state,
                &session,
                group_id,
                Some(&parsed),
                None,
                Some(error.to_string()),
                false,
            )
            .await
            {
                Ok(template) => {
                    (StatusCode::UNPROCESSABLE_ENTITY, render(&template)).into_response()
                }
                Err(error) => map_group_template_error(error),
            }
        }
        Err(error) => map_error(error),
    }
}

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
    match build_spending_form_template(
        &state,
        &session,
        group_id,
        None,
        Some(&spending),
        None,
        false,
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
        details: Vec::new(),
        destructive: true,
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
    let ordered = csrf_form.ordered();
    let review_fields = ordered
        .0
        .iter()
        .filter(|(key, _)| key != "csrf" && key != "submission_token")
        .cloned()
        .collect::<Vec<_>>();
    let form = ordered;
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
    let _approval_guard = session::spending_approval_lock().lock().await;
    if spending_id.is_none() {
        if let Err(response) = reserve_submission_token(&state, &session, &csrf_form).await {
            return response;
        }
        match session::take_matching_spending_preview(&session, group_id, &review_fields).await {
            Ok(true) => {}
            Ok(false) => {
                return crate::handlers::response::submission_token_conflict_for(
                    &format!("/groups/{group_id}/spendings"),
                    false,
                );
            }
            Err(_) => return super::response::session_error(),
        }
    }
    if spending_id.is_some()
        && let Err(response) = reserve_submission_token(&state, &session, &csrf_form).await
    {
        return response;
    }
    let result = if let Some(id) = spending_id {
        state.spendings.update_input(id, input).await
    } else {
        state.spending_mutations.create_spending(input).await
    };
    match result {
        Ok(_) => {
            let destination = if spending_id.is_some() {
                format!("/groups/{group_id}")
            } else {
                format!("/groups/{group_id}/transactions")
            };
            Redirect::to(&destination).into_response()
        }
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

async fn reserve_submission_token(
    state: &AppState,
    session: &Session,
    form: &CsrfValidatedForm,
) -> Result<(), Response> {
    let Some(session_id) = session.id() else {
        return Err(super::response::session_error());
    };
    form.reserve_and_dispatch(&state.submission_tokens, session_id)
        .await
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
        _ => return Err("Choose one Payer.".into()),
    };
    let shares = match form.split_mode.as_str() {
        "proportional" => ShareInput::Proportional(raw_proportional_allocations(&form.extra)?),
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

fn raw_proportional_allocations(
    fields: &HashMap<String, String>,
) -> Result<Vec<(i64, String)>, String> {
    let mut included = fields
        .keys()
        .filter_map(|key| key.strip_prefix("included_")?.parse::<i64>().ok())
        .collect::<Vec<_>>();
    included.sort_unstable();
    included.dedup();
    let mut allocations = Vec::with_capacity(included.len());
    for participant_id in included {
        let Some(weight) = fields.get(&format!("weight_{participant_id}")) else {
            return Err("Each included Participant needs a weight.".into());
        };
        if weight.trim().is_empty() {
            return Err("Each included Participant needs a weight.".into());
        }
        allocations.push((participant_id, weight.clone()));
    }
    Ok(allocations)
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
