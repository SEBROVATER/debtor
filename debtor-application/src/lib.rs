//! Application use cases and mockable ports for debtor.

use std::collections::BTreeMap;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use debtor_domain::currency::Currency;
use debtor_domain::debts::{Transfer, add_converted_spending, quantize_balances, simplify};
use debtor_domain::expenses::splitting::equal_split;
use debtor_domain::model::{
    Color, Description, EntityId, Group, GroupMember, Name, Participant, Spending, SpendingType,
    ValidationError,
};
use rust_decimal::Decimal;
use thiserror::Error;

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
    Unavailable(String),
    /// Persistence failed unexpectedly.
    #[error("persistence failed: {0}")]
    Storage(String),
}

/// Rate selection requested by the debts view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RateMode {
    /// Use each spending's historical date.
    Historical,
    /// Use the current UTC date.
    Current,
}

/// Exchange-rate information retained for transparent calculations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RateQuote {
    /// Requested source currency.
    pub base: Currency,
    /// Target currency.
    pub quote: Currency,
    /// Date requested by the calculation mode.
    pub requested_date: NaiveDate,
    /// Date returned by the provider.
    pub effective_date: NaiveDate,
    /// Exact rate.
    pub rate: Decimal,
    /// Whether an old cached result was used.
    pub is_stale: bool,
    /// Whether a future historical date used a current fallback.
    pub is_provisional: bool,
}

/// Source of wall-clock time, injected for deterministic tests.
pub trait Clock: Send + Sync {
    /// Returns the current UTC instant.
    fn now(&self) -> DateTime<Utc>;
}

/// Production UTC clock.
pub struct UtcClock;

impl Clock for UtcClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

/// Loads a rate for a currency pair and requested date.
#[async_trait]
pub trait ExchangeRateProvider: Send + Sync {
    /// Returns a rate quote, including fallback metadata.
    async fn rate(
        &self,
        base: Currency,
        quote: Currency,
        requested_date: NaiveDate,
        today: NaiveDate,
    ) -> Result<RateQuote, ApplicationError>;
}

/// Verifies the configured password without exposing a hash to web handlers.
#[async_trait]
pub trait PasswordVerifier: Send + Sync {
    /// Returns whether the submitted password is valid.
    async fn verify(&self, password: &str) -> Result<bool, ApplicationError>;
}

/// Admission result for a login attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoginAdmission {
    /// The attempt may proceed.
    Allowed,
    /// The attempt is blocked until the indicated number of seconds elapses.
    RetryAfter(u64),
}

/// Limits password attempts by resolved client identity.
#[async_trait]
pub trait LoginAttemptLimiter: Send + Sync {
    /// Reserves one password attempt.
    async fn reserve(&self, client: std::net::IpAddr) -> LoginAdmission;
    /// Clears attempts after a successful authenticated session is created.
    async fn reset(&self, client: std::net::IpAddr);
}

/// Atomic persistence operations used by application workflows.
#[async_trait]
pub trait LedgerStore: Send + Sync {
    /// Lists groups by archive state.
    async fn list_groups(&self, archived: bool) -> Result<Vec<Group>, ApplicationError>;
    /// Loads one group.
    async fn group(&self, id: EntityId) -> Result<Group, ApplicationError>;
    /// Creates a group.
    async fn create_group(&self, name: Name, currency: Currency)
    -> Result<Group, ApplicationError>;
    /// Updates group metadata.
    async fn update_group(
        &self,
        id: EntityId,
        name: Name,
        currency: Currency,
    ) -> Result<Group, ApplicationError>;
    /// Changes archive state.
    async fn set_group_archived(
        &self,
        id: EntityId,
        archived: bool,
    ) -> Result<(), ApplicationError>;
    /// Deletes an empty group.
    async fn delete_empty_group(&self, id: EntityId) -> Result<(), ApplicationError>;
    /// Lists participants by archive state.
    async fn list_participants(&self, archived: bool)
    -> Result<Vec<Participant>, ApplicationError>;
    /// Creates a participant.
    async fn create_participant(
        &self,
        name: Name,
        color: Color,
    ) -> Result<Participant, ApplicationError>;
    /// Loads one participant.
    async fn participant(&self, id: EntityId) -> Result<Participant, ApplicationError>;
    /// Creates a participant and active membership atomically.
    async fn create_group_participant(
        &self,
        group_id: EntityId,
        name: Name,
        color: Color,
    ) -> Result<Participant, ApplicationError>;
    /// Updates one active participant.
    ///
    /// Archived identities are retained for history and reject direct updates.
    async fn update_participant(
        &self,
        id: EntityId,
        name: Name,
        color: Color,
    ) -> Result<Participant, ApplicationError>;
    /// Changes participant archive state.
    async fn set_participant_archived(
        &self,
        id: EntityId,
        archived: bool,
    ) -> Result<(), ApplicationError>;
    /// Lists group memberships with participant data.
    async fn group_members(
        &self,
        group_id: EntityId,
    ) -> Result<Vec<(Participant, GroupMember)>, ApplicationError>;
    /// Adds an active group membership.
    async fn add_member(
        &self,
        group_id: EntityId,
        participant_id: EntityId,
    ) -> Result<(), ApplicationError>;
    /// Changes membership activity.
    async fn set_member_active(
        &self,
        group_id: EntityId,
        participant_id: EntityId,
        active: bool,
    ) -> Result<(), ApplicationError>;
    /// Lists a group's complete spendings.
    async fn spendings(&self, group_id: EntityId) -> Result<Vec<Spending>, ApplicationError>;
    /// Loads one complete spending.
    async fn spending(
        &self,
        group_id: EntityId,
        spending_id: EntityId,
    ) -> Result<Spending, ApplicationError>;
    /// Atomically creates a validated spending whose referenced members are active.
    async fn create_spending(&self, spending: Spending) -> Result<Spending, ApplicationError>;
    /// Atomically replaces a validated spending while preserving historical-member rules.
    async fn update_spending(&self, spending: Spending) -> Result<Spending, ApplicationError>;
    /// Deletes a spending and its allocations.
    async fn delete_spending(
        &self,
        group_id: EntityId,
        spending_id: EntityId,
    ) -> Result<(), ApplicationError>;
}

/// Inbound group operations.
#[async_trait]
pub trait GroupUseCases: Send + Sync {
    /// Lists groups.
    async fn list_groups(&self, archived: bool) -> Result<Vec<Group>, ApplicationError>;
    /// Loads one group.
    async fn group(&self, id: EntityId) -> Result<Group, ApplicationError>;
    /// Creates a group.
    async fn create_group(
        &self,
        name: String,
        currency: Currency,
    ) -> Result<Group, ApplicationError>;
    /// Updates a group.
    async fn update_group(
        &self,
        id: EntityId,
        name: String,
        currency: Currency,
    ) -> Result<Group, ApplicationError>;
    /// Archives or restores a group.
    async fn set_archived(&self, id: EntityId, archived: bool) -> Result<(), ApplicationError>;
    /// Deletes an empty group.
    async fn delete_empty(&self, id: EntityId) -> Result<(), ApplicationError>;
}

/// Group workflow implementation.
pub struct GroupService {
    store: Arc<dyn LedgerStore>,
}

/// Inbound participant and membership operations.
#[async_trait]
pub trait ParticipantUseCases: Send + Sync {
    /// Lists globally active or archived participants.
    async fn list_participants(&self, archived: bool)
    -> Result<Vec<Participant>, ApplicationError>;
    /// Creates a reusable participant.
    async fn create_participant(
        &self,
        name: String,
        color: String,
    ) -> Result<Participant, ApplicationError>;
    /// Loads one participant.
    async fn participant(&self, id: EntityId) -> Result<Participant, ApplicationError>;
    /// Updates an active reusable participant.
    ///
    /// Archived identities are retained for history and reject direct updates.
    async fn update_participant(
        &self,
        id: EntityId,
        name: String,
        color: String,
    ) -> Result<Participant, ApplicationError>;
    /// Creates and joins a participant in one transaction.
    async fn create_group_participant(
        &self,
        group_id: EntityId,
        name: String,
        color: String,
    ) -> Result<Participant, ApplicationError>;
    /// Archives or restores a participant.
    async fn set_archived(&self, id: EntityId, archived: bool) -> Result<(), ApplicationError>;
    /// Lists memberships with participant data.
    async fn members(
        &self,
        group_id: EntityId,
    ) -> Result<Vec<(Participant, GroupMember)>, ApplicationError>;
    /// Adds a participant to a group.
    async fn add_member(
        &self,
        group_id: EntityId,
        participant_id: EntityId,
    ) -> Result<(), ApplicationError>;
    /// Deactivates a membership while preserving its history.
    async fn deactivate_member(
        &self,
        group_id: EntityId,
        participant_id: EntityId,
    ) -> Result<(), ApplicationError>;
}

/// Participant and membership workflow implementation.
pub struct ParticipantService {
    store: Arc<dyn LedgerStore>,
}

impl ParticipantService {
    /// Creates a service with injected persistence.
    pub fn new(store: Arc<dyn LedgerStore>) -> Self {
        Self { store }
    }
}

#[async_trait]
impl ParticipantUseCases for ParticipantService {
    async fn list_participants(
        &self,
        archived: bool,
    ) -> Result<Vec<Participant>, ApplicationError> {
        self.store.list_participants(archived).await
    }

    async fn create_participant(
        &self,
        name: String,
        color: String,
    ) -> Result<Participant, ApplicationError> {
        self.store
            .create_participant(Name::new(name)?, Color::new(color)?)
            .await
    }

    async fn participant(&self, id: EntityId) -> Result<Participant, ApplicationError> {
        self.store.participant(id).await
    }

    async fn update_participant(
        &self,
        id: EntityId,
        name: String,
        color: String,
    ) -> Result<Participant, ApplicationError> {
        self.store
            .update_participant(id, Name::new(name)?, Color::new(color)?)
            .await
    }

    async fn create_group_participant(
        &self,
        group_id: EntityId,
        name: String,
        color: String,
    ) -> Result<Participant, ApplicationError> {
        self.store
            .create_group_participant(group_id, Name::new(name)?, Color::new(color)?)
            .await
    }

    async fn set_archived(&self, id: EntityId, archived: bool) -> Result<(), ApplicationError> {
        self.store.set_participant_archived(id, archived).await
    }

    async fn members(
        &self,
        group_id: EntityId,
    ) -> Result<Vec<(Participant, GroupMember)>, ApplicationError> {
        self.store.group_members(group_id).await
    }

    async fn add_member(
        &self,
        group_id: EntityId,
        participant_id: EntityId,
    ) -> Result<(), ApplicationError> {
        self.store.add_member(group_id, participant_id).await
    }

    async fn deactivate_member(
        &self,
        group_id: EntityId,
        participant_id: EntityId,
    ) -> Result<(), ApplicationError> {
        self.store
            .set_member_active(group_id, participant_id, false)
            .await
    }
}

impl GroupService {
    /// Creates a service with injected persistence.
    pub fn new(store: Arc<dyn LedgerStore>) -> Self {
        Self { store }
    }
}

#[async_trait]
impl GroupUseCases for GroupService {
    async fn list_groups(&self, archived: bool) -> Result<Vec<Group>, ApplicationError> {
        self.store.list_groups(archived).await
    }
    async fn group(&self, id: EntityId) -> Result<Group, ApplicationError> {
        self.store.group(id).await
    }
    async fn create_group(
        &self,
        name: String,
        currency: Currency,
    ) -> Result<Group, ApplicationError> {
        self.store.create_group(Name::new(name)?, currency).await
    }
    async fn update_group(
        &self,
        id: EntityId,
        name: String,
        currency: Currency,
    ) -> Result<Group, ApplicationError> {
        self.store
            .update_group(id, Name::new(name)?, currency)
            .await
    }
    async fn set_archived(&self, id: EntityId, archived: bool) -> Result<(), ApplicationError> {
        self.store.set_group_archived(id, archived).await
    }
    async fn delete_empty(&self, id: EntityId) -> Result<(), ApplicationError> {
        self.store.delete_empty_group(id).await
    }
}

/// Result of a debt calculation.
#[derive(Debug, Clone)]
pub struct DebtResult {
    /// Group currency.
    pub currency: Currency,
    /// Transfers which settle the rounded balances.
    pub transfers: Vec<Transfer>,
    /// Final target-currency balances by participant.
    pub balances: BTreeMap<EntityId, Decimal>,
    /// Unique provider quotes used.
    pub rates: Vec<RateQuote>,
    /// UTC instant at which this calculation was performed.
    pub calculated_at: DateTime<Utc>,
}

/// Inbound debt operations.
#[async_trait]
pub trait DebtUseCases: Send + Sync {
    /// Calculates advisory transfers for all group history.
    async fn calculate(
        &self,
        group_id: EntityId,
        mode: RateMode,
    ) -> Result<DebtResult, ApplicationError>;
}

/// Debt workflow implementation.
pub struct DebtService {
    store: Arc<dyn LedgerStore>,
    rates: Arc<dyn ExchangeRateProvider>,
    clock: Arc<dyn Clock>,
}

impl DebtService {
    /// Creates a service with injected dependencies.
    pub fn new(
        store: Arc<dyn LedgerStore>,
        rates: Arc<dyn ExchangeRateProvider>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            store,
            rates,
            clock,
        }
    }
}

#[async_trait]
impl DebtUseCases for DebtService {
    async fn calculate(
        &self,
        group_id: EntityId,
        mode: RateMode,
    ) -> Result<DebtResult, ApplicationError> {
        let group = self.store.group(group_id).await?;
        let calculated_at = self.clock.now();
        let today = calculated_at.date_naive();
        let mut balances = BTreeMap::new();
        let mut rates = Vec::new();
        for spending in self.store.spendings(group_id).await? {
            let requested_date = match mode {
                RateMode::Historical => spending.spent_date,
                RateMode::Current => today,
            };
            let quote = self
                .rates
                .rate(spending.currency, group.currency, requested_date, today)
                .await?;
            add_converted_spending(&mut balances, &spending, quote.rate);
            if !rates.contains(&quote) {
                rates.push(quote);
            }
        }
        quantize_balances(&mut balances, group.currency);
        Ok(DebtResult {
            currency: group.currency,
            transfers: simplify(&balances),
            balances,
            rates,
            calculated_at,
        })
    }
}

/// Input for a new equal-split spending.
pub struct EqualSpendingCommand {
    /// Owning group.
    pub group_id: EntityId,
    /// Description entered by the admin.
    pub description: String,
    /// Positive source-currency total.
    pub total: Decimal,
    /// Source currency.
    pub currency: Currency,
    /// Fixed category.
    pub spending_type: SpendingType,
    /// Spending date.
    pub spent_date: NaiveDate,
    /// One or more payer allocations.
    pub payers: Vec<debtor_domain::model::Allocation>,
    /// Selected share recipients.
    pub share_participant_ids: Vec<EntityId>,
}

/// Input for a new exact-share spending.
pub struct ExactSpendingCommand {
    /// Owning group.
    pub group_id: EntityId,
    /// Description entered by the admin.
    pub description: String,
    /// Positive source-currency total.
    pub total: Decimal,
    /// Source currency.
    pub currency: Currency,
    /// Fixed category.
    pub spending_type: SpendingType,
    /// Spending date.
    pub spent_date: NaiveDate,
    /// One or more payer allocations.
    pub payers: Vec<debtor_domain::model::Allocation>,
    /// One or more positive exact owed-share allocations.
    pub shares: Vec<debtor_domain::model::Allocation>,
}

/// Inbound spending operations.
#[async_trait]
pub trait SpendingUseCases: Send + Sync {
    /// Lists a group's complete spending history.
    async fn list_spendings(&self, group_id: EntityId) -> Result<Vec<Spending>, ApplicationError>;
    /// Loads one spending scoped to a group.
    async fn spending(
        &self,
        group_id: EntityId,
        spending_id: EntityId,
    ) -> Result<Spending, ApplicationError>;
    /// Creates a validated equal-split spending.
    async fn create_equal(
        &self,
        command: EqualSpendingCommand,
    ) -> Result<Spending, ApplicationError>;
    /// Creates a validated exact-share spending.
    async fn create_exact(
        &self,
        command: ExactSpendingCommand,
    ) -> Result<Spending, ApplicationError>;
    /// Updates an equal-split spending.
    async fn update_equal(
        &self,
        spending_id: EntityId,
        command: EqualSpendingCommand,
    ) -> Result<Spending, ApplicationError>;
    /// Updates an exact-share spending.
    async fn update_exact(
        &self,
        spending_id: EntityId,
        command: ExactSpendingCommand,
    ) -> Result<Spending, ApplicationError>;
    /// Deletes a spending correction.
    async fn delete(
        &self,
        group_id: EntityId,
        spending_id: EntityId,
    ) -> Result<(), ApplicationError>;
}

/// Spending workflow implementation.
pub struct SpendingService {
    store: Arc<dyn LedgerStore>,
}

impl SpendingService {
    /// Creates a service with injected persistence.
    pub fn new(store: Arc<dyn LedgerStore>) -> Self {
        Self { store }
    }
}

#[async_trait]
impl SpendingUseCases for SpendingService {
    async fn list_spendings(&self, group_id: EntityId) -> Result<Vec<Spending>, ApplicationError> {
        self.store.spendings(group_id).await
    }

    async fn spending(
        &self,
        group_id: EntityId,
        spending_id: EntityId,
    ) -> Result<Spending, ApplicationError> {
        self.store.spending(group_id, spending_id).await
    }

    async fn create_equal(
        &self,
        command: EqualSpendingCommand,
    ) -> Result<Spending, ApplicationError> {
        let shares = equal_split(
            command.total,
            command.currency,
            &command.share_participant_ids,
        )?;
        let spending = Spending {
            id: 0,
            group_id: command.group_id,
            description: Description::new(command.description)?,
            total: command.total,
            currency: command.currency,
            spending_type: command.spending_type,
            spent_date: command.spent_date,
            payers: command.payers,
            shares,
        };
        spending.validate()?;
        self.store.create_spending(spending).await
    }

    async fn create_exact(
        &self,
        command: ExactSpendingCommand,
    ) -> Result<Spending, ApplicationError> {
        let spending = Spending {
            id: 0,
            group_id: command.group_id,
            description: Description::new(command.description)?,
            total: command.total,
            currency: command.currency,
            spending_type: command.spending_type,
            spent_date: command.spent_date,
            payers: command.payers,
            shares: command.shares,
        };
        spending.validate()?;
        self.store.create_spending(spending).await
    }

    async fn update_equal(
        &self,
        spending_id: EntityId,
        command: EqualSpendingCommand,
    ) -> Result<Spending, ApplicationError> {
        let shares = equal_split(
            command.total,
            command.currency,
            &command.share_participant_ids,
        )?;
        let spending = Spending {
            id: spending_id,
            group_id: command.group_id,
            description: Description::new(command.description)?,
            total: command.total,
            currency: command.currency,
            spending_type: command.spending_type,
            spent_date: command.spent_date,
            payers: command.payers,
            shares,
        };
        spending.validate()?;
        self.store.update_spending(spending).await
    }

    async fn update_exact(
        &self,
        spending_id: EntityId,
        command: ExactSpendingCommand,
    ) -> Result<Spending, ApplicationError> {
        let spending = Spending {
            id: spending_id,
            group_id: command.group_id,
            description: Description::new(command.description)?,
            total: command.total,
            currency: command.currency,
            spending_type: command.spending_type,
            spent_date: command.spent_date,
            payers: command.payers,
            shares: command.shares,
        };
        spending.validate()?;
        self.store.update_spending(spending).await
    }

    async fn delete(
        &self,
        group_id: EntityId,
        spending_id: EntityId,
    ) -> Result<(), ApplicationError> {
        self.store.delete_spending(group_id, spending_id).await
    }
}
