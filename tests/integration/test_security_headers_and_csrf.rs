use debtor::app::config::AppConfig;
use debtor::auth::session_repo::{SameSitePolicy, SessionCookiePolicy};
use debtor::web::csrf::{CsrfError, CsrfToken, validate_csrf};
use debtor::web::router::RouteMethod;

#[test]
fn session_cookie_policy_remains_secure_defaults() {
    let config = AppConfig {
        database_url: "sqlite://test.db?mode=rwc".to_string(),
        session_cookie_name: "debtor_session".to_string(),
        admin_username: "owner".to_string(),
        admin_password_hash: None,
        secure_cookie: true,
        exchange_base_url: "https://api.frankfurter.app".to_string(),
    };

    let policy = SessionCookiePolicy::from_config(&config);

    assert!(policy.http_only);
    assert_eq!(policy.same_site, SameSitePolicy::Lax);
    assert!(policy.secure);
    assert_eq!(policy.max_age_days, 30);
    assert_eq!(policy.path, "/");
}

#[test]
fn csrf_tokens_rotate_and_reject_old_values() {
    let first = CsrfToken::generate();
    let second = CsrfToken::generate();

    assert_ne!(first.value(), second.value());

    let result = validate_csrf(RouteMethod::Post, Some(&first), Some(&second));
    assert!(matches!(result, Err(CsrfError::Mismatch)));
}
