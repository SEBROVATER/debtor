use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::state::AppState;

pub(crate) async fn health() -> &'static str {
    "ok"
}

pub(crate) async fn readiness(State(state): State<AppState>) -> Response {
    match state.readiness.check().await {
        Ok(()) => "ok".into_response(),
        Err(_) => (
            StatusCode::SERVICE_UNAVAILABLE,
            "Service temporarily unavailable.",
        )
            .into_response(),
    }
}
