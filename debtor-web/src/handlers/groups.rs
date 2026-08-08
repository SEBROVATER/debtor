use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use debtor_domain::currency::Currency;
use tower_sessions::Session;

use super::{
    GroupsQuery,
    auth::{csrf, require_auth},
    response::{error_response, map_error, render},
    spending_views::{build_group_template, map_group_template_error},
};
use crate::{
    forms::{CsrfValidatedForm, GroupForm, parse_group_form},
    state::AppState,
    templates::{ConfirmTemplate, GroupEditTemplate, GroupRow, GroupsTemplate, SelectOption},
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
    match groups_template(&state, &session, archived, "", "USD", None).await {
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
    let form = match parse_group_form(form.into_inner()) {
        Ok(form) => form,
        Err(error) => return error_response(error.status, error.message),
    };
    let GroupForm {
        name,
        currency: currency_value,
        ..
    } = form;
    let Ok(currency) = currency_value.parse::<Currency>() else {
        return render_group_create_error(
            &state,
            &session,
            name,
            currency_value,
            "Invalid currency.".into(),
        )
        .await;
    };
    match state.groups.create_group(name.clone(), currency).await {
        Ok(_) => Redirect::to("/groups").into_response(),
        Err(debtor_application::ApplicationError::Validation(error)) => {
            render_group_create_error(&state, &session, name, currency_value, error.to_string())
                .await
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

pub(crate) async fn archive_group(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<i64>,
    form: CsrfValidatedForm,
) -> Response {
    archive(state, session, id, true, form).await
}

pub(crate) async fn restore_group(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<i64>,
    form: CsrfValidatedForm,
) -> Response {
    archive(state, session, id, false, form).await
}

pub(crate) async fn group_edit_form(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<i64>,
) -> Response {
    if let Err(response) = require_auth(&session).await {
        return response;
    }
    match group_edit_template(&state, &session, id, None, None).await {
        Ok(template) => render(&template),
        Err(response) => response,
    }
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
    let form = match parse_group_form(form.into_inner()) {
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
    let Ok(currency) = currency_value.parse::<Currency>() else {
        return render_group_edit_error(
            &state,
            &session,
            id,
            name,
            currency_value,
            "Invalid currency.".into(),
        )
        .await;
    };
    match state.groups.update_group(id, name.clone(), currency).await {
        Ok(_) => Redirect::to(&format!("/groups/{id}")).into_response(),
        Err(debtor_application::ApplicationError::Validation(error)) => {
            render_group_edit_error(
                &state,
                &session,
                id,
                name,
                currency_value,
                error.to_string(),
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
        Ok(group) if !group.is_archived => render(&ConfirmTemplate {
            heading: "Delete empty group".into(),
            message: "This permanently deletes the group only if it has no expenses.".into(),
            action: format!("/groups/{id}/delete"),
            cancel: format!("/groups/{id}"),
            csrf: match csrf(&session).await {
                Ok(token) => token,
                Err(response) => return response,
            },
        }),
        Ok(_) => error_response(StatusCode::CONFLICT, "Archived groups cannot be deleted."),
        Err(error) => map_error(error),
    }
}

pub(crate) async fn delete_group(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<i64>,
    _form: CsrfValidatedForm,
) -> Response {
    if let Err(response) = require_auth(&session).await {
        return response;
    }
    if let Err(response) = require_writable_group(&state, id).await {
        return response;
    }
    match state.groups.delete_empty(id).await {
        Ok(()) => Redirect::to("/groups").into_response(),
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

async fn groups_template(
    state: &AppState,
    session: &Session,
    archived: bool,
    create_name: &str,
    create_currency: &str,
    error: Option<String>,
) -> Result<GroupsTemplate, Response> {
    let items = state
        .groups
        .list_groups(archived)
        .await
        .map_err(map_error)?;
    let csrf = csrf(session).await?;
    Ok(GroupsTemplate {
        groups: items
            .into_iter()
            .map(|g| GroupRow {
                id: g.id,
                name: g.name.to_string(),
                currency: g.currency.to_string(),
            })
            .collect(),
        csrf,
        archived,
        create_name: create_name.to_owned(),
        create_currency: create_currency.to_owned(),
        currencies: currency_options(create_currency),
        error,
    })
}

async fn render_group_create_error(
    state: &AppState,
    session: &Session,
    name: String,
    currency: String,
    error: String,
) -> Response {
    match groups_template(state, session, false, &name, &currency, Some(error)).await {
        Ok(template) => (StatusCode::UNPROCESSABLE_ENTITY, render(&template)).into_response(),
        Err(response) => response,
    }
}

async fn group_edit_template(
    state: &AppState,
    session: &Session,
    id: i64,
    draft: Option<(String, String)>,
    error: Option<String>,
) -> Result<GroupEditTemplate, Response> {
    let group = state.groups.group(id).await.map_err(map_error)?;
    if group.is_archived {
        return Err(error_response(
            StatusCode::CONFLICT,
            "Archived groups are read-only.",
        ));
    }
    let (name, currency) =
        draft.unwrap_or_else(|| (group.name.to_string(), group.currency.to_string()));
    Ok(GroupEditTemplate {
        id,
        name,
        currency: currency.clone(),
        currencies: currency_options(&currency),
        csrf: csrf(session).await?,
        error,
    })
}

async fn render_group_edit_error(
    state: &AppState,
    session: &Session,
    id: i64,
    name: String,
    currency: String,
    error: String,
) -> Response {
    match group_edit_template(state, session, id, Some((name, currency)), Some(error)).await {
        Ok(template) => (StatusCode::UNPROCESSABLE_ENTITY, render(&template)).into_response(),
        Err(response) => response,
    }
}

fn currency_options(selected: &str) -> Vec<SelectOption> {
    let mut options = Currency::ALL
        .iter()
        .map(|currency| SelectOption {
            value: currency.to_string(),
            label: currency.to_string(),
            selected: currency.to_string() == selected,
        })
        .collect::<Vec<_>>();
    if !selected.is_empty() && !options.iter().any(|option| option.value == selected) {
        options.insert(
            0,
            SelectOption {
                value: selected.to_owned(),
                label: selected.to_owned(),
                selected: true,
            },
        );
    }
    options
}

async fn archive(
    state: AppState,
    session: Session,
    id: i64,
    archived: bool,
    _form: CsrfValidatedForm,
) -> Response {
    if let Err(response) = require_auth(&session).await {
        return response;
    }
    match state.groups.set_archived(id, archived).await {
        Ok(()) => Redirect::to("/groups").into_response(),
        Err(error) => map_error(error),
    }
}
