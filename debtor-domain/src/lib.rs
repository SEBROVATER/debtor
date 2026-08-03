#![warn(missing_docs)]

//! Pure domain logic for the debtor application.
//!
//! This crate contains business rules and domain types. It has zero I/O
//! dependencies, so all logic can be tested without async runtimes or databases.

/// Supported currencies as ISO 4217 codes.
pub mod currency;
/// Balance calculation and debt simplification.
pub mod debts;
/// Expense domain logic and share splitting.
pub mod expenses;
/// Group domain logic and membership rules.
pub mod groups;
/// Core entities and validated value objects.
pub mod model;
/// Canonical exact monetary text conversion.
pub mod money;
