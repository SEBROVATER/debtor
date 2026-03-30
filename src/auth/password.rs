use argon2::{Argon2, PasswordHash, PasswordVerifier, password_hash::Error as PasswordHashError};

#[derive(Debug, thiserror::Error)]
pub enum PasswordError {
    #[error("invalid password hash")]
    InvalidHash,
    #[error("password verification failed")]
    VerificationFailed,
}

pub fn verify_password(password_hash: &str, password: &str) -> Result<(), PasswordError> {
    let parsed = PasswordHash::new(password_hash).map_err(|_| PasswordError::InvalidHash)?;
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .map_err(|err| match err {
            PasswordHashError::Password => PasswordError::VerificationFailed,
            _ => PasswordError::InvalidHash,
        })
}
