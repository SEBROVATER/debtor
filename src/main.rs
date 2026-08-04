//! debtor composition root and local server entry point.

use std::net::SocketAddr;
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
use debtor_infra::db::repos::SqliteLedgerStore;
use debtor_infra::exchange_rates::FrankfurterClient;
use debtor_web::session;
use debtor_web::session_store::ReapingMemoryStore;
use debtor_web::state::{AppState, TrustedProxyConfig};
use tokio::sync::Semaphore;
use tower::limit::concurrency::GlobalConcurrencyLimitLayer;
use tower::{BoxError, ServiceBuilder};
use tower_http::services::ServeDir;
use tower_http::timeout::TimeoutLayer;
use tower_sessions::SessionManagerLayer;

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
    let bind = config.bind;
    let app = build_app(config).await?;
    let listener = tokio::net::TcpListener::bind(bind)
        .await
        .context("unable to bind APP_BIND")?;
    tracing::info!(url = %format!("http://{}", bind), "debtor listening");
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    .context("HTTP server failed")
}

async fn build_app(config: Config) -> Result<axum::Router> {
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

    let store = Arc::new(SqliteLedgerStore::new(pool));
    let readiness: Arc<dyn ReadinessUseCases> = Arc::new(ReadinessService::new(store.clone()));
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
    let sessions = SessionManagerLayer::new(ReapingMemoryStore::default())
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
    Ok(
        debtor_web::router::router_with_sessions(state, sessions, user_limit)
            .nest_service("/static", static_service),
    )
}

async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };
    #[cfg(unix)]
    let terminate = async {
        if let Ok(mut signal) =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        {
            signal.recv().await;
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! { () = ctrl_c => {}, () = terminate => {} }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod composition_tests {
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicUsize, Ordering};

    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    use super::build_app;
    use super::config::Config;

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
        let app = build_app(config(&path, VALID_HASH))
            .await
            .expect("build application");
        let response = app
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
        let readiness = app
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
}
