#![warn(missing_docs)]

//! Infrastructure adapters for the debtor application.
//!
//! Implements domain traits using concrete I/O libraries:
//! - `SQLx` for database access
//! - reqwest for HTTP clients (exchange rates)
//! - argon2 for password hashing

pub mod auth;
pub mod db;
pub mod exchange_rates;
