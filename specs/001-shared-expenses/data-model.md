# Data Model: Shared Expenses Manager

## Entity Overview

| Entity | Purpose |
|--------|---------|
| `admin_users` | Stores the single owner account credentials. |
| `auth_state` | Tracks failed logins and lockout state. |
| `sessions` | Server-side session lifecycle for authenticated access. |
| `groups` | Top-level expense-sharing containers with target currency. |
| `members` | Group participants (active/inactive). |
| `expenses` | Expense records with payer, currency, note, and date. |
| `expense_shares` | Per-participant share definitions and computed amounts. |
| `exchange_rates` | Cached currency pair rates with fetch date and source metadata. |

## Entities

### `admin_users`

- `id` (`INTEGER`, PK, must be `1`)
- `username` (`TEXT`, unique, 3..64 chars)
- `password_hash` (`TEXT`, Argon2id hash string)
- `created_at` (`DATETIME`)
- `updated_at` (`DATETIME`)

Validation rules:
- Exactly one row is allowed.
- No registration flow; row created via bootstrap/migration or secure setup command.

### `auth_state`

- `id` (`INTEGER`, PK, must be `1`)
- `failed_attempt_count` (`INTEGER`, default `0`)
- `lockout_until` (`DATETIME`, nullable)
- `last_failed_at` (`DATETIME`, nullable)
- `updated_at` (`DATETIME`)

Validation rules:
- `failed_attempt_count >= 0`.
- When count reaches `5`, `lockout_until = now + 15 minutes`.
- Counter resets to `0` on successful login.

### `sessions`

- `id` (`TEXT`, UUID, PK)
- `user_id` (`INTEGER`, FK -> `admin_users.id`)
- `token_hash` (`TEXT`, unique)
- `created_at` (`DATETIME`)
- `last_seen_at` (`DATETIME`)
- `expires_at` (`DATETIME`)
- `revoked_at` (`DATETIME`, nullable)

Validation rules:
- Sliding expiry: each authenticated request sets `expires_at = now + 30 days`.
- Logout sets `revoked_at` immediately.
- Auth guard rejects expired or revoked sessions.

### `groups`

- `id` (`TEXT`, UUID, PK)
- `name` (`TEXT`, 1..80 chars)
- `target_currency` (`TEXT`, ISO-4217 uppercase, length `3`)
- `created_at` (`DATETIME`)
- `updated_at` (`DATETIME`)

Validation rules:
- Name required and trimmed.
- Target currency must be supported by exchange provider.

### `members`

- `id` (`TEXT`, UUID, PK)
- `group_id` (`TEXT`, FK -> `groups.id`, cascade delete)
- `display_name` (`TEXT`, 1..80 chars)
- `is_active` (`BOOLEAN`, default `1`)
- `created_at` (`DATETIME`)
- `updated_at` (`DATETIME`)
- `removed_at` (`DATETIME`, nullable)

Validation rules:
- Unique active member name within a group (case-insensitive).
- Removal is soft (`is_active = 0`) to preserve history references.

### `expenses`

- `id` (`TEXT`, UUID, PK)
- `group_id` (`TEXT`, FK -> `groups.id`, cascade delete)
- `payer_member_id` (`TEXT`, FK -> `members.id`)
- `amount` (`DECIMAL`, `> 0`)
- `currency` (`TEXT`, ISO-4217 uppercase, length `3`)
- `note` (`TEXT`, nullable, max 500 chars)
- `expense_date` (`DATE`)
- `created_at` (`DATETIME`)
- `updated_at` (`DATETIME`)

Validation rules:
- `payer_member_id` must belong to the same `group_id`.
- At least one participant share must exist.
- Original amount/currency are immutable historical facts unless expense is edited.

### `expense_shares`

- `id` (`TEXT`, UUID, PK)
- `expense_id` (`TEXT`, FK -> `expenses.id`, cascade delete)
- `member_id` (`TEXT`, FK -> `members.id`)
- `share_mode` (`TEXT`, enum: `equal|percent|amount`)
- `share_value` (`DECIMAL`, interpretation depends on mode)
- `computed_amount` (`DECIMAL`, finalized share amount in expense currency)
- `created_at` (`DATETIME`)
- `updated_at` (`DATETIME`)

Validation rules:
- At least one share row per expense.
- `percent` rows: `0 < share_value <= 100`.
- `amount` rows: `0 < share_value <= expense.amount`.
- Sum of all `computed_amount` values must equal `expenses.amount`.
- Mixed-mode support:
  - explicit `amount` shares reserved first,
  - explicit `percent` shares computed next from total,
  - remaining amount split equally across `equal` rows.

### `exchange_rates`

- `id` (`TEXT`, UUID, PK)
- `from_currency` (`TEXT`, ISO-4217 uppercase, length `3`)
- `to_currency` (`TEXT`, ISO-4217 uppercase, length `3`)
- `rate` (`DECIMAL`, `> 0`)
- `fetched_at` (`DATETIME`)
- `rate_date` (`DATE`)  
- `provider` (`TEXT`, e.g., `frankfurter`)

Indexes/constraints:
- Unique key on (`from_currency`, `to_currency`, `rate_date`).

Validation rules:
- Use latest available rate for conversion.
- If refresh fails, use newest cached row and emit stale-rate warning.

## Relationships

- `admin_users (1) -> (many) sessions`
- `groups (1) -> (many) members`
- `groups (1) -> (many) expenses`
- `expenses (1) -> (many) expense_shares`
- `members (1) -> (many) expense_shares`
- `members (1) -> (many) expenses` through `payer_member_id`

## Derived Read Models (Not Persisted)

### `member_balances`

- Derived per group in target currency.
- `balance > 0`: creditor.
- `balance < 0`: debtor.

### `debt_transactions`

- Minimal directed transfers generated from subset-DP simplification.
- Fields: `from_member_id`, `to_member_id`, `amount_target_currency`.

### `group_summary_view`

- Group metadata + computed debts + warnings:
  - `no_outstanding_debts`
  - `rates_stale_warning`
  - `conversion_blocked_no_cache`

## State Transitions

### Authentication State

- `UNLOCKED` -> `LOCKED` after 5 consecutive failures.
- `LOCKED` -> `UNLOCKED` when `lockout_until <= now` or on successful login after lockout period.

### Session State

- `ACTIVE` -> `ACTIVE` on authenticated request (extends expiry).
- `ACTIVE` -> `EXPIRED` when `expires_at < now`.
- `ACTIVE` -> `REVOKED` on logout.

### Member State

- `ACTIVE` -> `INACTIVE` on remove action.
- `INACTIVE` remains referenceable by historical expenses.

### Expense Lifecycle

- `CREATED` -> `UPDATED` on edit.
- `CREATED/UPDATED` -> `DELETED` on removal.
- Any transition triggers debt recalculation.

### Exchange Rate Freshness

- `FRESH` when cached entry age <= 24h.
- `STALE` when cached entry age > 24h.
- `MISSING` when no row exists for required pair/date.
