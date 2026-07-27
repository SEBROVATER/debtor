#![warn(missing_docs)]

//! debtor composition root and local server entry point.

use std::env;
use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::{Context, Result};
use debtor_application::{
    Clock, DebtService, DebtUseCases, GroupService, GroupUseCases, LedgerStore, ParticipantService,
    ParticipantUseCases, PasswordVerifier, SpendingService, SpendingUseCases, UtcClock,
};
use debtor_infra::auth::ArgonPasswordGate;
use debtor_infra::db::repos::SqliteLedgerStore;
use debtor_infra::exchange_rates::FrankfurterClient;
use debtor_web::state::AppState;
use tower_http::services::ServeDir;
use tower_sessions::{Expiry, MemoryStore, SessionManagerLayer};

/// Validated process configuration.
struct Config {
    database_url: String,
    bind: SocketAddr,
    password_hash: String,
    cookie_secure: bool,
    session_cookie_name: String,
    exchange_base_url: String,
}

impl Config {
    fn from_env() -> Result<Self> {
        let password_hash =
            env::var("APP_ADMIN_PASSWORD_HASH").context("APP_ADMIN_PASSWORD_HASH is required")?;
        if password_hash.trim().is_empty() {
            anyhow::bail!("APP_ADMIN_PASSWORD_HASH must not be empty");
        }
        Ok(Self {
            database_url: env::var("APP_DATABASE_URL")
                .unwrap_or_else(|_| "sqlite://debtor.db?mode=rwc".to_owned()),
            bind: env::var("APP_BIND")
                .unwrap_or_else(|_| "127.0.0.1:3000".to_owned())
                .parse()
                .context("APP_BIND must be a socket address")?,
            password_hash,
            cookie_secure: env::var("APP_SESSION_COOKIE_SECURE")
                .unwrap_or_else(|_| "false".to_owned())
                .parse()
                .context("APP_SESSION_COOKIE_SECURE must be true or false")?,
            session_cookie_name: env::var("APP_SESSION_COOKIE_NAME")
                .unwrap_or_else(|_| "debtor_session".to_owned()),
            exchange_base_url: env::var("APP_EXCHANGE_BASE_URL")
                .unwrap_or_else(|_| "https://api.frankfurter.dev/v2".to_owned()),
        })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "debtor=info,tower_http=info".into()),
        )
        .init();
    let config = Config::from_env()?;
    let pool = debtor_infra::db::connect(&config.database_url)
        .await
        .context("unable to connect SQLite")?;
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .context("unable to apply SQLite migrations")?;

    let store: Arc<dyn LedgerStore> = Arc::new(SqliteLedgerStore::new(pool));
    let groups: Arc<dyn GroupUseCases> = Arc::new(GroupService::new(store.clone()));
    let participants: Arc<dyn ParticipantUseCases> =
        Arc::new(ParticipantService::new(store.clone()));
    let spendings: Arc<dyn SpendingUseCases> = Arc::new(SpendingService::new(store.clone()));
    let rates = Arc::new(FrankfurterClient::with_base_url(&config.exchange_base_url));
    let clock: Arc<dyn Clock> = Arc::new(UtcClock);
    let debts: Arc<dyn DebtUseCases> = Arc::new(DebtService::new(store, rates, clock));
    let password: Arc<dyn PasswordVerifier> =
        Arc::new(ArgonPasswordGate::new(config.password_hash)?);
    let state = AppState {
        groups,
        participants,
        spendings,
        debts,
        password,
    };
    let sessions = SessionManagerLayer::new(MemoryStore::default())
        .with_name(config.session_cookie_name)
        .with_secure(config.cookie_secure)
        .with_expiry(Expiry::OnInactivity(time::Duration::days(30)));
    let app = debtor_web::router::router(state)
        .nest_service("/static", ServeDir::new("static"))
        .layer(sessions);
    let listener = tokio::net::TcpListener::bind(config.bind)
        .await
        .context("unable to bind APP_BIND")?;
    tracing::info!(address = %config.bind, "debtor listening");
    axum::serve(listener, app)
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
