use std::sync::Arc;

use async_trait::async_trait;
use debtor_domain::model::{Color, EntityId, GroupMember, Name, Participant};

use crate::ApplicationError;

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
    impl ParticipantRepository for Fake {
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
                .expect("created participants lock")
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
        let service = ParticipantService::new(fake.clone());

        service
            .create_participant("  Ada  ".into(), "#aabbcc".into())
            .await
            .expect("create participant");
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
}
