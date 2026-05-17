# debtor

A personal, single-owner expense-sharing web application built with Rust.

## Features

- **Group management** — create, edit, delete groups; each group has a name and a target currency for debt display
- **Participant management** — reusable across groups; each participant has a display name and a color for visual distinction; support for soft-delete (mark inactive, reversible) and full deletion
- **Spending tracking** — add, edit, delete spendings within a group; each spending has a description, a spending type, a user-chosen date, and a currency; multiple payers can contribute to a single spending
- **Flexible share splitting** — equal split among selected participants by default; custom override with proportions, ratios, or absolute amounts; validation ensures shares sum to the spending total
- **On-demand debt calculation** — debts are never stored; recalculated from spendings using live exchange rates (Frankfurter API); results are simplified into the minimum number of transfers
- **Statistics & analytics** — filter by custom date range; aggregate by spending type, by participant, by currency; daily, weekly, and monthly trend views; sortable tables
- **Multi-currency support** — 12 supported currencies with live conversion rates: USD, EUR, RUB, KGS, TRY, KZT, UZS, CNY, KRW, JPY, OMR, TJS
- **Authentication** — single-owner account with Argon2id password hashing and server-side HTTP-only session cookies
- **HTMX-driven UI** — zero custom JavaScript; all interactivity via HTMX attributes and hypermedia responses

## Architecture

`debtor` is a [Cargo workspace] with the binary crate at the root and three library crates:

```
debtor (root) → debtor-web → debtor-domain ← debtor-infra
```

| Crate | Purpose |
|---|---|
| Root (`debtor`) | Binary — composition root that wires concrete implementations and starts the server. |
| **debtor-domain** | Pure business logic — balance calculation, debt simplification, share splitting. Zero I/O dependencies; fully unit-testable without async runtimes. |
| **debtor-infra** | Infrastructure adapters — `SQLx` repositories, `reqwest` HTTP clients, `argon2` password hashing. Implements traits defined in `debtor-domain`. |
| **debtor-web** | HTTP layer — Axum handlers, `Askama` templates, `axum-htmx` response helpers, middleware (auth, CSRF). |

### Dependency Inversion

`debtor-domain` defines trait interfaces for external dependencies (repositories, providers). `debtor-web` handlers receive `Arc<dyn Trait>` references, enabling straightforward mocking in tests without infrastructure.

## Tech Stack

| Layer | Technology |
|---|---|
| HTTP Framework | [Axum] 0.8 |
| HTMX Integration | [axum-htmx] |
| Templates | [Askama] (compile-time type-safe) |
| Database | [SQLite] via [SQLx] (compile-time verified queries) |
| Sessions | [tower-sessions] (cookie-based, server-side) |
| Auth | [Argon2id] password hashing, HTTP-only cookies |
| Frontend | [HTMX] — zero custom JavaScript |
| Styling | Vanilla CSS (no frameworks) |
| HTTP Client | [reqwest] with rustls-tls (exchange rates) |

## Database Schema

```
┌──────────────┐       ┌──────────────────┐       ┌──────────────┐
│ participants │──────<│  group_members   │>──────│    groups    │
│              │       │                  │       │              │
│ id           │       │ group_id         │       │ id           │
│ name         │       │ participant_id   │       │ name         │
│ color        │       │ is_active        │       │ currency     │
│ created_at   │       │ joined_at        │       │ created_at   │
│ updated_at   │       └──────────────────┘       │ updated_at   │
└──────────────┘                                  └──────┬───────┘
                                                         │
                                                  ┌──────┴───────┐
                                                  │  spendings   │
                                                  │              │
                                                  │ id           │
                                                  │ group_id     │
                                                  │ description  │
                                                  │ total_amount │
                                                  │ currency     │
                                                  │ spending_type│
                                                  │ spent_date   │
                                                  │ created_at   │
                                                  │ updated_at   │
                                                  └──────┬───────┘
                                                         │
                                           ┌─────────────┴──────────────┐
                                           │                            │
                                  ┌────────┴──────────┐    ┌───────────┴──────────┐
                                  │ spending_payers   │    │   spending_shares    │
                                  │                   │    │                      │
                                  │ spending_id       │    │ spending_id          │
                                  │ participant_id    │    │ participant_id       │
                                  │ paid_amount       │    │ share_amount         │
                                  └───────────────────┘    └──────────────────────┘
```

### Tables

| Table | Purpose |
|---|---|
| `groups` | Groups (rooms) with a name and a target currency |
| `participants` | Reusable participants with a name and a color |
| `group_members` | N:M junction — links participants to groups; supports soft-delete via `is_active` |
| `spendings` | Expenses within a group — description, type, amount, currency, user-chosen date |
| `spending_payers` | Who paid how much for a spending (multiple payers per spending) |
| `spending_shares` | Who owes how much for a spending (flexible split among selected participants) |

Monetary amounts (`total_amount`, `paid_amount`, `share_amount`) are stored as `TEXT` in SQLite (which has no native `DECIMAL` type) and parsed to `rust_decimal::Decimal` in Rust for exact arithmetic. All numeric aggregations (SUM, AVG, etc.) are performed in Rust, not in SQL — SQLite's `SUM()` over TEXT coerces to `REAL` (float64), which would reintroduce the precision errors the TEXT storage is designed to prevent. Date-based aggregation (`GROUP BY spent_date`, `BETWEEN` ranges) is safe in SQL because dates use ISO 8601 format, which sorts correctly as plain text.

Debts are **not stored** — they are recalculated on demand from spendings, converted to the group's target currency via live exchange rates, and simplified into minimal transfers.

### Spending Types

Fixed set of 8 categories: `food`, `transport`, `housing`, `fun`, `shopping`, `bills`, `health`, `other`.

### Supported Currencies

USD, EUR, RUB, KGS, TRY, KZT, UZS, CNY, KRW, JPY, OMR, TJS. Exchange rates are fetched from the [Frankfurter API].

### Route Structure

```
/groups                        List all groups
/groups/new                    Create a new group
/groups/{id}                   Group detail — spendings list
/groups/{id}/edit              Edit group (name, currency, participants)
/groups/{id}/spendings/new     Add a new spending
/groups/{id}/spendings/{sid}   View spending detail
/groups/{id}/spendings/{sid}/edit  Edit spending
/groups/{id}/debts             Calculated debts view
/groups/{id}/statistics        Statistics dashboard (date range filter)
/participants                  Manage global participant pool
```

## Quick Start

```bash
# Copy environment config
cp .env.example .env

# Generate an Argon2 password hash
echo -n "yourpassword" | argon2 somesalt -e

# Edit .env and set APP_ADMIN_PASSWORD_HASH to the generated hash
$EDITOR .env

# Build and run
cargo run
```

Visit `http://localhost:3000` (default).

## Development

```bash
# Fast compile check
cargo check

# Lint and auto-fix (pedantic + nursery clippy)
cargo clippy --fix --allow-dirty

# Format
cargo fmt
```

### Project Structure

```
debtor/
├── Cargo.toml                  # Workspace root + binary crate
├── src/
│   └── main.rs                 # Application entry point (composition root)
├── migrations/                 # SQLx migrations (workspace-level)
├── debtor-domain/              # Pure domain logic
├── debtor-infra/               # Infrastructure adapters
├── debtor-web/                 # HTTP layer
├── static/css/                 # Vanilla CSS
└── specs/                      # Feature specs (speckit workflow)
```

## Principles

- **No custom JavaScript** — HTMX attributes only; no JS frameworks, bundlers, or scripts
- **Axum + composable crates** — mature, independently-maintained components (axum-htmx, Askama, SQLx, tower-sessions, argon2)
- **Modern HTML & CSS** — prefer modern web platform features over deprecated patterns (e.g., CSS logical properties, `:has()`, container queries, `<dialog>`, `popover`, `inert` attribute); avoid legacy workarounds when native solutions exist
- **Vanilla CSS & semantic HTML** — no CSS frameworks; CSS custom properties for design tokens; Grid/Flexbox layouts; correct semantic elements (`<nav>`, `<main>`, `<time>`, etc.)
- **Single-user auth** — one owner account, no self-registration, Argon2-hashed password, HTTP-only server-side sessions
- **Workspace architecture** — 4-crate Cargo workspace with unidirectional dependency flow and `Arc<dyn Trait>` dependency inversion

## License

MIT OR Apache-2.0

[Axum]: https://github.com/tokio-rs/axum
[axum-htmx]: https://github.com/robertwayne/axum-htmx
[Askama]: https://github.com/djc/askama
[SQLite]: https://sqlite.org
[SQLx]: https://github.com/launchbadge/sqlx
[tower-sessions]: https://github.com/maxcountryman/tower-sessions
[Argon2id]: https://en.wikipedia.org/wiki/Argon2
[HTMX]: https://htmx.org
[reqwest]: https://github.com/seanmonstar/reqwest
[Cargo workspace]: https://doc.rust-lang.org/cargo/reference/workspaces.html
[Frankfurter API]: https://www.frankfurter.app
