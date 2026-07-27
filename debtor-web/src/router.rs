//! Axum route definitions.

use axum::{
    Router,
    routing::{get, post},
};

use crate::{handlers, state::AppState};

/// Builds the application router from application-facing state.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(handlers::root))
        .route("/healthz", get(handlers::health))
        .route("/readyz", get(handlers::health))
        .route("/login", get(handlers::login_form).post(handlers::login))
        .route("/logout", post(handlers::logout))
        .route(
            "/groups",
            get(handlers::groups).post(handlers::create_group),
        )
        .route("/groups/{id}", get(handlers::group_detail))
        .route("/groups/{id}/members", post(handlers::add_member))
        .route(
            "/groups/{id}/spendings",
            post(handlers::create_equal_spending),
        )
        .route(
            "/groups/{id}/spendings/exact",
            post(handlers::create_exact_spending),
        )
        .route("/groups/{id}/archive", post(handlers::archive_group))
        .route("/groups/{id}/restore", post(handlers::restore_group))
        .route("/groups/{id}/debts", get(handlers::debts))
        .route(
            "/participants",
            get(handlers::participants).post(handlers::create_participant),
        )
        .route(
            "/participants/{id}/archive",
            post(handlers::archive_participant),
        )
        .route(
            "/participants/{id}/restore",
            post(handlers::restore_participant),
        )
        .with_state(state)
}
