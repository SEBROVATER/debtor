# Quickstart: Sea-ORM to SQLx Migration

**Feature**: 004-seaorm-to-sqlx  
**Date**: 2026-04-14

This document covers the developer workflow changes introduced by this migration.

---

## What Changed for Developers

| Before | After |
|--------|-------|
| `sea-orm` / `sea-orm-migration` dependencies | `sqlx` with sqlite + macros + chrono + rust_decimal |
| `migrations/` Cargo binary crate | Deleted |
| Schema applied by running `cargo run -p debtor-migration` | Schema applied automatically on app startup |
| No build-time database requirement | `.sqlx/` offline metadata committed; `SQLX_OFFLINE=true` in `.env` |
| `src/db/entities/` Sea-ORM model types | Deleted; plain row structs in each repo module |

---

## First-Time Setup (after pulling this branch)

```bash
# Copy env template if you haven't already
cp .env.example .env

# Ensure SQLX_OFFLINE=true is set in .env (added automatically by this feature)
# This allows cargo build without a live database

# Build and run — schema is applied on first startup
cargo run
```

The app will:
1. Connect to SQLite at `APP_DATABASE_URL` (default: `sqlite://debtor.db?mode=rwc`)
2. Run `sqlx::migrate!()` — applies `migrations/20260223000001_init_schema.sql`
3. Bootstrap the admin user from `APP_ADMIN_PASSWORD_HASH`

---

## Resetting the Database

No backward compatibility with Sea-ORM databases. To start fresh:

```bash
rm debtor.db debtor.db-shm debtor.db-wal 2>/dev/null; true
cargo run
```

---

## After Changing a Query

Any time a `query!` or `query_as!` macro call is added, modified, or removed, regenerate the offline metadata:

```bash
# Requires a database file at DATABASE_URL
DATABASE_URL=sqlite://debtor.db?mode=rwc cargo sqlx prepare

# Commit the updated .sqlx/ directory alongside your code change
git add .sqlx/
```

The `.sqlx/` directory must always be in sync with the queries in the codebase. `cargo build` will fail with a descriptive error if it is stale.

---

## Running Tests

Tests are unchanged from the user perspective:

```bash
cargo test                    # all suites
cargo test --test unit        # unit tests only
cargo test --test integration # integration tests only
cargo test --test contract    # contract tests only
```

Integration tests create a fresh `tempfile`-based SQLite database per test. The schema is applied via `sqlx::migrate!()` in the test `setup_test_state()` helper. No seeded `.db` file needed.

---

## Installing `sqlx-cli` (one-time, for query prep)

```bash
cargo install sqlx-cli --no-default-features --features sqlite,rustls
```

Only needed when modifying queries. Not required for building or running tests.

---

## Environment Variables

No new variables. Existing variables are unchanged:

| Variable | Purpose | Default |
|----------|---------|---------|
| `APP_DATABASE_URL` | SQLite file path | `sqlite://debtor.db?mode=rwc` |
| `APP_ADMIN_PASSWORD_HASH` | Argon2 hash for login | (required for login) |
| `APP_ADMIN_USERNAME` | Admin username | `owner` |
| `APP_SESSION_COOKIE_NAME` | Cookie name | `debtor_session` |
| `APP_SESSION_COOKIE_SECURE` | Secure cookie flag | `false` |
| `APP_EXCHANGE_BASE_URL` | Exchange rate API | `https://api.frankfurter.app` |
| `SQLX_OFFLINE` | Use `.sqlx/` metadata at build time | `true` (set in `.env`) |
| `DATABASE_URL` | Used only by `cargo sqlx prepare` | Set locally when regenerating `.sqlx/` |
