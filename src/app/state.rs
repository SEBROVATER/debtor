use std::sync::Arc;

use sqlx::SqlitePool;

use crate::app::config::AppConfig;
use crate::exchange_rates::rate_service::ExchangeProvider;

pub struct AppState {
    pub config: AppConfig,
    pub db: SqlitePool,
    pub exchange_provider: Option<Arc<dyn ExchangeProvider>>,
}

impl AppState {
    pub fn new(
        config: AppConfig,
        db: SqlitePool,
        exchange_provider: Option<Arc<dyn ExchangeProvider>>,
    ) -> Arc<Self> {
        Arc::new(Self {
            config,
            db,
            exchange_provider,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::AppState;
    use crate::app::config::AppConfig;
    use crate::exchange_rates::rate_service::{ExchangeProvider, RateError, RateQuote};
    use sqlx::SqlitePool;
    use std::sync::Arc;

    #[tokio::test]
    async fn stores_config_in_shared_state() {
        let cfg = AppConfig {
            database_url: "sqlite::memory:".to_string(),
            session_cookie_name: "test_session".to_string(),
            admin_username: "owner".to_string(),
            admin_password_hash: None,
            secure_cookie: false,
            exchange_base_url: "https://api.frankfurter.app".to_string(),
        };

        let db = SqlitePool::connect("sqlite::memory:").await.expect("db");
        let provider = Arc::new(NoopProvider);
        let state = AppState::new(cfg, db, Some(provider));
        assert_eq!(state.config.session_cookie_name, "test_session");
    }

    #[derive(Clone)]
    struct NoopProvider;

    #[async_trait::async_trait]
    impl ExchangeProvider for NoopProvider {
        async fn fetch_rates(
            &self,
            _from: &str,
            _to: &[String],
        ) -> Result<Vec<RateQuote>, RateError> {
            Err(RateError::ProviderFailed("noop".to_string()))
        }
    }
}
