use axum::{
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, StatusCode},
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
    templates::ParticipantEditRowTemplate,
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

pub(crate) async fn edit_group_participant_form(
    State(state): State<AppState>,
    session: Session,
    Path((group_id, participant_id)): Path<(i64, i64)>,
) -> Response {
    if let Err(response) = require_auth(&session).await {
        return response;
    }
    if let Err(response) = require_writable_group(&state, group_id).await {
        return response;
    }
    match active_group_participant(&state, group_id, participant_id).await {
        Ok(true) => {}
        Ok(false) => return error_response(StatusCode::NOT_FOUND, "Participant not found."),
        Err(error) => return map_error(error),
    }
    match build_group_manage_template(&state, &session, group_id, None, None, None, None).await {
        Ok(mut template) => {
            set_edit_row(&mut template, participant_id, None, None, None, None);
            template.focus_participant = Some(participant_id);
            super::response::render(&template)
        }
        Err(error) => map_group_template_error(error),
    }
}

pub(crate) async fn update_group_participant(
    State(state): State<AppState>,
    headers: HeaderMap,
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
    let csrf_form = form;
    let enhanced = headers.contains_key("hx-request");
    let form = match parse_participant_form(csrf_form.ordered()) {
        Ok(form) => form,
        Err(error) => return error_response(error.status, error.message),
    };
    let ParticipantForm { name, color, .. } = form;
    let input = debtor_application::ParticipantUpdateInput {
        group_id,
        participant_id,
        name: name.clone(),
        color: color.clone(),
    };
    if let Err(error) = debtor_application::validate_participant_update(&input) {
        return render_group_participant_edit_error(
            &state,
            &session,
            group_id,
            participant_id,
            name,
            color,
            error.to_string(),
            application_validation_field(&error),
            enhanced,
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
    match state.group_mutations.update_group_participant(input).await {
        Ok(participant) => {
            let location = format!(
                "/groups/{group_id}/manage?participant={}&participant_saved=1",
                participant.id
            );
            if enhanced {
                let mut response = StatusCode::OK.into_response();
                if let Ok(value) = HeaderValue::from_str(&location) {
                    response.headers_mut().insert("hx-redirect", value);
                }
                response
            } else {
                Redirect::to(&location).into_response()
            }
        }
        Err(debtor_application::ApplicationError::Validation(error)) => {
            render_group_participant_edit_error(
                &state,
                &session,
                group_id,
                participant_id,
                name,
                color,
                error.to_string(),
                participant_validation_field(&error),
                enhanced,
            )
            .await
        }
        Err(error) => map_error(error),
    }
}

async fn active_group_participant(
    state: &AppState,
    group_id: i64,
    participant_id: i64,
) -> Result<bool, debtor_application::ApplicationError> {
    state.participants.members(group_id).await.map(|members| {
        members.into_iter().any(|(participant, member)| {
            participant.id == participant_id && member.is_active && !participant.is_archived
        })
    })
}

fn set_edit_row(
    template: &mut crate::templates::GroupTemplate,
    participant_id: i64,
    name: Option<String>,
    color: Option<String>,
    error: Option<String>,
    invalid_field: Option<String>,
) {
    if let Some(row) = template
        .members
        .iter_mut()
        .find(|row| row.id == participant_id)
    {
        row.editing = true;
        if let Some(name) = name {
            row.edit_name = name;
        }
        if let Some(color) = color {
            row.edit_color = color;
        }
        row.edit_error = error;
        row.edit_invalid_field = invalid_field;
    }
}

#[allow(clippy::too_many_arguments)]
async fn render_group_participant_edit_error(
    state: &AppState,
    session: &Session,
    group_id: i64,
    participant_id: i64,
    name: String,
    color: String,
    error: String,
    invalid_field: Option<String>,
    enhanced: bool,
) -> Response {
    match build_group_manage_template(state, session, group_id, None, None, None, None).await {
        Ok(mut template) => {
            set_edit_row(
                &mut template,
                participant_id,
                Some(name),
                Some(color),
                Some(error),
                invalid_field,
            );
            template.focus_participant = None;
            if enhanced {
                let Some(member) = template
                    .members
                    .iter()
                    .find(|member| member.id == participant_id)
                    .cloned()
                else {
                    return error_response(StatusCode::NOT_FOUND, "Participant not found.");
                };
                let fragment = ParticipantEditRowTemplate {
                    group_id,
                    csrf: template.csrf,
                    submission_token: template.shell.submission_token,
                    member,
                    focus_row: false,
                };
                let mut response = super::response::render(&fragment);
                *response.status_mut() = StatusCode::UNPROCESSABLE_ENTITY;
                response
            } else {
                (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    super::response::render(&template),
                )
                    .into_response()
            }
        }
        Err(error) => map_group_template_error(error),
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
