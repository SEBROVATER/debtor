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
    spending_views::{
        build_group_manage_template, build_group_template, build_transactions_template,
        map_group_template_error,
    },
};
use crate::{
    forms::{
        CsrfValidatedForm, GroupForm, parse_group_create_form, parse_group_form,
        parse_lifecycle_form,
    },
    session,
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
    let focus_group = if query.notice.as_deref() == Some("restored") {
        match session::take_restore_focus(&session).await {
            Ok(focus) => focus,
            Err(_) => return super::response::session_error(),
        }
    } else {
        None
    };
    let notice = match query.notice.as_deref() {
        Some("archived") => Some("Group archived. History remains readable.".to_owned()),
        Some("restored") => Some("Group restored.".to_owned()),
        Some("deleted") => Some("Group deleted.".to_owned()),
        _ => None,
    };
    match groups_template(&state, &session, archived, "", None, notice, focus_group).await {
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
        Ok(mut template) => {
            template.focus_participant = query.participant;
            template.participant_notice = (query.participant_saved.as_deref() == Some("1"))
                .then_some(query.participant)
                .flatten();
            render(&template)
        }
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
    match build_transactions_template(&state, &session, id, cursor).await {
        Ok(template) => render(&template),
        Err(error) => map_group_template_error(error),
    }
}

pub(crate) async fn archive_group(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<i64>,
    form: CsrfValidatedForm,
) -> Response {
    if let Err(response) = require_auth(&session).await {
        return response;
    }
    if let Err(error) = parse_lifecycle_form(&form.ordered()) {
        return error_response(error.status, error.message);
    }
    if let Err(response) = require_active_group(&state, id).await {
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
    match state.group_mutations.archive_group(id).await {
        Ok(()) => Redirect::to("/groups?notice=archived").into_response(),
        Err(error) => map_error(error),
    }
}

pub(crate) async fn archive_group_form(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<i64>,
) -> Response {
    if let Err(response) = require_auth(&session).await {
        return response;
    }
    if let Err(response) = require_active_group(&state, id).await {
        return response;
    }
    let group = match state.groups.group(id).await {
        Ok(group) => group,
        Err(error) => return map_error(error),
    };
    let shell = match authenticated_shell(&state, &session).await {
        Ok(shell) => shell,
        Err(response) => return response,
    };
    render(&ConfirmTemplate {
        heading: "Archive Group".into(),
        message: format!(
            "Archive {}? This is reversible, and its history will remain readable.",
            group.name
        ),
        action: format!("/groups/{id}/archive"),
        cancel: format!("/groups/{id}/manage#group-archive"),
        csrf: shell.csrf.clone(),
        shell,
        details: Vec::new(),
        destructive: false,
    })
}

pub(crate) async fn restore_group(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<i64>,
    form: CsrfValidatedForm,
) -> Response {
    if let Err(response) = require_auth(&session).await {
        return response;
    }
    if let Err(error) = parse_lifecycle_form(&form.ordered()) {
        return error_response(error.status, error.message);
    }
    if let Err(response) = require_archived_group(&state, id).await {
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
    match state.group_mutations.restore_group(id).await {
        Ok(()) => {
            if session::set_restore_focus(&session, id).await.is_err() {
                return super::response::session_error();
            }
            Redirect::to("/groups?notice=restored").into_response()
        }
        Err(error) => map_error(error),
    }
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
            let spending_page = match state.spendings.spending_page(id, None).await {
                Ok(page) => page,
                Err(error) => return map_error(error),
            };
            if !spending_page.items.is_empty() {
                return error_response(
                    StatusCode::CONFLICT,
                    "Groups with spending history cannot be deleted.",
                );
            }
            let members = match state.participants.members(id).await {
                Ok(members) => members,
                Err(error) => return map_error(error),
            };
            let participant_ids = members
                .iter()
                .map(|(participant, _)| participant.id)
                .collect::<Vec<_>>();
            let shell = match authenticated_shell(&state, &session).await {
                Ok(shell) => shell,
                Err(response) => return response,
            };
            if session::set_group_delete_confirmation(
                &session,
                id,
                participant_ids,
                &shell.submission_token,
            )
            .await
            .is_err()
            {
                return super::response::session_error();
            }
            let mut details = members
                .into_iter()
                .map(|(participant, _)| participant.name.to_string())
                .collect::<Vec<_>>();
            details.sort();
            render(&ConfirmTemplate {
                heading: "Delete Group".into(),
                message: format!(
                    "Permanently delete {}? This cannot be undone and removes these history-free Participants:",
                    group.name
                ),
                action: format!("/groups/{id}/delete"),
                cancel: format!("/groups/{id}/manage#group-delete"),
                csrf: shell.csrf.clone(),
                shell,
                details,
                destructive: true,
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
    if let Err(error) = parse_lifecycle_form(&form.ordered()) {
        return error_response(error.status, error.message);
    }
    if let Err(response) = require_active_group(&state, id).await {
        return response;
    }
    let Some((confirmed_group_id, participant_ids, confirmed_token)) =
        session::group_delete_confirmation(&session)
            .await
            .ok()
            .flatten()
    else {
        return error_response(StatusCode::CONFLICT, "Delete confirmation expired.");
    };
    if confirmed_group_id != id {
        return error_response(StatusCode::CONFLICT, "Delete confirmation expired.");
    }
    if form.submission_token() != Some(confirmed_token.as_str()) {
        return error_response(StatusCode::CONFLICT, "Delete confirmation expired.");
    }
    if session::clear_group_delete_confirmation(&session)
        .await
        .is_err()
    {
        return super::response::session_error();
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
    match state
        .group_mutations
        .delete_empty_group(debtor_application::GroupDeleteInput {
            group_id: id,
            participant_ids,
        })
        .await
    {
        Ok(()) => Redirect::to("/groups?notice=deleted").into_response(),
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

async fn require_active_group(state: &AppState, id: i64) -> Result<(), Response> {
    match state.groups.group(id).await {
        Ok(group) if group.is_archived => Err(error_response(
            StatusCode::CONFLICT,
            "Archived groups are read-only.",
        )),
        Ok(_) => Ok(()),
        Err(error) => Err(map_error(error)),
    }
}

async fn require_archived_group(state: &AppState, id: i64) -> Result<(), Response> {
    match state.groups.group(id).await {
        Ok(group) if !group.is_archived => Err(error_response(
            StatusCode::CONFLICT,
            "Only archived groups can be restored.",
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
    notice: Option<String>,
    focus_group: Option<i64>,
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
            focused: focus_group == Some(g.id),
        });
    }
    Ok(GroupsTemplate {
        groups: rows,
        csrf: shell.csrf.clone(),
        shell,
        archived,
        create_name: create_name.to_owned(),
        error,
        notice,
        focus_group,
    })
}

async fn render_group_create_error(
    state: &AppState,
    session: &Session,
    name: String,
    error: String,
) -> Response {
    match groups_template(state, session, false, &name, Some(error), None, None).await {
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
