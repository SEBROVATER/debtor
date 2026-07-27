# debtor

A pre-release Rust scaffold for a private, single-owner expense-sharing ledger.

## Status

The repository currently contains workspace scaffolding, initial domain code, database migrations, and adapter/web placeholders. It does not yet provide a runnable ledger server, completed authentication, or the product features described by the design.

The intended first-release product and architecture contract is documented in [specs/design.md](specs/design.md). That document is authoritative for planned behavior; it is not a claim that all behavior is implemented.

## Current Structure

```
debtor (root)
├── debtor-domain     # domain scaffolding
├── debtor-infra      # infrastructure scaffolding
└── debtor-web        # web scaffolding
```

## Development

```bash
cargo check
cargo test
cargo fmt
cargo clippy --fix --allow-dirty --workspace
cargo build --release
```

`cargo run` currently initializes the scaffold and exits. The complete local-run contract, including `.env` setup and `APP_ADMIN_PASSWORD_HASH`, is specified in [specs/design.md](specs/design.md).

## License

MIT OR Apache-2.0
