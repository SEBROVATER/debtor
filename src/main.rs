//! debtor composition root and local server entry point.

use std::future::Future;
use std::net::SocketAddr;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;

use anyhow::{Context, Result, anyhow};
use axum::error_handling::HandleErrorLayer;
use debtor_application::{
    AuthenticationService, AuthenticationUseCases, Clock, DebtService, DebtUseCases, GroupReader,
    GroupRepository, GroupService, GroupUseCases, LedgerSnapshotReader, ParticipantRepository,
    ParticipantService, ParticipantUseCases, ReadinessService, ReadinessUseCases, SpendingReader,
    SpendingRepository, SpendingService, SpendingUseCases, SupervisorReadiness, UtcClock,
};
use debtor_infra::auth::{ArgonPasswordGate, MemoryLoginAttemptLimiter};
use debtor_infra::db::repos::SqliteLedgerStore;
use debtor_infra::exchange_rates::FrankfurterClient;
use debtor_web::session;
use debtor_web::session_store::ReapingMemoryStore;
use debtor_web::state::{AppState, TrustedProxyConfig};
use sqlx::SqlitePool;
use tokio::sync::{Mutex, Notify, Semaphore};
use tokio::task::JoinHandle;
use tower::limit::concurrency::GlobalConcurrencyLimitLayer;
use tower::{BoxError, ServiceBuilder};
use tower_http::services::ServeDir;
use tower_http::timeout::TimeoutLayer;
use tower_sessions::{ExpiredDeletion, SessionManagerLayer};

const CLEANUP_INTERVAL: Duration = Duration::from_secs(5 * 60);
const HTTP_DRAIN_TIMEOUT: Duration = Duration::from_secs(10);
const CLEANUP_STOP_TIMEOUT: Duration = Duration::from_secs(5);
const WAL_CHECKPOINT_TIMEOUT: Duration = Duration::from_secs(5);
const POOL_CLOSE_TIMEOUT: Duration = Duration::from_secs(5);

mod config;

use config::Config;

#[tokio::main]
async fn main() -> Result<()> {
    match dotenvy::dotenv() {
        Ok(_) => {}
        Err(error) if error.not_found() => {}
        Err(_) => anyhow::bail!("unable to load .env: malformed or unreadable file"),
    }
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "debtor=info,tower_http=info".into()),
        )
        .init();
    let config = Config::from_lookup(|name| std::env::var(name).ok(), cfg!(debug_assertions))?;
    let signals = SignalReceivers::install()?;
    let bind = config.bind;
    let runtime = build_app(config).await?;
    let listener = tokio::net::TcpListener::bind(bind)
        .await
        .context("unable to bind APP_BIND")?;
    tracing::info!(url = %format!("http://{}", bind), "debtor listening");
    run_runtime(runtime, listener, signals).await
}

struct BuiltApp {
    app: axum::Router,
    pool: SqlitePool,
    session_store: ReapingMemoryStore,
    cleanup_health: CleanupHealth,
}

struct WalCheckpoint {
    busy: i64,
    log: i64,
    checkpointed: i64,
}

async fn build_app(config: Config) -> Result<BuiltApp> {
    let proxy =
        TrustedProxyConfig::parse(&config.trusted_proxy_cidrs, &config.trusted_proxy_header)
            .map_err(anyhow::Error::msg)?;
    let password = Arc::new(ArgonPasswordGate::new(config.password_hash)?);
    let pool = debtor_infra::db::connect(&config.database_url)
        .await
        .context("unable to connect SQLite")?;
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .context("unable to apply SQLite migrations")?;

    let store = Arc::new(SqliteLedgerStore::new(pool.clone()));
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
        Arc::new(ParticipantService::new(participant_repository));
    let spendings: Arc<dyn SpendingUseCases> = Arc::new(SpendingService::new(
        spending_reader.clone(),
        spending_repository,
    ));
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
    Ok(BuiltApp {
        app: debtor_web::router::router_with_sessions(state, sessions, user_limit)
            .nest_service("/static", static_service),
        pool,
        session_store,
        cleanup_health,
    })
}

#[derive(Clone)]
struct CleanupHealth(Arc<AtomicBool>);

impl CleanupHealth {
    fn new() -> Self {
        Self(Arc::new(AtomicBool::new(true)))
    }

    fn mark_unhealthy(&self) {
        self.0.store(false, Ordering::Release);
    }
}

impl SupervisorReadiness for CleanupHealth {
    fn is_healthy(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ShutdownTrigger {
    Signal,
    SignalFailure,
    CleanupFailure,
    HttpFailure,
    CheckpointFailure,
    PoolCloseFailure,
}

impl ShutdownTrigger {
    fn is_fatal(self) -> bool {
        !matches!(self, Self::Signal)
    }
}

#[derive(Debug, Default)]
struct ShutdownState {
    first: Option<ShutdownTrigger>,
    fatal_triggers: Vec<ShutdownTrigger>,
}

#[derive(Clone, Default)]
struct ShutdownCoordinator {
    state: Arc<Mutex<ShutdownState>>,
    notify: Arc<Notify>,
}

#[derive(Debug)]
struct ShutdownOutcome {
    first: Option<ShutdownTrigger>,
    fatal_triggers: Vec<ShutdownTrigger>,
}

impl ShutdownCoordinator {
    async fn request(&self, trigger: ShutdownTrigger) {
        let mut state = self.state.lock().await;
        if state.first.is_none() {
            state.first = Some(trigger);
        }
        if trigger.is_fatal() {
            state.fatal_triggers.push(trigger);
        }
        drop(state);
        self.notify.notify_waiters();
    }

    async fn wait(&self) {
        loop {
            let notified = self.notify.notified();
            if self.state.lock().await.first.is_some() {
                return;
            }
            notified.await;
        }
    }

    async fn outcome(&self) -> ShutdownOutcome {
        let state = self.state.lock().await;
        ShutdownOutcome {
            first: state.first,
            fatal_triggers: state.fatal_triggers.clone(),
        }
    }
}

#[cfg(unix)]
struct SignalReceivers {
    interrupt: tokio::signal::unix::Signal,
    terminate: tokio::signal::unix::Signal,
}

#[cfg(unix)]
impl SignalReceivers {
    fn install() -> Result<Self> {
        Ok(Self {
            interrupt: tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())
                .context("unable to register SIGINT handler")?,
            terminate: tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                .context("unable to register SIGTERM handler")?,
        })
    }
}

#[cfg(not(unix))]
struct SignalReceivers;

#[cfg(not(unix))]
impl SignalReceivers {
    fn install() -> Result<Self> {
        Ok(Self)
    }
}

async fn signal_worker(
    #[cfg(unix)] mut signals: SignalReceivers,
    #[cfg(not(unix))] _signals: SignalReceivers,
    coordinator: ShutdownCoordinator,
) {
    #[cfg(unix)]
    {
        tokio::select! {
            () = coordinator.wait() => {}
            value = signals.interrupt.recv() => {
                if value.is_some() {
                    coordinator.request(ShutdownTrigger::Signal).await;
                } else {
                    coordinator.request(ShutdownTrigger::SignalFailure).await;
                }
            }
            value = signals.terminate.recv() => {
                if value.is_some() {
                    coordinator.request(ShutdownTrigger::Signal).await;
                } else {
                    coordinator.request(ShutdownTrigger::SignalFailure).await;
                }
            }
        }
    }
    #[cfg(not(unix))]
    {
        tokio::select! {
            () = coordinator.wait() => {}
            result = tokio::signal::ctrl_c() => {
                if result.is_ok() {
                    coordinator.request(ShutdownTrigger::Signal).await;
                } else {
                    coordinator.request(ShutdownTrigger::SignalFailure).await;
                }
            }
        }
    }
}

async fn cleanup_worker<S>(
    store: S,
    coordinator: ShutdownCoordinator,
    health: CleanupHealth,
    interval: Duration,
) where
    S: ExpiredDeletion + Clone,
{
    loop {
        tokio::select! {
            () = coordinator.wait() => return,
            () = tokio::time::sleep(interval) => {
                if store.delete_expired().await.is_err() {
                    health.mark_unhealthy();
                    coordinator.request(ShutdownTrigger::CleanupFailure).await;
                    return;
                }
            }
        }
    }
}

async fn checkpoint_pool(pool: &SqlitePool) -> bool {
    // SQLite exposes wal_checkpoint output without declared column types.
    // Keep this static pragma checked for syntax while decoding its fixed shape explicitly.
    tokio::time::timeout(
        WAL_CHECKPOINT_TIMEOUT,
        sqlx::query_as_unchecked!(WalCheckpoint, "PRAGMA wal_checkpoint(TRUNCATE)").fetch_one(pool),
    )
    .await
    .is_ok_and(|result| {
        result.is_ok_and(|checkpoint| {
            let _ = (checkpoint.log, checkpoint.checkpointed);
            checkpoint.busy == 0
        })
    })
}

async fn close_pool(pool: &SqlitePool) -> bool {
    tokio::time::timeout(POOL_CLOSE_TIMEOUT, pool.close())
        .await
        .is_ok()
}

async fn run_runtime(
    runtime: BuiltApp,
    listener: tokio::net::TcpListener,
    signals: SignalReceivers,
) -> Result<()> {
    let coordinator = ShutdownCoordinator::default();
    let mut cleanup_handle: JoinHandle<()> = tokio::spawn(cleanup_worker(
        runtime.session_store.clone(),
        coordinator.clone(),
        runtime.cleanup_health.clone(),
        CLEANUP_INTERVAL,
    ));
    let mut signal_handle: JoinHandle<()> =
        tokio::spawn(signal_worker(signals, coordinator.clone()));
    let server_shutdown = coordinator.clone();
    let mut server = Box::pin(
        axum::serve(
            listener,
            runtime
                .app
                .into_make_service_with_connect_info::<SocketAddr>(),
        )
        .with_graceful_shutdown(async move { server_shutdown.wait().await })
        .into_future(),
    );

    let server_finished = tokio::select! {
        result = &mut server => {
            if result.is_err() {
                coordinator.request(ShutdownTrigger::HttpFailure).await;
            }
            true
        }
        () = coordinator.wait() => false,
    };

    if !server_finished {
        if let Some(result) = drain_result(&mut server, HTTP_DRAIN_TIMEOUT).await {
            if result.is_err() {
                coordinator.request(ShutdownTrigger::HttpFailure).await;
            }
        }
    }
    drop(server);

    match tokio::time::timeout(CLEANUP_STOP_TIMEOUT, &mut cleanup_handle).await {
        Ok(Ok(())) => {}
        Ok(Err(_)) => coordinator.request(ShutdownTrigger::CleanupFailure).await,
        Err(_) => {
            cleanup_handle.abort();
            let _ = cleanup_handle.await;
            coordinator.request(ShutdownTrigger::CleanupFailure).await;
        }
    }
    match tokio::time::timeout(CLEANUP_STOP_TIMEOUT, &mut signal_handle).await {
        Ok(Ok(())) => {}
        Ok(Err(_)) => coordinator.request(ShutdownTrigger::SignalFailure).await,
        Err(_) => {
            signal_handle.abort();
            let _ = signal_handle.await;
            coordinator.request(ShutdownTrigger::SignalFailure).await;
        }
    }

    if !checkpoint_pool(&runtime.pool).await {
        coordinator
            .request(ShutdownTrigger::CheckpointFailure)
            .await;
    }
    if !close_pool(&runtime.pool).await {
        coordinator.request(ShutdownTrigger::PoolCloseFailure).await;
    }

    let outcome = coordinator.outcome().await;
    if outcome.first.is_none() || !outcome.fatal_triggers.is_empty() {
        return Err(anyhow!("runtime shutdown failed"));
    }
    Ok(())
}

async fn drain_result<F>(future: F, timeout: Duration) -> Option<F::Output>
where
    F: Future,
{
    tokio::time::timeout(timeout, future).await.ok()
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod composition_tests {
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicUsize, Ordering};

    use async_trait::async_trait;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use debtor_application::{
        ApplicationError, DatabaseReadiness, ReadinessService, ReadinessUseCases,
        SupervisorReadiness, UnavailableReason,
    };
    use tower::ServiceExt;
    use tower_sessions::{
        ExpiredDeletion, SessionStore,
        session::{Id, Record},
        session_store,
    };

    use super::config::Config;
    use super::*;

    static DATABASE_COUNTER: AtomicUsize = AtomicUsize::new(0);

    const VALID_HASH: &str = "$argon2id$v=19$m=19456,t=2,p=1$AAAAAAAAAAAAAAAAAAAAAA$AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";

    fn database_path() -> PathBuf {
        let id = DATABASE_COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("debtor-slice13-{}-{id}.db", std::process::id()))
    }

    fn config(path: &Path, password_hash: &str) -> Config {
        Config {
            database_url: format!("sqlite://{}?mode=rwc", path.display()),
            bind: "127.0.0.1:0".parse().unwrap(),
            password_hash: password_hash.to_owned(),
            cookie_secure: false,
            session_cookie_name: "debtor_session".to_owned(),
            exchange_base_url: "http://127.0.0.1:1".to_owned(),
            trusted_proxy_cidrs: String::new(),
            trusted_proxy_header: String::new(),
        }
    }

    fn remove_database(path: &Path) {
        let _ = std::fs::remove_file(path);
        let _ = std::fs::remove_file(format!("{}-wal", path.display()));
        let _ = std::fs::remove_file(format!("{}-shm", path.display()));
    }

    #[tokio::test]
    async fn invalid_password_hash_has_no_database_side_effect() {
        let path = database_path();
        let result = build_app(config(&path, "not-a-password-hash")).await;
        assert!(result.is_err());
        assert!(!path.exists());
        remove_database(&path);
    }

    #[tokio::test]
    async fn invalid_proxy_configuration_has_no_database_side_effect() {
        let path = database_path();
        let mut invalid = config(&path, VALID_HASH);
        invalid.trusted_proxy_cidrs = "not-a-cidr".to_owned();
        invalid.trusted_proxy_header = "x-forwarded-for".to_owned();
        let result = build_app(invalid).await;
        assert!(result.is_err());
        assert!(!path.exists());
        remove_database(&path);
    }

    #[tokio::test]
    async fn build_app_migrates_and_constructs_router_without_binding() {
        let path = database_path();
        let runtime = build_app(config(&path, VALID_HASH))
            .await
            .expect("build application");
        let response = runtime
            .app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/healthz")
                    .body(Body::empty())
                    .expect("health request"),
            )
            .await
            .expect("health response");
        assert_eq!(response.status(), StatusCode::OK);
        let readiness = runtime
            .app
            .oneshot(
                Request::builder()
                    .uri("/readyz")
                    .body(Body::empty())
                    .expect("readiness request"),
            )
            .await
            .expect("readiness response");
        assert_eq!(readiness.status(), StatusCode::OK);
        assert!(path.exists());
        remove_database(&path);
    }

    #[derive(Clone, Debug)]
    struct FailingCleanupStore;

    #[async_trait]
    impl SessionStore for FailingCleanupStore {
        async fn create(&self, _: &mut Record) -> session_store::Result<()> {
            Ok(())
        }

        async fn save(&self, _: &Record) -> session_store::Result<()> {
            Ok(())
        }

        async fn load(&self, _: &Id) -> session_store::Result<Option<Record>> {
            Ok(None)
        }

        async fn delete(&self, _: &Id) -> session_store::Result<()> {
            Ok(())
        }
    }

    #[async_trait]
    impl ExpiredDeletion for FailingCleanupStore {
        async fn delete_expired(&self) -> session_store::Result<()> {
            Err(session_store::Error::Backend("cleanup failed".to_owned()))
        }
    }

    struct HealthyDatabase;

    #[async_trait]
    impl DatabaseReadiness for HealthyDatabase {
        async fn check(&self) -> Result<(), ApplicationError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn cleanup_failure_marks_readiness_unhealthy_before_shutdown() {
        let coordinator = ShutdownCoordinator::default();
        let health = CleanupHealth::new();
        cleanup_worker(
            FailingCleanupStore,
            coordinator.clone(),
            health.clone(),
            Duration::from_millis(1),
        )
        .await;

        assert!(!health.is_healthy());
        let readiness =
            ReadinessService::with_supervisor(Arc::new(HealthyDatabase), Arc::new(health));
        assert!(matches!(
            readiness.check().await,
            Err(ApplicationError::Unavailable(
                UnavailableReason::RuntimeSupervisor
            ))
        ));
        let outcome = coordinator.outcome().await;
        assert_eq!(outcome.first, Some(ShutdownTrigger::CleanupFailure));
        assert_eq!(
            outcome.fatal_triggers,
            vec![ShutdownTrigger::CleanupFailure]
        );
    }

    #[tokio::test]
    async fn coordinator_preserves_first_trigger_and_collects_later_failures() {
        let coordinator = ShutdownCoordinator::default();
        coordinator.request(ShutdownTrigger::Signal).await;
        coordinator.request(ShutdownTrigger::CleanupFailure).await;
        coordinator.request(ShutdownTrigger::PoolCloseFailure).await;

        let outcome = coordinator.outcome().await;
        assert_eq!(outcome.first, Some(ShutdownTrigger::Signal));
        assert_eq!(
            outcome.fatal_triggers,
            vec![
                ShutdownTrigger::CleanupFailure,
                ShutdownTrigger::PoolCloseFailure
            ]
        );
    }

    #[tokio::test]
    async fn drain_result_bounds_a_stuck_server_future() {
        assert_eq!(
            drain_result(async { 7_u8 }, Duration::from_millis(10)).await,
            Some(7)
        );
        assert!(
            drain_result(std::future::pending::<()>(), Duration::from_millis(1))
                .await
                .is_none()
        );
    }

    #[tokio::test]
    async fn checkpoint_and_pool_close_complete_for_a_temporary_sqlite_pool() {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("temporary pool");
        assert!(checkpoint_pool(&pool).await);
        assert!(close_pool(&pool).await);
    }
}
