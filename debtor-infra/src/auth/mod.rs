//! Password verification using Argon2id.

use argon2::{Argon2, PasswordHash, PasswordVerifier as ArgonPasswordVerifier};
use async_trait::async_trait;
use debtor_application::{ApplicationError, PasswordVerifier};

/// Password verifier backed by one configured Argon2id hash.
pub struct ArgonPasswordGate {
    hash: String,
}

impl ArgonPasswordGate {
    /// Parses and validates a configured password hash.
    ///
    /// # Errors
    ///
    /// Returns an error when the configured PHC string is malformed.
    pub fn new(hash: String) -> Result<Self, ApplicationError> {
        PasswordHash::new(&hash).map_err(|error| {
            ApplicationError::Storage(format!("invalid password hash: {error}"))
        })?;
        Ok(Self { hash })
    }
}

#[async_trait]
impl PasswordVerifier for ArgonPasswordGate {
    async fn verify(&self, password: &str) -> Result<bool, ApplicationError> {
        let hash = PasswordHash::new(&self.hash).map_err(|error| {
            ApplicationError::Storage(format!("invalid password hash: {error}"))
        })?;
        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &hash)
            .is_ok())
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use argon2::{Argon2, PasswordHasher, password_hash::SaltString};
    use debtor_application::PasswordVerifier;

    use super::ArgonPasswordGate;

    #[tokio::test]
    async fn verifies_the_configured_password() {
        let salt = SaltString::encode_b64(b"debtor-local-test").expect("valid test salt");
        let hash = Argon2::default()
            .hash_password(b"correct horse battery staple", &salt)
            .expect("hash generation")
            .to_string();
        let gate = ArgonPasswordGate::new(hash).expect("valid hash");
        assert!(
            gate.verify("correct horse battery staple")
                .await
                .expect("verification")
        );
        assert!(!gate.verify("wrong password").await.expect("verification"));
    }
}
