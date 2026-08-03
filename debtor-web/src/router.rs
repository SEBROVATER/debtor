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
        .route(
            "/groups/{id}/edit",
            get(handlers::group_edit_form).post(handlers::update_group),
        )
        .route(
            "/groups/{id}/delete",
            get(handlers::delete_group_form).post(handlers::delete_group),
        )
        .route("/groups/{id}/members", post(handlers::add_member))
        .route(
            "/groups/{group_id}/members/{participant_id}/deactivate",
            post(handlers::deactivate_member),
        )
        .route(
            "/groups/{id}/participants",
            post(handlers::create_group_participant),
        )
        .route("/groups/{id}/spendings", post(handlers::create_spending))
        .route(
            "/groups/{group_id}/spendings/{spending_id}",
            get(handlers::spending_detail).post(handlers::update_spending),
        )
        .route(
            "/groups/{group_id}/spendings/{spending_id}/edit",
            get(handlers::edit_spending_form),
        )
        .route(
            "/groups/{group_id}/spendings/{spending_id}/delete",
            get(handlers::delete_spending_form).post(handlers::delete_spending),
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
        .route(
            "/participants/{id}/edit",
            get(handlers::participant_edit_form),
        )
        .route("/participants/{id}", post(handlers::update_participant))
        .with_state(state)
}
