//! Generate `APP_ADMIN_PASSWORD_HASH` for local or deployed configuration.
//!
//! The fixed profile is Argon2id v19 with 19,456 KiB of memory, two passes,
//! one lane, a 16-byte OS-generated salt, and a 32-byte output.

use std::error::Error;
use std::fmt::{self, Display, Formatter};

use argon2::{
    Algorithm, Argon2, Params, Version,
    password_hash::{PasswordHasher, SaltString},
};
use rand::{TryRng, rngs::SysRng};
use zeroize::Zeroizing;

const MEMORY_COST_KIB: u32 = 19_456;
const TIME_COST: u32 = 2;
const PARALLELISM: u32 = 1;
const SALT_LENGTH: usize = 16;
const OUTPUT_LENGTH: usize = 32;

#[derive(Debug, Eq, PartialEq)]
enum ValidationError {
    Empty,
    Mismatch,
}

impl Display for ValidationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => formatter.write_str("password must not be empty"),
            Self::Mismatch => formatter.write_str("passwords do not match"),
        }
    }
}

impl Error for ValidationError {}

fn validate_passwords(password: &str, confirmation: &str) -> Result<(), ValidationError> {
    if password.is_empty() {
        return Err(ValidationError::Empty);
    }

    if password != confirmation {
        return Err(ValidationError::Mismatch);
    }

    Ok(())
}

fn configured_argon2() -> Result<Argon2<'static>, argon2::Error> {
    let params = Params::new(MEMORY_COST_KIB, TIME_COST, PARALLELISM, Some(OUTPUT_LENGTH))?;

    Ok(Argon2::new(Algorithm::Argon2id, Version::V0x13, params))
}

fn hash_password(
    password: &[u8],
    salt: &[u8; SALT_LENGTH],
) -> Result<Zeroizing<String>, Box<dyn Error>> {
    let salt_string = SaltString::encode_b64(salt)?;
    let password_hash = configured_argon2()?.hash_password(password, &salt_string)?;

    Ok(Zeroizing::new(password_hash.to_string()))
}

fn main() -> Result<(), Box<dyn Error>> {
    let password = Zeroizing::new(rpassword::prompt_password("Password: ")?);
    let confirmation = Zeroizing::new(rpassword::prompt_password("Confirm password: ")?);
    validate_passwords(password.as_str(), confirmation.as_str())?;

    let mut salt = Zeroizing::new([0_u8; SALT_LENGTH]);
    let mut system_rng = SysRng;
    system_rng.try_fill_bytes(&mut salt[..])?;

    let password_hash = hash_password(password.as_bytes(), &salt)?;
    println!("APP_ADMIN_PASSWORD_HASH='{}'", password_hash.as_str());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use argon2::password_hash::{PasswordHash, PasswordVerifier};

    #[test]
    fn rejects_empty_password() {
        assert_eq!(validate_passwords("", ""), Err(ValidationError::Empty));
    }

    #[test]
    fn rejects_mismatched_passwords() {
        assert_eq!(
            validate_passwords("password", "different"),
            Err(ValidationError::Mismatch)
        );
    }

    #[test]
    fn hashes_with_documented_profile() {
        let salt = [7_u8; SALT_LENGTH];
        let password_hash = hash_password(b"test password", &salt).expect("hashing succeeds");

        assert!(password_hash.starts_with("$argon2id$v=19$m=19456,t=2,p=1$"));

        let parsed_hash = PasswordHash::new(password_hash.as_str()).expect("PHC hash parses");
        assert_eq!(
            parsed_hash
                .hash
                .as_ref()
                .expect("hash output exists")
                .as_bytes()
                .len(),
            OUTPUT_LENGTH
        );
        assert!(
            configured_argon2()
                .expect("parameters are valid")
                .verify_password(b"test password", &parsed_hash)
                .is_ok()
        );
    }
}
