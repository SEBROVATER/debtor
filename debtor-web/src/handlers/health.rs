use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use debtor_application::{ApplicationError, StorageReason, UnavailableReason};

use crate::state::AppState;

pub(crate) async fn health() -> &'static str {
    "ok"
}

pub(crate) async fn readiness(State(state): State<AppState>) -> Response {
    if !state.runtime.user_admission_open() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            "Service temporarily unavailable.",
        )
            .into_response();
    }
    match state.readiness.check().await {
        Ok(()) => "ok".into_response(),
        Err(error) => {
            state.runtime.fail_readiness();
            tracing::warn!(
                target: "debtor.readiness",
                event = "readiness_failure",
                category = readiness_category(&error),
            );
            (
                StatusCode::SERVICE_UNAVAILABLE,
                "Service temporarily unavailable.",
            )
                .into_response()
        }
    }
}

fn readiness_category(error: &ApplicationError) -> &'static str {
    match error {
        ApplicationError::Unavailable(UnavailableReason::RuntimeSupervisor) => "runtime_supervisor",
        ApplicationError::Unavailable(_) => "dependency_unavailable",
        ApplicationError::Storage(StorageReason::Contention) => "storage_contention",
        ApplicationError::Storage(_) => "storage_failure",
        _ => "readiness_failure",
    }
}

#[cfg(test)]
mod tests {
    use debtor_application::{ApplicationError, StorageReason, UnavailableReason};

    use super::readiness_category;

    #[test]
    fn readiness_logs_only_safe_categories() {
        assert_eq!(
            readiness_category(&ApplicationError::Unavailable(
                UnavailableReason::RuntimeSupervisor,
            )),
            "runtime_supervisor"
        );
        assert_eq!(
            readiness_category(&ApplicationError::Storage(StorageReason::Contention)),
            "storage_contention"
        );
        assert_eq!(
            readiness_category(&ApplicationError::Storage(StorageReason::Unexpected)),
            "storage_failure"
        );
    }
}
