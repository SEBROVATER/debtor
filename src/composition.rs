use std::sync::Arc;

use anyhow::{Context, Result};
use axum::error_handling::HandleErrorLayer;
use debtor_application::{
    AuthenticationService, AuthenticationUseCases, Clock, DebtService, DebtUseCases, GroupReader,
    GroupRepository, GroupService, GroupUseCases, LedgerSnapshotReader, ParticipantRepository,
    ParticipantService, ParticipantUseCases, ReadinessService, ReadinessUseCases, SpendingReader,
    SpendingRepository, SpendingService, SpendingUseCases, UtcClock,
};
use debtor_infra::auth::{ArgonPasswordGate, MemoryLoginAttemptLimiter};
use debtor_infra::db::repos::SqliteLedgerRuntime;
use debtor_infra::exchange_rates::FrankfurterClient;
use debtor_web::session;
use debtor_web::session_store::ReapingMemoryStore;
use debtor_web::state::{AppState, TrustedProxyConfig};
use sqlx::SqlitePool;
use tokio::sync::Semaphore;
use tower::limit::concurrency::GlobalConcurrencyLimitLayer;
use tower::{BoxError, ServiceBuilder};
use tower_http::services::ServeDir;
use tower_http::timeout::TimeoutLayer;
use tower_sessions::SessionManagerLayer;

use crate::config::Config;
use crate::runtime::CleanupHealth;

pub(crate) struct BuiltApp {
    pub(crate) app: axum::Router,
    pub(crate) pool: SqlitePool,
    pub(crate) session_store: ReapingMemoryStore,
    pub(crate) cleanup_health: CleanupHealth,
}

pub(crate) async fn build_app(config: Config) -> Result<BuiltApp> {
    let proxy =
        TrustedProxyConfig::parse(&config.trusted_proxy_cidrs, &config.trusted_proxy_header)
            .map_err(anyhow::Error::msg)?;
    let password = Arc::new(ArgonPasswordGate::new(config.password_hash)?);
    tracing::info!(
        target: "debtor.startup",
        event = "startup_stage",
        stage = "configuration_validated",
    );
    let pool = debtor_infra::db::connect(&config.database_url)
        .await
        .context("unable to connect SQLite")?;
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .context("unable to apply SQLite migrations")?;
    tracing::info!(
        target: "debtor.startup",
        event = "startup_stage",
        stage = "migrations_complete",
    );

    let database = SqliteLedgerRuntime::new(pool.clone());
    let store = Arc::new(database.store());
    let cleanup_health = CleanupHealth::new();
    let readiness: Arc<dyn ReadinessUseCases> = Arc::new(ReadinessService::with_supervisor(
        store.clone(),
        Arc::new(cleanup_health.clone()),
    ));
    let group_reader: Arc<dyn GroupReader> = store.clone();
    let group_repository: Arc<dyn GroupRepository> = store.clone();
    let participant_repository: Arc<dyn ParticipantRepository> = store.clone();
    let spending_reader: Arc<dyn SpendingReader> = store.clone();
    let snapshot_reader: Arc<dyn LedgerSnapshotReader> = store.clone();
    let spending_repository: Arc<dyn SpendingRepository> = store;
    let groups: Arc<dyn GroupUseCases> =
        Arc::new(GroupService::new(group_reader.clone(), group_repository));
    let participants: Arc<dyn ParticipantUseCases> =
        Arc::new(ParticipantService::new(participant_repository.clone()));
    let spendings: Arc<dyn SpendingUseCases> = Arc::new(
        SpendingService::new(spending_reader.clone(), spending_repository)
            .with_eligibility(participant_repository),
    );
    let rates = Arc::new(FrankfurterClient::with_base_url(&config.exchange_base_url));
    let clock: Arc<dyn Clock> = Arc::new(UtcClock);
    let debts: Arc<dyn DebtUseCases> =
        Arc::new(DebtService::new(snapshot_reader, rates, clock.clone()));
    let limiter = Arc::new(MemoryLoginAttemptLimiter::default());
    let authentication: Arc<dyn AuthenticationUseCases> =
        Arc::new(AuthenticationService::new(limiter, password));
    let state = AppState {
        groups,
        participants,
        spendings,
        debts,
        authentication,
        clock,
        readiness,
        proxy,
    };
    let session_store = ReapingMemoryStore::default();
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
        .nest_service("/static", static_service);
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
    })
}
