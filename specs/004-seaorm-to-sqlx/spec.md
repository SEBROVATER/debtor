# Feature Specification: Sea-ORM to SQLx Migration

**Feature Branch**: `004-seaorm-to-sqlx`  
**Created**: 2026-04-14  
**Status**: Draft  
**Input**: User description: "Continue specifying migration task for SeaORM to pure SQLx stack"

## Clarifications

### Session 2026-04-14

- Q: How should the new migration runner handle pre-existing databases that were managed by Sea-ORM? → A: No backward compatibility required. Operators must delete the existing database file; the new migration runner creates a fresh schema from scratch.
- Q: Which compile-time query verification approach should be used? → A: `query!` / `query_as!` macros with a checked-in `sqlx-data.json` offline metadata file; no live database required at build time.
- Q: Where should the SQLx migration runner live after the `migrations/` crate is removed? → A: Delete the `migrations/` crate entirely; embed `sqlx::migrate!()` in the main application startup using plain `.sql` files in a `migrations/` directory at the repo root.
- Q: How should monetary decimal values be stored in SQLite with the new data access layer? → A: Store as `TEXT` via `rust_decimal::Decimal` using the `rust_decimal` feature on `sqlx` — exact representation, matching current Sea-ORM storage behaviour.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Application Continues to Work After Migration (Priority: P1)

As the application owner, the entire app — login, expense management, group management, exchange rate caching — continues to function correctly after the data access layer is replaced. No data is lost, no features are degraded, and all existing tests pass.

**Why this priority**: This is the definition of a successful migration. Every other story is subordinate to it. If the app is broken at the end, the migration has failed.

**Independent Test**: Can be fully tested by running the full test suite and manually exercising the golden paths (login → create group → add members → record expense → view debt summary) and confirming all pass.

**Acceptance Scenarios**:

1. **Given** the application has been migrated, **When** the full automated test suite is run, **Then** all unit, integration, and contract tests pass with zero failures.
2. **Given** a fresh database, **When** the application starts, **Then** all tables are created automatically with the correct schema (columns, types, constraints, foreign keys, indexes).
3. **Given** a Sea-ORM–managed database file exists, **When** the operator wants to run the migrated application, **Then** the operator deletes the old database file and starts fresh — no automatic data migration or compatibility layer is required.

---

### User Story 2 - Sea-ORM Dependency Completely Removed (Priority: P2)

As the application owner, the project no longer depends on Sea-ORM or sea-orm-migration. The dependency tree is leaner and the codebase aligns with the technology stack defined in the project constitution (v3.0.0).

**Why this priority**: The constitution mandates SQLx. Partial migration leaves the project in an inconsistent state and is harder to maintain than either the old or new approach fully applied. This story is complete only when `sea-orm` and `sea-orm-migration` are fully absent from `Cargo.toml`.

**Independent Test**: Can be fully tested by confirming `sea-orm` and `sea-orm-migration` no longer appear in `Cargo.toml` or `Cargo.lock`, and that `cargo build` succeeds.

**Acceptance Scenarios**:

1. **Given** the migration is complete, **When** `Cargo.toml` is inspected, **Then** neither `sea-orm` nor `sea-orm-migration` are listed as dependencies.
2. **Given** the migration is complete, **When** the project is compiled from scratch, **Then** it compiles without errors or warnings related to the removed dependencies.

---

### User Story 3 - Schema Migrations Managed Without Sea-ORM (Priority: P3)

As the application owner, database schema creation and future schema changes are managed using a plain SQL migration approach compatible with the new data access layer. The `migrations/` crate, which currently uses `sea-orm-migration`, is replaced or rewritten.

**Why this priority**: The migration crate (`migrations/`) currently depends on `sea-orm-migration` and produces Sea-ORM DSL. It must be replaced for the project to be fully Sea-ORM-free. However, this is lower priority than the core app working — a functional app with a manual schema bootstrap is better than a broken app with a perfect migration runner.

**Independent Test**: Can be tested by deleting the SQLite database file, starting the application, and confirming all tables are created correctly with the expected schema.

**Acceptance Scenarios**:

1. **Given** no database file exists, **When** the application starts, **Then** the schema is applied automatically and the app is usable immediately.
2. **Given** the schema migration tooling has been replaced, **When** a new migration file is added in the future, **Then** it runs automatically on the next application start.
3. **Given** the migration is complete, **When** `migrations/Cargo.toml` is inspected, **Then** `sea-orm-migration` is not listed as a dependency.

---

### Edge Cases

- What happens when a query returns no rows — does the new data access layer handle empty result sets as gracefully as Sea-ORM did?
- How does the system handle decimal precision for monetary amounts (`amount`, `share_value`, `computed_amount`) — the new layer must preserve the same precision without silent rounding.
- What happens if a migration is partially applied when the process is interrupted — does the schema remain consistent on the next startup?
- How are nullable datetime fields (e.g., `lockout_until`, `last_failed_at`, `revoked_at`, `removed_at`) handled in query results?
- If a pre-existing SQLite database (from a Sea-ORM deployment) is present when the new application starts, no automatic compatibility is guaranteed — the operator must delete the existing database file and let the new migration runner create a fresh schema.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: All existing CRUD operations for groups, members, expenses, expense shares, sessions, auth state, and exchange rates MUST be re-implemented using the new data access layer.
- **FR-002**: The new data access layer MUST use `query!` / `query_as!` macros for compile-time verified queries. A `sqlx-data.json` offline metadata file MUST be checked into the repository so that `cargo build` succeeds without a live database connection. This file MUST be regenerated via `cargo sqlx prepare` whenever queries change.
- **FR-003**: The database schema MUST be applied automatically on application startup with no manual step required.
- **FR-004**: All existing integration tests MUST continue to pass without modification to their assertions (test helper infrastructure may change).
- **FR-005**: The project MUST compile with zero errors and zero Clippy warnings after the migration.
- **FR-006**: Monetary decimal values (`amount`, `share_value`, `computed_amount`, `rate`) MUST be stored as `TEXT` in SQLite via `rust_decimal::Decimal` and retrieved without truncation or rounding. The `rust_decimal` feature on the `sqlx` crate MUST be used to enable this mapping.
- **FR-007**: All foreign key constraints and cascade behaviours defined in the current schema MUST be preserved in the new schema definition.
- **FR-008**: The `migrations/` Cargo crate MUST be deleted entirely. Plain `.sql` migration files MUST be placed in a `migrations/` directory at the repo root, and `sqlx::migrate!()` MUST be embedded in the main application's startup sequence to apply pending migrations automatically.
- **FR-009**: The `src/db/entities/` directory of Sea-ORM entity models MUST be removed once all consumers are migrated.

### Key Entities

- **Group**: Expense-sharing group with a target currency. Has many Members and Expenses.
- **Member**: Named participant within a Group. Has many ExpenseShares. Has a soft-delete timestamp.
- **Expense**: A recorded payment by one Member (payer) within a Group. Has many ExpenseShares.
- **ExpenseShare**: Per-member allocation of an Expense, storing share mode, share value, and computed amount.
- **Session**: Authenticated session for the single admin user. Linked to AdminUser.
- **AdminUser**: The single owner account with hashed password credentials.
- **AuthState**: Tracks failed login attempts and lockout status.
- **ExchangeRate**: Cached currency conversion rate with provider, date, and fetch timestamp.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of the existing automated tests (unit, integration, contract) pass after the migration, with no tests deleted or skipped to achieve this.
- **SC-002**: The project dependency list no longer includes Sea-ORM or sea-orm-migration — verified by inspecting the lockfile.
- **SC-003**: A fresh application start against an empty database produces a fully usable application within the same startup time as before the migration.
- **SC-004**: All monetary values stored and retrieved through the new layer match their original values exactly — no precision loss across the full test suite.
- **SC-005**: The project compiles cleanly with zero warnings on `cargo clippy` after the migration is complete.

## Assumptions

- The database schema itself (tables, columns, types, constraints) does not change — this is a data access layer swap, not a schema redesign.
- No backward compatibility with existing Sea-ORM–managed databases is required. Operators must delete the existing database file and allow the new migration runner to create a fresh schema from scratch.
- The `sqlx` crate with `sqlite` feature and `macros` will be used with `query!` / `query_as!` macros in offline mode. A `sqlx-data.json` file is checked into the repository and regenerated locally via `cargo sqlx prepare` after any query change. No live `DATABASE_URL` is required at build time.
- The `migrations/` Cargo crate will be deleted. Plain `.sql` migration files will live in a `migrations/` directory at the repo root. `sqlx::migrate!()` will be called during main application startup to apply them automatically.
- Monetary decimal values (`amount`, `share_value`, `computed_amount`, `rate`) MUST be stored as `TEXT` in SQLite via `rust_decimal::Decimal`, using the `rust_decimal` feature on the `sqlx` crate. This matches the current Sea-ORM storage format exactly and guarantees no precision loss.
- The single-user, personal-use nature of the project means zero-downtime migration of a live production database is not a requirement — a clean restart is acceptable.
- Spec artifacts from feature 003 (plan.md, research.md, data-model.md, etc.) that reference Sea-ORM are out of scope for this feature — they are updated separately as noted in the constitution sync impact report.
