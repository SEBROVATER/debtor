//! Application-facing state shared by handlers.

use axum::http::HeaderMap;
use ipnet::IpNet;
use std::sync::Arc;
use std::{
    net::{IpAddr, SocketAddr},
    str::FromStr,
};

use debtor_application::{
    Clock, DebtUseCases, GroupUseCases, LoginAttemptLimiter, ParticipantUseCases, PasswordVerifier,
    SpendingUseCases,
};

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
    /// Password gate verifier.
    pub password: Arc<dyn PasswordVerifier>,
    /// Shared UTC clock for deterministic form defaults.
    pub clock: Arc<dyn Clock>,
    /// Login attempt limiter.
    pub limiter: Arc<dyn LoginAttemptLimiter>,
    /// Trusted reverse-proxy client-IP policy.
    pub proxy: TrustedProxyConfig,
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

    /// Resolves the client address using only configured trusted hops.
    ///
    /// # Errors
    ///
    /// Returns an error when the selected forwarding header is malformed.
    pub fn resolve(&self, peer: SocketAddr, headers: &HeaderMap) -> Result<IpAddr, String> {
        if !self
            .cidrs
            .iter()
            .any(|network| network.contains(&peer.ip()))
        {
            return Ok(peer.ip());
        }
        let Some(header) = self.header else {
            return Ok(peer.ip());
        };
        let values = match header {
            ProxyHeader::Forwarded => headers.get_all("forwarded").iter().collect::<Vec<_>>(),
            ProxyHeader::XForwardedFor => headers
                .get_all("x-forwarded-for")
                .iter()
                .collect::<Vec<_>>(),
        };
        if values.is_empty() {
            return Ok(peer.ip());
        }
        let mut chain = Vec::new();
        for value in values {
            let text = value
                .to_str()
                .map_err(|_| "malformed forwarding header".to_string())?;
            if matches!(header, ProxyHeader::Forwarded) {
                for element in text.split(',') {
                    let mut values = element.trim().split(';').map(str::trim);
                    let item = values
                        .next()
                        .and_then(|part| part.strip_prefix("for="))
                        .ok_or_else(|| "malformed Forwarded header".to_string())?;
                    if values.next().is_some() {
                        return Err("malformed Forwarded header".into());
                    }
                    chain.push(parse_forwarded_ip(item)?);
                }
            } else {
                for item in text.split(',') {
                    chain.push(parse_forwarded_ip(item)?);
                }
            }
        }
        let mut current = peer.ip();
        for candidate in chain.into_iter().rev() {
            if !self.cidrs.iter().any(|network| network.contains(&current)) {
                break;
            }
            current = candidate;
        }
        Ok(current)
    }
}

fn parse_forwarded_ip(value: &str) -> Result<IpAddr, String> {
    let value = value.trim();
    let value = if value.starts_with('"') && value.ends_with('"') && value.len() >= 2 {
        &value[1..value.len() - 1]
    } else if value.contains('"') {
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
        if !(suffix.is_empty() || suffix.starts_with(':') && suffix[1..].parse::<u16>().is_ok()) {
            return Err("malformed forwarding header".into());
        }
        return Ok(canonical_ip(
            value[1..end]
                .parse()
                .map_err(|_| "malformed forwarding header".to_string())?,
        ));
    }
    if let Ok(ip) = value.parse() {
        return Ok(canonical_ip(ip));
    }
    let (host, port) = value
        .rsplit_once(':')
        .ok_or_else(|| "malformed forwarding header".to_string())?;
    if port.parse::<u16>().is_err() {
        return Err("malformed forwarding header".into());
    }
    Ok(canonical_ip(
        host.parse()
            .map_err(|_| "malformed forwarding header".to_string())?,
    ))
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
            (&forwarded, "forwarded", "for=192.0.2.1;proto=https"),
            (&forwarded, "forwarded", "for=[2001:db8::1"),
            (&x_forwarded_for, "x-forwarded-for", "192.0.2.1,"),
            (&x_forwarded_for, "x-forwarded-for", "_hidden"),
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
            config.resolve(peer("10.0.0.9:443"), &headers),
            Ok(ip("192.0.2.25"))
        );
    }
}
