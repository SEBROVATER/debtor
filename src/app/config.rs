use std::env;

use thiserror::Error;

use crate::auth::session_repo::SessionCookiePolicy;

const DEFAULT_DATABASE_URL: &str = "sqlite://debtor.db?mode=rwc";
const DEFAULT_SESSION_COOKIE_NAME: &str = "debtor_session";
const DEFAULT_ADMIN_USERNAME: &str = "owner";
const DEFAULT_EXCHANGE_BASE_URL: &str = "https://api.frankfurter.app";

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub database_url: String,
    pub session_cookie_name: String,
    pub admin_username: String,
    pub admin_password_hash: Option<String>,
    pub secure_cookie: bool,
    pub exchange_base_url: String,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("invalid APP_SESSION_COOKIE_SECURE value `{0}`; expected true/false")]
    InvalidSecureCookie(String),
}

impl AppConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let database_url =
            env::var("APP_DATABASE_URL").unwrap_or_else(|_| DEFAULT_DATABASE_URL.to_string());
        let session_cookie_name = env::var("APP_SESSION_COOKIE_NAME")
            .unwrap_or_else(|_| DEFAULT_SESSION_COOKIE_NAME.to_string());
        let admin_username =
            env::var("APP_ADMIN_USERNAME").unwrap_or_else(|_| DEFAULT_ADMIN_USERNAME.to_string());
        let admin_password_hash = env::var("APP_ADMIN_PASSWORD_HASH").ok();
        let secure_cookie = parse_bool_env("APP_SESSION_COOKIE_SECURE")?.unwrap_or(false);
        let exchange_base_url = env::var("APP_EXCHANGE_BASE_URL")
            .unwrap_or_else(|_| DEFAULT_EXCHANGE_BASE_URL.to_string());

        Ok(Self {
            database_url,
            session_cookie_name,
            admin_username,
            admin_password_hash,
            secure_cookie,
            exchange_base_url,
        })
    }

    pub fn session_cookie_policy(&self) -> SessionCookiePolicy {
        SessionCookiePolicy::from_config(self)
    }
}

fn parse_bool_env(name: &str) -> Result<Option<bool>, ConfigError> {
    match env::var(name) {
        Ok(value) => {
            let normalized = value.trim().to_ascii_lowercase();
            match normalized.as_str() {
                "1" | "true" | "yes" | "on" => Ok(Some(true)),
                "0" | "false" | "no" | "off" => Ok(Some(false)),
                _ => Err(ConfigError::InvalidSecureCookie(value)),
            }
        }
        Err(env::VarError::NotPresent) => Ok(None),
        Err(env::VarError::NotUnicode(_)) => Err(ConfigError::InvalidSecureCookie(
            "<non-unicode-value>".to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::{AppConfig, parse_bool_env};
    use std::env;

    #[test]
    fn uses_defaults_when_env_vars_absent() {
        unsafe {
            env::remove_var("APP_DATABASE_URL");
            env::remove_var("APP_SESSION_COOKIE_NAME");
            env::remove_var("APP_ADMIN_USERNAME");
            env::remove_var("APP_ADMIN_PASSWORD_HASH");
            env::remove_var("APP_SESSION_COOKIE_SECURE");
            env::remove_var("APP_EXCHANGE_BASE_URL");
        }

        let cfg = AppConfig::from_env().expect("config should load");
        assert_eq!(cfg.database_url, "sqlite://debtor.db?mode=rwc");
        assert_eq!(cfg.session_cookie_name, "debtor_session");
        assert_eq!(cfg.admin_username, "owner");
        assert_eq!(cfg.admin_password_hash, None);
        assert!(!cfg.secure_cookie);
        assert_eq!(cfg.exchange_base_url, "https://api.frankfurter.app");
    }

    #[test]
    fn parses_secure_cookie_boolean_values() {
        unsafe {
            env::set_var("APP_SESSION_COOKIE_SECURE", "true");
        }
        assert_eq!(
            parse_bool_env("APP_SESSION_COOKIE_SECURE").unwrap(),
            Some(true)
        );
        unsafe {
            env::set_var("APP_SESSION_COOKIE_SECURE", "0");
        }
        assert_eq!(
            parse_bool_env("APP_SESSION_COOKIE_SECURE").unwrap(),
            Some(false)
        );
        unsafe {
            env::remove_var("APP_SESSION_COOKIE_SECURE");
        }
    }
}
