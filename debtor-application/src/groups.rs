use std::sync::Arc;

use async_trait::async_trait;
use debtor_domain::currency::Currency;
use debtor_domain::model::{EntityId, Group, Name};

use crate::ApplicationError;

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
        service
            .create_group("  Summer trip  ".into(), Currency::Eur)
            .await
            .expect("create group");
        service
            .update_group(7, "  Updated trip  ".into(), Currency::Usd)
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
            fake.updated.lock().expect("updated lock")[0].1.as_str(),
            "Updated trip"
        );

        let failing = Arc::new(Fake {
            listed_archived: Mutex::new(Vec::new()),
            created: Mutex::new(Vec::new()),
            updated: Mutex::new(Vec::new()),
            fail_create: true,
        });
        let error = GroupService::new(failing.clone(), failing)
            .create_group("Trip".into(), Currency::Usd)
            .await
            .expect_err("storage error");
        assert!(matches!(
            error,
            ApplicationError::Storage(StorageReason::Unexpected)
        ));
    }
}
