use std::sync::{Arc, Mutex};

use chrono::{NaiveDate, NaiveDateTime};
use debtor::exchange_rates::rate_repo::RateRepo;
use debtor::exchange_rates::rate_service::{ExchangeProvider, RateError, RateQuote, RateService};
use rust_decimal::Decimal;
use std::str::FromStr;

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
async fn falls_back_to_stale_cache_on_provider_failure() {
    let (_dir, state, _hash) = support::setup_test_state().await;
    let repo = RateRepo::new(state.db.clone());

    let yesterday = NaiveDate::from_ymd_opt(2026, 2, 28).unwrap();
    let fetched_at =
        NaiveDateTime::parse_from_str("2026-02-28 08:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
    repo.insert_manual(
        "USD",
        "EUR",
        Decimal::from_str("0.88").unwrap(),
        fetched_at,
        yesterday,
        "fake",
    )
    .await
    .expect("insert");

    let calls = Arc::new(Mutex::new(0));
    let provider = FailingProvider {
        calls: calls.clone(),
    };
    let service = RateService::new(repo, Arc::new(provider));

    let now = NaiveDateTime::parse_from_str("2026-03-01 12:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
    let result = service.get_rate("USD", "EUR", now).await.expect("rate");

    assert!(result.stale);
    assert_eq!(result.rate.to_string(), "0.88");
    assert_eq!(*calls.lock().unwrap(), 1);
}
