use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Redirect, Response},
};
use serde::Deserialize;
use tower_sessions::Session;

use super::{
    auth::require_auth,
    groups::require_writable_group,
    response::{error_response, map_error},
    spending_views::{build_group_manage_template, map_group_template_error},
};
use crate::{
    forms::{CsrfValidatedForm, ParticipantForm, parse_participant_form},
    session,
    state::AppState,
    templates::{ArchivedParticipantsTemplate, ConfirmTemplate, ParticipantEditRowTemplate},
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

pub(crate) async fn archive_group_participant_form(
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
    let members = match state.participants.members(group_id).await {
        Ok(members) => members,
        Err(error) => return map_error(error),
    };
    let Some((participant, _)) = members.into_iter().find(|(participant, member)| {
        participant.id == participant_id && member.is_active && !participant.is_archived
    }) else {
        return error_response(StatusCode::NOT_FOUND, "Participant not found.");
    };
    match state
        .debts
        .calculate(group_id, debtor_application::RateMode::Historical)
        .await
    {
        Ok(result) if matches!(result.balances.get(&participant_id), Some(balance) if balance.is_zero()) =>
            {}
        Ok(_) => {
            return error_response(
                StatusCode::CONFLICT,
                "Archive requires an exactly zero Historical Balance.",
            );
        }
        Err(error) => return map_error(error),
    }
    let group = match state.groups.group(group_id).await {
        Ok(group) => group,
        Err(error) => return map_error(error),
    };
    let shell = match super::auth::authenticated_shell(&state, &session).await {
        Ok(shell) => shell,
        Err(response) => return response,
    };
    super::response::render(&ConfirmTemplate {
        heading: "Archive Participant".into(),
        message: format!(
            "Archive {} from {}? This is reversible, removes this identity from new allocations, and preserves all history.",
            participant.name, group.name
        ),
        action: format!("/groups/{group_id}/participants/{participant_id}/archive"),
        cancel: format!("/groups/{group_id}/manage#participant-{participant_id}-archive"),
        csrf: shell.csrf.clone(),
        shell,
        details: Vec::new(),
        destructive: false,
        facts: Vec::new(),
        focus_id: "confirm-heading".into(),
    })
}

pub(crate) async fn archive_group_participant(
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
    if let Err(error) = crate::forms::parse_lifecycle_form(&form.ordered()) {
        return error_response(error.status, error.message);
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
        .archive_group_participant(group_id, participant_id)
        .await
    {
        Ok(()) => Redirect::to(&format!("/groups/{group_id}/manage?participant_archived=1"))
            .into_response(),
        Err(_) => archive_failure_response(group_id, participant_id),
    }
}

#[derive(Deserialize, Default)]
pub(crate) struct RestoreQuery {
    restore: Option<String>,
}

pub(crate) async fn archived_group_participants(
    State(state): State<AppState>,
    session: Session,
    Path(group_id): Path<i64>,
    Query(query): Query<RestoreQuery>,
) -> Response {
    if let Err(response) = require_auth(&session).await {
        return response;
    }
    let group = match state.groups.group(group_id).await {
        Ok(group) => group,
        Err(error) => return map_error(error),
    };
    let members = match state.participants.members(group_id).await {
        Ok(members) => members
            .into_iter()
            .filter(|(participant, _)| participant.is_archived)
            .map(|(participant, member)| crate::templates::MemberRow {
                id: participant.id,
                name: participant.name.to_string(),
                color: participant.color.as_str().to_owned(),
                active: member.is_active,
                archived: true,
                payer_allowed: false,
                share_allowed: false,
                selected: false,
                allocation_error: None,
                amount: String::new(),
                derived_amount: String::new(),
                editing: false,
                edit_name: participant.name.to_string(),
                edit_color: participant.color.as_str().to_owned(),
                edit_error: None,
                edit_invalid_field: None,
                historical_balance: None,
                archive_eligibility: crate::templates::ArchiveEligibility::RatesUnavailable,
            })
            .collect(),
        Err(error) => return map_error(error),
    };
    let shell = match super::auth::authenticated_shell(&state, &session).await {
        Ok(shell) => shell,
        Err(response) => return response,
    };
    let restore_nonce = query.restore.as_deref().unwrap_or_default();
    let Ok(restore_notice) =
        session::take_participant_restore_notice(&session, group_id, restore_nonce).await
    else {
        return super::response::session_error();
    };
    super::response::render(&ArchivedParticipantsTemplate {
        group_id,
        group_name: group.name.to_string(),
        members,
        shell,
        group_archived: group.is_archived,
        focus_participant: restore_notice
            .and_then(|(participant_id, succeeded)| (!succeeded).then_some(participant_id)),
        restore_error: restore_notice.and_then(|(_, succeeded)| {
            (!succeeded)
                .then(|| "Participant was not restored. Reopen this page to retry.".to_owned())
        }),
    })
}

pub(crate) async fn restore_group_participant(
    State(state): State<AppState>,
    session: Session,
    Path((group_id, participant_id)): Path<(i64, i64)>,
    headers: HeaderMap,
    form: CsrfValidatedForm,
) -> Response {
    if let Err(response) = require_auth(&session).await {
        return response;
    }
    if let Err(response) = require_writable_group(&state, group_id).await {
        return response;
    }
    if let Err(error) = crate::forms::parse_lifecycle_form(&form.ordered()) {
        return error_response(error.status, error.message);
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
        .restore_group_participant(group_id, participant_id)
        .await
    {
        Ok(()) => {
            let Ok(nonce) =
                session::set_participant_restore_focus(&session, group_id, participant_id).await
            else {
                return super::response::session_error();
            };
            let destination = format!("/groups/{group_id}/manage?restore={nonce}");
            if headers.contains_key("hx-request") {
                return [("HX-Redirect", destination)].into_response();
            }
            Redirect::to(&destination).into_response()
        }
        Err(_) if headers.contains_key("hx-request") => (
            StatusCode::CONFLICT,
            format!(
                "<p id=\"participant-{participant_id}-restore-status\" class=\"mutation-status warning\" role=\"status\" aria-live=\"polite\">Participant was not restored. Reopen this page to retry.</p>"
            ),
        )
            .into_response(),
        Err(_) => {
            let Ok(nonce) = session::set_participant_restore_failure_focus(
                &session,
                group_id,
                participant_id,
            )
            .await
            else {
                return super::response::session_error();
            };
            Redirect::to(&format!("/groups/{group_id}/participants/archived?restore={nonce}"))
                .into_response()
        }
    }
}

fn archive_failure_response(group_id: i64, participant_id: i64) -> Response {
    Redirect::to(&format!(
        "/groups/{group_id}/manage?participant_archive_failed=1#participant-{participant_id}-archive"
    ))
    .into_response()
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

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use axum::http::{StatusCode, header};

    use super::archive_failure_response;

    #[test]
    fn archive_failure_returns_to_the_invoking_control_with_a_safe_status() {
        let response = archive_failure_response(7, 11);

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(
            response
                .headers()
                .get(header::LOCATION)
                .expect("archive failure location"),
            "/groups/7/manage?participant_archive_failed=1#participant-11-archive"
        );
    }
}
