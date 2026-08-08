use std::net::IpAddr;
use std::sync::Arc;

use async_trait::async_trait;

use crate::ApplicationError;

/// Verifies the configured password without exposing a hash to web handlers.
#[async_trait]
pub trait PasswordVerifier: Send + Sync {
    /// Returns whether the submitted password is valid.
    async fn verify(&self, password: &str) -> Result<bool, ApplicationError>;
}

/// Admission result for a login attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoginAdmission {
    /// The attempt may proceed.
    Allowed,
    /// The attempt is blocked until the indicated number of seconds elapses.
    RetryAfter(u64),
}

/// Limits password attempts by resolved client identity.
#[async_trait]
pub trait LoginAttemptLimiter: Send + Sync {
    /// Reserves one password attempt.
    async fn reserve(&self, client: IpAddr) -> LoginAdmission;
    /// Clears attempts after a successful authenticated session is created.
    async fn reset(&self, client: IpAddr);
}

/// Result of one admitted password attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthenticationAttempt {
    /// The password was valid and the caller may establish a session.
    Authenticated,
    /// The password was invalid.
    InvalidPassword,
    /// The client must wait before another attempt.
    RetryAfter(u64),
}

/// Inbound authentication policy.
#[async_trait]
pub trait AuthenticationUseCases: Send + Sync {
    /// Applies login admission and verifies one submitted password.
    async fn attempt(
        &self,
        client: IpAddr,
        password: &str,
    ) -> Result<AuthenticationAttempt, ApplicationError>;
    /// Clears the client's limiter state after durable session establishment.
    async fn complete_login(&self, client: IpAddr);
}

/// Authentication workflow implementation.
pub struct AuthenticationService {
    limiter: Arc<dyn LoginAttemptLimiter>,
    password: Arc<dyn PasswordVerifier>,
}

impl AuthenticationService {
    /// Creates an authentication service with injected policy adapters.
    pub fn new(limiter: Arc<dyn LoginAttemptLimiter>, password: Arc<dyn PasswordVerifier>) -> Self {
        Self { limiter, password }
    }
}

#[async_trait]
impl AuthenticationUseCases for AuthenticationService {
    async fn attempt(
        &self,
        client: IpAddr,
        password: &str,
    ) -> Result<AuthenticationAttempt, ApplicationError> {
        match self.limiter.reserve(client).await {
            LoginAdmission::Allowed => {}
            LoginAdmission::RetryAfter(seconds) => {
                return Ok(AuthenticationAttempt::RetryAfter(seconds));
            }
        }
        if self.password.verify(password).await? {
            Ok(AuthenticationAttempt::Authenticated)
        } else {
            Ok(AuthenticationAttempt::InvalidPassword)
        }
    }

    async fn complete_login(&self, client: IpAddr) {
        self.limiter.reset(client).await;
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;

    struct Password(bool);

    #[async_trait]
    impl PasswordVerifier for Password {
        async fn verify(&self, _: &str) -> Result<bool, ApplicationError> {
            Ok(self.0)
        }
    }

    struct Limiter {
        admission: LoginAdmission,
        resets: AtomicUsize,
    }

    #[async_trait]
    impl LoginAttemptLimiter for Limiter {
        async fn reserve(&self, _: IpAddr) -> LoginAdmission {
            self.admission
        }

        async fn reset(&self, _: IpAddr) {
            self.resets.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[tokio::test]
    async fn resets_only_after_completion() {
        let limiter = Arc::new(Limiter {
            admission: LoginAdmission::Allowed,
            resets: AtomicUsize::new(0),
        });
        let service = AuthenticationService::new(limiter.clone(), Arc::new(Password(true)));
        let client: IpAddr = "192.0.2.25".parse().expect("valid test address");

        assert_eq!(
            service.attempt(client, "secret").await.expect("attempt"),
            AuthenticationAttempt::Authenticated
        );
        assert_eq!(limiter.resets.load(Ordering::SeqCst), 0);
        service.complete_login(client).await;
        assert_eq!(limiter.resets.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn distinguishes_invalid_and_rate_limited_attempts() {
        let client: IpAddr = "192.0.2.25".parse().expect("valid test address");
        let invalid = AuthenticationService::new(
            Arc::new(Limiter {
                admission: LoginAdmission::Allowed,
                resets: AtomicUsize::new(0),
            }),
            Arc::new(Password(false)),
        );
        assert_eq!(
            invalid.attempt(client, "wrong").await.expect("attempt"),
            AuthenticationAttempt::InvalidPassword
        );

        let limited = AuthenticationService::new(
            Arc::new(Limiter {
                admission: LoginAdmission::RetryAfter(17),
                resets: AtomicUsize::new(0),
            }),
            Arc::new(Password(true)),
        );
        assert_eq!(
            limited.attempt(client, "secret").await.expect("attempt"),
            AuthenticationAttempt::RetryAfter(17)
        );
    }
}
