use axum::{
    Form,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use tower_sessions::Session;

use super::{
    CsrfForm, MemberForm, ParticipantForm,
    auth::{authed, matches_csrf},
    response::{error_response, map_error},
};
use crate::state::AppState;

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
        return error_response(StatusCode::FORBIDDEN, "Invalid form token.");
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
        return error_response(StatusCode::FORBIDDEN, "Invalid form token.");
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
        return error_response(StatusCode::FORBIDDEN, "Invalid form token.");
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
