use chrono::Utc;
use debtor::auth::login_service::{LoginResult, LoginService};
use debtor::auth::session_repo::SessionRepo;

#[path = "../support/mod.rs"]
mod support;

#[tokio::test]
async fn login_with_valid_credentials_creates_session() {
    let (_dir, state, _hash) = support::setup_test_state().await;
    let service = LoginService::new(state.db.clone());
    let now = Utc::now().naive_utc();

    let result = service
        .login("owner", "correct_password", now)
        .await
        .expect("login");

    match result {
        LoginResult::Success(token) => {
            let repo = SessionRepo::new(state.db.clone());
            let session = repo
                .find_active_session(&token.raw, now)
                .await
                .expect("session lookup");
            assert!(session.is_some(), "expected session to be stored");
        }
        other => panic!("unexpected login result: {other:?}"),
    }
}
