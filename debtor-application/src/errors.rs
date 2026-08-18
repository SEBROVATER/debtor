use debtor_domain::model::ValidationError;
use thiserror::Error;

/// Safe categories for unavailable external dependencies.
#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum UnavailableReason {
    /// The exchange-rate provider could not provide a quote.
    #[error("exchange-rate provider unavailable")]
    ExchangeRates,
    /// The authentication dependency could not process a request.
    #[error("authentication dependency unavailable")]
    Authentication,
    /// A mandatory in-process supervisor is unhealthy.
    #[error("runtime supervisor unavailable")]
    RuntimeSupervisor,
}

/// Safe categories for persistence failures.
#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum StorageReason {
    /// The database or application write gate is busy.
    #[error("storage contention")]
    Contention,
    /// Persisted data failed domain decoding or validation.
    #[error("invalid persisted data")]
    InvalidData,
    /// An unexpected persistence operation failed.
    #[error("unexpected storage failure")]
    Unexpected,
    /// A dispatched mutation ended without a definitive outcome.
    #[error("unknown mutation outcome")]
    Unknown,
}

/// Safe categories for startup configuration failures.
#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum ConfigurationError {
    /// The configured administrator password hash is not acceptable.
    #[error("invalid administrator password configuration")]
    InvalidPasswordHash,
}

/// Safe categories for debt calculation failures.
#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum CalculationReason {
    /// A checked Decimal operation overflowed.
    #[error("debt arithmetic overflow")]
    ArithmeticOverflow,
    /// A balance set was not exactly zero-sum.
    #[error("invalid debt balance sum")]
    NonZeroSum,
    /// Settlement could not satisfy its deterministic invariants.
    #[error("invalid settlement result")]
    SettlementInvariant,
}

/// Application-level failures suitable for HTTP mapping.
#[derive(Debug, Error)]
pub enum ApplicationError {
    /// Input failed domain validation.
    #[error(transparent)]
    Validation(#[from] ValidationError),
    /// A resource does not exist.
    #[error("resource not found")]
    NotFound,
    /// A requested mutation conflicts with preserved history.
    #[error("operation conflicts with preserved history")]
    Conflict,
    /// An external dependency failed.
    #[error("external dependency unavailable: {0}")]
    Unavailable(UnavailableReason),
    /// Persistence failed unexpectedly.
    #[error("persistence failed: {0}")]
    Storage(StorageReason),
    /// Startup configuration is invalid.
    #[error("invalid application configuration: {0}")]
    Configuration(ConfigurationError),
    /// Debt arithmetic or settlement failed a safe calculation invariant.
    #[error("debt calculation failed: {0}")]
    Calculation(CalculationReason),
}
