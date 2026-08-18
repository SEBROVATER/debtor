use std::sync::Arc;

use axum::{error_handling::HandleErrorLayer, middleware};
use axum::{extract::Request, response::Response};
use debtor_application::{
    AuthenticationService, AuthenticationUseCases, Clock, DebtService, DebtUseCases,
    GroupCreateInput, GroupDeleteInput, GroupInput, GroupMutationExecutor, GroupReader,
    GroupRepository, GroupService, GroupUseCases, LedgerSnapshotReader, ParticipantCreateInput,
    ParticipantReader, ParticipantRepository, ParticipantService, ParticipantUpdateInput,
    ParticipantUseCases, ReadinessService, ReadinessUseCases, SpendingEligibilityReader,
    SpendingInput, SpendingMutationExecutor, SpendingReader, SpendingRepository, SpendingService,
    SpendingUseCases, UtcClock,
};
use debtor_infra::auth::{ArgonPasswordGate, MemoryLoginAttemptLimiter};
use debtor_infra::db::repos::SqliteLedgerRuntime;
use debtor_infra::exchange_rates::FrankfurterClient;
use debtor_web::session;
use debtor_web::session_store::ReapingMemoryStore;
use debtor_web::state::{AppState, RuntimeControl, TrustedProxyConfig};
use debtor_web::submission_tokens::SubmissionTokenStore;
use sqlx::SqlitePool;
use tokio::sync::Semaphore;
use tower::limit::concurrency::GlobalConcurrencyLimitLayer;
use tower::{BoxError, ServiceBuilder};
use tower_http::services::ServeDir;
use tower_http::timeout::TimeoutLayer;
use tower_sessions::SessionManagerLayer;

use crate::config::Config;
use crate::runtime::{CleanupHealth, DispatchedMutationRegistry};
use crate::startup_error::StartupError;

pub(crate) struct BuiltApp {
    pub(crate) app: axum::Router,
    pub(crate) pool: SqlitePool,
    pub(crate) session_store: ReapingMemoryStore,
    pub(crate) cleanup_health: CleanupHealth,
    pub(crate) submission_token_store: SubmissionTokenStore,
    pub(crate) runtime: RuntimeControl,
    pub(crate) mutations: DispatchedMutationRegistry,
    #[cfg(test)]
    pub(crate) shutdown_events: crate::runtime::ShutdownEvents,
}

struct RootGroupMutationExecutor {
    groups: Arc<dyn GroupUseCases>,
    participants: Arc<dyn ParticipantUseCases>,
    spendings: Arc<dyn SpendingUseCases>,
    mutations: DispatchedMutationRegistry,
    runtime: RuntimeControl,
}

impl SpendingMutationExecutor for RootGroupMutationExecutor {
    fn create_spending(
        &self,
        input: SpendingInput,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = Result<
                        debtor_application::Spending,
                        debtor_application::ApplicationError,
                    >,
                > + Send
                + '_,
        >,
    > {
        let spendings = self.spendings.clone();
        let mutations = self.mutations.clone();
        let runtime = self.runtime.clone();
        Box::pin(async move {
            let Some(lease) = mutations.try_register() else {
                return Err(debtor_application::ApplicationError::Storage(
                    debtor_application::StorageReason::Contention,
                ));
            };
            let task = tokio::spawn(async move {
                let mut guard = GroupMutationGuard::new(lease, mutations, runtime);
                match spendings.create_input(input).await {
                    Ok(spending) => {
                        guard.committed();
                        Ok(spending)
                    }
                    Err(error) => {
                        guard.rolled_back();
                        Err(error)
                    }
                }
            });
            match task.await {
                Ok(result) => result,
                Err(_) => Err(debtor_application::ApplicationError::Storage(
                    debtor_application::StorageReason::Unexpected,
                )),
            }
        })
    }

    fn update_spending(
        &self,
        spending_id: i64,
        input: SpendingInput,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = Result<
                        debtor_application::Spending,
                        debtor_application::ApplicationError,
                    >,
                > + Send
                + '_,
        >,
    > {
        let spendings = self.spendings.clone();
        let mutations = self.mutations.clone();
        let runtime = self.runtime.clone();
        Box::pin(async move {
            let Some(lease) = mutations.try_register() else {
                return Err(debtor_application::ApplicationError::Storage(
                    debtor_application::StorageReason::Contention,
                ));
            };
            let task = tokio::spawn(async move {
                let mut guard = GroupMutationGuard::new(lease, mutations, runtime);
                match spendings.update_input(spending_id, input).await {
                    Ok(spending) => {
                        guard.committed();
                        Ok(spending)
                    }
                    Err(error) => {
                        guard.rolled_back();
                        Err(error)
                    }
                }
            });
            match task.await {
                Ok(result) => result,
                Err(_) => Err(debtor_application::ApplicationError::Storage(
                    debtor_application::StorageReason::Unknown,
                )),
            }
        })
    }
}

struct GroupMutationGuard {
    lease: Option<crate::runtime::MutationLease>,
    mutations: DispatchedMutationRegistry,
    runtime: RuntimeControl,
    terminal: bool,
}

impl GroupMutationGuard {
    fn new(
        lease: crate::runtime::MutationLease,
        mutations: DispatchedMutationRegistry,
        runtime: RuntimeControl,
    ) -> Self {
        Self {
            lease: Some(lease),
            mutations,
            runtime,
            terminal: false,
        }
    }

    fn committed(&mut self) {
        self.mutations.advance_epoch();
        self.terminal = true;
    }

    fn rolled_back(&mut self) {
        self.terminal = true;
    }
}

impl Drop for GroupMutationGuard {
    fn drop(&mut self) {
        if !self.terminal {
            self.runtime.fail_readiness();
        }
        self.lease.take();
    }
}

impl GroupMutationExecutor for RootGroupMutationExecutor {
    fn create_group(
        &self,
        input: GroupCreateInput,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = Result<
                        debtor_application::Group,
                        debtor_application::ApplicationError,
                    >,
                > + Send
                + '_,
        >,
    > {
        Box::pin(async move {
            let Some(lease) = self.mutations.try_register() else {
                return Err(debtor_application::ApplicationError::Storage(
                    debtor_application::StorageReason::Contention,
                ));
            };
            let groups = self.groups.clone();
            let mutations = self.mutations.clone();
            let runtime = self.runtime.clone();
            let task = tokio::spawn(async move {
                let mut guard = GroupMutationGuard::new(lease, mutations, runtime);
                match groups.create_group(input).await {
                    Ok(group) => {
                        guard.committed();
                        Ok(group)
                    }
                    Err(error) => {
                        guard.rolled_back();
                        Err(error)
                    }
                }
            });
            match task.await {
                Ok(result) => result,
                Err(_) => Err(debtor_application::ApplicationError::Storage(
                    debtor_application::StorageReason::Unexpected,
                )),
            }
        })
    }

    fn update_group(
        &self,
        id: i64,
        input: GroupInput,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = Result<
                        debtor_application::Group,
                        debtor_application::ApplicationError,
                    >,
                > + Send
                + '_,
        >,
    > {
        Box::pin(async move {
            let Some(lease) = self.mutations.try_register() else {
                return Err(debtor_application::ApplicationError::Storage(
                    debtor_application::StorageReason::Contention,
                ));
            };
            let groups = self.groups.clone();
            let mutations = self.mutations.clone();
            let runtime = self.runtime.clone();
            let task = tokio::spawn(async move {
                let mut guard = GroupMutationGuard::new(lease, mutations, runtime);
                match groups.update_group(id, input).await {
                    Ok(group) => {
                        guard.committed();
                        Ok(group)
                    }
                    Err(error) => {
                        guard.rolled_back();
                        Err(error)
                    }
                }
            });
            match task.await {
                Ok(result) => result,
                Err(_) => Err(debtor_application::ApplicationError::Storage(
                    debtor_application::StorageReason::Unexpected,
                )),
            }
        })
    }

    fn archive_group(
        &self,
        id: i64,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<(), debtor_application::ApplicationError>>
                + Send
                + '_,
        >,
    > {
        self.dispatch_unit(move |groups| async move { groups.archive_group(id).await })
    }

    fn restore_group(
        &self,
        id: i64,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<(), debtor_application::ApplicationError>>
                + Send
                + '_,
        >,
    > {
        self.dispatch_unit(move |groups| async move { groups.restore_group(id).await })
    }

    fn delete_empty_group(
        &self,
        input: GroupDeleteInput,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<(), debtor_application::ApplicationError>>
                + Send
                + '_,
        >,
    > {
        self.dispatch_unit(move |groups| async move { groups.delete_empty(input).await })
    }
    fn create_group_participant(
        &self,
        input: ParticipantCreateInput,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = Result<
                        debtor_application::Participant,
                        debtor_application::ApplicationError,
                    >,
                > + Send
                + '_,
        >,
    > {
        Box::pin(async move {
            let Some(lease) = self.mutations.try_register() else {
                return Err(debtor_application::ApplicationError::Storage(
                    debtor_application::StorageReason::Contention,
                ));
            };
            let participants = self.participants.clone();
            let mutations = self.mutations.clone();
            let runtime = self.runtime.clone();
            let task = tokio::spawn(async move {
                let mut guard = GroupMutationGuard::new(lease, mutations, runtime);
                match participants
                    .create_group_participant(input.group_id, input.name, input.color)
                    .await
                {
                    Ok(participant) => {
                        guard.committed();
                        Ok(participant)
                    }
                    Err(error) => {
                        guard.rolled_back();
                        Err(error)
                    }
                }
            });
            match task.await {
                Ok(result) => result,
                Err(_) => Err(debtor_application::ApplicationError::Storage(
                    debtor_application::StorageReason::Unexpected,
                )),
            }
        })
    }

    fn update_group_participant(
        &self,
        input: ParticipantUpdateInput,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = Result<
                        debtor_application::Participant,
                        debtor_application::ApplicationError,
                    >,
                > + Send
                + '_,
        >,
    > {
        Box::pin(async move {
            let Some(lease) = self.mutations.try_register() else {
                return Err(debtor_application::ApplicationError::Storage(
                    debtor_application::StorageReason::Contention,
                ));
            };
            let participants = self.participants.clone();
            let mutations = self.mutations.clone();
            let runtime = self.runtime.clone();
            let task = tokio::spawn(async move {
                let mut guard = GroupMutationGuard::new(lease, mutations, runtime);
                match participants.update_group_participant(input).await {
                    Ok(participant) => {
                        guard.committed();
                        Ok(participant)
                    }
                    Err(error) => {
                        guard.rolled_back();
                        Err(error)
                    }
                }
            });
            match task.await {
                Ok(result) => result,
                Err(_) => Err(debtor_application::ApplicationError::Storage(
                    debtor_application::StorageReason::Unexpected,
                )),
            }
        })
    }
}

impl RootGroupMutationExecutor {
    fn dispatch_unit<F, Fut>(
        &self,
        operation: F,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<(), debtor_application::ApplicationError>>
                + Send
                + '_,
        >,
    >
    where
        F: FnOnce(Arc<dyn GroupUseCases>) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = Result<(), debtor_application::ApplicationError>>
            + Send
            + 'static,
    {
        let groups = self.groups.clone();
        let mutations_owner = self.mutations.clone();
        let runtime_owner = self.runtime.clone();
        Box::pin(async move {
            let Some(lease) = mutations_owner.try_register() else {
                return Err(debtor_application::ApplicationError::Storage(
                    debtor_application::StorageReason::Contention,
                ));
            };
            let task = tokio::spawn(async move {
                let mut guard = GroupMutationGuard::new(lease, mutations_owner, runtime_owner);
                match operation(groups).await {
                    Ok(()) => {
                        guard.committed();
                        Ok(())
                    }
                    Err(error) => {
                        guard.rolled_back();
                        Err(error)
                    }
                }
            });
            match task.await {
                Ok(result) => result,
                Err(_) => Err(debtor_application::ApplicationError::Storage(
                    debtor_application::StorageReason::Unexpected,
                )),
            }
        })
    }
}

async fn static_headers(request: Request, next: axum::middleware::Next) -> Response {
    let mut response = next.run(request).await;
    response.headers_mut().insert(
        axum::http::header::X_CONTENT_TYPE_OPTIONS,
        axum::http::HeaderValue::from_static("nosniff"),
    );
    response
}

#[allow(clippy::too_many_lines)]
#[cfg(test)]
pub(crate) async fn build_app(config: Config) -> Result<BuiltApp, StartupError> {
    build_app_with_control(config, RuntimeControl::default()).await
}

#[allow(clippy::too_many_lines)]
pub(crate) async fn build_app_with_control(
    config: Config,
    runtime_control: RuntimeControl,
) -> Result<BuiltApp, StartupError> {
    let proxy = TrustedProxyConfig::parse_for_environment(
        &config.trusted_proxy_cidrs,
        &config.trusted_proxy_header,
        cfg!(debug_assertions),
    )
    .map_err(|_| StartupError::Configuration)?;
    let password = Arc::new(
        ArgonPasswordGate::new(config.password_hash).map_err(|_| StartupError::Configuration)?,
    );
    let rates = Arc::new(
        FrankfurterClient::try_with_base_url(&config.exchange_base_url)
            .map_err(|_| StartupError::Configuration)?,
    );
    tracing::info!(
        target: "debtor.startup",
        event = "startup_stage",
        stage = "configuration_validated",
    );
    let pool = debtor_infra::db::connect(&config.database_url)
        .await
        .map_err(|_| StartupError::DatabaseConnect)?;
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(|_| StartupError::Migration)?;
    tracing::info!(
        target: "debtor.startup",
        event = "startup_stage",
        stage = "migrations_complete",
    );

    let database = SqliteLedgerRuntime::new(pool.clone());
    let store = Arc::new(database.store());
    let cleanup_health = CleanupHealth::new();
    let mutations = DispatchedMutationRegistry::default();
    let readiness: Arc<dyn ReadinessUseCases> = Arc::new(ReadinessService::with_supervisor(
        store.clone(),
        Arc::new(cleanup_health.clone()),
    ));
    let group_reader: Arc<dyn GroupReader> = store.clone();
    let group_repository: Arc<dyn GroupRepository> = store.clone();
    let participant_repository: Arc<dyn ParticipantRepository> = store.clone();
    let participant_reader: Arc<dyn ParticipantReader> = store.clone();
    let spending_eligibility: Arc<dyn SpendingEligibilityReader> = store.clone();
    let spending_reader: Arc<dyn SpendingReader> = store.clone();
    let snapshot_reader: Arc<dyn LedgerSnapshotReader> = store.clone();
    let spending_repository: Arc<dyn SpendingRepository> = store;
    let groups: Arc<dyn GroupUseCases> =
        Arc::new(GroupService::new(group_reader.clone(), group_repository));
    let participants: Arc<dyn ParticipantUseCases> = Arc::new(ParticipantService::new(
        participant_reader,
        participant_repository,
        group_reader.clone(),
    ));
    let spendings: Arc<dyn SpendingUseCases> = Arc::new(SpendingService::new(
        spending_reader,
        spending_repository,
        spending_eligibility,
    ));
    let mutation_executor = Arc::new(RootGroupMutationExecutor {
        groups: groups.clone(),
        participants: participants.clone(),
        spendings: spendings.clone(),
        mutations: mutations.clone(),
        runtime: runtime_control.clone(),
    });
    let group_mutations: Arc<dyn GroupMutationExecutor> = mutation_executor.clone();
    let spending_mutations: Arc<dyn SpendingMutationExecutor> = mutation_executor;
    let clock: Arc<dyn Clock> = Arc::new(UtcClock);
    let debts: Arc<dyn DebtUseCases> =
        Arc::new(DebtService::new(snapshot_reader, rates, clock.clone()));
    let limiter = Arc::new(MemoryLoginAttemptLimiter::default());
    let authentication: Arc<dyn AuthenticationUseCases> =
        Arc::new(AuthenticationService::new(limiter, password));
    let state = AppState {
        groups,
        group_mutations,
        participants,
        spendings,
        spending_mutations,
        debts,
        authentication,
        clock,
        readiness,
        proxy,
        submission_tokens: SubmissionTokenStore::default(),
        runtime: runtime_control.clone(),
    };
    let session_store = ReapingMemoryStore::default();
    let submission_token_store = state.submission_tokens.clone();
    let sessions = SessionManagerLayer::new(session_store.clone())
        .with_name(config.session_cookie_name)
        .with_secure(config.cookie_secure)
        .with_http_only(true)
        .with_same_site(tower_sessions::cookie::SameSite::Strict)
        .with_path("/")
        .with_always_save(true)
        .with_expiry(session::anonymous_expiry());
    let user_limit = Arc::new(Semaphore::new(64));
    let static_service = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(|_: BoxError| async {
            (
                axum::http::StatusCode::SERVICE_UNAVAILABLE,
                "Service temporarily unavailable.",
            )
        }))
        .load_shed()
        .layer(GlobalConcurrencyLimitLayer::with_semaphore(
            user_limit.clone(),
        ))
        .layer(TimeoutLayer::with_status_code(
            axum::http::StatusCode::GATEWAY_TIMEOUT,
            std::time::Duration::from_secs(30),
        ))
        .service(ServeDir::new("static"));
    let app = debtor_web::router::router_with_sessions(state, sessions, user_limit)
        .layer(middleware::from_fn(static_headers))
        .nest_service("/static", static_service)
        .layer(middleware::from_fn({
            let runtime_control = runtime_control.clone();
            move |request, next| {
                debtor_web::middleware::user_admission_or_probe(
                    runtime_control.clone(),
                    request,
                    next,
                )
            }
        }));
    tracing::info!(
        target: "debtor.startup",
        event = "startup_stage",
        stage = "application_composed",
    );
    Ok(BuiltApp {
        app,
        pool,
        session_store,
        cleanup_health,
        submission_token_store,
        runtime: runtime_control,
        mutations,
        #[cfg(test)]
        shutdown_events: crate::runtime::ShutdownEvents::default(),
    })
}
