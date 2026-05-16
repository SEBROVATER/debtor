use chrono::Utc;
use debtor::auth::login_service::{LoginResult, LoginService};
use debtor::groups::group_service::GroupService;
use std::time::Instant;

#[path = "../support/mod.rs"]
mod support;

#[tokio::test]
async fn login_to_dashboard_meets_budget() {
    let (_dir, state, _hash) = support::setup_test_state().await;
    let login_service = LoginService::new(state.db.clone());
    let group_service = GroupService::new(state.db.clone());
    let now = Utc::now().naive_utc();

    let start = Instant::now();
    let result = login_service
        .login("owner", "correct_password", now)
        .await
        .expect("login");

    match result {
        LoginResult::Success(_) => {}
        other => panic!("unexpected login result: {other:?}"),
    }

    let _ = group_service.list_groups().await.expect("list groups");
    let elapsed = start.elapsed();

    assert!(elapsed.as_millis() < 5000, "took {:?}", elapsed);
}
