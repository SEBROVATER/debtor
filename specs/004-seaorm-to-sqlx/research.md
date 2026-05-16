# Research: Sea-ORM to SQLx Migration

**Feature**: 004-seaorm-to-sqlx  
**Date**: 2026-04-14

---

## R-001: SQLx Version and Feature Selection

**Decision**: Use `sqlx = { version = "0.8", features = ["sqlite", "runtime-tokio-rustls", "macros", "chrono", "rust_decimal"] }`

**Rationale**: SQLx 0.8 is the current stable release. The `sqlite` feature enables `SqlitePool`. The `macros` feature enables `query!`/`query_as!`. The `chrono` feature provides automatic mapping between `NaiveDate`/`NaiveDateTime` and SQLite TEXT columns. The `rust_decimal` feature maps `rust_decimal::Decimal` to/from TEXT (exact representation, matching current Sea-ORM storage).

**Alternatives considered**:
- `sqlx 0.7`: Older; missing some ergonomics in 0.8. No reason to stay on 0.7.
- Dropping `rust_decimal` feature and storing as `f64`: Rejected — precision loss for monetary values (FR-006).
- Storing decimals as INTEGER cents: Rejected — requires application-layer scaling on every read/write, adds complexity.

---

## R-002: `DatabaseConnection` → `SqlitePool` Propagation

**Decision**: Replace `sea_orm::DatabaseConnection` with `sqlx::SqlitePool` everywhere. `SqlitePool` is cheaply cloneable (it is an `Arc` internally) so the existing pattern of `.clone()` in service constructors continues unchanged.

**Rationale**: `SqlitePool` is the SQLx equivalent of `DatabaseConnection` for SQLite. It manages connection pooling, WAL concurrency, and is the expected argument type for `query!` macro calls via `&pool` or `pool.acquire()`.

**Alternatives considered**:
- `SqliteConnection` (single connection, no pool): Rejected — tests create multiple concurrent connections; `SqlitePool` is safe for multi-threaded test execution.

---

## R-003: Offline Mode — `.sqlx/` Directory

**Decision**: Use SQLx offline mode. Run `cargo sqlx prepare` once after any query change to regenerate the `.sqlx/` directory. Commit `.sqlx/` to the repository. Set `SQLX_OFFLINE=true` in `.env` (and CI) so `cargo build` succeeds without a live database.

**Rationale**: The `.sqlx/` directory (introduced in sqlx 0.6) replaces the older `sqlx-data.json` file. Each file in `.sqlx/` contains the compile-time type information for one query macro call. `SQLX_OFFLINE=true` tells the macros to use these cached descriptors instead of querying a live database.

**Workflow**:
1. Developer makes a query change.
2. Developer runs `DATABASE_URL=sqlite://debtor.db?mode=rwc cargo sqlx prepare` (requires a database file).
3. Commit the updated `.sqlx/` files alongside the code change.

**Alternatives considered**:
- Requiring live `DATABASE_URL` at build time: Rejected — breaks CI without a pre-seeded database; rejected per clarification Q2.
- Runtime `query_as` without macros: Rejected — drops compile-time verification, violates FR-002.

---

## R-004: `sqlx::migrate!()` Embedded Migration Runner

**Decision**: Embed `sqlx::migrate!("migrations")` in the main application's startup sequence (`src/db/bootstrap.rs`). The macro reads `.sql` files from the `migrations/` directory at compile time and applies any unapplied migrations at runtime using the `_sqlx_migrations` tracking table.

**Migration file naming**: `{YYYYMMDDHHMMSS}_{description}.sql` — e.g., `20260223000001_init_schema.sql`.

**Rationale**: `sqlx::migrate!()` is zero-configuration, embeds the SQL at compile time (no runtime file I/O), and handles incremental runs correctly. Deleting the `migrations/` Cargo crate and replacing it with SQL files in the same-named directory is the minimal possible change.

**Schema content**: A direct SQL translation of the current `sea_orm_migration` DSL. SQLite does not enforce `DECIMAL(16,6)` precision — columns are stored as TEXT via rust_decimal. Use `TEXT NOT NULL` for all decimal columns.

**Alternatives considered**:
- Keeping `migrations/` as a separate Rust binary that calls `sqlx::migrate!()`: Rejected — unnecessary moving part; rejected per clarification Q3.
- `refinery` crate: Rejected — introduces a new dependency when sqlx has the feature built-in.

---

## R-005: `query_as!` Return Type Pattern

**Decision**: Define plain Rust row structs for each entity (replacing Sea-ORM `Model` types). Use `#[derive(Debug, Clone)]` on each. Use `query_as!(RowStruct, "SELECT ...")` to map results. For insert/update/delete operations that don't return rows, use `query!("...", ...)`.exec(&pool).await?`.

**Column nullability mapping**:
- Nullable columns (e.g., `revoked_at`, `lockout_until`, `note`) → `Option<T>`
- `bool` columns in SQLite (stored as `INTEGER`) → `bool` (sqlx handles the 0/1 mapping automatically with the sqlite feature)

**Rationale**: `query_as!` infers column types from the schema at prepare time. Plain structs with matching field names are idiomatic SQLx. The Sea-ORM `Model` types (which derived from `DeriveEntityModel`) are deleted with `src/db/entities/`.

**Alternatives considered**:
- `#[derive(sqlx::FromRow)]` structs: Also valid, but `query_as!` with an inline struct is more explicit and keeps types close to queries.
- Returning `sqlx::Row` directly: Rejected — dynamic, loses type safety.

---

## R-006: Error Type Replacement

**Decision**: Replace `sea_orm::DbErr` with `sqlx::Error` throughout. Specifically:
- `AppError::Database(#[from] DbErr)` → `AppError::Database(#[from] sqlx::Error)`
- `DebtSummaryError::Database(#[from] sea_orm::DbErr)` → `DebtSummaryError::Database(#[from] sqlx::Error)`
- All repo return types change from `Result<_, DbErr>` to `Result<_, sqlx::Error>`

**Rationale**: `sqlx::Error` is the direct equivalent. All call sites that convert repo errors to `AppError` or `DebtSummaryError` use `?` and the `#[from]` derive, so no manual conversion code changes.

**Alternatives considered**:
- Introducing a custom error wrapper: Rejected — YAGNI; `sqlx::Error` is sufficient and public.

---

## R-007: `SqliteConnectOptions` for PRAGMA Configuration

**Decision**: Replace the current `conn.execute(Statement::from_string(...))` PRAGMA calls with `SqliteConnectOptions` builder methods:

```rust
SqliteConnectOptions::from_str(database_url)?
    .journal_mode(SqliteJournalMode::Wal)
    .foreign_keys(true)
    .create_if_missing(true)
```

Then create the pool: `SqlitePoolOptions::new().connect_with(options).await?`

**Rationale**: `SqliteConnectOptions` is the idiomatic SQLx way to configure PRAGMAs at connection time. Avoids raw SQL statements in setup code. `create_if_missing(true)` replaces the `?mode=rwc` URL suffix.

---

## R-008: `time` Crate Removal

**Decision**: Remove the `time` crate from `Cargo.toml` after migration. The `time` crate is currently listed as a direct dependency with `formatting`, `macros`, `parsing`, and `serde` features — but zero `use time::` imports exist in `src/`. It was pulled in as a Sea-ORM `with-time` feature requirement. After Sea-ORM removal, it becomes unused.

**Rationale**: `chrono` is used exclusively throughout the codebase for all date/time values. The `time` crate is dead weight post-migration.

---

## R-009: Test Support Rewrite Strategy

**Decision**: Rewrite `tests/support/mod.rs` to use `sqlx::SqlitePool` and direct `query!` macro calls for seeding. The `seed_admin_user` function becomes:

```rust
pub async fn seed_admin_user(pool: &SqlitePool, username: &str, password_hash: &str) -> AdminUserRow {
    let now = Utc::now().naive_utc();
    query!("INSERT INTO admin_users (id, username, password_hash, created_at, updated_at) VALUES (1, ?, ?, ?, ?)",
        username, password_hash, now, now)
        .execute(pool).await.expect("insert admin user");
    // ...
}
```

The `initialize_database` function switches to `sqlx::migrate!()`.

**Rationale**: The test support module is infrastructure, not feature code. Its changes are entirely driven by the failing integration tests (TDD: tests are red because `DatabaseConnection` no longer exists; rewriting support makes them green).

---

## R-010: `bool` Handling in SQLite via SQLx

**Decision**: SQLite stores booleans as `INTEGER` (0/1). SQLx with the `sqlite` feature automatically encodes `bool` to `0`/`1` and decodes back. No manual casting needed. The `is_active` field on Members maps directly.

**Rationale**: Verified in SQLx 0.8 documentation — `bool` encode/decode is built into the SQLite type system adapter.
