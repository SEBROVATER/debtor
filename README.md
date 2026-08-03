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
cargo check
cargo test
cargo fmt
cargo clippy --fix --allow-dirty --workspace
cargo run
```

Copy `.env.example` to `.env`, generate `APP_ADMIN_PASSWORD_HASH` with `cargo run --manifest-path tools/password-hash/Cargo.toml`, then run `cargo run`. Startup creates/connects SQLite, applies migrations, and serves the application. The complete local-run contract is specified in [specs/design.md](specs/design.md).

## License

MIT OR Apache-2.0
