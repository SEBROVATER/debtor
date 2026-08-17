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

/// Participant ownership snapshot bound to a history-free Group deletion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupDeleteInput {
    /// Group being deleted.
    pub group_id: EntityId,
    /// Owned Participant IDs disclosed by the confirmation page.
    pub participant_ids: Vec<EntityId>,
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
    /// Archives an active Group.
    async fn archive_group(&self, id: EntityId) -> Result<(), ApplicationError>;
    /// Restores an archived Group.
    async fn restore_group(&self, id: EntityId) -> Result<(), ApplicationError>;
    /// Deletes an empty group.
    async fn delete_empty_group(&self, input: GroupDeleteInput) -> Result<(), ApplicationError>;
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
    /// Archives an active Group.
    async fn archive_group(&self, id: EntityId) -> Result<(), ApplicationError>;
    /// Restores an archived Group.
    async fn restore_group(&self, id: EntityId) -> Result<(), ApplicationError>;
    /// Deletes an empty group.
    async fn delete_empty(&self, input: GroupDeleteInput) -> Result<(), ApplicationError>;
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

    /// Archives an active Group under the shared mutation owner.
    fn archive_group(
        &self,
        id: EntityId,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<(), ApplicationError>> + Send + '_>,
    >;

    /// Restores an archived Group under the shared mutation owner.
    fn restore_group(
        &self,
        id: EntityId,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<(), ApplicationError>> + Send + '_>,
    >;

    /// Deletes a history-free Group under the shared mutation owner.
    fn delete_empty_group(
        &self,
        input: GroupDeleteInput,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<(), ApplicationError>> + Send + '_>,
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

    async fn archive_group(&self, id: EntityId) -> Result<(), ApplicationError> {
        if self.reader.group(id).await?.is_archived {
            return Err(ApplicationError::Conflict);
        }
        self.repository.archive_group(id).await
    }

    async fn restore_group(&self, id: EntityId) -> Result<(), ApplicationError> {
        if !self.reader.group(id).await?.is_archived {
            return Err(ApplicationError::Conflict);
        }
        self.repository.restore_group(id).await
    }

    async fn delete_empty(&self, input: GroupDeleteInput) -> Result<(), ApplicationError> {
        if self.reader.group(input.group_id).await?.is_archived {
            return Err(ApplicationError::Conflict);
        }
        self.repository.delete_empty_group(input).await
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
        group_archived: bool,
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
            let mut value = group(id);
            value.is_archived = self.group_archived;
            Ok(value)
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

        async fn archive_group(&self, _: EntityId) -> Result<(), ApplicationError> {
            Ok(())
        }

        async fn restore_group(&self, _: EntityId) -> Result<(), ApplicationError> {
            Ok(())
        }

        async fn delete_empty_group(&self, _: GroupDeleteInput) -> Result<(), ApplicationError> {
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
            group_archived: false,
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
            group_archived: false,
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

    #[tokio::test]
    async fn lifecycle_intent_rejects_the_wrong_group_state_before_repository_access() {
        let archived = Arc::new(Fake {
            listed_archived: Mutex::new(Vec::new()),
            created: Mutex::new(Vec::new()),
            updated: Mutex::new(Vec::new()),
            fail_create: false,
            group_archived: true,
        });
        let archived_service = GroupService::new(archived.clone(), archived.clone());
        assert!(matches!(
            archived_service.archive_group(1).await,
            Err(ApplicationError::Conflict)
        ));
        archived_service
            .restore_group(1)
            .await
            .expect("restore archived");

        let active = Arc::new(Fake {
            listed_archived: Mutex::new(Vec::new()),
            created: Mutex::new(Vec::new()),
            updated: Mutex::new(Vec::new()),
            fail_create: false,
            group_archived: false,
        });
        let active_service = GroupService::new(active.clone(), active.clone());
        assert!(matches!(
            active_service.restore_group(1).await,
            Err(ApplicationError::Conflict)
        ));
        active_service
            .delete_empty(GroupDeleteInput {
                group_id: 1,
                participant_ids: Vec::new(),
            })
            .await
            .expect("delete active group");
    }
}
