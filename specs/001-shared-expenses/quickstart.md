# Quickstart: Shared Expenses Manager

## 1. Prerequisites

- Rust toolchain (stable 1.78+)
- `cargo`
- SQLite 3
- Network access to `api.frankfurter.app` (for rate refresh)

## 2. Project bootstrap

```bash
cd /mnt/d/projects/pet/debtor
cargo init --bin .
```

Add core dependencies (planned):
- `acton-htmx`
- `tokio`
- `sea-orm`
- `sea-orm-migration`
- `argon2`
- `rust_decimal`
- `uuid`
- `time`

## 3. Environment configuration

Set required environment variables before run:

```bash
export APP_DATABASE_URL="sqlite://debtor.db?mode=rwc"
export APP_SESSION_COOKIE_NAME="debtor_session"
export APP_ADMIN_USERNAME="owner"
export APP_ADMIN_PASSWORD_HASH="<argon2id hash>"
```

## 4. TDD-first implementation flow (mandatory)

For every story and sub-feature:

1. Write a failing test first (unit/integration/contract).
2. Implement the minimal code required to pass.
3. Refactor while keeping tests green.

Never merge implementation without prior failing tests.

## 5. Suggested test sequence

1. Auth guard and login/logout session tests.
2. Lockout rule tests (5 failures -> 15-minute lockout).
3. Group and member CRUD tests.
4. Expense share validation tests (equal/percent/amount mixed mode).
5. Currency conversion and cache TTL tests.
6. Debt simplification optimality tests (subset-DP correctness).
7. Full integration path tests for each P1 story.

## 6. Run tests and app

```bash
cargo test
cargo run
```

## 7. Manual acceptance checks

- Unauthenticated access redirects to `/login`.
- Correct credentials reach dashboard.
- Failed credentials remain on login page with error.
- Group/member/expense CRUD updates debt summaries immediately.
- Multi-currency summary uses cached rates and shows stale warning on fallback.
- No route uses custom JavaScript; only HTMX attributes are present in templates.

## 8. Definition of done

- Constitution gates remain PASS.
- All acceptance scenarios mapped to automated tests.
- `cargo test` passes locally.
- Route contracts in `/mnt/d/projects/pet/debtor/specs/001-shared-expenses/contracts/`
  remain aligned with implementation.
