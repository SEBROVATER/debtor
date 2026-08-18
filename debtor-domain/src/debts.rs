//! Balance calculation and debt simplification.

use thiserror::Error;

pub mod balance;
pub mod simplify;

/// Recoverable failures from debt arithmetic and settlement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum CalculationError {
    /// A checked decimal operation exceeded the Decimal representation.
    #[error("debt arithmetic overflow")]
    ArithmeticOverflow,
    /// A balance set did not sum to exactly zero.
    #[error("balances must sum to zero")]
    NonZeroSum,
    /// Quantization produced a residual that was not an exact number of minor units.
    #[error("balance residual is not an exact minor-unit amount")]
    NonIntegralResidual,
    /// Greedy settlement could not consume every balance.
    #[error("settlement left an unmatched balance")]
    UnsettledBalances,
    /// A generated settlement violated its deterministic transfer invariants.
    #[error("settlement invariant failed")]
    SettlementInvariant,
}

pub use balance::{add_converted_spending, quantize_balances, quantize_positive_totals};
pub use simplify::{Transfer, simplify};
