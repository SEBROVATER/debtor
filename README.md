# debtor

A pre-release Rust scaffold for a private, single-owner expense-sharing ledger.

## Status

The repository now provides a runnable password-gated server with group and participant management, group membership, equal-split spendings with multiple payer contributions, advisory debt calculation, SQLite migrations, and Frankfurter rate integration. Editing, exact-share forms, statistics, and several hardening tasks remain in progress.

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
cargo check
cargo test
cargo fmt
cargo clippy --fix --allow-dirty --workspace
cargo run
```

Copy `.env.example` to `.env`, set `APP_ADMIN_PASSWORD_HASH`, then run `cargo run`. Startup creates/connects SQLite, applies migrations, and serves the application. The complete local-run contract is specified in [specs/design.md](specs/design.md).

## License

MIT OR Apache-2.0
