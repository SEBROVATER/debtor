use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use debtor_domain::currency::Currency;
use tower_sessions::Session;

use super::{
    GroupsQuery,
    auth::{csrf, require_auth, require_csrf},
    response::{error_response, map_error, render},
    spendings::{build_group_template, map_group_template_error},
};
use crate::{
    forms::{OrderedForm, parse_csrf_form, parse_group_form},
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
    match state.groups.list_groups(archived).await {
        Ok(items) => render(&GroupsTemplate {
            groups: items
                .into_iter()
                .map(|g| GroupRow {
                    id: g.id,
                    name: g.name.to_string(),
                    currency: g.currency.to_string(),
                })
                .collect(),
            csrf: match csrf(&session).await {
                Ok(token) => token,
                Err(response) => return response,
            },
            archived,
        }),
        Err(_) => error_response(StatusCode::INTERNAL_SERVER_ERROR, "Unable to load groups."),
    }
}

pub(crate) async fn create_group(
    State(state): State<AppState>,
    session: Session,
    form: OrderedForm,
) -> Response {
    if let Err(response) = require_auth(&session).await {
        return response;
    }
    let form = match parse_group_form(form) {
        Ok(form) => form,
        Err(error) => return error_response(error.status, error.message),
    };
    if let Err(response) = require_csrf(&session, &form.csrf).await {
        return response;
    }
    let Ok(currency) = form.currency.parse::<Currency>() else {
        return error_response(StatusCode::UNPROCESSABLE_ENTITY, "Invalid currency.");
    };
    match state.groups.create_group(form.name, currency).await {
        Ok(_) => Redirect::to("/groups").into_response(),
        Err(error) => error_response(StatusCode::UNPROCESSABLE_ENTITY, &error.to_string()),
    }
}

pub(crate) async fn group_detail(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<i64>,
) -> Response {
    if let Err(response) = require_auth(&session).await {
        return response;
    }
    match build_group_template(&state, &session, id, None, None, None).await {
        Ok(template) => render(&template),
        Err(error) => map_group_template_error(error),
    }
}

pub(crate) async fn archive_group(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<i64>,
    form: OrderedForm,
) -> Response {
    archive(state, session, id, true, form).await
}

pub(crate) async fn restore_group(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<i64>,
    form: OrderedForm,
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
    match state.groups.group(id).await {
        Ok(group) if !group.is_archived => render(&GroupEditTemplate {
            id,
            name: group.name.to_string(),
            currency: group.currency.to_string(),
            currencies: Currency::ALL
                .iter()
                .map(|c| SelectOption {
                    value: c.to_string(),
                    label: c.to_string(),
                    selected: *c == group.currency,
                })
                .collect(),
            csrf: match csrf(&session).await {
                Ok(token) => token,
                Err(response) => return response,
            },
            error: None,
        }),
        Ok(_) => error_response(StatusCode::CONFLICT, "Archived groups are read-only."),
        Err(error) => map_error(error),
    }
}

pub(crate) async fn update_group(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<i64>,
    form: OrderedForm,
) -> Response {
    if let Err(response) = require_auth(&session).await {
        return response;
    }
    let form = match parse_group_form(form) {
        Ok(form) => form,
        Err(error) => return error_response(error.status, error.message),
    };
    if let Err(response) = require_csrf(&session, &form.csrf).await {
        return response;
    }
    let Ok(currency) = form.currency.parse::<Currency>() else {
        return error_response(StatusCode::UNPROCESSABLE_ENTITY, "Invalid currency.");
    };
    match state.groups.update_group(id, form.name, currency).await {
        Ok(_) => Redirect::to(&format!("/groups/{id}")).into_response(),
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
    match state.groups.delete_empty(id).await {
        Ok(()) => Redirect::to("/groups").into_response(),
        Err(error) => map_error(error),
    }
}

async fn archive(
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
    match state.groups.set_archived(id, archived).await {
        Ok(()) => Redirect::to("/groups").into_response(),
        Err(error) => map_error(error),
    }
}
