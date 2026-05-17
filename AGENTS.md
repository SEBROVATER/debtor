# Repository Guidelines

## Project Overview

**debtor** is a personal, single-owner expense-sharing web application built as a Rust Cargo workspace with an Axum backend, HTMX-driven frontend, SQLite database, and zero custom JavaScript.

---

## Project Structure

```
debtor/
├── Cargo.toml                  # Workspace root + binary crate
├── src/
│   └── main.rs                 # Application entry point (composition root)
├── migrations/                 # SQLx migration files (workspace-level)
├── debtor-domain/              # Pure domain logic — zero I/O dependencies
│   └── src/
│       ├── lib.rs
│       ├── traits.rs           # Repository/service trait definitions
│       ├── debts/              # Balance calculation, debt simplification
│       ├── expenses/           # Share splitting logic
│       └── groups/             # Group membership rules
├── debtor-infra/               # Infrastructure adapters
│   └── src/
│       ├── lib.rs
│       ├── db/repos/           # SQLx repository implementations
│       ├── exchange_rates/     # Frankfurter HTTP client, caching
│       └── auth/               # Argon2id password hashing
├── debtor-web/                 # HTTP layer
│   └── src/
│       ├── lib.rs
│       ├── router.rs           # Axum route definitions
│       ├── state.rs            # AppState (shared handler context)
│       ├── handlers/           # Request handlers (auth, expenses, groups, debts)
│       ├── middleware/         # Auth, CSRF middleware
│       └── templates/          # Askama template types
│           └── partials/
├── static/css/                 # Vanilla CSS — no frameworks
├── specs/                      # Feature specs and plans (speckit workflow)
└── .env.example                # Environment variable reference
```

### Dependency Flow

```
debtor (root) → debtor-web → debtor-domain ← debtor-infra
```

- `debtor-domain` defines traits; `debtor-infra` implements them.
- `debtor-web` depends on domain traits, not infrastructure.
- Root crate wires concrete implementations at startup.

---

## Build, Test, and Development Commands

```bash
cargo check              # Fast compilation check
cargo test               # Run all tests (workspace-wide)
cargo test -p <crate>    # Run tests for a specific crate
cargo clippy --fix --allow-dirty  # Lint and auto-fix (pedantic + nursery, workspace-wide)
cargo fmt                # Format code
cargo build --release    # Production build
cargo run                # Run the application (requires .env)
```

Copy `.env.example` to `.env` and populate `APP_ADMIN_PASSWORD_HASH` before running. Generate a hash with:

```bash
echo -n "yourpassword" | argon2 somesalt -e
```

---

## Core Principles

### I. JavaScript-Free Frontend

The frontend MUST be implemented without JavaScript except for the single permitted library: [htmx](https://htmx.org). No JavaScript frameworks, bundlers, build steps, or custom JS scripts are allowed. All interactivity MUST be expressed through hypermedia (HTMX attributes, HTML forms, server responses). Browser-native behaviour is preferred over any programmatic workaround.

### II. Rust Backend (Axum)

The backend MUST be implemented in Rust using [Axum](https://github.com/tokio-rs/axum) as the HTTP framework, composed with mature, independently-maintained crates:

- **axum-htmx**: HTMX request extractors and response helpers.
- **Askama**: compile-time type-safe HTML templates.
- **tower-sessions**: cookie-based server-side session management.
- **SQLx**: compile-time verified SQL queries against SQLite.
- **argon2**: password hashing (Argon2id).

External crates MAY be used freely provided they are reliable, actively maintained, and have a meaningful user base. New technology additions SHOULD be documented in the relevant feature plan.

### III. Vanilla CSS & Modern HTML

Styling MUST use modern, vanilla CSS only. CSS frameworks (Bootstrap, Tailwind, Bulma, etc.) are NOT permitted. HTML MUST be semantic — use the correct element for the correct purpose (`<nav>`, `<main>`, `<section>`, `<article>`, `<time>`, etc.). CSS custom properties MUST be used for design tokens (colours, spacing, typography). Layouts MUST use CSS Grid or Flexbox.

Modern web platform features MUST be preferred over deprecated patterns:
- Use CSS logical properties (`margin-block`, `padding-inline`) over physical ones.
- Use `<dialog>`, `popover`, and `inert` instead of custom modal/tab implementations.
- Use `:has()`, container queries, and cascade layers where appropriate.
- Avoid legacy workarounds (e.g., clearfix hacks, float-based layouts) when native solutions exist.

### IV. Single-User Secured Access

Authentication MUST be implemented. The system MUST support exactly one user account (the owner). There MUST be no self-registration flow. All expense-related routes MUST be behind authentication. Credentials MUST be stored securely (hashed with a modern algorithm, e.g., Argon2). Sessions MUST be managed server-side with secure, HTTP-only cookies.

### V. Workspace Architecture

The project MUST be organised as a [Cargo workspace] with the binary crate at the root and three library crates:

| Crate | Responsibility |
|---|---|
| Root (`debtor`) | Binary crate; composition root that wires everything together |
| `debtor-domain` | Pure domain logic (zero I/O deps); defines repository and service traits |
| `debtor-infra` | Infrastructure adapters; implements domain traits with SQLx, reqwest, argon2 |
| `debtor-web` | HTTP layer; Axum handlers, Askama templates, middleware, routing |

Dependency flow MUST be unidirectional:

```
debtor (root) → debtor-web → debtor-domain ← debtor-infra
```

`debtor-domain` defines traits for external dependencies (repositories, providers). Infrastructure implements them. Dependency inversion uses `Arc<dyn Trait>` to keep handlers testable.

---

## Coding Style & Naming Conventions

- **Language**: Rust, edition 2024. Follow idiomatic Rust patterns throughout.
- **Formatting**: `cargo fmt` — all code must be formatted before committing.
- **Linting**: `cargo clippy --fix --allow-dirty --workspace` — workspace uses pedantic + nursery lints. Resolve all warnings; treat new warnings as errors.
- **Naming**: `snake_case` for functions, variables, and modules; `PascalCase` for types and traits.
- **Modules**: one domain concept per directory. Expose the public API through `mod.rs`.
- **Error handling**: use `thiserror`-derived error types; avoid `.unwrap()` and `.expect()` in production code paths.
- **`unsafe`**: must include a comment justifying its use.
- **Documentation**: all public items must have doc comments (`#![warn(missing_docs)]` enforced per crate).

---

## Planning Protocol

Before creating any plan, the agent MUST ask comprehensive clarifying questions to resolve undefined spots. At minimum, cover:

- **Scope boundaries**: What exactly is in-scope vs out-of-scope?
- **Error handling**: How should errors be surfaced to users? Redirect? Flash message? Error page?
- **Edge cases**: Empty states, zero amounts, single-member groups, duplicate entries, concurrent modifications.
- **Data model specifics**: Field names, types, constraints, defaults, nullable columns.
- **UI/UX expectations**: Which pages need what content? Navigation flow? Form validation UX?
- **Naming conventions**: Preferred names for domain concepts, database columns, routes.
- **Integration points**: Does this feature touch external APIs, existing services, or other features?

No plan should be created until all undefined spots are clarified. This prevents rework and ensures the plan is actionable.

---

## Documentation-First Development

Before using any Rust crate or framework, the agent MUST consult its documentation via the Context7 MCP tool. This ensures:

- Code follows the current API patterns (not outdated examples).
- Feature flags and configuration options are used correctly.
- Deprecations and breaking changes are avoided.
- Recommended idioms and best practices from the crate authors are followed.

**Workflow**:
1. Use `context7_resolve-library-id` to find the correct library ID.
2. Use `context7_query-docs` to fetch relevant documentation for the specific use case.
3. Write code based on the fetched documentation, not from memory or assumptions.

---

## Configuration & Security Notes

- All configuration is loaded from environment variables (with `.env` fallback via `dotenvy`).
- `APP_ADMIN_PASSWORD_HASH` is **required** — the app will not authenticate without it.
- Session cookies are HTTP-only and server-side managed. Set `APP_SESSION_COOKIE_SECURE=true` in production.
- See `.env.example` for all available variables and their defaults.

---

## Domain Model

### Entities

| Entity | Table | Description |
|---|---|---|
| Group | `groups` | A "room" — has a name and a target currency for debt display |
| Participant | `participants` | A person — reusable across groups; has a name and a color |
| GroupMember | `group_members` | Junction table linking participants to groups; supports soft-delete via `is_active` |
| Spending | `spendings` | An expense within a group — has description, type, amount, currency, user-chosen date |
| SpendingPayer | `spending_payers` | Records how much a participant paid toward a spending (multiple payers per spending) |
| SpendingShare | `spending_shares` | Records how much a participant owes for a spending (flexible split) |

### Relationships

```
participants 1───N group_members N───1 groups
groups        1───N spendings
spendings     1───N spending_payers   N───1 participants
spendings     1───N spending_shares   N───1 participants
```

- A participant can belong to many groups (via `group_members`).
- A group can have many participants.
- `group_members.is_active` enables soft-delete (inactive mark, reversible) or full deletion.
- A spending has one or more payers (`spending_payers`) and one or more shares (`spending_shares`).
- All payers and sharers MUST be active members of the spending's group.

### Key Constraints

- `sum(spending_payers.paid_amount) == spendings.total_amount` — payers cover the full amount.
- `sum(spending_shares.share_amount) == spendings.total_amount` — shares cover the full amount.
- `spendings.total_amount > 0` — positive amounts only.
- All monetary amounts are validated in the application layer (Rust).

---

## Spending Types

Fixed enum of 8 categories. Stored as `TEXT` in SQLite, represented as a Rust enum in application code.

| Value | Display Name |
|---|---|
| `food` | Food & Dining |
| `transport` | Transport |
| `housing` | Housing |
| `fun` | Fun & Entertainment |
| `shopping` | Shopping |
| `bills` | Bills & Utilities |
| `health` | Health |
| `other` | Other |

The Rust enum MUST implement `Display` and `FromStr` for template rendering and form parsing. Default value is `other`.

---

## Supported Currencies

Fixed set of 12 currencies. Stored as ISO 4217 code (TEXT) in SQLite, represented as a Rust enum in application code.

| Code | Name |
|---|---|
| USD | US Dollar |
| EUR | Euro |
| RUB | Russian Ruble |
| KGS | Kyrgyz Som |
| TRY | Turkish Lira |
| KZT | Kazakh Tenge |
| UZS | Uzbekistan Som |
| CNY | Chinese Yuan |
| KRW | South Korean Won |
| JPY | Japanese Yen |
| OMR | Omani Rial |
| TJS | Tajikistani Somoni |

- Each group has a `currency` field — this is the target currency for debt display.
- Each spending has its own `currency` — the currency in which the expense was made.
- Exchange rates are fetched from the [Frankfurter API](https://www.frankfurter.app) and cached.
- The Rust enum MUST implement `Display` and `FromStr`.

---

## Database Conventions

### Naming

- **Tables**: plural `snake_case` (`groups`, `participants`, `spendings`).
- **Junction tables**: named by the two entities in alphabetical order or domain order (`group_members`, `spending_payers`, `spending_shares`).
- **Columns**: `snake_case`. Foreign keys use `{referenced_table}_id` (e.g., `group_id`, `participant_id`, `spending_id`).

### Types in SQLite

| Concept | SQLite Type | Rust Type | Notes |
|---|---|---|---|
| Primary key | `INTEGER PRIMARY KEY AUTOINCREMENT` | `i64` | |
| Foreign key | `INTEGER REFERENCES ...` | `i64` | |
| Monetary amount | `TEXT` | `rust_decimal::Decimal` | SQLite has no native DECIMAL; store as string, parse in Rust |
| Boolean | `INTEGER` (0 or 1) | `bool` | SQLite has no native boolean |
| Timestamp | `TEXT` | `String` or `chrono::NaiveDateTime` | ISO 8601 format via `datetime('now')` |
| Date | `TEXT` | `String` or `chrono::NaiveDate` | YYYY-MM-DD format (user-chosen, no time component) |
| Enum/varchar | `TEXT` | Rust enum | Validated in application layer |

### Decimal Handling

All monetary amounts (`total_amount`, `paid_amount`, `share_amount`) are stored as `TEXT` in SQLite because SQLite has no native `DECIMAL`/`NUMERIC` type with exact precision. They MUST be parsed to `rust_decimal::Decimal` in Rust to avoid floating-point arithmetic errors.

**Aggregation strategy:** All numeric aggregations (SUM, AVG, MIN, MAX) on monetary amounts MUST be performed in Rust using `Decimal`, NOT in SQL. SQLite's `SUM()` over TEXT columns coerces values to `REAL` (IEEE 754 float64), which reintroduces the floating-point precision errors that TEXT storage is designed to prevent. The correct flow is: SQL fetches raw TEXT rows → Rust parses to `Decimal` → application code groups and aggregates.

**Date-based aggregation** (e.g., `GROUP BY spent_date`, `BETWEEN` on date ranges) is safe in SQL because `spent_date` uses ISO 8601 format (`YYYY-MM-DD`), which sorts correctly as plain text.

### Timestamps

Every table has `created_at` and `updated_at` columns (except junction tables):
- `created_at`: set once on insert via `DEFAULT (datetime('now'))`.
- `updated_at`: set on insert via `DEFAULT (datetime('now'))`, MUST be updated on every modification via application code (or a SQLite trigger).

### Foreign Keys

- `PRAGMA foreign_keys = ON` MUST be set on every connection (via connection string `?foreign_keys=on` or a startup query).
- All foreign keys use `ON DELETE CASCADE` to maintain referential integrity.

---

## Validation Rules

### Application-Layer Validations (enforced in Rust)

These MUST be validated in `debtor-domain` before any database write:

- `total_amount > 0` — spending amounts must be positive.
- `sum(payers.paid_amount) == spendings.total_amount` — payers must cover the full spending amount.
- `sum(shares.share_amount) == spendings.total_amount` — shares must cover the full spending amount.
- `paid_amount > 0` and `share_amount >= 0` — individual amounts must be non-negative.
- `currency` is a valid supported currency (checked against the enum).
- `spending_type` is a valid enum variant.
- All payers and sharers are active members (`is_active = 1`) of the spending's group.
- `spent_date` is a valid date in YYYY-MM-DD format.
- `color` is a valid hex color code.

### DB-Layer Validations (CHECK / NOT NULL constraints)

- All `NOT NULL` columns are enforced by SQLite.
- Composite primary keys prevent duplicate junction table entries.

---

## Debt Calculation Model

Debts are **never stored in the database**. They are computed on demand in `debtor-domain`:

1. Fetch all spendings for the group (optionally filtered by date range).
2. For each spending, convert amounts from `spending.currency` to `group.currency` using exchange rates from the Frankfurter API (cached).
3. For each participant: `net_balance = sum(paid_amount, converted) - sum(share_amount, converted)`.
4. Simplify the net balances into the minimum number of debt transfers using a min-flow / graph simplification algorithm.
5. Return the simplified debts (who owes whom, how much).

### Recalculation Triggers

Debts MUST be recalculated whenever:
- A spending is added, edited, or deleted.
- A participant is added, removed, reactivated, or fully deleted from a group.
- A group's target currency is changed.
- The user explicitly requests a refresh (e.g., to get updated exchange rates).

---

## Route Structure

All expense-related routes are behind authentication. The route hierarchy is:

```
/groups                               GET    — list all groups
/groups                               POST   — create a new group
/groups/new                           GET    — create group form
/groups/{id}                          GET    — group detail (spendings list)
/groups/{id}                          DELETE — delete group
/groups/{id}/edit                     GET    — edit group form
/groups/{id}                          PATCH  — update group (name, currency)
/groups/{id}/members                  POST   — add participant to group
/groups/{id}/members/{pid}            PATCH  — toggle active/inactive (soft delete)
/groups/{id}/members/{pid}            DELETE — fully remove participant from group
/groups/{id}/spendings                POST   — create spending (with payers + shares)
/groups/{id}/spendings/new            GET    — add spending form
/groups/{id}/spendings/{sid}          GET    — spending detail
/groups/{id}/spendings/{sid}/edit     GET    — edit spending form
/groups/{id}/spendings/{sid}          PATCH  — update spending
/groups/{id}/spendings/{sid}          DELETE — delete spending
/groups/{id}/debts                    GET    — calculated debts view
/groups/{id}/statistics               GET    — statistics dashboard (date range filter)
/participants                         GET    — list all participants
/participants                         POST   — create participant
/participants/new                     GET    — create participant form
/participants/{id}                    GET    — participant detail
/participants/{id}/edit               GET    — edit participant form
/participants/{id}                    PATCH  — update participant
/participants/{id}                    DELETE — delete participant
/login                                GET    — login form
/login                                POST   — authenticate
/logout                               POST   — end session
```
