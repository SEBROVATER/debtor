use axum::{
    extract::{Path, State},
    response::{IntoResponse, Redirect, Response},
};
use tower_sessions::Session;

use super::{
    auth::{require_auth, require_csrf},
    response::{error_response, map_error},
};
use crate::{
    forms::{OrderedForm, parse_csrf_form, parse_member_form, parse_participant_form},
    state::AppState,
};

pub(crate) async fn add_member(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<i64>,
    form: OrderedForm,
) -> Response {
    if let Err(response) = require_auth(&session).await {
        return response;
    }
    let form = match parse_member_form(form) {
        Ok(form) => form,
        Err(error) => return error_response(error.status, error.message),
    };
    if let Err(response) = require_csrf(&session, &form.csrf).await {
        return response;
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
    match state
        .participants
        .deactivate_member(group_id, participant_id)
        .await
    {
        Ok(()) => Redirect::to(&format!("/groups/{group_id}")).into_response(),
        Err(error) => map_error(error),
    }
}
