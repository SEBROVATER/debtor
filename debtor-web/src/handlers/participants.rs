use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use tower_sessions::Session;

use super::{
    GroupsQuery,
    auth::{csrf, require_auth, require_csrf},
    response::{error_response, map_error, render},
};
use crate::{
    forms::{OrderedForm, parse_csrf_form, parse_participant_form},
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
            csrf: match csrf(&session).await {
                Ok(token) => token,
                Err(response) => return response,
            },
            archived,
        }),
        Err(_) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Unable to load participants.",
        ),
    }
}

pub(crate) async fn create_participant(
    State(state): State<AppState>,
    session: Session,
    form: OrderedForm,
) -> Response {
    if let Err(response) = require_auth(&session).await {
        return response;
    }
    let form = match parse_participant_form(form) {
        Ok(form) => form,
        Err(error) => return error_response(error.status, error.message),
    };
    if let Err(response) = require_csrf(&session, &form.csrf).await {
        return response;
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
    if let Err(response) = require_auth(&session).await {
        return response;
    }
    match state.participants.participant(id).await {
        Ok(p) if !p.is_archived => render(&ParticipantEditTemplate {
            id,
            name: p.name.to_string(),
            color: p.color.as_str().to_owned(),
            csrf: match csrf(&session).await {
                Ok(token) => token,
                Err(response) => return response,
            },
            error: None,
        }),
        Ok(_) => error_response(
            StatusCode::CONFLICT,
            "Archived participants must be restored before editing.",
        ),
        Err(error) => map_error(error),
    }
}

pub(crate) async fn update_participant(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<i64>,
    form: OrderedForm,
) -> Response {
    if let Err(response) = require_auth(&session).await {
        return response;
    }
    let form = match parse_participant_form(form) {
        Ok(form) => form,
        Err(error) => return error_response(error.status, error.message),
    };
    if let Err(response) = require_csrf(&session, &form.csrf).await {
        return response;
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
    form: OrderedForm,
) -> Response {
    set_participant_archive(state, session, id, true, form).await
}

pub(crate) async fn restore_participant(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<i64>,
    form: OrderedForm,
) -> Response {
    set_participant_archive(state, session, id, false, form).await
}

async fn set_participant_archive(
    state: AppState,
    session: Session,
    id: i64,
    archived: bool,
    form: OrderedForm,
) -> Response {
    if let Err(response) = require_auth(&session).await {
        return response;
    }
    let form = match parse_csrf_form(form) {
        Ok(form) => form,
        Err(error) => return error_response(error.status, error.message),
    };
    if let Err(response) = require_csrf(&session, &form.csrf).await {
        return response;
    }
    match state.participants.set_archived(id, archived).await {
        Ok(()) => Redirect::to("/participants").into_response(),
        Err(error) => map_error(error),
    }
}
