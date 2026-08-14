# debtor

A pre-release Rust scaffold for a private, single-owner expense-sharing ledger.

## Status

The repository provides a runnable password-gated server with group and participant management, memberships, one shared equal/exact expense form with single or multiple payers, spending detail/edit/delete, advisory settlements, SQLite migrations, and Frankfurter rate integration.

The intended first-release product and architecture contract is documented in [specs/design.md](specs/design.md). That document is authoritative for planned behavior; it is not a claim that all behavior is implemented.

## Current Structure

```
debtor (root)
├── debtor-domain       # pure business rules
├── debtor-application  # use cases and mockable ports
├── debtor-infra        # SQLx, Argon2, and Frankfurter adapters
└── debtor-web          # Axum and Askama HTTP layer
```

## Development

```bash
cargo fmt --all -- --check
cargo run --bin architecture-check --locked
cargo deny check
cargo check --workspace --all-features --locked
SQLX_OFFLINE=true cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-features --locked
cargo run
```

CI runs formatting, architecture fitness, locked checks, offline lint, and tests for both the production workspace and the independent `tools/password-hash` helper. For local automatic Clippy fixes, use `cargo clippy --fix --allow-dirty --workspace` and review the resulting changes.

The independent password-helper checks are:

```bash
cargo fmt --manifest-path tools/password-hash/Cargo.toml -- --check
cargo deny --manifest-path tools/password-hash/Cargo.toml --config tools/password-hash/deny.toml check
cargo clippy --manifest-path tools/password-hash/Cargo.toml --all-targets --all-features --locked -- -D warnings
cargo test --manifest-path tools/password-hash/Cargo.toml --locked
```

Copy `.env.example` to `.env`, generate `APP_ADMIN_PASSWORD_HASH` with `cargo run --manifest-path tools/password-hash/Cargo.toml`, then run `cargo run`. Startup creates/connects SQLite, applies migrations, and serves the application. The complete local-run contract is specified in [specs/design.md](specs/design.md).

The database schema is pre-release. After migration or canonical monetary-persistence changes, stop the server and delete the local SQLite database so `cargo run` can recreate it; live database compatibility is not promised.

The server enforces fixed request budgets: 8 KiB login bodies, 256 KiB other form bodies, 64 shared in-flight permits for user and static traffic, four login permits, and four separate probe permits. Safe reads and login have a 30-second budget; debt reads have 90 seconds. An admitted ledger mutation is not cut off by the generic read timeout and must receive a definitive commit or rollback response, so the production reverse proxy must not impose a shorter mutation timeout.

Sessions are process-local and restart-invalidation is intentional. Anonymous login/CSRF sessions use a fixed 10-minute inactivity lifetime and are admitted up to 4,096 live records; authenticated sessions use a fixed 30-day inactivity lifetime and are capped at 32 without consuming anonymous capacity. Expiry cleanup uses indexed buckets rather than scanning all records. A correct login at the authenticated cap flushes the anonymous session and returns a retryable sanitized `503` without evicting existing users. Expired records are removed lazily during load/admission and by a supervised five-minute deletion pass. A cleanup failure marks readiness unhealthy and initiates fatal shutdown; login session-capacity or storage failures return a retryable sanitized `503`. No session-capacity environment knobs are supported.

Ordinary group expense history is rendered as 25-item keyset pages ordered by date and ID; complete history materialization is reserved for debt snapshots. Malformed history cursors return a sanitized `400`. The stable historical exchange-rate cache is bounded to 4,096 entries with deterministic LRU eviction.

## Edge Deployment

The Rust process remains behind a private-interface reverse proxy. TLS, certificates, HTTP/3, `Alt-Svc`, and HTTP/2 or HTTP/1.1 fallback terminate at the edge; Caddy is not a Rust runtime dependency.

The selected pre-production edge product is Caddy `2.11.2`, recorded in [ADR 0003](specs/adr/0003-pre-production-edge-gate.md). The repository source of truth for the selected edge template is [deploy/Caddyfile.example](deploy/Caddyfile.example), and the reproducible verification matrix is [deploy/edge-verification.md](deploy/edge-verification.md). Production rollout remains blocked until that matrix is executed in the target environment and evidence is captured.

- Validate the selected version and template with `docker run --rm -v "$PWD/deploy/Caddyfile.example:/etc/caddy/Caddyfile:ro" caddy:2.11.2 caddy validate --config /etc/caddy/Caddyfile --adapter caddyfile`.
- Set `APP_TRUSTED_PROXY_CIDRS` to Caddy's backend source CIDR and `APP_TRUSTED_PROXY_HEADER=x-forwarded-for`; the Caddy template strips untrusted forwarding input before setting `X-Forwarded-For` from Caddy-controlled peer data.
- Non-debug startup requires both a nonempty `APP_TRUSTED_PROXY_CIDRS` and exactly one supported `APP_TRUSTED_PROXY_HEADER`; direct-peer fallback is debug/local only.
- Caddy `2.11.2` is configured with `0rtt off`. The defense-in-depth early-data matcher only allows marked early data for `GET` or `HEAD` requests to session-free probes and immutable static assets; login, authenticated HTML, and every session/token-touching path are excluded even if they use `GET` or `HEAD`.
- Keep backend HTTP/1.1 connection reuse enabled. The selected template bounds connection setup only; do not set response-header, read, write, stream, request, or route timeouts that could expire before an admitted mutation reaches Debtor's definitive commit/rollback path.
- Enforce edge limits no larger than 8 KiB for `/login` and 256 KiB for other forms.
- Start HTTP/3 with the template's short `Alt-Svc: h3=":443"; ma=300` advertisement and do not increase the lifetime until UDP/443, blocked-UDP fallback, early-data behavior, edge telemetry, and identical resolved client IPs/rate limits are verified for HTTP/3 and fallback requests.

Validate an edge rollout with controlled test traffic before increasing the `Alt-Svc` lifetime or routing real administrator traffic:

1. Confirm the initial response advertises the short lifetime and that UDP/443 reaches the edge.
2. Block UDP from a test client and confirm the same endpoint succeeds over HTTP/2 or HTTP/1.1.
3. Send spoofed forwarding headers over HTTP/3 and fallback transport; Debtor must resolve the same trusted client identity and apply one shared login-limiter budget without logging that identity.
4. Send a marked unsafe early-data request and confirm it receives `425 Too Early` or is impossible because 0-RTT is disabled, without reaching Debtor.
5. Send oversized sentinel payloads and confirm `/login` bodies over 8 KiB and other unsafe form bodies over 256 KiB are rejected at or before the edge.
6. Inspect edge telemetry for backend HTTP/1.1 connection reuse and verify no configured edge timeout can cut off a deliberately slow, admitted mutation before its final response. Real slow-mutation evidence is deferred until the first ledger mutation story provides a route to exercise.
7. Inspect application logs and confirm they exclude credentials, hashes, cookies, session/CSRF/submission tokens, client identities, forwarding chains, query strings, provider URLs, SQL/database messages, raw diagnostics, and request-derived values.

`/healthz` is allocation-light process liveness and remains healthy while the process is running. `/readyz` is the local SQLite readiness probe: it acquires a pool connection and runs a trivial query with a one-second total budget, returning a sanitized `503` when SQLite is closed, unavailable, or contended. Both probes bypass sessions and use the dedicated four-request probe budget. Frankfurter availability, session counts, and ledger contents do not gate readiness. Use `/healthz` for process liveness and `/readyz` for local traffic admission or orchestrator readiness.

On Ctrl-C or SIGTERM, the server stops accepting new connections and drains active requests for at most 10 seconds before forced cancellation. Shutdown then checkpoints the WAL and closes SQLite with separate five-second bounds. Signal-only shutdown succeeds; cleanup, HTTP, checkpoint, or pool-close failures produce an unsuccessful exit. WAL sidecars are never manually removed during shutdown and remain available for SQLite recovery after a timeout.

Structured diagnostics report safe startup stages, listening address, readiness categories, route-pattern response status/latency, login rate-limit rejection categories, provider fallback categories, and shutdown outcomes. Request bodies, credentials, hashes, cookies, session/CSRF identifiers, forwarding details, query strings, and configured provider URLs are excluded.

## License

MIT OR Apache-2.0
