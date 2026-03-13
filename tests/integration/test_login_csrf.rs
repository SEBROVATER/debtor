use chrono::Utc;
use debtor::web::error::AppError;
use debtor::web::handlers::auth_handlers::{LoginRequest, handle_login};

#[path = "../support/mod.rs"]
mod support;

#[tokio::test]
async fn login_requires_csrf_token() {
    let (_dir, state, _hash) = support::setup_test_state().await;
    let now = Utc::now().naive_utc();

    let request = LoginRequest {
        username: "owner".to_string(),
        password: "correct_password".to_string(),
        csrf_token: None,
    };

    let result = handle_login(&state, request, None, now).await;
    assert!(matches!(result, Err(AppError::Csrf(_))));
}
