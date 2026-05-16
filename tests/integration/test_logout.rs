use chrono::Utc;
use debtor::auth::login_service::{LoginResult, LoginService};
use debtor::auth::session_repo::SessionRepo;

#[path = "../support/mod.rs"]
mod support;

#[tokio::test]
async fn logout_revokes_session() {
    let (_dir, state, _hash) = support::setup_test_state().await;
    let service = LoginService::new(state.db.clone());
    let now = Utc::now().naive_utc();

    let result = service
        .login("owner", "correct_password", now)
        .await
        .expect("login");

    let LoginResult::Success(token) = result else {
        panic!("expected successful login");
    };

    let revoked = service.logout(&token.raw, now).await.expect("logout");
    assert!(revoked, "logout should revoke session");

    let repo = SessionRepo::new(state.db.clone());
    let active = repo
        .find_active_session(&token.raw, now)
        .await
        .expect("session lookup");
    assert!(active.is_none(), "session should not be active");

    let stored = sqlx::query!(
        "SELECT revoked_at FROM sessions WHERE token_hash = ?",
        token.hash
    )
    .fetch_one(&state.db)
    .await
    .expect("session row");
    assert!(stored.revoked_at.is_some(), "revoked_at should be set");
}
