use debtor::app::config::AppConfig;
use debtor::auth::session_repo::{SameSitePolicy, SessionCookiePolicy};

#[test]
fn session_cookie_policy_defaults_to_http_only_lax_and_insecure() {
    let config = AppConfig {
        database_url: "sqlite://test.db?mode=rwc".to_string(),
        session_cookie_name: "debtor_session".to_string(),
        admin_username: "owner".to_string(),
        admin_password_hash: None,
        secure_cookie: false,
        exchange_base_url: "https://api.frankfurter.app".to_string(),
    };

    let policy = SessionCookiePolicy::from_config(&config);

    assert!(policy.http_only);
    assert_eq!(policy.same_site, SameSitePolicy::Lax);
    assert!(!policy.secure);
    assert_eq!(policy.max_age_days, 30);
    assert_eq!(policy.path, "/");
}

#[test]
fn session_cookie_policy_respects_secure_flag() {
    let config = AppConfig {
        database_url: "sqlite://test.db?mode=rwc".to_string(),
        session_cookie_name: "debtor_session".to_string(),
        admin_username: "owner".to_string(),
        admin_password_hash: None,
        secure_cookie: true,
        exchange_base_url: "https://api.frankfurter.app".to_string(),
    };

    let policy = SessionCookiePolicy::from_config(&config);
    assert!(policy.secure);
}
