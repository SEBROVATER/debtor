---
baseline_commit: 1bbfbc3eac091f97c8ad3a17f2ce5e24f7d8d74d
---

# Story 1.2: Start a Persistent Local Application

Status: done

## Story

As the administrator,
I want one validated command to start a persistent local Debtor process,
so that I can reach my private ledger without external build or provider prerequisites.

## Acceptance Criteria

1. **Given** the repository is checked out with the pinned Rust 1.97.1 toolchain and lockfiles
   **When** the production workspace is inspected
   **Then** it uses edition 2024, MSRV 1.97, Cargo resolver 3, the minimal rustfmt/Clippy profile, and the root plus four required crates with inward-only manifest dependencies
   **And** `tools/password-hash` remains independent and routine work never uses `cargo build --release`.

2. **Given** a local operator copies `.env.example` and supplies a Story 1.1 password hash
   **When** configuration is loaded
   **Then** every mandatory variable is documented without a secret and bare `cargo run` selects the application binary
   **And** invalid required configuration fails before database connection, migration, or socket admission.

3. **Given** local configuration is valid
   **When** `cargo run` starts Debtor
   **Then** it creates or connects to the configured persistent SQLite database, runs only migrations required by this slice, enables foreign keys, WAL, `synchronous=FULL`, and a five-second busy timeout, composes concrete adapters behind application-owned ports, and binds the configured address
   **And** it reports only a non-secret local URL including `http://`.

4. **Given** Frankfurter is unavailable and no Docker service, frontend build, manual migration, or SQLx metadata generation has run
   **When** startup occurs
   **Then** Debtor reaches socket admission
   **And** provider availability is not consulted.

5. **Given** production manifests are resolved
   **When** dependency versions are inspected
   **Then** they retain the adopted pinned versions and features recorded by the architecture and project context
   **And** lockfiles are preserved, validation uses `--locked`, and current crate documentation is consulted before framework API changes.

6. **Given** SQL or migrations required by this runnable slice change
   **When** persistence work is validated
   **Then** every SQL statement uses checked SQLx macros except the fixed WAL-checkpoint PRAGMA, temporary-database migration and online prepare checks pass, and refreshed `.sqlx` metadata is committed
   **And** SQLite constraints stay structural and do not duplicate Unicode trimming or monetary arithmetic.

7. **Given** the current brownfield application contains retained and superseded startup paths
   **When** this story completes
   **Then** the existing crate direction, root composition, and checked-query workflow are retained; obsolete configuration, migration, and startup paths are replaced or removed
   **And** no parallel compatibility startup remains.

## Tasks / Subtasks

- [x] Establish the Story 1.2 startup boundary and preserve only required prior work (AC: 1, 7)
  - [x] Read `specs/design.md`, the Story 1.1 record, and every currently modified nested-workspace file before changing it. Story 1.1's canonical bounded password admission is a prerequisite and must remain intact.
  - [x] Retain the production workspace shape in `Cargo.toml`: root `debtor`, `debtor-domain`, `debtor-application`, `debtor-infra`, and `debtor-web`; resolver 3; edition 2024; MSRV 1.97; default binary `debtor`; and independent `tools/password-hash` workspace.
  - [x] Retain and update `src/bin/architecture-check.rs` with any manifest changes so it continues to reject outward normal/build dependencies, direct root-to-domain dependency, missing production packages, and a missing default binary.
  - [x] Do not add framework, SQLx, HTTP, Argon2, session, or concrete adapter types to domain/application. Do not create a second startup/composition path or compatibility shim.

- [x] Deliver a safe provider-independent local startup sequence (AC: 2-4)
  - [x] Update `src/config.rs`, `src/composition.rs`, `src/main.rs`, and `debtor-infra/src/db.rs` only as needed for this ordered path: load `.env`; validate all configuration and concrete constructor inputs; connect/configure SQLite; run the slice's embedded migrations; compose application-owned ports with concrete outer adapters in root; bind `APP_BIND`; run the server.
  - [x] Preserve the existing `Config::from_lookup` password-hash validation before `build_app`. Invalid password, bind, cookie, proxy, database URL, or other required startup configuration must fail before SQLite creation/connection, migration, or listener binding. Map failures to safe startup categories; never interpolate secrets, hashes, database URL, or raw adapter errors.
  - [x] Keep `debtor_infra::db::connect` as the only SQLite connection adapter. It must use `SqliteConnectOptions` with `create_if_missing(true)`, foreign keys enabled, WAL journal mode, `SqliteSynchronous::Full`, and a five-second busy timeout. SQLx remains in infra/root only.
  - [x] Construct `FrankfurterClient` only after local configuration validation and never make a provider request during startup, migration, readiness, or socket admission. An unreachable configured local URL is valid startup-test evidence.
  - [x] Bind only after `build_app` returns successfully. Log only a local `http://<bind-address>` URL and fixed safe startup stages. Do not log environment values, password/hash data, SQLite diagnostics, provider URLs, request values, or identifiers.
  - [x] Keep `cargo run` as the root package's default local command. Do not require Docker, a frontend build, manual migrations, a live provider, or SQLx preparation for ordinary startup.

- [x] Remove premature brownfield surfaces rather than preserving an incompatible runtime (AC: 3, 6, 7)
  - [x] Audit all `migrations/20260517000001_*` through `20260517000006_*`, `debtor-infra` repositories, root composition, router/routes, templates, and tests. These currently pre-create/wire Groups, Participants, legacy memberships, Spendings, allocations, rate/debt functionality, sessions, and lifecycle behavior that are owned by later stories.
  - [x] Keep only the persistent-runtime/migration capability actually consumed by Story 1.2. Group schema and mutation evidence belong to Story 2.1; Participant schema belongs to Story 2.3; Spending/allocation schema belongs to Story 3.1. Do not leave future tables, routes, repositories, or test fixtures merely because they compile.
  - [x] Remove superseded legacy membership/global-participant/multiple-payer/equal-share paths outright. Never add a legacy schema, startup branch, or dual composition just to preserve an unshipped database or API.
  - [x] If checked SQL or migrations are removed or changed, remove stale dependent tests and stale `.sqlx/query-*.json` metadata in the same change; regenerate only metadata for checked SQL that remains. Do not use unchecked SQL except the established fixed WAL-checkpoint exception.
  - [x] Do not claim later story outcomes as complete: no Login/session/CSRF UI (Stories 1.4-1.7), probes/admission supervisors (1.8), shutdown/restart proof (1.3/1.9), Group user flow (2.1), or ledger/rate/debt behavior. Preserve only code that is a genuine dependency of the runnable startup slice or Story 1.1.

- [x] Synchronize operator documentation without secrets (AC: 1, 2, 7)
  - [x] Update `.env.example` and `README.md` to describe the current slice accurately: copy `.env.example`, generate the hash via the independent helper, set `APP_ADMIN_PASSWORD_HASH`, and run `cargo run`.
  - [x] Keep `APP_ADMIN_PASSWORD_HASH` blank in examples. Document defaults and required variables without embedding a hash, database path containing user data, or a provider secret.
  - [x] Remove stale claims of future completed Groups, memberships, Spendings, settlements, rate integration, authentication UI, probes, or shutdown behavior if those paths are removed for this story. `specs/design.md` is already the normative target; update it first only if implementation changes the specified behavior rather than removing superseded brownfield scaffolding.

- [x] Prove the runnable increment at the owning layers (AC: 1-6)
  - [x] Extend `src/config.rs` unit tests for defaults and invalid configuration. Ensure password/hash validation stays pre-side-effect and assertions never expose supplied secrets.
  - [x] Extend root composition/startup tests in `src/main.rs`: invalid configuration leaves the temporary SQLite file absent; valid configuration creates/reopens a file database, runs only the intended migrations, binds `127.0.0.1:0`, and admits a basic socket request without a provider call.
  - [x] Retain or adapt `debtor-infra/tests/db.rs` to prove foreign keys, WAL, `synchronous=FULL`, `busy_timeout=5000`, and persistence after reopening. Keep these adapter tests separate from root socket-admission proof.
  - [x] Rewrite or remove future-schema fixtures in `debtor-infra/tests/migrations.rs` when future migrations are removed. Test only the migration set that belongs to this slice; do not retain assertions for future Group/Participant/Spending tables to make empty infrastructure appear complete.
  - [x] Run focused tests while iterating, then run the complete validation below. Use debug `cargo check`, `cargo test`, and `cargo run` only; never use `cargo build --release`.

### Review Findings

- [x] [Review][Patch] Reject non-persistent SQLite URLs [debtor/src/config.rs:27] - `APP_DATABASE_URL=sqlite::memory:` is accepted and successfully starts an ephemeral database, violating AC 3's persistent local database requirement and bypassing the required durable file/WAL topology. Parse and validate the URL during configuration loading, rejecting memory and temporary targets before `build_app` can connect.
- [x] [Review][Patch] Log the actual listener address [debtor/src/main.rs:28] - `APP_BIND=127.0.0.1:0` binds successfully but the startup log prints `http://127.0.0.1:0`, not the reachable kernel-assigned port. Obtain `listener.local_addr()` after bind and use it in the safe URL log.
- [x] [Review][Patch] Keep test SQL checked and represented in metadata [debtor/src/main.rs:109] - The root schema assertion now uses `sqlx::query!` with refreshed metadata. SQLite reports dynamic NULL metadata for its fixed adapter-test PRAGMAs, which SQLx cannot compile-time decode; their narrow raw reads are documented locally and remain limited to test verification.

## Dev Notes

### Scope And Dependencies

- Epic 1 delivers secure local operation before any ledger capability. This packet is intentionally bounded to validated configuration, durable local SQLite startup/migration, root composition, and provider-independent socket admission (estimated 3-5 days).
- Story 1.1 is the direct predecessor. Reuse its `ArgonPasswordGate` policy and configuration admission; do not duplicate PHC parsing or weaken the no-external-side-effect ordering.
- Story 1.3 owns restart, WAL recovery, broad architecture/dependency governance, and shutdown evidence. Story 1.4 begins the Login/session/CSRF/submission-token surface. Story 2.1 supplies the first real ledger mutation. Do not pre-implement those outcomes here.
- No UX registry identifier applies: this slice adds no Administrator-facing HTML contract. A bindable minimal runtime is acceptable; do not keep a user-facing scaffold merely to simulate later UX.

### Existing Implementation To Read And Reconcile

| Path | Current state | Required story treatment |
| --- | --- | --- |
| `Cargo.toml` | Already declares resolver 3, 2024/MSRV 1.97 workspace settings, root default-run, four production crates, pinned dependency ranges, and lint policy. | Retain this structure and lockfile discipline. Update architecture-check fixtures/allowlists atomically if manifests change; do not pull outer dependencies inward. |
| `src/config.rs` | `Config::from_lookup` validates the Story 1.1 hash before returning configuration; defaults database to `sqlite://debtor.db?mode=rwc` and bind to `127.0.0.1:3000`. | Preserve early safe validation and defaults, including debug-only insecure-cookie allowance. Do not return raw configuration or password diagnostics. |
| `src/composition.rs` | Validates proxy/password/rate constructor inputs, connects SQLite, runs `sqlx::migrate!`, then eagerly composes future ledger, rate, auth, session, static, and web functionality. | Preserve safe pre-DB validation, database connect/migrate ordering, and root-only concrete composition. Remove/replace future-capability composition not consumed by this slice; no second startup path. |
| `src/main.rs` | Loads `.env`, configures tracing, validates config, installs signals, builds app, binds, logs `http://`, and enters the later runtime. Existing tests include composition plus later auth/shutdown behavior. | Bind only after successful build. Keep safe URL logging. Retain/adjust only test evidence owned by startup; later lifecycle evidence belongs to its owner stories. |
| `debtor-infra/src/db.rs` | Sole SQLite pool adapter; configures file creation, foreign keys, WAL, FULL synchronous mode, five-second busy timeout, and five connections. | Retain the adapter and settings. Do not move SQLx into application/domain or replace it with an in-memory/nonpersistent startup path. |
| `migrations/` and `debtor-infra/tests/migrations.rs` | Six migrations and tests currently establish future Groups, Participants, legacy memberships, Spendings, Payers, and Shares. | Audit and remove/rewrite premature schema and matching tests/metadata. This story must not claim or retain later ledger capability through an empty startup slice. |
| `.env.example`, `README.md` | Hash guidance is current, but README claims future ledger, auth, rate, probe, and shutdown behavior as completed. | Keep safe hash/local-run instructions; make status truthful for the delivered slice and remove stale implementation claims. |
| `src/bin/architecture-check.rs` | Inspects `cargo metadata --locked` and validates required packages, default run, non-publishability, allowlists, and inward direction. | Preserve it as architecture-fitness evidence; synchronize only if the actual manifest graph changes. |

### Architecture And Security Guardrails

- Preserve `debtor (root) -> debtor-web/debtor-infra -> debtor-application -> debtor-domain`. Root owns configuration, migrations, concrete adapter composition, listener startup, and process orchestration; it is the only place concrete adapters are wired.
- Domain remains synchronous, deterministic, I/O-free, and framework-free. Application keeps only narrow ports and use cases; external effects remain constructor-injected and fake-testable.
- Keep SQLite's supported topology: one process and one local WAL volume. Do not introduce external writers, multiple instances, an external database, Docker dependency, or provider startup dependency.
- Maintain exact-money/historical rules even though this story does not exercise them: `Decimal`, canonical `TEXT`, Rust aggregation, and no SQL monetary arithmetic remain the required future persistence contract. Do not encode premature monetary policy in SQLite migrations.
- Apply `thiserror` in domain/application/infra and confine `anyhow` to root orchestration/config. Production code uses neither `unsafe` nor `unwrap`/`expect`.
- Never log credentials, hashes, session values, tokens, SQLite diagnostics, provider URLs, raw errors, database values, or request-derived data. Safe startup error categories are the only outward diagnostics.

### Library And Framework Requirements

- Keep the adopted locked dependency set. `Cargo.lock` is the exact dependency authority; do not opportunistically upgrade Axum, SQLx, Tokio, or any crate in this story.
- SQLx 0.9: use the embedded `sqlx::migrate!` migrator for the repository migration directory and call `Migrator::run` against the configured `SqlitePool`. It validates applied migrations and executes pending ones. Add a Cargo migration-change trigger only if required by the chosen embedded-migration build behavior. [Source: Context7 `/websites/rs_sqlx_sqlx`, `migrate!` and `Migrator::run`]
- Axum 0.8: bind a `tokio::net::TcpListener` only after successful composition, then serve a fully-state-provided `Router<()>`. Do not substitute a custom server stack. Graceful shutdown details are Story 1.3/1.9 scope. [Source: Context7 `/tokio-rs/axum`, Router state and `axum::serve`]
- SQL changes require checked SQLx macros and refreshed committed metadata. Run the online prepare validation only after applying the intended migrations to a temporary database; local normal startup must not require this command.

### Testing And Validation Requirements

Run from `debtor/` after the targeted tests:

```bash
cargo fmt --all -- --check
cargo run --bin architecture-check --locked
cargo check --workspace --all-features --locked
SQLX_OFFLINE=true cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-features --locked

cargo fmt --manifest-path tools/password-hash/Cargo.toml -- --check
cargo clippy --manifest-path tools/password-hash/Cargo.toml --all-targets --all-features --locked -- -D warnings
cargo test --manifest-path tools/password-hash/Cargo.toml --locked
```

If checked SQL or migrations changed, additionally migrate a temporary database and run:

```bash
SQLX_OFFLINE=false DATABASE_URL=sqlite:///tmp/debtor-sqlx.db?mode=rwc cargo sqlx prepare --workspace --check
```

Run `cargo deny check` only if dependency manifests, lockfiles, or dependency-policy files change. Do not use `cargo build --release`.

### Previous Story Intelligence

- Story 1.1 is in review, with completed work recorded in `debtor-infra/src/auth/password.rs`, `src/config.rs`, `src/main.rs`, `tools/password-hash/src/main.rs`, `.env.example`, and `README.md`. Treat current uncommitted nested-workspace changes as active work: inspect and preserve relevant password-policy edits rather than overwriting or reverting them.
- Reuse the shared infra-owned `validate_password_hash`/`ArgonPasswordGate` boundary. The valid helper profile is `m=19456,t=2,p=1`, but the runtime validator accepts the wider approved production range; do not accidentally constrain it to helper output.
- Existing implementation tests demonstrate the correct style: temporary file databases, no secret-bearing diagnostics, and root-level no-database-side-effect assertions. Extend these rather than adding a mocking framework.

### Git Intelligence

- The last five repository commits are BMAD planning/agent commits. The current nested Rust workspace is untracked from the outer repository, so git history is not implementation precedent for this packet.
- The parent worktree contains untracked `_bmad-output/implementation-artifacts/` and `debtor/` directories. These are not disposable generated noise: read their current state and avoid reverting any unrelated user/previous-story work.

### Project Structure Notes

- The actual Rust workspace is nested at `debtor/`; all Rust paths in this story are relative to that directory.
- Current planning and sprint artifacts are at the repository root under `_bmad-output/`; the nested `debtor/_bmad-output/` copy is older and must not be used as the planning authority.
- Architecture defers physical route/template layout. Do not invent it in this no-HTML startup story.

### References

- [Source: `_bmad-output/planning-artifacts/epics.md#Story 1.2: Start a Persistent Local Application`]
- [Source: `_bmad-output/planning-artifacts/epics.md#Assignment Packets`]
- [Source: `_bmad-output/planning-artifacts/epics.md#Cross-Cutting Story Rule`]
- [Source: `specs/design.md#Architecture`]
- [Source: `specs/design.md#Local Run Contract`]
- [Source: `specs/design.md#Operational Limits`]
- [Source: `_bmad-output/project-context.md#Framework-Specific Rules`]
- [Source: `_bmad-output/project-context.md#Development Workflow Rules`]
- [Source: `_bmad-output/planning-artifacts/prds/prd-debtor-2026-08-10/addendum.md#Local Run And Tool Independence`]
- [Source: `_bmad-output/planning-artifacts/prds/prd-debtor-2026-08-10/addendum.md#SQLite Integrity And Write Semantics`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-1 - Inward dependency direction [ADOPTED]`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-6 - Single ledger runtime and mutation epoch [ADOPTED]`]
- [Source: `debtor/Cargo.toml`]
- [Source: `debtor/src/config.rs`]
- [Source: `debtor/src/composition.rs`]
- [Source: `debtor/src/main.rs`]
- [Source: `debtor/debtor-infra/src/db.rs`]
- [Source: Context7 `/websites/rs_sqlx_sqlx`, SQLx 0.9 `migrate!` and `Migrator::run` documentation]
- [Source: Context7 `/tokio-rs/axum`, Axum 0.8 Router state and `axum::serve` documentation]

## Dev Agent Record

### Agent Model Used

openai/gpt-5.6-terra

### Debug Log References

- Ultimate context engine analysis completed - comprehensive developer guide created.
- Red/green: added the minimal-schema startup test before removing the premature migrations and composition; it initially selected no test, then passed once the slice was implemented.
- SQLx online metadata validation found no checked queries after the future repository code was removed; the committed `.sqlx` metadata directory is empty by design.

### Completion Notes List

- Status set to `ready-for-dev`.
- Scope, dependencies, current code, prior story intelligence, architecture, UX applicability, git state, and current SQLx/Axum documentation were analyzed.
- This story requires a single provider-independent persistent startup path and explicitly prevents future-capability scaffolding from being retained as this slice's implementation evidence.
- Replaced the eager future ledger/auth/rate runtime with a minimal `/healthz` router, persistent SQLite connection/migration, and post-composition listener binding.
- Preserved the Story 1.1 bounded canonical Argon2id validator and its test coverage; invalid hashes still create no database file.
- Removed premature ledger migrations, SQLx metadata, repositories, routes, templates, legacy memberships, allocation modes, session/runtime code, and corresponding future-story tests.
- Validation passed: format, architecture check, locked workspace check/test, offline Clippy with denied warnings, independent helper format/Clippy/test, online SQLx prepare check, and cargo-deny policy check.
- Code review fixes: rejected memory-mode SQLite URLs before connection, logged the kernel-assigned listener address, and regenerated the metadata for the checked schema assertion. SQLite's fixed test-only PRAGMAs remain locally documented raw reads because SQLx reports their result metadata as dynamic NULL.

### File List

- `_bmad-output/implementation-artifacts/1-2-start-a-persistent-local-application.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `debtor/Cargo.toml`
- `debtor/Cargo.lock`
- `debtor/.env.example`
- `debtor/README.md`
- `debtor/src/config.rs`
- `debtor/src/composition.rs`
- `debtor/src/main.rs`
- `debtor/src/runtime.rs` (deleted)
- `debtor/debtor-domain/Cargo.toml`
- `debtor/debtor-domain/src/*` future ledger modules (deleted)
- `debtor/debtor-application/Cargo.toml`
- `debtor/debtor-application/src/{authentication,errors,lib}.rs`
- `debtor/debtor-application/src/{debts,groups,participants,readiness,spendings}.rs` (deleted)
- `debtor/debtor-infra/Cargo.toml`
- `debtor/debtor-infra/src/{auth,db,lib}.rs`
- `debtor/debtor-infra/src/auth/login_limiter.rs` (deleted)
- `debtor/debtor-infra/src/db/repos*/` and `exchange_rates*/` (deleted)
- `debtor/debtor-infra/tests/db.rs`
- `debtor/debtor-infra/tests/{migrations,repos}.rs` (deleted)
- `debtor/debtor-web/Cargo.toml`
- `debtor/debtor-web/src/{lib,router}.rs`
- `debtor/debtor-web/src/{forms,middleware,participant_color,session,session_store,state,templates}.rs` and `handlers*/` (deleted)
- `debtor/debtor-web/templates/*` (deleted)
- `debtor/migrations/*` (deleted)
- `debtor/.sqlx/query-*.json` (deleted)

### Change Log

- 2026-08-12: Implemented the persistent local startup slice, removed premature future runtime/schema paths, refreshed the lockfile and SQLx metadata, and moved the story to review.
- 2026-08-12: Resolved three code-review findings and revalidated the workspace and SQLx metadata.
