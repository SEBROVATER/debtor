---
story_key: 2-5-archive-restore-or-delete-a-history-free-group
story_id: 2.5
epic: 2
status: done
created: 2026-08-17
baseline_commit: 987f420
completion_note: Ultimate context engine analysis completed - comprehensive developer guide created
---

# Story 2.5: Archive, Restore, or Delete a History-Free Group

Status: done

## Story

As the administrator,
I want to retire unused Groups without losing referenced history,
so that active navigation stays clean while destructive actions remain safe.

## Acceptance Criteria

1. Archiving an active Group starts from Manage and opens a full server-rendered confirmation page naming the Group, stating that archive is reversible and history remains readable, and carrying only an allow-listed return URL plus stable invoker focus ID. Cancel returns to the exact Manage archive control without mutation.
2. A protected archive confirmation is single-use. While dispatch is pending, the initiator becomes unavailable under one scoped polite status and `aria-busy`. The shared write gate atomically archives the Group; success redirects to `/groups`, focuses the active-list heading, and emits exactly one completion/count announcement. Replay returns `409 Conflict` without a second dispatch.
3. An archived Group is absent from active Group rows and present in a separate contextual Archived Groups view, visibly labelled `Archived`. Its Summary, Transactions, Debts, and read-only Manage views remain readable through the five-link shell. Archived pages expose no mutation/settings controls except protected Restore.
4. Every direct archived-Group mutation or mutation-form route other than Restore returns `409 Conflict` before submission-token reservation and application/use-case dispatch. Web tests prove zero dispatch. Missing Groups use the existing sanitized not-found mapping.
5. Restore is available from Archived Groups, requires CSRF and a submission token, has no confirmation page and no Balance/rate calculation, and only restores an actually archived Group. Success redirects to canonical active Groups, focuses the restored Group link, emits one announcement, and preserves Group ownership, Participants, and history unchanged. Restore must not use the active-only writable precheck that currently blocks it.
6. An active Group with no Spendings exposes irreversible Delete only through a full server-rendered confirmation page. The page names the Group, lists the exact unreferenced Group-owned Participants that will also be deleted, distinguishes Delete from reversible Archive in text and Editorial Contrast, and has an allow-listed Cancel target back to Manage. The disclosed Participant-ID set is bound to the server-owned confirmation state; arbitrary return URLs, focus IDs, or client-supplied ownership data are invalid.
7. Confirmed deletion atomically removes the history-free Group and its unreferenced owned Participants/membership rows, then redirects to `/groups`, focuses the active-list heading, and emits one completion announcement. The committing transaction rechecks that the Group is active, has no Spendings, and still has exactly the disclosed owned Participant-ID set. If Spendings exist, the Group is archived, the set changed, or a constraint/transaction fails, nothing is deleted and the response is a sanitized conflict/storage failure. Story 3.1 supplies the later structural proof that a Spending-backed Group cannot be deleted.
8. Archive, Restore, and Delete use the one existing root-owned Group mutation executor, one process-local mutation registry, one five-second SQLite write gate, one mutation epoch, and existing definitive outcome/shutdown handling. Every lifecycle write is serialized and transactional; successful commit advances the epoch, rollback publishes a definitive non-commit, and no generic timeout cancels dispatched work. Last committed valid write wins; no revision column, stale-edit conflict, retry loop, or second executor/gate is introduced.
9. Valid pre-dispatch validation and lifecycle rejection do not consume the submission token. Missing, malformed, duplicate, unknown, expired, reserved, or replayed security fields follow the shared strict boundary and invoke no lifecycle use case. Archive, Restore, and Delete retain native full-page behavior; optional pinned HTMX responses have equivalent status, focus, security, and fallback behavior without custom JavaScript or inline script attributes.
10. Lifecycle confirmations, active/archived Groups views, and canonical returns work at 320 CSS pixels and 400% zoom. Names, participant lists, scope/reversibility copy, actions, and status text wrap without clipping or page-level horizontal overflow. Every control/action remains at least 48 by 48 CSS pixels with a visible two-pixel focus indicator. Archive is reversible; Delete is irreversible; color never carries that meaning alone.
11. Existing authentication/session/CSRF/submission-token behavior, Group shell and five-destination reading order, active/archived filtering, Group settings, Group-owned Participant identity/history rules, Add Spending setup state, security headers, readiness, lifecycle, shutdown, and existing real-socket regressions continue to work end-to-end. Remove the stale `/participants` navigation link left by the old global Participant surface; do not restore that surface or add independent Participant deletion.

Requirements: `SPEC-FR24..SPEC-FR28`, `SPEC-FR36`, `SPEC-FR87..SPEC-FR90`; `SPEC-NFR3`, `SPEC-NFR15..SPEC-NFR16`, `SPEC-NFR22`, `SPEC-NFR25`, `SPEC-NFR31..SPEC-NFR34`; UX contracts `UX-SHELL-01`, `UX-TARGET-01`, `UX-FOCUS-01`, `UX-STATUS-01`, `UX-CONFIRM-01`, `UX-RESPONSIVE-01`, `UX-VISUAL-01`. This story introduces the history-aware deletion rule; Story 3.1 provides final evidence that a Group with persisted Spendings cannot be deleted.

## Scope Boundary

- Implement Group archive, Group restore, and history-free Group delete, including confirmation pages, archived Group navigation, active/archived list behavior, transaction boundaries, and regression evidence.
- Do not implement Participant archive/restore, Participant balance eligibility, Spending CRUD, summaries, debts, or rate-provider orchestration. Participant archive remains owned by Stories 5.4-5.5.
- Do not introduce users, memberships as identities, tenants, registration, participant authentication, global Participant routes, independent Participant deletion, optimistic revisions, or compatibility shims.
- Use the existing Group routes unless a route change is required by the acceptance criteria. Preserve the five-link shell and native fallback.
- Read and update `specs/design.md` first only if the implementation exposes a normative mismatch; synchronize migrations, tests, README/config examples, and `.sqlx` metadata whenever affected.

## Tasks / Subtasks

- [x] Establish lifecycle policy and mutation contracts (AC: 2-9)
  - [x] Replace ambiguous `set_archived(id, bool)` behavior with explicit archive/restore intent or an equally unambiguous application contract; archive requires active state, Restore requires archived state, and delete requires active history-free state.
  - [x] Keep checks side-effect-free before token reservation where the web boundary can establish them; repeat race-sensitive checks authoritatively in the committing repository transaction.
  - [x] Add archive, restore, and delete operations to `GroupMutationExecutor` while retaining current Group create/settings and Participant mutation APIs.
  - [x] Map missing, lifecycle conflict, contention, storage corruption, and unexpected failures through the existing safe `ApplicationError` taxonomy.

- [x] Reuse the root mutation lifecycle owner (AC: 2, 5, 7, 8)
  - [x] Extend `RootGroupMutationExecutor` with archive, restore, and delete methods using the existing lease, `GroupMutationGuard`, definitive outcome publication, epoch advancement-after-commit, readiness failure, and shutdown tracking.
  - [x] Ensure handlers never call `state.groups.set_archived` or `state.groups.delete_empty` directly.
  - [x] Preserve no generic post-dispatch cancellation and committed-result safety; do not turn a committed lifecycle write into a false retryable failure through an optional reload.

- [x] Make persistence transactional and race-safe (AC: 2, 5, 7, 8)
  - [x] Archive with a checked SQL transaction and active-state predicate; Restore with a checked SQL transaction and archived-state predicate.
  - [x] Delete under the write gate in one transaction, rechecking active Group, no Spendings, and the exact disclosed owned Participant-ID set before deleting the Group. Let the existing foreign keys enforce Spending restriction and Group-owned cleanup; do not independently delete Participants from an application path.
  - [x] Treat zero affected rows as a safe lifecycle/not-found/conflict result using existing transaction helpers. Do not expose IDs, names, SQL, or diagnostics.
  - [x] Preserve `groups.is_archived`, immutable Participant ownership, `spendings.group_id ON DELETE RESTRICT`, Group-owned Participant cascade behavior for an empty Group, and checked SQLx macros. Refresh `.sqlx` only if SQL changes.

- [x] Implement confirmation state and web lifecycle routes (AC: 1-7, 9-11)
  - [x] Keep the existing route inventory (`POST /groups/{id}/archive`, `POST /groups/{id}/restore`, `GET|POST /groups/{id}/delete`) unless a minimal canonical confirmation route adjustment is needed.
  - [x] Split active-only archive/delete prechecks from archived-only restore prechecks. Archived direct archive/delete/edit/settings/Participant/Spending mutations return `409` before token reservation and dispatch; Restore is the sole exception.
  - [x] Build Delete confirmation from a complete current Group/Participant snapshot and bind an allow-listed return destination plus exact Participant-ID disclosure to server-owned state. Do not trust a hidden Group or Participant list supplied by the browser.
  - [x] Preserve strict CSRF and submission-token extraction, token reservation immediately before supervised dispatch, 303 success redirects, native HTML authority, optional response-targets handling, and safe HTMX error fragments.
  - [x] Ensure archive/delete confirmation Cancel returns to `/groups/{id}/manage` with an allow-listed stable focus target; success returns to `/groups` with active heading focus; Restore success focuses the restored Group row/link.
  - [x] Remove the stale `/participants` link from `groups.html`; preserve contextual Archived Groups navigation and readable archived Group shell.

- [x] Align templates and UX states (AC: 1, 3, 6, 10, 11)
  - [x] Extend `ConfirmTemplate` or add a narrowly scoped render projection for explicit Group name, archive/delete effect, reversibility, disclosed Participants, return/focus state, CSRF, and submission token.
  - [x] Keep active Groups and Archived Groups separate. Render visible `Archived` text in Group identity/context. Do not mix archived rows into active lists.
  - [x] Hide all archived mutation controls except Restore. Keep active Manage lifecycle ordering and active-only Delete visibility. Do not expose Delete for a Group known to contain Spendings; if history is concurrently added, the transaction remains authoritative.
  - [x] Reuse Editorial Contrast rules, square geometry, existing status/focus classes, 48px targets, responsive layout, and no-card/no-gradient/no-animation rules. Status colors must always be paired with explanatory text.

- [x] Add invariant-owning, adapter, web, and composed regression coverage (AC: all)
  - [x] Application tests cover archive-active, restore-archived, delete-active-empty, archived/missing conflicts, explicit intent, safe errors, and no repository call for invalid lifecycle state where the application owns the invariant.
  - [x] Infrastructure tests with `#[sqlx::test]`/temporary SQLite cover atomic archive/restore/delete, exact Participant-set binding, active/archived predicates, Group-with-Spending restriction, cascade cleanup of history-free owned Participants/memberships, rollback, constraint failure, write-gate contention, and concurrent lifecycle races using barriers/notifications/held locks rather than sleeps.
  - [x] Web tests use the real router/session/CSRF/submission-token path to verify confirmation copy, allow-listed cancel/focus state, exact statuses and redirects, one-shot replay conflict, restore success, archived read-only rendering, zero dispatch on archived direct routes, missing Group mapping, strict malformed/duplicate/unknown fields, oversized body, and no `/participants` route/link.
  - [x] Verify native and HTMX parity, one scoped pending status/`aria-busy`, success focus and one announcement, 320px/400% geometry, 48px controls, no horizontal overflow, and explicit reversible versus irreversible wording.
  - [x] Retain Story 2.1-2.4 authentication, shell, settings, Participant ownership/editing, readiness, shutdown, SQLx, and root real-socket smoke regressions.

### Review Findings

- [x] [Review][Patch] [High] Delete confirmation is exposed for Groups with Spending history [debtor-web/src/handlers/groups.rs:339-376] — Added history verification before rendering confirmation and fail-closed Manage fallback behavior.
- [x] [Review][Patch] [High] Lifecycle success focus violates the single-target contract [debtor-web/templates/groups.html:29-38,53-54] — Removed competing notice autofocus and focus the restored Group link through a server-owned session target.
- [x] [Review][Patch] [Medium] Client-controlled focus query is not allow-listed [debtor-web/src/handlers/groups.rs:30-37,487] — Removed the client-controlled focus query and consumed the session-bound restore focus target.
- [x] [Review][Patch] [High] Delete confirmation snapshot is reusable with a later token [debtor-web/src/handlers/groups.rs:349-350,395-419] — Bound the session snapshot to the confirmation token and clear it before dispatch.
- [x] [Review][Patch] [Medium] Lifecycle forms bypass strict unknown-field rejection [debtor-web/src/forms.rs:127-175; debtor-web/src/handlers/groups.rs:383-424,495-570] — Added strict lifecycle form parsing before state-changing dispatch.
- [x] [Review][Patch] [Medium] HTMX confirmation errors are not scoped to the status region [debtor-web/templates/confirm.html:29-34] — Added response-targets mappings for 4xx and 5xx responses.
- [x] [Review][Patch] [Medium] Archived archive routes lack zero-dispatch regression coverage [debtor-web/src/router.rs:1561-1594] — Added archived archive route coverage and dispatch-counter assertions.
- [x] [Review][Patch] [Medium] Required repository race and rollback evidence is missing [debtor-infra/tests/repos.rs:55-104] — Added Spending restriction, mismatch rollback, and concurrent archive race coverage.
- [x] [Review][Patch] [Medium] Required lifecycle UX and native/HTMX coverage is missing [debtor-web/src/router.rs:1597-1740] — Added lifecycle pending/native/HTMX assertions and CSS contract checks for targets, focus, wrapping, and responsive rules.

## Dev Notes

### Developer Context

This is a brownfield vertical slice immediately after completed Story 2.4. Group creation/settings, Group-owned Participant persistence and editing, strict forms, CSRF and submission-token stores, the five-second SQLite write gate, mutation epoch, root mutation executor, native/HTMX boundary, archived read-only rendering, and Group shell already exist. Extend those paths; do not build a parallel lifecycle subsystem.

The current implementation has deliberate gaps: `GroupRepository::set_group_archived` performs an unscoped update; `delete_empty_group` is a single non-transactional DELETE; `GroupService::set_archived` has no state intent; `RootGroupMutationExecutor` supervises create/settings/Participant mutations only; handlers call archive/restore/delete directly. Most importantly, `restore_group` currently calls `require_writable_group`, so restoring an archived Group incorrectly returns `409`. Fix this rather than weakening the archived read-only rule.

The current delete confirmation is generic and does not list owned Participants or bind their IDs. The final delete transaction must re-read the authoritative state and compare the disclosed set before deleting. SQLite foreign keys already define the intended history boundary: `spendings.group_id` restricts deletion, while history-free Group-owned Participants and membership rows can be removed by the Group cascade. Story 3.1 later proves the Spending-backed restriction through actual Spending persistence.

### Current Files To Update And Preserve

| Path | Current state | Required change / preservation |
|---|---|---|
| `specs/design.md` | Normative source for lifecycle, history, security, and layer boundaries. | Read first; update only for a genuine contract clarification, then synchronize all affected artifacts. |
| `debtor-application/src/groups.rs` | Has `set_group_archived`, `delete_empty`, and repository ports; mutation executor lacks lifecycle operations. | Add explicit archive/restore/delete intent and supervised executor contracts; preserve Group create/settings, safe errors, and `*Reader`/`*Repository`/`*UseCases` naming. |
| `debtor-infra/src/db/repos/groups.rs` | Archive is unscoped/non-transactional; delete is one DELETE with active/no-Spending predicate. | Use checked transactional state predicates; atomically delete only the bound history-free Group/Participant set; preserve write gate and no post-commit false failure. |
| `src/composition.rs` | `RootGroupMutationExecutor` owns the single registry/guard/epoch path for existing mutations. | Add lifecycle methods through the same owner; never create another registry, gate, or timeout path. |
| `debtor-web/src/handlers/groups.rs` | Archive/restore share active-only precheck; delete directly calls use cases after token reservation; confirmation is generic. | Split archive/restore lifecycle checks, render bound confirmation, validate before reservation, and dispatch only through `state.group_mutations`. |
| `debtor-web/templates/groups.html` | Separate active/archived lists and forms exist, but stale `/participants` link remains. | Preserve separate lists and restore/archive actions; remove stale global Participant navigation and add required focus/status projections. |
| `debtor-web/templates/group.html` | Active pages expose “Delete empty group”; archived pages are read-only and readable. | Keep archived controls suppressed and active Manage lifecycle placement; ensure confirmation returns to Manage and no Spending-backed delete affordance is exposed. |
| `debtor-web/src/templates.rs` | Shared `ConfirmTemplate`, `GroupsTemplate`, and `GroupTemplate` projections. | Extend minimally for named Group, disclosed Participant rows, stable return/focus, and lifecycle announcements; keep projections render-only. |
| `debtor-web/src/router.rs` | Archive, restore, and delete routes are protected and already registered. | Preserve canonical paths/middleware; add route tests and no-dispatch coverage rather than aliases. |
| `debtor-web/src/forms.rs` | Shared strict CSRF form boundary and exact field parsers. | Reuse it; lifecycle confirmation fields must remain only the approved security/confirmation fields, with no trusted client ownership list. |
| `debtor-web/src/handlers/test_support.rs` | Fakes record existing Group/Participant mutation dispatch. | Record archive/restore/delete calls and exact no-dispatch behavior. |
| `debtor-infra/tests/repos.rs` and `debtor-infra/tests/migrations.rs` | Existing schema and repository tests cover Group/Participant ownership and some cascade/restrict behavior. | Add lifecycle transaction, exact-set, race, rollback, and referenced-history assertions. |
| `static/css/app.css` | Existing Editorial Contrast, status, confirmation, focus, and responsive rules. | Reuse and extend minimally only for disclosed Participant confirmation rows or lifecycle status. |

### Architecture Compliance

- Preserve `root -> debtor-web/debtor-infra -> debtor-application -> debtor-domain`; domain/application ports never depend on Axum, Askama, SQLx, sessions, or concrete adapters.
- Follow AD-4/AD-5: referenced Groups are archived rather than deleted; only an active history-free Group may be deleted; Group-owned Participants are never independently deleted or reassigned; race-sensitive lifecycle facts are enforced in the committing transaction.
- Follow AD-6/AD-13: one `SqliteLedgerRuntime`, one write gate, one mutation epoch, and one root mutation registry. Gate timeout starts no transaction or guarded side effect; epoch advances only after commit.
- Follow AD-10/AD-11: authentication, strict structure, CSRF, lifecycle precheck, token reservation immediately before dispatch, one supervised mutation, and definitive outcome. Native HTML is authoritative; pinned HTMX 2.0.10 and response-targets 2.0.4 remain optional.
- Follow AD-15: `Validation` 422, `NotFound` 404, lifecycle/token conflict 409, contention/unavailability 503, and sanitized storage failures. Never expose/log SQL, identifiers, names, Participant sets, tokens, cookies, URLs, or request-derived diagnostics.
- Follow AD-18 and UX contracts: `UX-CONFIRM-01` governs full-page confirmations and allow-listed return focus; `UX-FOCUS-01` governs exactly one stable focus target; `UX-STATUS-01` governs scoped polite atomic status and `aria-busy`; `UX-SHELL-01`, `UX-TARGET-01`, `UX-RESPONSIVE-01`, and `UX-VISUAL-01` govern shell, geometry, and Editorial Contrast.

### Library / Framework Requirements

- Keep the pinned stack: Rust 1.97.1/edition 2024, Axum 0.8.9, Askama 0.16.0, Tokio 1.53.1, SQLx/sqlx-cli 0.9.0, Tower 0.5.3, tower-http 0.6.11, tower-sessions 0.15.0, and existing HTMX assets. Do not add or upgrade dependencies.
- Use compile-time checked `sqlx::query!`/`query_as!` statements. SQLx transaction usage must execute all lifecycle statements through the transaction and commit only after all checks succeed; use affected-row results for safe conflict mapping. The fixed WAL-checkpoint PRAGMA remains the only unchecked SQL exception.
- Preserve existing Axum routing, middleware, `Response`/redirect, Askama context, strict form, and HTMX response patterns. Do not migrate framework APIs or add custom JavaScript.
- Context7 review on 2026-08-17 confirms current SQLx transaction semantics: execute checked queries against `&mut *transaction`, inspect `rows_affected()`, and call `commit()` only after the complete operation succeeds. The repository's pinned version and lockfile remain authoritative.

### Testing Requirements

- Application: fake-backed lifecycle policy tests without Axum, SQLite, network, or wall clock; assert explicit active/archived intent, no invalid-state repository call, safe conflict/not-found mapping, and no duplicate lifecycle abstraction.
- Infrastructure: temporary SQLite/`#[sqlx::test]` tests for active archive, archived restore, history-free cascade, Spending restriction, exact disclosed Participant set, Group/Participant atomicity, concurrent archive/restore/delete, write-gate contention, and no post-commit reload dependency. Verify foreign keys are enabled.
- Web: real router/session/CSRF/submission-token pipeline; assert confirmation copy and field/security shape, allow-listed Cancel URL/focus, 303 destinations, active/archived list separation, Restore success, archived direct-route 409 with zero dispatch, replay 409, no token reservation before lifecycle rejection, and no global Participant route/link.
- Hostile inputs: malformed encoding, missing/duplicate/unknown fields, invalid CSRF, unknown/reused token, unauthenticated request, oversized body, negative/invalid IDs, archived direct mutations, missing Group, stale confirmation Participant set, and crafted arbitrary return/focus values. Prove no state-changing side effect on pre-dispatch rejection.
- UX: native and HTMX parity, stable heading/row focus, one announcement, scoped `aria-busy`, one-shot initiator disablement, explicit “reversible”/“irreversible” copy, 48px targets, visible focus, long-name/participant-list wrapping, and no page overflow at 320px/400% zoom.
- Required validation: `cargo fmt --all -- --check`; `cargo check --workspace --all-features --locked`; `SQLX_OFFLINE=true cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`; `cargo test --workspace --all-features --locked`; `cargo run --bin architecture-check --locked`.
- If checked SQL changes: migrate a temporary database and run `SQLX_OFFLINE=false DATABASE_URL=sqlite:///tmp/debtor-sqlx.db?mode=rwc cargo sqlx prepare --workspace --check`; refresh committed `.sqlx`. Never use `cargo build --release`.

### Previous Story Intelligence

- Story 2.4 is the direct predecessor. Reuse its Group-scoped ownership enforcement, active/archived pre-dispatch boundary, validation-before-token-reservation order, root mutation owner, committed-result safety, safe mapping, stable focus/status IDs, and native/HTMX parity. Its review specifically fixed transactional lifecycle predicates, false read failures, focus duplication, guidance associations, and missing hostile-input coverage.
- Story 2.3 established the Group-owned schema, immutable Participant ownership, restrictive referenced-history rules, Manage shell, active filtering, and the rule that global Participant APIs/routes must stay removed. Do not add independent Participant deletion as part of Group deletion; deletion is an owning-Group operation constrained by history.
- Story 2.2 established canonical Manage settings, archived read-only behavior, status/focus conventions, write-gate/epoch behavior, last-commit semantics, and confirmation-safe patterns. Fit lifecycle sections into that existing Manage structure.
- Story 2.1 established the five-destination shell, URL-selected Group, real mutation lifecycle/shutdown integration, and authenticated native/HTMX boundary. Preserve all of it.

### Git Intelligence

- Baseline is commit `987f420 feat: implement 2-4 bmad`; preceding commits implement Stories 2.3, 2.2, and 2.1. Recent work consistently spans application ports, infra repositories/tests, web handlers/templates/router/test support, root composition, CSS, and `.sqlx` when SQL changes.
- The worktree was clean during analysis. Do not revert unrelated concurrent changes. Inspect current code rather than relying on older story prose; older global Participant APIs are superseded.

### Project Structure Notes

- Keep plural capability modules (`groups`, `participants`, `spendings`, `debts`) and `*Input`, `*Reader`, `*Repository`, `*UseCases`, `*Service`, `*Store`, `*Template`, `*Row`, and `*View` naming.
- Preserve positive `i64` ledger IDs, canonical Group/Participant ownership, restrictive `spendings.group_id` deletion, and Rust-owned policy. Do not put SQLx or HTTP types into application ports.
- The current `ConfirmTemplate` is too small for the required named Participant deletion disclosure; extend it narrowly rather than creating a generic confirmation framework or arbitrary return URL mechanism.
- Group archive is reversible and may apply to Groups with history. Group delete is irreversible and only applies to an active, history-free Group. Active/archived lists must remain separate and contextual.

### References

- [Source: `_bmad-output/planning-artifacts/epics.md#Story 2.5: Archive, Restore, or Delete a History-Free Group`]
- [Source: `_bmad-output/planning-artifacts/epics.md#Final-Evidence Ledger`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-4 - Group-owned identity and history`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-5 - Application policy with transactional enforcement`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-6 - Single ledger runtime and mutation epoch`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-10 - Shared unsafe-request admission boundary`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-11 - Native HTML authority`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-15 - Safe failures, logging, and readiness`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-18 - Governed UX contracts and traceability`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md#Lifecycle and Access`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md#Interaction Focus Matrix`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md#HTTP and Session Outcomes`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/DESIGN.md#Layout & Spacing`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/DESIGN.md#Management form`]
- [Source: `_bmad-output/implementation-artifacts/2-4-edit-active-participants-without-deleting-identity.md#Previous Story Intelligence`]
- [Source: `_bmad-output/implementation-artifacts/2-3-add-group-owned-participants.md#Previous Story Intelligence`]
- [Source: `_bmad-output/project-context.md#Critical Implementation Rules`]
- [Source: `specs/design.md#Accounting And History`]
- [Source: `debtor-application/src/groups.rs`]
- [Source: `debtor-infra/src/db/repos/groups.rs`]
- [Source: `debtor-web/src/handlers/groups.rs`]
- [Source: `debtor-web/src/templates.rs`]
- [Source: `debtor-web/templates/groups.html`]
- [Source: `debtor-web/templates/group.html`]
- [Source: `debtor-web/src/router.rs`]
- [Source: `src/composition.rs`]
- [Source: `migrations/20260517000001_create_groups.up.sql`]
- [Source: `migrations/20260517000002_create_participants.up.sql`]
- [Source: `migrations/20260517000004_create_spendings.up.sql`]
- [Source: `static/css/app.css`]
- [Source: SQLx current transaction/query macro documentation via Context7, `/websites/rs_sqlx`]

## Dev Agent Record

### Agent Model Used

openai/gpt-5.6-luna

### Debug Log References

- Resolved create-story customization: no activation prepend/append steps; loaded persistent fact `_bmad-output/project-context.md`.
- Read the complete `sprint-status.yaml`; selected the first backlog story in order: `2-5-archive-restore-or-delete-a-history-free-group`.
- Loaded the complete Epic 2/Story 2.5 context, architecture spine, UX contracts, project context, completed Stories 2.2-2.4, current Group lifecycle code, templates, router, schema, and recent Git history.
- Current implementation audit found restore incorrectly uses the active-only writable check, archive/restore/delete bypass the root mutation supervisor, delete lacks explicit transaction/set binding, and Groups retains a stale global Participant link.
- Reviewed current SQLx transaction guidance; no dependency or framework API change is authorized.
- Implemented explicit archive/restore/delete application ports and reused the existing root mutation registry, lease, epoch, readiness, and shutdown guard.
- Added session-bound deletion confirmation snapshots and transactional repository checks for active state, empty history, and exact owned Participant IDs.
- Added full-page archive/delete confirmations, archived-only restore, active/archived Group focus and announcements, Manage lifecycle controls, and removed the stale global Participant link.

### Implementation Plan

- Keep lifecycle policy in `debtor-application`, race-sensitive checks in the SQLite transaction, HTTP/security/rendering in `debtor-web`, and composition/supervision in root.
- Use server-owned session state for the delete confirmation snapshot; never trust hidden participant IDs or arbitrary return targets.
- Validate the vertical slice with application, infrastructure, web, full workspace, Clippy, architecture fitness, formatting, and SQLx metadata checks.

### Completion Notes List

- Implemented Group archive confirmation and supervised archive dispatch with active-state enforcement.
- Implemented archived Group restore with archived-state enforcement, canonical active-list focus, and completion announcements.
- Implemented history-free Group delete confirmation with session-bound Participant disclosure and atomic exact-set deletion checks.
- Preserved restrictive Spending-backed deletion, Group-owned Participant cascade cleanup, native/HTMX fallback, CSRF, submission tokens, and read-only archived views.
- Added application, SQLite repository, and router regression coverage for lifecycle intent, confirmations, replay, restore, deletion, and exact Participant-set binding.
- Validation passed: `cargo fmt --all -- --check`; `cargo check --workspace --all-features --locked`; strict offline Clippy; `cargo test --workspace --all-features --locked`; `cargo run --bin architecture-check --locked`; and online `cargo sqlx prepare --workspace --check`.
- Resolved all 9 code-review findings: history-aware delete confirmation, single-target/server-owned focus, one-shot delete confirmation binding, strict lifecycle parsing, scoped HTMX errors, archived-route dispatch tests, repository race/rollback coverage, and lifecycle UX/CSS contract coverage.
- Story status set to `done`; sprint tracking synchronized to `done`.

### File List

- `_bmad-output/implementation-artifacts/2-5-archive-restore-or-delete-a-history-free-group.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `.sqlx/query-a22ff0c96d27dc4202c6548e14d4eab31f1eab72b4af764e52deca26944acf0d.json` (deleted obsolete archive query metadata)
- `.sqlx/query-06d935ee8aa7324b126e3a0a508411e37b402630cff7fd9494e0f6b4230b7167.json`
- `.sqlx/query-2785db8f647bfe1b3e98d155f72964dfa9bd2186f9c8c21b2fe57d6edb2ffed8.json`
- `.sqlx/query-62ce033d2dafd7e98e242ced1cafdb538b1c05c6784b6df98b5c8a5937d6c0dc.json`
- `.sqlx/query-be2dbda04bb3674d5c9d538acdf2d0ecbed5c67fac56105bcb10cf72d2209b9a.json`
- `debtor-application/src/groups.rs`
- `debtor-infra/src/db/repos/groups.rs`
- `debtor-infra/tests/repos.rs`
- `debtor-web/src/handlers.rs`
- `debtor-web/src/handlers/groups.rs`
- `debtor-web/src/forms.rs`
- `debtor-web/src/handlers/spending_views.rs`
- `debtor-web/src/handlers/spendings.rs`
- `debtor-web/src/handlers/test_support.rs`
- `debtor-web/src/router.rs`
- `debtor-web/src/session.rs`
- `debtor-web/src/templates.rs`
- `debtor-web/templates/confirm.html`
- `debtor-web/templates/group.html`
- `debtor-web/templates/groups.html`
- `src/composition.rs`

### Change Log

- 2026-08-17: Implemented Group archive, restore, and history-free delete lifecycle with transactional enforcement, confirmations, focus/status UX, regression tests, and refreshed SQLx metadata; moved story to `review`.
- 2026-08-17: Addressed all 9 code-review findings and revalidated the full workspace; moved story to `done`.
