#![warn(missing_docs)]

//! Pure domain logic for the debtor application.
//!
//! This crate contains business rules, domain types, and trait definitions
//! for external dependencies. It has zero I/O dependencies — all logic is
//! pure and can be tested without async runtimes or databases.

pub mod debts;
pub mod expenses;
pub mod groups;
pub mod traits;
