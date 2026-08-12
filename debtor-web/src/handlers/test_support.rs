#![allow(clippy::expect_used, clippy::fn_params_excessive_bools)]

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use debtor_application::{
    ApplicationError, AuthenticationService, AuthenticationUseCases, Clock, DebtResult,
    DebtUseCases, GroupInput, GroupUseCases, LoginAdmission, LoginAttemptLimiter,
    ParticipantUseCases, PasswordVerifier, RateMode, ReadinessUseCases, SpendingInput,
    SpendingPage, SpendingUseCases, UtcClock,
};
use debtor_domain::{
    currency::Currency,
    model::{Color, Group, GroupMember, Name, Participant, Spending, ValidationError},
};

use crate::state::{AppState, TrustedProxyConfig};
use crate::submission_tokens::AnonymousSubmissionTokenStore;

pub(crate) struct TestState {
    pub(crate) app: AppState,
    pub(crate) groups: Arc<FakeGroups>,
    pub(crate) participants: Arc<FakeParticipants>,
    pub(crate) auth_resets: Arc<AtomicUsize>,
    pub(crate) auth_attempts: Arc<AtomicUsize>,
    pub(crate) password_verifications: Arc<AtomicUsize>,
}

pub(crate) struct FakeGroups {
    pub(crate) group: Group,
    pub(crate) create_validation_error: bool,
    pub(crate) update_validation_error: bool,
    pub(crate) created: Mutex<Vec<(String, Currency)>>,
    pub(crate) updated: Mutex<Vec<(i64, String, Currency)>>,
}

pub(crate) struct FakeParticipants {
    pub(crate) participant: Participant,
    pub(crate) create_validation_error: bool,
    pub(crate) update_validation_error: bool,
    pub(crate) group_create_validation_error: bool,
    pub(crate) created: Mutex<Vec<(String, String)>>,
    pub(crate) updated: Mutex<Vec<(i64, String, String)>>,
    pub(crate) group_created: Mutex<Vec<(i64, String, String)>>,
}

pub(crate) fn state(archived: bool) -> TestState {
    state_with_errors_and_password(
        archived,
        false,
        false,
        false,
        false,
        false,
        LoginAdmission::Allowed,
        true,
    )
}

pub(crate) fn state_with_errors(
    archived: bool,
    group_create_validation_error: bool,
    group_update_validation_error: bool,
    participant_create_validation_error: bool,
    participant_update_validation_error: bool,
    group_create_participant_validation_error: bool,
) -> TestState {
    state_with_errors_and_password(
        archived,
        group_create_validation_error,
        group_update_validation_error,
        participant_create_validation_error,
        participant_update_validation_error,
        group_create_participant_validation_error,
        LoginAdmission::Allowed,
        true,
    )
}

pub(crate) fn state_with_password(valid: bool) -> TestState {
    state_with_errors_and_password(
        false,
        false,
        false,
        false,
        false,
        false,
        LoginAdmission::Allowed,
        valid,
    )
}

pub(crate) fn state_with_login_admission(admission: LoginAdmission) -> TestState {
    state_with_errors_and_password(false, false, false, false, false, false, admission, true)
}

#[allow(clippy::too_many_arguments)]
fn state_with_errors_and_password(
    archived: bool,
    group_create_validation_error: bool,
    group_update_validation_error: bool,
    participant_create_validation_error: bool,
    participant_update_validation_error: bool,
    group_create_participant_validation_error: bool,
    login_admission: LoginAdmission,
    password_valid: bool,
) -> TestState {
    let groups = Arc::new(FakeGroups {
        group: Group {
            id: 1,
            name: Name::new("Trip").expect("valid test group"),
            currency: Currency::Usd,
            is_archived: archived,
        },
        create_validation_error: group_create_validation_error,
        update_validation_error: group_update_validation_error,
        created: Mutex::new(Vec::new()),
        updated: Mutex::new(Vec::new()),
    });
    let participants = Arc::new(FakeParticipants {
        participant: Participant {
            id: 1,
            name: Name::new("Ada").expect("valid test participant"),
            color: Color::new("#123456").expect("valid test color"),
            is_archived: false,
        },
        create_validation_error: participant_create_validation_error,
        update_validation_error: participant_update_validation_error,
        group_create_validation_error: group_create_participant_validation_error,
        created: Mutex::new(Vec::new()),
        updated: Mutex::new(Vec::new()),
        group_created: Mutex::new(Vec::new()),
    });
    let spendings: Arc<dyn SpendingUseCases> = Arc::new(FakeSpendings);
    let debts: Arc<dyn DebtUseCases> = Arc::new(FakeDebts);
    let auth_attempts = Arc::new(AtomicUsize::new(0));
    let password_verifications = Arc::new(AtomicUsize::new(0));
    let password: Arc<dyn PasswordVerifier> = Arc::new(FakePassword {
        valid: password_valid,
        verifications: password_verifications.clone(),
    });
    let auth_resets = Arc::new(AtomicUsize::new(0));
    let limiter: Arc<dyn LoginAttemptLimiter> = Arc::new(FakeLimiter {
        resets: auth_resets.clone(),
        attempts: auth_attempts.clone(),
        admission: login_admission,
    });
    let authentication: Arc<dyn AuthenticationUseCases> =
        Arc::new(AuthenticationService::new(limiter, password));
    let clock: Arc<dyn Clock> = Arc::new(UtcClock);
    let groups_use_cases: Arc<dyn GroupUseCases> = groups.clone();
    let participants_use_cases: Arc<dyn ParticipantUseCases> = participants.clone();
    TestState {
        app: AppState {
            groups: groups_use_cases,
            participants: participants_use_cases,
            spendings,
            debts,
            authentication,
            clock,
            readiness: Arc::new(FakeReadiness { healthy: true }),
            proxy: TrustedProxyConfig::default(),
            submission_tokens: AnonymousSubmissionTokenStore::default(),
        },
        groups,
        participants,
        auth_resets,
        auth_attempts,
        password_verifications,
    }
}

pub(crate) fn state_with_readiness_failure() -> TestState {
    let mut test_state = state(false);
    test_state.app.readiness = Arc::new(FakeReadiness { healthy: false });
    test_state
}

struct FakeReadiness {
    healthy: bool,
}

#[async_trait]
impl ReadinessUseCases for FakeReadiness {
    async fn check(&self) -> Result<(), ApplicationError> {
        if self.healthy {
            Ok(())
        } else {
            Err(ApplicationError::Storage(
                debtor_application::StorageReason::Unexpected,
            ))
        }
    }
}

fn validation_error() -> ApplicationError {
    ApplicationError::Validation(ValidationError::Empty { field: "name" })
}

#[async_trait]
impl GroupUseCases for FakeGroups {
    async fn list_groups(&self, _: bool) -> Result<Vec<Group>, ApplicationError> {
        Ok(vec![self.group.clone()])
    }

    async fn group(&self, _: i64) -> Result<Group, ApplicationError> {
        Ok(self.group.clone())
    }

    async fn create_group(&self, input: GroupInput) -> Result<Group, ApplicationError> {
        if self.create_validation_error {
            return Err(validation_error());
        }
        self.created.lock().expect("group calls lock").push((
            input.name,
            input.currency.parse().map_err(|_| validation_error())?,
        ));
        Ok(self.group.clone())
    }

    async fn update_group(&self, id: i64, input: GroupInput) -> Result<Group, ApplicationError> {
        if self.update_validation_error {
            return Err(validation_error());
        }
        self.updated.lock().expect("group calls lock").push((
            id,
            input.name,
            input.currency.parse().map_err(|_| validation_error())?,
        ));
        Ok(self.group.clone())
    }

    async fn set_archived(&self, _: i64, _: bool) -> Result<(), ApplicationError> {
        Ok(())
    }

    async fn delete_empty(&self, _: i64) -> Result<(), ApplicationError> {
        Ok(())
    }
}

#[async_trait]
impl ParticipantUseCases for FakeParticipants {
    async fn list_participants(&self, _: bool) -> Result<Vec<Participant>, ApplicationError> {
        Ok(vec![self.participant.clone()])
    }

    async fn create_participant(
        &self,
        name: String,
        color: String,
    ) -> Result<Participant, ApplicationError> {
        if self.create_validation_error {
            return Err(validation_error());
        }
        self.created
            .lock()
            .expect("participant calls lock")
            .push((name, color));
        Ok(self.participant.clone())
    }

    async fn participant(&self, _: i64) -> Result<Participant, ApplicationError> {
        Ok(self.participant.clone())
    }

    async fn update_participant(
        &self,
        id: i64,
        name: String,
        color: String,
    ) -> Result<Participant, ApplicationError> {
        if self.update_validation_error {
            return Err(validation_error());
        }
        self.updated
            .lock()
            .expect("participant calls lock")
            .push((id, name, color));
        Ok(self.participant.clone())
    }

    async fn create_group_participant(
        &self,
        group_id: i64,
        name: String,
        color: String,
    ) -> Result<Participant, ApplicationError> {
        if self.group_create_validation_error {
            return Err(validation_error());
        }
        self.group_created
            .lock()
            .expect("participant calls lock")
            .push((group_id, name, color));
        Ok(self.participant.clone())
    }

    async fn set_archived(&self, _: i64, _: bool) -> Result<(), ApplicationError> {
        Ok(())
    }

    async fn members(
        &self,
        group_id: i64,
    ) -> Result<Vec<(Participant, GroupMember)>, ApplicationError> {
        Ok(vec![(
            self.participant.clone(),
            GroupMember {
                group_id,
                participant_id: self.participant.id,
                is_active: true,
            },
        )])
    }

    async fn add_member(&self, _: i64, _: i64) -> Result<(), ApplicationError> {
        Ok(())
    }

    async fn deactivate_member(&self, _: i64, _: i64) -> Result<(), ApplicationError> {
        Ok(())
    }
}

struct FakeSpendings;

#[async_trait]
impl SpendingUseCases for FakeSpendings {
    async fn spending(&self, _: i64, _: i64) -> Result<Spending, ApplicationError> {
        Err(ApplicationError::NotFound)
    }

    async fn spending_page(
        &self,
        _: i64,
        _: Option<debtor_application::SpendingCursor>,
    ) -> Result<SpendingPage, ApplicationError> {
        Ok(SpendingPage {
            items: Vec::new(),
            older: None,
            newer: None,
        })
    }

    async fn create_input(&self, _: SpendingInput) -> Result<Spending, ApplicationError> {
        Err(validation_error())
    }

    async fn update_input(&self, _: i64, _: SpendingInput) -> Result<Spending, ApplicationError> {
        Err(validation_error())
    }

    async fn delete(&self, _: i64, _: i64) -> Result<(), ApplicationError> {
        Ok(())
    }
}

struct FakeDebts;

#[async_trait]
impl DebtUseCases for FakeDebts {
    async fn calculate(&self, _: i64, _: RateMode) -> Result<DebtResult, ApplicationError> {
        Err(ApplicationError::NotFound)
    }
}

struct FakePassword {
    valid: bool,
    verifications: Arc<AtomicUsize>,
}

#[async_trait]
impl PasswordVerifier for FakePassword {
    async fn verify(&self, _: &str) -> Result<bool, ApplicationError> {
        self.verifications.fetch_add(1, Ordering::SeqCst);
        Ok(self.valid)
    }
}

struct FakeLimiter {
    resets: Arc<AtomicUsize>,
    attempts: Arc<AtomicUsize>,
    admission: LoginAdmission,
}

#[async_trait]
impl LoginAttemptLimiter for FakeLimiter {
    async fn reserve(&self, _: std::net::IpAddr) -> LoginAdmission {
        self.attempts.fetch_add(1, Ordering::SeqCst);
        self.admission
    }

    async fn reset(&self, _: std::net::IpAddr) {
        self.resets.fetch_add(1, Ordering::SeqCst);
    }
}
