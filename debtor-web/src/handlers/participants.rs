use axum::{
    Form,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use tower_sessions::Session;

use super::{
    CsrfForm, GroupsQuery, ParticipantForm,
    auth::{authed, csrf, matches_csrf},
    response::{error_response, map_error, render},
};
use crate::{
    state::AppState,
    templates::{ParticipantEditTemplate, ParticipantRow, ParticipantsTemplate},
};

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
            StatusCode::INTERNAL_SERVER_ERROR,
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
        return error_response(StatusCode::FORBIDDEN, "Invalid form token.");
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
    Form(form): Form<ParticipantForm>,
) -> Response {
    if !authed(&session).await {
        return Redirect::to("/login").into_response();
    }
    if !matches_csrf(&session, &form.csrf).await {
        return error_response(StatusCode::FORBIDDEN, "Invalid form token.");
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
        return error_response(StatusCode::FORBIDDEN, "Invalid form token.");
    }
    match state.participants.set_archived(id, archived).await {
        Ok(()) => Redirect::to("/participants").into_response(),
        Err(error) => map_error(error),
    }
}
