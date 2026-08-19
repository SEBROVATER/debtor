use askama::Template;
use axum::{
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Response},
};
use debtor_application::{ApplicationError, RateMode};

use crate::templates::ErrorTemplate;

pub(super) fn render(template: &impl Template) -> Response {
    template.render().map_or_else(
        |_| error_response(StatusCode::INTERNAL_SERVER_ERROR, "Template error."),
        |value| Html(value).into_response(),
    )
}

pub(super) fn error_response(status: StatusCode, message: &str) -> Response {
    error_response_with_recovery(status, message, "/groups")
}

pub(super) fn debt_error_response(
    error: ApplicationError,
    mode: RateMode,
    calculated_at: chrono::DateTime<chrono::Utc>,
) -> Response {
    let (status, message) = match error {
        ApplicationError::Unavailable(debtor_application::UnavailableReason::ExchangeRates) => (
            StatusCode::SERVICE_UNAVAILABLE,
            "Exchange-rate service unavailable.",
        ),
        ApplicationError::Calculation(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Unable to calculate debts.",
        ),
        ApplicationError::Storage(debtor_application::StorageReason::InvalidData) => {
            (StatusCode::INTERNAL_SERVER_ERROR, "Stored data is invalid.")
        }
        other => return map_error(other),
    };
    let mode = match mode {
        RateMode::Historical => "Historical",
        RateMode::Current => "Current",
    };
    let message = format!(
        "{message} Attempted {mode} calculation at {calculated_at} UTC. Target currency was not resolved."
    );
    error_response(status, &message)
}

pub(crate) fn debt_timeout_response(query: Option<&str>) -> Response {
    let mode =
        if query.is_some_and(|value| value.split('&').any(|part| part == "rate_mode=current")) {
            "Current"
        } else {
            "Historical"
        };
    let message = format!(
        "Debt calculation timed out. Attempted {mode} calculation at {} UTC. Target currency was not resolved.",
        chrono::Utc::now()
    );
    error_response(StatusCode::GATEWAY_TIMEOUT, &message)
}

fn error_response_with_recovery(
    status: StatusCode,
    message: &str,
    recovery_path: &str,
) -> Response {
    let template = ErrorTemplate {
        message,
        login_recovery: false,
        recovery_path,
    };
    (status, render(&template)).into_response()
}

pub(crate) fn logout_error_response(
    headers: &HeaderMap,
    status: StatusCode,
    message: &'static str,
) -> Response {
    if headers.contains_key("hx-request") {
        return Html(format!(
            "<section id=\"sign-out-status\" aria-labelledby=\"sign-out-conflict-heading\"><h2 id=\"sign-out-conflict-heading\" tabindex=\"-1\">{status}</h2><p class=\"status error\" role=\"alert\" aria-live=\"assertive\" aria-atomic=\"true\">{message} No change occurred.</p><a href=\"/login\">Reload Sign in</a></section>"
        ))
        .into_response();
    }
    error_response(status, message)
}

pub(crate) fn login_error_response(status: StatusCode, message: &str) -> Response {
    let template = ErrorTemplate {
        message,
        login_recovery: true,
        recovery_path: "/login",
    };
    (status, render(&template)).into_response()
}

pub(crate) fn login_token_conflict() -> Response {
    login_error_response(
        StatusCode::CONFLICT,
        "This sign-in form is no longer valid. Open Sign in to try again.",
    )
}

pub(crate) fn submission_token_conflict_for(path: &str, enhanced: bool) -> Response {
    let recovery_path = canonical_recovery_path(path);
    let message = format!(
        "409 Conflict. This form is no longer valid. No change occurred. Reload it at {recovery_path} to try again."
    );
    if enhanced {
        return Html(format!(
            "<section id=\"mutation-conflict\" aria-labelledby=\"mutation-conflict-heading\"><h2 id=\"mutation-conflict-heading\" tabindex=\"-1\">409 Conflict</h2><p class=\"status error\" role=\"alert\" aria-live=\"assertive\" aria-atomic=\"true\">{message}</p><a href=\"{recovery_path}\">Reload the form</a></section>"
        ))
        .into_response();
    }
    error_response_with_recovery(StatusCode::CONFLICT, &message, &recovery_path)
}

fn canonical_recovery_path(path: &str) -> String {
    if path == "/groups"
        || path == "/participants"
        || path.ends_with("/edit")
        || path.ends_with("/delete")
    {
        return path.to_owned();
    }
    if let Some(participant_id) = path
        .strip_prefix("/participants/")
        .and_then(|value| value.split('/').next())
        .filter(|value| value.parse::<i64>().is_ok())
    {
        return format!("/participants/{participant_id}/edit");
    }
    if let Some((prefix, suffix)) = path.rsplit_once("/spendings/") {
        if suffix.parse::<i64>().is_ok() {
            return path.to_owned();
        }
        return prefix.to_owned();
    }
    if let Some(group_id) = path
        .strip_prefix("/groups/")
        .and_then(|value| value.split('/').next())
        && group_id.parse::<i64>().is_ok()
    {
        return format!("/groups/{group_id}");
    }
    "/groups".to_owned()
}

pub(crate) fn timeout_response() -> Response {
    error_response(StatusCode::GATEWAY_TIMEOUT, "Request timed out.")
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

pub(super) fn submission_capacity_unavailable() -> Response {
    error_response(
        StatusCode::SERVICE_UNAVAILABLE,
        "Form capacity is temporarily unavailable. Try again.",
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
        debtor_application::ApplicationError::Storage(
            debtor_application::StorageReason::Unknown,
        ) => error_response(
            StatusCode::SERVICE_UNAVAILABLE,
            "Mutation outcome is unknown. Restart before retrying.",
        ),
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
                ApplicationError::Storage(StorageReason::Unknown),
                StatusCode::SERVICE_UNAVAILABLE,
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
