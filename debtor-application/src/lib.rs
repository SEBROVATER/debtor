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

/// Reads group records.
#[async_trait]
pub trait GroupReader: Send + Sync {
    /// Lists groups by archive state.
    async fn list_groups(&self, archived: bool) -> Result<Vec<Group>, ApplicationError>;
    /// Loads one group.
    async fn group(&self, id: EntityId) -> Result<Group, ApplicationError>;
}

/// Writes group records.
#[async_trait]
pub trait GroupRepository: Send + Sync {
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
}

/// Reads and writes participant identities and memberships.
#[async_trait]
pub trait ParticipantRepository: Send + Sync {
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
}

/// Reads complete spending aggregates.
#[async_trait]
pub trait SpendingReader: Send + Sync {
    /// Lists a group's complete spendings.
    async fn spendings(&self, group_id: EntityId) -> Result<Vec<Spending>, ApplicationError>;
    /// Loads one complete spending.
    async fn spending(
        &self,
        group_id: EntityId,
        spending_id: EntityId,
    ) -> Result<Spending, ApplicationError>;
}

/// Writes complete spending aggregates atomically.
#[async_trait]
pub trait SpendingRepository: Send + Sync {
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

/// Compatibility facade for adapters that provide all ledger persistence ports.
pub trait LedgerStore:
    GroupReader + GroupRepository + ParticipantRepository + SpendingReader + SpendingRepository
{
}

impl<T> LedgerStore for T where
    T: GroupReader + GroupRepository + ParticipantRepository + SpendingReader + SpendingRepository
{
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
    reader: Arc<dyn GroupReader>,
    repository: Arc<dyn GroupRepository>,
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
    repository: Arc<dyn ParticipantRepository>,
}

impl ParticipantService {
    /// Creates a service with injected persistence.
    pub fn new(repository: Arc<dyn ParticipantRepository>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl ParticipantUseCases for ParticipantService {
    async fn list_participants(
        &self,
        archived: bool,
    ) -> Result<Vec<Participant>, ApplicationError> {
        self.repository.list_participants(archived).await
    }

    async fn create_participant(
        &self,
        name: String,
        color: String,
    ) -> Result<Participant, ApplicationError> {
        self.repository
            .create_participant(Name::new(name)?, Color::new(color)?)
            .await
    }

    async fn participant(&self, id: EntityId) -> Result<Participant, ApplicationError> {
        self.repository.participant(id).await
    }

    async fn update_participant(
        &self,
        id: EntityId,
        name: String,
        color: String,
    ) -> Result<Participant, ApplicationError> {
        self.repository
            .update_participant(id, Name::new(name)?, Color::new(color)?)
            .await
    }

    async fn create_group_participant(
        &self,
        group_id: EntityId,
        name: String,
        color: String,
    ) -> Result<Participant, ApplicationError> {
        self.repository
            .create_group_participant(group_id, Name::new(name)?, Color::new(color)?)
            .await
    }

    async fn set_archived(&self, id: EntityId, archived: bool) -> Result<(), ApplicationError> {
        self.repository.set_participant_archived(id, archived).await
    }

    async fn members(
        &self,
        group_id: EntityId,
    ) -> Result<Vec<(Participant, GroupMember)>, ApplicationError> {
        self.repository.group_members(group_id).await
    }

    async fn add_member(
        &self,
        group_id: EntityId,
        participant_id: EntityId,
    ) -> Result<(), ApplicationError> {
        self.repository.add_member(group_id, participant_id).await
    }

    async fn deactivate_member(
        &self,
        group_id: EntityId,
        participant_id: EntityId,
    ) -> Result<(), ApplicationError> {
        self.repository
            .set_member_active(group_id, participant_id, false)
            .await
    }
}

impl GroupService {
    /// Creates a service with injected persistence.
    pub fn new(reader: Arc<dyn GroupReader>, repository: Arc<dyn GroupRepository>) -> Self {
        Self { reader, repository }
    }
}

#[async_trait]
impl GroupUseCases for GroupService {
    async fn list_groups(&self, archived: bool) -> Result<Vec<Group>, ApplicationError> {
        self.reader.list_groups(archived).await
    }
    async fn group(&self, id: EntityId) -> Result<Group, ApplicationError> {
        self.reader.group(id).await
    }
    async fn create_group(
        &self,
        name: String,
        currency: Currency,
    ) -> Result<Group, ApplicationError> {
        self.repository
            .create_group(Name::new(name)?, currency)
            .await
    }
    async fn update_group(
        &self,
        id: EntityId,
        name: String,
        currency: Currency,
    ) -> Result<Group, ApplicationError> {
        self.repository
            .update_group(id, Name::new(name)?, currency)
            .await
    }
    async fn set_archived(&self, id: EntityId, archived: bool) -> Result<(), ApplicationError> {
        self.repository.set_group_archived(id, archived).await
    }
    async fn delete_empty(&self, id: EntityId) -> Result<(), ApplicationError> {
        self.repository.delete_empty_group(id).await
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
    groups: Arc<dyn GroupReader>,
    spendings: Arc<dyn SpendingReader>,
    rates: Arc<dyn ExchangeRateProvider>,
    clock: Arc<dyn Clock>,
}

impl DebtService {
    /// Creates a service with injected dependencies.
    pub fn new(
        groups: Arc<dyn GroupReader>,
        spendings: Arc<dyn SpendingReader>,
        rates: Arc<dyn ExchangeRateProvider>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            groups,
            spendings,
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
        let group = self.groups.group(group_id).await?;
        let calculated_at = self.clock.now();
        let today = calculated_at.date_naive();
        let mut balances = BTreeMap::new();
        let mut rates = Vec::new();
        for spending in self.spendings.spendings(group_id).await? {
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
    reader: Arc<dyn SpendingReader>,
    repository: Arc<dyn SpendingRepository>,
}

impl SpendingService {
    /// Creates a service with injected persistence.
    pub fn new(reader: Arc<dyn SpendingReader>, repository: Arc<dyn SpendingRepository>) -> Self {
        Self { reader, repository }
    }
}

#[async_trait]
impl SpendingUseCases for SpendingService {
    async fn list_spendings(&self, group_id: EntityId) -> Result<Vec<Spending>, ApplicationError> {
        self.reader.spendings(group_id).await
    }

    async fn spending(
        &self,
        group_id: EntityId,
        spending_id: EntityId,
    ) -> Result<Spending, ApplicationError> {
        self.reader.spending(group_id, spending_id).await
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
        self.repository.create_spending(spending).await
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
        self.repository.create_spending(spending).await
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
        self.repository.update_spending(spending).await
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
        self.repository.update_spending(spending).await
    }

    async fn delete(
        &self,
        group_id: EntityId,
        spending_id: EntityId,
    ) -> Result<(), ApplicationError> {
        self.repository.delete_spending(group_id, spending_id).await
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use std::sync::Mutex;

    use chrono::TimeZone;
    use debtor_domain::model::Allocation;

    use super::*;

    const GROUP_ID: EntityId = 10;
    const PARTICIPANT_ONE: EntityId = 1;
    const PARTICIPANT_TWO: EntityId = 2;

    fn date(day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 7, day).unwrap()
    }

    fn allocation(participant_id: EntityId, amount: i64) -> Allocation {
        Allocation {
            participant_id,
            amount: Decimal::new(amount, 2),
        }
    }

    fn group(id: EntityId) -> Group {
        Group {
            id,
            name: Name::new("Trip").unwrap(),
            currency: Currency::Usd,
            is_archived: false,
        }
    }

    fn participant(id: EntityId) -> Participant {
        Participant {
            id,
            name: Name::new("Ada").unwrap(),
            color: Color::new("#123456").unwrap(),
            is_archived: false,
        }
    }

    struct GroupFake {
        listed_archived: Mutex<Vec<bool>>,
        created: Mutex<Vec<(Name, Currency)>>,
        updated: Mutex<Vec<(EntityId, Name, Currency)>>,
        fail_create: bool,
    }

    #[async_trait]
    impl GroupReader for GroupFake {
        async fn list_groups(&self, archived: bool) -> Result<Vec<Group>, ApplicationError> {
            self.listed_archived.lock().unwrap().push(archived);
            Ok(vec![group(if archived { 2 } else { 1 })])
        }

        async fn group(&self, id: EntityId) -> Result<Group, ApplicationError> {
            Ok(group(id))
        }
    }

    #[async_trait]
    impl GroupRepository for GroupFake {
        async fn create_group(
            &self,
            name: Name,
            currency: Currency,
        ) -> Result<Group, ApplicationError> {
            if self.fail_create {
                return Err(ApplicationError::Storage("groups unavailable".into()));
            }
            self.created.lock().unwrap().push((name.clone(), currency));
            Ok(Group {
                id: 1,
                name,
                currency,
                is_archived: false,
            })
        }

        async fn update_group(
            &self,
            id: EntityId,
            name: Name,
            currency: Currency,
        ) -> Result<Group, ApplicationError> {
            self.updated
                .lock()
                .unwrap()
                .push((id, name.clone(), currency));
            Ok(Group {
                id,
                name,
                currency,
                is_archived: false,
            })
        }

        async fn set_group_archived(&self, _: EntityId, _: bool) -> Result<(), ApplicationError> {
            Ok(())
        }

        async fn delete_empty_group(&self, _: EntityId) -> Result<(), ApplicationError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn group_service_normalizes_writes_scopes_reads_and_propagates_storage_errors() {
        let fake = Arc::new(GroupFake {
            listed_archived: Mutex::new(Vec::new()),
            created: Mutex::new(Vec::new()),
            updated: Mutex::new(Vec::new()),
            fail_create: false,
        });
        let service = GroupService::new(fake.clone(), fake.clone());

        let groups = service.list_groups(true).await.unwrap();
        service
            .create_group("  Summer trip  ".into(), Currency::Eur)
            .await
            .unwrap();
        service
            .update_group(7, "  Updated trip  ".into(), Currency::Usd)
            .await
            .unwrap();

        assert_eq!(groups[0].id, 2);
        assert_eq!(*fake.listed_archived.lock().unwrap(), vec![true]);
        assert_eq!(fake.created.lock().unwrap()[0].0.as_str(), "Summer trip");
        assert_eq!(fake.updated.lock().unwrap()[0].1.as_str(), "Updated trip");

        let failing = Arc::new(GroupFake {
            listed_archived: Mutex::new(Vec::new()),
            created: Mutex::new(Vec::new()),
            updated: Mutex::new(Vec::new()),
            fail_create: true,
        });
        let error = GroupService::new(failing.clone(), failing)
            .create_group("Trip".into(), Currency::Usd)
            .await
            .unwrap_err();
        assert!(
            matches!(error, ApplicationError::Storage(message) if message == "groups unavailable")
        );
    }

    struct ParticipantFake {
        created: Mutex<Vec<(Name, Color)>>,
        updated: Mutex<Vec<(EntityId, Name, Color)>>,
        member_requests: Mutex<Vec<EntityId>>,
        deactivated: Mutex<Vec<(EntityId, EntityId, bool)>>,
    }

    #[async_trait]
    impl ParticipantRepository for ParticipantFake {
        async fn list_participants(&self, _: bool) -> Result<Vec<Participant>, ApplicationError> {
            Ok(Vec::new())
        }
        async fn create_participant(
            &self,
            name: Name,
            color: Color,
        ) -> Result<Participant, ApplicationError> {
            self.created
                .lock()
                .unwrap()
                .push((name.clone(), color.clone()));
            Ok(Participant {
                id: 1,
                name,
                color,
                is_archived: false,
            })
        }
        async fn participant(&self, id: EntityId) -> Result<Participant, ApplicationError> {
            Ok(participant(id))
        }
        async fn create_group_participant(
            &self,
            _: EntityId,
            name: Name,
            color: Color,
        ) -> Result<Participant, ApplicationError> {
            Ok(Participant {
                id: 1,
                name,
                color,
                is_archived: false,
            })
        }
        async fn update_participant(
            &self,
            id: EntityId,
            name: Name,
            color: Color,
        ) -> Result<Participant, ApplicationError> {
            self.updated
                .lock()
                .unwrap()
                .push((id, name.clone(), color.clone()));
            Ok(Participant {
                id,
                name,
                color,
                is_archived: false,
            })
        }
        async fn set_participant_archived(
            &self,
            _: EntityId,
            _: bool,
        ) -> Result<(), ApplicationError> {
            Ok(())
        }
        async fn group_members(
            &self,
            group_id: EntityId,
        ) -> Result<Vec<(Participant, GroupMember)>, ApplicationError> {
            self.member_requests.lock().unwrap().push(group_id);
            Ok(Vec::new())
        }
        async fn add_member(&self, _: EntityId, _: EntityId) -> Result<(), ApplicationError> {
            Ok(())
        }
        async fn set_member_active(
            &self,
            group_id: EntityId,
            participant_id: EntityId,
            active: bool,
        ) -> Result<(), ApplicationError> {
            self.deactivated
                .lock()
                .unwrap()
                .push((group_id, participant_id, active));
            Ok(())
        }
    }

    #[tokio::test]
    async fn participant_service_normalizes_writes_and_scopes_membership_actions() {
        let fake = Arc::new(ParticipantFake {
            created: Mutex::new(Vec::new()),
            updated: Mutex::new(Vec::new()),
            member_requests: Mutex::new(Vec::new()),
            deactivated: Mutex::new(Vec::new()),
        });
        let service = ParticipantService::new(fake.clone());

        service
            .create_participant("  Ada  ".into(), "#aabbcc".into())
            .await
            .unwrap();
        service
            .update_participant(3, "  Grace  ".into(), "#abcdef".into())
            .await
            .unwrap();
        service.members(GROUP_ID).await.unwrap();
        service
            .deactivate_member(GROUP_ID, PARTICIPANT_ONE)
            .await
            .unwrap();

        assert_eq!(fake.created.lock().unwrap()[0].0.as_str(), "Ada");
        assert_eq!(fake.created.lock().unwrap()[0].1.as_str(), "#AABBCC");
        assert_eq!(fake.updated.lock().unwrap()[0].1.as_str(), "Grace");
        assert_eq!(*fake.member_requests.lock().unwrap(), vec![GROUP_ID]);
        assert_eq!(
            *fake.deactivated.lock().unwrap(),
            vec![(GROUP_ID, PARTICIPANT_ONE, false)]
        );
    }

    struct SpendingFake {
        listed_groups: Mutex<Vec<EntityId>>,
        read_requests: Mutex<Vec<(EntityId, EntityId)>>,
        created: Mutex<Vec<Spending>>,
        updated: Mutex<Vec<Spending>>,
        fail_update: bool,
    }

    #[async_trait]
    impl SpendingReader for SpendingFake {
        async fn spendings(&self, group_id: EntityId) -> Result<Vec<Spending>, ApplicationError> {
            self.listed_groups.lock().unwrap().push(group_id);
            Ok(Vec::new())
        }
        async fn spending(
            &self,
            group_id: EntityId,
            spending_id: EntityId,
        ) -> Result<Spending, ApplicationError> {
            self.read_requests
                .lock()
                .unwrap()
                .push((group_id, spending_id));
            Err(ApplicationError::NotFound)
        }
    }

    #[async_trait]
    impl SpendingRepository for SpendingFake {
        async fn create_spending(&self, spending: Spending) -> Result<Spending, ApplicationError> {
            self.created.lock().unwrap().push(spending.clone());
            Ok(spending)
        }
        async fn update_spending(&self, spending: Spending) -> Result<Spending, ApplicationError> {
            if self.fail_update {
                return Err(ApplicationError::Conflict);
            }
            self.updated.lock().unwrap().push(spending.clone());
            Ok(spending)
        }
        async fn delete_spending(&self, _: EntityId, _: EntityId) -> Result<(), ApplicationError> {
            Ok(())
        }
    }

    fn equal_command() -> EqualSpendingCommand {
        EqualSpendingCommand {
            group_id: GROUP_ID,
            description: "  Dinner  ".into(),
            total: Decimal::new(1001, 2),
            currency: Currency::Usd,
            spending_type: SpendingType::Food,
            spent_date: date(5),
            payers: vec![allocation(PARTICIPANT_ONE, 1001)],
            share_participant_ids: vec![PARTICIPANT_TWO, PARTICIPANT_ONE],
        }
    }

    fn exact_command() -> ExactSpendingCommand {
        ExactSpendingCommand {
            group_id: GROUP_ID,
            description: "Taxi".into(),
            total: Decimal::new(1000, 2),
            currency: Currency::Usd,
            spending_type: SpendingType::Transport,
            spent_date: date(6),
            payers: vec![allocation(PARTICIPANT_ONE, 1000)],
            shares: vec![
                allocation(PARTICIPANT_ONE, 400),
                allocation(PARTICIPANT_TWO, 600),
            ],
        }
    }

    #[tokio::test]
    async fn spending_service_builds_equal_and_exact_create_update_aggregates_and_scopes_reads() {
        let fake = Arc::new(SpendingFake {
            listed_groups: Mutex::new(Vec::new()),
            read_requests: Mutex::new(Vec::new()),
            created: Mutex::new(Vec::new()),
            updated: Mutex::new(Vec::new()),
            fail_update: false,
        });
        let service = SpendingService::new(fake.clone(), fake.clone());

        service.create_equal(equal_command()).await.unwrap();
        service.create_exact(exact_command()).await.unwrap();
        service.update_equal(99, equal_command()).await.unwrap();
        service.update_exact(100, exact_command()).await.unwrap();
        assert!(matches!(
            service.spending(GROUP_ID, 77).await,
            Err(ApplicationError::NotFound)
        ));
        service.list_spendings(GROUP_ID).await.unwrap();

        let created = fake.created.lock().unwrap();
        assert_eq!(created[0].description.as_str(), "Dinner");
        assert_eq!(
            created[0].shares,
            vec![
                allocation(PARTICIPANT_ONE, 501),
                allocation(PARTICIPANT_TWO, 500)
            ]
        );
        assert_eq!(created[1].shares, exact_command().shares);
        let updated = fake.updated.lock().unwrap();
        assert_eq!(updated[0].id, 99);
        assert_eq!(updated[1].id, 100);
        assert_eq!(*fake.read_requests.lock().unwrap(), vec![(GROUP_ID, 77)]);
        assert_eq!(*fake.listed_groups.lock().unwrap(), vec![GROUP_ID]);
    }

    #[tokio::test]
    async fn spending_service_propagates_repository_errors() {
        let fake = Arc::new(SpendingFake {
            listed_groups: Mutex::new(Vec::new()),
            read_requests: Mutex::new(Vec::new()),
            created: Mutex::new(Vec::new()),
            updated: Mutex::new(Vec::new()),
            fail_update: true,
        });
        let error = SpendingService::new(fake.clone(), fake)
            .update_exact(9, exact_command())
            .await
            .unwrap_err();
        assert!(matches!(error, ApplicationError::Conflict));
    }

    struct FixedClock(DateTime<Utc>);
    impl Clock for FixedClock {
        fn now(&self) -> DateTime<Utc> {
            self.0
        }
    }

    struct DebtGroups;
    #[async_trait]
    impl GroupReader for DebtGroups {
        async fn list_groups(&self, _: bool) -> Result<Vec<Group>, ApplicationError> {
            Ok(Vec::new())
        }
        async fn group(&self, id: EntityId) -> Result<Group, ApplicationError> {
            Ok(group(id))
        }
    }

    struct DebtSpendings(Vec<Spending>);
    #[async_trait]
    impl SpendingReader for DebtSpendings {
        async fn spendings(&self, _: EntityId) -> Result<Vec<Spending>, ApplicationError> {
            Ok(self.0.clone())
        }
        async fn spending(&self, _: EntityId, _: EntityId) -> Result<Spending, ApplicationError> {
            Err(ApplicationError::NotFound)
        }
    }

    struct RateFake(Mutex<Vec<(Currency, Currency, NaiveDate, NaiveDate)>>);
    #[async_trait]
    impl ExchangeRateProvider for RateFake {
        async fn rate(
            &self,
            base: Currency,
            quote: Currency,
            requested_date: NaiveDate,
            today: NaiveDate,
        ) -> Result<RateQuote, ApplicationError> {
            self.0
                .lock()
                .unwrap()
                .push((base, quote, requested_date, today));
            Ok(RateQuote {
                base,
                quote,
                requested_date,
                effective_date: requested_date,
                rate: Decimal::ONE,
                is_stale: false,
                is_provisional: false,
            })
        }
    }

    struct FailingRates;
    #[async_trait]
    impl ExchangeRateProvider for FailingRates {
        async fn rate(
            &self,
            _: Currency,
            _: Currency,
            _: NaiveDate,
            _: NaiveDate,
        ) -> Result<RateQuote, ApplicationError> {
            Err(ApplicationError::Unavailable("rates unavailable".into()))
        }
    }

    #[tokio::test]
    async fn debt_service_uses_fixed_clock_for_current_and_spending_dates_for_historical_rates() {
        let spending = Spending {
            id: 1,
            group_id: GROUP_ID,
            description: Description::new("Lunch").unwrap(),
            total: Decimal::new(100, 2),
            currency: Currency::Eur,
            spending_type: SpendingType::Food,
            spent_date: date(4),
            payers: vec![allocation(PARTICIPANT_ONE, 100)],
            shares: vec![allocation(PARTICIPANT_TWO, 100)],
        };
        let rates = Arc::new(RateFake(Mutex::new(Vec::new())));
        let clock_time = Utc.with_ymd_and_hms(2026, 7, 8, 12, 30, 0).unwrap();
        let service = DebtService::new(
            Arc::new(DebtGroups),
            Arc::new(DebtSpendings(vec![spending])),
            rates.clone(),
            Arc::new(FixedClock(clock_time)),
        );

        let historical = service
            .calculate(GROUP_ID, RateMode::Historical)
            .await
            .unwrap();
        let current = service
            .calculate(GROUP_ID, RateMode::Current)
            .await
            .unwrap();

        assert_eq!(historical.calculated_at, clock_time);
        assert_eq!(current.calculated_at, clock_time);
        assert_eq!(
            *rates.0.lock().unwrap(),
            vec![
                (Currency::Eur, Currency::Usd, date(4), date(8)),
                (Currency::Eur, Currency::Usd, date(8), date(8)),
            ]
        );
    }

    #[tokio::test]
    async fn debt_service_propagates_rate_provider_errors() {
        let spending = Spending {
            id: 1,
            group_id: GROUP_ID,
            description: Description::new("Lunch").unwrap(),
            total: Decimal::new(100, 2),
            currency: Currency::Eur,
            spending_type: SpendingType::Food,
            spent_date: date(4),
            payers: vec![allocation(PARTICIPANT_ONE, 100)],
            shares: vec![allocation(PARTICIPANT_TWO, 100)],
        };
        let service = DebtService::new(
            Arc::new(DebtGroups),
            Arc::new(DebtSpendings(vec![spending])),
            Arc::new(FailingRates),
            Arc::new(FixedClock(
                Utc.with_ymd_and_hms(2026, 7, 8, 12, 30, 0).unwrap(),
            )),
        );

        let error = service
            .calculate(GROUP_ID, RateMode::Historical)
            .await
            .unwrap_err();

        assert!(
            matches!(error, ApplicationError::Unavailable(message) if message == "rates unavailable")
        );
    }
}
