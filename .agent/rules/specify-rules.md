# debtor Development Guidelines

Auto-generated from all feature plans. Last updated: 2026-04-14

## Active Technologies
- Rust 1.94.1 (edition 2024) + sqlx 0.8 (sqlite + macros + chrono + rust_decimal features), acton-htmx (acton-dx 1.0.0-beta.10), argon2, chrono 0.4, rust_decimal 1, reqwest 0.12 (004-seaorm-to-sqlx)
- SQLite via sqlx `SqlitePool`; WAL journal mode; foreign keys enabled (004-seaorm-to-sqlx)

- Rust stable 1.90+ (Edition 2024) + `acton-htmx`, `tokio`, `sea-orm`, `sea-orm-migration`, `argon2`, `rust_decimal`, `time`, `uuid`, browser `htmx` (001-shared-expenses)

## Project Structure

```text
backend/
frontend/
tests/
```

## Commands

cargo test [ONLY COMMANDS FOR ACTIVE TECHNOLOGIES][ONLY COMMANDS FOR ACTIVE TECHNOLOGIES] cargo clippy

## Code Style

Rust stable 1.78+ (Edition 2024): Follow standard conventions

## Recent Changes
- 004-seaorm-to-sqlx: Added Rust 1.94.1 (edition 2024) + sqlx 0.8 (sqlite + macros + chrono + rust_decimal features), acton-htmx (acton-dx 1.0.0-beta.10), argon2, chrono 0.4, rust_decimal 1, reqwest 0.12

- 001-shared-expenses: Added Rust stable 1.78+ (Edition 2024) + `acton-htmx`, `tokio`, `sea-orm`, `sea-orm-migration`, `argon2`, `rust_decimal`, `time`, `uuid`, browser `htmx`

<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->
