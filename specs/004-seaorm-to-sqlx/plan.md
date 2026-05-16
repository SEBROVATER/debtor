# Implementation Plan: Sea-ORM to SQLx Migration

**Branch**: `004-seaorm-to-sqlx` | **Date**: 2026-04-14 | **Spec**: [spec.md](spec.md)  
**Input**: Feature specification from `/specs/004-seaorm-to-sqlx/spec.md`

## Summary

Replace every Sea-ORM and sea-orm-migration call in the codebase with direct SQLx queries using `query!`/`query_as!` macros in offline mode. The schema definition moves from the Rust DSL in `migrations/src/` to a single plain SQL file under `migrations/` at the repo root, applied automatically via `sqlx::migrate!()` on application startup. All existing tests must pass without assertion changes. No schema or feature behaviour changes.

## Technical Context

**Language/Version**: Rust 1.94.1 (edition 2024)  
**Primary Dependencies**: sqlx 0.8 (sqlite + macros + chrono + rust_decimal features), acton-htmx (acton-dx 1.0.0-beta.10), argon2, chrono 0.4, rust_decimal 1, reqwest 0.12  
**Storage**: SQLite via sqlx `SqlitePool`; WAL journal mode; foreign keys enabled  
**Testing**: cargo test; unit / integration / contract suites; tempfile for test databases  
**Target Platform**: Linux server (WSL2 dev, Docker Compose deploy)  
**Project Type**: Web service (server-rendered hypermedia via acton-htmx)  
**Performance Goals**: Startup completes within the same wall-clock time as before; no regression in existing performance test timings  
**Constraints**: `SQLX_OFFLINE=true` must work with checked-in `.sqlx/` metadata; no live DB at `cargo build` time  
**Scale/Scope**: Single-user personal app; SQLite with no concurrency requirements

## Constitution Check

| Principle | Status | Notes |
|-----------|--------|-------|
| I. JS-Free Frontend | ✅ Pass | No frontend changes |
| II. Rust Backend (acton-htmx) | ✅ Pass | Remains Rust + acton-htmx; sqlx is the sanctioned data layer per constitution v3.0.0 |
| III. Vanilla CSS & Semantic HTML | ✅ Pass | No frontend changes |
| IV. Single-User Secured Access | ✅ Pass | Auth behaviour unchanged; session and auth state repos rewritten, not redesigned |
| V. Simplicity & Personal-First | ✅ Pass | Direct one-for-one replacement; `migrations/` crate deleted (simpler); no new abstractions |
| VI. TDD | ✅ Pass | All existing tests must pass; `tests/support/mod.rs` rewrite is test-infrastructure, not feature code — driven by failing integration tests |

**Gate result**: All pass. No violations to justify.

## Project Structure

### Documentation (this feature)

```text
specs/004-seaorm-to-sqlx/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
└── tasks.md             # Phase 2 output (/speckit.tasks)
```

### Source Code Changes

```text
Cargo.toml
  remove: sea-orm, sea-orm-migration, debtor-migration (path dep), time
  add: sqlx (0.8, sqlite + macros + chrono + rust_decimal features)

migrations/                         ← DELETED (entire Cargo crate)
migrations/                         ← NEW plain SQL files at repo root
└── 20260223000001_init_schema.sql  ← SQL translation of current Sea-ORM DSL

.sqlx/                              ← NEW offline query metadata (committed to repo)

src/db/
├── connection.rs    rewritten — returns SqlitePool; sets WAL + foreign_keys via SqliteConnectOptions
├── bootstrap.rs     rewritten — calls sqlx::migrate!() then inserts admin user via query!
├── mod.rs           updated  — removes `pub mod entities`
└── entities/        DELETED

src/app/state.rs     updated  — DatabaseConnection → SqlitePool

src/web/error.rs     updated  — DbErr → sqlx::Error

src/auth/
├── session_repo.rs      rewritten — query!/query_as! replacing EntityTrait calls
├── auth_state_repo.rs   rewritten — query!/query_as! replacing EntityTrait calls
└── login_service.rs     updated   — DatabaseConnection → SqlitePool

src/groups/
├── group_repo.rs    rewritten — query!/query_as! replacing EntityTrait calls
└── member_repo.rs   rewritten — query!/query_as! replacing EntityTrait calls

src/expenses/
├── expense_repo.rs  rewritten — query!/query_as! replacing EntityTrait calls
└── share_repo.rs    rewritten — query!/query_as! replacing EntityTrait calls

src/exchange_rates/
└── rate_repo.rs     rewritten — query!/query_as! replacing EntityTrait calls

src/debts/
└── debt_summary_service.rs  updated — entity types replaced with plain row structs; DbErr → sqlx::Error

tests/support/mod.rs  rewritten — seed helpers use query! directly; DatabaseConnection → SqlitePool
```

**Structure Decision**: Single Cargo workspace project. The `migrations/` path is reused for the SQL files directory; the Rust crate at that path is deleted and replaced with plain `.sql` files consumed by `sqlx::migrate!()` embedded in the main app.

## Complexity Tracking

No constitution violations requiring justification.
