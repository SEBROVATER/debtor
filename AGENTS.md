# Agent Guide

Read [specs/design.md](specs/design.md) before feature work. It is the product and architecture source of truth.

## Crate Boundaries

Preserve `root -> web/infra -> application -> domain`. Domain owns pure rules; application owns use cases and ports; infra owns external adapters; web owns HTTP; root only composes. Keep outer framework, SQLx, reqwest, and Argon2 types out of inner crates. Use narrow injected ports and keep handlers thin.

## Non-Negotiable Rules

- Use exact `Decimal`, canonical TEXT persistence, minor-unit validation, and Rust monetary aggregation. Never use floating point or SQL monetary aggregates.
- Preserve history: archive referenced identities, restrict destructive participant cascades, and write spending aggregates transactionally.
- Use server-rendered Askama/HTMX, vanilla CSS, semantic HTML, and no custom JavaScript.
- Protect unsafe routes with authentication and session-backed CSRF. Never log credentials, hashes, session IDs, or tokens.
- Use compile-time checked SQLx queries and refresh committed `.sqlx` metadata whenever SQL or migrations change.
- Consult current crate documentation through Context7 before changing framework or library APIs.
- Keep use cases testable with fakes; test infrastructure adapters separately and retain a composed startup smoke test.
- Never use `cargo build --release` for testing, validation, checks, or routine development. Use debug `cargo check`, `cargo test`, and `cargo run` only.
- Update `specs/design.md` first when behavior changes, then synchronize README, config examples, migrations, tests, and SQLx metadata.
- Prefer the smallest correct change. Do not overwrite unrelated worktree changes or add compatibility paths without a concrete consumer.
- Debtor is permanently single-administrator. Participants are accounting identities; do not add user, tenant, registration, participant-authentication, or multi-user authorization abstractions.
- Before first deployment, breaking Rust APIs, configuration, routes, and database schemas are allowed when they produce a cleaner architecture. Remove superseded paths rather than keeping shims; security, accounting, and historical-integrity invariants remain mandatory.

## Commands

```bash
cargo check --workspace --all-features --locked
cargo test --workspace --all-features --locked
cargo test -p <crate>
cargo fmt --all -- --check
SQLX_OFFLINE=true cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo clippy --fix --allow-dirty --workspace # optional autofix; review changes
cargo run
```

The password helper is an independent workspace. Validate it with `cargo fmt --manifest-path tools/password-hash/Cargo.toml -- --check`, `cargo clippy --manifest-path tools/password-hash/Cargo.toml --all-targets --all-features --locked -- -D warnings`, and `cargo test --manifest-path tools/password-hash/Cargo.toml --locked`.

When checked SQL or migrations change, migrate a temporary SQLite database and run `SQLX_OFFLINE=false DATABASE_URL=sqlite:///tmp/debtor-sqlx.db?mode=rwc cargo sqlx prepare --workspace --check`.

Copy `.env.example` to `.env` and set `APP_ADMIN_PASSWORD_HASH` before running. Generate a hash with `cargo run --manifest-path tools/password-hash/Cargo.toml`.
