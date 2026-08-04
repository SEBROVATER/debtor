use argon2::{Argon2, PasswordHash, PasswordVerifier as ArgonPasswordVerifier};
use async_trait::async_trait;
use debtor_application::{
    ApplicationError, ConfigurationError, PasswordVerifier, UnavailableReason,
};
use std::sync::OnceLock;
use tokio::sync::Semaphore;

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
        let parsed = PasswordHash::new(&hash).map_err(|_| {
            ApplicationError::Configuration(ConfigurationError::InvalidPasswordHash)
        })?;
        let salt_length = parsed.salt.and_then(decoded_salt_length);
        let hash_length = parsed.hash.map(|output| output.as_bytes().len());
        let memory_cost = parsed.params.get("m").and_then(parse_parameter);
        let time_cost = parsed.params.get("t").and_then(parse_parameter);
        let parallelism = parsed.params.get("p").and_then(parse_parameter);
        if parsed.algorithm.to_string() != "argon2id"
            || parsed.version.map(|version| version.to_string()) != Some("19".into())
            || !matches!(salt_length, Some(16..=64))
            || !matches!(hash_length, Some(32..=64))
            || !matches!(memory_cost, Some(19_456..=65_536))
            || !matches!(time_cost, Some(2..=5))
            || !matches!(parallelism, Some(1..=4))
        {
            return Err(ApplicationError::Configuration(
                ConfigurationError::InvalidPasswordHash,
            ));
        }
        Ok(Self { hash })
    }
}

fn decoded_salt_length(salt: argon2::password_hash::Salt<'_>) -> Option<usize> {
    let mut bytes = [0_u8; 64];
    salt.decode_b64(&mut bytes).ok().map(<[u8]>::len)
}

fn parse_parameter(value: argon2::password_hash::Value<'_>) -> Option<u32> {
    value.as_str().parse().ok()
}

#[async_trait]
impl PasswordVerifier for ArgonPasswordGate {
    async fn verify(&self, password: &str) -> Result<bool, ApplicationError> {
        let hash = self.hash.clone();
        let password = password.to_owned();
        let permit = semaphore()
            .clone()
            .acquire_owned()
            .await
            .map_err(|_| ApplicationError::Unavailable(UnavailableReason::Authentication))?;
        tokio::task::spawn_blocking(move || {
            let _permit = permit;
            let hash = PasswordHash::new(&hash).map_err(|_| {
                ApplicationError::Configuration(ConfigurationError::InvalidPasswordHash)
            })?;
            Ok(Argon2::default()
                .verify_password(password.as_bytes(), &hash)
                .is_ok())
        })
        .await
        .map_err(|_| ApplicationError::Unavailable(UnavailableReason::Authentication))?
    }
}

fn semaphore() -> &'static std::sync::Arc<Semaphore> {
    static LIMIT: OnceLock<std::sync::Arc<Semaphore>> = OnceLock::new();
    LIMIT.get_or_init(|| std::sync::Arc::new(Semaphore::new(2)))
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use argon2::{Argon2, PasswordHasher, password_hash::SaltString};
    use debtor_application::{ApplicationError, ConfigurationError, PasswordVerifier};

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

    #[test]
    fn accepts_each_argon2_policy_boundary() {
        for (memory, iterations, parallelism) in [(19_456, 2, 1), (65_536, 5, 4)] {
            for (salt_length, output_length) in [(16, 32), (48, 64)] {
                assert!(
                    ArgonPasswordGate::new(test_hash(
                        memory,
                        iterations,
                        parallelism,
                        salt_length,
                        output_length,
                    ))
                    .is_ok()
                );
            }
        }
    }

    #[test]
    fn rejects_argon2_policy_values_outside_each_bound() {
        for (memory, iterations, parallelism, salt_length, output_length) in [
            (19_455, 2, 1, 16, 32),
            (65_537, 2, 1, 16, 32),
            (19_456, 1, 1, 16, 32),
            (19_456, 6, 1, 16, 32),
            (19_456, 2, 0, 16, 32),
            (19_456, 2, 5, 16, 32),
            (19_456, 2, 1, 15, 32),
            (19_456, 2, 1, 65, 32),
            (19_456, 2, 1, 16, 31),
            (19_456, 2, 1, 16, 65),
        ] {
            assert!(
                ArgonPasswordGate::new(test_hash(
                    memory,
                    iterations,
                    parallelism,
                    salt_length,
                    output_length,
                ))
                .is_err()
            );
        }
    }

    #[test]
    fn rejects_wrong_algorithm_version_and_missing_required_fields() {
        let valid = test_hash(19_456, 2, 1, 16, 32);
        for invalid in [
            valid.replacen("argon2id", "argon2i", 1),
            valid.replacen("v=19", "v=16", 1),
            "$argon2id$v=19$m=19456,t=2,p=1$c2FsdA$".into(),
            "$argon2id$v=19$c2FsdA$AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".into(),
            "$argon2id$v=19$m=19456,t=2,p=1$$AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".into(),
        ] {
            assert!(ArgonPasswordGate::new(invalid).is_err());
        }
    }

    #[test]
    fn reports_invalid_hash_as_configuration_error() {
        assert!(matches!(
            ArgonPasswordGate::new("not-a-password-hash".into()),
            Err(ApplicationError::Configuration(
                ConfigurationError::InvalidPasswordHash
            ))
        ));
    }

    fn test_hash(
        memory: u32,
        iterations: u32,
        parallelism: u32,
        salt_length: usize,
        output_length: usize,
    ) -> String {
        let encoded_salt_length = (salt_length * 8).div_ceil(6);
        let encoded_output_length = (output_length * 8).div_ceil(6);
        format!(
            "$argon2id$v=19$m={memory},t={iterations},p={parallelism}${}${}",
            "A".repeat(encoded_salt_length),
            "A".repeat(encoded_output_length)
        )
    }
}
