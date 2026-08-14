use axum::{
    extract::{Path, State},
    response::{IntoResponse, Redirect, Response},
};
use tower_sessions::Session;

use super::{
    auth::require_auth,
    groups::require_writable_group,
    response::{error_response, map_error},
    spending_views::{ParticipantDraft, build_group_template, map_group_template_error},
};
use crate::{
    forms::{CsrfValidatedForm, ParticipantForm, parse_member_form, parse_participant_form},
    state::AppState,
};

pub(crate) async fn add_member(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<i64>,
    form: CsrfValidatedForm,
) -> Response {
    if let Err(response) = require_auth(&session).await {
        return response;
    }
    let csrf_form = form;
    let form = match parse_member_form(csrf_form.ordered()) {
        Ok(form) => form,
        Err(error) => return error_response(error.status, error.message),
    };
    if let Err(response) = require_writable_group(&state, id).await {
        return response;
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
    match state.participants.add_member(id, form.participant_id).await {
        Ok(()) => Redirect::to(&format!("/groups/{id}")).into_response(),
        Err(error) => map_error(error),
    }
}

pub(crate) async fn create_group_participant(
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
    if let Err(response) = require_writable_group(&state, id).await {
        return response;
    }
    let ParticipantForm { name, color, .. } = form;
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
        .participants
        .create_group_participant(id, name.clone(), color.clone())
        .await
    {
        Ok(_) => Redirect::to(&format!("/groups/{id}")).into_response(),
        Err(debtor_application::ApplicationError::Validation(error)) => match build_group_template(
            &state,
            &session,
            id,
            None,
            None,
            Some(error.to_string()),
            None,
            Some(ParticipantDraft { name, color }),
        )
        .await
        {
            Ok(template) => (
                axum::http::StatusCode::UNPROCESSABLE_ENTITY,
                super::response::render(&template),
            )
                .into_response(),
            Err(error) => map_group_template_error(error),
        },
        Err(error) => map_error(error),
    }
}

pub(crate) async fn deactivate_member(
    State(state): State<AppState>,
    session: Session,
    Path((group_id, participant_id)): Path<(i64, i64)>,
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
    match state
        .participants
        .deactivate_member(group_id, participant_id)
        .await
    {
        Ok(()) => Redirect::to(&format!("/groups/{group_id}")).into_response(),
        Err(error) => map_error(error),
    }
}
