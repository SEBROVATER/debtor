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
SQLX_OFFLINE=true cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-features --locked
cargo run
```

CI runs equivalent formatting, lint, and test checks for both the production workspace and the independent `tools/password-hash` helper. For local automatic Clippy fixes, use `cargo clippy --fix --allow-dirty --workspace` and review the resulting changes.

Copy `.env.example` to `.env`, generate `APP_ADMIN_PASSWORD_HASH` with `cargo run --manifest-path tools/password-hash/Cargo.toml`, then run `cargo run`. Startup creates/connects SQLite, applies migrations, and serves the application. The complete local-run contract is specified in [specs/design.md](specs/design.md).

The database schema is pre-release. After migration or canonical monetary-persistence changes, stop the server and delete the local SQLite database so `cargo run` can recreate it; live database compatibility is not promised.

The server enforces fixed request budgets: 8 KiB login bodies, 256 KiB other form bodies, 64 shared in-flight permits for user and static traffic, four login permits, and four separate probe permits. Safe reads and login have a 30-second budget; debt reads have 90 seconds. An admitted ledger mutation is not cut off by the generic read timeout and must receive a definitive commit or rollback response, so the production reverse proxy must not impose a shorter mutation timeout.

Sessions are process-local and restart-invalidation is intentional. Anonymous login/CSRF sessions use a fixed 10-minute inactivity lifetime and are admitted up to 4,096 live records; authenticated sessions use a fixed 30-day inactivity lifetime and do not consume anonymous capacity. Expired records are removed lazily during load/admission, and the explicit expired-deletion pass is available without a periodic worker in this slice. Login session-capacity or storage failures return a retryable sanitized `503`; no session-capacity environment knobs are supported.

`/healthz` is allocation-light process liveness and remains healthy while the process is running. `/readyz` is the local SQLite readiness probe: it acquires a pool connection and runs a trivial query with a one-second total budget, returning a sanitized `503` when SQLite is closed, unavailable, or contended. Both probes bypass sessions and use the dedicated four-request probe budget. Frankfurter availability, session counts, and ledger contents do not gate readiness. Use `/healthz` for process liveness and `/readyz` for local traffic admission or orchestrator readiness.

## License

MIT OR Apache-2.0
