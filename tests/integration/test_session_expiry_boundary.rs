use chrono::{Duration, Utc};
use debtor::auth::login_service::{LoginResult, LoginService};
use debtor::auth::session_repo::SessionRepo;

#[path = "../support/mod.rs"]
mod support;

#[tokio::test]
async fn expired_session_is_rejected() {
    let (_dir, state, _hash) = support::setup_test_state().await;
    let service = LoginService::new(state.db.clone());
    let repo = SessionRepo::new(state.db.clone());

    let now = Utc::now().naive_utc();
    let result = service
        .login("owner", "correct_password", now)
        .await
        .expect("login");

    let LoginResult::Success(token) = result else {
        panic!("expected successful login");
    };

    let expired = now + Duration::days(31);
    let session = repo
        .find_active_session(&token.raw, expired)
        .await
        .expect("find session");
    assert!(session.is_none(), "session should be expired");
}
