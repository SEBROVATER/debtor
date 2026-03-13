use chrono::{Duration, Utc};
use debtor::auth::login_service::{LoginResult, LoginService};
use debtor::auth::session_repo::SessionRepo;

#[path = "../support/mod.rs"]
mod support;

#[tokio::test]
async fn session_expiry_slides_on_activity() {
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

    let later = now + Duration::days(1);
    let updated = repo
        .touch_session(&token.raw, later)
        .await
        .expect("touch session")
        .expect("session exists");

    assert!(updated.expires_at > token.expires_at);
}
