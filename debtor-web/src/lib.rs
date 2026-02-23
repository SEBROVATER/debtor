#![warn(missing_docs)]

//! HTTP layer for the debtor application.
//!
//! Contains Axum route definitions, request handlers, middleware,
//! and Askama template types. Depends on `debtor-domain` traits
//! but not on infrastructure implementations.

pub mod handlers;
pub mod middleware;
pub mod router;
pub mod state;
pub mod templates;
