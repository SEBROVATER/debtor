use chrono::Utc;
use debtor::auth::login_service::{LoginResult, LoginService};
use debtor::db::entities::auth_state;
use sea_orm::EntityTrait;

#[path = "../support/mod.rs"]
mod support;

#[tokio::test]
async fn lockout_triggers_after_five_failures() {
    let (_dir, state, _hash) = support::setup_test_state().await;
    let service = LoginService::new(state.db.clone());
    let now = Utc::now().naive_utc();

    for _ in 0..4 {
        let result = service
            .login("owner", "wrong_password", now)
            .await
            .expect("login");
        assert!(matches!(result, LoginResult::InvalidCredentials));
    }

    let result = service
        .login("owner", "wrong_password", now)
        .await
        .expect("login");
    assert!(matches!(result, LoginResult::LockedOut { .. }));

    let stored = auth_state::Entity::find_by_id(1)
        .one(&state.db)
        .await
        .expect("auth_state")
        .expect("auth_state row");
    assert!(stored.lockout_until.is_some(), "lockout should be set");
}
