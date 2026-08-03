//! HTTP layer for the debtor application.
//!
//! Contains Axum route definitions, request handlers, middleware,
//! and Askama template types. Depends on `debtor-domain` traits
//! but not on infrastructure implementations.

pub mod forms;
pub mod handlers;
pub mod middleware;
pub mod router;
pub mod session;
pub mod state;
pub mod templates;
