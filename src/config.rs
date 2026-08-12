use std::net::SocketAddr;

use anyhow::{Context, Result, anyhow, bail};
use debtor_infra::auth::validate_password_hash;

/// Validated process configuration.
pub(super) struct Config {
    pub(super) database_url: String,
    pub(super) bind: SocketAddr,
    pub(super) password_hash: String,
    pub(super) cookie_secure: bool,
    pub(super) session_cookie_name: String,
    pub(super) exchange_base_url: String,
    pub(super) trusted_proxy_cidrs: String,
    pub(super) trusted_proxy_header: String,
}

impl Config {
    pub(super) fn from_lookup(
        lookup: impl Fn(&str) -> Option<String>,
        debug_assertions: bool,
    ) -> Result<Self> {
        let password_hash = lookup("APP_ADMIN_PASSWORD_HASH")
            .ok_or_else(|| anyhow!("APP_ADMIN_PASSWORD_HASH is required"))?;
        if password_hash.trim().is_empty() {
            bail!("APP_ADMIN_PASSWORD_HASH must not be empty");
        }
        validate_password_hash(&password_hash)
            .map_err(|_| anyhow!("APP_ADMIN_PASSWORD_HASH is invalid"))?;

        let cookie_secure = lookup("APP_SESSION_COOKIE_SECURE")
            .unwrap_or_else(|| (!debug_assertions).to_string())
            .parse::<bool>()
            .context("APP_SESSION_COOKIE_SECURE must be true or false")?;
        if !debug_assertions && !cookie_secure {
            bail!("insecure session cookies are only allowed in debug builds");
        }

        let session_cookie_name =
            lookup("APP_SESSION_COOKIE_NAME").unwrap_or_else(|| "debtor_session".to_owned());
        validate_cookie_name(&session_cookie_name)?;

        Ok(Self {
            database_url: lookup("APP_DATABASE_URL")
                .unwrap_or_else(|| "sqlite://debtor.db?mode=rwc".to_owned()),
            bind: lookup("APP_BIND")
                .unwrap_or_else(|| "127.0.0.1:3000".to_owned())
                .parse()
                .context("APP_BIND must be a socket address")?,
            password_hash,
            cookie_secure,
            session_cookie_name,
            exchange_base_url: lookup("APP_EXCHANGE_BASE_URL")
                .unwrap_or_else(|| "https://api.frankfurter.dev/v2".to_owned()),
            trusted_proxy_cidrs: lookup("APP_TRUSTED_PROXY_CIDRS").unwrap_or_default(),
            trusted_proxy_header: lookup("APP_TRUSTED_PROXY_HEADER").unwrap_or_default(),
        })
    }
}

fn validate_cookie_name(name: &str) -> Result<()> {
    if name.is_empty() || !name.bytes().all(is_cookie_name_byte) {
        bail!("APP_SESSION_COOKIE_NAME must be a valid cookie name");
    }
    Ok(())
}

fn is_cookie_name_byte(byte: u8) -> bool {
    byte.is_ascii_graphic()
        && !matches!(
            byte,
            b'(' | b')'
                | b'<'
                | b'>'
                | b'@'
                | b','
                | b';'
                | b':'
                | b'\\'
                | b'"'
                | b'/'
                | b'['
                | b']'
                | b'?'
                | b'='
                | b'{'
                | b'}'
        )
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::collections::HashMap;

    use super::Config;

    const VALID_HASH: &str = "$argon2id$v=19$m=19456,t=2,p=1$AAAAAAAAAAAAAAAAAAAAAA$AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";

    fn config(values: &[(&str, &str)], debug_assertions: bool) -> anyhow::Result<Config> {
        let values = values
            .iter()
            .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
            .collect::<HashMap<_, _>>();
        Config::from_lookup(|key| values.get(key).cloned(), debug_assertions)
    }

    #[test]
    fn applies_defaults_with_insecure_debug_cookies() {
        let config = config(&[("APP_ADMIN_PASSWORD_HASH", VALID_HASH)], true).unwrap();

        assert_eq!(config.database_url, "sqlite://debtor.db?mode=rwc");
        assert_eq!(config.bind.to_string(), "127.0.0.1:3000");
        assert_eq!(config.password_hash, VALID_HASH);
        assert!(!config.cookie_secure);
        assert_eq!(config.session_cookie_name, "debtor_session");
        assert_eq!(config.exchange_base_url, "https://api.frankfurter.dev/v2");
        assert!(config.trusted_proxy_cidrs.is_empty());
        assert!(config.trusted_proxy_header.is_empty());
    }

    #[test]
    fn requires_a_nonempty_password_hash() {
        assert!(config(&[], true).is_err());
        assert!(config(&[("APP_ADMIN_PASSWORD_HASH", " \t")], true).is_err());
    }

    #[test]
    fn rejects_password_hashes_that_do_not_meet_the_startup_policy() {
        for hash in [
            "not-a-password-hash".to_owned(),
            "A".repeat(257),
            "$argon2id$v=19$m=019456,t=2,p=1$AAAAAAAAAAAAAAAAAAAAAA$AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_owned(),
        ] {
            let Err(error) = config(&[("APP_ADMIN_PASSWORD_HASH", &hash)], true) else {
                panic!("invalid hashes must not produce configuration");
            };
            assert!(!error.to_string().contains(&hash));
        }
    }

    #[test]
    fn rejects_invalid_values() {
        assert!(
            config(
                &[
                    ("APP_ADMIN_PASSWORD_HASH", VALID_HASH),
                    ("APP_BIND", "not-an-address")
                ],
                true,
            )
            .is_err()
        );
        assert!(
            config(
                &[
                    ("APP_ADMIN_PASSWORD_HASH", VALID_HASH),
                    ("APP_SESSION_COOKIE_SECURE", "sometimes"),
                ],
                true,
            )
            .is_err()
        );
    }

    #[test]
    fn rejects_insecure_cookies_outside_debug_builds() {
        assert!(
            config(&[("APP_ADMIN_PASSWORD_HASH", VALID_HASH)], false)
                .unwrap()
                .cookie_secure
        );
        assert!(
            config(
                &[
                    ("APP_ADMIN_PASSWORD_HASH", VALID_HASH),
                    ("APP_SESSION_COOKIE_SECURE", "false"),
                ],
                false,
            )
            .is_err()
        );
    }

    #[test]
    fn validates_session_cookie_names() {
        for name in ["debtor_session", "__Host-session", "session123"] {
            assert!(
                config(
                    &[
                        ("APP_ADMIN_PASSWORD_HASH", VALID_HASH),
                        ("APP_SESSION_COOKIE_NAME", name),
                    ],
                    true,
                )
                .is_ok()
            );
        }

        for name in [
            "",
            "session cookie",
            "session;cookie",
            "session=cookie",
            "caf\u{7f}",
        ] {
            assert!(
                config(
                    &[
                        ("APP_ADMIN_PASSWORD_HASH", VALID_HASH),
                        ("APP_SESSION_COOKIE_NAME", name),
                    ],
                    true,
                )
                .is_err()
            );
        }
    }
}
