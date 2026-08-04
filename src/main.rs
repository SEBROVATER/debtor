//! debtor composition root and local server entry point.

use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::{Context, Result};
use debtor_application::{
    AuthenticationService, AuthenticationUseCases, Clock, DebtService, DebtUseCases, GroupReader,
    GroupRepository, GroupService, GroupUseCases, LedgerSnapshotReader, ParticipantRepository,
    ParticipantService, ParticipantUseCases, SpendingReader, SpendingRepository, SpendingService,
    SpendingUseCases, UtcClock,
};
use debtor_infra::auth::{ArgonPasswordGate, MemoryLoginAttemptLimiter};
use debtor_infra::db::repos::SqliteLedgerStore;
use debtor_infra::exchange_rates::FrankfurterClient;
use debtor_web::state::{AppState, TrustedProxyConfig};
use tower_http::services::ServeDir;
use tower_sessions::{Expiry, MemoryStore, SessionManagerLayer};

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
    let proxy =
        TrustedProxyConfig::parse(&config.trusted_proxy_cidrs, &config.trusted_proxy_header)
            .map_err(anyhow::Error::msg)?;
    let pool = debtor_infra::db::connect(&config.database_url)
        .await
        .context("unable to connect SQLite")?;
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .context("unable to apply SQLite migrations")?;

    let store = Arc::new(SqliteLedgerStore::new(pool));
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
    let password = Arc::new(ArgonPasswordGate::new(config.password_hash)?);
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
        proxy,
    };
    let sessions = SessionManagerLayer::new(MemoryStore::default())
        .with_name(config.session_cookie_name)
        .with_secure(config.cookie_secure)
        .with_http_only(true)
        .with_same_site(tower_sessions::cookie::SameSite::Strict)
        .with_path("/")
        .with_always_save(true)
        .with_expiry(Expiry::OnInactivity(time::Duration::days(30)));
    let app = debtor_web::router::router(state)
        .nest_service("/static", ServeDir::new("static"))
        .layer(sessions);
    let listener = tokio::net::TcpListener::bind(config.bind)
        .await
        .context("unable to bind APP_BIND")?;
    tracing::info!(url = %format!("http://{}", config.bind), "debtor listening");
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    .context("HTTP server failed")
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
