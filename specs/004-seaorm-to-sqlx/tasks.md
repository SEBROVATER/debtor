# Tasks: Sea-ORM to SQLx Migration

**Input**: Design documents from `/specs/004-seaorm-to-sqlx/`  
**Prerequisites**: plan.md ✓, spec.md ✓, research.md ✓, data-model.md ✓, quickstart.md ✓

**TDD note**: The existing test suite (unit / integration / contract) serves as the red/green indicator for this migration. The Red phase is Phase 2 (Foundation switches AppState type, breaking all repo compilation). The Green phase is Phase 3 (all repos migrated, `cargo test` passes). No new test files are introduced — TDD here means the existing tests must not be modified and must pass at the end.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no shared dependencies in flight)
- **[Story]**: Which user story this task belongs to

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Add new dependencies and create the SQL schema file. No breaking changes — `cargo check` still passes after this phase.

- [ ] T001 Add `sqlx = { version = "0.8", features = ["sqlite", "runtime-tokio-rustls", "macros", "chrono", "rust_decimal"] }` to `Cargo.toml`; remove `debtor_migration` path dependency from `[dependencies]`
- [ ] T002 Add `SQLX_OFFLINE=true` and `DATABASE_URL=sqlite://debtor.db?mode=rwc` to `.env.example` (DATABASE_URL is used only by `cargo sqlx prepare`)
- [ ] T003 Create `migrations/20260223000001_init_schema.sql` — SQL translation of current Sea-ORM DSL (see `data-model.md` for full SQL). All decimal columns as `TEXT NOT NULL`. Include WAL-safe `IF NOT EXISTS` guards and the unique index on `exchange_rates`.

**Checkpoint**: `cargo check` still passes. New SQL file and dependency present but unused.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Switch the core database infrastructure to SQLx. After T007, the project will **not compile** until Phase 3 repos are migrated — this is the intended Red phase.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [ ] T004 Rewrite `src/db/connection.rs` — replace `sea_orm::Database::connect` with `SqlitePoolOptions::new().connect_with(SqliteConnectOptions::from_str(url)?.journal_mode(Wal).foreign_keys(true).create_if_missing(true))`. Return `sqlx::SqlitePool`. Keep function signature `connect_sqlite(url: &str) -> Result<SqlitePool, sqlx::Error>`.
- [ ] T005 Rewrite `src/db/bootstrap.rs` — call `sqlx::migrate!("migrations").run(&pool).await?` instead of `Migrator::up`. Replace `admin_users::ActiveModel` insert with a `sqlx::query!` INSERT. Replace `admin_users::Entity::find().filter(...)` lookup with `sqlx::query_as!`. Accept `&SqlitePool` instead of `&DatabaseConnection`. Remove all `use sea_orm::*` and `use debtor_migration::*` imports.
- [ ] T006 Remove `sea-orm-migration` from `Cargo.toml` `[dependencies]`. Keep `sea-orm` for now (still needed by repos until Phase 3).
- [ ] T007 Update `src/app/state.rs` — change `pub db: DatabaseConnection` to `pub db: SqlitePool`. Update `AppState::new` signature. Remove `use sea_orm::DatabaseConnection`. Add `use sqlx::SqlitePool`. *(This makes the project fail to compile — expected.)*
- [ ] T008 Update `src/web/error.rs` — replace `sea_orm::DbErr` with `sqlx::Error` in `AppError::Database(#[from] ...)`. Update import.
- [ ] T009 Update `src/debts/debt_summary_service.rs` error type — replace `#[from] sea_orm::DbErr` with `#[from] sqlx::Error` in `DebtSummaryError::Database`. Update `use sea_orm::DatabaseConnection` to `use sqlx::SqlitePool` in the struct and constructor signatures. *(Entity type imports will be fixed in T021.)*

**Checkpoint**: `cargo check` has compile errors in all repo/service files that still use `DatabaseConnection`. This is correct — proceed to Phase 3.

---

## Phase 3: User Story 1 — Application Continues to Work (Priority: P1) 🎯 MVP

**Goal**: Every repo and service is rewritten to use SQLx. The full test suite passes.

**Independent Test**: `cargo test` passes with zero failures. All of: unit, integration, contract suites.

### Auth repos (can be parallelised)

- [ ] T010 [P] [US1] Rewrite `src/auth/auth_state_repo.rs` — define `AuthStateRow` struct (see `data-model.md`); replace all `auth_state::Entity`, `ActiveModel`, and `EntityTrait` calls with `query_as!(AuthStateRow, ...)` / `query!(...)`. Change `conn: DatabaseConnection` to `pool: SqlitePool`.
- [ ] T011 [P] [US1] Rewrite `src/auth/session_repo.rs` — define `SessionRow` struct; replace all `sessions::Entity` / `ActiveModel` calls with `query_as!(SessionRow, ...)` / `query!(...)`. Change `conn: DatabaseConnection` to `pool: SqlitePool`. Note: `user_id` changes from `i32` to `i64`.
- [ ] T012 [US1] Update `src/auth/login_service.rs` — define `AdminUserRow` struct; replace `admin_users::Entity::find().filter(...)` with `query_as!(AdminUserRow, "SELECT ... WHERE username = ?", username)`; change `conn: DatabaseConnection` to `pool: SqlitePool`. Depends on T010, T011.

### Group and member repos (can be parallelised)

- [ ] T013 [P] [US1] Rewrite `src/groups/group_repo.rs` — define `GroupRow` struct; replace all `groups::Entity` / `ActiveModel` calls with `query_as!(GroupRow, ...)` / `query!(...)`. Change `conn: DatabaseConnection` to `pool: SqlitePool`.
- [ ] T014 [P] [US1] Rewrite `src/groups/member_repo.rs` — define `MemberRow` struct; replace all `members::Entity` / `ActiveModel` calls with `query_as!(MemberRow, ...)` / `query!(...)`. Change `conn: DatabaseConnection` to `pool: SqlitePool`. Note: `is_active` maps to `bool` via SQLx sqlite feature.
- [ ] T015 [US1] Update `src/groups/group_service.rs` — change `conn: DatabaseConnection` to `pool: SqlitePool`; propagate to `GroupRepo::new` and `MemberRepo::new` calls. Depends on T013, T014.
- [ ] T016 [US1] Update `src/groups/member_service.rs` — same type change as T015. Depends on T013, T014.

### Expense repos (can be parallelised)

- [ ] T017 [P] [US1] Rewrite `src/expenses/expense_repo.rs` — define `ExpenseRow` struct; replace all `expenses::Entity` / `ActiveModel` calls with `query_as!(ExpenseRow, ...)` / `query!(...)`. The `list_with_shares_by_group` method requires two queries joined in application code: fetch expenses by group, then fetch shares by expense IDs. Change `conn: DatabaseConnection` to `pool: SqlitePool`.
- [ ] T018 [P] [US1] Rewrite `src/expenses/share_repo.rs` — define `ExpenseShareRow` struct; replace all `expense_shares::Entity` calls with `query_as!(ExpenseShareRow, ...)` / `query!(...)`. The `replace_shares` bulk insert uses individual `query!` calls in a loop (or a single INSERT with multiple value rows). Change `conn: DatabaseConnection` to `pool: SqlitePool`.
- [ ] T019 [US1] Update `src/expenses/expense_service.rs` — change `conn: DatabaseConnection` to `pool: SqlitePool`; propagate to `ExpenseRepo::new` and `ShareRepo::new`. Depends on T017, T018.

### Exchange rate repo

- [ ] T020 [US1] Rewrite `src/exchange_rates/rate_repo.rs` — define `ExchangeRateRow` struct; replace all `exchange_rates::Entity` / `ActiveModel` calls with `query_as!(ExchangeRateRow, ...)` / `query!(...)`. The `upsert_rate` method uses a SELECT then INSERT or UPDATE pattern (SQLite's `INSERT OR REPLACE` is an alternative). Change `conn: DatabaseConnection` to `pool: SqlitePool`.
- [ ] T021 [US1] Update `src/exchange_rates/rate_service.rs` — change `conn: DatabaseConnection` / `DatabaseConnection` parameters to `SqlitePool`; propagate to `RateRepo::new`. Depends on T020.

### Debt summary service

- [ ] T022 [US1] Update `src/debts/debt_summary_service.rs` — replace `use crate::db::entities::{expense_shares, expenses, groups}` imports with imports of `ExpenseRow`, `ExpenseShareRow`, `GroupRow` from their respective repo modules; update `collect_expense_summaries` method signature and `map_share` helper to use the new row structs. Change `conn: DatabaseConnection` to `pool: SqlitePool`. Depends on T013, T017, T018.

### Test infrastructure

- [ ] T023 [US1] Rewrite `tests/support/mod.rs` — replace `DatabaseConnection` with `SqlitePool`; replace `admin_users::ActiveModel.insert(conn)` seed with `sqlx::query!("INSERT INTO admin_users ...", ...)` call; replace `sea_orm::Database::connect` with `connect_sqlite` (which now returns `SqlitePool`); remove all `use sea_orm::*` imports. Depends on T004, T005, T010–T022.

### Offline metadata

- [ ] T024 [US1] Run `DATABASE_URL=sqlite://debtor.db?mode=rwc cargo sqlx prepare` to generate `.sqlx/` offline query metadata directory. Commit `.sqlx/` to the repository. Verify `SQLX_OFFLINE=true cargo build` succeeds with no live database.

**Checkpoint**: `cargo test` passes — all unit, integration, and contract suites green. US1 complete.

---

## Phase 4: User Story 2 — Sea-ORM Dependency Completely Removed (Priority: P2)

**Goal**: `sea-orm` and `sea-orm-migration` are absent from `Cargo.toml` and `Cargo.lock`.

**Independent Test**: `grep 'sea-orm' Cargo.toml` returns no matches. `cargo build` succeeds.

- [ ] T025 [US2] Remove `sea-orm` and `time` from `Cargo.toml` `[dependencies]`. Verify `cargo check` still passes.
- [ ] T026 [P] [US2] Delete `src/db/entities/` directory entirely (all 9 files: `admin_users.rs`, `auth_state.rs`, `exchange_rates.rs`, `expense_shares.rs`, `expenses.rs`, `groups.rs`, `members.rs`, `sessions.rs`, `mod.rs`).
- [ ] T027 [P] [US2] Update `src/db/mod.rs` — remove `pub mod entities;` line.

**Checkpoint**: `cargo build` succeeds with no Sea-ORM references. `cargo test` still passes.

---

## Phase 5: User Story 3 — Schema Migrations Managed Without Sea-ORM (Priority: P3)

**Goal**: The `migrations/` Cargo crate is gone. Schema is applied automatically on startup via SQLx plain SQL files.

**Independent Test**: Delete `debtor.db`; run `cargo run`; confirm app starts and schema is applied. Inspect `migrations/` — only `.sql` files remain.

- [ ] T028 [US3] Delete the `migrations/` Cargo crate: remove `migrations/Cargo.toml`, `migrations/src/lib.rs`, `migrations/src/main.rs`, `migrations/src/m20260223_000001_init_schema.rs`, and `migrations/src/` directory. Verify `migrations/20260223000001_init_schema.sql` (created in T003) is the only remaining file under `migrations/`.
- [ ] T029 [US3] Verify end-to-end schema bootstrap: delete `debtor.db`, run `cargo run`, confirm app starts successfully and `_sqlx_migrations` tracking table is present in the database. Confirm `migrations/Cargo.toml` no longer exists.

**Checkpoint**: US3 complete. All three user stories independently verified.

---

## Phase 6: Polish & Cross-Cutting Concerns

- [ ] T030 [P] Run `cargo clippy -- -D warnings` and fix all emitted warnings across the migrated files.
- [ ] T031 [P] Run `SQLX_OFFLINE=true cargo build --release` and confirm it succeeds with no live database.
- [ ] T032 Run full test suite one final time: `cargo test` — confirm 100% pass rate with no tests skipped or deleted.
- [ ] T033 [P] Update `AGENTS.md` — replace references to `sea-orm-migration crate` and `debtor-migration` binary with the new SQLx migration approach (as flagged in constitution v3.0.0 sync impact report).
- [ ] T034 [P] Update `CLAUDE.md` project overview note — remove the "codebase currently uses Sea-ORM" caveat and "do not add new Sea-ORM usage" instruction; update migration commands section if it references `debtor-migration`.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: No dependencies — start immediately
- **Phase 2 (Foundation)**: Depends on Phase 1 (T003 must exist before T005 can embed `sqlx::migrate!("migrations")`) — BLOCKS Phase 3
- **Phase 3 (US1)**: Depends on Phase 2 complete; repo tasks T010–T021 can run in parallel; T022 (test support) and T024 (sqlx prepare) must be last
- **Phase 4 (US2)**: Depends on Phase 3 complete (all sea-orm usages replaced before removal)
- **Phase 5 (US3)**: Depends on Phase 2 complete (T005 embeds `sqlx::migrate!()`); can run after Phase 3
- **Phase 6 (Polish)**: Depends on Phases 4 and 5 complete

### Within Phase 3

```
T010, T011 (auth repos) ──→ T012 (login_service)
T013, T014 (group repos) ──→ T015, T016 (group/member services)
T017, T018 (expense repos) ──→ T019 (expense service)
T020 ──→ T021 (rate service)
T013, T017, T018 ──→ T022 (debt summary service)
T010–T022 all complete ──→ T023 (test support rewrite)
T023 ──→ T024 (sqlx prepare)
```

### Parallel Opportunities

```bash
# Phase 3 — all repo rewrites are independent files, run in any order:
T010  # auth_state_repo.rs
T011  # session_repo.rs
T013  # group_repo.rs
T014  # member_repo.rs
T017  # expense_repo.rs
T018  # share_repo.rs
T020  # rate_repo.rs
```

---

## Implementation Strategy

### MVP First (User Story 1 — App Works)

1. Complete Phase 1: Setup (T001–T003)
2. Complete Phase 2: Foundation (T004–T009) — compilation breaks here intentionally
3. Complete Phase 3: All repos migrated (T010–T024) — compilation and tests restored
4. **STOP and VALIDATE**: `cargo test` — all green

### Incremental Delivery

1. Phase 1 + 2 → Foundation ready (Red phase)
2. Phase 3 → Tests pass again (US1 MVP)
3. Phase 4 → Sea-ORM gone (US2)
4. Phase 5 → migrations/ crate gone (US3)
5. Phase 6 → Zero warnings, clean build

---

## Summary

| Phase | Tasks | User Story |
|-------|-------|------------|
| Phase 1: Setup | T001–T003 | — |
| Phase 2: Foundation | T004–T009 | — |
| Phase 3: App Works | T010–T024 | US1 (P1) |
| Phase 4: Remove sea-orm | T025–T027 | US2 (P2) |
| Phase 5: Remove crate | T028–T029 | US3 (P3) |
| Phase 6: Polish | T030–T034 | — |
| **Total** | **34 tasks** | |

- [P] tasks in Phase 3: **8** (T010, T011, T013, T014, T017, T018, T020 — 7 repo rewrites — plus T025, T026, T027, T030, T031, T033, T034 in other phases)
- Suggested MVP scope: Phases 1–3 (US1 only) — app fully functional on SQLx
