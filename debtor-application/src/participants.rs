use std::sync::Arc;

use async_trait::async_trait;
use debtor_domain::model::{Color, EntityId, GroupMember, Name, Participant};

use crate::{ApplicationError, GroupReader};

/// Transport-neutral raw input for creating a Group-owned Participant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParticipantCreateInput {
    /// Owning Group identifier.
    pub group_id: EntityId,
    /// Participant name before domain normalization.
    pub name: String,
    /// Participant color before domain normalization.
    pub color: String,
}

/// Validates Participant fields without performing I/O or a mutation.
///
/// # Errors
///
/// Returns the domain validation error for an invalid name or color.
pub fn validate_participant_create(input: &ParticipantCreateInput) -> Result<(), ApplicationError> {
    Name::new(input.name.clone())?;
    Color::new(input.color.clone())?;
    if input.group_id <= 0 {
        return Err(
            debtor_domain::model::ValidationError::InvalidField { field: "group_id" }.into(),
        );
    }
    Ok(())
}

/// Reads participant identities and group memberships.
#[async_trait]
pub trait ParticipantReader: Send + Sync {
    /// Loads one participant.
    async fn participant(&self, id: EntityId) -> Result<Participant, ApplicationError>;
    /// Lists group memberships with participant data.
    async fn group_members(
        &self,
        group_id: EntityId,
    ) -> Result<Vec<(Participant, GroupMember)>, ApplicationError>;
}

/// Writes participant identities and memberships.
#[async_trait]
pub trait ParticipantRepository: Send + Sync {
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

/// Inbound participant and membership operations.
#[async_trait]
pub trait ParticipantUseCases: Send + Sync {
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
    reader: Arc<dyn ParticipantReader>,
    repository: Arc<dyn ParticipantRepository>,
    groups: Arc<dyn GroupReader>,
}

impl ParticipantService {
    /// Creates a service with injected persistence.
    pub fn new(
        reader: Arc<dyn ParticipantReader>,
        repository: Arc<dyn ParticipantRepository>,
        groups: Arc<dyn GroupReader>,
    ) -> Self {
        Self {
            reader,
            repository,
            groups,
        }
    }
}

#[async_trait]
impl ParticipantUseCases for ParticipantService {
    async fn participant(&self, id: EntityId) -> Result<Participant, ApplicationError> {
        self.reader.participant(id).await
    }

    async fn update_participant(
        &self,
        id: EntityId,
        name: String,
        color: String,
    ) -> Result<Participant, ApplicationError> {
        if self.reader.participant(id).await?.is_archived {
            return Err(ApplicationError::Conflict);
        }
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
        if self.groups.group(group_id).await?.is_archived {
            return Err(ApplicationError::Conflict);
        }
        validate_participant_create(&ParticipantCreateInput {
            group_id,
            name: name.clone(),
            color: color.clone(),
        })?;
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
        self.reader.group_members(group_id).await
    }

    async fn add_member(
        &self,
        group_id: EntityId,
        participant_id: EntityId,
    ) -> Result<(), ApplicationError> {
        if self.groups.group(group_id).await?.is_archived
            || self.reader.participant(participant_id).await?.is_archived
        {
            return Err(ApplicationError::Conflict);
        }
        self.repository.add_member(group_id, participant_id).await
    }

    async fn deactivate_member(
        &self,
        group_id: EntityId,
        participant_id: EntityId,
    ) -> Result<(), ApplicationError> {
        if self.groups.group(group_id).await?.is_archived {
            return Err(ApplicationError::Conflict);
        }
        self.repository
            .set_member_active(group_id, participant_id, false)
            .await
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use std::sync::{Arc, Mutex};

    use super::*;

    const GROUP_ID: EntityId = 10;
    const PARTICIPANT_ONE: EntityId = 1;

    struct Fake {
        created: Mutex<Vec<(Name, Color)>>,
        updated: Mutex<Vec<(EntityId, Name, Color)>>,
        member_requests: Mutex<Vec<EntityId>>,
        deactivated: Mutex<Vec<(EntityId, EntityId, bool)>>,
    }

    fn participant(id: EntityId) -> Participant {
        Participant {
            id,
            name: Name::new("Ada").expect("valid participant name"),
            color: Color::new("#123456").expect("valid participant color"),
            is_archived: false,
        }
    }

    #[async_trait]
    impl GroupReader for Fake {
        async fn list_groups(
            &self,
            _: bool,
        ) -> Result<Vec<debtor_domain::model::Group>, ApplicationError> {
            Ok(Vec::new())
        }

        async fn group(
            &self,
            id: EntityId,
        ) -> Result<debtor_domain::model::Group, ApplicationError> {
            Ok(debtor_domain::model::Group {
                id,
                name: Name::new("Trip").expect("valid group name"),
                currency: debtor_domain::currency::Currency::Usd,
                is_archived: false,
            })
        }
    }

    #[async_trait]
    impl ParticipantReader for Fake {
        async fn participant(&self, id: EntityId) -> Result<Participant, ApplicationError> {
            Ok(participant(id))
        }

        async fn group_members(
            &self,
            group_id: EntityId,
        ) -> Result<Vec<(Participant, GroupMember)>, ApplicationError> {
            self.member_requests
                .lock()
                .expect("member requests lock")
                .push(group_id);
            Ok(Vec::new())
        }
    }

    #[async_trait]
    impl ParticipantRepository for Fake {
        async fn create_group_participant(
            &self,
            _: EntityId,
            name: Name,
            color: Color,
        ) -> Result<Participant, ApplicationError> {
            self.created
                .lock()
                .expect("created participants lock")
                .push((name.clone(), color.clone()));
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
                .expect("updated participants lock")
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
                .expect("deactivated memberships lock")
                .push((group_id, participant_id, active));
            Ok(())
        }
    }

    #[tokio::test]
    async fn normalizes_writes_and_scopes_membership_actions() {
        let fake = Arc::new(Fake {
            created: Mutex::new(Vec::new()),
            updated: Mutex::new(Vec::new()),
            member_requests: Mutex::new(Vec::new()),
            deactivated: Mutex::new(Vec::new()),
        });
        let service = ParticipantService::new(fake.clone(), fake.clone(), fake.clone());

        service
            .create_group_participant(GROUP_ID, "  Ada  ".into(), "#aabbcc".into())
            .await
            .expect("create group participant");
        service
            .update_participant(3, "  Grace  ".into(), "#abcdef".into())
            .await
            .expect("update participant");
        service.members(GROUP_ID).await.expect("list members");
        service
            .deactivate_member(GROUP_ID, PARTICIPANT_ONE)
            .await
            .expect("deactivate member");

        assert_eq!(
            fake.created.lock().expect("created lock")[0].0.as_str(),
            "Ada"
        );
        assert_eq!(
            fake.created.lock().expect("created lock")[0].1.as_str(),
            "#AABBCC"
        );
        assert_eq!(
            fake.updated.lock().expect("updated lock")[0].1.as_str(),
            "Grace"
        );
        assert_eq!(
            *fake.member_requests.lock().expect("member requests lock"),
            vec![GROUP_ID]
        );
        assert_eq!(
            *fake
                .deactivated
                .lock()
                .expect("deactivated memberships lock"),
            vec![(GROUP_ID, PARTICIPANT_ONE, false)]
        );
    }

    #[test]
    fn validates_group_participant_input_without_side_effects() {
        validate_participant_create(&ParticipantCreateInput {
            group_id: GROUP_ID,
            name: "  Ada  ".into(),
            color: "#aabbcc".into(),
        })
        .expect("valid participant input");

        assert!(matches!(
            validate_participant_create(&ParticipantCreateInput {
                group_id: GROUP_ID,
                name: "  ".into(),
                color: "#aabbcc".into(),
            }),
            Err(ApplicationError::Validation(
                debtor_domain::model::ValidationError::Empty { field: "name" }
            ))
        ));
        assert!(matches!(
            validate_participant_create(&ParticipantCreateInput {
                group_id: GROUP_ID,
                name: "x".repeat(101),
                color: "#aabbcc".into(),
            }),
            Err(ApplicationError::Validation(
                debtor_domain::model::ValidationError::TooLong { field: "name", .. }
            ))
        ));
        assert!(matches!(
            validate_participant_create(&ParticipantCreateInput {
                group_id: GROUP_ID,
                name: "Ada".into(),
                color: "#abc".into(),
            }),
            Err(ApplicationError::Validation(
                debtor_domain::model::ValidationError::InvalidColor
            ))
        ));
    }
}
