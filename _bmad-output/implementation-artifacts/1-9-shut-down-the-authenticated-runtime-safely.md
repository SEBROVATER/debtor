---
story_key: 1-9-shut-down-the-authenticated-runtime-safely
story_id: 1.9
epic: 1
status: done
baseline_commit: ff80a9cf2109ac406173f48a2e4ec9484077e3b7
created: 2026-08-14
---

# Story 1.9: Shut Down the Authenticated Runtime Safely

Status: done

## Story

As the administrator,
I want the authenticated runtime to stop admission and close its resources safely,
so that the service can shut down cleanly before ledger mutations are introduced.

## Acceptance Criteria

1. **Admission and HTTP drain:** When coordinated shutdown begins, the root lifecycle coordinator closes new user admission before draining HTTP. `/healthz` and `/readyz` retain their independent probe path during the bounded drain, while new login, authenticated, and static user work is rejected before session loading. HTTP drain is bounded to at most ten seconds. The implementation must not add a generic post-dispatch cancellation path for ledger mutations.
2. **Empty mutation barrier:** The root composition owns exactly one dispatched-mutation registry/lifecycle barrier. For this story's runnable evidence, no ledger mutation is dispatched; shutdown observes the registry as empty rather than fabricating a mutation, marking a fake outcome, or treating route absence as a substitute for the registry observation. Real ledger-mutation registration and definitive outcome integration remain Story 2.1 scope.
3. **Storage shutdown order:** After HTTP drain and the empty mutation observation, supervised cleanup workers and the signal worker finish through their existing coordinated-stop path. Only then does storage shutdown attempt the fixed `PRAGMA wal_checkpoint(TRUNCATE)` operation with the existing bounded five-second production limit, and only after checkpoint handling does it await `SqlitePool::close()`. Checkpoint failure is fatal, but never deletes, replaces, or silently recreates the database.
4. **WAL recovery:** If checkpointing cannot complete because a reader/snapshot keeps WAL busy, the `-wal` and `-shm` sidecars remain intact. Reopening the same database path either recovers the committed state or fails safely before socket admission. No failure may be represented as a successful checkpoint or as an invented rollback.
5. **Safe diagnostics:** SQLite and runtime diagnostics expose only fixed operation names, bounded result/failure categories, and approved low-cardinality fields. Logs and captured test output must not contain credentials, password hashes, cookies, session IDs, CSRF/submission tokens, client IPs/limiter keys, SQL, database messages, monetary values, entity identifiers, query strings, provider URLs, or raw adapter/task errors.
6. **Authenticated real-socket smoke:** The composed application starts on a kernel-assigned local socket, serves `/login`, accepts exactly one valid CSRF token and one single-use submission token, returns the existing successful login redirect, serves an authenticated read such as `GET /groups`, then receives coordinated shutdown while the authenticated session is still active. The test observes successful completion, admission closure before storage shutdown, the empty mutation barrier, and resource closure. It must use explicit notifications/barriers or completed responses, not a timing sleep, and must not log out before the primary shutdown assertion.
7. **Existing lifecycle behavior:** Restart, session invalidation, session/token cleanup supervision, readiness/liveness separation, independent probe capacity, safe headers, and the existing no-provider startup contract continue to pass. A normal worker exit caused by coordinated shutdown is not a supervisor failure; panic, cancellation, unexpected return, cleanup error, checkpoint failure, and pool-close failure remain fatal categories.

## Scope Boundary

- This story hardens the root shutdown path and proves authenticated shutdown with no dispatched ledger mutation.
- Existing ledger routes are present in the brownfield tree even though the epic wording describes the pre-ledger phase. Interpret the criterion as “no ledger mutation is dispatched in this story's shutdown evidence,” not as permission to remove or ignore existing routes.
- Do not implement the first real mutation executor consumer, real `Committed`/`RolledBack`/`Unknown` publication, or shutdown waiting on an active ledger mutation. Story 2.1 owns that final evidence.
- Do not add a shutdown page, browser UI, custom JavaScript, HTMX behavior, UX route, migration, provider call, financial policy, new dependency, persistent runtime state, or compatibility scaffold.
- Do not retrofit route-local registries, token stores, retry/idempotency paths, or a second shutdown coordinator.

## Tasks / Subtasks

- [x] Establish one root-owned empty dispatched-mutation lifecycle seam (AC: 2, 3)
  - [x] Add the smallest process-local registry/barrier needed for `BuiltApp`/runtime composition to observe that no mutation is dispatched.
  - [x] Keep the owner in the root lifecycle/composition boundary; do not expose root coordinator, Tokio, SQLx, or Axum types through `debtor-application` ports.
  - [x] Ensure the seam can be consumed by Story 2.1 without creating a parallel executor or fake terminal outcome now.
  - [x] Make the empty-state observation deterministic and testable with `Notify`, a barrier, or an equivalent explicit signal rather than elapsed-time polling.

- [x] Correct shutdown ordering and admission behavior (AC: 1, 3, 7)
  - [x] Close user admission exactly once before beginning HTTP drain; retain the current session-free probe router and independent four-request probe budget during the drain.
  - [x] Preserve the current trigger taxonomy and one-shot coordinator semantics for signal, readiness, cleanup, HTTP, checkpoint, and pool-close failures.
  - [x] Keep the ten-second production HTTP-drain bound and the existing short injected timeout used by deterministic tests.
  - [x] Stop cleanup/signal workers through the existing supervised path and distinguish coordinator-requested normal return from panic, cancellation, join failure, unexpected exit, and cleanup error using `JoinError` state without logging raw details.
  - [x] Wait for the empty mutation registry before checkpointing and close the pool only after checkpoint handling.
  - [x] Do not wrap post-dispatch mutation execution in a generic timeout or cancel it because a client disconnected; leave the real mutation behavior for Story 2.1.

- [x] Preserve WAL and diagnostic contracts (AC: 3-5)
  - [x] Retain the fixed unchecked SQLx exception only for `PRAGMA wal_checkpoint(TRUNCATE)`; do not add unchecked SQL for lifecycle state.
  - [x] Keep checkpoint and pool-close failures as fatal shutdown outcomes while preserving WAL sidecars and avoiding database recreation.
  - [x] Keep logs limited to fixed runtime/storage operation names and bounded categories; never stringify `JoinError`, SQLx errors, panic payloads, SQL, paths, request values, or identifiers.

- [x] Update composed runtime tests (AC: 1-7)
  - [x] Replace the current authenticated smoke sequence that logs out before shutdown with login, authenticated read, coordinated shutdown, and explicit completion assertions.
  - [x] Remove the existing smoke-test timing sleep and coordinate shutdown from a known completed request state.
  - [x] Prove the server observes closed user admission before checkpoint/pool-close activity and keeps probes available during the bounded drain.
  - [x] Prove the empty mutation barrier is observed; never create a fake mutation solely to exercise shutdown.
  - [x] Exercise forced drain with a server-side request drop/termination signal, not only by aborting the client task, and preserve the existing successful drain test.
  - [x] Exercise full-path checkpoint failure with a held SQLite snapshot/WAL frame, assert fatal shutdown, assert sidecars remain, and reopen the same database to verify recovery.
  - [x] Capture or inspect runtime/storage diagnostics using a safe test subscriber and assert forbidden secrets, identifiers, SQL, values, query strings, provider URLs, and raw errors are absent.
  - [x] Retain existing cleanup-supervisor, readiness, probe-capacity, no-cookie, restart, stale-session, and security-header tests.

- [x] Validate the runnable vertical increment (AC: 7)
  - [x] Run `cargo fmt --all -- --check`.
  - [x] Run `cargo check --workspace --all-features --locked`.
  - [x] Run `SQLX_OFFLINE=true cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`.
  - [x] Run `cargo test --workspace --all-features --locked`.
  - [x] Run `cargo run --bin architecture-check --locked`.
  - [x] Run `cargo deny check` only if dependency manifests, lockfiles, or dependency policy change.
  - [x] Run online SQLx preparation only if checked SQL or migrations change; no SQL/migration change is expected for this story.
  - [x] Never use `cargo build --release` for validation.

### Review Findings

- [x] [Review][Patch] Authenticated smoke observes admission closure only after storage shutdown can already have run [src/main.rs:482]
- [x] [Review][Patch] Checkpoint-failure runtime test can pass for the wrong fatal trigger [src/main.rs:932]
- [x] [Review][Patch] New lifecycle evidence uses unchecked SQLx statements beyond the fixed WAL PRAGMA exception [src/main.rs:899]
- [x] [Review][Defer] Real mutation registration and definitive outcome publication remain unwired [src/runtime.rs:78] — deferred, pre-existing

## Dev Notes

### Developer Context

This is a brownfield runtime-completion story. The current code already has admission closure, independent probes, readiness failure signaling, supervised cleanup workers, fixed WAL checkpointing, pool close, and a real-socket authentication smoke test. The missing story-specific behavior is an explicit root-owned empty dispatched-mutation lifecycle boundary and authenticated shutdown evidence that does not sign out first.

Current runtime flow in `src/runtime.rs` is:

1. Spawn session and submission-token cleanup workers and supervisors.
2. Spawn signal supervision and serve Axum with graceful shutdown.
3. Observe server completion or coordinator shutdown.
4. Close user admission and drain HTTP for the configured bound.
5. Stop/join cleanup and signal workers.
6. Checkpoint WAL and close the SQLite pool.

The implementation must insert the empty mutation-registry observation before checkpoint/pool close without disturbing the completed Story 1.8 admission and supervisor behavior. The current `BuiltApp` retains the pool, session/token owners, cleanup health, and `RuntimeControl`; it does not retain a mutation registry. `SqliteLedgerRuntime` owns the infrastructure write gate and future mutation epoch, but Story 1.9 must not prematurely integrate real ledger mutation dispatch.

### Architecture Compliance

- Preserve `root -> debtor-web/debtor-infra -> debtor-application -> debtor-domain`. Root owns configuration, composition, lifecycle, supervision, and shutdown. Web owns HTTP/session/CSRF/token mechanics. Infra owns SQLite and adapter diagnostics. Application ports remain framework- and runtime-free.
- Follow AD-6 and AD-13: one process-local root lifecycle owner and one future-dispatch registry. No route-local registry, application singleton, database shutdown flag, or second admission owner.
- Follow AD-10 and AD-14: the shared unsafe-form boundary and 30-second pre-dispatch deadline remain unchanged; after a future mutation dispatch, generic timeout/cancellation cannot own execution.
- Follow AD-15: `/healthz` remains liveness-only; `/readyz` checks SQLite and mandatory supervisor health only; provider availability and ledger contents never gate readiness; diagnostics are fixed-category and sanitized.
- Follow AD-16: keep root/composition and real-socket tests in the root, web middleware tests in `debtor-web`, storage/checkpoint tests in the infrastructure boundary, and application/domain layers unchanged unless compilation proves a narrow interface adjustment is necessary.
- Follow AD-17: do not add users, tenants, participant authentication, registration, or authorization abstractions.
- No UX contract applies. This story changes no rendered Administrator control. If existing templates or CSS are touched, treat that as scope creep and retain all native HTML, security-header, accessibility, and no-custom-JavaScript guarantees.

### Technical Requirements

- Production HTTP drain is at most 10 seconds. This bounds HTTP draining only; it is not a fixed total shutdown deadline for future dispatched mutations.
- The current production constants are five minutes for cleanup cadence, five seconds for cleanup-stop, five seconds for WAL checkpoint, and five seconds for pool close. Preserve these unless a test-only injected timeout is used.
- Close user admission before draining. Requests arriving after closure must return the existing sanitized `503` before session load, authentication, token issuance, or handler work. Already admitted requests retain their existing behavior.
- Keep `/healthz` and `/readyz` admissible through drain through the separate probe budget. `/readyz` must become not-ready after runtime admission closes; `/healthz` must remain truthful until process exit.
- A coordinated worker return is normal. A `JoinError` with `is_cancelled()` or `is_panic()`, an unexpected normal return before shutdown, cleanup iteration failure, failed forced termination, or join failure is a supervisor failure. Emit only a safe category such as `worker_cancelled`, `worker_panic`, `worker_exit`, or `worker_join_failure`.
- `axum::serve(...).with_graceful_shutdown(...)` drains in-flight connections after its signal. Do not rely on client-task abortion as evidence that server-side request work stopped; explicitly test the server-side behavior needed by the ten-second forced-drain path.
- `SqlitePool::close()` must be awaited. It wakes waiters and waits for checked-out connections to return/close; call it after checkpoint handling, not concurrently with checkpoint or active runtime work.
- The sole unchecked SQLx query remains the fixed-shape `PRAGMA wal_checkpoint(TRUNCATE)` already implemented in `src/runtime.rs`. Do not expose SQLite diagnostics or raw `sqlx::Error` details to logs or HTTP.
- Never log raw task errors, panic payloads, SQL, database messages, provider URLs, request paths with query strings, cookies, session/token IDs, client identities, identifiers, or values. Existing safe `tracing` targets and fixed categories are authoritative.
- Use `Notify`, barriers, held permits/connections, or controlled fakes for ordering and failure tests. Do not use sleeps as synchronization proof.
- Avoid `unwrap`/`expect` in production code, unsafe Rust, broad lint allowances, floating-point values, new dependencies, and compatibility shims.

### Current Files To Update And Preserve

| Path | Current state | Required change or preservation |
| --- | --- | --- |
| `src/runtime.rs` | Owns `CleanupHealth`, `ShutdownCoordinator`, signal worker, cleanup workers, Axum drain, WAL checkpoint, and pool close. It currently proceeds from worker shutdown directly to checkpoint/pool close. | Add the smallest empty mutation-registry/barrier seam and wait for it before checkpoint. Preserve admission-first ordering, ten-second drain, supervisor classification, trigger taxonomy, safe logs, and checkpoint/pool-close failure handling. |
| `src/composition.rs` | Builds `BuiltApp`, the singleton session/token stores, readiness service, router, and root `RuntimeControl`; no mutation registry is retained. | Compose exactly one root-owned registry/lifecycle handle if needed and carry it through `BuiltApp`. Do not duplicate web state owners or add application ports. |
| `src/main.rs` | Owns local startup and root composition tests, including `real_socket_smoke_covers_login_authenticated_read_and_bounded_shutdown`; current smoke logs out and then sleeps before shutdown. | Change the smoke to keep the authenticated cookie through coordinated shutdown, remove the timing sleep, assert empty-registry/resource ordering and safe completion, and retain restart/session invalidation coverage. Add deterministic ordering/log tests in this root boundary. |
| `debtor-web/src/state.rs` | `RuntimeControl` is a narrow cloneable user-admission/failure callback handle. | Preserve its narrow API. Do not leak root registry, `ShutdownCoordinator`, Axum, Tokio, or SQLx types into application-facing state. Only adjust it if a genuinely narrow outer-layer callback is required. |
| `debtor-web/src/middleware.rs` | Owns admission, mutation preflight, safe-read/login/probe timeout classes, and safe HTTP diagnostics. | Preserve all completed Story 1.8 behavior and the no-generic-timeout-after-dispatch rule. Do not add route-local shutdown or post-dispatch cancellation. |
| `debtor-web/src/router.rs` | Separates probes, login, protected routes, and static assets; probes have independent four-request admission and user admission is outside session loading. | Preserve router/layer ordering, probe session-free behavior, and closed-admission `503` behavior. No new route or UX work. |
| `tests/restart.rs` | Process-boundary restart test starts the real binary and verifies persistent DB/migrations. | Retain and run it; change only if the new shutdown completion contract requires a focused assertion. Do not replace deterministic composition coverage with polling changes unrelated to this story. |
| `debtor-infra/src/db/repos/snapshots.rs` | Provides checked SQLite readiness and existing storage adapter behavior. | No change expected. If checkpoint/storage diagnostics must be adapted, keep raw adapter details inside infra and use existing safe categories. |
| `_bmad-output/implementation-artifacts/deferred-work.md` | Records that forced-drain testing does not yet prove server-side request cancellation. | Revisit this specific deferred item through implementation/tests; update only if the repository convention records its resolution. |

Expected unchanged areas: `debtor-domain`, financial application use cases, migrations, `.sqlx`, exchange-rate provider code, Cargo manifests, lockfiles, templates, CSS, and static assets.

### Mutation Boundary and Story 2.1 Handoff

The existing web code has mutation routes and `MutationPreflight::dispatch()`, but there is no root dispatched-mutation registry or authoritative terminal outcome publication. Do not mistake the web preflight marker for the required root lifecycle registry. For this story:

- The registry must be observable as empty during the authenticated smoke.
- No fake mutation may be inserted to make the barrier non-empty.
- No handler needs to be moved to an executor yet unless a minimal compile-safe seam is unavoidable; real Group creation dispatch belongs to Story 2.1.
- Do not claim `SPEC-FR103` or the real-mutation portion of `SPEC-FR104` complete.
- Preserve the future contract: authoritative `Committed`/`RolledBack` must be published synchronously before response work; if outcome is not established it is `Unknown`, never rollback, and shutdown is fatal with automatic retry suppressed. This story only prevents the lifecycle boundary from making any false outcome.

### Testing Requirements

Test invariants in their owning layer and keep the root smoke as the composition contract.

- **Root lifecycle:** verify admission closes before drain; probes remain available during drain; the registry is empty; checkpoint runs after the barrier; pool close follows checkpoint; and normal shutdown returns success.
- **Authenticated smoke:** bind `127.0.0.1:0`, use a temporary file-backed SQLite database, use the real `build_app`/runtime composition, disable redirects in `reqwest`, parse the login CSRF/submission fields from rendered HTML, post the password once, fetch `/groups` with the authenticated cookie, request shutdown, and await the runtime. Do not call logout in the primary path and do not use `sleep` for ordering.
- **Forced drain:** hold a server-side request with `Notify`, initiate shutdown, wait for the configured short test drain bound, and assert the server-side hold is released/dropped through an explicit guard or notification. Aborting only the client task is insufficient.
- **Checkpoint failure:** use a file-backed WAL database, hold a read snapshot, create a WAL frame, invoke the full runtime shutdown path with a short injected checkpoint timeout, assert a fatal result, verify `-wal`/`-shm` are still present, release the snapshot, reopen the same path, and verify committed state remains readable.
- **Supervisor regression:** preserve tests for cleanup error, panic, cancellation, unexpected return, readiness failure, exactly-once shutdown request, and normal worker return after coordinator shutdown.
- **Logging:** use a test subscriber or existing safe logging hooks and assert output excludes all forbidden secret, identity, SQL, value, query-string, provider-URL, and raw-error classes. Do not put sentinel secrets in production logs or fixtures.
- Use barriers, notifications, held connections, or held semaphores. Timing bounds may be tested with injected short timeouts, but elapsed sleeps must not establish correctness.

### Library / Framework Requirements

- Versions are pinned by `Cargo.lock` and project context: Rust 1.97.1/edition 2024, Axum 0.8.9, Tokio 1.53.1, Tower 0.5.3, tower-http 0.6.11, SQLx 0.9.0, tower-sessions 0.15.0. Do not upgrade or add a framework dependency.
- Tokio guidance: supervise `JoinHandle`s by inspecting `JoinError::is_cancelled()` and `JoinError::is_panic()`; coordinate stop and completion with `select!`, `Notify`, or task tracking rather than sleeps. Never stringify panic payloads or raw task errors.
- Axum guidance: `with_graceful_shutdown` signals accept-loop shutdown and drains in-flight connections; its future resolves after the drain. Preserve the existing explicit ten-second bound and do not assume a client disconnect proves server task cancellation.
- SQLx guidance: await `SqlitePool::close()` for resource completion. Keep the fixed WAL PRAGMA isolated, bounded, and before pool close. SQLx query macros remain checked everywhere else.
- Current documentation consulted through Context7: Tokio `/tokio-rs/tokio` graceful-shutdown and `JoinError` guidance; Axum `/tokio-rs/axum` `with_graceful_shutdown` semantics; SQLx `/websites/rs_sqlx` SQLite pool close semantics.

### Previous Story Intelligence

Story 1.8 completed the admission/readiness/supervision foundation and had all review findings resolved. Preserve these lessons:

- Keep exactly one root-composed `RuntimeControl`; admission is outside session middleware, and probes bypass user admission.
- Cleanup stores remain process-local, indexed, shared, and supervised. A coordinator-driven normal worker return is not a failure; unexpected exit, cancellation, and panic are failures.
- Readiness failure must mark health unhealthy, close user admission, and request shutdown exactly once. Do not conflate readiness failure with cleanup failure.
- Keep the fixed 64 user, 4 login, and 4 probe budgets; keep body limits and timeout classes; never introduce a generic post-dispatch timeout.
- Existing tests intentionally use barriers, `Notify`, semaphores, and controlled fakes. Do not copy the old smoke-test sleep into this story.
- The Story 1.8 file list and commit are the current implementation baseline, but inspect actual code because earlier story file lists omitted changed paths.

### Git Intelligence

- Current `HEAD`: `ff80a9c` (`feat: impelement bmad 1-8`), with a clean worktree at analysis time.
- `ff80a9c` changed `src/runtime.rs`, `src/main.rs`, `src/composition.rs`, and web admission/readiness/router files. Build on these completed patterns rather than reconstructing Stories 1.6-1.8 from prose.
- `b3b3628` generalized the shared authenticated submission-token boundary across forms. Do not create a second replay boundary or change token reservation semantics.
- `fdaca09` established authenticated sessions, logout, cleanup groundwork, and the authenticated socket path. The new smoke must preserve stale-cookie invalidation after restart.
- Earlier `f699809` and `0a79328` established password validation, local startup, SQLite composition, and restart coverage.

### Project Structure Notes

- Root runtime changes belong in `src/runtime.rs`, composition wiring in `src/composition.rs`, and composed integration tests in `src/main.rs` unless an existing test module owns the invariant better.
- `debtor-web` must remain an HTTP adapter. `debtor-application` and `debtor-domain` must not learn about shutdown coordinators, Tokio tasks, Axum servers, sessions, or SQLx pools.
- Preserve plural feature-module naming and existing `RuntimeControl`, `ShutdownCoordinator`, `CleanupHealth`, `BuiltApp`, `*Store`, `*Gate`, and `*UseCases` naming conventions.
- No migration or `.sqlx` change is expected. If that changes, update `specs/design.md` first, migrate a temporary SQLite database, run online `cargo sqlx prepare --workspace --check`, and commit metadata.

### References

- [Source: `_bmad-output/planning-artifacts/epics.md#Story 1.9: Shut Down the Authenticated Runtime Safely`]
- [Source: `_bmad-output/planning-artifacts/epics.md#Final-Evidence Ledger`]
- [Source: `specs/design.md#Architecture`]
- [Source: `specs/design.md#Security`]
- [Source: `specs/design.md#Operational Limits`]
- [Source: `specs/adr/0001-foundation-architecture.md#11. Local readiness and shutdown`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-6 - Single ledger runtime and mutation epoch`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-13 - Process-local owner uniqueness`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-14 - Bounded operational envelope`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-15 - Safe failures, logging, and readiness`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-16 - Injected effects and layer-owned verification`]
- [Source: `_bmad-output/planning-artifacts/prds/prd-debtor-2026-08-10/addendum.md#Admission, Timeouts, Probes, And Shutdown`]
- [Source: `_bmad-output/planning-artifacts/prds/prd-debtor-2026-08-10/addendum.md#Testing Contract`]
- [Source: `_bmad-output/implementation-artifacts/1-8-expose-health-readiness-and-bounded-admission.md#Current Files To Update And Preserve`]
- [Source: `_bmad-output/implementation-artifacts/1-8-expose-health-readiness-and-bounded-admission.md#Previous Story Intelligence`]
- [Source: `_bmad-output/implementation-artifacts/deferred-work.md`]
- [Source: `src/runtime.rs`]
- [Source: `src/composition.rs`]
- [Source: `src/main.rs`]
- [Source: `debtor-web/src/state.rs`]
- [Source: `debtor-web/src/middleware.rs`]
- [Source: `debtor-web/src/router.rs`]
- [Source: `tests/restart.rs`]
- [Source: Context7 `/tokio-rs/tokio` graceful shutdown and `JoinError` documentation]
- [Source: Context7 `/tokio-rs/axum` `with_graceful_shutdown` documentation]
- [Source: Context7 `/websites/rs_sqlx` SQLite `Pool::close` documentation]

## Dev Agent Record

### Agent Model Used

openai/gpt-5.6-luna

### Implementation Plan

- Add one root-owned `DispatchedMutationRegistry` with admission closure, lease tracking, and a notification-backed empty barrier; compose it in `BuiltApp` without wiring a real ledger mutation consumer.
- Close registry admission alongside user admission, force-cancel only safe in-flight reads after the bounded drain, wait for the empty registry, then checkpoint WAL and await pool closure.
- Preserve all existing supervisor, readiness, probe, authentication, session, token, and mutation pre-dispatch behavior.
- Add deterministic registry, forced-drain, unsafe-request non-cancellation, authenticated real-socket, and full checkpoint-recovery coverage.

### Debug Log References

- Ultimate context engine analysis completed - comprehensive developer guide created.
- Selected the first backlog story in complete sprint order: `1-9-shut-down-the-authenticated-runtime-safely`.
- Loaded project context, normative design contract, Epic 1, PRD shutdown addendum, architecture spine and ADR, Story 1.8, deferred work, current runtime/composition/web/test files, and recent commit history.
- Consulted current Tokio, Axum, and SQLx documentation through Context7. No dependency update is authorized.
- Red phase: registry and server-side forced-drain tests failed before their implementations; green phase passed after the lifecycle seam and safe-read cancellation signal were added.
- Added a test-only storage-timeout seam to exercise full runtime checkpoint failure/recovery without changing production five-second bounds.

### Completion Notes List

- Ultimate context engine analysis completed - comprehensive developer guide created.
- Story scope is limited to authenticated no-active-mutation shutdown; real dispatched-ledger mutation outcome evidence remains assigned to Story 2.1.
- Added root-owned mutation registration closure and notification-backed empty barrier, with no fake mutation outcome or real ledger executor integration.
- Added bounded forced-drain cancellation for safe `GET`/`HEAD` work while explicitly preserving unsafe request execution after dispatch.
- Updated the authenticated real-socket smoke to retain authentication through shutdown, remove logout and timing sleeps, and verify admission/registry completion.
- Added full-path WAL checkpoint failure, sidecar preservation, recovery, and server-side request cancellation tests.
- Validation passed: workspace format check, locked check, offline strict Clippy, complete workspace tests, architecture fitness, and independent password-helper format/Clippy/tests.
- Code review patches resolved: smoke now observes admission closure before checkpoint start, checkpoint failure is asserted by fatal trigger, and new lifecycle SQL evidence uses checked SQLx metadata plus application-level group use cases.
- Final review validation passed: formatting, locked workspace check, strict offline Clippy, complete workspace tests, architecture fitness, and all-target SQLx metadata check.

### File List

- `_bmad-output/implementation-artifacts/1-9-shut-down-the-authenticated-runtime-safely.md`
- `_bmad-output/implementation-artifacts/deferred-work.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `.sqlx/query-294cf42d65023d91e29d63302e682659035a5eea27ed466fe869c2c7f26a3d9c.json`
- `src/runtime.rs`
- `src/composition.rs`
- `src/main.rs`
- `debtor-web/src/state.rs`
- `debtor-web/src/middleware.rs`

### Change Log

- 2026-08-14: Implemented Story 1.9 shutdown registry/barrier, bounded safe-request forced drain, authenticated shutdown smoke coverage, WAL recovery coverage, and validation suite; status moved to `review`.
- 2026-08-14: Resolved code review findings, refreshed SQLx metadata for checked test SQL, recorded deferred Story 2.1 mutation-executor scope, and moved status to `done`.
