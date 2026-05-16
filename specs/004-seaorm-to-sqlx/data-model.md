# Data Model: Sea-ORM to SQLx Migration

**Feature**: 004-seaorm-to-sqlx  
**Date**: 2026-04-14

The schema is **unchanged**. This document describes the plain Rust row structs that replace Sea-ORM entity `Model` types, and the SQL schema used in the new migration file.

---

## Plain Rust Row Structs

Each struct replaces the corresponding `src/db/entities/<name>::Model`. These structs live in the consuming module (or in `src/db/`) and are mapped by `query_as!`.

### AdminUserRow

Replaces `db::entities::admin_users::Model`

```rust
#[derive(Debug, Clone)]
pub struct AdminUserRow {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
```

### AuthStateRow

Replaces `db::entities::auth_state::Model`

```rust
#[derive(Debug, Clone)]
pub struct AuthStateRow {
    pub id: i64,
    pub failed_attempt_count: i64,
    pub lockout_until: Option<NaiveDateTime>,
    pub last_failed_at: Option<NaiveDateTime>,
    pub updated_at: NaiveDateTime,
}
```

### SessionRow

Replaces `db::entities::sessions::Model`

```rust
#[derive(Debug, Clone)]
pub struct SessionRow {
    pub id: String,
    pub user_id: i64,
    pub token_hash: String,
    pub created_at: NaiveDateTime,
    pub last_seen_at: NaiveDateTime,
    pub expires_at: NaiveDateTime,
    pub revoked_at: Option<NaiveDateTime>,
}
```

### GroupRow

Replaces `db::entities::groups::Model`

```rust
#[derive(Debug, Clone)]
pub struct GroupRow {
    pub id: String,
    pub name: String,
    pub target_currency: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
```

### MemberRow

Replaces `db::entities::members::Model`

```rust
#[derive(Debug, Clone)]
pub struct MemberRow {
    pub id: String,
    pub group_id: String,
    pub display_name: String,
    pub is_active: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub removed_at: Option<NaiveDateTime>,
}
```

### ExpenseRow

Replaces `db::entities::expenses::Model`

```rust
#[derive(Debug, Clone)]
pub struct ExpenseRow {
    pub id: String,
    pub group_id: String,
    pub payer_member_id: String,
    pub amount: Decimal,
    pub currency: String,
    pub note: Option<String>,
    pub expense_date: NaiveDate,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
```

### ExpenseShareRow

Replaces `db::entities::expense_shares::Model`

```rust
#[derive(Debug, Clone)]
pub struct ExpenseShareRow {
    pub id: String,
    pub expense_id: String,
    pub member_id: String,
    pub share_mode: String,
    pub share_value: Decimal,
    pub computed_amount: Decimal,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
```

### ExchangeRateRow

Replaces `db::entities::exchange_rates::Model`

```rust
#[derive(Debug, Clone)]
pub struct ExchangeRateRow {
    pub id: String,
    pub from_currency: String,
    pub to_currency: String,
    pub rate: Decimal,
    pub fetched_at: NaiveDateTime,
    pub rate_date: NaiveDate,
    pub provider: String,
}
```

---

## Struct Placement

Each row struct lives in the same module as its repo (not in a shared `db::entities` module):

| Struct | Location |
|--------|----------|
| `AdminUserRow` | `src/auth/login_service.rs` or new `src/auth/row_types.rs` |
| `AuthStateRow` | `src/auth/auth_state_repo.rs` |
| `SessionRow` | `src/auth/session_repo.rs` |
| `GroupRow` | `src/groups/group_repo.rs` |
| `MemberRow` | `src/groups/member_repo.rs` |
| `ExpenseRow` | `src/expenses/expense_repo.rs` |
| `ExpenseShareRow` | `src/expenses/share_repo.rs` |
| `ExchangeRateRow` | `src/exchange_rates/rate_repo.rs` |

Public re-exports through `mod.rs` files mirror the current pattern where entity models are imported across modules (e.g., `debt_summary_service.rs` imports `expenses::Model` — it will import `ExpenseRow` and `ExpenseShareRow` instead).

---

## SQL Schema (`migrations/20260223000001_init_schema.sql`)

Direct SQL translation of the current Sea-ORM migration DSL. Decimal columns use `TEXT` to store `rust_decimal::Decimal` values exactly.

```sql
CREATE TABLE IF NOT EXISTS admin_users (
    id            INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    username      TEXT    NOT NULL UNIQUE,
    password_hash TEXT    NOT NULL,
    created_at    TEXT    NOT NULL,
    updated_at    TEXT    NOT NULL
);

CREATE TABLE IF NOT EXISTS auth_state (
    id                    INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    failed_attempt_count  INTEGER NOT NULL DEFAULT 0,
    lockout_until         TEXT,
    last_failed_at        TEXT,
    updated_at            TEXT    NOT NULL
);

CREATE TABLE IF NOT EXISTS sessions (
    id           TEXT    PRIMARY KEY NOT NULL,
    user_id      INTEGER NOT NULL REFERENCES admin_users(id) ON DELETE CASCADE,
    token_hash   TEXT    NOT NULL UNIQUE,
    created_at   TEXT    NOT NULL,
    last_seen_at TEXT    NOT NULL,
    expires_at   TEXT    NOT NULL,
    revoked_at   TEXT
);

CREATE TABLE IF NOT EXISTS groups (
    id              TEXT PRIMARY KEY NOT NULL,
    name            TEXT NOT NULL,
    target_currency TEXT NOT NULL,
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS members (
    id           TEXT    PRIMARY KEY NOT NULL,
    group_id     TEXT    NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
    display_name TEXT    NOT NULL,
    is_active    INTEGER NOT NULL DEFAULT 1,
    created_at   TEXT    NOT NULL,
    updated_at   TEXT    NOT NULL,
    removed_at   TEXT
);

CREATE TABLE IF NOT EXISTS expenses (
    id               TEXT PRIMARY KEY NOT NULL,
    group_id         TEXT NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
    payer_member_id  TEXT NOT NULL REFERENCES members(id) ON DELETE RESTRICT,
    amount           TEXT NOT NULL,
    currency         TEXT NOT NULL,
    note             TEXT,
    expense_date     TEXT NOT NULL,
    created_at       TEXT NOT NULL,
    updated_at       TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS expense_shares (
    id              TEXT PRIMARY KEY NOT NULL,
    expense_id      TEXT NOT NULL REFERENCES expenses(id) ON DELETE CASCADE,
    member_id       TEXT NOT NULL REFERENCES members(id) ON DELETE RESTRICT,
    share_mode      TEXT NOT NULL,
    share_value     TEXT NOT NULL,
    computed_amount TEXT NOT NULL,
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS exchange_rates (
    id            TEXT PRIMARY KEY NOT NULL,
    from_currency TEXT NOT NULL,
    to_currency   TEXT NOT NULL,
    rate          TEXT NOT NULL,
    fetched_at    TEXT NOT NULL,
    rate_date     TEXT NOT NULL,
    provider      TEXT NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_exchange_rates_unique
    ON exchange_rates (from_currency, to_currency, rate_date);
```

---

## Type Mapping Reference

| Rust type | SQLite storage | SQLx feature |
|-----------|---------------|--------------|
| `String` | TEXT | built-in |
| `i64` | INTEGER | built-in |
| `bool` | INTEGER (0/1) | built-in (sqlite feature) |
| `NaiveDateTime` | TEXT (ISO 8601) | `chrono` feature |
| `NaiveDate` | TEXT (ISO 8601) | `chrono` feature |
| `rust_decimal::Decimal` | TEXT | `rust_decimal` feature |
| `Option<T>` | NULL-able column | built-in |

> **Note on `id` integer type**: Sea-ORM used `i32` for `admin_users.id` and `auth_state.id`. SQLx with SQLite returns `INTEGER` columns as `i64`. The row structs use `i64` accordingly. All call sites that used `i32` (e.g., `sessions.user_id: i32`) are updated to `i64`.
