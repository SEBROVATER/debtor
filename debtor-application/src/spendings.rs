use std::collections::BTreeSet;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::NaiveDate;
use debtor_domain::currency::Currency;
use debtor_domain::expenses::splitting::equal_split;
use debtor_domain::model::{
    Allocation, Description, EntityId, Spending, SpendingType, ValidationError,
};
use rust_decimal::Decimal;

use crate::ApplicationError;

/// Direction for a bounded keyset history query.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpendingPageDirection {
    /// Load rows older than the anchor.
    Older,
    /// Load rows newer than the anchor.
    Newer,
}

/// Stable keyset cursor for ordinary spending history.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpendingCursor {
    /// Direction relative to the anchor.
    pub direction: SpendingPageDirection,
    /// Anchor date.
    pub spent_date: NaiveDate,
    /// Anchor identity used to break equal-date ties.
    pub id: EntityId,
}

/// Bounded summary row for ordinary spending history.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpendingSummary {
    /// Spending identity.
    pub id: EntityId,
    /// Owning group.
    pub group_id: EntityId,
    /// Validated description.
    pub description: Description,
    /// Exact source total.
    pub total: Decimal,
    /// Source currency.
    pub currency: Currency,
    /// Spending date.
    pub spent_date: NaiveDate,
}

/// One bounded spending-history page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpendingPage {
    /// Summary rows in display order.
    pub items: Vec<SpendingSummary>,
    /// Cursor for the next older page, if known.
    pub older: Option<SpendingCursor>,
    /// Cursor for the next newer page, if known.
    pub newer: Option<SpendingCursor>,
}

/// Reads complete spending aggregates and bounded history summaries.
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
    /// Loads one bounded keyset page of spending summaries.
    async fn spending_page(
        &self,
        group_id: EntityId,
        cursor: Option<SpendingCursor>,
    ) -> Result<SpendingPage, ApplicationError>;
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

/// Reads participants eligible for new spending allocations in a group.
#[async_trait]
pub trait SpendingEligibilityReader: Send + Sync {
    /// Returns active, non-archived participant identities for the group.
    async fn eligible_participant_ids(
        &self,
        group_id: EntityId,
    ) -> Result<BTreeSet<EntityId>, ApplicationError>;
}

/// Raw payer selection decoded from a transport adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PayerInput {
    /// One participant paid the full raw total.
    Single(EntityId),
    /// Raw exact amounts keyed by participant identity.
    Exact(Vec<(EntityId, String)>),
}

/// Raw share selection decoded from a transport adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShareInput {
    /// Participant identities receiving an equal split.
    Equal(Vec<EntityId>),
    /// Raw exact amounts keyed by participant identity.
    Exact(Vec<(EntityId, String)>),
}

/// Transport-neutral raw spending input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpendingInput {
    /// Owning group.
    pub group_id: EntityId,
    /// Description text before domain normalization.
    pub description: String,
    /// Total amount text before Decimal parsing.
    pub total: String,
    /// Currency code text.
    pub currency: String,
    /// Spending category text.
    pub spending_type: String,
    /// ISO date text.
    pub spent_date: String,
    /// Raw payer selection.
    pub payers: PayerInput,
    /// Raw share selection.
    pub shares: ShareInput,
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
    /// Loads a bounded keyset page of spending summaries.
    async fn spending_page(
        &self,
        group_id: EntityId,
        cursor: Option<SpendingCursor>,
    ) -> Result<SpendingPage, ApplicationError>;
    /// Creates a spending from raw, transport-neutral input.
    async fn create_input(&self, input: SpendingInput) -> Result<Spending, ApplicationError>;
    /// Updates a spending from raw, transport-neutral input.
    async fn update_input(
        &self,
        spending_id: EntityId,
        input: SpendingInput,
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
    eligibility: Arc<dyn SpendingEligibilityReader>,
}

impl SpendingService {
    /// Creates a service with injected persistence.
    pub fn new(
        reader: Arc<dyn SpendingReader>,
        repository: Arc<dyn SpendingRepository>,
        eligibility: Arc<dyn SpendingEligibilityReader>,
    ) -> Self {
        Self {
            reader,
            repository,
            eligibility,
        }
    }
}

fn parse_input(input: SpendingInput, spending_id: EntityId) -> Result<Spending, ApplicationError> {
    let total = input
        .total
        .parse::<Decimal>()
        .map_err(|_| ValidationError::InvalidField { field: "total" })?;
    let currency = input
        .currency
        .parse::<Currency>()
        .map_err(|_| ValidationError::InvalidField { field: "currency" })?;
    let spending_type =
        input
            .spending_type
            .parse::<SpendingType>()
            .map_err(|_| ValidationError::InvalidField {
                field: "spending type",
            })?;
    let spent_date = NaiveDate::parse_from_str(&input.spent_date, "%Y-%m-%d").map_err(|_| {
        ValidationError::InvalidField {
            field: "spent date",
        }
    })?;
    let parse_allocations = |values: Vec<(EntityId, String)>, field: &'static str| {
        values
            .into_iter()
            .map(|(participant_id, amount)| {
                if participant_id <= 0 {
                    return Err(ValidationError::InvalidParticipantId.into());
                }
                Ok(Allocation {
                    participant_id,
                    amount: amount
                        .parse::<Decimal>()
                        .map_err(|_| ValidationError::InvalidField { field })?,
                })
            })
            .collect::<Result<Vec<_>, ApplicationError>>()
    };
    let payers = match input.payers {
        PayerInput::Single(participant_id) if participant_id > 0 => vec![Allocation {
            participant_id,
            amount: total,
        }],
        PayerInput::Single(_) => return Err(ValidationError::InvalidParticipantId.into()),
        PayerInput::Exact(values) => parse_allocations(values, "paid amount")?,
    };
    let shares = match input.shares {
        ShareInput::Equal(ids) => equal_split(total, currency, &ids)?,
        ShareInput::Exact(values) => parse_allocations(values, "owed amount")?,
    };
    let spending = Spending {
        id: spending_id,
        group_id: input.group_id,
        description: Description::new(input.description)?,
        total,
        currency,
        spending_type,
        spent_date,
        payers,
        shares,
    };
    spending.validate()?;
    Ok(spending)
}

async fn validate_eligible(
    eligibility: &dyn SpendingEligibilityReader,
    group_id: EntityId,
    spending: &Spending,
) -> Result<(), ApplicationError> {
    let eligible = eligibility.eligible_participant_ids(group_id).await?;
    if spending
        .payers
        .iter()
        .chain(&spending.shares)
        .any(|allocation| !eligible.contains(&allocation.participant_id))
    {
        return Err(ValidationError::InvalidParticipantId.into());
    }
    Ok(())
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

    async fn spending_page(
        &self,
        group_id: EntityId,
        cursor: Option<SpendingCursor>,
    ) -> Result<SpendingPage, ApplicationError> {
        self.reader.spending_page(group_id, cursor).await
    }

    async fn create_input(&self, input: SpendingInput) -> Result<Spending, ApplicationError> {
        let spending = parse_input(input, 0)?;
        validate_eligible(self.eligibility.as_ref(), spending.group_id, &spending).await?;
        self.repository.create_spending(spending).await
    }

    async fn update_input(
        &self,
        spending_id: EntityId,
        input: SpendingInput,
    ) -> Result<Spending, ApplicationError> {
        let spending = parse_input(input, spending_id)?;
        validate_eligible(self.eligibility.as_ref(), spending.group_id, &spending).await?;
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
    use std::collections::BTreeSet;
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;
    use chrono::NaiveDate;
    use debtor_domain::model::{Allocation, EntityId, ValidationError};
    use rust_decimal::Decimal;

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

        async fn spending_page(
            &self,
            _: EntityId,
            _: Option<SpendingCursor>,
        ) -> Result<SpendingPage, ApplicationError> {
            Ok(SpendingPage {
                items: Vec::new(),
                older: None,
                newer: None,
            })
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

    #[async_trait]
    impl SpendingEligibilityReader for SpendingFake {
        async fn eligible_participant_ids(
            &self,
            _: EntityId,
        ) -> Result<BTreeSet<EntityId>, ApplicationError> {
            Ok([PARTICIPANT_ONE, PARTICIPANT_TWO].into_iter().collect())
        }
    }

    fn equal_input() -> SpendingInput {
        SpendingInput {
            group_id: GROUP_ID,
            description: "  Dinner  ".into(),
            total: "10.01".into(),
            currency: "USD".into(),
            spending_type: "food".into(),
            spent_date: date(5).to_string(),
            payers: PayerInput::Single(PARTICIPANT_ONE),
            shares: ShareInput::Equal(vec![PARTICIPANT_TWO, PARTICIPANT_ONE]),
        }
    }

    fn exact_input() -> SpendingInput {
        SpendingInput {
            group_id: GROUP_ID,
            description: "Taxi".into(),
            total: "10.00".into(),
            currency: "USD".into(),
            spending_type: "transport".into(),
            spent_date: date(6).to_string(),
            payers: PayerInput::Exact(vec![
                (PARTICIPANT_ONE, "4.00".into()),
                (PARTICIPANT_TWO, "6.00".into()),
            ]),
            shares: ShareInput::Exact(vec![
                (PARTICIPANT_ONE, "4.00".into()),
                (PARTICIPANT_TWO, "6.00".into()),
            ]),
        }
    }

    #[tokio::test]
    async fn spending_service_parses_raw_input_for_create_update_and_scopes_reads() {
        let fake = Arc::new(SpendingFake {
            listed_groups: Mutex::new(Vec::new()),
            read_requests: Mutex::new(Vec::new()),
            created: Mutex::new(Vec::new()),
            updated: Mutex::new(Vec::new()),
            fail_update: false,
        });
        let service = SpendingService::new(fake.clone(), fake.clone(), fake.clone());

        service.create_input(equal_input()).await.unwrap();
        service.create_input(exact_input()).await.unwrap();
        service.update_input(99, equal_input()).await.unwrap();
        service.update_input(100, exact_input()).await.unwrap();
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
        assert_eq!(
            created[1].shares,
            vec![
                allocation(PARTICIPANT_ONE, 400),
                allocation(PARTICIPANT_TWO, 600)
            ]
        );
        let updated = fake.updated.lock().unwrap();
        assert_eq!(updated[0].id, 99);
        assert_eq!(updated[1].id, 100);
        assert_eq!(*fake.read_requests.lock().unwrap(), vec![(GROUP_ID, 77)]);
        assert_eq!(*fake.listed_groups.lock().unwrap(), vec![GROUP_ID]);
    }

    #[tokio::test]
    async fn spending_input_covers_all_payer_and_share_modes_and_validates_selection() {
        let fake = Arc::new(SpendingFake {
            listed_groups: Mutex::new(Vec::new()),
            read_requests: Mutex::new(Vec::new()),
            created: Mutex::new(Vec::new()),
            updated: Mutex::new(Vec::new()),
            fail_update: false,
        });
        let service = SpendingService::new(fake.clone(), fake.clone(), fake);

        let mut equal_multiple_payers = equal_input();
        equal_multiple_payers.payers = PayerInput::Exact(vec![(PARTICIPANT_ONE, "10.01".into())]);
        service.create_input(equal_multiple_payers).await.unwrap();

        let mut exact_single_payer = exact_input();
        exact_single_payer.payers = PayerInput::Single(PARTICIPANT_ONE);
        service.create_input(exact_single_payer).await.unwrap();

        let mut duplicate_payers = equal_input();
        duplicate_payers.payers = PayerInput::Exact(vec![
            (PARTICIPANT_ONE, "5.00".into()),
            (PARTICIPANT_ONE, "5.01".into()),
        ]);
        assert!(matches!(
            service.create_input(duplicate_payers).await,
            Err(ApplicationError::Validation(
                ValidationError::DuplicateParticipant { .. }
            ))
        ));

        let mut empty_shares = exact_input();
        empty_shares.shares = ShareInput::Exact(Vec::new());
        assert!(matches!(
            service.create_input(empty_shares).await,
            Err(ApplicationError::Validation(
                ValidationError::EmptyAllocations { field: "share" }
            ))
        ));

        let mut invalid_id = equal_input();
        invalid_id.shares = ShareInput::Equal(vec![-1]);
        assert!(matches!(
            service.create_input(invalid_id).await,
            Err(ApplicationError::Validation(
                ValidationError::InvalidParticipantId
            ))
        ));
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
        let error = SpendingService::new(fake.clone(), fake.clone(), fake)
            .update_input(9, exact_input())
            .await
            .unwrap_err();
        assert!(matches!(error, ApplicationError::Conflict));
    }
}
