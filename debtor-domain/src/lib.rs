#![warn(missing_docs)]

//! Pure domain logic for the debtor application.
//!
//! This crate contains business rules, domain types, and trait definitions
//! for external dependencies. It has zero I/O dependencies — all logic is
//! pure and can be tested without async runtimes or databases.

/// Supported currencies as ISO 4217 codes.
pub mod currency;
/// Balance calculation and debt simplification.
pub mod debts;
/// Expense domain logic and share splitting.
pub mod expenses;
/// Group domain logic and membership rules.
pub mod groups;
/// Repository and service trait definitions.
pub mod traits;
