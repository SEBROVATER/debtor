# Repository Guidelines

## Project Overview

**debtor** is a personal, single-owner expense-sharing web application built with a Rust backend, HTMX-driven frontend, SQLite database, and zero custom JavaScript. All development must comply with the [project constitution](.specify/memory/constitution.md).

---

## Project Structure

```
debtor/
├── src/                  # Application source code
│   ├── main.rs           # Entry point
│   ├── lib.rs            # Library root (re-exports modules)
│   ├── app/              # Config and shared application state
│   ├── auth/             # Session management, login, password hashing
│   ├── db/               # Database connection, bootstrap, ORM entities
│   ├── debts/            # Balance calculation and debt simplification
│   ├── exchange_rates/   # Frankfurter API client and caching
│   ├── expenses/         # Expense CRUD, share splitting
│   ├── groups/           # Group and member management
│   └── web/              # HTTP router, handlers, CSRF, HTML templates
├── migrations/           # sea-orm-migration crate (schema migrations)
├── tests/
│   ├── unit/             # Pure logic tests (no I/O)
│   ├── integration/      # Full-stack tests against an in-memory SQLite DB
│   ├── contract/         # Behavioural contract tests for services
│   └── support/mod.rs    # Shared test helpers (setup_test_state, etc.)
├── static/css/           # Vanilla CSS — no frameworks
├── specs/                # Feature specs and plans (speckit workflow)
└── .env.example          # Environment variable reference
```

---

## Build, Test, and Development Commands

```bash
# Check compilation without producing binaries (fast feedback)
cargo check

# Run all tests (unit, integration, contract)
cargo test

# Run only unit tests
cargo test --test unit

# Run only integration tests
cargo test --test integration

# Run only contract tests
cargo test --test contract

# Build a release binary
cargo build --release

# Run the application (requires .env or environment variables)
cargo run
```

Copy `.env.example` to `.env` and populate `APP_ADMIN_PASSWORD_HASH` before running. Generate a hash with:

```bash
echo -n "yourpassword" | argon2 somesalt -e
```

---

## Coding Style & Naming Conventions

- **Language**: Rust, edition 2024. Follow idiomatic Rust patterns throughout.
- **Formatting**: `cargo fmt` — all code must be formatted before committing.
- **Linting**: `cargo clippy` — resolve all warnings; treat new warnings as errors.
- **Naming**: `snake_case` for functions, variables, and modules; `PascalCase` for types and traits.
- **Modules**: one domain concept per directory (e.g., `expenses/`, `groups/`). Expose the public API through `mod.rs`.
- **Error handling**: use `thiserror`-derived error types; avoid `.unwrap()` in production code paths.
- **`unsafe`**: must include a comment justifying its use.
- **Frontend**: HTMX attributes only — no custom JavaScript, no CSS frameworks.

---

## Testing Guidelines

Tests are written using Rust's built-in test framework (`#[test]`, `#[tokio::test]`).

**TDD is mandatory.** Follow Red → Green → Refactor strictly. Tests must be written before implementation; no feature is complete without prior-written tests.

| Suite | Location | Purpose |
|---|---|---|
| Unit | `tests/unit/` | Pure logic, no I/O |
| Integration | `tests/integration/` | Full stack against a temporary SQLite DB |
| Contract | `tests/contract/` | Behavioural contracts for service interfaces |

**Conventions:**
- Test files are named `test_<subject>.rs` (e.g., `test_balance_calculator.rs`).
- Test functions use descriptive `snake_case` names that read as sentences (e.g., `aggregates_balances_across_expenses`).
- Use helpers from `tests/support/mod.rs` (`setup_test_state`, `temp_sqlite_url`, `hash_password`) to avoid boilerplate.
- Each test must be independent — no shared mutable state between tests.

---

## Commit & Pull Request Guidelines

Commits follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add expense deletion endpoint
fix: correct rounding in share splitter
docs: amend constitution to v2.3.0 (add TDD principle)
refactor: extract balance calculator into separate module
test: add contract tests for group member service
chore: update sea-orm to 1.1
```

- Use the **imperative mood** in the subject line.
- Keep the subject line under 72 characters.
- Reference a spec when relevant (e.g., `feat(specs/001): implement expense creation`).

Pull requests must:
- Include a clear description of what changed and why.
- Pass `cargo fmt --check`, `cargo clippy`, and `cargo test` without errors.
- Contain tests written before the implementation (TDD — no after-the-fact test additions).

---

## Configuration & Security Notes

- All configuration is loaded from environment variables (with `.env` fallback via `dotenvy`).
- `APP_ADMIN_PASSWORD_HASH` is **required** — the app will not authenticate without it.
- Session cookies are HTTP-only and server-side managed. Set `APP_SESSION_COOKIE_SECURE=true` in production.
- See `.env.example` for all available variables and their defaults.
