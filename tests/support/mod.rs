use argon2::{Argon2, PasswordHasher, password_hash::SaltString};
use chrono::Utc;
use debtor::app::config::AppConfig;
use debtor::app::state::AppState;
use debtor::db::bootstrap::initialize_database;
use debtor::db::entities::admin_users;
use debtor::exchange_rates::rate_service::{ExchangeProvider, RateError, RateQuote};
use sea_orm::{ActiveModelTrait, DatabaseConnection, Set};
use std::sync::Arc;
use tempfile::TempDir;

pub fn temp_sqlite_url() -> (TempDir, String) {
    let dir = tempfile::tempdir().expect("tempdir");
    let db_path = dir.path().join("test.db");
    let db_path_str = db_path.to_string_lossy().replace("\\", "/");
    let url = format!("sqlite://{}?mode=rwc", db_path_str);
    (dir, url)
}

pub fn hash_password(password: &str) -> String {
    let salt = SaltString::encode_b64(b"debtor-test-salt").expect("salt");
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .expect("hash password")
        .to_string()
}

pub async fn seed_admin_user(
    conn: &DatabaseConnection,
    username: &str,
    password_hash: &str,
) -> admin_users::Model {
    let now = Utc::now().naive_utc();
    admin_users::ActiveModel {
        id: Set(1),
        username: Set(username.to_string()),
        password_hash: Set(password_hash.to_string()),
        created_at: Set(now),
        updated_at: Set(now),
    }
    .insert(conn)
    .await
    .expect("insert admin user")
}

pub async fn setup_test_state() -> (TempDir, Arc<AppState>, String) {
    let (dir, url) = temp_sqlite_url();
    let conn = initialize_database(&url).await.expect("init db");
    let password_hash = hash_password("correct_password");
    seed_admin_user(&conn, "owner", &password_hash).await;

    let config = AppConfig {
        database_url: url.clone(),
        session_cookie_name: "debtor_session".to_string(),
        admin_username: "owner".to_string(),
        admin_password_hash: Some(password_hash.clone()),
        secure_cookie: false,
        exchange_base_url: "https://api.frankfurter.app".to_string(),
    };

    let provider = Arc::new(NoopProvider);
    let state = AppState::new(config, conn, Some(provider));
    (dir, state, password_hash)
}

#[derive(Clone)]
struct NoopProvider;

#[async_trait::async_trait]
impl ExchangeProvider for NoopProvider {
    async fn fetch_rates(&self, _from: &str, _to: &[String]) -> Result<Vec<RateQuote>, RateError> {
        Err(RateError::ProviderFailed("noop".to_string()))
    }
}
