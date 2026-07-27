#![warn(missing_docs)]

//! Application use cases and mockable ports for debtor.

use std::collections::BTreeMap;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{NaiveDate, Utc};
use debtor_domain::currency::Currency;
use debtor_domain::debts::{Transfer, add_converted_spending, simplify};
use debtor_domain::model::{
    Color, EntityId, Group, GroupMember, Name, Participant, Spending, ValidationError,
};
use rust_decimal::prelude::Signed;
use rust_decimal::{Decimal, RoundingStrategy};
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
    /// Returns the current UTC date.
    fn today(&self) -> NaiveDate;
}

/// Production UTC clock.
pub struct UtcClock;

impl Clock for UtcClock {
    fn today(&self) -> NaiveDate {
        Utc::now().date_naive()
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
    /// Updates one participant.
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
    /// Removes an unused membership.
    async fn delete_unused_member(
        &self,
        group_id: EntityId,
        participant_id: EntityId,
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
        let today = self.clock.today();
        let mut balances = BTreeMap::new();
        let mut rates = Vec::new();
        for spending in self.store.spendings(group_id).await? {
            let requested_date = match mode {
                RateMode::Historical if spending.spent_date <= today => spending.spent_date,
                _ => today,
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
        })
    }
}

/// Rounds balances while preserving their exact zero sum.
pub fn quantize_balances(balances: &mut BTreeMap<EntityId, Decimal>, currency: Currency) {
    let scale = currency.minor_unit_scale();
    let rounded: BTreeMap<_, _> = balances
        .iter()
        .map(|(id, amount)| {
            (
                *id,
                amount.round_dp_with_strategy(scale, RoundingStrategy::ToZero),
            )
        })
        .collect();
    let residual = -rounded.values().copied().sum::<Decimal>();
    *balances = rounded;
    if !residual.is_zero() {
        let unit = Decimal::new(1, scale);
        let target = if residual.is_sign_positive() {
            balances
                .iter()
                .filter(|(_, value)| value.is_sign_positive() || value.is_zero())
                .map(|(id, value)| (*id, *value))
                .max_by(|a, b| a.1.cmp(&b.1).then_with(|| b.0.cmp(&a.0)))
        } else {
            balances
                .iter()
                .filter(|(_, value)| value.is_sign_negative() || value.is_zero())
                .map(|(id, value)| (*id, *value))
                .min_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.cmp(&b.0)))
        };
        if let Some((id, _)) = target {
            *balances.entry(id).or_default() += residual.signum() * unit;
        }
    }
}
