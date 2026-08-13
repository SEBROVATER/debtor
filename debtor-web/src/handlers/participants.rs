use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use tower_sessions::Session;

use super::{
    GroupsQuery,
    auth::{authenticated_shell, require_auth},
    response::{error_response, map_error, render},
};
use crate::{
    forms::{CsrfValidatedForm, ParticipantForm, parse_participant_form},
    participant_color::suggested_participant_color,
    state::AppState,
    templates::{ParticipantEditTemplate, ParticipantRow, ParticipantsTemplate},
};

pub(crate) async fn participants(
    State(state): State<AppState>,
    session: Session,
    Query(query): Query<GroupsQuery>,
) -> Response {
    if let Err(response) = require_auth(&session).await {
        return response;
    }
    let archived = query.archived.unwrap_or(false);
    match participants_template(
        &state,
        &session,
        archived,
        "",
        suggested_participant_color(),
        None,
    )
    .await
    {
        Ok(template) => render(&template),
        Err(response) => response,
    }
}

pub(crate) async fn create_participant(
    State(state): State<AppState>,
    session: Session,
    form: CsrfValidatedForm,
) -> Response {
    if let Err(response) = require_auth(&session).await {
        return response;
    }
    let csrf_form = form;
    let form = match parse_participant_form(csrf_form.ordered()) {
        Ok(form) => form,
        Err(error) => return error_response(error.status, error.message),
    };
    let ParticipantForm { name, color, .. } = form;
    if let Err(response) = csrf_form.dispatch() {
        return response;
    }
    match state
        .participants
        .create_participant(name.clone(), color.clone())
        .await
    {
        Ok(_) => Redirect::to("/participants").into_response(),
        Err(debtor_application::ApplicationError::Validation(error)) => {
            render_participant_create_error(&state, &session, name, color, error.to_string()).await
        }
        Err(error) => map_error(error),
    }
}

pub(crate) async fn participant_edit_form(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<i64>,
) -> Response {
    if let Err(response) = require_auth(&session).await {
        return response;
    }
    match participant_edit_template(&state, &session, id, None, None).await {
        Ok(template) => render(&template),
        Err(response) => response,
    }
}

pub(crate) async fn update_participant(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<i64>,
    form: CsrfValidatedForm,
) -> Response {
    if let Err(response) = require_auth(&session).await {
        return response;
    }
    let csrf_form = form;
    let form = match parse_participant_form(csrf_form.ordered()) {
        Ok(form) => form,
        Err(error) => return error_response(error.status, error.message),
    };
    let ParticipantForm { name, color, .. } = form;
    if let Err(response) = csrf_form.dispatch() {
        return response;
    }
    match state
        .participants
        .update_participant(id, name.clone(), color.clone())
        .await
    {
        Ok(_) => Redirect::to("/participants").into_response(),
        Err(debtor_application::ApplicationError::Validation(error)) => {
            render_participant_edit_error(&state, &session, id, name, color, error.to_string())
                .await
        }
        Err(error) => map_error(error),
    }
}

pub(crate) async fn archive_participant(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<i64>,
    form: CsrfValidatedForm,
) -> Response {
    set_participant_archive(state, session, id, true, form).await
}

pub(crate) async fn restore_participant(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<i64>,
    form: CsrfValidatedForm,
) -> Response {
    set_participant_archive(state, session, id, false, form).await
}

async fn set_participant_archive(
    state: AppState,
    session: Session,
    id: i64,
    archived: bool,
    form: CsrfValidatedForm,
) -> Response {
    if let Err(response) = require_auth(&session).await {
        return response;
    }
    if let Err(response) = form.dispatch() {
        return response;
    }
    match state.participants.set_archived(id, archived).await {
        Ok(()) => Redirect::to("/participants").into_response(),
        Err(error) => map_error(error),
    }
}

async fn participants_template(
    state: &AppState,
    session: &Session,
    archived: bool,
    create_name: &str,
    create_color: &str,
    error: Option<String>,
) -> Result<ParticipantsTemplate, Response> {
    let items = state
        .participants
        .list_participants(archived)
        .await
        .map_err(map_error)?;
    let shell = authenticated_shell(state, session).await?;
    Ok(ParticipantsTemplate {
        participants: items
            .into_iter()
            .map(|p| ParticipantRow {
                id: p.id,
                name: p.name.to_string(),
                color: p.color.as_str().to_owned(),
            })
            .collect(),
        csrf: shell.csrf.clone(),
        shell,
        archived,
        create_name: create_name.to_owned(),
        create_color: create_color.to_owned(),
        error,
    })
}

async fn render_participant_create_error(
    state: &AppState,
    session: &Session,
    name: String,
    color: String,
    error: String,
) -> Response {
    match participants_template(state, session, false, &name, &color, Some(error)).await {
        Ok(template) => (StatusCode::UNPROCESSABLE_ENTITY, render(&template)).into_response(),
        Err(response) => response,
    }
}

async fn participant_edit_template(
    state: &AppState,
    session: &Session,
    id: i64,
    draft: Option<(String, String)>,
    error: Option<String>,
) -> Result<ParticipantEditTemplate, Response> {
    let participant = state
        .participants
        .participant(id)
        .await
        .map_err(map_error)?;
    if participant.is_archived {
        return Err(error_response(
            StatusCode::CONFLICT,
            "Archived participants must be restored before editing.",
        ));
    }
    let (name, color) = draft.unwrap_or_else(|| {
        (
            participant.name.to_string(),
            participant.color.as_str().to_owned(),
        )
    });
    let shell = authenticated_shell(state, session).await?;
    Ok(ParticipantEditTemplate {
        id,
        name,
        color,
        csrf: shell.csrf.clone(),
        shell,
        error,
    })
}

async fn render_participant_edit_error(
    state: &AppState,
    session: &Session,
    id: i64,
    name: String,
    color: String,
    error: String,
) -> Response {
    match participant_edit_template(state, session, id, Some((name, color)), Some(error)).await {
        Ok(template) => (StatusCode::UNPROCESSABLE_ENTITY, render(&template)).into_response(),
        Err(response) => response,
    }
}
