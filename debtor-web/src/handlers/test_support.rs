#![allow(clippy::expect_used, clippy::fn_params_excessive_bools)]

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use debtor_application::{
    ApplicationError, AuthenticationService, AuthenticationUseCases, Clock, DebtResult,
    DebtUseCases, GroupCreateInput, GroupDeleteInput, GroupInput, GroupMutationExecutor,
    GroupUseCases, LoginAdmission, LoginAttemptLimiter, MonthlySummary, ParticipantCreateInput,
    ParticipantUpdateInput, ParticipantUseCases, PasswordVerifier, RateMode, ReadinessUseCases,
    SourceCurrencySummary, SourcePayerTotal, SourceSummary, SpendingInput,
    SpendingMutationExecutor, SpendingPage, SpendingUseCases, SummaryUseCases, UtcClock,
};
use debtor_domain::{
    currency::Currency,
    model::{
        Allocation, Color, Description, Group, GroupMember, Name, Participant, Spending,
        SpendingType, ValidationError,
    },
};

use crate::state::{AppState, TrustedProxyConfig};
use crate::submission_tokens::SubmissionTokenStore;

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
    #[allow(dead_code)]
    pub(crate) create_validation_error: bool,
    pub(crate) update_validation_error: bool,
    pub(crate) participant_create_validation_error: bool,
    pub(crate) created: Mutex<Vec<(String, Currency)>>,
    pub(crate) updated: Mutex<Vec<(i64, String, Currency)>>,
    pub(crate) archived: AtomicUsize,
    pub(crate) deleted: AtomicUsize,
}

pub(crate) struct FakeParticipants {
    pub(crate) participant: Participant,
    #[allow(dead_code)]
    pub(crate) create_validation_error: bool,
    pub(crate) update_validation_error: bool,
    pub(crate) created: Mutex<Vec<(String, String)>>,
    pub(crate) updated: Mutex<Vec<(i64, String, String)>>,
    pub(crate) group_created: Mutex<Vec<(i64, String, String)>>,
    pub(crate) archived: AtomicUsize,
    pub(crate) memberships: AtomicUsize,
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

pub(crate) fn state_with_current_debts() -> TestState {
    let mut test_state = state(false);
    test_state.app.debts = Arc::new(CurrentDebts);
    test_state
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
        participant_create_validation_error: group_create_participant_validation_error,
        created: Mutex::new(Vec::new()),
        updated: Mutex::new(Vec::new()),
        archived: AtomicUsize::new(0),
        deleted: AtomicUsize::new(0),
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
        created: Mutex::new(Vec::new()),
        updated: Mutex::new(Vec::new()),
        group_created: Mutex::new(Vec::new()),
        archived: AtomicUsize::new(0),
        memberships: AtomicUsize::new(0),
    });
    let fake_spendings = Arc::new(FakeSpendings);
    let spendings: Arc<dyn SpendingUseCases> = fake_spendings.clone();
    let spending_mutations: Arc<dyn SpendingMutationExecutor> = fake_spendings;
    let debts: Arc<dyn DebtUseCases> = Arc::new(FakeDebts);
    let summaries: Arc<dyn SummaryUseCases> = Arc::new(FakeSummaries);
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
            group_mutations: groups.clone(),
            participants: participants_use_cases,
            spendings,
            spending_mutations,
            debts,
            summaries,
            authentication,
            clock,
            readiness: Arc::new(FakeReadiness { healthy: true }),
            proxy: TrustedProxyConfig::default(),
            submission_tokens: SubmissionTokenStore::default(),
            runtime: crate::state::RuntimeControl::default(),
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

    async fn group(&self, id: i64) -> Result<Group, ApplicationError> {
        if id != self.group.id {
            return Err(ApplicationError::NotFound);
        }
        Ok(self.group.clone())
    }

    async fn create_group(&self, input: GroupCreateInput) -> Result<Group, ApplicationError> {
        if self.create_validation_error {
            return Err(validation_error());
        }
        self.created
            .lock()
            .expect("group calls lock")
            .push((input.name, Currency::Usd));
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

    async fn archive_group(&self, _: i64) -> Result<(), ApplicationError> {
        self.archived.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    async fn restore_group(&self, _: i64) -> Result<(), ApplicationError> {
        self.archived.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    async fn delete_empty(&self, _: GroupDeleteInput) -> Result<(), ApplicationError> {
        self.deleted.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }
}

impl GroupMutationExecutor for FakeGroups {
    fn create_group(
        &self,
        input: GroupCreateInput,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Group, ApplicationError>> + Send + '_>,
    > {
        Box::pin(async move { GroupUseCases::create_group(self, input).await })
    }

    fn update_group(
        &self,
        id: i64,
        input: GroupInput,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Group, ApplicationError>> + Send + '_>,
    > {
        Box::pin(async move { GroupUseCases::update_group(self, id, input).await })
    }

    fn archive_group(
        &self,
        id: i64,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<(), ApplicationError>> + Send + '_>,
    > {
        Box::pin(async move { GroupUseCases::archive_group(self, id).await })
    }

    fn restore_group(
        &self,
        id: i64,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<(), ApplicationError>> + Send + '_>,
    > {
        Box::pin(async move { GroupUseCases::restore_group(self, id).await })
    }

    fn delete_empty_group(
        &self,
        input: GroupDeleteInput,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<(), ApplicationError>> + Send + '_>,
    > {
        Box::pin(async move { GroupUseCases::delete_empty(self, input).await })
    }

    fn create_group_participant(
        &self,
        input: ParticipantCreateInput,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Participant, ApplicationError>> + Send + '_>,
    > {
        Box::pin(async move {
            if self.participant_create_validation_error {
                return Err(validation_error());
            }
            Ok(Participant {
                id: 1,
                name: debtor_domain::model::Name::new(input.name)?,
                color: debtor_domain::model::Color::new(input.color)?,
                is_archived: false,
            })
        })
    }

    fn update_group_participant(
        &self,
        input: ParticipantUpdateInput,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Participant, ApplicationError>> + Send + '_>,
    > {
        Box::pin(async move {
            if self.participant_create_validation_error {
                return Err(validation_error());
            }
            Ok(Participant {
                id: input.participant_id,
                name: Name::new(input.name)?,
                color: Color::new(input.color)?,
                is_archived: false,
            })
        })
    }
}

#[async_trait]
impl ParticipantUseCases for FakeParticipants {
    async fn participant(&self, _: i64) -> Result<Participant, ApplicationError> {
        Ok(self.participant.clone())
    }

    async fn update_group_participant(
        &self,
        input: ParticipantUpdateInput,
    ) -> Result<Participant, ApplicationError> {
        if self.update_validation_error {
            return Err(validation_error());
        }
        self.updated.lock().expect("participant calls lock").push((
            input.participant_id,
            input.name,
            input.color,
        ));
        Ok(self.participant.clone())
    }

    async fn create_group_participant(
        &self,
        group_id: i64,
        name: String,
        color: String,
    ) -> Result<Participant, ApplicationError> {
        self.group_created
            .lock()
            .expect("participant calls lock")
            .push((group_id, name, color));
        Ok(self.participant.clone())
    }

    async fn set_archived(&self, _: i64, _: bool) -> Result<(), ApplicationError> {
        self.archived.fetch_add(1, Ordering::Relaxed);
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
        self.memberships.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    async fn deactivate_member(&self, _: i64, _: i64) -> Result<(), ApplicationError> {
        self.memberships.fetch_add(1, Ordering::Relaxed);
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

    async fn spending_detail(
        &self,
        group_id: i64,
        spending_id: i64,
    ) -> Result<debtor_application::SpendingDetail, ApplicationError> {
        Ok(test_spending_detail(group_id, spending_id))
    }

    async fn spending_history_page(
        &self,
        _: i64,
        _: Option<debtor_application::SpendingCursor>,
    ) -> Result<debtor_application::SpendingHistoryPage, ApplicationError> {
        Ok(debtor_application::SpendingHistoryPage {
            group: Group {
                id: 1,
                name: Name::new("Test Group").expect("group name"),
                currency: Currency::Usd,
                is_archived: false,
            },
            items: Vec::new(),
            older: None,
            newer: None,
        })
    }

    async fn create_input(&self, _: SpendingInput) -> Result<Spending, ApplicationError> {
        Err(validation_error())
    }

    async fn preview_input(&self, _: SpendingInput) -> Result<Spending, ApplicationError> {
        Err(validation_error())
    }

    async fn update_input(&self, _: i64, _: SpendingInput) -> Result<Spending, ApplicationError> {
        Err(validation_error())
    }

    async fn validate_update_input(
        &self,
        _: i64,
        _: SpendingInput,
    ) -> Result<Spending, ApplicationError> {
        Err(validation_error())
    }

    async fn delete(&self, _: i64, _: i64) -> Result<(), ApplicationError> {
        Ok(())
    }
}

fn test_spending_detail(group_id: i64, spending_id: i64) -> debtor_application::SpendingDetail {
    let amount = debtor_domain::money::parse_decimal("10").expect("test amount");
    let participant = Participant {
        id: 1,
        name: Name::new("Ada").expect("participant name"),
        color: Color::new("#123456").expect("participant color"),
        is_archived: false,
    };
    let allocation = Allocation {
        participant_id: participant.id,
        amount,
    };
    let spending = Spending {
        id: spending_id,
        group_id,
        description: Description::new("Dinner").expect("description"),
        total: amount,
        currency: Currency::Usd,
        spending_type: SpendingType::Food,
        spent_date: chrono::NaiveDate::from_ymd_opt(2026, 8, 18).expect("date"),
        payers: vec![allocation.clone()],
        shares: vec![allocation.clone()],
    };
    debtor_application::SpendingDetail {
        group: Group {
            id: group_id,
            name: Name::new("Test Group").expect("group name"),
            currency: Currency::Usd,
            is_archived: false,
        },
        spending,
        payers: vec![(participant.clone(), allocation.clone())],
        shares: vec![(participant, allocation)],
    }
}

impl SpendingMutationExecutor for FakeSpendings {
    fn create_spending(
        &self,
        _: SpendingInput,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Spending, ApplicationError>> + Send + '_>,
    > {
        Box::pin(async { Err(validation_error()) })
    }

    fn update_spending(
        &self,
        _: i64,
        _: SpendingInput,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Spending, ApplicationError>> + Send + '_>,
    > {
        Box::pin(async { Err(validation_error()) })
    }

    fn delete_spending(
        &self,
        _: i64,
        _: i64,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<(), ApplicationError>> + Send + '_>,
    > {
        Box::pin(async { Ok(()) })
    }
}

struct FakeDebts;

struct CurrentDebts;

#[async_trait]
impl DebtUseCases for FakeDebts {
    async fn calculate(&self, _: i64, _: RateMode) -> Result<DebtResult, ApplicationError> {
        Err(ApplicationError::NotFound)
    }
}

#[async_trait]
impl DebtUseCases for CurrentDebts {
    async fn calculate(&self, group_id: i64, _: RateMode) -> Result<DebtResult, ApplicationError> {
        let participant = Participant {
            id: 1,
            name: Name::new("Ada").expect("test participant"),
            color: Color::new("#123456").expect("test color"),
            is_archived: false,
        };
        Ok(DebtResult {
            group_is_archived: false,
            currency: Currency::Usd,
            participants: vec![(
                participant,
                GroupMember {
                    group_id,
                    participant_id: 1,
                    is_active: true,
                },
            )],
            has_spendings: false,
            transfers: Vec::new(),
            balances: std::collections::BTreeMap::from([(
                1,
                debtor_domain::money::parse_decimal("0").expect("zero balance"),
            )]),
            rates: Vec::new(),
            calculated_at: chrono::Utc::now(),
        })
    }
}

struct FakeSummaries;

#[async_trait]
impl SummaryUseCases for FakeSummaries {
    async fn source_summary(&self, _: i64) -> Result<SourceSummary, ApplicationError> {
        let participant = Participant {
            id: 1,
            name: Name::new("Archived Ada").expect("summary participant"),
            color: Color::new("#123456").expect("summary color"),
            is_archived: true,
        };
        Ok(SourceSummary {
            month: chrono::NaiveDate::from_ymd_opt(2026, 8, 1).expect("test month"),
            currencies: vec![
                SourceCurrencySummary {
                    currency: Currency::Eur,
                    total: debtor_domain::money::parse_decimal("12.34").expect("summary total"),
                    display_total: "€12.34 EUR".into(),
                    payers: vec![SourcePayerTotal {
                        participant: participant.clone(),
                        total: debtor_domain::money::parse_decimal("12.34").expect("payer total"),
                        display_total: "€12.34 EUR".into(),
                    }],
                },
                SourceCurrencySummary {
                    currency: Currency::Usd,
                    total: debtor_domain::money::parse_decimal("10").expect("summary total"),
                    display_total: "$10.00 USD".into(),
                    payers: vec![SourcePayerTotal {
                        participant,
                        total: debtor_domain::money::parse_decimal("10").expect("payer total"),
                        display_total: "$10.00 USD".into(),
                    }],
                },
            ],
        })
    }

    async fn converted_summary(
        &self,
        _: i64,
    ) -> Result<debtor_application::ConvertedSummary, ApplicationError> {
        Err(ApplicationError::Unavailable(
            debtor_application::UnavailableReason::ExchangeRates,
        ))
    }

    async fn monthly_summary(&self, group_id: i64) -> Result<MonthlySummary, ApplicationError> {
        Ok(MonthlySummary {
            currency: Currency::Usd,
            source: self.source_summary(group_id).await,
            converted: self.converted_summary(group_id).await,
        })
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
