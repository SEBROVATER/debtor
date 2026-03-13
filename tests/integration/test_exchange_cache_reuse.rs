use std::sync::{Arc, Mutex};

use chrono::{NaiveDate, NaiveDateTime};
use debtor::exchange_rates::rate_repo::RateRepo;
use debtor::exchange_rates::rate_service::{ExchangeProvider, RateError, RateQuote, RateService};
use rust_decimal::Decimal;
use std::str::FromStr;

#[path = "../support/mod.rs"]
mod support;

#[derive(Clone)]
struct FakeProvider {
    calls: Arc<Mutex<usize>>,
}

#[async_trait::async_trait]
impl ExchangeProvider for FakeProvider {
    async fn fetch_rates(&self, _from: &str, _to: &[String]) -> Result<Vec<RateQuote>, RateError> {
        *self.calls.lock().unwrap() += 1;
        Ok(Vec::new())
    }
}

#[tokio::test]
async fn reuses_same_day_cache_without_fetching() {
    let (_dir, state, _hash) = support::setup_test_state().await;
    let repo = RateRepo::new(state.db.clone());

    let now = NaiveDateTime::parse_from_str("2026-03-01 12:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
    repo.insert_manual(
        "USD",
        "EUR",
        Decimal::from_str("0.91").unwrap(),
        now,
        NaiveDate::from_ymd_opt(2026, 3, 1).unwrap(),
        "fake",
    )
    .await
    .expect("insert");

    let calls = Arc::new(Mutex::new(0));
    let provider = FakeProvider { calls: calls.clone() };
    let service = RateService::new(repo, Arc::new(provider));

    let result = service.get_rate("USD", "EUR", now).await.expect("rate");
    assert_eq!(result.rate.to_string(), "0.91");
    assert_eq!(*calls.lock().unwrap(), 0);
}
