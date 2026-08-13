use askama::Template;
use axum::{
    http::{HeaderMap, HeaderValue, StatusCode, header::CONTENT_TYPE},
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
    let template = ErrorTemplate {
        message,
        login_recovery: false,
    };
    (status, render(&template)).into_response()
}

pub(super) fn logout_error_response(
    headers: &HeaderMap,
    status: StatusCode,
    message: &'static str,
) -> Response {
    if headers.contains_key("hx-request") {
        return (
            status,
            [(
                CONTENT_TYPE,
                HeaderValue::from_static("text/plain; charset=utf-8"),
            )],
            message,
        )
            .into_response();
    }
    error_response(status, message)
}

pub(crate) fn login_error_response(status: StatusCode, message: &str) -> Response {
    let template = ErrorTemplate {
        message,
        login_recovery: true,
    };
    (status, render(&template)).into_response()
}

pub(crate) fn login_token_conflict() -> Response {
    login_error_response(
        StatusCode::CONFLICT,
        "This sign-in form is no longer valid. Open Sign in to try again.",
    )
}

pub(crate) fn login_timeout() -> Response {
    login_error_response(
        StatusCode::GATEWAY_TIMEOUT,
        "Sign-in request timed out. Try again.",
    )
}

pub(crate) fn with_status(mut response: Response, status: StatusCode) -> Response {
    *response.status_mut() = status;
    response
}

pub(super) fn session_error() -> Response {
    error_response(StatusCode::INTERNAL_SERVER_ERROR, "Session error.")
}

pub(super) fn session_unavailable() -> Response {
    error_response(
        StatusCode::SERVICE_UNAVAILABLE,
        "Session storage is temporarily unavailable. Try again.",
    )
}

pub(super) fn login_session_unavailable() -> Response {
    login_error_response(
        StatusCode::SERVICE_UNAVAILABLE,
        "Session storage is temporarily unavailable. Try again.",
    )
}

pub(super) fn login_session_error() -> Response {
    login_error_response(StatusCode::INTERNAL_SERVER_ERROR, "Session error.")
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
        debtor_application::ApplicationError::Unavailable(
            debtor_application::UnavailableReason::ExchangeRates,
        ) => error_response(
            StatusCode::SERVICE_UNAVAILABLE,
            "Exchange-rate service unavailable.",
        ),
        debtor_application::ApplicationError::Unavailable(
            debtor_application::UnavailableReason::Authentication,
        ) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Authentication service unavailable.",
        ),
        debtor_application::ApplicationError::Unavailable(
            debtor_application::UnavailableReason::RuntimeSupervisor,
        ) => error_response(
            StatusCode::SERVICE_UNAVAILABLE,
            "Runtime supervisor unavailable.",
        ),
        debtor_application::ApplicationError::Storage(
            debtor_application::StorageReason::Contention,
        ) => error_response(
            StatusCode::SERVICE_UNAVAILABLE,
            "Storage is busy. Try again.",
        ),
        debtor_application::ApplicationError::Storage(
            debtor_application::StorageReason::InvalidData,
        ) => error_response(StatusCode::INTERNAL_SERVER_ERROR, "Stored data is invalid."),
        debtor_application::ApplicationError::Storage(
            debtor_application::StorageReason::Unexpected,
        ) => error_response(StatusCode::INTERNAL_SERVER_ERROR, "Storage error."),
        debtor_application::ApplicationError::Configuration(_) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Application configuration error.",
        ),
        debtor_application::ApplicationError::Calculation(_) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Unable to calculate debts.",
        ),
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use axum::{body::to_bytes, http::StatusCode};
    use debtor_application::{
        ApplicationError, CalculationReason, ConfigurationError, StorageReason, UnavailableReason,
    };

    use super::map_error;

    #[test]
    fn maps_each_safe_error_category_to_its_status() {
        let cases = [
            (
                ApplicationError::Unavailable(UnavailableReason::ExchangeRates),
                StatusCode::SERVICE_UNAVAILABLE,
            ),
            (
                ApplicationError::Unavailable(UnavailableReason::Authentication),
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
            (
                ApplicationError::Unavailable(UnavailableReason::RuntimeSupervisor),
                StatusCode::SERVICE_UNAVAILABLE,
            ),
            (
                ApplicationError::Storage(StorageReason::Contention),
                StatusCode::SERVICE_UNAVAILABLE,
            ),
            (
                ApplicationError::Storage(StorageReason::InvalidData),
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
            (
                ApplicationError::Storage(StorageReason::Unexpected),
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
            (
                ApplicationError::Configuration(ConfigurationError::InvalidPasswordHash),
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
            (
                ApplicationError::Calculation(CalculationReason::ArithmeticOverflow),
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
        ];

        for (error, expected) in cases {
            assert_eq!(map_error(error).status(), expected);
        }
    }

    #[tokio::test]
    async fn unavailable_response_uses_fixed_text_without_adapter_details() {
        let response = map_error(ApplicationError::Unavailable(
            UnavailableReason::ExchangeRates,
        ));
        let body = to_bytes(response.into_body(), 4096)
            .await
            .expect("rendered error body");
        let body = String::from_utf8(body.to_vec()).expect("UTF-8 error body");

        assert!(body.contains("Exchange-rate service unavailable."));
        assert!(!body.contains("provider-internal-detail"));
    }
}
