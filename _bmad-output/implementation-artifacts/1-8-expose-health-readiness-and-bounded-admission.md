---
story_key: 1-8-expose-health-readiness-and-bounded-admission
story_id: 1.8
epic: 1
status: done
baseline_commit: b3b3628cfcc6420d57c8730dc332499110488085
created: 2026-08-14
---

# Story 1.8: Expose Health, Readiness, and Bounded Admission

Status: done

## Story

As the administrator,
I want Debtor to report trustworthy health while bounding incoming work,
so that overload or failed mandatory maintenance cannot leave the service falsely ready.

## Acceptance Criteria

1. **Liveness:** `GET /healthz` returns process liveness within the two-second outer probe timeout. It does not load sessions, query SQLite, inspect ledger contents, call Frankfurter, or depend on user admission. It uses the independent four-request probe budget.
2. **Healthy readiness:** `GET /readyz` returns ready within the two-second outer timeout when the one-second SQLite readiness check succeeds and every mandatory cleanup supervisor is healthy. Provider availability and ledger contents are never consulted.
3. **Readiness failure:** SQLite readiness failure or unhealthy session-expiry/submission-token cleanup causes `/readyz` to return sanitized `503`, closes new user admission, and begins coordinated shutdown. `/healthz` remains available and truthful until process exit.
4. **Independent capacity:** User traffic is capped at 64 in-flight requests and login at four. Probes have a separate four-request budget; up to four probes remain admissible when user/login capacity is exhausted. Probe saturation returns the existing sanitized overload response and never creates or loads a session.
5. **Timeout classes:** Login and ordinary safe dynamic reads use the 30-second class, Debts keeps its 90-second class, and probes use a two-second outer timeout with a one-second SQLite readiness timeout. Ledger mutations retain the 30-second absolute pre-dispatch deadline and are not cancelled by a generic timeout after dispatch.
6. **Bounded cleanup:** Anonymous sessions, authenticated sessions, and both submission-token pools are physically reaped through their existing indexed expiry structures at the mandatory five-minute cadence. Cleanup is bounded by the fixed process-local stores, does not evict live authenticated records, and exposes only supervisor health, never records or counts, to readiness.
7. **Supervisor failure:** A cleanup iteration error, panic, cancellation, join failure, or unexpected worker exit marks supervisor health unhealthy, closes user admission, and initiates the cleanup-failure shutdown path exactly once. Logs contain only fixed supervisor/operation categories and no secrets, identifiers, request data, or raw errors.
8. **Deterministic verification:** Tests use held permits, barriers, notifications, or controlled fake dependencies, never timing sleeps, to prove readiness/liveness separation, admission closure, cleanup failure handling, slow SQLite readiness, provider independence, probe capacity, and zero probe session activity.

**Requirements:** `SPEC-FR96..SPEC-FR102`; `SPEC-NFR1..SPEC-NFR4`, `SPEC-NFR25..SPEC-NFR27`, `SPEC-NFR31..SPEC-NFR34`; separate probe admission, bounded user/login/form resources, timeout classification, mandatory indexed cleanup supervision, safe diagnostics, and deterministic concurrency testing. No UX IDs apply because probes and supervisors are machine/operator interfaces rather than rendered Administrator controls.

**Scope boundary:** Retain the existing probe/readiness ports, session/token stores, and root lifecycle coordinator while completing their runnable admission and supervision behavior. Do not add ledger/domain/application financial work, provider readiness, new routes, persistent state, new dependencies, or final real-mutation shutdown evidence owned by Stories 1.9 and 2.1.

## Tasks / Subtasks

- [x] Add one process-local runtime admission control boundary (AC: 3-5, 7)
  - [x] Compose exactly one cloneable control owner at the root and pass only a narrow outer-layer handle into `debtor-web`; do not add an application port or duplicate per-route gates.
  - [x] Keep user admission state separate from probe admission. The control must close login, protected routes, and static assets after shutdown/readiness failure while leaving `/healthz` and `/readyz` probe capacity available for orchestration.
  - [x] Reject newly arriving user work before session middleware, authentication, token issuance, or handler side effects with the existing sanitized retryable `503` response. Already admitted requests are not retroactively cancelled by this gate.
  - [x] Make closure and shutdown notification idempotent. Multiple observations of one failure must produce one lifecycle initiation, not repeated cleanup/shutdown actions.

- [x] Complete probe and readiness behavior (AC: 1-4, 8)
  - [x] Preserve the session-free public probe router and its independent `ConcurrencyLimitLayer::new(4)`; do not merge it into the 64-request user semaphore.
  - [x] Keep the liveness handler constant and allocation-light: no `State`, session extractor, readiness service, database, provider, or ledger access.
  - [x] Keep readiness application-owned as a narrow database/supervisor check, with SQLite `SELECT 1` under the existing one-second inner timeout and safe error categories.
  - [x] On readiness failure, invoke the outer runtime-control callback/signal that closes user admission and requests coordinated shutdown; do not put root shutdown types or Axum/SQLx types in `debtor-application`.
  - [x] Test readiness under held user permits and confirm probes remain available; test the fifth held probe is rejected without a session cookie or session-store load.

- [x] Supervise both mandatory cleanup workers while the server is serving (AC: 2, 3, 6, 7)
  - [x] Preserve one five-minute session cleanup worker and one five-minute submission-token cleanup worker, using `ExpiredDeletion` and `SubmissionTokenCleanup` respectively.
  - [x] Continue using indexed expiry deletion in `ReapingMemoryStore` and `SubmissionTokenStore`; do not replace it with full-map scans, per-feature workers, global eviction, or persistence changes.
  - [x] Monitor each spawned `JoinHandle` concurrently with the HTTP server, not only after HTTP drain. Distinguish normal return after coordinated shutdown from unexpected return, cancellation, and panic using `JoinError` state.
  - [x] For an iteration error or task failure, atomically mark shared cleanup health false before requesting `CleanupFailure`; readiness must observe the unhealthy state immediately.
  - [x] Preserve the current shutdown ordering: stop admission, drain HTTP for at most ten seconds, stop workers, wait for dispatched-mutation lifecycle owned by the later runtime story, checkpoint WAL, then close the pool. Do not add generic post-dispatch cancellation or a fixed total deadline for future dispatched mutations in this story.

- [x] Reconcile timeout and layer ordering (AC: 4-5)
  - [x] Keep `MutationPreflight` as the sole 30-second absolute pre-dispatch boundary for unsafe mutations and preserve its dispatch marker.
  - [x] Ensure no `TimeoutLayer` wraps a ledger mutation after its first state-changing use-case call. Login is not a ledger mutation and must still satisfy the 30-second login class; use the existing preflight/deadline behavior or a safe route-specific composition without weakening completed Login semantics.
  - [x] Preserve the 90-second `/groups/{id}/debts` read timeout and 30-second ordinary protected read timeout.
  - [x] Verify layer order explicitly. Axum `ServiceBuilder` layers execute top-to-bottom, while repeated `.layer(...)` calls wrap in reverse order; admission must be outside session-loading middleware, and probe layers must remain outside all user/session layers.

- [x] Add composed and adapter-level tests (AC: 1-8)
  - [x] Root tests: normal readiness, readiness failure triggering admission closure/shutdown, health remaining live, cleanup worker error, panic, cancellation/unexpected exit, and exactly-once failure initiation.
  - [x] Web/router tests: 64 held user permits, four held login permits, four independent probes, fifth probe rejection, post-failure rejection before session load, no probe cookie, and no provider call.
  - [x] Readiness tests: healthy/unhealthy supervisor ordering, successful SQLite check, held/slow SQLite connection returning sanitized failure within the outer bound, and provider/ledger independence.
  - [x] Cleanup store tests: indexed deletion of expiring anonymous/authenticated sessions and both token pools, capacity reuse, live authenticated-session preservation, five-minute supervisor cadence wiring, and safe failure mapping.
  - [x] Retain the real-socket startup/authenticated-read/shutdown smoke path. Extend it only for Story 1.8 evidence; do not claim the later real-ledger-mutation shutdown evidence owned by Story 2.1.
  - [x] Coordinate concurrent tests with `Barrier`, `Notify`, held semaphore permits, or controllable fakes. Do not copy the existing smoke-test sleep into new admission/supervision proofs.

### Review Findings

- [x] [Review][Patch] Cleanup task failure can be masked by a concurrent shutdown request [src/runtime.rs:267-286] — fixed by treating only `Ok(())` during coordinated shutdown as expected and classifying all `JoinError` outcomes as supervisor failures.
- [x] [Review][Patch] Supervisor join and forced-abort failures do not mark cleanup health unhealthy [src/runtime.rs:414-437] — fixed by routing supervisor join/timeout failures through the shared fail-closed cleanup-health transition.
- [x] [Review][Patch] Probe endpoints become unreachable as soon as graceful shutdown starts [src/runtime.rs:392-412] — fixed by keeping the server accepting probe traffic during the bounded drain and stopping it only at the drain deadline.
- [x] [Review][Patch] Readiness timeout does not close admission or begin shutdown [debtor-web/src/middleware.rs:274-279] — fixed by making readiness timeout responses sanitized `503` and invoking runtime failure signaling from the timeout layer.
- [x] [Review][Patch] `/readyz` can return `200` after runtime admission has closed [debtor-web/src/handlers/health.rs:14-29] — fixed by returning sanitized not-ready before dependency checks when runtime admission is closed.
- [x] [Review][Patch] Production composes two runtime admission middleware boundaries [debtor-web/src/router.rs:133-153; src/composition.rs:163-175] — fixed by keeping the web router constructor neutral and applying one admission boundary at each complete composition (`router` for direct web tests, root composition for production).
- [x] [Review][Patch] Readiness failures are reported as cleanup failures [src/main.rs:59-63] — fixed with a distinct `ReadinessFailure` shutdown trigger and category.
- [x] [Review][Patch] Readiness failure can race with intentional signal shutdown [debtor-web/src/state.rs:76-87; src/main.rs:57-63] — fixed with an atomic coordinator `request_if_unrequested` path.
- [x] [Review][Patch] New concurrency tests rely on timer sleeps [src/main.rs:579-631; debtor-web/src/middleware.rs:446-475] — fixed with direct cleanup-iteration helpers, `Notify`, `Semaphore`, and pending futures.
- [x] [Review][Patch] Composition wiring from real `/readyz` failure to root shutdown is untested [debtor-web/src/router.rs:1240-1266; src/main.rs:640-665] — fixed with an actual web handler callback test and drain-time health probe coverage.
- [x] [Review][Patch] Required capacity/isolation evidence is incomplete [debtor-web/src/router.rs:200-275] — fixed with deterministic four-login saturation, independent probe, no-cookie, and session-store-gated tests.

## Dev Notes

### Developer Context

This is a brownfield completion story. Existing code already has most constants and local adapters, but the runtime contract is incomplete:

- `src/runtime.rs` marks cleanup health false and requests shutdown when an iteration returns an error, but spawned worker panics/unexpected exits are only noticed after HTTP drain.
- `src/composition.rs` shares one 64-permit semaphore between protected user routes and static assets, while login has a separate four-request limit and probes have a separate four-request limit.
- `debtor-web/src/handlers/health.rs` returns sanitized readiness failure but does not currently close user admission or notify root lifecycle coordination.
- `debtor-application/src/readiness.rs` correctly owns only the narrow SQLite/supervisor check and must remain framework-free.
- `debtor-infra/src/db/repos/snapshots.rs` already performs checked `SELECT 1` readiness with a one-second timeout; preserve it unless a test exposes a concrete contract gap.
- `debtor-web/src/session_store.rs` and `submission_tokens.rs` already use bounded indexed expiry state and must remain the sole cleanup owners.

The intended flow is:

```text
probe: independent 4-permit budget -> 2s outer timeout -> /healthz or /readyz
user: runtime admission-open check -> route-specific login/protected/static budget -> session/auth middleware -> handler
readiness failure: safe 503 -> close user admission once -> request coordinated shutdown once
cleanup failure/panic/exit: mark health unhealthy -> close user admission -> request CleanupFailure -> keep probes live during drain
```

Readiness is not merely a status endpoint. A failed readiness evaluation is an operational state transition. Keep the check and the lifecycle signal separate: application code reports a safe reason, web/root composition performs admission closure and shutdown notification.

### Technical Requirements

- Preserve the permanent single-administrator model. No users, tenants, participant authentication, registration, or authorization abstraction.
- Preserve `root -> debtor-web/debtor-infra -> debtor-application -> debtor-domain`. Root owns composition, supervision, startup, and shutdown; web owns HTTP layers and response mapping; application owns the readiness port/service; infra owns the SQLite adapter.
- Use atomics/notifications or an equivalent narrow control handle for process-local admission state. Do not use a database flag, session state, provider call, or application-domain singleton.
- Admission closure must be fail-closed and race-safe: requests that observe closed state receive sanitized `503`; a request already admitted may finish according to its existing timeout/dispatch contract.
- `CleanupHealth` must be shared by the session and token readiness checks. The first failure transition should be observable without exposing which records were affected.
- Treat task cancellation and panic as failure when they occur before coordinated shutdown. A normal worker return caused by the coordinator is not a failure.
- Keep fixed budgets: 64 user, 4 login, 4 probes; 8 KiB login body, 256 KiB other forms; 30-second login/ordinary reads, 90-second Debts, two-second probes, one-second SQLite readiness.
- Keep SQLx query macros checked. No migration, database schema, monetary query, provider, rate-cache, dependency, or `.sqlx` changes are expected.
- Never log session IDs, CSRF/submission tokens, cookies, client IPs, limiter keys, SQL/database messages, provider URLs, identifiers, values, request paths with query strings, or raw task/persistence errors. Use fixed categories such as `cleanup_failure`, `runtime_supervisor`, `storage_contention`, and `request_admission_rejected`.
- Avoid `unwrap`/`expect` in production paths, unsafe Rust, broad lint allowances, floating point, and generic timeout cancellation after mutation dispatch.

### Architecture Compliance

- **AD-13:** exactly one root-composed session store, submission-token store, admission/probe budgets, cleanup-health state, and runtime lifecycle owner. Do not introduce a second route-local semaphore or supervisor.
- **AD-14:** fixed body/concurrency/timeout envelope; user and probe budgets are independent; mutations have only a pre-dispatch deadline.
- **AD-15:** `/healthz` is liveness; `/readyz` checks SQLite and mandatory cleanup supervisors only; Frankfurter and ledger contents never gate readiness; safe diagnostics are fixed-category only.
- **AD-16:** keep use-case/readiness checks fakeable and keep runtime/web/infrastructure tests in their owning layers.
- **AD-1/AD-2:** no Axum, SQLx, tower-sessions, or root coordinator types cross application-owned ports. If web needs a lifecycle callback, define a narrow outer-layer handle/trait and inject it from root rather than coupling `debtor-application` to runtime.
- **AD-18:** no UX contract applies to machine/operator probe routes. Do not add rendered Administrator controls, custom JavaScript, HTMX behavior, or a dashboard for this story.

### Library / Framework Requirements

- Locked versions remain authoritative: Rust 1.97.1/edition 2024, Axum 0.8.9, Tokio 1.53.1, Tower 0.5.3, tower-http 0.6.11, tower-sessions 0.15.0, SQLx 0.9.0. Do not upgrade dependencies for this story.
- Current Axum guidance: compose probe, login, protected, and static routers separately and merge them; use `ServiceBuilder` when layer order matters. `HandleErrorLayer` must be placed so it receives errors from load-shed/timeout layers. Request extensions are appropriate for narrow request-scoped control data, but body-consuming extractors remain last.
- Current Tokio guidance: inspect `JoinError::is_cancelled()` and `JoinError::is_panic()` when supervising `JoinHandle`s; use `select!`/`Notify` for coordinated stop signals rather than sleeps. Do not stringify panic payloads or raw task errors into logs.
- Current tower-sessions guidance: `SessionStore::load` treats expired sessions as absent; `ExpiredDeletion::delete_expired` is the hard-delete background cleanup contract. Preserve both lazy request-time expiry filtering and supervised indexed physical cleanup.
- Use the existing Tower `ConcurrencyLimitLayer`, `GlobalConcurrencyLimitLayer`, `load_shed`, `HandleErrorLayer`, `RequestBodyLimitLayer`, and timeout composition instead of adding a custom HTTP server or dependency.

### Current Files To Update And Preserve

| Path | Current state | Story change / preservation |
|---|---|---|
| `src/runtime.rs` | Owns `CleanupHealth`, `ShutdownCoordinator`, two cleanup workers, signal worker, HTTP drain, WAL checkpoint, and pool close. Worker handles are joined only after drain. | Add live supervision and idempotent failure propagation; preserve trigger taxonomy, five-minute workers, drain/checkpoint/pool-close order, safe logs, and later mutation barrier boundary. |
| `src/composition.rs` | Builds `BuiltApp`, readiness service, one session/token owner, 64-permit user/static path, and router. | Compose one admission-control handle and inject it into web plus runtime; keep singleton owners and no new application port. |
| `src/main.rs` | Builds runtime, binds listener, runs root tests and real-socket smoke tests. | Update `BuiltApp` helpers and add deterministic composed admission/supervisor tests; retain startup ordering and smoke coverage. |
| `debtor-web/src/router.rs` | Public probes have a separate four-request layer; login has four; protected/static user path has 64. | Preserve probe isolation; add user admission gate to login/protected/static paths before session middleware and test exact capacity behavior. |
| `debtor-web/src/middleware.rs` | Has mutation preflight, auth/session middleware, safe-read/debts timeout, login timeout, probe timeout, and overload mapping. | Add or wire the narrow admission check without changing mutation dispatch semantics; verify all timeout classes and layer ordering. |
| `debtor-web/src/handlers/health.rs` | Liveness is constant; readiness maps safe errors to `503` but only logs. | On readiness failure, invoke injected outer runtime control exactly once while retaining sanitized response/category mapping. |
| `debtor-web/src/state.rs` | Holds application use cases, readiness, proxy, and submission-token store. | Add only the narrow runtime/admission control needed by web; do not expose root, SQLx, or session-store internals. |
| `debtor-application/src/readiness.rs` | Narrow `DatabaseReadiness`, `ReadinessUseCases`, and `SupervisorReadiness` ports; supervisor is checked before database. | Preserve this API and add only isolated fake-backed tests or a minimal contract adjustment justified by compilation. |
| `debtor-infra/src/db/repos/snapshots.rs` | Checked `SELECT 1` with one-second timeout implements `DatabaseReadiness`. | Preserve query and safe storage mapping; add adapter tests only if needed for held connection/timeout behavior. |
| `debtor-web/src/session_store.rs` | Bounded anonymous/authenticated records with `BTreeMap` expiry index and `delete_expired`. | Preserve indexed deletion, capacity isolation, lazy expiry, and no live-session eviction; do not add unbounded scans. |
| `debtor-web/src/submission_tokens.rs` | One owner with isolated anonymous/authenticated pools and indexed cleanup; cleanup adapter is infallible in production. | Preserve token state machine and pool bounds; use it as the supervised worker target and test failure through fakes. |
| `specs/design.md` and companions | Normative contract already defines this story's behavior. | Do not edit unless implementation reveals a genuine contract divergence; if behavior must change, update this file first and synchronize companions before code. |

Files expected to remain unchanged: `debtor-domain`, financial application use cases, migrations, `.sqlx`, provider/rate code, Cargo manifests, and lockfiles.

### Testing Requirements

- Root/composition tests must prove failure transitions through the real router/runtime composition, not only unit-test an atomic flag.
- Use `Notify` to hold a fake SQLite readiness operation, held semaphore permits to saturate user/login traffic, and a fake cleanup worker or controlled task to produce error/panic/early return. Release controls explicitly; never use elapsed-time sleeps as proof.
- Prove `/healthz` does not invoke readiness, database, provider, session, or token stores. Prove `/readyz` does not invoke provider/ledger reads and does invoke only SQLite/supervisor checks.
- Prove a readiness failure returns `503`, closes newly arriving user/login/static requests before session load/token issuance, leaves probes independently reachable, and initiates one shutdown transition.
- Prove cleanup health becomes false before readiness reports failure and that all subsequent `/readyz` responses remain sanitized.
- Prove normal worker return after an explicit coordinator shutdown is not misclassified as cleanup failure; prove panic, cancellation, and unexpected return before shutdown are failures.
- Test four concurrent probes are admitted independently while all 64 user permits and all four login permits are held. Test the fifth probe receives the existing safe overload response and creates no session.
- Retain existing web tests for no probe cookies, body limits, 30/90-second read classes, and no generic timeout after mutation dispatch. Update only assertions that necessarily reflect the new admission gate.
- Run the production workspace checks required by the repository guide: `cargo fmt --all -- --check`, `cargo check --workspace --all-features --locked`, `SQLX_OFFLINE=true cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`, `cargo test --workspace --all-features --locked`, and `cargo run --bin architecture-check --locked`.
- No SQLx prepare command is needed unless SQL or migrations change. Do not run `cargo build --release`.

### Previous Story Intelligence

Story 1.7 generalized one submission-token owner, added page-scoped authenticated tokens, moved all current unsafe forms through one reservation/dispatch boundary, and preserved session/token cleanup wiring. Its critical lessons apply here:

- Do not create route-local stores, per-form/per-row token owners, or a second replay boundary.
- Cleanup remains process-local, indexed, shared, and supervised. Story 1.7 explicitly did not claim Story 1.8 readiness/admission or Story 1.9 shutdown completion.
- Preserve token terminal reservation, session-flush deletion, authenticated/anonymous pool isolation, and all existing Login/Sign-out behavior.
- Existing tests use barriers/atomics and fakes; browser geometry is not executable in this repository and must not be claimed as automated.
- The Story 1.7 file list omitted `debtor-web/src/handlers/spending_views.rs` even though commit `b3b3628` changed it. Use the actual current tree and commit diff, not that list alone, when checking regressions.

### Git Intelligence

- Current `HEAD`: `b3b3628` (`feat: impelement bmad 1-7`); worktree was clean during analysis.
- `b3b3628` changed shared forms, token storage, router tests, handlers, templates/CSS, composition, and root tests. Build on those completed patterns; do not reconstruct Story 1.6/1.7 from prose.
- `fdaca09` established authenticated sessions, Sign-out, temporary authenticated token handling, session cleanup, and real-socket auth/logout coverage.
- `f699809` established password/login admission, initial submission-token storage, cleanup/runtime groundwork, and restart tests.
- `0a79328` established password validation and persistent local startup; `1bbfbc3` established the current planning/architecture correction.

### Latest Technical Information

- Context7 `/tokio-rs/axum`: `ServiceBuilder` layers execute top-to-bottom; repeated `.layer(...)` calls wrap in reverse order. Separate routers can carry distinct middleware and be merged, which is the appropriate shape for session-free probes versus user routes. `HandleErrorLayer` must wrap the layers whose errors it maps.
- Context7 `/tokio-rs/tokio`: `JoinHandle` failures must distinguish cancellation and panic through `JoinError`; `Notify` and `select!` support coordinated shutdown without timing sleeps. A cleanup worker that exits before the coordinator must be treated as failed supervision.
- Context7 `/maxcountryman/tower-sessions`: expired `SessionStore::load` results are treated as absent, while `ExpiredDeletion::delete_expired` provides physical background deletion. This supports retaining lazy expiry checks plus one mandatory supervised five-minute cleanup worker.
- The pinned `Cargo.lock` and project context remain the dependency/version authority. No upgrade or new library is justified.

### Project Context Reference

Read and follow `_bmad-output/project-context.md` and `specs/design.md` before implementation. Especially binding are: exact bounded operational limits, safe diagnostics, session-free probes, root-to-web/application layer boundaries, mandatory cleanup-supervisor failure policy, no generic timeout after mutation dispatch, deterministic concurrency tests, and the prohibition on `cargo build --release` for validation.

### References

- [Source: `_bmad-output/planning-artifacts/epics.md#Story 1.8: Expose Health, Readiness, and Bounded Admission`]
- [Source: `_bmad-output/planning-artifacts/epics.md#Requirements Inventory`]
- [Source: `specs/design.md#Operational Limits`]
- [Source: `specs/design.md#Security`]
- [Source: `specs/design.md#Architecture`]
- [Source: `_bmad-output/planning-artifacts/prds/prd-debtor-2026-08-10/addendum.md#Admission, Timeouts, Probes, And Shutdown`]
- [Source: `_bmad-output/planning-artifacts/prds/prd-debtor-2026-08-10/addendum.md#Testing Contract`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-13 - Process-local owner uniqueness`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-14 - Bounded operational envelope`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-15 - Safe failures, logging, and readiness`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-16 - Injected effects and layer-owned verification`]
- [Source: `_bmad-output/project-context.md#Framework-Specific Rules`]
- [Source: `_bmad-output/project-context.md#Testing Rules`]
- [Source: `_bmad-output/implementation-artifacts/1-7-extend-replay-protection-beyond-login-and-sign-out.md#Current Files To Update And Preserve`]
- [Source: `_bmad-output/implementation-artifacts/1-7-extend-replay-protection-beyond-login-and-sign-out.md#Previous Story Intelligence`]
- [Source: `src/runtime.rs`]
- [Source: `src/composition.rs`]
- [Source: `src/main.rs`]
- [Source: `debtor-web/src/router.rs`]
- [Source: `debtor-web/src/middleware.rs`]
- [Source: `debtor-web/src/handlers/health.rs`]
- [Source: `debtor-web/src/state.rs`]
- [Source: `debtor-application/src/readiness.rs`]
- [Source: `debtor-infra/src/db/repos/snapshots.rs`]
- [Source: `debtor-web/src/session_store.rs`]
- [Source: `debtor-web/src/submission_tokens.rs`]
- [Source: Context7 `/tokio-rs/axum`]
- [Source: Context7 `/tokio-rs/tokio`]
- [Source: Context7 `/maxcountryman/tower-sessions`]

## Dev Agent Record

### Agent Model Used

openai/gpt-5.6-luna

### Implementation Plan

- Added a cloneable web-owned `RuntimeControl` with atomic user-admission closure and one-shot root shutdown signaling.
- Applied the control outside session middleware to login, protected, and composed static traffic while explicitly bypassing the independent probe router.
- Connected readiness failure to admission closure and coordinated shutdown; supervised both indexed cleanup workers while the server remained live and classified worker panic/cancellation/unexpected exit safely.
- Preserved the existing SQLite/readiness ports, five-minute cleanup cadence, mutation pre-dispatch boundary, shutdown ordering, and no-provider/no-ledger probe behavior.
- Added deterministic barrier/semaphore/notification tests at web and root runtime boundaries.

### Debug Log References

- Ultimate context engine analysis completed - comprehensive developer guide created.
- Selected first backlog story in complete sprint order: `1-8-expose-health-readiness-and-bounded-admission`.
- Loaded project context, normative design contract, PRD addendum, complete Epic 1 context, architecture spine, final UX contracts, previous Story 1.7, current runtime/web/application/infra files, and recent git history.
- Consulted current Context7 guidance for Axum layer ordering/router composition, Tokio task supervision, and tower-sessions expiry cleanup. No dependency update is authorized.
- Red/green evidence: admission-control and probe-capacity tests were added before their production implementations; initial runs failed at unresolved API/behavior boundaries, then passed after implementation.
- Validation evidence: complete workspace tests, formatting, locked workspace check, offline Clippy with warnings denied, and architecture fitness all passed on 2026-08-14.

### Completion Notes List

- Ultimate context engine analysis completed - comprehensive developer guide created.
- Implemented bounded runtime admission with independent session-free probe capacity; closed login, protected, and static user traffic before session loading after shutdown/readiness failure.
- Connected readiness failure and cleanup-supervisor failure to one-shot admission closure and coordinated shutdown while preserving liveness until process exit.
- Added live cleanup-worker JoinHandle supervision for error, panic, cancellation, and unexpected exit, with indexed five-minute cleanup ownership unchanged.
- Applied the fixed 30-second login timeout to POST as well as GET while retaining the no-generic-timeout-after-dispatch rule for ledger mutations.
- Added deterministic runtime/web tests for admission closure, four-probe capacity, readiness failure, static routes, timeout classification, cleanup failures, supervisor task failure, and exactly-once shutdown signaling.
- Validation passed: `cargo fmt --all -- --check`, `cargo check --workspace --all-features --locked`, `SQLX_OFFLINE=true cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`, `cargo test --workspace --all-features --locked`, and `cargo run --bin architecture-check --locked`.
- No SQL, migration, dependency, lockfile, provider, domain, or financial application changes were made.
- Review resolution: all 11 actionable findings were patched and verified; no unresolved high/medium findings remain.

### File List

- `_bmad-output/implementation-artifacts/1-8-expose-health-readiness-and-bounded-admission.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `debtor-web/src/handlers/health.rs`
- `debtor-web/src/handlers/test_support.rs`
- `debtor-web/src/middleware.rs`
- `debtor-web/src/router.rs`
- `debtor-web/src/state.rs`
- `src/composition.rs`
- `src/main.rs`
- `src/runtime.rs`

### Change Log

- 2026-08-14: Implemented Story 1.8 runtime admission, readiness failure signaling, live cleanup supervision, timeout classification, and deterministic composed/web tests; status moved to `review`.
- 2026-08-14: Applied all 11 code-review patches covering supervisor races, drain-time probes, readiness lifecycle classification, admission composition, deterministic tests, and capacity/isolation evidence; status moved to `done`.
