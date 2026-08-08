//! Sanitized process-startup failure categories.

use std::fmt;

/// Safe startup categories that intentionally retain no adapter diagnostic source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StartupError {
    /// Local application configuration is invalid.
    Configuration,
    /// `SQLite` could not be connected.
    DatabaseConnect,
    /// `SQLite` migrations could not be applied.
    Migration,
}

impl fmt::Display for StartupError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Configuration => "invalid application configuration",
            Self::DatabaseConnect => "database connection failed",
            Self::Migration => "database migration failed",
        })
    }
}

impl std::error::Error for StartupError {}

#[cfg(test)]
mod tests {
    use super::StartupError;

    #[test]
    fn startup_errors_never_include_adapter_diagnostics() {
        for error in [
            StartupError::Configuration,
            StartupError::DatabaseConnect,
            StartupError::Migration,
        ] {
            let display = error.to_string();
            assert!(!display.contains("sqlite"));
            assert!(!display.contains("SELECT"));
            assert!(!display.contains("sentinel"));
            assert!(std::error::Error::source(&error).is_none());
        }
    }
}
