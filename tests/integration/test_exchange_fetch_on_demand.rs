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
    rate: Decimal,
    rate_date: NaiveDate,
    should_fail: bool,
}

#[async_trait::async_trait]
impl ExchangeProvider for FakeProvider {
    async fn fetch_rates(&self, from: &str, to: &[String]) -> Result<Vec<RateQuote>, RateError> {
        *self.calls.lock().unwrap() += 1;
        if self.should_fail {
            return Err(RateError::ProviderFailed("boom".to_string()));
        }
        Ok(to
            .iter()
            .map(|currency| RateQuote {
                from_currency: from.to_string(),
                to_currency: currency.clone(),
                rate: self.rate,
                fetched_at: NaiveDateTime::parse_from_str(
                    "2026-03-01 10:00:00",
                    "%Y-%m-%d %H:%M:%S",
                )
                .unwrap(),
                rate_date: self.rate_date,
                provider: "fake".to_string(),
            })
            .collect())
    }
}

#[tokio::test]
async fn fetches_on_demand_when_cache_missing() {
    let (_dir, state, _hash) = support::setup_test_state().await;
    let repo = RateRepo::new(state.db.clone());
    let calls = Arc::new(Mutex::new(0));
    let provider = FakeProvider {
        calls: calls.clone(),
        rate: Decimal::from_str("1.25").unwrap(),
        rate_date: NaiveDate::from_ymd_opt(2026, 3, 1).unwrap(),
        should_fail: false,
    };
    let service = RateService::new(repo, Arc::new(provider));

    let now = NaiveDateTime::parse_from_str("2026-03-01 12:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
    let result = service.get_rate("USD", "EUR", now).await.expect("rate");

    assert_eq!(result.rate.to_string(), "1.25");
    assert!(!result.stale);
    assert_eq!(*calls.lock().unwrap(), 1);
}
