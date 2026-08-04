//! Axum route definitions.

use axum::{
    Router,
    routing::{get, post},
};

use crate::{handlers, state::AppState};

/// Builds the application router from application-facing state.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(handlers::root))
        .route("/healthz", get(handlers::health))
        .route("/readyz", get(handlers::health))
        .route("/login", get(handlers::login_form).post(handlers::login))
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
        .with_state(state)
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
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
    use tower_sessions::{MemoryStore, SessionManagerLayer};

    use super::router;
    use crate::handlers::test_support::{TestState, state, state_with_errors};

    const PEER: std::net::SocketAddr =
        std::net::SocketAddr::new(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST), 4000);

    fn app(test_state: &TestState) -> axum::Router {
        router(test_state.app.clone()).layer(
            SessionManagerLayer::new(MemoryStore::default())
                .with_secure(false)
                .with_always_save(true),
        )
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
}
