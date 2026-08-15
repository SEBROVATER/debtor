use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use debtor_domain::currency::Currency;
use tower_sessions::Session;

use super::{
    GroupsQuery, ManageQuery,
    auth::{authenticated_shell, require_auth},
    response::{error_response, map_error, render},
    spending_views::{build_group_manage_template, build_group_template, map_group_template_error},
};
use crate::{
    forms::{CsrfValidatedForm, GroupForm, parse_group_create_form, parse_group_form},
    state::AppState,
    templates::{ConfirmTemplate, GroupRow, GroupsTemplate, SelectOption},
};

pub(crate) async fn groups(
    State(state): State<AppState>,
    session: Session,
    Query(query): Query<GroupsQuery>,
) -> Response {
    if let Err(response) = require_auth(&session).await {
        return response;
    }
    let archived = query.archived.unwrap_or(false);
    match groups_template(&state, &session, archived, "", None).await {
        Ok(template) => render(&template),
        Err(response) => response,
    }
}

pub(crate) async fn create_group(
    State(state): State<AppState>,
    session: Session,
    form: CsrfValidatedForm,
) -> Response {
    if let Err(response) = require_auth(&session).await {
        return response;
    }
    let csrf_form = form;
    let form = match parse_group_create_form(csrf_form.ordered()) {
        Ok(form) => form,
        Err(error) => return error_response(error.status, error.message),
    };
    let name = form.name;
    if let Err(error) =
        debtor_application::validate_group_create(&debtor_application::GroupCreateInput {
            name: name.clone(),
        })
    {
        return render_group_create_error(&state, &session, name, error.to_string()).await;
    }
    let Some(session_id) = session.id() else {
        return super::response::session_error();
    };
    if let Err(response) = csrf_form
        .reserve_and_dispatch(&state.submission_tokens, session_id)
        .await
    {
        return response;
    }
    match state
        .group_mutations
        .create_group(debtor_application::GroupCreateInput { name: name.clone() })
        .await
    {
        Ok(group) => Redirect::to(&format!("/groups/{}/manage", group.id)).into_response(),
        Err(debtor_application::ApplicationError::Validation(error)) => {
            render_group_create_error(&state, &session, name, error.to_string()).await
        }
        Err(error) => map_error(error),
    }
}

pub(crate) async fn group_detail(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<i64>,
    Query(query): Query<super::SpendingQuery>,
) -> Response {
    if let Err(response) = require_auth(&session).await {
        return response;
    }
    let cursor = match super::spendings::parse_cursor(query.cursor.as_deref()) {
        Ok(cursor) => cursor,
        Err(message) => return error_response(StatusCode::BAD_REQUEST, message),
    };
    match build_group_template(&state, &session, id, cursor, None, None, None, None).await {
        Ok(template) => render(&template),
        Err(error) => map_group_template_error(error),
    }
}

pub(crate) async fn group_manage(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<i64>,
    Query(query): Query<ManageQuery>,
) -> Response {
    if let Err(response) = require_auth(&session).await {
        return response;
    }
    let notice = (query.saved.as_deref() == Some("1")).then(|| "Group settings saved.".to_owned());
    match build_group_manage_template(&state, &session, id, None, None, None, notice).await {
        Ok(template) => render(&template),
        Err(error) => map_group_template_error(error),
    }
}

pub(crate) async fn group_transactions(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<i64>,
    Query(query): Query<super::SpendingQuery>,
) -> Response {
    if let Err(response) = require_auth(&session).await {
        return response;
    }
    let cursor = match super::spendings::parse_cursor(query.cursor.as_deref()) {
        Ok(cursor) => cursor,
        Err(message) => return error_response(StatusCode::BAD_REQUEST, message),
    };
    match build_group_template(&state, &session, id, cursor, None, None, None, None).await {
        Ok(mut template) => {
            "transactions".clone_into(&mut template.section);
            render(&template)
        }
        Err(error) => map_group_template_error(error),
    }
}

pub(crate) async fn archive_group(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<i64>,
    form: CsrfValidatedForm,
) -> Response {
    archive(state, session, id, true, form).await
}

pub(crate) async fn restore_group(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<i64>,
    form: CsrfValidatedForm,
) -> Response {
    archive(state, session, id, false, form).await
}

pub(crate) async fn group_edit_form(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<i64>,
) -> Response {
    if let Err(response) = require_auth(&session).await {
        return response;
    }
    if let Err(response) = require_writable_group(&state, id).await {
        return response;
    }
    group_manage(
        State(state),
        session,
        Path(id),
        Query(ManageQuery::default()),
    )
    .await
}

pub(crate) async fn update_group(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<i64>,
    form: CsrfValidatedForm,
) -> Response {
    if let Err(response) = require_auth(&session).await {
        return response;
    }
    let csrf_form = form;
    let form = match parse_group_form(csrf_form.ordered()) {
        Ok(form) => form,
        Err(error) => return error_response(error.status, error.message),
    };
    if let Err(response) = require_writable_group(&state, id).await {
        return response;
    }
    let GroupForm {
        name,
        currency: currency_value,
        ..
    } = form;
    if let Err(error) = debtor_application::validate_group_update(&debtor_application::GroupInput {
        name: name.clone(),
        currency: currency_value.clone(),
    }) {
        return render_group_edit_error(
            &state,
            &session,
            id,
            name,
            currency_value,
            error.to_string(),
            validation_field(&error),
        )
        .await;
    }
    let Some(session_id) = session.id() else {
        return super::response::session_error();
    };
    if let Err(response) = csrf_form
        .reserve_and_dispatch(&state.submission_tokens, session_id)
        .await
    {
        return response;
    }
    match state
        .group_mutations
        .update_group(
            id,
            debtor_application::GroupInput {
                name: name.clone(),
                currency: currency_value.clone(),
            },
        )
        .await
    {
        Ok(_) => Redirect::to(&format!("/groups/{id}/manage?saved=1")).into_response(),
        Err(debtor_application::ApplicationError::Validation(error)) => {
            render_group_edit_error(
                &state,
                &session,
                id,
                name,
                currency_value,
                error.to_string(),
                domain_validation_field(&error),
            )
            .await
        }
        Err(error) => map_error(error),
    }
}

pub(crate) async fn delete_group_form(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<i64>,
) -> Response {
    if let Err(response) = require_auth(&session).await {
        return response;
    }
    match state.groups.group(id).await {
        Ok(group) if !group.is_archived => {
            let shell = match authenticated_shell(&state, &session).await {
                Ok(shell) => shell,
                Err(response) => return response,
            };
            render(&ConfirmTemplate {
                heading: "Delete empty group".into(),
                message: "This permanently deletes the group only if it has no expenses.".into(),
                action: format!("/groups/{id}/delete"),
                cancel: format!("/groups/{id}"),
                csrf: shell.csrf.clone(),
                shell,
            })
        }
        Ok(_) => error_response(StatusCode::CONFLICT, "Archived groups cannot be deleted."),
        Err(error) => map_error(error),
    }
}

pub(crate) async fn delete_group(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<i64>,
    form: CsrfValidatedForm,
) -> Response {
    if let Err(response) = require_auth(&session).await {
        return response;
    }
    if let Err(response) = require_writable_group(&state, id).await {
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
    match state.groups.delete_empty(id).await {
        Ok(()) => Redirect::to("/groups").into_response(),
        Err(error) => map_error(error),
    }
}

pub(super) async fn require_writable_group(state: &AppState, id: i64) -> Result<(), Response> {
    match state.groups.group(id).await {
        Ok(group) if group.is_archived => Err(error_response(
            StatusCode::CONFLICT,
            "Archived groups are read-only.",
        )),
        Ok(_) => Ok(()),
        Err(error) => Err(map_error(error)),
    }
}

async fn groups_template(
    state: &AppState,
    session: &Session,
    archived: bool,
    create_name: &str,
    error: Option<String>,
) -> Result<GroupsTemplate, Response> {
    let items = state
        .groups
        .list_groups(archived)
        .await
        .map_err(map_error)?;
    let shell = authenticated_shell(state, session).await?;
    let mut rows = Vec::with_capacity(items.len());
    for g in items {
        let members = state.participants.members(g.id).await.map_err(map_error)?;
        let active_participants = members
            .iter()
            .filter(|(participant, membership)| membership.is_active && !participant.is_archived)
            .count();
        rows.push(GroupRow {
            id: g.id,
            name: g.name.to_string(),
            currency: g.currency.to_string(),
            active_participants,
        });
    }
    Ok(GroupsTemplate {
        groups: rows,
        csrf: shell.csrf.clone(),
        shell,
        archived,
        create_name: create_name.to_owned(),
        error,
    })
}

async fn render_group_create_error(
    state: &AppState,
    session: &Session,
    name: String,
    error: String,
) -> Response {
    match groups_template(state, session, false, &name, Some(error)).await {
        Ok(template) => (StatusCode::UNPROCESSABLE_ENTITY, render(&template)).into_response(),
        Err(response) => response,
    }
}

async fn render_group_edit_error(
    state: &AppState,
    session: &Session,
    id: i64,
    name: String,
    currency: String,
    error: String,
    invalid_field: Option<String>,
) -> Response {
    match build_group_manage_template(
        state,
        session,
        id,
        Some((name, currency)),
        Some(error),
        invalid_field,
        None,
    )
    .await
    {
        Ok(template) => (StatusCode::UNPROCESSABLE_ENTITY, render(&template)).into_response(),
        Err(error) => map_group_template_error(error),
    }
}

fn validation_field(error: &debtor_application::ApplicationError) -> Option<String> {
    match error {
        debtor_application::ApplicationError::Validation(
            debtor_domain::model::ValidationError::Empty { field }
            | debtor_domain::model::ValidationError::TooLong { field, .. }
            | debtor_domain::model::ValidationError::InvalidField { field },
        ) => Some((*field).to_owned()),
        _ => None,
    }
}

fn domain_validation_field(error: &debtor_domain::model::ValidationError) -> Option<String> {
    match error {
        debtor_domain::model::ValidationError::Empty { field }
        | debtor_domain::model::ValidationError::TooLong { field, .. }
        | debtor_domain::model::ValidationError::InvalidField { field } => {
            Some((*field).to_owned())
        }
        _ => None,
    }
}

pub(super) fn currency_options(selected: &str) -> Vec<SelectOption> {
    Currency::ALL
        .iter()
        .map(|currency| SelectOption {
            value: currency.to_string(),
            label: currency.to_string(),
            selected: currency.to_string() == selected,
        })
        .collect::<Vec<_>>()
}

async fn archive(
    state: AppState,
    session: Session,
    id: i64,
    archived: bool,
    form: CsrfValidatedForm,
) -> Response {
    if let Err(response) = require_auth(&session).await {
        return response;
    }
    if let Err(response) = require_writable_group(&state, id).await {
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
    match state.groups.set_archived(id, archived).await {
        Ok(()) => Redirect::to("/groups").into_response(),
        Err(error) => map_error(error),
    }
}
