use std::sync::Arc;

use async_trait::async_trait;
use debtor_domain::model::{Color, EntityId, GroupMember, Name, Participant};

use crate::{ApplicationError, ArchiveAdmission, ArchiveCalculationUseCases, GroupReader};

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

/// Transport-neutral raw input for editing an active Group-owned Participant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParticipantUpdateInput {
    /// Owning Group identifier.
    pub group_id: EntityId,
    /// Stable Participant identifier.
    pub participant_id: EntityId,
    /// Participant name before normalization.
    pub name: String,
    /// Participant color before normalization.
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

/// Validates a raw Group-scoped Participant edit without performing I/O.
///
/// # Errors
///
/// Returns a domain validation error when an identifier, name, or color is invalid.
pub fn validate_participant_update(input: &ParticipantUpdateInput) -> Result<(), ApplicationError> {
    Name::new(input.name.clone())?;
    Color::new(input.color.clone())?;
    if input.group_id <= 0 || input.participant_id <= 0 {
        return Err(debtor_domain::model::ValidationError::InvalidField {
            field: "participant_id",
        }
        .into());
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
    async fn update_group_participant(
        &self,
        group_id: EntityId,
        id: EntityId,
        name: Name,
        color: Color,
    ) -> Result<Participant, ApplicationError>;
    /// Archives one active Participant belonging to the supplied active Group.
    async fn archive_group_participant(
        &self,
        group_id: EntityId,
        participant_id: EntityId,
        admission: ArchiveAdmission,
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
    async fn update_group_participant(
        &self,
        input: ParticipantUpdateInput,
    ) -> Result<Participant, ApplicationError>;
    /// Creates and joins a participant in one transaction.
    async fn create_group_participant(
        &self,
        group_id: EntityId,
        name: String,
        color: String,
    ) -> Result<Participant, ApplicationError>;
    /// Archives an active Group-owned Participant only at an exact Historical zero balance.
    async fn archive_group_participant(
        &self,
        group_id: EntityId,
        participant_id: EntityId,
    ) -> Result<(), ApplicationError>;
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
    debts: Arc<dyn ArchiveCalculationUseCases>,
}

impl ParticipantService {
    /// Creates a service with injected persistence.
    pub fn new(
        reader: Arc<dyn ParticipantReader>,
        repository: Arc<dyn ParticipantRepository>,
        groups: Arc<dyn GroupReader>,
        debts: Arc<dyn ArchiveCalculationUseCases>,
    ) -> Self {
        Self {
            reader,
            repository,
            groups,
            debts,
        }
    }
}

#[async_trait]
impl ParticipantUseCases for ParticipantService {
    async fn participant(&self, id: EntityId) -> Result<Participant, ApplicationError> {
        self.reader.participant(id).await
    }

    async fn update_group_participant(
        &self,
        input: ParticipantUpdateInput,
    ) -> Result<Participant, ApplicationError> {
        validate_participant_update(&input)?;
        if self.groups.group(input.group_id).await?.is_archived {
            return Err(ApplicationError::Conflict);
        }
        let member = self
            .reader
            .group_members(input.group_id)
            .await?
            .into_iter()
            .find(|(participant, membership)| {
                participant.id == input.participant_id
                    && membership.is_active
                    && !participant.is_archived
            })
            .ok_or(ApplicationError::NotFound)?;
        self.repository
            .update_group_participant(
                input.group_id,
                member.0.id,
                Name::new(input.name)?,
                Color::new(input.color)?,
            )
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

    async fn archive_group_participant(
        &self,
        group_id: EntityId,
        participant_id: EntityId,
    ) -> Result<(), ApplicationError> {
        if group_id <= 0 || participant_id <= 0 || self.groups.group(group_id).await?.is_archived {
            return Err(ApplicationError::Conflict);
        }
        let result = self.debts.calculate_archive(group_id).await?;
        if result.capture.snapshot.group.is_archived
            || result.balances.get(&participant_id) != Some(&rust_decimal::Decimal::ZERO)
        {
            return Err(ApplicationError::Conflict);
        }
        self.repository
            .archive_group_participant(
                group_id,
                participant_id,
                ArchiveAdmission {
                    generation: result.capture.generation,
                    utc_date: result.calculated_at.date_naive(),
                    quotes: result.quotes,
                },
            )
            .await
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

    struct FakeDebts;

    #[async_trait]
    impl ArchiveCalculationUseCases for FakeDebts {
        async fn calculate_archive(
            &self,
            _: EntityId,
        ) -> Result<crate::ArchiveCalculation, ApplicationError> {
            Err(ApplicationError::NotFound)
        }
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
            Ok(vec![(
                participant(PARTICIPANT_ONE),
                GroupMember {
                    group_id,
                    participant_id: PARTICIPANT_ONE,
                    is_active: true,
                },
            )])
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

        async fn update_group_participant(
            &self,
            _group_id: EntityId,
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

        async fn archive_group_participant(
            &self,
            _: EntityId,
            _: EntityId,
            _: ArchiveAdmission,
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
        let service = ParticipantService::new(
            fake.clone(),
            fake.clone(),
            fake.clone(),
            Arc::new(FakeDebts),
        );

        service
            .create_group_participant(GROUP_ID, "  Ada  ".into(), "#aabbcc".into())
            .await
            .expect("create group participant");
        service
            .update_group_participant(ParticipantUpdateInput {
                group_id: GROUP_ID,
                participant_id: PARTICIPANT_ONE,
                name: "  Grace  ".into(),
                color: "#abcdef".into(),
            })
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
            vec![GROUP_ID, GROUP_ID]
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

    #[test]
    fn validates_group_scoped_edit_before_any_io() {
        let valid = ParticipantUpdateInput {
            group_id: GROUP_ID,
            participant_id: PARTICIPANT_ONE,
            name: "  Grace  ".into(),
            color: "#abcdef".into(),
        };
        validate_participant_update(&valid).expect("valid edit");
        for input in [
            ParticipantUpdateInput {
                name: "  ".into(),
                ..valid.clone()
            },
            ParticipantUpdateInput {
                name: "x".repeat(101),
                ..valid.clone()
            },
            ParticipantUpdateInput {
                color: "#abc".into(),
                ..valid.clone()
            },
            ParticipantUpdateInput {
                participant_id: 0,
                ..valid.clone()
            },
        ] {
            assert!(validate_participant_update(&input).is_err());
        }
    }
}
