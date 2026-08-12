//! HTTP layer for the debtor application.
//!
//! Contains Axum route definitions, request handlers, middleware,
//! and Askama template types. Depends on `debtor-domain` traits
//! but not on infrastructure implementations.

pub mod forms;
pub mod handlers;
pub mod middleware;
mod participant_color;
pub mod router;
pub mod session;
pub mod session_store;
pub mod state;
pub mod submission_tokens;
pub mod templates;
