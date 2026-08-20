---
story_key: 5-5-browse-and-restore-archived-participants
story_id: 5.5
epic: 5
status: done
created: 2026-08-20
baseline_commit: 2763caa
completion_note: Ultimate context engine analysis completed - comprehensive developer guide created
---

# Story 5.5: Browse and Restore Archived Participants

Status: done

## Story

As the administrator,
I want archived Participants separated from active choices and restorable in context,
so that historical identities remain available without cluttering new allocations.

## Acceptance Criteria

1. Given a Participant is archived, when active Group Manage or a new Spending form renders, then the identity is absent from active Participant lists, Payer choices, and new Share choices, and is available only from a separate contextual archived-Participants view in its owning Group.
2. Given the archived-Participants view is opened, when identities render, then each shows its current name, supplementary outlined color marker, visible `Archived` text, and one protected Restore action; no independent delete action exists. The empty state has a safe 48-by-48 Manage return.
3. Given an archived Participant is referenced by history, when Transactions, Spending detail, monthly Summary, Historical/Current Balances, or Settlement Transfers render, then current name and associated visible `Archived` text remain and Payer/Share records are neither removed nor rewritten.
4. Given an existing Spending containing an archived identity is edited, when the Exact allocation form opens, then it retains that identity only in its stored Payer or Share role; it cannot be introduced or moved to another role.
5. Given a valid protected restore targets an archived Participant in an active owning Group, when it dispatches, then the shared write gate transaction atomically sets only the participant archive state active and redirects `303` to canonical Group Manage. It performs no Balance calculation, provider request, quote check, or ledger-generation eligibility check.
6. Given a restored Participant reloads in active Group and new Spending views, then it is an eligible active Payer/Share choice while all historical relationships remain unchanged.
7. Given the owning Group is archived or another Group's Participant ID is used, when the route processes, then archived-Group mutation returns pre-dispatch `409`; an ownership mismatch is rejected without exposing cross-Group details; no state changes.
8. Given restore is replayed, races another lifecycle mutation, or persistence fails, then at most one valid lifecycle change commits; errors remain sanitized; no identity is deleted or duplicated; deterministic tests prove list membership and preserved history.
9. Given all Participants are archived, when Manage and the Group shell render, then the active roster is empty and Add Spending remains disabled with distinct no-active-Participant guidance and a 48-by-48 Archived Participants recovery link. Archived rows are never mixed into active controls.
10. Given Restore is activated, when it is pending then commits, then its initiator is unavailable under one scoped pending status; the canonical Manage response focuses the restored Participant row/action and announces once; replay cannot dispatch twice. A failure focuses Restore or scoped status, leaves the identity archived/readable, and exposes no cross-Group detail.

Requirements: `SPEC-FR30`, `SPEC-FR35`, `SPEC-FR40..SPEC-FR42`; `SPEC-NFR3`, `SPEC-NFR10`, `SPEC-NFR15..SPEC-NFR16`, `SPEC-NFR22`, `SPEC-NFR25`, `SPEC-NFR28..SPEC-NFR34`; UX contracts `UX-SHELL-01`, `UX-TARGET-01`, `UX-FOCUS-01`, `UX-STATUS-01`, `UX-RESPONSIVE-01`, `UX-VISUAL-01`.

## Scope Boundary

- Implement only the Group-contextual archived Participant list and direct protected restore flow. Archive eligibility, confirmation, immutable Historical snapshots, rates, quotes, and generation revalidation are complete Story 5.4 behavior and must not be reused for restore.
- Do not add migrations, dependencies, global Participant routes/views, Participant deletion, membership reassignment/reactivation, optimistic revisions, stale-edit UX, rate evidence persistence, payments, manual retry, custom JavaScript, custom HTMX extensions, or post-dispatch generic timeouts.
- `specs/design.md` already defines this behavior. Do not change it unless a real contract divergence is discovered; if one is discovered, update it first and synchronize affected artifacts.

## Tasks / Subtasks

- [x] Add the narrow Group-scoped restore use case and root mutation execution path (AC: 5-8)
  - [x] Add `restore_group_participant(group_id, participant_id)` to `ParticipantRepository`, `ParticipantUseCases`, and `GroupMutationExecutor` in `debtor-application/src/participants.rs` and `debtor-application/src/groups.rs`.
  - [x] In `ParticipantService`, reject nonpositive IDs, archived Groups, missing/cross-Group targets, and already-active targets before repository invocation. The sole successful path calls the repository with `(group_id, participant_id)` and does not touch `ArchiveCalculationUseCases`.
  - [x] Extend `RootGroupMutationExecutor` in `src/composition.rs` using the same registration, spawned definitive-outcome, `GroupMutationGuard`, and no-post-dispatch-timeout pattern as `archive_group_participant`. Do not invoke `ParticipantUseCases` directly from a handler.

- [x] Add atomic write-gated persistence restore (AC: 5-8)
  - [x] Implement the repository operation in `debtor-infra/src/db/repos/participants.rs`: acquire `write_guard()` before opening a transaction; conditionally update only `participants.is_archived = 0` and `updated_at` for the supplied archived Participant owned by the supplied active Group.
  - [x] Require the Group active-state and participant ownership/archive-state predicates in the checked SQL statement. Inspect `rows_affected()` and roll back/map failure through existing safe Group/Participant failure helpers.
  - [x] Commit explicitly, then call `self.committed()` only after the authoritative commit. Do not alter `group_members.is_active`, names, colors, Spendings, allocations, rate evidence, or any historical record.
  - [x] Keep shared-gate and last-committed-write semantics. Restore does not require archive-admission generation/date/quote validation.

- [x] Add Group-contextual archived view and protected route (AC: 1-2, 7, 9-10)
  - [x] Add `GET /groups/{group_id}/participants/archived` and `POST /groups/{group_id}/participants/{participant_id}/restore` in `debtor-web/src/router.rs`; no global route.
  - [x] Add handlers beside archive/edit in `debtor-web/src/handlers/memberships.rs`. Reuse authentication, `require_writable_group`, `parse_lifecycle_form`, `CsrfValidatedForm::reserve_and_dispatch`, safe response mapping, and the root mutation executor. Do not call `state.debts` in any restore path.
  - [x] For restore POST, call `require_writable_group` before parsing or token reservation, validate only `csrf` and `submission_token`, reserve immediately before dispatch, redirect with `303` after success, and preserve terminal token behavior after dispatch.
  - [x] Add server-owned restored-participant focus state in `debtor-web/src/session.rs` and consume it from canonical Manage rendering. Never accept a client-controlled participant focus ID.
  - [x] Extend `ManageQuery`/Group Manage handler state in `debtor-web/src/handlers.rs` and `debtor-web/src/handlers/groups.rs` only as needed for one committed restore announcement and restored-row focus.

- [x] Build separate active and archived projections/templates (AC: 1-4, 6, 9-10)
  - [x] In `debtor-web/src/handlers/spending_views.rs`, retain `member.is_active && !participant.is_archived` for active Manage and new allocation choices. Add a distinct archived projection from `participant.is_archived`; do not conflate it with `inactive_members`.
  - [x] Add a typed Askama template/projection in `debtor-web/src/templates.rs` and a dedicated template under `debtor-web/templates/` for the archived Group-contextual list. Do not revive the commented legacy global `ParticipantsTemplate`.
  - [x] Update `debtor-web/templates/group.html` to link to Archived Participants from Manage, distinguish no-Participant setup guidance from the all-archived recovery path, and remove stale wording that lifecycle work is deferred. Preserve Group section order: settings, Participants, lifecycle.
  - [x] The archived list shows current name/color, visible `Archived` text, one direct protected Restore form, no delete, a scoped polite status, and an empty-state Manage return. Success redirects to Manage and autofocuses the restored row/action; failure returns focus to Restore/status.
  - [x] Preserve existing Transactions/detail/Summary/Debts/form projections that already show archived state and current names. Do not change stored allocation facts or edit-role allowances.

- [x] Apply minimal responsive and accessibility styling (AC: 2, 9-10)
  - [x] Extend `static/css/app.css` using existing participant-list/row/action/status rules and the dark Editorial Contrast tokens. Archived state must be textual and color marker remains supplementary with `var(--line)` outline.
  - [x] Ensure restored/archived rows, Manage recovery link, and Restore control are keyboard-operable, named, at least 48 by 48 CSS pixels, retain the existing two-pixel high-contrast focus outline, wrap long names at 320px/400% zoom, and never create page-level horizontal scrolling.
  - [x] Native links/forms remain authoritative. If using existing HTMX progressive enhancement attributes, keep equivalent native action/status/focus behavior and use only the pinned response-targets integration.

- [x] Test at the owning layer and retain regressions (AC: all)
  - [x] Application tests in `debtor-application/src/participants.rs`: valid archived owned target restores through the repository; active/missing/cross-Group/archived-Group requests reject before repository; prove restore does not invoke the debts/archive-calculation dependency.
  - [x] Infrastructure tests in `debtor-infra/tests/repos.rs`: conditional restore changes only archive flag/timestamp; ownership, active Group, already-active, and missing states reject atomically; restored identity returns to active new-allocation eligibility; historical aggregates/memberships/spending allocations remain intact; concurrent lifecycle outcomes and gate contention are deterministic.
  - [x] Web/router tests: authenticated contextual archived view, empty view, all-archived guidance, no delete/global route, direct protected restore `303`, server-owned focus and single announcement, strict unknown/duplicate/missing field and CSRF/token no-dispatch paths, archived-Group `409` before dispatch, cross-Group sanitization, terminal replay, no debt calculation, native/HTMX parity, and archived historical markers.
  - [x] Preserve Story 5.1-5.4 debt/rate/archive tests plus existing Spending archived-role and historical identity tests. Do not bundle the deferred `simplify.rs` saturating-arithmetic item.

- [x] Run validation (AC: all)
  - [x] `cargo fmt --all -- --check`
  - [x] `cargo check --workspace --all-features --locked`
  - [x] `SQLX_OFFLINE=true cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`
  - [x] `cargo test --workspace --all-features --locked`
  - [x] `cargo run --bin architecture-check --locked`
  - [x] If checked SQL changes, migrate a temporary SQLite database and run `SQLX_OFFLINE=false DATABASE_URL=sqlite:///tmp/debtor-sqlx.db?mode=rwc cargo sqlx prepare --workspace --check`; commit resulting `.sqlx` metadata.

### Review Findings

- [x] [Review][Patch] Restore pending and failure feedback is non-functional [debtor-web/templates/archived_participants.html:24]
- [x] [Review][Patch] Archived Groups render enabled Participant Restore controls [debtor-web/src/handlers/memberships.rs:199]
- [x] [Review][Patch] Restore success focus and announcement are not bound to the owning Group [debtor-web/src/handlers/groups.rs:153]
- [x] [Review][Patch] Archived Participants page omits the authenticated shell [debtor-web/templates/archived_participants.html:10]
- [x] [Review][Patch] Restore lacks application, infrastructure, and HTTP lifecycle regression coverage [debtor-application/src/participants.rs:281]
- [x] [Review][Patch] Enhanced restore swaps redirected full pages into its status target [debtor-web/src/handlers/memberships.rs:290]
- [x] [Review][Patch] Restore failure focus and success announcement are client-query controlled and repeatable [debtor-web/src/handlers/memberships.rs:192]
- [x] [Review][Patch] Restore feedback is stored in one session-wide slot, so concurrent restores can overwrite another initiator's focus and announcement [debtor-web/src/session.rs:405]
- [x] [Review][Patch] Restore safety paths lack required web and persistence regression coverage [debtor-web/src/router.rs:518]

## Dev Notes

### Developer Context

Story 5.4 completed safe archival: `participants.is_archived` is the current identity lifecycle bit; it is distinct from `group_members.is_active`. Restore reverses only the participant archive state. Do not call `set_member_active`, alter membership rows, or reconstruct any history.

`ParticipantReader::group_members` already returns all Group membership identities, ordered by name/ID, including archived records. The active Manage/spending projection deliberately filters `member.is_active && !participant.is_archived`; preserve that filter and derive the archived view separately from `participant.is_archived`. Existing direct Spending edits already add historical participants back only for stored roles and mark their allowed roles, which must remain untouched.

`RootGroupMutationExecutor` is the definitive mutation/shutdown boundary. Restore must reserve the submission token in web then enter this executor, where a registered lease publishes the definitive committed/rolled-back outcome. Do not add a direct handler-to-service mutation path or generic cancellation after dispatch.

### Architecture Compliance

- Preserve `root -> debtor-web/debtor-infra -> debtor-application -> debtor-domain`. Application defines policy and narrow ports; infra owns SQLx/gate/transaction; web owns auth, strict form extraction, sessions, templates, and HTTP mapping; root composes and supervises.
- Reuse the existing five-second `SqliteLedgerRuntime` gate. Final SQL must atomically constrain active Group, Group ownership, and archived target state. A stale/replayed concurrent request must result in one commit at most.
- Restore is unconditional only after lifecycle/ownership checks: no `DebtService`, `ArchiveCalculationUseCases`, rate provider/cache, quote bundle, UTC date, epoch, Balance, Settlement Transfer, or archive-confirmation state participates.
- Keep all safe error surfaces generic. Do not render or log IDs, raw SQLx/provider errors, URLs, tokens, sessions, CSRF values, monetary values, or cross-Group existence.
- Use existing typed errors, Askama escaping, checked SQLx macros, and existing strict security headers/CSP. No framework, SQLx, session, or HTTP types cross application ports.

### File Structure Requirements

| Path | Required work and preservation |
| --- | --- |
| `debtor-application/src/participants.rs` | Add scoped restore port/use case/service policy. Preserve create/edit/archive and archive-specific calculation behavior. |
| `debtor-application/src/groups.rs` | Extend only `GroupMutationExecutor` with participant restore. |
| `debtor-infra/src/db/repos/participants.rs` | Add conditional gated restore update; preserve canonical reads, archive admission, and active allocation eligibility. |
| `src/composition.rs` | Wire restore through `RootGroupMutationExecutor` using existing definitive lifecycle pattern. |
| `debtor-web/src/router.rs` | Add only Group-scoped archived-list and restore routes. |
| `debtor-web/src/handlers/memberships.rs` | Add restore/list handlers beside current lifecycle handlers; reuse strict form and token flow; no debt calls. |
| `debtor-web/src/handlers/spending_views.rs` | Add separate archived-member projection while preserving active allocation filter and edit-role retention. |
| `debtor-web/src/handlers.rs`, `debtor-web/src/handlers/groups.rs`, `debtor-web/src/session.rs` | Add tightly allow-listed query/session focus/notice state for post-restore Manage return. |
| `debtor-web/src/templates.rs`, `debtor-web/templates/group.html`, new archived Participant template | Add typed contextual archived view, Manage link/recovery, stable focus/status nodes; do not restore the legacy global template. |
| `static/css/app.css` | Minimal participant archived-view responsive styles using existing tokens and geometry. |
| `debtor-application/src/participants.rs`, `debtor-infra/tests/repos.rs`, `debtor-web/src/router.rs`, handler/template tests | Add owning-layer coverage and preserve relevant financial/history regression tests. |

### UX Requirements

- Retain the five-destination shell order: Groups, Summary, Transactions, Debts, Manage. Archived Participants is a contextual Manage destination, not a sixth shell destination or a global screen.
- Restore is direct but protected, not a confirmation page. Archive remains confirmation-based.
- Archived rows use explicit `Archived` text and readable current name, plus an outlined stored-color marker only as a supplemental cue. Never use color alone for identity or lifecycle state.
- For all-archived Groups, distinguish “no active Participant exists” from the first-Participant setup state; keep Add Spending disabled and provide a target-sized recovery link to archived Participants.
- Use stable server-owned focus IDs. Successful restore targets restored active row/action and announces once. Pending/failure retains or focuses Restore/status. Do not use client-provided focus IDs or custom client state.
- Preserve native full-page form/link behavior. Enhanced expected error handling may use only the existing pinned HTMX response-targets pattern, with no custom script/extension/inline script attributes.

### Library And Framework Requirements

- Preserve pinned Rust 1.97.1, Axum 0.8.9, Askama 0.16.0, Tokio 1.53.1, SQLx 0.9.0, and HTMX 2.0.10 plus response-targets 2.0.4. Add no dependency.
- SQLx conditional `UPDATE` uses checked query macros, executes via `&mut *tx`, checks `rows_affected()`, and explicitly commits/rolls back. [Source: Context7 `/websites/rs_sqlx`, consulted 2026-08-20]
- Axum routes remain explicit Group-scoped `get`/`post` handlers with `State` and tuple `Path` extraction. [Source: Context7 `/tokio-rs/axum`, consulted 2026-08-20]
- Askama template data remains typed and HTML-escaped. Templates render server-projected state only; no lifecycle, authorization, SQL, or financial policy in markup. [Source: Context7 `/askama-rs/askama`, consulted 2026-08-20]

### Previous Story Intelligence

- Story 5.4 made `SqliteLedgerRuntime` the shared owner of write gate and generation. Restore needs the gate and post-commit generation advancement, but not archive's snapshot/generation/date/quote admission.
- Story 5.4's `archive_group_participant` handler is the safe form/dispatch template only. Do not inherit its confirmation GET or Historical Balance precheck.
- Existing `group_members` contains archived identities; the missing behavior is a separate contextual projection and restore write, not a new read model or schema.
- The deferred `debtor-domain/src/debts/simplify.rs` saturating-arithmetic finding is unrelated and out of scope.

### Anti-Patterns To Avoid

- Do not restore `group_members.is_active` instead of `participants.is_archived`.
- Do not perform a balance/rate/quote/generation check, invoke DebtService, cache eligibility, or use archive-confirmation display state to authorize restore.
- Do not create a global Participant list/route, independent delete action, membership reassignment, or mixed active/archived roster.
- Do not bypass `GroupMutationExecutor`, the strict extractor, CSRF, session-backed submission token, or the shared SQLite gate.
- Do not expose cross-Group identity existence, add migrations unnecessarily, alter historical allocations, add optimistic stale-edit behavior, custom JavaScript, or manual retry UI.

### References

- [Source: `specs/design.md#Accounting And History`]
- [Source: `_bmad-output/planning-artifacts/epics.md#Story 5.5: Browse and Restore Archived Participants`]
- [Source: `_bmad-output/planning-artifacts/epics.md#Epic 5: Calculate Debts, Settle, and Safely Retire Identities`]
- [Source: `_bmad-output/planning-artifacts/prds/prd-debtor-2026-08-10/prd.md#Group-owned Participants`]
- [Source: `_bmad-output/planning-artifacts/prds/prd-debtor-2026-08-10/addendum.md#SQLite Integrity And Write Semantics`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-5 - Application policy with transactional enforcement`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-6 - Single ledger runtime and mutation epoch`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-10 - Shared unsafe-request admission boundary`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/DESIGN.md#Components`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md#Lifecycle and Access`]
- [Source: `debtor-application/src/participants.rs`]
- [Source: `debtor-application/src/groups.rs`]
- [Source: `debtor-infra/src/db/repos/participants.rs`]
- [Source: `src/composition.rs`]
- [Source: `debtor-web/src/handlers/memberships.rs`]
- [Source: `debtor-web/src/handlers/spending_views.rs`]
- [Source: `debtor-web/src/router.rs`]
- [Source: `debtor-web/src/templates.rs`]
- [Source: `debtor-web/templates/group.html`]
- [Source: `debtor-web/src/session.rs`]

## Dev Agent Record

### Agent Model Used

openai/gpt-5.6-terra

### Debug Log References

- Resolved `bmad-create-story` customization: no activation prepend/append or completion terminal steps configured.
- Loaded full sprint state, persistent project context, normative design contract, Epic 5 context, PRD/addendum, architecture spine, UX contracts, Story 5.4 intelligence, deferred-work ledger, affected source files, and recent commits.
- Used parallel artifact/codebase analysis and consulted current SQLx, Axum, and Askama documentation through Context7.
- Implemented Group-scoped direct Participant restore through the application port, root mutation executor, shared SQLite write gate, protected routes, session-owned focus, and contextual archived view.
- Refreshed SQLx metadata with a temporary migrated SQLite database and validated all targets offline.
- Added and ran focused application, SQLite, HTTP, and session restore lifecycle regressions; the initial server-owned failure-focus test correctly failed before its implementation.
- Re-ran the complete workspace validation after resolving both outstanding review findings.
- Resolved the final code-review findings with bounded nonce-bound restore notices and expanded web/persistence lifecycle regressions.

### Completion Notes List

- Ultimate context engine analysis completed - comprehensive developer guide created.
- Restored Participants now return to active allocation eligibility without modifying membership activity, historical allocations, or invoking debts/rates/quote eligibility.
- Validation passed: format, workspace check, strict offline Clippy, full workspace tests, architecture fitness, and SQLx metadata check.
- Added lifecycle regression coverage for valid restore, invalid ownership/lifecycle requests, preserved historical allocations, protected HTTP replay, and no archive-calculation dependency.
- Moved restore success and failure focus/announcements to scoped, server-owned, single-use session state; URL parameters can no longer repeat them.
- Final validation passed: `cargo fmt --all -- --check`, workspace check, strict offline Clippy, full workspace tests, and architecture fitness.
- Code review follow-up validation passed: formatting, workspace check, strict offline Clippy, full workspace tests, and architecture fitness.

### File List

- `_bmad-output/implementation-artifacts/5-5-browse-and-restore-archived-participants.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `_bmad-output/planning-artifacts/epics.md`
- `.sqlx/query-db08f969ff3c8349f492481053bd3bca0a66b5559427c30dc5824876f9c558f3.json`
- `debtor-application/src/groups.rs`
- `debtor-application/src/participants.rs`
- `debtor-infra/src/db/repos/participants.rs`
- `debtor-web/src/handlers.rs`
- `debtor-web/src/handlers/groups.rs`
- `debtor-web/src/handlers/memberships.rs`
- `debtor-web/src/handlers/spending_views.rs`
- `debtor-web/src/handlers/test_support.rs`
- `debtor-web/src/router.rs`
- `debtor-web/src/session.rs`
- `debtor-web/src/templates.rs`
- `debtor-web/templates/archived_participants.html`
- `debtor-web/templates/group.html`
- `src/composition.rs`

### Change Log

- 2026-08-20: Created comprehensive implementation context; status set to ready-for-dev.
- 2026-08-20: Implemented contextual archived Participant browse and direct protected restore; status set to review.
- 2026-08-20: Addressed four code-review findings; restore lifecycle coverage remains open and status returned to in-progress.
- 2026-08-20: Addressed the remaining lifecycle coverage and server-owned restore focus review findings; status set to review.
- 2026-08-20: Resolved all code-review findings; status set to done.
