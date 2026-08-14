# ADR 0003: Pre-Production HTTPS Edge Gate

- Status: Accepted
- Date: 2026-08-14
- Scope: Pre-production reverse-proxy product selection, edge configuration, and rollout evidence

## Context

Debtor's production topology is fixed by `specs/design.md`, ADR 0001, and ADR 0002: one private HTTP/1.1 application process behind a sanitizing HTTPS reverse proxy. The application must not grow backend TLS, QUIC, HTTP/3, UDP, certificate automation, or vendor-specific edge behavior. Before production rollout, operations needs one approved edge product/version, one configuration source of truth, and reproducible evidence that the edge preserves forwarding, body-limit, early-data, fallback, timeout, and diagnostic invariants.

The repository already carried a Caddy example, but it was non-normative and not complete production evidence. This ADR promotes a checked Caddy template as the pre-production gate artifact while keeping final production rollout blocked until the verification matrix below is executed in the target environment.

## Decisions

1. Select Caddy `2.11.2` as the pre-production HTTPS edge product and exact version.
2. Use the official `caddy:2.11.2` container image, or a locally installed `caddy` binary that reports `2.11.2`, for configuration validation and pre-production verification.
3. Use `deploy/Caddyfile.example` as the repository source of truth for the selected Caddy edge configuration template. Operators must replace the hostname and the literal backend `host:port` without changing the security semantics; the template deliberately does not interpolate an upstream URL or accept an upstream URL scheme.
4. Use a single pre-production verification environment that mirrors production network topology: browser or test client to Caddy over HTTPS with HTTP/2 and HTTP/3 enabled, Caddy to Debtor over private HTTP/1.1 on `APP_BIND`, and Debtor configured with `APP_TRUSTED_PROXY_HEADER=x-forwarded-for` plus `APP_TRUSTED_PROXY_CIDRS` matching Caddy's backend source CIDR.
5. Disable QUIC 0-RTT at Caddy using the documented global `servers { 0rtt off }` option. Debtor does not rely on CSRF or submission tokens for early-data replay safety.
6. Strip untrusted forwarding and proxy-identity headers, including `X-Forwarded-For`, `Forwarded`, `X-Forwarded-Host`, `X-Forwarded-Port`, `X-Forwarded-Proto`, `X-Forwarded-Server`, `X-Real-IP`, `Client-IP`, `True-Client-IP`, `CF-Connecting-IP`, and `X-Cluster-Client-IP` before forwarding, then set only the selected `X-Forwarded-For` and supporting proto/host values from Caddy-controlled placeholders.
7. Keep Debtor as a private HTTP/1.1 backend. The edge owns TLS, automatic certificates, HTTP/2, HTTP/3/QUIC, `Alt-Svc`, and client-facing protocol fallback.
8. Enforce Caddy request body limits at or below Debtor limits: 8 KiB for `POST /login` and 256 KiB for every other unsafe form method.
9. Keep backend connection reuse enabled and restrict upstream transport versions to HTTP/1.1. Bound connection setup only. Do not configure response-header, read, write, stream, or request timeouts that can expire before a future admitted post-dispatch mutation returns its definitive result.
10. Roll out HTTP/3 with an explicit short initial `Alt-Svc` lifetime of 300 seconds. Do not increase advertised `Alt-Svc` lifetime until UDP/443 reachability, TCP fallback, disabled early data, and cross-protocol trusted-client parity are proven.
11. Production rollout cannot proceed until the verification evidence in this ADR and README is captured for the selected environment. Story 2.1 remains responsible for evidence involving a real admitted ledger mutation and definitive mutation outcome publication.

## Fixed Edge Obligations And Evidence

| Obligation | Caddy 2.11.2 configuration | Required executable evidence |
| --- | --- | --- |
| Product/version selected | `caddy:2.11.2` or `caddy version` reporting `2.11.2` | Record `caddy version` or container image digest used for verification. |
| Private backend | `reverse_proxy 127.0.0.1:3000` with `transport http { versions 1.1 }`; operators replace only the literal host:port | Capture Caddy config adaptation/validation and backend telemetry showing HTTP/1.1 upstream reuse. |
| Forwarding sanitation | Deletes the listed forwarding/proxy-identity headers, then sets `X-Forwarded-For {remote_host}` and controlled proto/host values | Send spoofed forwarding headers over HTTP/3 and TCP fallback and verify Debtor applies one trusted-client identity and limiter budget. |
| Early-data replay safety | Global `servers { 0rtt off }` | Verify QUIC 0-RTT is disabled, or that marked early-data unsafe traffic receives `425` before backend dispatch if an upstream edge re-enables early data. |
| Body limits | `request_body` max size `8KiB` for `POST /login`; `256KiB` for other unsafe form methods | Send oversized login and form payloads and verify rejection at or before Caddy without mutation dispatch. |
| Backend reuse and timeouts | HTTP transport keepalive enabled, `dial_timeout 5s`, no response-header/read/write/stream/request timeout | Inspect Caddy configuration and live upstream connections; real slow-mutation evidence remains a Story 2.1 gate. |
| HTTP/3 staged rollout | Explicit `Alt-Svc: h3=":443"; ma=300`; no longer lifetime configured in repo | Verify initial `Alt-Svc`, UDP/443 reachability, blocked-UDP fallback to HTTP/2 or HTTP/1.1, and identity parity before any longer advertisement. |
| Safe diagnostics | Debtor logging policy unchanged; edge verification uses sentinel test values only | Capture application logs and assert credentials, hashes, cookies, session/CSRF/submission tokens, client identity, forwarding chains, query strings, provider URLs, SQL/database details, and request-derived values are absent. |

## Rollout Assumptions

- Debtor is deployed as one process with one local SQLite volume and a private backend bind address, for example `127.0.0.1:3000` or a private container network address.
- Caddy is the only trusted reverse proxy directly connected to Debtor.
- `APP_TRUSTED_PROXY_CIDRS` contains Caddy's backend source CIDR and is nonempty outside debug builds.
- `APP_TRUSTED_PROXY_HEADER=x-forwarded-for` remains the selected mode for this Caddy template.
- Certificate automation, public DNS, firewalling, UDP/443 exposure, Caddy persistence, and service supervision are operations-owned and remain outside Rust application stories.

## Rollback Assumptions

- If any verification fails, keep Debtor private and do not advertise production HTTPS traffic through the edge.
- Reverting to debug/local direct HTTP is allowed only for local development, not production fallback.
- If Caddy version behavior changes, either pin back to `2.11.2` or open a new ADR that records the replacement product/version and synchronized config/evidence.
- If Story 2.1 later proves a real mutation path needs different edge timeout handling, update this ADR, `specs/design.md` if behavior changes, and the Caddy template in the same change.

## Consequences

- Application feature stories remain free of vendor-specific edge work and continue to target private HTTP/1.1.
- Operators get one concrete edge artifact and a verification checklist instead of ambiguous deployment advice.
- Production rollout remains intentionally gated until environment-specific evidence is captured; the committed template alone is not enough to claim production readiness.

## Superseded Decisions

This ADR does not supersede ADR 0001 or ADR 0002. It resolves their deferred reverse-proxy product/configuration choice for pre-production verification while preserving the edge responsibilities and private backend topology already fixed by those ADRs and `specs/design.md`.
