use std::sync::{Arc, Mutex};

use chrono::NaiveDateTime;
use debtor::exchange_rates::rate_repo::RateRepo;
use debtor::exchange_rates::rate_service::{ExchangeProvider, RateError, RateQuote, RateService};

#[path = "../support/mod.rs"]
mod support;

#[derive(Clone)]
struct FailingProvider {
    calls: Arc<Mutex<usize>>,
}

#[async_trait::async_trait]
impl ExchangeProvider for FailingProvider {
    async fn fetch_rates(&self, _from: &str, _to: &[String]) -> Result<Vec<RateQuote>, RateError> {
        *self.calls.lock().unwrap() += 1;
        Err(RateError::ProviderFailed("down".to_string()))
    }
}

#[tokio::test]
async fn provider_failure_without_cache_errors() {
    let (_dir, state, _hash) = support::setup_test_state().await;
    let repo = RateRepo::new(state.db.clone());

    let calls = Arc::new(Mutex::new(0));
    let provider = FailingProvider { calls: calls.clone() };
    let service = RateService::new(repo, Arc::new(provider));

    let now = NaiveDateTime::parse_from_str("2026-03-01 12:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
    let result = service.get_rate("USD", "EUR", now).await;

    assert!(matches!(result, Err(RateError::MissingRate)));
    assert_eq!(*calls.lock().unwrap(), 1);
}
