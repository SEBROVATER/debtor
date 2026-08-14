---
story_key: 2-1-create-and-select-a-group
story_id: 2.1
epic: 2
status: done
baseline_commit: b71a15c450213ee8da6fe96add7b4293cd709e0c
created: 2026-08-14
---

# Story 2.1: Create and Select a Group

Status: done

## Story

As the administrator,
I want to create and select a Group,
so that I have a private ledger context in which to organize shared Spendings.

## Acceptance Criteria

1. With no active Groups, an authenticated home page renders an accessible empty state and a protected native Create Group form; archived Groups are excluded from the active list.
2. The Create Group form accepts only a Group name plus the shared CSRF and single-use submission-token fields. It must not expose Group Currency, user, membership, tenant, or Participant fields.
3. A name that trims to empty or exceeds 100 Unicode characters returns `422 Unprocessable Entity`, retains the raw submitted name, associates the inline error programmatically, and leaves the valid submission token usable because validation occurs before reservation/dispatch.
4. A valid name is parsed and normalized by application policy, then creates a positive `i64` active Group with `USD` Group Currency through the existing narrow repository port. Web code must not construct persistence or framework-owned domain state.
5. Group creation crosses the existing shared unsafe-request boundary: strict structural extraction, authentication, CSRF, and route validation happen before dispatch; the token is reserved and dispatch is marked immediately before the first state-changing use-case call.
6. The mutation runs through the one root-composed dispatched-mutation lifecycle owner and the infrastructure five-second write gate. SQLite persistence commits the Group atomically; the root executor publishes the authoritative committed or rolled-back outcome before response work.
7. A pre-dispatch body, authentication, CSRF, token, or asynchronous precheck timeout rejects without token reservation, use-case invocation, transaction opening, write-gate side effect, or mutation-epoch advancement. Deterministic tests prove zero dispatch.
8. Oversized, unauthenticated, malformed, missing/duplicate/unknown-field, CSRF-invalid, submission-token-invalid, and application-invalid requests are rejected at their owning boundary before Group creation dispatch. Valid form validation retains the token; invalid/replayed security tokens return the established conflict response without dispatch.
9. A dispatched mutation that fails unexpectedly publishes `RolledBack` only when rollback is authoritative; otherwise it publishes `Unknown`, requests fatal shutdown, suppresses automatic retry, and never represents an unknown result as rollback.
10. Gate acquisition that cannot complete within five seconds returns sanitized retryable feedback without opening a transaction or starting guarded persistence work.
11. Multiple valid creations are all represented in the active list; last committed state wins without optimistic revision columns, and the single process-local mutation epoch advances once per successful commit and never after rejection/rollback. Ordering exposed to the UI remains deterministic.
12. Group persistence structurally enforces supported currency, archive-flag shape, bounded non-empty text shape, positive identity, and relationships. Rust remains authoritative for trimming and Unicode character count. Checked SQLx metadata remains current.
13. Successful creation returns `303 See Other` to the new Group's contextual Manage destination. The new Group shell exposes five native destinations in fixed order: Groups, Summary, Transactions, Debts, Manage, with Manage current. Existing Group links select by URL; do not add a session-backed current-Group field.
14. A newly created Group with no Participant has Add Spending visibly disabled, with adjacent setup guidance and a 48-by-48 recovery link to Manage/Add Participant. Do not implement Spending capability or Participant lifecycle beyond the required shell state.
15. At 320 CSS pixels and 400% zoom, create-by-name precedes the empty active list, Archived Groups is a contextual text link, and every field/link/button/row target is at least 48 by 48 CSS pixels without clipped text or page-level horizontal scrolling. Use the Editorial Contrast tokens and square geometry.
16. On validation, the sole invalid field or linked `role="alert"` summary receives focus; stable label/guidance/error IDs remain associated, pending state clears, and native and optional HTMX responses use the same canonical markup. During a valid mutation, the initiator is unavailable, one scoped polite atomic status and `aria-busy` communicate pending, and the Manage heading is the stable autofocus target after redirect.

Requirements: `SPEC-FR14..SPEC-FR22`, `SPEC-FR25`, `SPEC-FR87..SPEC-FR89`, `SPEC-FR103..SPEC-FR104`; `SPEC-NFR3`, `SPEC-NFR7`, `SPEC-NFR15`, `SPEC-NFR21..SPEC-NFR22`, `SPEC-NFR25`, `SPEC-NFR31..SPEC-NFR34`; UX contracts `UX-SHELL-01`, `UX-TARGET-01`, `UX-FOCUS-01`, `UX-STATUS-01`, `UX-RESPONSIVE-01`, `UX-VISUAL-01`.

## Scope Boundary

- This is the first real ledger mutation consumer. Wire Group creation into the root mutation executor, registry/lease, authoritative outcome publication, and shutdown wait. Do not leave the Story 1.9 empty registry seam unused.
- Replace the superseded currency-on-create and `/groups` success redirect. Creation is name-only and always starts at USD; currency configuration remains a later Manage/settings concern owned by Story 2.2.
- Implement the Groups empty/active selection surface and the contextual five-link Group shell required to land a new Group in Manage. Selection is URL navigation, not persisted administrator state.
- Reuse existing `GroupService`, `GroupReader`, `GroupRepository`, `SqliteLedgerStore`, `Name::new`, strict form extractor, `build_group_template`/render projections, session/token stores, write gate, and runtime lifecycle owners. Do not create parallel Group APIs or route-local replay/mutation registries.
- Do not complete Group settings, Participant add/edit/archive/restore, Spending CRUD, monthly summaries, debts, or history-free deletion. Later stories own those capabilities, although the shell must render their correct navigation and disabled setup guidance.
- Do not add users, memberships as application identity, tenants, registration, participant authentication, persistent sessions, optimistic revisions, a global current-Group session field, floating point, SQL monetary logic, custom JavaScript, or compatibility shims.

## Tasks / Subtasks

- [x] Establish the application Group-create contract (AC: 2-4, 11)
  - [x] Change the transport-neutral create input so the application owns the `USD` default; keep `GroupInput`/equivalent update input from conflating name-only creation with currency-editing.
  - [x] Preserve `Name::new` trimming and 100-Unicode-character validation; map failures to the existing safe `Validation` taxonomy.
  - [x] Add fake-backed application tests for USD default, normalized name, invalid name, repository error propagation, and deterministic returned positive ID.
- [x] Update strict HTTP form and Group list rendering (AC: 1-3, 8, 12, 15-16)
  - [x] Remove `currency` from the Create Group field set and template; retain currency options only for the existing Group settings/edit flow.
  - [x] Preserve exact duplicate/unknown/missing-field rejection and the shared CSRF/submission-token extraction order.
  - [x] Retain raw name on `422`; ensure error IDs/labels/alert focus and the scoped request status remain valid when HTMX is absent.
  - [x] Render active Groups only in the default view, keep archived Groups in the contextual archived view, and preserve deterministic repository ordering.
- [x] Implement Group selection and contextual shell landing (AC: 13-16)
  - [x] Use the Group returned by creation for its ID; do not re-query the list or derive a destination from name/order.
  - [x] Add the canonical Manage destination needed by the UX contract while preserving the established Group Summary destination. Use native links for all five fixed destinations and `aria-current`; keep full-page responses authoritative.
  - [x] Project the new empty Group state with Manage as the current destination and Add Spending disabled plus distinct setup recovery guidance. Do not duplicate domain or persistence types in Askama contexts.
  - [x] Keep archived Group navigation readable and mutation-free; direct archived mutation/form behavior remains pre-dispatch `409`.
  - [x] Preserve stable server-owned heading IDs/autofocus and allow-listed native return destinations. Do not introduce custom script or HTMX history snapshots.
- [x] Integrate the first real mutation lifecycle path (AC: 5-10, 13)
  - [x] Reuse the single `DispatchedMutationRegistry` from `src/runtime.rs`; register a lease at the dispatch boundary and keep it alive through definitive persistence outcome publication.
  - [x] Ensure the root mutation executor, not a handler timeout or client connection, owns commit/rollback/unknown publication and is awaited by shutdown before checkpoint and pool close.
  - [x] Keep the five-second write-gate bound. A gate timeout must begin no transaction and produce only sanitized retryable feedback.
  - [x] Ensure successful Group creation advances the mutation epoch exactly once after commit; rejected, rolled-back, and unknown operations do not advance it.
  - [x] Exercise graceful shutdown while a real Group mutation is dispatched; shutdown must wait for its authoritative completion before WAL checkpoint/pool close.
- [x] Preserve and verify persistence (AC: 6, 10-12)
  - [x] Reuse the existing checked `INSERT INTO groups` and returned-row pattern unless a transaction boundary requires a narrow repository change.
  - [x] Keep SQLite structural checks for supported codes, archive flag, positive integer identity, bounded structural text, and relationships; Rust owns Unicode trimming and semantic validation.
  - [x] Do not change migrations unless necessary. If SQL/migrations change, update `specs/design.md` first, migrate a temporary SQLite database, run online `cargo sqlx prepare --workspace --check`, and commit refreshed `.sqlx` metadata.
- [x] Add invariant-owning web and composed tests (AC: all)
  - [x] Test empty active Groups, archived exclusion, name-only form, implicit USD, returned-ID Manage redirect, URL-based selection, native five-link shell, disabled Add Spending guidance, and 303 behavior.
  - [x] Test empty/overlong names with retained raw input and 422; focus/error/status/`aria-busy` markup; 320px/400% geometry through template assertions and the existing browser verification approach.
  - [x] Test missing/duplicate/unknown fields, invalid CSRF, unknown/reused token, unauthenticated access, oversized body, and malformed encoding with zero Group-use-case dispatch.
  - [x] Test concurrent valid creation with barriers/notifications, write-gate timeout without repository side effect, epoch advancement, authoritative result publication, and no automatic retry.
  - [x] Retain all existing auth/session/token/probe/readiness/shutdown/startup/restart coverage and add the real-socket Group mutation shutdown assertion required by `SPEC-FR103..SPEC-FR104`.

### Review Findings

- [x] [Review][Patch] Validate Group names before submission-token reservation and mutation dispatch [debtor-web/src/handlers/groups.rs:45-66] — application validation now runs before token reservation and mutation execution.
- [x] [Review][Patch] Keep dispatched mutation leases and publish authoritative outcomes under cancellation or task failure [src/composition.rs:65-90] — the mutation guard owns the lease in the spawned task, advances the epoch only on commit, and fails readiness on an unclassified task exit.
- [x] [Review][Patch] Give Transactions a distinct route context and current-navigation state [debtor-web/templates/group.html:15-19] — added `/groups/{id}/transactions` and an explicit Transactions section projection.
- [x] [Review][Patch] Restrict Manage to the Story 2.1 shell/setup projection [debtor-web/src/handlers/groups.rs:91-103] — Manage now renders setup guidance without exposing the legacy participant, spending, delete, or history workflows.
- [x] [Review][Patch] Render a real disabled Add Spending action for Groups without Participants [debtor-web/templates/group.html:38-42] — added a native disabled button alongside setup guidance and recovery navigation.
- [x] [Review][Patch] Associate and focus Group creation validation errors [debtor-web/templates/groups.html:26-36] — added focusable alert markup, conditional heading focus, `aria-invalid`, and stable input/error association without dangling success references.
- [x] [Review][Patch] Include active Participant count in Group rows [debtor-web/src/handlers/groups.rs:271-290] — Group rows now derive and render active non-archived Participant counts.
- [x] [Review][Patch] Implement pending `aria-busy` state for Group creation [debtor-web/templates/groups.html:33-36] — retained the HTMX-owned disabled initiator and scoped atomic status contract; no custom script was introduced because project policy forbids inline/custom application JavaScript.
- [x] [Review][Patch] Restore required contrast for no-Participant setup guidance [debtor-web/templates/group.html:38-42; static/css/app.css:38] — raised helper text to the established accessible muted token.
- [x] [Review][Defer] Add contextual Group navigation to the existing Debts page [debtor-web/templates/debts.html:12-20] — deferred, pre-existing; the current change adds the Group shell but does not modify the separate legacy Debts template, so contextual navigation across that page should be addressed with the broader debts/context-shell work.

## Dev Notes

### Developer Context

This is a brownfield vertical slice. The current repository already has working authentication, strict forms, CSRF, single-use submission tokens, an application Group port/service, checked SQLx Group persistence, an active/archived Group list, Group detail rendering, and a root-owned `DispatchedMutationRegistry`. The missing contract is not a new CRUD stack: creation currently requires a submitted currency, redirects to `/groups`, and existing ledger mutation handlers do not yet register a real dispatched mutation or publish authoritative terminal outcomes.

The current Group model is `Group { id: EntityId, name: Name, currency: Currency, is_archived: bool }`. `EntityId` is a positive persisted `i64`; `Name::new` trims and counts Unicode characters. The Group table already supports all twelve currencies and defaults `is_archived` to `0`; no schema default is needed for USD if the application supplies `Currency::Usd` on create.

The existing `/groups/{id}` handler builds the combined group page through `build_group_template`. The final UX contract requires a compact five-destination shell and distinguishes new Manage landing from established Summary landing. Keep the route inventory small and explicit: preserve `/groups/{id}` as the established Summary/canonical Group view, add the smallest native Manage route/projection needed for a newly created Group, and ensure each shell link has a valid full-page path. Do not infer or persist a new “configured” flag in this story. If implementation needs an established/new discriminator beyond the creation redirect, use explicit route context or existing persisted facts and document it; do not invent a database state that is absent from the contract.

### Current Files To Update And Preserve

| Path | Current state | Story-specific change/preservation |
| --- | --- | --- |
| `debtor-application/src/groups.rs` | `GroupInput` requires name and currency for both create/update; service parses currency before repository create. | Make create policy name-only with USD default while retaining currency validation for update. Preserve narrow ports, typed errors, injected fakes, and returned Group. |
| `debtor-domain/src/model.rs` | `Name::new` owns trim/non-empty/100-Unicode-character validation; `Group` carries currency/archive state. | Reuse unchanged unless a minimal API adjustment is proven necessary. Do not move lifecycle policy into web. |
| `debtor-infra/src/db/repos/groups.rs` | Checked list/create/update/archive/delete repository methods; create acquires write gate and returns inserted Group. | Reuse transaction/write-gate and checked query patterns. Add only the mutation-executor integration required to make this first real mutation authoritative. |
| `debtor-web/src/forms.rs` | `parse_group_form` requires `name`, `currency`, `csrf`, `submission_token`; shared extractor validates CSRF/token before handler. | Make create parsing name-only without weakening strict structure. Keep update parsing currency-bearing and preserve pre-dispatch validation/token semantics. |
| `debtor-web/src/handlers/groups.rs` | `POST /groups` parses currency, calls GroupInput with currency, and redirects to `/groups`; group detail is combined. | Remove create currency, pass raw name to application, use returned ID for Manage redirect, render safe 422 retention, and add/select contextual shell routes. Preserve archived guards and safe mapping. |
| `debtor-web/templates/groups.html` | Active list/create form includes a currency selector; shell is legacy two/three-link navigation. | Make create name-only, retain semantic status/error markup, active/archived separation, 48px targets, and native fallback. |
| `debtor-web/templates/group.html` | Combined members, spending, history, and archived read-only page; does not expose five fixed destinations. | Evolve into the required shell/projection without duplicating later lifecycle features. New no-Participant state must disable Add Spending and link to setup. |
| `debtor-web/src/templates.rs` | `GroupsTemplate` carries create currency/options; `GroupTemplate` is a combined rendering projection. | Remove create-only currency fields and add only explicit render-state fields needed for shell/Manage/Summary. Keep framework types out of application. |
| `debtor-web/src/handlers/spending_views.rs` | `build_group_template` loads group/member/spending/form projections and maps application errors. | Reuse or narrowly parameterize it for contextual Manage/Summary. Do not materialize unrelated history unnecessarily or create a parallel builder. |
| `debtor-web/src/router.rs` | Protected `/groups` create, `/groups/{id}` detail, and existing group mutation routes use shared middleware. | Add only the Manage/context routes required by the shell; preserve auth, body limits, preflight ordering, probe isolation, and safe read/mutation timeout classes. |
| `debtor-web/src/handlers/test_support.rs` | Fake GroupUseCases records `GroupInput` and returns test Groups. | Update fake and assertions to prove name-only USD create, returned ID, and zero dispatch on hostile input. |
| `debtor-web/src/router.rs` tests | Existing tests submit `currency=USD` to create and assert current combined markup. | Update for the normative name-only contract while retaining CSRF/token/replay/archived/no-dispatch regression coverage. |
| `src/composition.rs` | Composes one concrete Group service and one `DispatchedMutationRegistry` into `BuiltApp`. | Preserve singleton ownership and wire the first real mutation executor without leaking root/Tokio/SQLx types inward. |
| `src/runtime.rs` | Registry accepts leases and shutdown waits for empty, but no real Group dispatch uses it. | Integrate lease lifetime and definitive mutation outcome handling; shutdown must wait for real Group dispatch before checkpoint/pool close. |
| `src/main.rs` and `tests/restart.rs` | Root composition, socket, lifecycle, migration, and restart tests. | Add/adjust composed real Group mutation and shutdown ordering assertions; preserve restart/session invalidation and no-provider startup behavior. |
| `migrations/20260517000001_create_groups.up.sql` | Groups table has structural currency/archive/text checks and no currency default. | Prefer no migration. If changed, synchronize design first and refresh SQLx metadata. |
| `_bmad-output/implementation-artifacts/deferred-work.md` | Records that real mutation registration/outcome publication is deferred from Story 1.9. | Resolve this exact deferred item only when the implementation and tests truly prove it; do not create a second registry. |

### Architecture Compliance

- Preserve `debtor -> debtor-web/debtor-infra -> debtor-application -> debtor-domain`. Domain remains synchronous and framework-free; application owns Group input policy/use cases/ports; infra owns SQLx, SQLite gate, transactions, and safe adapter mapping; web owns Axum extraction, sessions/CSRF/tokens, Askama projections, navigation, and HTTP mapping; root owns composition, mutation execution, lifecycle, migrations, and shutdown.
- Follow AD-5: the web decodes structure and preserves raw text; application parses/normalizes name and applies the USD creation rule. No SQLx, Axum, Tokio, session, or persistence type crosses an application port.
- Follow AD-6/AD-13: exactly one `SqliteLedgerRuntime`/write gate, mutation epoch, root mutation registry/executor, session/token stores, and runtime owner. A lease covers the dispatched operation through authoritative result publication.
- Follow AD-10/AD-14: strict shared unsafe pipeline, 30-second pre-dispatch deadline, token reservation immediately before dispatch, bounded five-second gate/SQLite waits, and no generic timeout/cancellation after dispatch.
- Follow AD-11/AD-18: semantic server-rendered HTML and native links/forms are authoritative; pinned HTMX 2.0.10 and official response-targets 2.0.4 are optional only. Use stable IDs, focus matrix, scoped polite status, `aria-busy`, 48px targets, 320px/400% behavior, and Editorial Contrast. No custom JavaScript, inline scripts, or script attributes.
- Follow AD-15: map adapter failures to fixed safe categories. Never log names, IDs, tokens, cookies, client identities, SQL/database messages, request query strings, or raw adapter/task errors.
- Follow AD-17: Group is a private ledger context; do not model the administrator, Participants, membership, or tenant as new user/auth concepts.

### Mutation Outcome Guardrails

The Story 1.9 registry is a lifecycle seam, not yet proof of a real mutation. The developer must connect the Group mutation through one root-owned executor/lease boundary. The response may be rendered or redirected only after the executor has synchronously published the authoritative result. A successful SQLite commit is `Committed`; an established rollback is `RolledBack`; inability to establish either is `Unknown`, which is fatal and never retried automatically. Client disconnect, response rendering failure, or generic timeout must not cancel a dispatched operation or turn an unknown outcome into rollback. Shutdown closes admission, drains HTTP, waits for the registry to become empty, then performs WAL checkpoint and pool close.

Validation before dispatch is different: the token remains usable, no lease is registered, no epoch changes, and no transaction starts. A dispatched application validation failure consumes the token and has terminal mutation-attempt semantics even if no row is written, matching the established submission-token contract.

### UX and Navigation Guardrails

- Groups page: name-only creation above active rows; empty copy must not displace creation; Archived Groups is a contextual text link and archived rows are never mixed into active rows.
- New Group: `303` target is the stable Manage heading. Manage is the current item in five native destinations ordered Groups, Summary, Transactions, Debts, Manage. The active shell must remain usable at 320px and wide widths, with no overlaying bottom content or page-level horizontal scroll.
- Selection: clicking a Group row is selection by canonical URL. Do not create a session or cookie current-group state. Established Group navigation opens Summary; creation is the one explicit Manage landing.
- Add Spending: when no active Participant exists, render a disabled-looking but semantically disabled action with adjacent text guidance and a separate 48px recovery link. It must not pretend Spending is available or require a participant implementation in this story.
- Errors: retain safe submitted name exactly; do not claim retention for structurally undecodable input. Multiple errors use one linked alert summary; one error may target the invalid control. Status text is explicit, not color-only.
- Archived pages remain readable but no mutation/settings controls are exposed. Direct archived form/mutation requests are rejected before token reservation/use-case invocation according to the existing guard.

### Library / Framework Requirements

- Use the pinned project versions from `Cargo.lock` and `_bmad-output/project-context.md`: Rust 1.97.1/edition 2024, Axum 0.8.9, Askama 0.16.0, Tokio 1.53.1, Tower 0.5.3, tower-http 0.6.11, SQLx 0.9.0, and tower-sessions 0.15.0. Do not add or upgrade dependencies for this story.
- Axum supports the existing pattern of handlers returning a single `Response` after converting conditional `Redirect`/status responses with `IntoResponse`; preserve current middleware layering rather than moving authentication or preflight into handlers.
- Askama templates are compile-time checked from render structs. Keep template fields render-only and let conditional shell/error/status markup be represented by explicit fields; compile errors are expected if a field is removed without updating every template use.
- SQLx `query!`/`query_as!` macros are the required checked DML path. The sole unchecked exception remains the fixed WAL checkpoint PRAGMA. Do not use SQL monetary aggregation or unverified dynamic SQL.
- Current documentation was consulted through Context7 for Axum response/middleware patterns, Askama compile-time template contexts, and SQLx checked query macros. This story does not authorize framework API upgrades.

### Testing Requirements

- Domain/application: fake `GroupReader`/`GroupRepository` with injected state; assert name trimming, empty/Unicode-overlong rejection, USD default, returned ID, storage error mapping, and no framework/database dependency.
- Web: use the existing router `TestState` and real form/session pipeline. Assert exact allowed fields, name-only form, no currency/user/membership/participant inputs, 422 retention, 303 `Location`, Manage heading/autofocus, five link order/`aria-current`, disabled Add Spending guidance, and native full-page parity.
- Security/no-dispatch: assert missing, duplicate, unknown, malformed, oversized, unauthenticated, wrong-CSRF, unknown-token, and consumed-token requests do not invoke GroupUseCases. Validation before dispatch keeps the token; dispatched validation consumes it.
- Mutation/lifecycle: use barriers, `Notify`, held permits/connections, or controlled fakes, never sleeps as synchronization. Prove gate timeout starts no transaction, epoch advances only after commit, one valid concurrent commit path per admitted request, terminal outcome publication, and shutdown waits for a real Group mutation before checkpoint/pool close.
- Persistence: retain `#[sqlx::test]`/temporary SQLite coverage for Group constraints, active/archive filtering, checked create row retrieval, write contention, and migration. No monetary SQL is relevant to Group creation.
- Composition: retain a root real-socket smoke covering login/CSRF/token, authenticated Groups read, name-only Group creation, Manage redirect, persisted row, and coordinated shutdown. Preserve existing startup, readiness/liveness, provider-independent startup, restart, and safe-diagnostics tests.
- Required validation commands: `cargo fmt --all -- --check`; `cargo check --workspace --all-features --locked`; `SQLX_OFFLINE=true cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`; `cargo test --workspace --all-features --locked`; `cargo run --bin architecture-check --locked`. Run `cargo deny check` only if dependency policy/manifests change. Never use `cargo build --release`.
- If SQL/migrations change, also run `SQLX_OFFLINE=false DATABASE_URL=sqlite:///tmp/debtor-sqlx.db?mode=rwc cargo sqlx prepare --workspace --check` after migrating a temporary database, and commit refreshed `.sqlx` metadata.

### Previous Story Intelligence

There is no prior Story 2.x file; this is the first story in Epic 2. The immediately relevant completed work is Epic 1:

- Story 1.7 established one shared authenticated submission-token boundary. Reuse it; do not add route-local replay or idempotency systems.
- Story 1.8 established user/login/probe admission and readiness separation. Keep 64 user, 4 login, and 4 probe budgets, session-free probes/static assets, body limits, and pre-dispatch deadline behavior.
- Story 1.9 added the root-owned registry/empty barrier and safe shutdown ordering, but deliberately deferred real registration and authoritative `Committed`/`RolledBack`/`Unknown` integration to this story. Resolve only that deferred item.
- Story 1.10 fixed the private HTTP/1.1 backend and edge boundary. Do not add TLS/QUIC/HTTP3 or edge concerns to this Group route.

### Git Intelligence

- `HEAD` is `b71a15c` (`feat: impelement bmad 1-10`); the worktree was clean during analysis.
- Recent Story 1.9 work touched `src/runtime.rs`, `src/composition.rs`, `src/main.rs`, and web lifecycle state. Build on the current runtime and registry APIs; inspect actual code rather than copying older story prose.
- Recent Story 1.10 changed only edge/operations artifacts and did not change Rust, SQL, migrations, templates, or dependency versions.
- Existing Group/Participant/Spending code is a superseded brownfield scaffold in places. Remove conflicting paths where the normative contract requires it; do not preserve legacy multiple-payer/equal-share/global-participant models as parallel new behavior in this story.

### Project Structure Notes

- Feature modules use plural capability names (`groups`, `participants`, `spendings`, `debts`). Use `*Input`, `*Repository`, `*Reader`, `*UseCases`, `Db*`, and `*Template`/`*View` conventions.
- Group application changes belong in `debtor-application/src/groups.rs`; persistence in `debtor-infra/src/db/repos/groups.rs`; HTTP/forms/templates/tests in `debtor-web`; mutation composition/lifecycle in `src/composition.rs`, `src/runtime.rs`, and root tests.
- Keep outer types out of inner crates and keep handlers thin. Route handlers may choose redirects, parse safe query/route context, and render, but must not implement Group lifecycle policy, SQL, write-gate logic, or mutation outcome semantics.
- No migration is expected for name-only/USD creation. If a route split changes behavior beyond this story's acceptance criteria, update `specs/design.md` before code and synchronize planning/UX artifacts rather than silently diverging.

### References

- [Source: `_bmad-output/planning-artifacts/epics.md#Story 2.1: Create and Select a Group`]
- [Source: `_bmad-output/planning-artifacts/epics.md#Epic 2: Organize Groups and Participants`]
- [Source: `_bmad-output/planning-artifacts/epics.md#Final-Evidence Ledger`]
- [Source: `specs/design.md#Release Scope`]
- [Source: `specs/design.md#Architecture`]
- [Source: `specs/design.md#Accounting And History`]
- [Source: `specs/design.md#Security`]
- [Source: `specs/design.md#Operational Limits`]
- [Source: `specs/adr/0001-foundation-architecture.md#6. Application-owned policy`]
- [Source: `specs/adr/0001-foundation-architecture.md#11. Local readiness and shutdown`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-5 - Application policy with transactional enforcement`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-6 - Single ledger runtime and mutation epoch`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-10 - Shared unsafe-request admission boundary`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-11 - Native HTML authority`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-18 - Governed UX contracts and traceability`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md#Information Architecture`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md#Lifecycle and Access`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md#Interaction Focus Matrix`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/DESIGN.md#Layout & Spacing`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/DESIGN.md#Components`]
- [Source: `_bmad-output/project-context.md#Critical Implementation Rules`]
- [Source: `_bmad-output/implementation-artifacts/1-9-shut-down-the-authenticated-runtime-safely.md#Mutation Boundary and Story 2.1 Handoff`]
- [Source: `_bmad-output/implementation-artifacts/1-10-define-the-pre-production-https-edge-gate.md#Previous Story Intelligence`]
- [Source: `_bmad-output/implementation-artifacts/deferred-work.md`]
- [Source: `debtor-application/src/groups.rs`]
- [Source: `debtor-web/src/forms.rs`]
- [Source: `debtor-web/src/handlers/groups.rs`]
- [Source: `debtor-web/src/handlers/spending_views.rs`]
- [Source: `debtor-web/src/router.rs`]
- [Source: `debtor-web/templates/groups.html`]
- [Source: `debtor-web/templates/group.html`]
- [Source: `debtor-infra/src/db/repos/groups.rs`]
- [Source: `src/composition.rs`]
- [Source: `src/runtime.rs`]
- [Source: `migrations/20260517000001_create_groups.up.sql`]
- [Source: Context7 `/tokio-rs/axum` response and middleware documentation]
- [Source: Context7 `/askama-rs/askama` compile-time template documentation]
- [Source: Context7 `/websites/rs_sqlx` checked query macro documentation]

## Dev Agent Record

### Agent Model Used

openai/gpt-5.6-luna

### Debug Log References

- Ultimate context engine analysis completed - comprehensive developer guide created.
- Selected the first backlog story in complete sprint order: `2-1-create-and-select-a-group`.
- Loaded the complete sprint status, project context, normative design contract, Epic 2 and all-story context, PRD, architecture spine, UX Design/Experience contracts, ADR, prior Epic 1 implementation artifacts, deferred-work ledger, current Group code/templates/routes/tests, migrations, and recent Git history.
- No previous Story 2.x file exists. Story 1.9 is the direct lifecycle predecessor; Story 1.10 is the latest commit but is operations-only.
- Current brownfield conflicts recorded: creation currently accepts currency, success redirects to `/groups`, the Group page is a combined legacy projection, and the Story 1.9 mutation registry is not yet wired to a real Group dispatch.
- Consulted current Axum, Askama, and SQLx documentation through Context7. Pinned project versions remain authoritative; no dependency change is authorized.
- Open clarification retained for implementation: route inventory for the new five-destination shell is deferred by the architecture, so implementation must use the smallest explicit native Manage/Summary route projection and must not invent persistent setup state. This is a design guardrail, not permission to omit the Manage landing acceptance criterion.

### Implementation Plan

- Split Group creation from Group settings input so the application assigns `Currency::Usd` and the web form accepts only the name plus security fields.
- Add the native `/groups/{id}/manage` landing and five-link Group shell using URL context only; keep the existing Group route as Summary context and render setup guidance when no active Participant exists.
- Add an application-owned mutation executor port implemented by the root composition. The root executor registers the existing mutation lease, runs the Group use case in a supervised task, advances the epoch only after commit, reports safe failure, and lets shutdown await the lease.
- Extend application, web, and composed tests for name-only creation, USD default, Manage redirect, shell/setup state, epoch advancement, and real mutation shutdown.

### Completion Notes List

- Ultimate context engine analysis completed - comprehensive developer guide created.
- Group creation now accepts only a name and defaults to USD in application policy; Group settings retain their separate currency input.
- Added strict name-only form parsing, retained-value validation behavior, and regression coverage for malformed/security input.
- Added URL-based Manage landing, five native Group destinations, current-section state, and no-Participant Add Spending guidance.
- Wired the first real Group mutation through the root-owned mutation registry and executor, with commit epoch advancement and shutdown waiting.
- Added application, web, runtime, and real-socket coverage for the new vertical slice.
- Validation passed: `cargo fmt --all -- --check`, locked workspace check, strict offline Clippy, full locked workspace tests, and `cargo run --bin architecture-check --locked`.
- All nine actionable review findings were applied and covered by regression validation; the pre-existing Debts shell gap was recorded as deferred work.
- Story status set to `done`; sprint tracking is synchronized to the same state.

### File List

- `_bmad-output/implementation-artifacts/2-1-create-and-select-a-group.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `_bmad-output/implementation-artifacts/deferred-work.md`
- `debtor-application/src/groups.rs`
- `debtor-application/src/lib.rs`
- `debtor-web/src/forms.rs`
- `debtor-web/src/handlers.rs`
- `debtor-web/src/handlers/groups.rs`
- `debtor-web/src/handlers/spending_views.rs`
- `debtor-web/src/handlers/test_support.rs`
- `debtor-web/src/router.rs`
- `debtor-web/src/state.rs`
- `debtor-web/src/templates.rs`
- `debtor-web/templates/group.html`
- `debtor-web/templates/groups.html`
- `debtor-infra/src/db/repos/groups.rs`
- `static/css/app.css`
- `src/composition.rs`
- `src/main.rs`
- `src/runtime.rs`

### Change Log

- 2026-08-14: Created comprehensive Story 2.1 implementation context from the complete planning, UX, architecture, prior-story, Git, and codebase analysis.
- 2026-08-14: Implemented name-only USD Group creation, Manage landing/shell navigation, root mutation execution, epoch/shutdown integration, and invariant tests; moved story to review.
- 2026-08-14: Applied all code-review patches for validation ordering, mutation supervision, contextual navigation, Manage scope, setup affordances, accessibility associations, participant counts, and contrast.
