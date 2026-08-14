//! Axum route definitions.

use axum::{
    Router,
    error_handling::HandleErrorLayer,
    middleware,
    routing::{get, post},
};
use std::sync::Arc;
use tokio::sync::Semaphore;
use tower::ServiceBuilder;
use tower::limit::concurrency::{ConcurrencyLimitLayer, GlobalConcurrencyLimitLayer};
use tower_http::limit::RequestBodyLimitLayer;
use tower_sessions::{SessionManagerLayer, SessionStore};

use crate::{
    handlers, middleware as app_middleware, session, session_store::ReapingMemoryStore,
    state::AppState,
};

/// Builds the application router from application-facing state.
pub fn router(state: AppState) -> Router {
    let runtime_control = state.runtime.clone();
    user_admission_layer(
        router_with_sessions(
            state,
            SessionManagerLayer::new(ReapingMemoryStore::default())
                .with_secure(false)
                .with_expiry(session::anonymous_expiry())
                .with_always_save(true),
            Arc::new(Semaphore::new(64)),
        ),
        runtime_control,
    )
}

/// Builds the application router with the production-configured session layer.
#[allow(clippy::too_many_lines)]
pub fn router_with_sessions<S: SessionStore + Clone>(
    state: AppState,
    sessions: SessionManagerLayer<S>,
    user_limit: Arc<Semaphore>,
) -> Router {
    let public = Router::new()
        .route("/healthz", get(handlers::health))
        .route("/readyz", get(handlers::readiness))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            app_middleware::probe_timeout,
        ))
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(app_middleware::overload_error))
                .load_shed()
                .layer(ConcurrencyLimitLayer::new(4)),
        );
    let login = Router::new()
        .route("/login", get(handlers::login_form).post(handlers::login))
        .layer(middleware::from_fn(app_middleware::security_headers))
        .layer(sessions.clone())
        .layer(middleware::from_fn(app_middleware::mutation_preflight))
        .layer(middleware::from_fn(app_middleware::login_timeout))
        .layer(RequestBodyLimitLayer::new(8 * 1024))
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(app_middleware::overload_error))
                .load_shed()
                .layer(ConcurrencyLimitLayer::new(4)),
        );
    let protected = Router::new()
        .route("/", get(handlers::root))
        .route("/logout", post(handlers::logout))
        .route(
            "/groups",
            get(handlers::groups).post(handlers::create_group),
        )
        .route("/groups/{id}", get(handlers::group_detail))
        .route(
            "/groups/{id}/edit",
            get(handlers::group_edit_form).post(handlers::update_group),
        )
        .route(
            "/groups/{id}/delete",
            get(handlers::delete_group_form).post(handlers::delete_group),
        )
        .route("/groups/{id}/members", post(handlers::add_member))
        .route(
            "/groups/{group_id}/members/{participant_id}/deactivate",
            post(handlers::deactivate_member),
        )
        .route(
            "/groups/{id}/participants",
            post(handlers::create_group_participant),
        )
        .route("/groups/{id}/spendings", post(handlers::create_spending))
        .route(
            "/groups/{group_id}/spendings/{spending_id}",
            get(handlers::spending_detail).post(handlers::update_spending),
        )
        .route(
            "/groups/{group_id}/spendings/{spending_id}/edit",
            get(handlers::edit_spending_form),
        )
        .route(
            "/groups/{group_id}/spendings/{spending_id}/delete",
            get(handlers::delete_spending_form).post(handlers::delete_spending),
        )
        .route("/groups/{id}/archive", post(handlers::archive_group))
        .route("/groups/{id}/restore", post(handlers::restore_group))
        .route("/groups/{id}/debts", get(handlers::debts))
        .route(
            "/participants",
            get(handlers::participants).post(handlers::create_participant),
        )
        .route(
            "/participants/{id}/archive",
            post(handlers::archive_participant),
        )
        .route(
            "/participants/{id}/restore",
            post(handlers::restore_participant),
        )
        .route(
            "/participants/{id}/edit",
            get(handlers::participant_edit_form),
        )
        .route("/participants/{id}", post(handlers::update_participant))
        .layer(middleware::from_fn(app_middleware::security_headers))
        .layer(middleware::from_fn(app_middleware::require_authenticated))
        .layer(sessions)
        .layer(middleware::from_fn(app_middleware::mutation_preflight))
        .layer(middleware::from_fn(app_middleware::safe_read_timeout))
        .layer(RequestBodyLimitLayer::new(256 * 1024))
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(app_middleware::overload_error))
                .load_shed()
                .layer(GlobalConcurrencyLimitLayer::with_semaphore(user_limit)),
        );
    public
        .merge(login)
        .merge(protected)
        .layer(middleware::from_fn(app_middleware::http_observability))
        .with_state(state)
}

fn user_admission_layer<S>(
    router: Router<S>,
    runtime_control: crate::state::RuntimeControl,
) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    router.layer(middleware::from_fn(move |request, next| {
        app_middleware::user_admission_or_probe(runtime_control.clone(), request, next)
    }))
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use async_trait::async_trait;
    use std::sync::Arc;
    use tokio::sync::{Barrier, Notify, Semaphore};

    use axum::{
        body::{Body, to_bytes},
        extract::connect_info::ConnectInfo,
        http::{
            HeaderValue, Method, Request, StatusCode,
            header::{CONTENT_TYPE, COOKIE, SET_COOKIE},
        },
        response::Response,
    };
    use debtor_application::{ApplicationError, LoginAdmission, ReadinessUseCases};
    use debtor_domain::currency::Currency;
    use tower::ServiceExt;
    use tower_sessions::{
        SessionManagerLayer, SessionStore,
        session::{Id, Record},
    };

    use super::{router, router_with_sessions};
    use crate::handlers::test_support::{
        TestState, state, state_with_errors, state_with_login_admission, state_with_password,
        state_with_readiness_failure,
    };
    use crate::{
        session,
        session_store::{AUTHENTICATED_CAPACITY, ReapingMemoryStore},
    };

    const PEER: std::net::SocketAddr =
        std::net::SocketAddr::new(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST), 4000);

    struct BlockingReadiness {
        entered: Arc<Barrier>,
        release: Arc<Semaphore>,
    }

    #[derive(Clone, Debug)]
    struct BlockingSessionStore {
        inner: ReapingMemoryStore,
        entered: Arc<Barrier>,
        release: Arc<Semaphore>,
    }

    #[async_trait]
    impl SessionStore for BlockingSessionStore {
        async fn create(&self, record: &mut Record) -> tower_sessions::session_store::Result<()> {
            self.entered.wait().await;
            self.release
                .acquire()
                .await
                .map(|_| ())
                .map_err(|_| tower_sessions::session_store::Error::Backend("closed".to_owned()))?;
            self.inner.create(record).await
        }

        async fn save(&self, record: &Record) -> tower_sessions::session_store::Result<()> {
            self.inner.save(record).await
        }

        async fn load(
            &self,
            session_id: &Id,
        ) -> tower_sessions::session_store::Result<Option<Record>> {
            self.inner.load(session_id).await
        }

        async fn delete(&self, session_id: &Id) -> tower_sessions::session_store::Result<()> {
            self.inner.delete(session_id).await
        }
    }

    #[async_trait]
    impl ReadinessUseCases for BlockingReadiness {
        async fn check(&self) -> Result<(), ApplicationError> {
            self.entered.wait().await;
            self.release.acquire().await.map(|_| ()).map_err(|_| {
                ApplicationError::Storage(debtor_application::StorageReason::Unexpected)
            })?;
            Ok(())
        }
    }

    fn app(test_state: &TestState) -> axum::Router {
        router(test_state.app.clone())
    }

    #[tokio::test]
    async fn probes_remain_available_when_all_user_permits_are_held() {
        let test_state = state(false);
        let user_limit = Arc::new(tokio::sync::Semaphore::new(64));
        let permits = user_limit
            .clone()
            .acquire_many_owned(64)
            .await
            .expect("all user permits");
        let app = router_with_sessions(
            test_state.app,
            SessionManagerLayer::new(ReapingMemoryStore::default())
                .with_secure(false)
                .with_expiry(session::anonymous_expiry())
                .with_always_save(true),
            user_limit,
        );

        let response = app
            .oneshot(request(Method::GET, "/healthz", "", None))
            .await
            .expect("probe response");
        assert_eq!(response.status(), StatusCode::OK);
        drop(permits);
    }

    #[tokio::test]
    async fn probe_capacity_is_four_and_does_not_create_sessions() {
        let mut test_state = state(false);
        let entered = Arc::new(Barrier::new(5));
        let release = Arc::new(Semaphore::new(0));
        test_state.app.readiness = Arc::new(BlockingReadiness {
            entered: entered.clone(),
            release: release.clone(),
        });
        let app = app(&test_state);

        let mut requests = Vec::new();
        for _ in 0..4 {
            let app = app.clone();
            requests.push(tokio::spawn(async move {
                app.oneshot(request(Method::GET, "/readyz", "", None))
                    .await
                    .expect("held probe response")
            }));
        }
        entered.wait().await;

        let fifth = app
            .clone()
            .oneshot(request(Method::GET, "/readyz", "", None))
            .await
            .expect("probe overload response");
        assert_eq!(fifth.status(), StatusCode::SERVICE_UNAVAILABLE);
        assert!(fifth.headers().get(SET_COOKIE).is_none());

        release.add_permits(4);
        for request in requests {
            assert_eq!(
                request.await.expect("held probe task").status(),
                StatusCode::OK
            );
        }
    }

    #[tokio::test]
    async fn login_capacity_is_four_and_probe_capacity_is_independent() {
        let test_state = state(false);
        let entered = Arc::new(Barrier::new(5));
        let release = Arc::new(Semaphore::new(0));
        let store = BlockingSessionStore {
            inner: ReapingMemoryStore::default(),
            entered: entered.clone(),
            release: release.clone(),
        };
        let app = router_with_sessions(
            test_state.app,
            SessionManagerLayer::new(store)
                .with_secure(false)
                .with_expiry(session::anonymous_expiry())
                .with_always_save(true),
            Arc::new(Semaphore::new(64)),
        );

        let mut logins = Vec::new();
        for _ in 0..4 {
            let app = app.clone();
            logins.push(tokio::spawn(async move {
                app.oneshot(request(Method::GET, "/login", "", None))
                    .await
                    .expect("held login response")
            }));
        }
        entered.wait().await;

        let fifth_login = app
            .clone()
            .oneshot(request(Method::GET, "/login", "", None))
            .await
            .expect("login overload response");
        assert_eq!(fifth_login.status(), StatusCode::SERVICE_UNAVAILABLE);
        assert!(fifth_login.headers().get(SET_COOKIE).is_none());

        let probe = app
            .oneshot(request(Method::GET, "/healthz", "", None))
            .await
            .expect("independent probe response");
        assert_eq!(probe.status(), StatusCode::OK);
        assert!(probe.headers().get(SET_COOKIE).is_none());

        release.add_permits(4);
        for login in logins {
            assert_eq!(
                login.await.expect("held login task").status(),
                StatusCode::OK
            );
        }
    }

    #[tokio::test]
    async fn closed_user_admission_rejects_before_session_loading_but_keeps_probes_live() {
        let test_state = state(false);
        test_state.app.runtime.close_user_admission();
        let app = app(&test_state);

        let login = app
            .clone()
            .oneshot(request(Method::GET, "/login", "", None))
            .await
            .expect("closed login response");
        assert_eq!(login.status(), StatusCode::SERVICE_UNAVAILABLE);
        assert!(login.headers().get(SET_COOKIE).is_none());

        let health = app
            .oneshot(request(Method::GET, "/healthz", "", None))
            .await
            .expect("probe response");
        assert_eq!(health.status(), StatusCode::OK);
        assert!(health.headers().get(SET_COOKIE).is_none());
    }

    fn request(method: Method, uri: &str, body: &str, cookie: Option<&str>) -> Request<Body> {
        let mut request = Request::builder()
            .method(method)
            .uri(uri)
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(Body::from(body.to_owned()))
            .expect("valid test request");
        request.extensions_mut().insert(ConnectInfo(PEER));
        if let Some(cookie) = cookie {
            request.headers_mut().insert(
                COOKIE,
                HeaderValue::from_str(cookie).expect("valid test cookie"),
            );
        }
        request
    }

    async fn response_body(response: Response) -> String {
        String::from_utf8(
            to_bytes(response.into_body(), 128 * 1024)
                .await
                .expect("test response body")
                .to_vec(),
        )
        .expect("UTF-8 test response")
    }

    fn session_cookie(response: &Response) -> String {
        response
            .headers()
            .get(SET_COOKIE)
            .expect("session cookie")
            .to_str()
            .expect("UTF-8 session cookie")
            .split(';')
            .next()
            .expect("cookie pair")
            .to_owned()
    }

    fn csrf(body: &str) -> String {
        let marker = "name=\"csrf\" value=\"";
        let start = body.find(marker).expect("csrf field") + marker.len();
        body[start..]
            .split('"')
            .next()
            .expect("csrf value")
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

    async fn login(app: &axum::Router) -> String {
        let response = app
            .clone()
            .oneshot(request(Method::GET, "/login", "", None))
            .await
            .expect("login form response");
        let cookie = session_cookie(&response);
        let body = response_body(response).await;
        let token = csrf(&body);
        let submission = submission_token(&body);
        let response = app
            .clone()
            .oneshot(request(
                Method::POST,
                "/login",
                &format!("csrf={token}&submission_token={submission}&password=correct"),
                Some(&cookie),
            ))
            .await
            .expect("login response");
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        session_cookie(&response)
    }

    #[tokio::test]
    async fn empty_history_cursor_links_to_newest_page() {
        let test_state = state(false);
        let app = app(&test_state);
        let session_cookie = login(&app).await;

        let response = app
            .clone()
            .oneshot(request(
                Method::GET,
                "/groups/1?cursor=older:2026-01-01:1",
                "",
                Some(&session_cookie),
            ))
            .await
            .expect("history response");
        assert_eq!(response.status(), StatusCode::OK);
        assert!(
            response_body(response)
                .await
                .contains("href=\"/groups/1\">Newest</a>")
        );
    }

    #[tokio::test]
    async fn sign_out_flushes_the_session_and_expires_the_cookie() {
        let test_state = state(false);
        let app = app(&test_state);
        let authenticated_cookie = login(&app).await;
        let page = app
            .clone()
            .oneshot(request(
                Method::GET,
                "/groups",
                "",
                Some(&authenticated_cookie),
            ))
            .await
            .expect("groups page");
        assert_eq!(page.status(), StatusCode::OK);
        let body = response_body(page).await;
        assert_eq!(body.matches("name=\"submission_token\"").count(), 3);
        let first_token = submission_token(&body);
        assert_eq!(
            body.matches(&format!(
                "name=\"submission_token\" value=\"{first_token}\""
            ))
            .count(),
            3
        );
        assert_eq!(body.matches("action=\"/logout\"").count(), 1);
        let csrf_token = csrf(&body);
        let sign_out_token = submission_token(&body);

        let response = app
            .clone()
            .oneshot(request(
                Method::POST,
                "/logout",
                &format!("csrf={csrf_token}&submission_token={sign_out_token}"),
                Some(&authenticated_cookie),
            ))
            .await
            .expect("logout response");
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(response.headers()["location"], "/login");
        assert!(
            response
                .headers()
                .get(SET_COOKIE)
                .and_then(|value| value.to_str().ok())
                .is_some_and(|value| value.contains("Max-Age=0"))
        );

        let response = app
            .oneshot(request(
                Method::GET,
                "/groups",
                "",
                Some(&authenticated_cookie),
            ))
            .await
            .expect("post-logout protected response");
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(response.headers()["location"], "/login");
    }

    #[tokio::test]
    async fn invalid_sign_out_preserves_session_and_token_for_retry() {
        let test_state = state(false);
        let app = app(&test_state);
        let authenticated_cookie = login(&app).await;
        let page = app
            .clone()
            .oneshot(request(
                Method::GET,
                "/groups",
                "",
                Some(&authenticated_cookie),
            ))
            .await
            .expect("groups page");
        let body = response_body(page).await;
        let csrf_token = csrf(&body);
        let sign_out_token = submission_token(&body);

        let response = app
            .clone()
            .oneshot(request(
                Method::POST,
                "/logout",
                &format!("csrf=wrong&submission_token={sign_out_token}"),
                Some(&authenticated_cookie),
            ))
            .await
            .expect("invalid logout response");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);

        let response = app
            .clone()
            .oneshot(request(
                Method::GET,
                "/groups",
                "",
                Some(&authenticated_cookie),
            ))
            .await
            .expect("authenticated retry");
        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body(response).await;
        let response = app
            .oneshot(request(
                Method::POST,
                "/logout",
                &format!("csrf={csrf_token}&submission_token={sign_out_token}"),
                Some(&authenticated_cookie),
            ))
            .await
            .expect("retry logout response");
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert!(body.contains("Sign out"));
    }

    #[tokio::test]
    async fn sign_out_replay_returns_conflict_without_second_flush() {
        let test_state = state(false);
        let app = app(&test_state);
        let authenticated_cookie = login(&app).await;
        let page = app
            .clone()
            .oneshot(request(
                Method::GET,
                "/groups",
                "",
                Some(&authenticated_cookie),
            ))
            .await
            .expect("groups page");
        let body = response_body(page).await;
        let form = format!(
            "csrf={}&submission_token={}",
            csrf(&body),
            submission_token(&body)
        );
        let response = app
            .clone()
            .oneshot(request(
                Method::POST,
                "/logout",
                &form,
                Some(&authenticated_cookie),
            ))
            .await
            .expect("first logout");
        assert_eq!(response.status(), StatusCode::SEE_OTHER);

        let response = app
            .oneshot(request(
                Method::POST,
                "/logout",
                &form,
                Some(&authenticated_cookie),
            ))
            .await
            .expect("replayed logout");
        assert_eq!(response.status(), StatusCode::CONFLICT);
        assert!(response.headers().get("location").is_none());
    }

    #[tokio::test]
    async fn concurrent_sign_out_requests_have_one_winner() {
        let test_state = state(false);
        let app = app(&test_state);
        let cookie = login(&app).await;
        let page = app
            .clone()
            .oneshot(request(Method::GET, "/groups", "", Some(&cookie)))
            .await
            .expect("groups page");
        let body = response_body(page).await;
        let form = format!(
            "csrf={}&submission_token={}",
            csrf(&body),
            submission_token(&body)
        );
        let barrier = Arc::new(Barrier::new(3));
        let first = app.clone();
        let second = app;
        let first_form = form.clone();
        let second_form = form;
        let first_cookie = cookie.clone();
        let second_cookie = cookie;
        let first_barrier = barrier.clone();
        let second_barrier = barrier.clone();
        let first_task = tokio::spawn(async move {
            first_barrier.wait().await;
            first
                .oneshot(request(
                    Method::POST,
                    "/logout",
                    &first_form,
                    Some(&first_cookie),
                ))
                .await
                .expect("first concurrent logout")
                .status()
        });
        let second_task = tokio::spawn(async move {
            second_barrier.wait().await;
            second
                .oneshot(request(
                    Method::POST,
                    "/logout",
                    &second_form,
                    Some(&second_cookie),
                ))
                .await
                .expect("second concurrent logout")
                .status()
        });
        barrier.wait().await;
        let statuses = [
            first_task.await.expect("first task"),
            second_task.await.expect("second task"),
        ];
        assert!(statuses.contains(&StatusCode::SEE_OTHER));
        assert!(statuses.contains(&StatusCode::CONFLICT));
    }

    #[tokio::test]
    async fn authenticated_shell_exposes_pending_and_failure_targets() {
        let test_state = state(false);
        let app = app(&test_state);
        let cookie = login(&app).await;
        let response = app
            .oneshot(request(Method::GET, "/groups", "", Some(&cookie)))
            .await
            .expect("groups page");
        let body = response_body(response).await;
        assert!(body.contains("aria-busy=\"false\""));
        assert!(body.contains("hx-target-4xx=\"#sign-out-status\""));
        assert!(body.contains("hx-target-5xx=\"#sign-out-status\""));
        assert!(body.contains("response-targets.js"));
        assert!(body.contains("integrity=\"sha384-NtTh9TBZ2X/"));
        assert!(body.contains("id=\"sign-out-status\""));
    }

    #[tokio::test]
    async fn authenticated_mutation_forms_expose_pending_ownership() {
        let test_state = state(false);
        let app = app(&test_state);
        let cookie = login(&app).await;
        let response = app
            .oneshot(request(Method::GET, "/groups", "", Some(&cookie)))
            .await
            .expect("groups page");
        let body = response_body(response).await;
        assert!(body.matches("mutation-form").count() >= 2);
        assert!(body.matches("aria-busy=\"false\"").count() >= 3);
        assert!(body.matches("role=\"status\"").count() >= 3);
        assert!(body.contains("hx-disabled-elt=\"button\""));
    }

    #[tokio::test]
    async fn sign_out_rejects_duplicate_fields_before_flush() {
        let test_state = state(false);
        let app = app(&test_state);
        let authenticated_cookie = login(&app).await;
        let page = app
            .clone()
            .oneshot(request(
                Method::GET,
                "/groups",
                "",
                Some(&authenticated_cookie),
            ))
            .await
            .expect("groups page");
        let body = response_body(page).await;
        let token = submission_token(&body);
        let response = app
            .clone()
            .oneshot(request(
                Method::POST,
                "/logout",
                &format!(
                    "csrf={}&csrf=duplicate&submission_token={token}",
                    csrf(&body)
                ),
                Some(&authenticated_cookie),
            ))
            .await
            .expect("duplicate logout response");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);

        let response = app
            .oneshot(request(
                Method::GET,
                "/groups",
                "",
                Some(&authenticated_cookie),
            ))
            .await
            .expect("authenticated session after rejection");
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn authenticated_page_uses_one_shared_token_for_all_forms() {
        let test_state = state(false);
        let app = app(&test_state);
        let cookie = login(&app).await;
        let response = app
            .clone()
            .oneshot(request(Method::GET, "/groups", "", Some(&cookie)))
            .await
            .expect("groups page");
        let body = response_body(response).await;
        let token = submission_token(&body);
        assert!(body.matches("name=\"submission_token\"").count() >= 3);
        assert_eq!(
            body.matches(&format!("name=\"submission_token\" value=\"{token}\""))
                .count(),
            body.matches("name=\"submission_token\"").count()
        );

        let second = app
            .oneshot(request(Method::GET, "/groups", "", Some(&cookie)))
            .await
            .expect("second groups page");
        let second_body = response_body(second).await;
        assert_ne!(token, submission_token(&second_body));
    }

    #[tokio::test]
    async fn invalid_authenticated_token_returns_conflict_without_dispatch() {
        let test_state = state(false);
        let app = app(&test_state);
        let cookie = login(&app).await;
        let page = app
            .clone()
            .oneshot(request(Method::GET, "/groups", "", Some(&cookie)))
            .await
            .expect("groups page");
        let body = response_body(page).await;
        let response = app
            .oneshot(request(
                Method::POST,
                "/groups",
                &format!(
                    "csrf={}&submission_token=unknown&name=New&currency=USD",
                    csrf(&body)
                ),
                Some(&cookie),
            ))
            .await
            .expect("token conflict");
        assert_eq!(response.status(), StatusCode::CONFLICT);
        assert!(response_body(response).await.contains("No change occurred"));
        assert!(
            test_state
                .groups
                .created
                .lock()
                .expect("group calls")
                .is_empty()
        );
    }

    #[tokio::test]
    async fn login_missing_submission_token_keeps_login_recovery() {
        let test_state = state(false);
        let app = app(&test_state);
        let response = app
            .clone()
            .oneshot(request(Method::GET, "/login", "", None))
            .await
            .expect("login form");
        let cookie = session_cookie(&response);
        let body = response_body(response).await;
        let response = app
            .oneshot(request(
                Method::POST,
                "/login",
                &format!("csrf={}&password=wrong", csrf(&body)),
                Some(&cookie),
            ))
            .await
            .expect("login recovery");
        let body = response_body(response).await;
        assert!(body.contains("/login"));
        assert!(!body.contains("Reload the form"));
    }

    #[tokio::test]
    async fn dispatched_application_validation_consumes_authenticated_token() {
        let test_state = state_with_errors(false, true, false, false, false, false);
        let app = app(&test_state);
        let cookie = login(&app).await;
        let page = app
            .clone()
            .oneshot(request(Method::GET, "/groups", "", Some(&cookie)))
            .await
            .expect("groups page");
        let body = response_body(page).await;
        let form = format!(
            "csrf={}&submission_token={}&name=Draft&currency=USD",
            csrf(&body),
            submission_token(&body)
        );
        let response = app
            .clone()
            .oneshot(request(Method::POST, "/groups", &form, Some(&cookie)))
            .await
            .expect("validation response");
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let response = app
            .oneshot(request(Method::POST, "/groups", &form, Some(&cookie)))
            .await
            .expect("second validation response");
        assert_eq!(response.status(), StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn concurrent_promotions_at_authenticated_capacity_leave_no_anonymous_orphan() {
        let test_state = state(false);
        let store = ReapingMemoryStore::default();
        for _ in 0..AUTHENTICATED_CAPACITY - 1 {
            let mut record = Record {
                id: Id::default(),
                data: std::collections::HashMap::from([("authenticated".into(), true.into())]),
                expiry_date: time::OffsetDateTime::now_utc() + time::Duration::hours(1),
            };
            store
                .create(&mut record)
                .await
                .expect("fill authenticated capacity");
        }
        let app = router_with_sessions(
            test_state.app.clone(),
            SessionManagerLayer::new(store.clone())
                .with_secure(false)
                .with_expiry(session::anonymous_expiry())
                .with_always_save(true),
            Arc::new(tokio::sync::Semaphore::new(64)),
        );
        let mut submissions = Vec::new();
        for _ in 0..2 {
            let response = app
                .clone()
                .oneshot(request(Method::GET, "/login", "", None))
                .await
                .expect("login form response");
            let cookie = session_cookie(&response);
            let body = response_body(response).await;
            let token = csrf(&body);
            let submission = submission_token(&body);
            submissions.push((cookie, token, submission));
        }
        let [
            (first_cookie, first_token, first_submission),
            (second_cookie, second_token, second_submission),
        ]: [(String, String, String); 2] = submissions.try_into().expect("two login submissions");
        let first = app.clone().oneshot(request(
            Method::POST,
            "/login",
            &format!("csrf={first_token}&submission_token={first_submission}&password=correct"),
            Some(&first_cookie),
        ));
        let second = app.clone().oneshot(request(
            Method::POST,
            "/login",
            &format!("csrf={second_token}&submission_token={second_submission}&password=correct"),
            Some(&second_cookie),
        ));
        let (first, second) = tokio::join!(first, second);
        let statuses = [
            first.expect("first login response").status(),
            second.expect("second login response").status(),
        ];

        assert_eq!(
            statuses
                .iter()
                .filter(|&&status| status == StatusCode::SEE_OTHER)
                .count(),
            1
        );
        assert_eq!(
            statuses
                .iter()
                .filter(|&&status| status == StatusCode::SERVICE_UNAVAILABLE)
                .count(),
            1
        );
        assert_eq!(
            store.counts().await,
            (AUTHENTICATED_CAPACITY + 1, 1, AUTHENTICATED_CAPACITY)
        );
        assert_eq!(
            test_state
                .auth_resets
                .load(std::sync::atomic::Ordering::SeqCst),
            1
        );
    }

    #[tokio::test]
    #[allow(clippy::too_many_lines)]
    async fn rendered_forms_use_real_actions_and_retain_submitted_values() {
        let test_state = state_with_errors(false, true, true, true, true, true);
        let app = app(&test_state);
        let session_cookie = login(&app).await;
        assert_eq!(
            test_state
                .auth_resets
                .load(std::sync::atomic::Ordering::SeqCst),
            1
        );

        let response = app
            .clone()
            .oneshot(request(
                Method::GET,
                "/groups/1/edit",
                "",
                Some(&session_cookie),
            ))
            .await
            .expect("group edit response");
        let edit_form = response_body(response).await;
        assert!(edit_form.contains("action=\"/groups/1/edit\""));

        let response = app
            .clone()
            .oneshot(request(
                Method::POST,
                "/groups/1/edit",
                &format!(
                    "csrf={}&submission_token={}&name=Renamed&currency=USD",
                    csrf(&edit_form),
                    submission_token(&edit_form)
                ),
                Some(&session_cookie),
            ))
            .await
            .expect("group edit validation response");
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let body = response_body(response).await;
        assert!(body.contains("value=\"Renamed\""));
        assert!(body.contains("action=\"/groups/1/edit\""));

        let response = app
            .clone()
            .oneshot(request(Method::GET, "/groups", "", Some(&session_cookie)))
            .await
            .expect("groups response");
        let groups_page = response_body(response).await;
        let response = app
            .clone()
            .oneshot(request(
                Method::POST,
                "/groups",
                &format!(
                    "csrf={}&submission_token={}&name=New+group&currency=USD",
                    csrf(&groups_page),
                    submission_token(&groups_page)
                ),
                Some(&session_cookie),
            ))
            .await
            .expect("group create validation response");
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let body = response_body(response).await;
        assert!(body.contains("value=\"New group\""));

        let response = app
            .clone()
            .oneshot(request(
                Method::GET,
                "/participants",
                "",
                Some(&session_cookie),
            ))
            .await
            .expect("participants response");
        let participants_page = response_body(response).await;
        let response = app
            .clone()
            .oneshot(request(
                Method::POST,
                "/participants",
                &format!(
                    "csrf={}&submission_token={}&name=New+person&color=%23abcdef",
                    csrf(&participants_page),
                    submission_token(&participants_page)
                ),
                Some(&session_cookie),
            ))
            .await
            .expect("participant create validation response");
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let body = response_body(response).await;
        assert!(body.contains("value=\"New person\""));
        assert!(body.contains("value=\"#abcdef\""));

        let response = app
            .clone()
            .oneshot(request(Method::GET, "/groups/1", "", Some(&session_cookie)))
            .await
            .expect("group response");
        let group_detail_page = response_body(response).await;
        let response = app
            .clone()
            .oneshot(request(
                Method::POST,
                "/groups/1/participants",
                &format!(
                    "csrf={}&submission_token={}&name=Joined+person&color=%23fedcba",
                    csrf(&group_detail_page),
                    submission_token(&group_detail_page)
                ),
                Some(&session_cookie),
            ))
            .await
            .expect("group participant validation response");
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let body = response_body(response).await;
        assert!(body.contains("value=\"Joined person\""));
        assert!(body.contains("value=\"#fedcba\""));

        let response = app
            .clone()
            .oneshot(request(
                Method::GET,
                "/participants/1/edit",
                "",
                Some(&session_cookie),
            ))
            .await
            .expect("participant edit response");
        let participant_edit_page = response_body(response).await;
        let response = app
            .clone()
            .oneshot(request(
                Method::POST,
                "/participants/1",
                &format!(
                    "csrf={}&submission_token={}&name=Edited+person&color=%23aabbcc",
                    csrf(&participant_edit_page),
                    submission_token(&participant_edit_page)
                ),
                Some(&session_cookie),
            ))
            .await
            .expect("participant edit validation response");
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let body = response_body(response).await;
        assert!(body.contains("value=\"Edited person\""));
        assert!(body.contains("value=\"#aabbcc\""));
    }

    #[tokio::test]
    async fn valid_group_edit_uses_registered_route_and_records_values() {
        let test_state = state(false);
        let app = app(&test_state);
        let session_cookie = login(&app).await;
        let response = app
            .clone()
            .oneshot(request(
                Method::GET,
                "/groups/1/edit",
                "",
                Some(&session_cookie),
            ))
            .await
            .expect("group edit response");
        let form = response_body(response).await;
        let response = app
            .oneshot(request(
                Method::POST,
                "/groups/1/edit",
                &format!(
                    "csrf={}&submission_token={}&name=Updated+group&currency=EUR",
                    csrf(&form),
                    submission_token(&form)
                ),
                Some(&session_cookie),
            ))
            .await
            .expect("group update response");

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(
            *test_state.groups.updated.lock().expect("group calls lock"),
            vec![(1, "Updated group".to_owned(), Currency::Eur)]
        );
    }

    #[tokio::test]
    async fn spending_validation_retains_the_submitted_draft() {
        let test_state = state(false);
        let app = app(&test_state);
        let session_cookie = login(&app).await;
        let response = app
            .clone()
            .oneshot(request(Method::GET, "/groups/1", "", Some(&session_cookie)))
            .await
            .expect("group response");
        let group_page = response_body(response).await;
        let response = app
            .oneshot(request(
                Method::POST,
                "/groups/1/spendings",
                &format!(
                    "description=Lunch&total=12.00&currency=USD&spending_type=food&spent_date=2026-08-04&payer_mode=single&single_payer_id=1&split_mode=equal&csrf={}&submission_token={}&share_1=on",
                    csrf(&group_page),
                    submission_token(&group_page)
                ),
                Some(&session_cookie),
            ))
            .await
            .expect("spending validation response");

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let body = response_body(response).await;
        assert!(body.contains("value=\"Lunch\""));
        assert!(body.contains("value=\"12.00\""));
        assert!(body.contains("value=\"2026-08-04\""));
        assert!(body.contains("value=\"1\""));
    }

    #[tokio::test]
    async fn archived_group_is_read_only_and_direct_mutation_routes_conflict() {
        let test_state = state(true);
        let app = app(&test_state);
        let session_cookie = login(&app).await;
        let response = app
            .clone()
            .oneshot(request(
                Method::GET,
                "/groups?archived=true",
                "",
                Some(&session_cookie),
            ))
            .await
            .expect("archived groups response");
        let group_page = response_body(response).await;
        let response = app
            .clone()
            .oneshot(request(Method::GET, "/groups/1", "", Some(&session_cookie)))
            .await
            .expect("archived group response");
        let archived_group_page = response_body(response).await;
        assert!(!archived_group_page.contains("/groups/1/edit"));
        assert!(!archived_group_page.contains("/groups/1/delete"));
        assert!(!archived_group_page.contains("/groups/1/members"));

        for (method, uri, form) in [
            (Method::GET, "/groups/1/edit", ""),
            (Method::POST, "/groups/1/edit", "name=Nope&currency=USD"),
            (Method::GET, "/groups/1/delete", ""),
            (Method::POST, "/groups/1/delete", ""),
            (Method::POST, "/groups/1/members", "participant_id=1"),
            (
                Method::POST,
                "/groups/1/participants",
                "name=New&color=%23abcdef",
            ),
            (Method::POST, "/groups/1/members/1/deactivate", ""),
            (Method::POST, "/groups/1/spendings", ""),
            (Method::GET, "/groups/1/spendings/1/edit", ""),
            (Method::GET, "/groups/1/spendings/1/delete", ""),
            (Method::POST, "/groups/1/spendings/1", ""),
            (Method::POST, "/groups/1/spendings/1/delete", ""),
        ] {
            let response = app
                .clone()
                .oneshot(request(
                    method.clone(),
                    uri,
                    &format!("csrf={}&{form}", csrf(&group_page)),
                    Some(&session_cookie),
                ))
                .await
                .expect("archived mutation response");
            assert_eq!(response.status(), StatusCode::CONFLICT, "{method} {uri}");
        }
    }

    #[tokio::test]
    async fn protected_gets_redirect_and_probes_do_not_create_sessions() {
        let test_state = state(false);
        let app = app(&test_state);
        for uri in [
            "/",
            "/groups",
            "/groups/1",
            "/groups/1/edit",
            "/groups/1/delete",
            "/groups/1/spendings/1",
            "/groups/1/spendings/1/edit",
            "/groups/1/spendings/1/delete",
            "/groups/1/debts",
            "/participants",
            "/participants/1/edit",
        ] {
            let response = app
                .clone()
                .oneshot(request(Method::GET, uri, "", None))
                .await
                .expect("protected route response");
            assert_eq!(response.status(), StatusCode::SEE_OTHER, "GET {uri}");
            assert_eq!(
                response
                    .headers()
                    .get("location")
                    .expect("redirect location"),
                "/login"
            );
        }
        for uri in ["/healthz", "/readyz"] {
            let response = app
                .clone()
                .oneshot(request(Method::GET, uri, "", None))
                .await
                .expect("probe response");
            assert_eq!(response.status(), StatusCode::OK, "GET {uri}");
            assert!(
                response.headers().get(SET_COOKIE).is_none(),
                "probe session {uri}"
            );
        }
    }

    #[tokio::test]
    async fn readiness_failure_is_distinct_from_liveness_and_sanitized() {
        let test_state = state_with_readiness_failure();
        let app = app(&test_state);

        let health = app
            .clone()
            .oneshot(request(Method::GET, "/healthz", "", None))
            .await
            .expect("health response");
        assert_eq!(health.status(), StatusCode::OK);

        let readiness = app
            .clone()
            .oneshot(request(Method::GET, "/readyz", "", None))
            .await
            .expect("readiness response");
        assert_eq!(readiness.status(), StatusCode::SERVICE_UNAVAILABLE);
        let body = response_body(readiness).await;
        assert!(body.contains("Service temporarily unavailable."));
        assert!(!body.contains("unexpected storage failure"));

        let rejected = app
            .oneshot(request(Method::GET, "/login", "", None))
            .await
            .expect("closed login response");
        assert_eq!(rejected.status(), StatusCode::SERVICE_UNAVAILABLE);
        assert!(rejected.headers().get(SET_COOKIE).is_none());
    }

    #[tokio::test]
    async fn readiness_failure_invokes_the_injected_shutdown_callback() {
        let mut test_state = state_with_readiness_failure();
        let notified = Arc::new(Notify::new());
        let callback_notification = notified.clone();
        let control = crate::state::RuntimeControl::with_shutdown_request(move || {
            callback_notification.notify_one();
        });
        test_state.app.runtime = control.clone();
        let app = app(&test_state);

        let response = app
            .oneshot(request(Method::GET, "/readyz", "", None))
            .await
            .expect("readiness response");
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
        notified.notified().await;
        assert!(!control.user_admission_open());
    }

    #[tokio::test]
    #[allow(clippy::too_many_lines)]
    async fn unsafe_routes_reject_missing_wrong_and_duplicate_csrf_before_use_cases() {
        let test_state = state(false);
        let app = app(&test_state);
        let session_cookie = login(&app).await;
        let routes = [
            (Method::POST, "/logout"),
            (Method::POST, "/groups"),
            (Method::POST, "/groups/1/edit"),
            (Method::POST, "/groups/1/delete"),
            (Method::POST, "/groups/1/members"),
            (Method::POST, "/groups/1/members/1/deactivate"),
            (Method::POST, "/groups/1/participants"),
            (Method::POST, "/groups/1/spendings"),
            (Method::POST, "/groups/1/spendings/1"),
            (Method::POST, "/groups/1/spendings/1/delete"),
            (Method::POST, "/groups/1/archive"),
            (Method::POST, "/groups/1/restore"),
            (Method::POST, "/participants"),
            (Method::POST, "/participants/1"),
            (Method::POST, "/participants/1/archive"),
            (Method::POST, "/participants/1/restore"),
        ];
        for (method, uri) in routes {
            for body in ["", "csrf=wrong", "csrf=wrong&csrf=another"] {
                let response = app
                    .clone()
                    .oneshot(request(method.clone(), uri, body, Some(&session_cookie)))
                    .await
                    .expect("CSRF rejection response");
                assert_eq!(
                    response.status(),
                    StatusCode::FORBIDDEN,
                    "{method} {uri} {body}"
                );
            }
        }
        assert!(
            test_state
                .groups
                .created
                .lock()
                .expect("group calls")
                .is_empty()
        );
        assert!(
            test_state
                .groups
                .updated
                .lock()
                .expect("group calls")
                .is_empty()
        );
        assert!(
            test_state
                .participants
                .created
                .lock()
                .expect("participant calls")
                .is_empty()
        );
        assert!(
            test_state
                .participants
                .updated
                .lock()
                .expect("participant calls")
                .is_empty()
        );
        assert!(
            test_state
                .participants
                .group_created
                .lock()
                .expect("participant calls")
                .is_empty()
        );
    }

    #[tokio::test]
    async fn login_and_authenticated_html_include_security_headers() {
        let test_state = state(false);
        let app = app(&test_state);
        let response = app
            .clone()
            .oneshot(request(Method::GET, "/login", "", None))
            .await
            .expect("login response");
        assert_security_headers(&response);
        let login_cookie = session_cookie(&response);
        let set_cookie = response
            .headers()
            .get(SET_COOKIE)
            .expect("set-cookie")
            .to_str()
            .expect("cookie header");
        assert!(set_cookie.contains("HttpOnly"));
        assert!(set_cookie.contains("SameSite=Strict"));
        assert!(!set_cookie.contains("Secure"));
        let login_body = response_body(response).await;
        assert_eq!(login_body.matches("name=\"csrf\"").count(), 1);
        assert_eq!(login_body.matches("name=\"submission_token\"").count(), 1);
        assert!(login_body.contains("id=\"sign-in-heading\""));
        assert!(login_body.contains("role=\"status\""));
        assert!(login_body.contains("aria-live=\"polite\""));
        assert!(login_body.contains("aria-busy=\"false\""));
        assert!(login_body.contains("for=\"password\""));
        assert!(login_body.contains("action=\"/login\""));
        assert!(!login_body.contains("name=\"username\""));
        assert!(login_body.contains("autofocus"));
        assert_ne!(csrf(&login_body), submission_token(&login_body));
        let response = app
            .clone()
            .oneshot(request(
                Method::POST,
                "/login",
                &format!(
                    "csrf={}&submission_token={}&password=correct",
                    csrf(&login_body),
                    submission_token(&login_body)
                ),
                Some(&login_cookie),
            ))
            .await
            .expect("login completion response");
        let authenticated_cookie = session_cookie(&response);
        let response = app
            .oneshot(request(
                Method::GET,
                "/groups",
                "",
                Some(&authenticated_cookie),
            ))
            .await
            .expect("authenticated response");
        assert_security_headers(&response);
    }

    #[tokio::test]
    async fn login_rejects_unknown_submission_without_authentication() {
        let test_state = state(false);
        let app = app(&test_state);
        let response = app
            .clone()
            .oneshot(request(Method::GET, "/login", "", None))
            .await
            .expect("login response");
        let cookie = session_cookie(&response);
        let body = response_body(response).await;
        let response = app
            .oneshot(request(
                Method::POST,
                "/login",
                &format!(
                    "csrf={}&submission_token=unknown&password=correct",
                    csrf(&body)
                ),
                Some(&cookie),
            ))
            .await
            .expect("conflict response");
        assert_eq!(response.status(), StatusCode::CONFLICT);
        assert_eq!(
            test_state
                .auth_attempts
                .load(std::sync::atomic::Ordering::SeqCst),
            0
        );
        assert_eq!(
            test_state
                .password_verifications
                .load(std::sync::atomic::Ordering::SeqCst),
            0
        );
    }

    #[tokio::test]
    async fn invalid_password_consumes_token_and_renders_fresh_recovery() {
        let test_state = state_with_password(false);
        let app = app(&test_state);
        let response = app
            .clone()
            .oneshot(request(Method::GET, "/login", "", None))
            .await
            .expect("login response");
        let cookie = session_cookie(&response);
        let body = response_body(response).await;
        let csrf_token = csrf(&body);
        let token = submission_token(&body);
        let response = app
            .clone()
            .oneshot(request(
                Method::POST,
                "/login",
                &format!("csrf={csrf_token}&submission_token={token}&password=wrong"),
                Some(&cookie),
            ))
            .await
            .expect("invalid password response");
        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body(response).await;
        assert!(body.contains("Invalid password."));
        assert_ne!(submission_token(&body), token);
        assert_eq!(
            test_state
                .auth_attempts
                .load(std::sync::atomic::Ordering::SeqCst),
            1
        );
        assert_eq!(
            test_state
                .password_verifications
                .load(std::sync::atomic::Ordering::SeqCst),
            1
        );
    }

    #[tokio::test]
    async fn malformed_login_fields_do_not_consume_a_valid_token() {
        let test_state = state(false);
        let app = app(&test_state);
        let response = app
            .clone()
            .oneshot(request(Method::GET, "/login", "", None))
            .await
            .expect("login response");
        let cookie = session_cookie(&response);
        let body = response_body(response).await;
        let csrf_token = csrf(&body);
        let token = submission_token(&body);
        let response = app
            .clone()
            .oneshot(request(
                Method::POST,
                "/login",
                &format!("csrf={csrf_token}&submission_token={token}"),
                Some(&cookie),
            ))
            .await
            .expect("malformed response");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            test_state
                .auth_attempts
                .load(std::sync::atomic::Ordering::SeqCst),
            0
        );

        let response = app
            .oneshot(request(
                Method::POST,
                "/login",
                &format!("csrf={csrf_token}&submission_token={token}&password=correct"),
                Some(&cookie),
            ))
            .await
            .expect("reused token response");
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
    }

    #[tokio::test]
    async fn login_markup_contains_native_and_pending_enhancement_contract() {
        let test_state = state(false);
        let app = app(&test_state);
        let response = app
            .oneshot(request(Method::GET, "/login", "", None))
            .await
            .expect("login response");
        let body = response_body(response).await;
        assert!(body.contains("/static/htmx.min.js"));
        assert!(body.contains("hx-post=\"/login\""));
        assert!(body.contains("hx-disabled-elt=\"button\""));
        assert!(body.contains("id=\"login-form-region\""));
        assert!(body.contains("aria-busy=\"false\""));
    }

    #[tokio::test]
    async fn rate_limited_login_returns_retry_after_and_fresh_recovery() {
        let test_state = state_with_login_admission(LoginAdmission::RetryAfter(17));
        let app = app(&test_state);
        let response = app
            .clone()
            .oneshot(request(Method::GET, "/login", "", None))
            .await
            .expect("login response");
        let cookie = session_cookie(&response);
        let body = response_body(response).await;
        let response = app
            .oneshot(request(
                Method::POST,
                "/login",
                &format!(
                    "csrf={}&submission_token={}&password=correct",
                    csrf(&body),
                    submission_token(&body)
                ),
                Some(&cookie),
            ))
            .await
            .expect("rate-limit response");
        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(
            response.headers().get("retry-after"),
            Some(&HeaderValue::from_static("17"))
        );
        let body = response_body(response).await;
        assert!(body.contains("Too many login attempts."));
        assert_eq!(
            test_state
                .password_verifications
                .load(std::sync::atomic::Ordering::SeqCst),
            0
        );
        assert!(body.matches("name=\"submission_token\"").count() >= 1);
    }

    #[tokio::test]
    async fn authenticated_login_requests_redirect_without_downgrading_session() {
        let test_state = state(false);
        let store = ReapingMemoryStore::default();
        let session = tower_sessions::Session::new(
            None,
            Arc::new(store.clone()),
            Some(session::anonymous_expiry()),
        );
        session
            .insert("authenticated", true)
            .await
            .expect("authenticated marker");
        session.set_expiry(Some(session::authenticated_expiry()));
        session.save().await.expect("save session");
        let id = session.id().expect("session id");
        let cookie = format!("{}={}", "id", id);

        let response = router_with_sessions(
            test_state.app,
            SessionManagerLayer::new(store)
                .with_secure(false)
                .with_expiry(session::anonymous_expiry())
                .with_always_save(true),
            Arc::new(tokio::sync::Semaphore::new(64)),
        )
        .oneshot(request(Method::GET, "/login", "", Some(&cookie)))
        .await
        .expect("authenticated login response");

        assert!(matches!(
            response.status(),
            StatusCode::OK | StatusCode::SEE_OTHER
        ));
    }

    #[tokio::test]
    async fn fixed_body_limits_reject_login_and_protected_forms_before_handlers() {
        let test_state = state(false);
        let app = app(&test_state);
        let oversized_login = format!("password={}", "x".repeat(9 * 1024));
        let response = app
            .clone()
            .oneshot(request(Method::POST, "/login", &oversized_login, None))
            .await
            .expect("oversized login response");
        assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);

        let session_cookie = login(&app).await;
        let oversized_form = format!("csrf=wrong&name={}", "x".repeat(256 * 1024));
        let response = app
            .oneshot(request(
                Method::POST,
                "/groups",
                &oversized_form,
                Some(&session_cookie),
            ))
            .await
            .expect("oversized protected response");
        assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
        assert!(
            test_state
                .groups
                .created
                .lock()
                .expect("group calls")
                .is_empty()
        );
    }

    fn assert_security_headers(response: &Response) {
        assert_eq!(
            response
                .headers()
                .get("cache-control")
                .expect("cache header"),
            "no-store"
        );
        assert_eq!(
            response
                .headers()
                .get("x-content-type-options")
                .expect("content type header"),
            "nosniff"
        );
        assert_eq!(
            response
                .headers()
                .get("referrer-policy")
                .expect("referrer header"),
            "no-referrer"
        );
        assert_eq!(
            response
                .headers()
                .get("content-security-policy")
                .expect("CSP header"),
            "default-src 'none'; script-src 'self'; script-src-attr 'none'; connect-src 'self'; style-src 'self' 'unsafe-inline'; form-action 'self'; base-uri 'none'; frame-ancestors 'none'"
        );
    }
}
