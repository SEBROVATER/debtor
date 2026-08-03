use async_trait::async_trait;
use debtor_application::{LoginAdmission, LoginAttemptLimiter};
use std::collections::{HashMap, VecDeque};
use std::net::IpAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Process-local rolling login attempt limiter.
pub struct MemoryLoginAttemptLimiter {
    clock: Arc<dyn MonotonicClock>,
    attempts: Mutex<HashMap<IpAddr, VecDeque<Duration>>>,
}

impl Default for MemoryLoginAttemptLimiter {
    fn default() -> Self {
        Self {
            clock: Arc::new(ProcessMonotonicClock::default()),
            attempts: Mutex::new(HashMap::new()),
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
        Self {
            clock,
            attempts: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl LoginAttemptLimiter for MemoryLoginAttemptLimiter {
    async fn reserve(&self, client: IpAddr) -> LoginAdmission {
        const WINDOW: Duration = Duration::from_secs(300);
        let now = self.clock.elapsed();
        let Ok(mut attempts) = self.attempts.lock() else {
            return LoginAdmission::RetryAfter(WINDOW.as_secs());
        };
        attempts.retain(|_, values| {
            while values
                .front()
                .is_some_and(|time| now.saturating_sub(*time) >= WINDOW)
            {
                values.pop_front();
            }
            !values.is_empty()
        });
        let values = attempts.entry(client).or_default();
        if values.len() >= 5 {
            let retry = values.front().map_or(1, |time| {
                ceil_seconds(WINDOW.saturating_sub(now.saturating_sub(*time)))
            });
            LoginAdmission::RetryAfter(retry)
        } else {
            values.push_back(now);
            LoginAdmission::Allowed
        }
    }

    async fn reset(&self, client: IpAddr) {
        if let Ok(mut attempts) = self.attempts.lock() {
            attempts.remove(&client);
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
    async fn prunes_expired_clients_on_subsequent_reservations() {
        let clock = Arc::new(TestClock::default());
        let limiter = MemoryLoginAttemptLimiter::with_clock(clock.clone());
        let expired: IpAddr = "192.0.2.25".parse().expect("valid test IP");
        let current: IpAddr = "192.0.2.26".parse().expect("valid test IP");

        assert_eq!(limiter.reserve(expired).await, LoginAdmission::Allowed);
        clock.set(Duration::from_secs(300));
        assert_eq!(limiter.reserve(current).await, LoginAdmission::Allowed);

        let attempts = limiter.attempts.lock().expect("limiter lock");
        assert!(!attempts.contains_key(&expired));
        assert!(attempts.contains_key(&current));
    }
}
