use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use async_trait::async_trait;
use chrono::NaiveDate;
use debtor_application::{ApplicationError, ExchangeRateProvider, RateQuote, UnavailableReason};
use debtor_domain::currency::Currency;
use futures::FutureExt;
use rust_decimal::Decimal;
use serde::Deserialize;
use tokio::sync::{Mutex, Notify, Semaphore};

const DEFAULT_BASE_URL: &str = "https://api.frankfurter.dev/v2";
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const TOTAL_TIMEOUT: Duration = Duration::from_secs(20);
const MAX_RESPONSE_BYTES: usize = 64 * 1024;
const MAX_IN_FLIGHT_REQUESTS: usize = 4;
type CacheKey = (Currency, Currency, NaiveDate, NaiveDate);
type FlightResult = Result<RateQuote, UnavailableReason>;

#[derive(Debug, Deserialize)]
struct RateResponse {
    date: NaiveDate,
    #[serde(with = "rust_decimal::serde::arbitrary_precision")]
    rate: Decimal,
}

struct Flight {
    result: Mutex<Option<FlightResult>>,
    notify: Notify,
}

impl Flight {
    fn new() -> Self {
        Self {
            result: Mutex::new(None),
            notify: Notify::new(),
        }
    }

    async fn wait(&self) -> FlightResult {
        loop {
            let notified = self.notify.notified();
            if let Some(result) = self.result.lock().await.clone() {
                return result;
            }
            notified.await;
        }
    }
}

/// Frankfurter v2 exchange provider with dated process-local caching.
#[derive(Clone)]
pub struct FrankfurterClient {
    http: Option<reqwest::Client>,
    base_url: String,
    stable_cache: Arc<RwLock<HashMap<CacheKey, RateQuote>>>,
    refreshable_cache: Arc<RwLock<HashMap<CacheKey, RateQuote>>>,
    in_flight: Arc<Mutex<HashMap<CacheKey, Arc<Flight>>>>,
    requests: Arc<Semaphore>,
}

impl FrankfurterClient {
    /// Creates a client using the public v2 endpoint.
    pub fn new() -> Self {
        Self::with_base_url(DEFAULT_BASE_URL)
    }

    /// Creates a client using a custom endpoint, intended for local tests.
    pub fn with_base_url(base_url: &str) -> Self {
        let http = reqwest::Client::builder()
            .connect_timeout(CONNECT_TIMEOUT)
            .timeout(TOTAL_TIMEOUT)
            .read_timeout(TOTAL_TIMEOUT)
            .build()
            .ok();
        Self {
            http,
            base_url: base_url.trim_end_matches('/').to_owned(),
            stable_cache: Arc::new(RwLock::new(HashMap::new())),
            refreshable_cache: Arc::new(RwLock::new(HashMap::new())),
            in_flight: Arc::new(Mutex::new(HashMap::new())),
            requests: Arc::new(Semaphore::new(MAX_IN_FLIGHT_REQUESTS)),
        }
    }

    #[cfg(test)]
    fn with_http_client(base_url: &str, http: reqwest::Client) -> Self {
        let mut client = Self::with_base_url(base_url);
        client.http = Some(http);
        client
    }
}

impl Default for FrankfurterClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ExchangeRateProvider for FrankfurterClient {
    async fn rate(
        &self,
        base: Currency,
        quote: Currency,
        requested_date: NaiveDate,
        today: NaiveDate,
    ) -> Result<RateQuote, ApplicationError> {
        let original_requested_date = requested_date;
        let fetch_date = requested_date.min(today);
        if base == quote {
            return Ok(RateQuote {
                base,
                quote,
                requested_date: original_requested_date,
                effective_date: fetch_date,
                rate: Decimal::ONE,
                is_stale: false,
                is_provisional: original_requested_date > today,
            });
        }
        let key = (base, quote, original_requested_date, fetch_date);
        let cache = self.cache_for(original_requested_date, today);
        let cached = cache
            .read()
            .map_err(|_| ApplicationError::Unavailable(UnavailableReason::ExchangeRates))?
            .get(&key)
            .cloned();
        if let Some(value) = cached {
            return Ok(value);
        }

        let (flight, leader) = {
            let mut in_flight = self.in_flight.lock().await;
            if let Some(flight) = in_flight.get(&key) {
                (flight.clone(), false)
            } else {
                let flight = Arc::new(Flight::new());
                in_flight.insert(key, flight.clone());
                (flight, true)
            }
        };
        if leader {
            let client = self.clone();
            let worker_flight = flight.clone();
            tokio::spawn(async move {
                let result = std::panic::AssertUnwindSafe(client.fetch_uncached(key, today))
                    .catch_unwind()
                    .await
                    .unwrap_or(Err(UnavailableReason::ExchangeRates));
                client.finish_flight(key, worker_flight, result).await;
            });
        }
        flight.wait().await.map_err(ApplicationError::Unavailable)
    }
}

impl FrankfurterClient {
    async fn fetch_uncached(&self, key: CacheKey, today: NaiveDate) -> FlightResult {
        let permit = self
            .requests
            .clone()
            .acquire_owned()
            .await
            .map_err(|_| UnavailableReason::ExchangeRates)?;
        let result = self.fetch_with_permit(key, today).await;
        drop(permit);
        result
    }

    async fn fetch_with_permit(&self, key: CacheKey, today: NaiveDate) -> FlightResult {
        let (base, quote, original_requested_date, fetch_date) = key;
        let Some(http) = self.http.as_ref() else {
            return self.stale_or_error(key, today);
        };
        let url = format!(
            "{}/rate/{}/{}?date={fetch_date}",
            self.base_url,
            base.code(),
            quote.code()
        );
        let mut response = match http.get(url).send().await {
            Ok(response) if response.status().is_success() => response,
            Ok(_) | Err(_) => return self.stale_or_error(key, today),
        };
        let mut body = Vec::new();
        loop {
            match response.chunk().await {
                Ok(Some(chunk)) => {
                    if chunk.len() > MAX_RESPONSE_BYTES.saturating_sub(body.len()) {
                        return self.stale_or_error(key, today);
                    }
                    body.extend_from_slice(&chunk);
                }
                Ok(None) => break,
                Err(_) => return self.stale_or_error(key, today),
            }
        }
        let payload: RateResponse = match serde_json::from_slice(&body) {
            Ok(payload) => payload,
            Err(_) => return self.stale_or_error(key, today),
        };
        if payload.date > fetch_date || payload.rate <= Decimal::ZERO {
            return self.stale_or_error(key, today);
        }
        let value = RateQuote {
            base,
            quote,
            requested_date: original_requested_date,
            effective_date: payload.date,
            rate: payload.rate,
            is_stale: false,
            is_provisional: original_requested_date > today,
        };
        let cache = self.cache_for(original_requested_date, today);
        cache
            .write()
            .map_err(|_| UnavailableReason::ExchangeRates)?
            .insert(key, value.clone());
        Ok(value)
    }

    async fn finish_flight(&self, key: CacheKey, flight: Arc<Flight>, result: FlightResult) {
        *flight.result.lock().await = Some(result);
        flight.notify.notify_waiters();
        let mut in_flight = self.in_flight.lock().await;
        if in_flight
            .get(&key)
            .is_some_and(|candidate| Arc::ptr_eq(candidate, &flight))
        {
            in_flight.remove(&key);
        }
    }

    fn stale_or_error(&self, key: CacheKey, today: NaiveDate) -> FlightResult {
        let (base, quote, requested_date, fetch_date) = key;
        if requested_date < today {
            tracing::warn!(
                target: "debtor.provider",
                event = "provider_fallback",
                category = "historical_unavailable",
            );
            return Err(UnavailableReason::ExchangeRates);
        }
        let stale = self.refreshable_cache.read().ok().and_then(|cache| {
            cache
                .iter()
                .filter(
                    |((cached_base, cached_quote, cached_requested, cached_fetch), _)| {
                        *cached_base == base
                            && *cached_quote == quote
                            && *cached_fetch < fetch_date
                            && if requested_date == today {
                                *cached_requested == *cached_fetch
                            } else {
                                *cached_requested == requested_date
                            }
                    },
                )
                .max_by_key(|((_, _, _, cached_fetch), _)| *cached_fetch)
                .map(|(_, quote)| quote.clone())
        });
        if let Some(mut quote) = stale {
            tracing::info!(
                target: "debtor.provider",
                event = "provider_fallback",
                category = if requested_date > today {
                    "future_stale_cache"
                } else {
                    "current_stale_cache"
                },
            );
            return Ok({
                quote.requested_date = requested_date;
                quote.is_stale = true;
                quote.is_provisional = requested_date > today;
                quote
            });
        }

        tracing::warn!(
            target: "debtor.provider",
            event = "provider_fallback",
            category = if requested_date > today {
                "future_unavailable"
            } else {
                "current_unavailable"
            },
        );
        Err(UnavailableReason::ExchangeRates)
    }

    fn cache_for(
        &self,
        requested_date: NaiveDate,
        today: NaiveDate,
    ) -> &RwLock<HashMap<CacheKey, RateQuote>> {
        if requested_date < today {
            self.stable_cache.as_ref()
        } else {
            self.refreshable_cache.as_ref()
        }
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use std::{
        io::Write,
        sync::atomic::{AtomicUsize, Ordering},
        sync::{Arc, Mutex},
    };

    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;
    use tracing_subscriber::fmt::MakeWriter;

    #[derive(Clone)]
    struct SharedWriter(Arc<Mutex<Vec<u8>>>);

    struct Writer(Arc<Mutex<Vec<u8>>>);

    impl Write for Writer {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0.lock().expect("writer lock").extend_from_slice(bytes);
            Ok(bytes.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    impl<'a> MakeWriter<'a> for SharedWriter {
        type Writer = Writer;

        fn make_writer(&'a self) -> Self::Writer {
            Writer(self.0.clone())
        }
    }

    #[tokio::test]
    async fn identity_rate_is_exact_without_network() {
        let client = FrankfurterClient::with_base_url("http://127.0.0.1:1");
        let date = date(2026, 1, 1);
        assert_eq!(
            client
                .rate(Currency::Usd, Currency::Usd, date, date)
                .await
                .expect("identity rate")
                .rate,
            Decimal::ONE
        );
    }

    #[test]
    fn provider_fallback_event_contains_only_a_safe_category() {
        let buffer = Arc::new(Mutex::new(Vec::new()));
        let subscriber = tracing_subscriber::fmt()
            .with_ansi(false)
            .without_time()
            .with_max_level(tracing::Level::INFO)
            .with_writer(SharedWriter(buffer.clone()))
            .finish();
        let date = date(2026, 1, 1);
        let client = FrankfurterClient::with_base_url(
            "https://provider-sentinel.invalid/secret-password=sentinel",
        );
        let result = tracing::subscriber::with_default(subscriber, || {
            client.stale_or_error((Currency::Usd, Currency::Eur, date, date), date)
        });
        assert_eq!(result, Err(UnavailableReason::ExchangeRates));
        let output =
            String::from_utf8(buffer.lock().expect("log buffer").clone()).expect("UTF-8 logs");
        assert!(output.contains("provider_fallback"));
        assert!(output.contains("current_unavailable"));
        assert!(!output.contains("provider-sentinel"));
        assert!(!output.contains("sentinel"));
    }

    #[tokio::test]
    async fn decodes_rate_without_f64_rounding() {
        let (base_url, server) = server_with_responses(vec![(
            r#"{"date":"2026-01-01","rate":1.2345678901234567890123456789}"#,
            Duration::ZERO,
        )])
        .await;
        let client = FrankfurterClient::with_base_url(&base_url);
        let quote = client
            .rate(
                Currency::Usd,
                Currency::Eur,
                date(2026, 1, 1),
                date(2026, 1, 1),
            )
            .await
            .expect("exact rate");

        assert_eq!(quote.rate.to_string(), "1.2345678901234567890123456789");
        server.await.expect("server task");
    }

    #[tokio::test]
    async fn rejects_nonpositive_future_and_malformed_payloads() {
        for payload in [
            r#"{"date":"2026-01-01","rate":0}"#,
            r#"{"date":"2026-01-02","rate":1}"#,
            r#"{"date":"2026-01-01","rate":1e999}"#,
            "not-json",
        ] {
            let (base_url, server) = server_with_responses(vec![(payload, Duration::ZERO)]).await;
            let client = FrankfurterClient::with_base_url(&base_url);
            assert!(
                client
                    .rate(
                        Currency::Usd,
                        Currency::Eur,
                        date(2026, 1, 1),
                        date(2026, 1, 1),
                    )
                    .await
                    .is_err()
            );
            server.await.expect("server task");
        }
    }

    #[tokio::test]
    async fn rejects_oversized_response_bodies() {
        let body = format!(
            "{{\"date\":\"2026-01-01\",\"rate\":1,\"padding\":\"{}\"}}",
            "x".repeat(MAX_RESPONSE_BYTES)
        );
        let (base_url, server) = server_with_responses(vec![(&body, Duration::ZERO)]).await;
        let client = FrankfurterClient::with_base_url(&base_url);

        assert!(
            client
                .rate(
                    Currency::Usd,
                    Currency::Eur,
                    date(2026, 1, 1),
                    date(2026, 1, 1),
                )
                .await
                .is_err()
        );
        server.await.expect("server task");
    }

    #[tokio::test]
    async fn single_flight_shares_one_successful_request() {
        let count = Arc::new(AtomicUsize::new(0));
        let (base_url, server) = server_with_counter(
            r#"{"date":"2026-01-01","rate":1}"#,
            Duration::from_millis(25),
            count.clone(),
            1,
        )
        .await;
        let client = Arc::new(FrankfurterClient::with_base_url(&base_url));
        let mut calls = Vec::new();
        for _ in 0..8 {
            let client = client.clone();
            calls.push(tokio::spawn(async move {
                client
                    .rate(
                        Currency::Usd,
                        Currency::Eur,
                        date(2026, 1, 1),
                        date(2026, 1, 1),
                    )
                    .await
            }));
        }
        for call in calls {
            call.await.expect("caller task").expect("shared rate");
        }
        assert_eq!(count.load(Ordering::SeqCst), 1);
        server.await.expect("server task");
    }

    #[tokio::test]
    async fn global_provider_requests_never_exceed_four() {
        let count = Arc::new(AtomicUsize::new(0));
        let active = Arc::new(AtomicUsize::new(0));
        let maximum = Arc::new(AtomicUsize::new(0));
        let (base_url, server) = server_with_concurrency(
            r#"{"date":"2026-01-01","rate":1}"#,
            Duration::from_millis(25),
            count.clone(),
            active,
            maximum.clone(),
            8,
        )
        .await;
        let client = Arc::new(FrankfurterClient::with_base_url(&base_url));
        let mut calls = Vec::new();
        for day in 1..=8 {
            let client = client.clone();
            calls.push(tokio::spawn(async move {
                client
                    .rate(
                        Currency::Usd,
                        Currency::Eur,
                        date(2026, 1, day),
                        date(2026, 1, 8),
                    )
                    .await
            }));
        }
        for call in calls {
            call.await.expect("caller task").expect("rate response");
        }
        assert_eq!(count.load(Ordering::SeqCst), 8);
        assert!(maximum.load(Ordering::SeqCst) <= MAX_IN_FLIGHT_REQUESTS);
        server.await.expect("server task");
    }

    #[tokio::test]
    async fn timeout_client_returns_provider_error() {
        let (base_url, server) = server_with_responses(vec![(
            r#"{"date":"2026-01-01","rate":1}"#,
            Duration::from_millis(100),
        )])
        .await;
        let http = reqwest::Client::builder()
            .connect_timeout(Duration::from_millis(10))
            .timeout(Duration::from_millis(10))
            .read_timeout(Duration::from_millis(10))
            .build()
            .expect("test client");
        let client = FrankfurterClient::with_http_client(&base_url, http);

        assert!(
            client
                .rate(
                    Currency::Usd,
                    Currency::Eur,
                    date(2026, 1, 1),
                    date(2026, 1, 1),
                )
                .await
                .is_err()
        );
        server.await.expect("server task");
    }

    #[tokio::test]
    async fn current_rollover_failure_uses_latest_prior_current_quote() {
        let client = FrankfurterClient::with_base_url("http://127.0.0.1:1");
        let previous = date(2026, 1, 1);
        let today = date(2026, 1, 2);
        insert_refreshable(
            &client,
            Currency::Usd,
            Currency::Eur,
            previous,
            previous,
            previous,
        );

        let quote = client
            .rate(Currency::Usd, Currency::Eur, today, today)
            .await
            .expect("prior current quote is a fallback");

        assert_eq!(quote.requested_date, today);
        assert_eq!(quote.effective_date, previous);
        assert!(quote.is_stale);
        assert!(!quote.is_provisional);
    }

    #[tokio::test]
    async fn future_rollover_failure_uses_only_the_same_future_context() {
        let client = FrankfurterClient::with_base_url("http://127.0.0.1:1");
        let previous = date(2026, 1, 1);
        let today = date(2026, 1, 2);
        let requested = date(2026, 1, 10);
        insert_refreshable(
            &client,
            Currency::Usd,
            Currency::Eur,
            requested,
            previous,
            previous,
        );
        insert_refreshable(
            &client,
            Currency::Usd,
            Currency::Eur,
            date(2026, 1, 11),
            previous,
            previous,
        );

        let quote = client
            .rate(Currency::Usd, Currency::Eur, requested, today)
            .await
            .expect("matching future quote is a fallback");

        assert_eq!(quote.requested_date, requested);
        assert_eq!(quote.effective_date, previous);
        assert!(quote.is_stale);
        assert!(quote.is_provisional);
    }

    #[tokio::test]
    async fn failures_never_fall_back_across_rate_contexts() {
        let client = FrankfurterClient::with_base_url("http://127.0.0.1:1");
        let previous = date(2026, 1, 1);
        let today = date(2026, 1, 2);
        insert_refreshable(
            &client,
            Currency::Usd,
            Currency::Eur,
            date(2026, 1, 10),
            previous,
            previous,
        );

        assert!(
            client
                .rate(Currency::Usd, Currency::Eur, today, today)
                .await
                .is_err()
        );
        assert!(
            client
                .rate(Currency::Usd, Currency::Eur, date(2026, 1, 11), today)
                .await
                .is_err()
        );
        assert!(
            client
                .rate(Currency::Usd, Currency::Eur, previous, today)
                .await
                .is_err()
        );
    }

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).expect("valid constant date")
    }

    fn insert_refreshable(
        client: &FrankfurterClient,
        base: Currency,
        quote: Currency,
        requested_date: NaiveDate,
        fetch_date: NaiveDate,
        effective_date: NaiveDate,
    ) {
        client
            .refreshable_cache
            .write()
            .expect("cache lock")
            .insert(
                (base, quote, requested_date, fetch_date),
                RateQuote {
                    base,
                    quote,
                    requested_date,
                    effective_date,
                    rate: Decimal::ONE,
                    is_stale: false,
                    is_provisional: requested_date > fetch_date,
                },
            );
    }

    async fn server_with_responses(
        responses: Vec<(&str, Duration)>,
    ) -> (String, tokio::task::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind test server");
        let address = listener.local_addr().expect("server address");
        let responses = responses
            .into_iter()
            .map(|(body, delay)| (body.to_owned(), delay))
            .collect::<Vec<_>>();
        let server = tokio::spawn(async move {
            for (body, delay) in responses {
                let (mut stream, _) = listener.accept().await.expect("accept test request");
                read_request(&mut stream).await;
                if !delay.is_zero() {
                    tokio::time::sleep(delay).await;
                }
                write_response(&mut stream, &body).await;
            }
        });
        (format!("http://{address}"), server)
    }

    async fn server_with_counter(
        body: &str,
        delay: Duration,
        count: Arc<AtomicUsize>,
        requests: usize,
    ) -> (String, tokio::task::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind test server");
        let address = listener.local_addr().expect("server address");
        let body = body.to_owned();
        let server = tokio::spawn(async move {
            for _ in 0..requests {
                let (mut stream, _) = listener.accept().await.expect("accept test request");
                read_request(&mut stream).await;
                count.fetch_add(1, Ordering::SeqCst);
                if !delay.is_zero() {
                    tokio::time::sleep(delay).await;
                }
                write_response(&mut stream, &body).await;
            }
        });
        (format!("http://{address}"), server)
    }

    async fn server_with_concurrency(
        body: &str,
        delay: Duration,
        count: Arc<AtomicUsize>,
        active: Arc<AtomicUsize>,
        maximum: Arc<AtomicUsize>,
        requests: usize,
    ) -> (String, tokio::task::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind test server");
        let address = listener.local_addr().expect("server address");
        let body = body.to_owned();
        let server = tokio::spawn(async move {
            let mut workers = Vec::new();
            for _ in 0..requests {
                let (mut stream, _) = listener.accept().await.expect("accept test request");
                let body = body.clone();
                let count = count.clone();
                let active = active.clone();
                let maximum = maximum.clone();
                workers.push(tokio::spawn(async move {
                    read_request(&mut stream).await;
                    count.fetch_add(1, Ordering::SeqCst);
                    let current = active.fetch_add(1, Ordering::SeqCst) + 1;
                    maximum.fetch_max(current, Ordering::SeqCst);
                    if !delay.is_zero() {
                        tokio::time::sleep(delay).await;
                    }
                    write_response(&mut stream, &body).await;
                    active.fetch_sub(1, Ordering::SeqCst);
                }));
            }
            for worker in workers {
                worker.await.expect("server worker");
            }
        });
        (format!("http://{address}"), server)
    }

    async fn read_request(stream: &mut tokio::net::TcpStream) {
        let mut buffer = [0_u8; 1024];
        let _ = stream.read(&mut buffer).await.expect("read test request");
    }

    async fn write_response(stream: &mut tokio::net::TcpStream, body: &str) {
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        stream
            .write_all(response.as_bytes())
            .await
            .expect("write test response");
    }
}
