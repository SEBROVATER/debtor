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
    router_with_sessions(
        state,
        SessionManagerLayer::new(ReapingMemoryStore::default())
            .with_secure(false)
            .with_expiry(session::anonymous_expiry())
            .with_always_save(true),
        Arc::new(Semaphore::new(64)),
    )
}

/// Builds the application router with the production-configured session layer.
pub fn router_with_sessions<S: SessionStore + Clone>(
    state: AppState,
    sessions: SessionManagerLayer<S>,
    user_limit: Arc<Semaphore>,
) -> Router {
    let public = Router::new()
        .route("/healthz", get(handlers::health))
        .route("/readyz", get(handlers::readiness))
        .layer(middleware::from_fn(app_middleware::probe_timeout))
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

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use std::sync::Arc;

    use axum::{
        body::{Body, to_bytes},
        extract::connect_info::ConnectInfo,
        http::{
            HeaderValue, Method, Request, StatusCode,
            header::{CONTENT_TYPE, COOKIE, SET_COOKIE},
        },
        response::Response,
    };
    use debtor_domain::currency::Currency;
    use tower::ServiceExt;
    use tower_sessions::{
        SessionManagerLayer, SessionStore,
        session::{Id, Record},
    };

    use super::{router, router_with_sessions};
    use crate::handlers::test_support::{
        TestState, state, state_with_errors, state_with_readiness_failure,
    };
    use crate::{
        session,
        session_store::{AUTHENTICATED_CAPACITY, ReapingMemoryStore},
    };

    const PEER: std::net::SocketAddr =
        std::net::SocketAddr::new(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST), 4000);

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

    async fn login(app: &axum::Router) -> String {
        let response = app
            .clone()
            .oneshot(request(Method::GET, "/login", "", None))
            .await
            .expect("login form response");
        let cookie = session_cookie(&response);
        let token = csrf(&response_body(response).await);
        let response = app
            .clone()
            .oneshot(request(
                Method::POST,
                "/login",
                &format!("csrf={token}&password=correct"),
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
            let token = csrf(&response_body(response).await);
            submissions.push((cookie, token));
        }
        let [(first_cookie, first_token), (second_cookie, second_token)]: [(String, String); 2] =
            submissions.try_into().expect("two login submissions");
        let first = app.clone().oneshot(request(
            Method::POST,
            "/login",
            &format!("csrf={first_token}&password=correct"),
            Some(&first_cookie),
        ));
        let second = app.clone().oneshot(request(
            Method::POST,
            "/login",
            &format!("csrf={second_token}&password=correct"),
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
            (AUTHENTICATED_CAPACITY, 0, AUTHENTICATED_CAPACITY)
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
                &format!("csrf={}&name=Renamed&currency=USD", csrf(&edit_form)),
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
                &format!("csrf={}&name=New+group&currency=USD", csrf(&groups_page)),
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
                    "csrf={}&name=New+person&color=%23abcdef",
                    csrf(&participants_page)
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
                    "csrf={}&name=Joined+person&color=%23fedcba",
                    csrf(&group_detail_page)
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
                    "csrf={}&name=Edited+person&color=%23aabbcc",
                    csrf(&participant_edit_page)
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
                &format!("csrf={}&name=Updated+group&currency=EUR", csrf(&form)),
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
                    "description=Lunch&total=12.00&currency=USD&spending_type=food&spent_date=2026-08-04&payer_mode=single&single_payer_id=1&split_mode=equal&csrf={}&share_1=on",
                    csrf(&group_page)
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
            .oneshot(request(Method::GET, "/readyz", "", None))
            .await
            .expect("readiness response");
        assert_eq!(readiness.status(), StatusCode::SERVICE_UNAVAILABLE);
        let body = response_body(readiness).await;
        assert!(body.contains("Service temporarily unavailable."));
        assert!(!body.contains("unexpected storage failure"));
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
        let login_body = response_body(response).await;
        let response = app
            .clone()
            .oneshot(request(
                Method::POST,
                "/login",
                &format!("csrf={}&password=correct", csrf(&login_body)),
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
            "default-src 'none'; script-src 'none'; style-src 'self' 'unsafe-inline'; form-action 'self'; base-uri 'none'; frame-ancestors 'none'"
        );
    }
}
