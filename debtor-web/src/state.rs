//! Application-facing state shared by handlers.

use std::sync::Arc;

use debtor_application::{
    DebtUseCases, GroupUseCases, ParticipantUseCases, PasswordVerifier, SpendingUseCases,
};

/// Dependencies exposed to the HTTP layer as application interfaces.
#[derive(Clone)]
pub struct AppState {
    /// Group workflows.
    pub groups: Arc<dyn GroupUseCases>,
    /// Participant and membership workflows.
    pub participants: Arc<dyn ParticipantUseCases>,
    /// Spending workflows.
    pub spendings: Arc<dyn SpendingUseCases>,
    /// Debt workflows.
    pub debts: Arc<dyn DebtUseCases>,
    /// Password gate verifier.
    pub password: Arc<dyn PasswordVerifier>,
}
