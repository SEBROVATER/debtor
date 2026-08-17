use axum::{
    extract::{Path, State},
    response::{IntoResponse, Redirect, Response},
};
use tower_sessions::Session;

use super::{
    auth::require_auth,
    groups::require_writable_group,
    response::{error_response, map_error},
    spending_views::{build_group_manage_template, map_group_template_error},
};
use crate::{
    forms::{CsrfValidatedForm, ParticipantForm, parse_participant_form},
    state::AppState,
};

pub(crate) async fn create_group_participant(
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
    let csrf_form = form;
    let form = match parse_participant_form(csrf_form.ordered()) {
        Ok(form) => form,
        Err(error) => return error_response(error.status, error.message),
    };
    let ParticipantForm { name, color, .. } = form;
    if let Err(error) = debtor_application::validate_participant_create(
        &debtor_application::ParticipantCreateInput {
            group_id: id,
            name: name.clone(),
            color: color.clone(),
        },
    ) {
        return render_group_participant_error(
            &state,
            &session,
            id,
            name,
            color,
            error.to_string(),
            application_validation_field(&error),
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
        .create_group_participant(debtor_application::ParticipantCreateInput {
            group_id: id,
            name: name.clone(),
            color: color.clone(),
        })
        .await
    {
        Ok(participant) => Redirect::to(&format!(
            "/groups/{id}/manage?participant={}",
            participant.id
        ))
        .into_response(),
        Err(debtor_application::ApplicationError::Validation(error)) => {
            render_group_participant_error(
                &state,
                &session,
                id,
                name,
                color,
                error.to_string(),
                participant_validation_field(&error),
            )
            .await
        }
        Err(error) => map_error(error),
    }
}

async fn render_group_participant_error(
    state: &AppState,
    session: &Session,
    id: i64,
    name: String,
    color: String,
    error: String,
    invalid_field: Option<String>,
) -> Response {
    match build_group_manage_template(state, session, id, None, Some(error), None, None).await {
        Ok(mut template) => {
            template.create_name = name;
            template.create_color = color;
            template.participant_invalid_field = invalid_field;
            (
                axum::http::StatusCode::UNPROCESSABLE_ENTITY,
                super::response::render(&template),
            )
                .into_response()
        }
        Err(error) => map_group_template_error(error),
    }
}

fn participant_validation_field(error: &debtor_domain::model::ValidationError) -> Option<String> {
    match error {
        debtor_domain::model::ValidationError::Empty { field }
        | debtor_domain::model::ValidationError::TooLong { field, .. } => Some((*field).to_owned()),
        debtor_domain::model::ValidationError::InvalidColor => Some("color".to_owned()),
        _ => None,
    }
}

fn application_validation_field(error: &debtor_application::ApplicationError) -> Option<String> {
    match error {
        debtor_application::ApplicationError::Validation(error) => {
            participant_validation_field(error)
        }
        _ => None,
    }
}
