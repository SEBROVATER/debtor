use std::collections::{BTreeMap, HashMap};
use std::fmt;
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

const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const TOTAL_TIMEOUT: Duration = Duration::from_secs(20);
const MAX_RESPONSE_BYTES: usize = 64 * 1024;
const MAX_IN_FLIGHT_REQUESTS: usize = 4;
const STABLE_CACHE_CAPACITY: usize = 4_096;
type CacheKey = (Currency, Currency, NaiveDate, NaiveDate);
type FlightResult = Result<RateQuote, UnavailableReason>;

/// Safe startup failure for local exchange-provider configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrankfurterConfigurationError {
    /// The configured endpoint is not an acceptable HTTP(S) base URL.
    InvalidBaseUrl,
    /// The HTTP client could not be initialized locally.
    ClientConstruction,
}

impl fmt::Display for FrankfurterConfigurationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("invalid exchange-rate provider configuration")
    }
}

impl std::error::Error for FrankfurterConfigurationError {}

struct StableCache {
    values: HashMap<CacheKey, RateQuote>,
    access_order: BTreeMap<u64, CacheKey>,
    next_access: u64,
    evictions: u64,
}

impl StableCache {
    fn new() -> Self {
        Self {
            values: HashMap::new(),
            access_order: BTreeMap::new(),
            next_access: 0,
            evictions: 0,
        }
    }

    fn next_sequence(&mut self) -> u64 {
        if self.next_access == u64::MAX {
            self.access_order.clear();
            let keys = self.values.keys().copied().collect::<Vec<_>>();
            for (sequence, key) in keys.into_iter().enumerate() {
                self.access_order.insert(sequence as u64, key);
            }
            self.next_access = self.values.len() as u64;
        }
        let sequence = self.next_access;
        self.next_access += 1;
        sequence
    }

    fn touch(&mut self, key: CacheKey, value: RateQuote) {
        self.access_order.retain(|_, candidate| *candidate != key);
        let sequence = self.next_sequence();
        self.values.insert(key, value);
        self.access_order.insert(sequence, key);
        while self.values.len() > STABLE_CACHE_CAPACITY {
            let Some((sequence, oldest)) = self.access_order.pop_first() else {
                break;
            };
            self.values.remove(&oldest);
            self.evictions = self.evictions.saturating_add(1);
            tracing::debug!(
                target: "debtor.provider",
                event = "provider_cache_eviction",
                category = "stable_lru",
                count = 1_u64,
                sequence,
            );
        }
    }

    fn get(&mut self, key: CacheKey) -> Option<RateQuote> {
        let value = self.values.get(&key).cloned()?;
        self.touch(key, value.clone());
        Some(value)
    }
}

struct RefreshableCache {
    values: HashMap<CacheKey, RateQuote>,
    access_order: BTreeMap<u64, CacheKey>,
    next_access: u64,
    last_rollover: Option<NaiveDate>,
    evictions: u64,
}

impl RefreshableCache {
    fn new() -> Self {
        Self {
            values: HashMap::new(),
            access_order: BTreeMap::new(),
            next_access: 0,
            last_rollover: None,
            evictions: 0,
        }
    }

    fn prune_on_rollover(&mut self, today: NaiveDate) {
        if self.last_rollover == Some(today) {
            return;
        }
        self.last_rollover = Some(today);

        let mut newest_fallbacks = HashMap::new();
        for &key @ (base, quote, requested, fetch) in self.values.keys() {
            if fetch >= today {
                continue;
            }
            let context = if requested == fetch {
                (base, quote, None)
            } else {
                (base, quote, Some(requested))
            };
            newest_fallbacks
                .entry(context)
                .and_modify(|candidate: &mut CacheKey| {
                    if candidate.3 < fetch {
                        *candidate = key;
                    }
                })
                .or_insert(key);
        }
        let before = self.values.len();
        self.values.retain(|key, _| {
            // A former future request becomes an ordinary historical context.
            !(key.2 > key.3 && key.2 < today)
                && (key.3 >= today || newest_fallbacks.values().any(|candidate| candidate == key))
        });
        self.access_order
            .retain(|_, key| self.values.contains_key(key));
        self.evictions = self.evictions.saturating_add(
            u64::try_from(before - self.values.len()).map_or(u64::MAX, |count| count),
        );
    }

    fn next_sequence(&mut self) -> u64 {
        if self.next_access == u64::MAX {
            self.access_order.clear();
            for (sequence, key) in self.values.keys().copied().enumerate() {
                self.access_order.insert(sequence as u64, key);
            }
            self.next_access = self.values.len() as u64;
        }
        let sequence = self.next_access;
        self.next_access += 1;
        sequence
    }

    fn touch(&mut self, key: CacheKey, value: RateQuote) {
        self.access_order.retain(|_, candidate| *candidate != key);
        let sequence = self.next_sequence();
        self.values.insert(key, value);
        self.access_order.insert(sequence, key);
        while self.values.len() > STABLE_CACHE_CAPACITY {
            let Some((_, oldest)) = self.access_order.pop_first() else {
                break;
            };
            self.values.remove(&oldest);
            self.evictions = self.evictions.saturating_add(1);
        }
    }

    fn get(&mut self, key: &CacheKey) -> Option<RateQuote> {
        let value = self.values.get(key).cloned()?;
        self.touch(*key, value.clone());
        Some(value)
    }
}

/// Key-free, process-local exchange-rate cache counters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CacheCounters {
    /// Number of stable historical contexts retained.
    pub stable_entries: usize,
    /// Number of refreshable current/future contexts retained.
    pub refreshable_entries: usize,
    /// Number of stable contexts evicted by LRU capacity.
    pub stable_evictions: u64,
    /// Number of obsolete refreshable contexts pruned at UTC rollover.
    pub refreshable_evictions: u64,
}

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
    http: reqwest::Client,
    base_url: reqwest::Url,
    stable_cache: Arc<RwLock<StableCache>>,
    refreshable_cache: Arc<RwLock<RefreshableCache>>,
    in_flight: Arc<Mutex<HashMap<CacheKey, Arc<Flight>>>>,
    requests: Arc<Semaphore>,
}

impl FrankfurterClient {
    /// Creates a client using a custom endpoint, intended for local tests.
    #[cfg(test)]
    #[allow(clippy::expect_used, clippy::missing_panics_doc)]
    pub fn with_base_url(base_url: &str) -> Self {
        Self::try_with_base_url(base_url).expect("the test/default provider URL is valid")
    }

    /// Validates and constructs a client without making a provider request.
    ///
    /// # Errors
    ///
    /// Returns a safe category when the URL is invalid or the HTTP client cannot initialize.
    pub fn try_with_base_url(base_url: &str) -> Result<Self, FrankfurterConfigurationError> {
        let mut base_url = reqwest::Url::parse(base_url)
            .map_err(|_| FrankfurterConfigurationError::InvalidBaseUrl)?;
        if !matches!(base_url.scheme(), "http" | "https")
            || !base_url.username().is_empty()
            || base_url.password().is_some()
            || base_url.query().is_some()
            || base_url.fragment().is_some()
        {
            return Err(FrankfurterConfigurationError::InvalidBaseUrl);
        }
        if !base_url.path().ends_with('/') {
            let path = format!("{}/", base_url.path());
            base_url.set_path(&path);
        }
        let http = reqwest::Client::builder()
            .connect_timeout(CONNECT_TIMEOUT)
            .timeout(TOTAL_TIMEOUT)
            .read_timeout(TOTAL_TIMEOUT)
            .build()
            .map_err(|_| FrankfurterConfigurationError::ClientConstruction)?;
        Ok(Self {
            http,
            base_url,
            stable_cache: Arc::new(RwLock::new(StableCache::new())),
            refreshable_cache: Arc::new(RwLock::new(RefreshableCache::new())),
            in_flight: Arc::new(Mutex::new(HashMap::new())),
            requests: Arc::new(Semaphore::new(MAX_IN_FLIGHT_REQUESTS)),
        })
    }

    #[cfg(test)]
    fn with_http_client(base_url: &str, http: reqwest::Client) -> Self {
        let mut client = Self::with_base_url(base_url);
        client.http = http;
        client
    }

    /// Returns aggregate cache counters without exposing rate contexts.
    ///
    /// # Errors
    ///
    /// Returns an unavailable error if a cache lock is poisoned.
    pub fn cache_counters(&self) -> Result<CacheCounters, ApplicationError> {
        let stable = self
            .stable_cache
            .read()
            .map_err(|_| ApplicationError::Unavailable(UnavailableReason::ExchangeRates))?;
        let refreshable = self
            .refreshable_cache
            .read()
            .map_err(|_| ApplicationError::Unavailable(UnavailableReason::ExchangeRates))?;
        Ok(CacheCounters {
            stable_entries: stable.values.len(),
            refreshable_entries: refreshable.values.len(),
            stable_evictions: stable.evictions,
            refreshable_evictions: refreshable.evictions,
        })
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
                fetch_date,
                effective_date: fetch_date,
                rate: Decimal::ONE,
                is_stale: false,
                is_provisional: original_requested_date > today,
            });
        }
        let key = (base, quote, original_requested_date, fetch_date);
        let cached = if original_requested_date < today {
            self.stable_cache
                .write()
                .map_err(|_| ApplicationError::Unavailable(UnavailableReason::ExchangeRates))?
                .get(key)
        } else {
            let mut cache = self
                .refreshable_cache
                .write()
                .map_err(|_| ApplicationError::Unavailable(UnavailableReason::ExchangeRates))?;
            cache.prune_on_rollover(today);
            cache.get(&key)
        };
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
        let mut url = self
            .base_url
            .join(&format!("rate/{}/{}", base.code(), quote.code()))
            .map_err(|_| UnavailableReason::ExchangeRates)?;
        url.query_pairs_mut()
            .append_pair("date", &fetch_date.to_string());
        let mut response = match self.http.get(url).send().await {
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
            fetch_date,
            effective_date: payload.date,
            rate: payload.rate,
            is_stale: false,
            is_provisional: original_requested_date > today,
        };
        if original_requested_date < today {
            self.stable_cache
                .write()
                .map_err(|_| UnavailableReason::ExchangeRates)?
                .touch(key, value.clone());
        } else {
            let mut cache = self
                .refreshable_cache
                .write()
                .map_err(|_| UnavailableReason::ExchangeRates)?;
            cache.prune_on_rollover(today);
            cache.touch(key, value.clone());
        }
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
            let stale = self.stable_cache.write().ok().and_then(|mut cache| {
                cache.get(key).map(|mut value| {
                    value.is_stale = true;
                    value
                })
            });
            if let Some(value) = stale {
                return Ok(value);
            }
            tracing::warn!(
                target: "debtor.provider",
                event = "provider_fallback",
                category = "historical_unavailable",
            );
            return Err(UnavailableReason::ExchangeRates);
        }
        let stale = self.refreshable_cache.write().ok().and_then(|mut cache| {
            let candidate = cache
                .values
                .iter()
                .filter(
                    |((cached_base, cached_quote, cached_requested, cached_fetch), _)| {
                        *cached_base == base
                            && *cached_quote == quote
                            && *cached_fetch < fetch_date
                            && *cached_fetch + chrono::Duration::days(7) >= today
                            && if requested_date == today {
                                *cached_requested == *cached_fetch
                            } else {
                                *cached_requested == requested_date
                            }
                    },
                )
                .max_by_key(|((_, _, _, cached_fetch), _)| *cached_fetch)
                .map(|(key, value)| (*key, value.clone()));
            candidate.map(|(key, value)| {
                cache.touch(key, value.clone());
                value
            })
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

    #[test]
    fn provider_base_url_rejects_unsafe_components_but_allows_local_http() {
        for url in [
            "ftp://provider.example/v2",
            "https://user:password@provider.example/v2",
            "https://provider.example/v2?token=sentinel",
            "https://provider.example/v2#fragment",
        ] {
            assert!(
                matches!(
                    FrankfurterClient::try_with_base_url(url),
                    Err(FrankfurterConfigurationError::InvalidBaseUrl)
                ),
                "{url}"
            );
        }
        assert!(FrankfurterClient::try_with_base_url("http://127.0.0.1:3000/v2").is_ok());
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

    #[tokio::test]
    async fn fixed_past_failure_uses_only_the_exact_stable_context() {
        let client = FrankfurterClient::with_base_url("http://127.0.0.1:1");
        let requested = date(2026, 1, 1);
        let fetch = requested;
        let wrong_requested = date(2026, 1, 2);
        client.stable_cache.write().expect("stable cache").touch(
            (Currency::Usd, Currency::Eur, requested, fetch),
            RateQuote {
                base: Currency::Usd,
                quote: Currency::Eur,
                requested_date: requested,
                fetch_date: fetch,
                effective_date: fetch,
                rate: Decimal::ONE,
                is_stale: false,
                is_provisional: false,
            },
        );
        client.stable_cache.write().expect("stable cache").touch(
            (
                Currency::Usd,
                Currency::Eur,
                wrong_requested,
                wrong_requested,
            ),
            quote((
                Currency::Usd,
                Currency::Eur,
                wrong_requested,
                wrong_requested,
            )),
        );

        let value = client
            .stale_or_error(
                (Currency::Usd, Currency::Eur, requested, fetch),
                date(2026, 2, 1),
            )
            .expect("exact stable fallback");

        assert_eq!(value.requested_date, requested);
        assert_eq!(value.effective_date, fetch);
        assert!(value.is_stale);
    }

    #[tokio::test]
    async fn refreshable_fallback_is_inclusive_for_seven_days_and_rejects_day_eight() {
        let client = FrankfurterClient::with_base_url("http://127.0.0.1:1");
        let prior = date(2026, 1, 1);
        let requested = date(2026, 1, 1);
        insert_refreshable(
            &client,
            Currency::Usd,
            Currency::Eur,
            requested,
            prior,
            prior,
        );

        let eligible = client
            .rate(
                Currency::Usd,
                Currency::Eur,
                date(2026, 1, 8),
                date(2026, 1, 8),
            )
            .await
            .expect("seventh UTC day remains eligible");
        assert!(eligible.is_stale);
        assert!(!eligible.is_provisional);

        assert!(
            client
                .rate(
                    Currency::Usd,
                    Currency::Eur,
                    date(2026, 1, 9),
                    date(2026, 1, 9)
                )
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn future_fallback_requires_the_same_requested_date_and_preserves_fetch_date() {
        let client = FrankfurterClient::with_base_url("http://127.0.0.1:1");
        let prior_fetch = date(2026, 1, 3);
        let requested = date(2026, 1, 12);
        insert_refreshable(
            &client,
            Currency::Usd,
            Currency::Eur,
            requested,
            prior_fetch,
            prior_fetch,
        );

        let value = client
            .rate(Currency::Usd, Currency::Eur, requested, date(2026, 1, 10))
            .await
            .expect("matching future fallback");
        assert_eq!(value.requested_date, requested);
        assert_eq!(value.fetch_date, prior_fetch);
        assert!(value.is_stale);
        assert!(value.is_provisional);

        assert!(
            client
                .rate(
                    Currency::Usd,
                    Currency::Eur,
                    date(2026, 1, 13),
                    date(2026, 1, 10)
                )
                .await
                .is_err()
        );
    }

    #[test]
    fn stable_cache_evicts_oldest_context_after_refreshing_a_hot_entry() {
        let mut cache = StableCache::new();
        for index in 0..STABLE_CACHE_CAPACITY {
            let requested_date =
                date(2025, 1, 1) + chrono::Duration::days(i64::try_from(index).unwrap());
            let key = (Currency::Usd, Currency::Eur, requested_date, requested_date);
            cache.touch(key, quote(key));
        }
        let hot_date = date(2025, 1, 1) + chrono::Duration::days(100);
        let hot_key = (Currency::Usd, Currency::Eur, hot_date, hot_date);
        assert!(cache.get(hot_key).is_some());

        let extra_date = date(2040, 1, 1);
        let extra_key = (Currency::Usd, Currency::Eur, extra_date, extra_date);
        cache.touch(extra_key, quote(extra_key));

        let oldest_date = date(2025, 1, 1);
        let oldest_key = (Currency::Usd, Currency::Eur, oldest_date, oldest_date);
        assert!(cache.get(oldest_key).is_none());
        assert!(cache.get(hot_key).is_some());
        assert_eq!(cache.values.len(), STABLE_CACHE_CAPACITY);
        assert_eq!(cache.evictions, 1);
    }

    #[test]
    fn refreshable_cache_prunes_superseded_rollover_contexts_but_keeps_fallbacks() {
        let mut cache = RefreshableCache::new();
        let first = date(2026, 1, 1);
        let second = date(2026, 1, 2);
        let today = date(2026, 1, 3);
        let future = date(2026, 1, 10);
        let other_future = date(2026, 1, 11);
        let keys = [
            (Currency::Usd, Currency::Eur, first, first),
            (Currency::Usd, Currency::Eur, second, second),
            (Currency::Usd, Currency::Eur, future, first),
            (Currency::Usd, Currency::Eur, future, second),
            (Currency::Usd, Currency::Eur, other_future, first),
            (Currency::Usd, Currency::Eur, today, today),
        ];
        for key in keys {
            cache.touch(key, quote(key));
        }

        cache.prune_on_rollover(today);

        assert_eq!(cache.values.len(), 4);
        assert_eq!(cache.evictions, 2);
        assert!(
            cache
                .get(&(Currency::Usd, Currency::Eur, first, first))
                .is_none()
        );
        assert!(
            cache
                .get(&(Currency::Usd, Currency::Eur, second, second))
                .is_some()
        );
        assert!(
            cache
                .get(&(Currency::Usd, Currency::Eur, future, first))
                .is_none()
        );
        assert!(
            cache
                .get(&(Currency::Usd, Currency::Eur, future, second))
                .is_some()
        );
        assert!(
            cache
                .get(&(Currency::Usd, Currency::Eur, other_future, first))
                .is_some()
        );
        assert!(
            cache
                .get(&(Currency::Usd, Currency::Eur, today, today))
                .is_some()
        );
    }

    #[test]
    fn cache_counters_expose_only_aggregate_sizes_and_evictions() {
        let client = FrankfurterClient::with_base_url("http://127.0.0.1:1");
        let mut cache = client.stable_cache.write().expect("cache lock");
        for index in 0..=STABLE_CACHE_CAPACITY {
            let requested_date =
                date(2025, 1, 1) + chrono::Duration::days(i64::try_from(index).unwrap());
            let key = (Currency::Usd, Currency::Eur, requested_date, requested_date);
            cache.touch(key, quote(key));
        }
        drop(cache);

        assert_eq!(
            client.cache_counters().expect("cache counters"),
            CacheCounters {
                stable_entries: STABLE_CACHE_CAPACITY,
                refreshable_entries: 0,
                stable_evictions: 1,
                refreshable_evictions: 0,
            }
        );
    }

    fn quote(key: CacheKey) -> RateQuote {
        let (base, quote, requested_date, effective_date) = key;
        RateQuote {
            base,
            quote,
            requested_date,
            fetch_date: effective_date,
            effective_date,
            rate: Decimal::ONE,
            is_stale: false,
            is_provisional: false,
        }
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
        client.refreshable_cache.write().expect("cache lock").touch(
            (base, quote, requested_date, fetch_date),
            RateQuote {
                base,
                quote,
                requested_date,
                fetch_date,
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
