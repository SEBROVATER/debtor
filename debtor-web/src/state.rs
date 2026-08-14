//! Application-facing state shared by handlers.

use axum::http::HeaderMap;
use ipnet::IpNet;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::{
    net::{IpAddr, SocketAddr},
    str::FromStr,
};

use debtor_application::{
    AuthenticationUseCases, Clock, DebtUseCases, GroupUseCases, ParticipantUseCases,
    ReadinessUseCases, SpendingUseCases,
};

use crate::submission_tokens::SubmissionTokenStore;

/// Dependencies exposed to the HTTP layer as application interfaces.
#[derive(Clone)]
pub struct AppState {
    /// Group workflows.
    pub groups: Arc<dyn GroupUseCases>,
    /// Participant and membership workflows.
    pub participants: Arc<dyn ParticipantUseCases>,
    /// Spending workflows.
    pub spendings: Arc<dyn SpendingUseCases>,
    /// Debt workflows.
    pub debts: Arc<dyn DebtUseCases>,
    /// Authentication policy and password gate.
    pub authentication: Arc<dyn AuthenticationUseCases>,
    /// Shared UTC clock for deterministic form defaults.
    pub clock: Arc<dyn Clock>,
    /// Local dependency readiness checks.
    pub readiness: Arc<dyn ReadinessUseCases>,
    /// Trusted reverse-proxy client-IP policy.
    pub proxy: TrustedProxyConfig,
    /// Shared anonymous and authenticated submission-token owner.
    pub submission_tokens: SubmissionTokenStore,
    /// Process-local control for user admission and runtime failure signaling.
    pub runtime: RuntimeControl,
}

/// Narrow process-local control shared by the HTTP layer and root runtime.
#[derive(Clone)]
pub struct RuntimeControl {
    user_admission: Arc<AtomicBool>,
    shutdown_request: Arc<dyn Fn() + Send + Sync>,
}

impl Default for RuntimeControl {
    fn default() -> Self {
        Self::with_shutdown_request(|| {})
    }
}

impl RuntimeControl {
    /// Creates a control handle with an injected fatal-shutdown callback.
    pub fn with_shutdown_request<F>(shutdown_request: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        Self {
            user_admission: Arc::new(AtomicBool::new(true)),
            shutdown_request: Arc::new(shutdown_request),
        }
    }

    /// Returns whether new user traffic may enter the application.
    pub fn user_admission_open(&self) -> bool {
        self.user_admission.load(Ordering::Acquire)
    }

    /// Closes new user admission and reports whether this call changed state.
    pub fn close_user_admission(&self) -> bool {
        self.user_admission
            .compare_exchange(true, false, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
    }

    /// Closes user admission and requests one coordinated fatal shutdown.
    pub fn fail_readiness(&self) {
        if self.close_user_admission() {
            (self.shutdown_request)();
        }
    }
}

/// Selected forwarding-header policy.
#[derive(Clone, Copy)]
pub enum ProxyHeader {
    /// RFC Forwarded.
    Forwarded,
    /// X-Forwarded-For.
    XForwardedFor,
}

/// Trusted reverse-proxy configuration.
#[derive(Clone, Default)]
pub struct TrustedProxyConfig {
    cidrs: Arc<Vec<IpNet>>,
    header: Option<ProxyHeader>,
}

impl TrustedProxyConfig {
    /// Parses startup configuration.
    ///
    /// # Errors
    ///
    /// Returns an error for malformed CIDRs or unsupported header modes.
    pub fn parse(cidrs: &str, header: &str) -> Result<Self, String> {
        let networks = cidrs
            .split(',')
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|value| {
                IpNet::from_str(value).map_err(|_| format!("invalid trusted proxy CIDR: {value}"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let header = if networks.is_empty() {
            if !header.trim().is_empty() {
                return Err("APP_TRUSTED_PROXY_HEADER requires APP_TRUSTED_PROXY_CIDRS".into());
            }
            None
        } else {
            match header.trim() {
                "forwarded" => Some(ProxyHeader::Forwarded),
                "x-forwarded-for" => Some(ProxyHeader::XForwardedFor),
                _ => {
                    return Err(
                        "APP_TRUSTED_PROXY_HEADER must be forwarded or x-forwarded-for".into(),
                    );
                }
            }
        };
        Ok(Self {
            cidrs: Arc::new(networks),
            header,
        })
    }

    /// Parses startup configuration with the environment's direct-peer policy.
    ///
    /// # Errors
    ///
    /// Returns an error when production has no trusted proxy policy or when
    /// the CIDR/header configuration is malformed.
    pub fn parse_for_environment(
        cidrs: &str,
        header: &str,
        debug_assertions: bool,
    ) -> Result<Self, String> {
        if !debug_assertions && cidrs.split(',').all(|value| value.trim().is_empty()) {
            return Err("trusted proxy policy is required outside debug builds".into());
        }
        Self::parse(cidrs, header)
    }

    /// Resolves the client address using only configured trusted hops.
    ///
    /// # Errors
    ///
    /// Returns an error when the selected forwarding header is malformed.
    pub fn resolve(&self, peer: SocketAddr, headers: &HeaderMap) -> Result<IpAddr, String> {
        let peer = canonical_ip(peer.ip());
        if !self.cidrs.iter().any(|network| network.contains(&peer)) {
            return Ok(peer);
        }
        let Some(header) = self.header else {
            return Ok(peer);
        };
        let values = match header {
            ProxyHeader::Forwarded => headers.get_all("forwarded").iter().collect::<Vec<_>>(),
            ProxyHeader::XForwardedFor => headers
                .get_all("x-forwarded-for")
                .iter()
                .collect::<Vec<_>>(),
        };
        if values.is_empty() {
            return Ok(peer);
        }
        let mut chain = Vec::new();
        for value in values {
            let text = value
                .to_str()
                .map_err(|_| "malformed forwarding header".to_string())?;
            if matches!(header, ProxyHeader::Forwarded) {
                for element in split_forwarded(text, ',')? {
                    chain.push(parse_forwarded_element(element)?);
                }
            } else {
                for item in text.split(',') {
                    chain.push(parse_forwarded_ip(item)?);
                }
            }
        }
        let mut current = peer;
        for candidate in chain.into_iter().rev() {
            if !self.cidrs.iter().any(|network| network.contains(&current)) {
                break;
            }
            current = candidate;
        }
        Ok(current)
    }
}

fn split_forwarded(value: &str, delimiter: char) -> Result<Vec<&str>, String> {
    let mut values = Vec::new();
    let mut quoted = false;
    let mut start = 0;
    for (index, character) in value.char_indices() {
        match character {
            '"' => quoted = !quoted,
            '\\' if quoted => return Err("malformed Forwarded header".into()),
            _ if character == delimiter && !quoted => {
                let item = value[start..index].trim();
                if item.is_empty() {
                    return Err("malformed Forwarded header".into());
                }
                values.push(item);
                start = index + character.len_utf8();
            }
            _ => {}
        }
    }
    if quoted {
        return Err("malformed Forwarded header".into());
    }
    let item = value[start..].trim();
    if item.is_empty() {
        return Err("malformed Forwarded header".into());
    }
    values.push(item);
    Ok(values)
}

fn parse_forwarded_element(element: &str) -> Result<IpAddr, String> {
    let mut client = None;
    for parameter in split_forwarded(element, ';')? {
        let (name, value) = parameter
            .split_once('=')
            .ok_or_else(|| "malformed Forwarded header".to_string())?;
        if name.is_empty() || !name.bytes().all(is_token) || value.is_empty() {
            return Err("malformed Forwarded header".into());
        }
        if name.eq_ignore_ascii_case("for") {
            if client.replace(parse_forwarded_ip(value)?).is_some() {
                return Err("malformed Forwarded header".into());
            }
        } else if !is_forwarded_value(value) {
            return Err("malformed Forwarded header".into());
        }
    }
    client.ok_or_else(|| "malformed Forwarded header".into())
}

fn is_token(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || b"!#$%&'*+-.^_`|~".contains(&byte)
}

fn is_forwarded_value(value: &str) -> bool {
    value.bytes().all(is_token)
        || value.len() >= 2
            && value.starts_with('"')
            && value.ends_with('"')
            && value[1..value.len() - 1]
                .bytes()
                .all(|byte| byte.is_ascii_graphic() || byte == b' ')
}

fn parse_forwarded_ip(value: &str) -> Result<IpAddr, String> {
    let value = value.trim();
    let value = if value.starts_with('"') && value.ends_with('"') && value.len() >= 2 {
        &value[1..value.len() - 1]
    } else if value.contains(['"', '\\']) {
        return Err("malformed forwarding header".into());
    } else {
        value
    };
    if value.is_empty() || value.eq_ignore_ascii_case("unknown") || value.starts_with('_') {
        return Err("malformed forwarding header".into());
    }
    if value.starts_with('[') {
        let end = value
            .find(']')
            .ok_or_else(|| "malformed forwarding header".to_string())?;
        let suffix = &value[end + 1..];
        if !(suffix.is_empty() || suffix.strip_prefix(':').is_some_and(is_port)) {
            return Err("malformed forwarding header".into());
        }
        let IpAddr::V6(ip) = value[1..end]
            .parse()
            .map_err(|_| "malformed forwarding header".to_string())?
        else {
            return Err("malformed forwarding header".into());
        };
        return Ok(canonical_ip(IpAddr::V6(ip)));
    }
    if let Ok(ip) = value.parse() {
        return Ok(canonical_ip(ip));
    }
    let (host, port) = value
        .rsplit_once(':')
        .ok_or_else(|| "malformed forwarding header".to_string())?;
    if !is_port(port) {
        return Err("malformed forwarding header".into());
    }
    Ok(canonical_ip(
        host.parse()
            .map_err(|_| "malformed forwarding header".to_string())?,
    ))
}

fn is_port(value: &str) -> bool {
    !value.is_empty()
        && value.bytes().all(|byte| byte.is_ascii_digit())
        && value.parse::<u16>().is_ok()
}

fn canonical_ip(ip: IpAddr) -> IpAddr {
    match ip {
        IpAddr::V6(value) => value.to_ipv4().map_or(IpAddr::V6(value), IpAddr::V4),
        IpAddr::V4(value) => IpAddr::V4(value),
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use axum::http::{HeaderMap, HeaderValue};
    use std::net::{IpAddr, SocketAddr};

    use super::TrustedProxyConfig;

    fn peer(value: &str) -> SocketAddr {
        value.parse().expect("valid socket address")
    }

    fn ip(value: &str) -> IpAddr {
        value.parse().expect("valid IP address")
    }

    #[test]
    fn configuration_requires_proxy_cidrs_and_exactly_one_supported_header() {
        assert!(TrustedProxyConfig::parse("", "").is_ok());
        assert!(TrustedProxyConfig::parse("10.0.0.0/8", "forwarded").is_ok());
        assert!(TrustedProxyConfig::parse("", "forwarded").is_err());
        assert!(TrustedProxyConfig::parse("10.0.0.0/8", "").is_err());
        assert!(TrustedProxyConfig::parse("10.0.0.0/8", "Forwarded").is_err());
        assert!(TrustedProxyConfig::parse("not-a-cidr", "forwarded").is_err());
    }

    #[test]
    fn production_proxy_policy_rejects_direct_peer_fallback() {
        assert!(TrustedProxyConfig::parse_for_environment("", "", false).is_err());
        assert!(TrustedProxyConfig::parse_for_environment("10.0.0.0/8", "", false).is_err());
        assert!(
            TrustedProxyConfig::parse_for_environment("10.0.0.0/8", "forwarded", false).is_ok()
        );
        assert!(TrustedProxyConfig::parse_for_environment("", "", true).is_ok());
    }

    #[test]
    fn untrusted_peer_ignores_even_malformed_forwarding_headers() {
        let config = TrustedProxyConfig::parse("10.0.0.0/8", "x-forwarded-for")
            .expect("valid proxy configuration");
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", HeaderValue::from_static("not an IP"));

        assert_eq!(
            config.resolve(peer("192.0.2.10:443"), &headers),
            Ok(ip("192.0.2.10"))
        );
    }

    #[test]
    fn rejects_malformed_selected_forwarding_headers() {
        let forwarded = TrustedProxyConfig::parse("10.0.0.0/8", "forwarded")
            .expect("valid proxy configuration");
        let x_forwarded_for = TrustedProxyConfig::parse("10.0.0.0/8", "x-forwarded-for")
            .expect("valid proxy configuration");

        for (config, name, value) in [
            (&forwarded, "forwarded", "for=unknown"),
            (&forwarded, "forwarded", "for=192.0.2.1;proto"),
            (&forwarded, "forwarded", "for=192.0.2.1;for=192.0.2.2"),
            (&forwarded, "forwarded", "for=192.0.2.1;proto=\"https\\\""),
            (&forwarded, "forwarded", "for=[2001:db8::1"),
            (&x_forwarded_for, "x-forwarded-for", "192.0.2.1,"),
            (&x_forwarded_for, "x-forwarded-for", "_hidden"),
            (&x_forwarded_for, "x-forwarded-for", "192.0.2.1:+443"),
            (&x_forwarded_for, "x-forwarded-for", "[fe80::1%25eth0]:443"),
        ] {
            let mut headers = HeaderMap::new();
            headers.insert(name, HeaderValue::from_static(value));
            assert!(config.resolve(peer("10.0.0.5:443"), &headers).is_err());
        }
    }

    #[test]
    fn resolves_trusted_proxy_chain_from_right_to_left() {
        let config = TrustedProxyConfig::parse("10.0.0.0/8", "x-forwarded-for")
            .expect("valid proxy configuration");
        let mut headers = HeaderMap::new();
        headers.append(
            "x-forwarded-for",
            HeaderValue::from_static("192.0.2.25, 198.51.100.7"),
        );
        headers.append("x-forwarded-for", HeaderValue::from_static("10.0.0.8"));

        assert_eq!(
            config.resolve(peer("10.0.0.9:443"), &headers),
            Ok(ip("198.51.100.7"))
        );
    }

    #[test]
    fn canonicalizes_ipv4_mapped_ipv6_forwarding_addresses() {
        let config = TrustedProxyConfig::parse("10.0.0.0/8", "forwarded")
            .expect("valid proxy configuration");
        let mut headers = HeaderMap::new();
        headers.insert(
            "forwarded",
            HeaderValue::from_static("for=\"[::ffff:192.0.2.25]\""),
        );

        assert_eq!(
            config.resolve(peer("[::ffff:10.0.0.9]:443"), &headers),
            Ok(ip("192.0.2.25"))
        );
    }

    #[test]
    fn accepts_bare_addresses_and_strict_port_literals() {
        let config = TrustedProxyConfig::parse("10.0.0.0/8", "forwarded")
            .expect("valid proxy configuration");
        let mut headers = HeaderMap::new();
        headers.insert(
            "forwarded",
            HeaderValue::from_static("for=192.0.2.25:00443;proto=https"),
        );
        assert_eq!(
            config.resolve(peer("10.0.0.9:443"), &headers),
            Ok(ip("192.0.2.25"))
        );

        let config = TrustedProxyConfig::parse("10.0.0.0/8", "x-forwarded-for")
            .expect("valid proxy configuration");
        headers.insert(
            "x-forwarded-for",
            HeaderValue::from_static("[2001:db8::25]:65535"),
        );
        assert_eq!(
            config.resolve(peer("10.0.0.9:443"), &headers),
            Ok(ip("2001:db8::25"))
        );
    }
}

#[cfg(test)]
mod runtime_control_tests {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    use super::RuntimeControl;

    #[test]
    fn runtime_failure_closes_user_admission_and_notifies_once() {
        let notifications = Arc::new(AtomicUsize::new(0));
        let counter = notifications.clone();
        let control = RuntimeControl::with_shutdown_request(move || {
            counter.fetch_add(1, Ordering::SeqCst);
        });

        assert!(control.user_admission_open());
        control.fail_readiness();
        control.fail_readiness();

        assert!(!control.user_admission_open());
        assert_eq!(notifications.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn ordinary_shutdown_closes_admission_without_requesting_failure() {
        let notifications = Arc::new(AtomicUsize::new(0));
        let counter = notifications.clone();
        let control = RuntimeControl::with_shutdown_request(move || {
            counter.fetch_add(1, Ordering::SeqCst);
        });

        assert!(control.close_user_admission());
        assert!(!control.close_user_admission());
        assert!(!control.user_admission_open());
        assert_eq!(notifications.load(Ordering::SeqCst), 0);
    }
}
