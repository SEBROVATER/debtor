use async_trait::async_trait;
use debtor_application::{LoginAdmission, LoginAttemptLimiter};
use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};
use std::net::IpAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const WINDOW: Duration = Duration::from_secs(300);
const MAX_CLIENTS: usize = 4_096;

/// Process-local rolling login attempt limiter.
pub struct MemoryLoginAttemptLimiter {
    clock: Arc<dyn MonotonicClock>,
    state: Mutex<LimiterState>,
}

struct LimiterState {
    attempts: HashMap<IpAddr, VecDeque<Duration>>,
    expirations: BTreeMap<Duration, BTreeSet<IpAddr>>,
    capacity: usize,
}

impl LimiterState {
    fn new(capacity: usize) -> Self {
        Self {
            attempts: HashMap::new(),
            expirations: BTreeMap::new(),
            capacity,
        }
    }

    fn prune_expired(&mut self, now: Duration) {
        loop {
            let Some((&expires, _)) = self.expirations.first_key_value() else {
                return;
            };
            if expires > now {
                return;
            }
            let clients = self.expirations.remove(&expires).unwrap_or_default();
            for client in clients {
                let next_expiry = if let Some(values) = self.attempts.get_mut(&client) {
                    while values
                        .front()
                        .is_some_and(|time| now.saturating_sub(*time) >= WINDOW)
                    {
                        values.pop_front();
                    }
                    values.front().map(|time| time.saturating_add(WINDOW))
                } else {
                    None
                };
                if let Some(next_expiry) = next_expiry {
                    self.expirations
                        .entry(next_expiry)
                        .or_default()
                        .insert(client);
                } else {
                    self.attempts.remove(&client);
                }
            }
        }
    }

    fn remove_expiry(&mut self, client: IpAddr, expires: Duration) {
        if let Some(clients) = self.expirations.get_mut(&expires) {
            clients.remove(&client);
            if clients.is_empty() {
                self.expirations.remove(&expires);
            }
        }
    }
}

impl Default for MemoryLoginAttemptLimiter {
    fn default() -> Self {
        Self {
            clock: Arc::new(ProcessMonotonicClock::default()),
            state: Mutex::new(LimiterState::new(MAX_CLIENTS)),
        }
    }
}

/// Private, monotonic source used to make rate-limit timing deterministic.
trait MonotonicClock: Send + Sync {
    fn elapsed(&self) -> Duration;
}

struct ProcessMonotonicClock {
    started: Instant,
}

impl Default for ProcessMonotonicClock {
    fn default() -> Self {
        Self {
            started: Instant::now(),
        }
    }
}

impl MonotonicClock for ProcessMonotonicClock {
    fn elapsed(&self) -> Duration {
        self.started.elapsed()
    }
}

impl MemoryLoginAttemptLimiter {
    #[cfg(test)]
    fn with_clock(clock: Arc<dyn MonotonicClock>) -> Self {
        Self::with_clock_and_capacity(clock, MAX_CLIENTS)
    }

    #[cfg(test)]
    fn with_clock_and_capacity(clock: Arc<dyn MonotonicClock>, capacity: usize) -> Self {
        Self {
            clock,
            state: Mutex::new(LimiterState::new(capacity)),
        }
    }
}

#[async_trait]
impl LoginAttemptLimiter for MemoryLoginAttemptLimiter {
    async fn reserve(&self, client: IpAddr) -> LoginAdmission {
        let now = self.clock.elapsed();
        let Ok(mut state) = self.state.lock() else {
            return LoginAdmission::RetryAfter(WINDOW.as_secs());
        };
        state.prune_expired(now);
        if let Some(values) = state.attempts.get_mut(&client) {
            if values.len() >= 5 {
                let retry = values.front().map_or(1, |time| {
                    ceil_seconds(WINDOW.saturating_sub(now.saturating_sub(*time)))
                });
                return LoginAdmission::RetryAfter(retry);
            }
            values.push_back(now);
            return LoginAdmission::Allowed;
        }
        if state.attempts.len() >= state.capacity {
            let retry = state
                .expirations
                .first_key_value()
                .map_or(WINDOW.as_secs(), |(expires, _)| {
                    ceil_seconds(expires.saturating_sub(now))
                });
            return LoginAdmission::RetryAfter(retry.max(1));
        }
        state.attempts.insert(client, VecDeque::from([now]));
        state
            .expirations
            .entry(now.saturating_add(WINDOW))
            .or_default()
            .insert(client);
        LoginAdmission::Allowed
    }

    async fn reset(&self, client: IpAddr) {
        if let Ok(mut state) = self.state.lock() {
            if let Some(values) = state.attempts.remove(&client) {
                if let Some(first) = values.front() {
                    state.remove_expiry(client, first.saturating_add(WINDOW));
                }
            }
        }
    }
}

fn ceil_seconds(duration: Duration) -> u64 {
    duration
        .as_secs()
        .saturating_add(u64::from(duration.subsec_nanos() != 0))
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use debtor_application::{LoginAdmission, LoginAttemptLimiter};
    use std::{
        net::IpAddr,
        sync::{Arc, Mutex},
        time::Duration,
    };
    use tokio::sync::Barrier;

    use super::{MemoryLoginAttemptLimiter, MonotonicClock};

    #[derive(Default)]
    struct TestClock(Mutex<Duration>);

    impl TestClock {
        fn set(&self, elapsed: Duration) {
            *self.0.lock().expect("test clock lock") = elapsed;
        }
    }

    impl MonotonicClock for TestClock {
        fn elapsed(&self) -> Duration {
            *self.0.lock().expect("test clock lock")
        }
    }

    #[tokio::test]
    async fn enforces_window_boundaries_and_exact_retry_ceiling() {
        let clock = Arc::new(TestClock::default());
        let limiter = MemoryLoginAttemptLimiter::with_clock(clock.clone());
        let client: IpAddr = "192.0.2.25".parse().expect("valid test IP");

        for _ in 0..5 {
            assert_eq!(limiter.reserve(client).await, LoginAdmission::Allowed);
        }
        assert_eq!(
            limiter.reserve(client).await,
            LoginAdmission::RetryAfter(300)
        );
        clock.set(Duration::from_nanos(1));
        assert_eq!(
            limiter.reserve(client).await,
            LoginAdmission::RetryAfter(300)
        );
        clock.set(Duration::from_secs(1));
        assert_eq!(
            limiter.reserve(client).await,
            LoginAdmission::RetryAfter(299)
        );
        clock.set(Duration::from_secs(300));
        assert_eq!(limiter.reserve(client).await, LoginAdmission::Allowed);
        limiter.reset(client).await;
        assert_eq!(limiter.reserve(client).await, LoginAdmission::Allowed);
    }

    #[tokio::test]
    async fn tracks_attempts_independently_per_client() {
        let limiter = MemoryLoginAttemptLimiter::default();
        let first: IpAddr = "192.0.2.25".parse().expect("valid test IP");
        let second: IpAddr = "192.0.2.26".parse().expect("valid test IP");
        for _ in 0..5 {
            assert_eq!(limiter.reserve(first).await, LoginAdmission::Allowed);
        }
        assert_eq!(limiter.reserve(second).await, LoginAdmission::Allowed);
    }

    #[tokio::test]
    async fn admits_only_five_concurrent_reservations() {
        let limiter = Arc::new(MemoryLoginAttemptLimiter::default());
        let barrier = Arc::new(Barrier::new(11));
        let client: IpAddr = "192.0.2.25".parse().expect("valid test IP");
        let mut tasks = Vec::new();
        for _ in 0..10 {
            let limiter = limiter.clone();
            let barrier = barrier.clone();
            tasks.push(tokio::spawn(async move {
                barrier.wait().await;
                limiter.reserve(client).await
            }));
        }
        barrier.wait().await;
        let mut allowed = 0;
        for task in tasks {
            if task.await.expect("reservation task") == LoginAdmission::Allowed {
                allowed += 1;
            }
        }
        assert_eq!(allowed, 5);
    }

    #[tokio::test]
    async fn fails_closed_at_capacity_and_recovers_at_earliest_expiry() {
        let clock = Arc::new(TestClock::default());
        let limiter = MemoryLoginAttemptLimiter::with_clock_and_capacity(clock.clone(), 2);
        let first: IpAddr = "192.0.2.25".parse().expect("valid test IP");
        let second: IpAddr = "192.0.2.26".parse().expect("valid test IP");
        let unseen: IpAddr = "192.0.2.27".parse().expect("valid test IP");
        assert_eq!(limiter.reserve(first).await, LoginAdmission::Allowed);
        clock.set(Duration::from_secs(1));
        assert_eq!(limiter.reserve(second).await, LoginAdmission::Allowed);
        assert_eq!(
            limiter.reserve(unseen).await,
            LoginAdmission::RetryAfter(299)
        );
        assert_eq!(limiter.reserve(first).await, LoginAdmission::Allowed);
        clock.set(Duration::from_secs(300));
        assert_eq!(limiter.reserve(unseen).await, LoginAdmission::RetryAfter(1));
        clock.set(Duration::from_secs(301));
        assert_eq!(limiter.reserve(unseen).await, LoginAdmission::Allowed);
    }

    #[tokio::test]
    async fn existing_key_remains_admitted_when_capacity_is_full() {
        let clock = Arc::new(TestClock::default());
        let limiter = MemoryLoginAttemptLimiter::with_clock_and_capacity(clock, 1);
        let existing: IpAddr = "192.0.2.25".parse().expect("valid test IP");
        let unseen: IpAddr = "192.0.2.26".parse().expect("valid test IP");
        assert_eq!(limiter.reserve(existing).await, LoginAdmission::Allowed);
        assert_eq!(limiter.reserve(existing).await, LoginAdmission::Allowed);
        assert_eq!(
            limiter.reserve(unseen).await,
            LoginAdmission::RetryAfter(300)
        );
        limiter.reset(existing).await;
        assert_eq!(limiter.reserve(unseen).await, LoginAdmission::Allowed);
        let state = limiter.state.lock().expect("limiter state");
        assert_eq!(state.attempts.len(), 1);
        assert_eq!(state.expirations.len(), 1);
    }

    #[tokio::test]
    async fn prunes_expired_clients_without_scanning_unrelated_keys() {
        let clock = Arc::new(TestClock::default());
        let limiter = MemoryLoginAttemptLimiter::with_clock_and_capacity(clock.clone(), 2);
        let expired: IpAddr = "192.0.2.25".parse().expect("valid test IP");
        let current: IpAddr = "192.0.2.26".parse().expect("valid test IP");
        assert_eq!(limiter.reserve(expired).await, LoginAdmission::Allowed);
        clock.set(Duration::from_secs(300));
        assert_eq!(limiter.reserve(current).await, LoginAdmission::Allowed);
        let state = limiter.state.lock().expect("limiter state");
        assert!(!state.attempts.contains_key(&expired));
        assert!(state.attempts.contains_key(&current));
        assert_eq!(state.expirations.len(), 1);
    }
}
