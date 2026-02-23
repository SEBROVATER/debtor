//! Application state shared across handlers.
//!
//! `AppState` carries references to all domain services (as trait objects)
//! and is injected into the Axum router via `.with_state()`.
//!
//! Will be populated during feature implementation (TDD).
