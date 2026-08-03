use askama::Template;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};

use crate::templates::ErrorTemplate;

pub(super) fn render(template: &impl Template) -> Response {
    template.render().map_or_else(
        |_| error_response(StatusCode::INTERNAL_SERVER_ERROR, "Template error."),
        |value| Html(value).into_response(),
    )
}

pub(super) fn error_response(status: StatusCode, message: &str) -> Response {
    let template = ErrorTemplate { message };
    (status, render(&template)).into_response()
}

pub(super) fn session_error() -> Response {
    error_response(StatusCode::INTERNAL_SERVER_ERROR, "Session error.")
}

pub(super) fn map_error(error: debtor_application::ApplicationError) -> Response {
    match error {
        debtor_application::ApplicationError::NotFound => {
            error_response(StatusCode::NOT_FOUND, "Resource not found.")
        }
        debtor_application::ApplicationError::Conflict => error_response(
            StatusCode::CONFLICT,
            "This operation conflicts with preserved history.",
        ),
        debtor_application::ApplicationError::Validation(error) => {
            error_response(StatusCode::UNPROCESSABLE_ENTITY, &error.to_string())
        }
        debtor_application::ApplicationError::Unavailable(error) => {
            error_response(StatusCode::SERVICE_UNAVAILABLE, &error)
        }
        debtor_application::ApplicationError::Storage(_) => {
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "Storage error.")
        }
    }
}
