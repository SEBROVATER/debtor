//! debtor composition root and local server entry point.

use anyhow::{Context, Result};

mod composition;
mod config;
mod runtime;
mod startup_error;

#[cfg(test)]
use composition::build_app;
use composition::build_app_with_control;
use config::Config;
use runtime::{
    ShutdownCoordinator, ShutdownTrigger, SignalReceivers, run_runtime_with_coordinator,
};
use startup_error::StartupError;

fn listening_url(address: std::net::SocketAddr) -> String {
    format!("http://{address}")
}

#[cfg(test)]
use runtime::{
    CleanupHealth, await_server_or_shutdown, checkpoint_pool, checkpoint_pool_with_timeout,
    close_pool, close_pool_with_timeout, drain_result, run_runtime_with_options,
    run_runtime_with_timeouts, run_session_cleanup_iteration,
    run_submission_token_cleanup_iteration,
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
    let config = Config::from_lookup(|name| std::env::var(name).ok(), cfg!(debug_assertions))
        .map_err(|_| StartupError::Configuration)?;
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
    let coordinator = ShutdownCoordinator::default();
    let shutdown = coordinator.clone();
    let runtime_control = debtor_web::state::RuntimeControl::with_shutdown_request(move || {
        let shutdown = shutdown.clone();
        let _task = tokio::spawn(async move {
            shutdown
                .request_if_unrequested(ShutdownTrigger::ReadinessFailure)
                .await;
        });
    });
    let runtime = build_app_with_control(config, runtime_control).await?;
    let listener = tokio::net::TcpListener::bind(bind)
        .await
        .context("unable to bind APP_BIND")?;
    let address = listener
        .local_addr()
        .context("unable to determine bound listener address")?;
    tracing::info!(
        target: "debtor.startup",
        event = "listening",
        url = %listening_url(address),
    );
    run_runtime_with_coordinator(runtime, listener, signals, coordinator).await
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
        Router,
        body::Body,
        http::{Request, StatusCode},
        routing::get,
    };
    use debtor_application::{
        ApplicationError, DatabaseReadiness, ReadinessService, ReadinessUseCases,
        SupervisorReadiness, UnavailableReason,
    };
    use tokio::sync::Notify;
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
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "debtor-slice13-{}-{timestamp}-{id}.db",
            std::process::id()
        ))
    }

    fn database_url(path: &Path) -> String {
        format!("sqlite://{}?mode=rwc", path.display())
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

    #[test]
    fn listening_url_uses_the_assigned_listener_address() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("test listener");
        let address = listener.local_addr().expect("listener address");
        assert_ne!(address.port(), 0);
        assert_eq!(listening_url(address), format!("http://{address}"));
    }

    fn runtime_with(app: Router, pool: sqlx::SqlitePool) -> crate::composition::BuiltApp {
        crate::composition::BuiltApp {
            app,
            pool,
            session_store: debtor_web::session_store::ReapingMemoryStore::default(),
            cleanup_health: CleanupHealth::new(),
            submission_token_store: debtor_web::submission_tokens::SubmissionTokenStore::default(),
            runtime: debtor_web::state::RuntimeControl::default(),
        }
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

    fn submission_token(body: &str) -> String {
        let marker = "name=\"submission_token\" value=\"";
        let start = body.find(marker).expect("submission token field") + marker.len();
        body[start..]
            .split('"')
            .next()
            .expect("submission token value")
            .to_owned()
    }

    #[tokio::test]
    async fn invalid_password_hashes_have_no_database_side_effect() {
        for hash in [
            "not-a-password-hash".to_owned(),
            "A".repeat(257),
            "$argon2id$v=19$m=019456,t=2,p=1$AAAAAAAAAAAAAAAAAAAAAA$AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_owned(),
        ] {
            let path = database_path();
            assert!(build_app(config(&path, &hash)).await.is_err());
            assert!(!path.exists());
            remove_database(&path);
        }
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
        let static_response = runtime
            .app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/static/css/app.css")
                    .body(Body::empty())
                    .expect("static request"),
            )
            .await
            .expect("static response");
        assert_eq!(static_response.status(), StatusCode::OK);
        assert!(static_response.headers().get("set-cookie").is_none());
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
    async fn closed_runtime_admission_rejects_static_assets_before_service_work() {
        let path = database_path();
        let control = debtor_web::state::RuntimeControl::default();
        let runtime = build_app_with_control(config(&path, VALID_HASH), control.clone())
            .await
            .expect("build application");
        assert!(control.close_user_admission());

        let response = runtime
            .app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/static/css/app.css")
                    .body(Body::empty())
                    .expect("static request"),
            )
            .await
            .expect("closed static response");
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
        assert!(response.headers().get("set-cookie").is_none());
        assert!(close_pool(&runtime.pool).await);
        remove_database(&path);
    }

    #[tokio::test]
    async fn restarting_composed_application_reuses_migrations_and_database_state() {
        let path = database_path();
        let first = build_app(config(&path, VALID_HASH))
            .await
            .expect("first application");
        sqlx::query("INSERT INTO groups (name, currency) VALUES ('Restarted', 'USD')")
            .execute(&first.pool)
            .await
            .expect("persist restart state");
        let first_listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("first listener");
        let first_address = first_listener.local_addr().expect("first address");
        let first_coordinator = ShutdownCoordinator::default();
        let first_server = tokio::spawn(run_runtime_with_options(
            first,
            first_listener,
            SignalReceivers::install().expect("first signal handlers"),
            first_coordinator.clone(),
            Duration::from_mins(1),
        ));
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .expect("HTTP client");
        let response = client
            .get(format!("http://{first_address}/healthz"))
            .send()
            .await
            .expect("first health request");
        assert_eq!(response.status(), reqwest::StatusCode::OK);
        first_coordinator.request(ShutdownTrigger::Signal).await;
        assert!(first_server.await.expect("first server task").is_ok());

        let verification_pool = debtor_infra::db::connect(&database_url(&path))
            .await
            .expect("verification database");
        let first_migrations: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM _sqlx_migrations")
            .fetch_one(&verification_pool)
            .await
            .expect("first migration count");
        let group_name: String =
            sqlx::query_scalar("SELECT name FROM groups WHERE name = 'Restarted'")
                .fetch_one(&verification_pool)
                .await
                .expect("restarted group");
        assert!(first_migrations > 0);
        assert_eq!(group_name, "Restarted");
        assert!(close_pool(&verification_pool).await);

        let second = build_app(config(&path, VALID_HASH))
            .await
            .expect("second application");
        let second_listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("second listener");
        let second_address = second_listener.local_addr().expect("second address");
        let second_coordinator = ShutdownCoordinator::default();
        let second_server = tokio::spawn(run_runtime_with_options(
            second,
            second_listener,
            SignalReceivers::install().expect("second signal handlers"),
            second_coordinator.clone(),
            Duration::from_mins(1),
        ));
        let response = client
            .get(format!("http://{second_address}/healthz"))
            .send()
            .await
            .expect("second health request");
        assert_eq!(response.status(), reqwest::StatusCode::OK);
        second_coordinator.request(ShutdownTrigger::Signal).await;
        assert!(second_server.await.expect("second server task").is_ok());

        let reopened = debtor_infra::db::connect(&database_url(&path))
            .await
            .expect("reopened database");
        let second_migrations: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM _sqlx_migrations")
            .fetch_one(&reopened)
            .await
            .expect("second migration count");
        let group_name: String =
            sqlx::query_scalar("SELECT name FROM groups WHERE name = 'Restarted'")
                .fetch_one(&reopened)
                .await
                .expect("persisted restarted group");
        assert_eq!(second_migrations, first_migrations);
        assert_eq!(group_name, "Restarted");
        assert!(close_pool(&reopened).await);
        remove_database(&path);
    }

    #[allow(clippy::too_many_lines)]
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
        let submission = submission_token(&login_body);

        let response = client
            .post(format!("{base_url}/login"))
            .header("cookie", &login_cookie)
            .form(&[
                ("csrf", token.as_str()),
                ("submission_token", submission.as_str()),
                ("password", "correct horse battery staple"),
            ])
            .send()
            .await
            .expect("login request");
        assert_eq!(response.status(), reqwest::StatusCode::SEE_OTHER);
        let authenticated_cookie = cookie_pair(&response);

        let response = client
            .get(format!("{base_url}/groups"))
            .header("cookie", &authenticated_cookie)
            .send()
            .await
            .expect("authenticated request");
        assert_eq!(response.status(), reqwest::StatusCode::OK);
        let groups_body = response.text().await.expect("groups body");
        let anonymous_response = client
            .get(format!("{base_url}/login"))
            .send()
            .await
            .expect("anonymous session request");
        assert_eq!(anonymous_response.status(), reqwest::StatusCode::OK);
        let anonymous_cookie = cookie_pair(&anonymous_response);
        let logout_response = client
            .post(format!("{base_url}/logout"))
            .header("cookie", &authenticated_cookie)
            .form(&[
                ("csrf", csrf_token(&groups_body)),
                ("submission_token", submission_token(&groups_body)),
            ])
            .send()
            .await
            .expect("logout request");
        assert_eq!(logout_response.status(), reqwest::StatusCode::SEE_OTHER);
        let post_logout = client
            .get(format!("{base_url}/groups"))
            .header("cookie", &authenticated_cookie)
            .send()
            .await
            .expect("post-logout request");
        assert_eq!(post_logout.status(), reqwest::StatusCode::SEE_OTHER);
        assert_eq!(post_logout.headers()["location"], "/login");

        tokio::time::sleep(Duration::from_millis(20)).await;
        coordinator.request(ShutdownTrigger::Signal).await;
        let result = tokio::time::timeout(Duration::from_secs(15), server)
            .await
            .expect("bounded shutdown")
            .expect("server task");
        assert!(result.is_ok(), "runtime shutdown result: {result:?}");

        let restarted = build_app(config(&path, &password_hash))
            .await
            .expect("restart application");
        let restarted_listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("restart listener");
        let restarted_address = restarted_listener.local_addr().expect("restart address");
        let restarted_coordinator = ShutdownCoordinator::default();
        let restarted_server = tokio::spawn(run_runtime_with_options(
            restarted,
            restarted_listener,
            SignalReceivers::install().expect("restart signal handlers"),
            restarted_coordinator.clone(),
            Duration::from_millis(5),
        ));
        let restarted_base = format!("http://{restarted_address}");
        let stale_authenticated = client
            .get(format!("{restarted_base}/groups"))
            .header("cookie", &authenticated_cookie)
            .send()
            .await
            .expect("stale authenticated cookie request");
        assert_eq!(stale_authenticated.status(), reqwest::StatusCode::SEE_OTHER);
        assert_eq!(stale_authenticated.headers()["location"], "/login");
        let stale_anonymous = client
            .get(format!("{restarted_base}/groups"))
            .header("cookie", &anonymous_cookie)
            .send()
            .await
            .expect("stale anonymous cookie request");
        assert_eq!(stale_anonymous.status(), reqwest::StatusCode::SEE_OTHER);
        assert_eq!(stale_anonymous.headers()["location"], "/login");
        restarted_coordinator.request(ShutdownTrigger::Signal).await;
        assert!(
            tokio::time::timeout(Duration::from_secs(15), restarted_server)
                .await
                .expect("bounded restart shutdown")
                .expect("restart server task")
                .is_ok()
        );
        remove_database(&path);
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
        let runtime = debtor_web::state::RuntimeControl::default();
        run_session_cleanup_iteration(&FailingCleanupStore, &health, &runtime, &coordinator).await;

        assert!(!health.is_healthy());
        assert!(!runtime.user_admission_open());
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
    async fn submission_token_cleanup_failure_uses_the_same_supervisor_boundary() {
        #[derive(Clone)]
        struct FailingTokens;

        #[async_trait]
        impl debtor_web::submission_tokens::SubmissionTokenCleanup for FailingTokens {
            async fn cleanup_expired(&self) -> Result<(), ()> {
                Err(())
            }
        }

        let coordinator = ShutdownCoordinator::default();
        let health = CleanupHealth::new();
        let runtime = debtor_web::state::RuntimeControl::default();
        run_submission_token_cleanup_iteration(&FailingTokens, &health, &runtime, &coordinator)
            .await;
        assert!(!health.is_healthy());
        assert!(!runtime.user_admission_open());
        assert_eq!(
            coordinator.outcome().await.first,
            Some(ShutdownTrigger::CleanupFailure)
        );
    }

    #[tokio::test]
    async fn readiness_failure_callback_requests_coordinated_shutdown_once() {
        let coordinator = ShutdownCoordinator::default();
        let notified = Arc::new(Notify::new());
        let callback_notification = notified.clone();
        let shutdown = coordinator.clone();
        let control = debtor_web::state::RuntimeControl::with_shutdown_request(move || {
            let shutdown = shutdown.clone();
            let notification = callback_notification.clone();
            let _task = tokio::spawn(async move {
                shutdown
                    .request_if_unrequested(ShutdownTrigger::ReadinessFailure)
                    .await;
                notification.notify_one();
            });
        });

        control.fail_readiness();
        control.fail_readiness();
        notified.notified().await;

        assert_eq!(
            coordinator.outcome().await.fatal_triggers,
            vec![ShutdownTrigger::ReadinessFailure]
        );
        assert!(!control.user_admission_open());
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

    #[tokio::test]
    async fn active_request_drains_before_runtime_shutdown() {
        let entered = Arc::new(Notify::new());
        let release = Arc::new(Notify::new());
        let app = Router::new()
            .route("/healthz", get(|| async { "ok" }))
            .route(
                "/hold",
                get({
                    let entered = entered.clone();
                    let release = release.clone();
                    move || {
                        let entered = entered.clone();
                        let release = release.clone();
                        async move {
                            entered.notify_one();
                            release.notified().await;
                            "ok"
                        }
                    }
                }),
            );
        let pool = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("test pool");
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("listener");
        let address = listener.local_addr().expect("listener address");
        let coordinator = ShutdownCoordinator::default();
        let mut server = tokio::spawn(run_runtime_with_timeouts(
            runtime_with(app, pool),
            listener,
            SignalReceivers::install().expect("signal handlers"),
            coordinator.clone(),
            Duration::from_mins(1),
            Duration::from_secs(1),
        ));
        let request = tokio::spawn(async move {
            reqwest::get(format!("http://{address}/hold"))
                .await
                .expect("held response")
                .text()
                .await
                .expect("held body")
        });
        entered.notified().await;
        coordinator.request(ShutdownTrigger::Signal).await;
        assert!(
            tokio::time::timeout(Duration::from_millis(20), &mut server)
                .await
                .is_err()
        );
        let health = reqwest::get(format!("http://{address}/healthz"))
            .await
            .expect("draining health response");
        assert_eq!(health.status(), reqwest::StatusCode::OK);
        release.notify_one();
        assert_eq!(request.await.expect("request task"), "ok");
        assert!(server.await.expect("server task").is_ok());
    }

    #[tokio::test]
    async fn forced_drain_cancels_a_stuck_active_request() {
        let entered = Arc::new(Notify::new());
        let never_release = Arc::new(Notify::new());
        let app = Router::new().route(
            "/hold",
            get({
                let entered = entered.clone();
                let never_release = never_release.clone();
                move || {
                    let entered = entered.clone();
                    let never_release = never_release.clone();
                    async move {
                        entered.notify_one();
                        never_release.notified().await;
                        "unreachable"
                    }
                }
            }),
        );
        let pool = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("test pool");
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("listener");
        let address = listener.local_addr().expect("listener address");
        let coordinator = ShutdownCoordinator::default();
        let server = tokio::spawn(run_runtime_with_timeouts(
            runtime_with(app, pool),
            listener,
            SignalReceivers::install().expect("signal handlers"),
            coordinator.clone(),
            Duration::from_mins(1),
            Duration::from_millis(10),
        ));
        let request =
            tokio::spawn(async move { reqwest::get(format!("http://{address}/hold")).await });
        entered.notified().await;
        coordinator.request(ShutdownTrigger::Signal).await;
        assert!(
            tokio::time::timeout(Duration::from_secs(1), server)
                .await
                .expect("forced drain timeout")
                .expect("server task")
                .is_ok()
        );
        request.abort();
        assert!(request.await.expect_err("cancelled request").is_cancelled());
    }

    #[tokio::test]
    async fn busy_checkpoint_preserves_wal_sidecars() {
        let path = database_path();
        let pool = debtor_infra::db::connect(&format!("sqlite://{}?mode=rwc", path.display()))
            .await
            .expect("WAL pool");
        sqlx::query("CREATE TABLE checkpoint_test (value INTEGER)")
            .execute(&pool)
            .await
            .expect("create table");
        let mut reader = pool.acquire().await.expect("reader connection");
        sqlx::query("BEGIN")
            .execute(&mut *reader)
            .await
            .expect("begin read");
        sqlx::query("SELECT * FROM checkpoint_test")
            .fetch_all(&mut *reader)
            .await
            .expect("read snapshot");
        sqlx::query("INSERT INTO checkpoint_test (value) VALUES (1)")
            .execute(&pool)
            .await
            .expect("write WAL frame");

        assert!(!checkpoint_pool_with_timeout(&pool, Duration::from_millis(50)).await);
        assert!(PathBuf::from(format!("{}-wal", path.display())).exists());
        sqlx::query("ROLLBACK")
            .execute(&mut *reader)
            .await
            .expect("release snapshot");
        drop(reader);
        assert!(close_pool(&pool).await);

        let runtime = build_app(config(&path, VALID_HASH))
            .await
            .expect("rebuild application after recovery");
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("recovery listener");
        let address = listener.local_addr().expect("recovery address");
        let coordinator = ShutdownCoordinator::default();
        let server = tokio::spawn(run_runtime_with_options(
            runtime,
            listener,
            SignalReceivers::install().expect("recovery signal handlers"),
            coordinator.clone(),
            Duration::from_mins(1),
        ));
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .expect("recovery HTTP client");
        let response = client
            .get(format!("http://{address}/healthz"))
            .send()
            .await
            .expect("recovery health request");
        assert_eq!(response.status(), reqwest::StatusCode::OK);
        coordinator.request(ShutdownTrigger::Signal).await;
        assert!(server.await.expect("recovery server task").is_ok());

        let reopened = debtor_infra::db::connect(&database_url(&path))
            .await
            .expect("verify recovered database");
        let value: i64 = sqlx::query_scalar("SELECT value FROM checkpoint_test")
            .fetch_one(&reopened)
            .await
            .expect("recovered WAL value");
        assert_eq!(value, 1);
        assert!(close_pool(&reopened).await);
        remove_database(&path);
    }

    #[tokio::test]
    async fn pool_close_timeout_is_bounded_until_connections_release() {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("test pool");
        let connection = pool.acquire().await.expect("held connection");
        assert!(!close_pool_with_timeout(&pool, Duration::from_millis(10)).await);
        drop(connection);
        assert!(close_pool_with_timeout(&pool, Duration::from_secs(1)).await);
    }

    #[tokio::test]
    async fn unexpected_server_termination_is_a_fatal_shutdown_trigger() {
        let coordinator = ShutdownCoordinator::default();
        let mut server = std::future::ready(Ok(()));

        assert!(await_server_or_shutdown(&mut server, &coordinator).await);
        let outcome = coordinator.outcome().await;
        assert_eq!(outcome.first, Some(ShutdownTrigger::HttpFailure));
        assert_eq!(outcome.fatal_triggers, vec![ShutdownTrigger::HttpFailure]);
    }
}
