//! Authentication adapters for password verification and login rate limiting.

mod login_limiter;
mod password;

pub use login_limiter::MemoryLoginAttemptLimiter;
pub use password::{ArgonPasswordGate, validate_password_hash};
