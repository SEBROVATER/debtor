use std::sync::Arc;

use async_trait::async_trait;
use debtor_domain::currency::Currency;
use debtor_domain::model::{EntityId, Group, Name, Participant};

use crate::{ApplicationError, ParticipantCreateInput, ParticipantUpdateInput};

/// Transport-neutral raw input for creating a Group.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupCreateInput {
    /// Group name before domain normalization.
    pub name: String,
}

/// Validates a raw Group creation command without performing a mutation.
///
/// # Errors
///
/// Returns the domain validation error when the trimmed name is empty or too long.
pub fn validate_group_create(input: &GroupCreateInput) -> Result<(), ApplicationError> {
    Name::new(input.name.clone())
        .map(|_| ())
        .map_err(Into::into)
}

/// Transport-neutral raw input for updating Group metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupInput {
    /// Group name before domain normalization.
    pub name: String,
    /// Settlement currency code before parsing.
    pub currency: String,
}

/// Validates raw Group settings without performing a mutation.
///
/// # Errors
///
/// Returns a validation error when the name or currency is invalid.
pub fn validate_group_update(input: &GroupInput) -> Result<(), ApplicationError> {
    Name::new(input.name.clone())?;
    input.currency.parse::<Currency>().map(|_| ()).map_err(|_| {
        debtor_domain::model::ValidationError::InvalidField { field: "currency" }.into()
    })
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

/// Inbound group operations.
#[async_trait]
pub trait GroupUseCases: Send + Sync {
    /// Lists groups.
    async fn list_groups(&self, archived: bool) -> Result<Vec<Group>, ApplicationError>;
    /// Loads one group.
    async fn group(&self, id: EntityId) -> Result<Group, ApplicationError>;
    /// Creates a group.
    async fn create_group(&self, input: GroupCreateInput) -> Result<Group, ApplicationError>;
    /// Updates a group.
    async fn update_group(
        &self,
        id: EntityId,
        input: GroupInput,
    ) -> Result<Group, ApplicationError>;
    /// Archives or restores a group.
    async fn set_archived(&self, id: EntityId, archived: bool) -> Result<(), ApplicationError>;
    /// Deletes an empty group.
    async fn delete_empty(&self, id: EntityId) -> Result<(), ApplicationError>;
}

/// Executes a Group mutation with an outer runtime-owned definitive outcome.
pub trait GroupMutationExecutor: Send + Sync {
    /// Creates a Group and returns only after its mutation outcome is definitive.
    fn create_group(
        &self,
        input: GroupCreateInput,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Group, ApplicationError>> + Send + '_>,
    >;

    /// Updates Group settings and returns only after its mutation outcome is definitive.
    fn update_group(
        &self,
        id: EntityId,
        input: GroupInput,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Group, ApplicationError>> + Send + '_>,
    >;

    /// Creates a Group-owned Participant under the same supervised mutation owner.
    fn create_group_participant(
        &self,
        input: ParticipantCreateInput,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Participant, ApplicationError>> + Send + '_>,
    >;

    /// Updates a Group-owned active Participant under the same mutation owner.
    fn update_group_participant(
        &self,
        input: ParticipantUpdateInput,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Participant, ApplicationError>> + Send + '_>,
    >;
}

/// Group workflow implementation.
pub struct GroupService {
    reader: Arc<dyn GroupReader>,
    repository: Arc<dyn GroupRepository>,
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

    async fn create_group(&self, input: GroupCreateInput) -> Result<Group, ApplicationError> {
        validate_group_create(&input)?;
        self.repository
            .create_group(Name::new(input.name)?, Currency::Usd)
            .await
    }

    async fn update_group(
        &self,
        id: EntityId,
        input: GroupInput,
    ) -> Result<Group, ApplicationError> {
        if self.reader.group(id).await?.is_archived {
            return Err(ApplicationError::Conflict);
        }
        validate_group_update(&input)?;
        let currency = input.currency.parse::<Currency>().map_err(|_| {
            debtor_domain::model::ValidationError::InvalidField { field: "currency" }
        })?;
        self.repository
            .update_group(id, Name::new(input.name)?, currency)
            .await
    }

    async fn set_archived(&self, id: EntityId, archived: bool) -> Result<(), ApplicationError> {
        self.repository.set_group_archived(id, archived).await
    }

    async fn delete_empty(&self, id: EntityId) -> Result<(), ApplicationError> {
        if self.reader.group(id).await?.is_archived {
            return Err(ApplicationError::Conflict);
        }
        self.repository.delete_empty_group(id).await
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use std::sync::{Arc, Mutex};

    use super::*;
    use crate::StorageReason;

    struct Fake {
        listed_archived: Mutex<Vec<bool>>,
        created: Mutex<Vec<(Name, Currency)>>,
        updated: Mutex<Vec<(EntityId, Name, Currency)>>,
        fail_create: bool,
    }

    fn group(id: EntityId) -> Group {
        Group {
            id,
            name: Name::new("Trip").expect("valid group name"),
            currency: Currency::Usd,
            is_archived: false,
        }
    }

    #[async_trait]
    impl GroupReader for Fake {
        async fn list_groups(&self, archived: bool) -> Result<Vec<Group>, ApplicationError> {
            self.listed_archived
                .lock()
                .expect("listed groups lock")
                .push(archived);
            Ok(vec![group(if archived { 2 } else { 1 })])
        }

        async fn group(&self, id: EntityId) -> Result<Group, ApplicationError> {
            Ok(group(id))
        }
    }

    #[async_trait]
    impl GroupRepository for Fake {
        async fn create_group(
            &self,
            name: Name,
            currency: Currency,
        ) -> Result<Group, ApplicationError> {
            if self.fail_create {
                return Err(ApplicationError::Storage(StorageReason::Unexpected));
            }
            self.created
                .lock()
                .expect("created groups lock")
                .push((name.clone(), currency));
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
                .expect("updated groups lock")
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
    async fn normalizes_writes_scopes_reads_and_propagates_storage_errors() {
        let fake = Arc::new(Fake {
            listed_archived: Mutex::new(Vec::new()),
            created: Mutex::new(Vec::new()),
            updated: Mutex::new(Vec::new()),
            fail_create: false,
        });
        let service = GroupService::new(fake.clone(), fake.clone());

        let groups = service.list_groups(true).await.expect("list groups");
        let created = service
            .create_group(GroupCreateInput {
                name: "  Summer trip  ".into(),
            })
            .await
            .expect("create group");
        assert_eq!(created.id, 1);
        assert!(!created.is_archived);
        service
            .update_group(
                7,
                GroupInput {
                    name: "  Updated trip  ".into(),
                    currency: "USD".into(),
                },
            )
            .await
            .expect("update group");

        assert_eq!(groups[0].id, 2);
        assert_eq!(
            *fake.listed_archived.lock().expect("listed lock"),
            vec![true]
        );
        assert_eq!(
            fake.created.lock().expect("created lock")[0].0.as_str(),
            "Summer trip"
        );
        assert_eq!(
            fake.created.lock().expect("created lock")[0].1,
            Currency::Usd
        );
        assert_eq!(
            fake.updated.lock().expect("updated lock")[0].1.as_str(),
            "Updated trip"
        );

        let invalid = service
            .create_group(GroupCreateInput { name: "   ".into() })
            .await
            .expect_err("empty group name");
        assert!(matches!(
            invalid,
            ApplicationError::Validation(debtor_domain::model::ValidationError::Empty {
                field: "name"
            })
        ));

        let failing = Arc::new(Fake {
            listed_archived: Mutex::new(Vec::new()),
            created: Mutex::new(Vec::new()),
            updated: Mutex::new(Vec::new()),
            fail_create: true,
        });
        let error = GroupService::new(failing.clone(), failing)
            .create_group(GroupCreateInput {
                name: "Trip".into(),
            })
            .await
            .expect_err("storage error");
        assert!(matches!(
            error,
            ApplicationError::Storage(StorageReason::Unexpected)
        ));
    }

    #[test]
    fn validates_group_settings_without_repository_access() {
        validate_group_update(&GroupInput {
            name: "  Renamed  ".into(),
            currency: "EUR".into(),
        })
        .expect("valid settings");
        for currency in Currency::ALL {
            validate_group_update(&GroupInput {
                name: "Trip".into(),
                currency: currency.to_string(),
            })
            .expect("supported currency");
        }
        let overlong_name = "x".repeat(101);
        assert!(matches!(
            validate_group_update(&GroupInput {
                name: overlong_name,
                currency: "USD".into(),
            }),
            Err(ApplicationError::Validation(
                debtor_domain::model::ValidationError::TooLong { field: "name", .. }
            ))
        ));

        let invalid_name = validate_group_update(&GroupInput {
            name: "  ".into(),
            currency: "USD".into(),
        })
        .expect_err("empty name");
        assert!(matches!(
            invalid_name,
            ApplicationError::Validation(debtor_domain::model::ValidationError::Empty {
                field: "name"
            })
        ));

        let invalid_currency = validate_group_update(&GroupInput {
            name: "Trip".into(),
            currency: "usd".into(),
        })
        .expect_err("unknown currency");
        assert!(matches!(
            invalid_currency,
            ApplicationError::Validation(debtor_domain::model::ValidationError::InvalidField {
                field: "currency"
            })
        ));
    }
}
