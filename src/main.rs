//! debtor composition root and local server entry point.

use anyhow::{Context, Result};

mod composition;
mod config;
mod runtime;

use composition::build_app;
use config::Config;
use runtime::{SignalReceivers, run_runtime};

#[cfg(test)]
use runtime::{
    CleanupHealth, ShutdownCoordinator, ShutdownTrigger, checkpoint_pool, cleanup_worker,
    close_pool, drain_result, run_runtime_with_options,
};

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
    tracing::info!(
        target: "debtor.startup",
        event = "startup_stage",
        stage = "environment_loaded",
    );
    let signals = SignalReceivers::install()?;
    tracing::info!(
        target: "debtor.startup",
        event = "startup_stage",
        stage = "signals_registered",
    );
    let bind = config.bind;
    let runtime = build_app(config).await?;
    let listener = tokio::net::TcpListener::bind(bind)
        .await
        .context("unable to bind APP_BIND")?;
    tracing::info!(
        target: "debtor.startup",
        event = "listening",
        url = %format!("http://{}", bind),
    );
    run_runtime(runtime, listener, signals).await
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod composition_tests {
    use std::path::{Path, PathBuf};
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    use std::time::Duration;

    use argon2::{Argon2, PasswordHasher, password_hash::SaltString};
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

    fn cookie_pair(response: &reqwest::Response) -> String {
        response
            .headers()
            .get("set-cookie")
            .expect("session cookie")
            .to_str()
            .expect("cookie header")
            .split(';')
            .next()
            .expect("cookie pair")
            .to_owned()
    }

    fn csrf_token(body: &str) -> String {
        let marker = "name=\"csrf\" value=\"";
        let start = body.find(marker).expect("CSRF field") + marker.len();
        body[start..]
            .split('"')
            .next()
            .expect("CSRF value")
            .to_owned()
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

    #[tokio::test]
    async fn real_socket_smoke_covers_login_authenticated_read_and_bounded_shutdown() {
        let salt = SaltString::encode_b64(b"slice18-real-socket").expect("test salt");
        let password_hash = Argon2::default()
            .hash_password(b"correct horse battery staple", &salt)
            .expect("test hash")
            .to_string();
        let path = database_path();
        let runtime = build_app(config(&path, &password_hash))
            .await
            .expect("build application");
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("test listener");
        let address = listener.local_addr().expect("listener address");
        let signals = SignalReceivers::install().expect("signal handlers");
        let coordinator = ShutdownCoordinator::default();
        let shutdown = coordinator.clone();
        let server = tokio::spawn(run_runtime_with_options(
            runtime,
            listener,
            signals,
            shutdown,
            Duration::from_millis(5),
        ));
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .expect("HTTP client");
        let base_url = format!("http://{address}");

        let response = client
            .get(format!("{base_url}/login"))
            .send()
            .await
            .expect("login form request");
        assert_eq!(response.status(), reqwest::StatusCode::OK);
        let login_cookie = cookie_pair(&response);
        let login_body = response.text().await.expect("login form body");
        let token = csrf_token(&login_body);

        let response = client
            .post(format!("{base_url}/login"))
            .header("cookie", &login_cookie)
            .form(&[
                ("csrf", token.as_str()),
                ("password", "correct horse battery staple"),
            ])
            .send()
            .await
            .expect("login request");
        assert_eq!(response.status(), reqwest::StatusCode::SEE_OTHER);
        let authenticated_cookie = cookie_pair(&response);

        let response = client
            .get(format!("{base_url}/groups"))
            .header("cookie", authenticated_cookie)
            .send()
            .await
            .expect("authenticated request");
        assert_eq!(response.status(), reqwest::StatusCode::OK);

        tokio::time::sleep(Duration::from_millis(20)).await;
        coordinator.request(ShutdownTrigger::Signal).await;
        let result = tokio::time::timeout(Duration::from_secs(15), server)
            .await
            .expect("bounded shutdown")
            .expect("server task");
        assert!(result.is_ok(), "runtime shutdown result: {result:?}");
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
