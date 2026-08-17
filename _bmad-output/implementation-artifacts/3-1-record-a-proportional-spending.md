---
story_key: 3-1-record-a-proportional-spending
story_id: 3.1
epic: 3
status: done
baseline_commit: 6aa5e636cecb50f065bbc5a3e8cb0b3c87604d83
created: 2026-08-17
completion_note: Ultimate context engine analysis completed - comprehensive developer guide created
---

# Story 3.1: Record a Proportional Spending

Status: done

## Story

As the administrator,
I want to preview and record a Spending divided by proportional weights,
so that I can complete an exact shared transaction in one usable flow.

## Acceptance Criteria

1. An active Group with active Participants exposes a native Add Spending link from every active Group section. It opens a focused full-page form. The new form starts with empty Description and Total, current UTC date, Source Currency equal to Group Currency, no Category, no Payer, and every active Participant selected with Proportional weight `1`. Archived Participants are unavailable for new allocations.
2. The form offers exactly the twelve supported currencies, eight supported categories, Proportional and Exact share modes, and exactly one Payer. Description is trimmed, non-empty, and at most 200 Unicode characters. Date is strict `YYYY-MM-DD` and on or after `2025-01-01`.
3. Selecting one Payer assigns that active Group-owned Participant the full Total while leaving Payer selection independent from Share responsibility.
4. Application-owned input parsing uses exact `Decimal`. Total is positive, precision-valid for the selected currency, and at most `999_999_999_999`. Selected Proportional weights are positive, at most `1_000_000`, and at most six fractional digits. Web parsing preserves raw fields and never constructs financial allocations.
5. A valid Proportional Preview normalizes weights at the maximum submitted scale and performs one checked `i128` minor-unit integer-ratio allocation. Residual units are assigned by descending remainder, then ascending Participant ID. The result is deterministic, ordered, exactly conserves Total, and rejects any zero resulting Share.
6. Empty selection, duplicate/unknown/cross-Group/inactive/archived Participant, invalid weight/amount/date/category/currency, normalization failure, checked arithmetic failure, or zero-result Share returns `422 Unprocessable Entity` with safe retained raw values and programmatically associated errors. Preview produces no aggregate and consumes no submission token. Invalid values are never rounded, substituted, or logged.
7. Native Preview rerenders a reviewed, non-editable full page. Approve is available only for that reviewed input; Edit allocation returns to the editable state. The reviewed input is server-bound, and Approve reparses/revalidates the same raw input, so stale, changed, mismatched, invalid, or replayed review state cannot create a Spending.
8. Optional HTMX Preview uses the same application/domain operation as native Preview. It is latest-input-wins: superseded responses never swap; only derived allocation cells, status, and approval state change; focus, caret, selection, keyboard, active row, and table/page scroll remain unchanged. One polite atomic status owns pending/ready/error and the owning region exposes `aria-busy`. HTMX failure leaves the native path usable.
9. Approve validates the reviewed binding and atomically persists one complete Spending, its single Payer, and Shares through the sole shared create aggregate path. Persistence rechecks active Group state and active owned Participants inside the write-gated transaction, stores canonical decimal `TEXT`, and rolls back all rows on any race, constraint, or checked failure.
10. Successful creation returns `303 See Other` to Transactions with the committed row visible and its summary as the single forward focus target. No completion badge or optimistic success is shown.
11. Archived Group form/Preview/mutation requests return pre-use-case `409 Conflict`; invalid ownership returns sanitized `422`; neither path performs state-changing dispatch or provider work.
12. The first persisted Spending proves the Group deletion boundary: application deletion is refused in favor of archive, direct SQLite Group deletion is restricted, and the Group, owned Participants, Spending, Payer, and Shares remain intact after the rejected delete.
13. The focused form is one document scroll owner with `min-height: 100dvh`; fields use two columns and stack at 350px or narrower; the in-flow sticky action bar clears keyboard and safe-area growth. At 320 CSS pixels and 400% zoom all controls remain at least 48 by 48 CSS pixels with no page-level horizontal scroll.
14. The allocation table is semantic and inside a labelled, keyboard-focusable internal horizontal scroll region. It is 520px wide with columns `116/76/76/92/160`, keeps Participant identity sticky, associates headers and controls explicitly, wraps 100-character names, and keeps Payer, Included, Weight, and derived Share controls at least 48 by 48 CSS pixels.

Requirements: `SPEC-FR29`, `SPEC-FR43..SPEC-FR56`, `SPEC-FR59`, `SPEC-FR62..SPEC-FR64`, `SPEC-FR87..SPEC-FR94`; `SPEC-NFR5..SPEC-NFR10`, `SPEC-NFR14..SPEC-NFR16`, `SPEC-NFR22`, `SPEC-NFR25`, `SPEC-NFR28..SPEC-NFR34`; UX contracts `UX-SHELL-01`, `UX-TARGET-01`, `UX-ALLOC-01`, `UX-FOCUS-01`, `UX-STATUS-01`, `UX-PREVIEW-NATIVE-01`, `UX-PREVIEW-LATEST-01`, `UX-RESPONSIVE-01`, `UX-VISUAL-01`.

## Scope Boundary

- Own the first canonical Proportional Preview, reviewed approval, and complete Spending create aggregate path.
- Build shared create-input, allocation, persistence, admission, and success-routing seams so Story 3.2 can reuse them for Exact mode. Do not build a second persistence or mutation executor for Exact.
- Replace superseded legacy create behavior rather than retaining parallel paths: multiple Payers, Equal mode, inline/modal entry, client-constructed allocations, and full-form HTMX swaps.
- Do not implement Exact creation beyond the shared mode contract/projection seam required to keep the form closed to the two supported modes; Exact behavior belongs to Story 3.2.
- Do not implement Spending correction, deletion redesign, history redesign, current-month summaries, exchange rates, debts, settlements, Participant archive/restore, or global Participant management.
- Do not add users, memberships as identities, tenants, registration, participant authentication, optimistic revisions, stale-edit conflicts, compatibility shims, custom JavaScript, custom HTMX extensions, inline scripts, modals, or offline queues.
- Update `specs/design.md` first only if implementation reveals a genuine normative mismatch. If changed, synchronize all affected artifacts in the same change.

## Tasks / Subtasks

- [x] Replace legacy Spending domain/application input modes with the canonical single-Payer and Proportional/Exact boundary (AC: 2-6, 9)
  - [x] Add or adapt a synchronous deterministic proportional allocator in `debtor-domain`, using checked `i128`, currency minor units, maximum weight scale, descending remainder, and ascending Participant-ID ties.
  - [x] Make the aggregate enforce exactly one payer paying Total; retain positive unique exact-conserving Shares and canonical amount validation.
  - [x] Parse raw unsigned plain-decimal amounts/weights in `debtor-application`; reject whitespace, signs, exponent notation, excess precision, overflow, nonpositive values, and out-of-range values without rounding.
  - [x] Keep Preview and commit on one shared operation and add fake-backed application tests for policy, eligibility, and no-dispatch/no-repository behavior on invalid input.
- [x] Establish the sole supervised Spending create mutation path (AC: 9-12)
  - [x] Route dispatched create through the existing root-owned mutation registry/runtime lifecycle and the existing five-second SQLite write gate; do not create another registry, gate, epoch, timeout, or retry path.
  - [x] Preserve definitive commit/rollback semantics and shutdown waiting after dispatch. Never let a generic timeout cancel a dispatched mutation or report a committed write as retryable failure.
  - [x] Recheck Group active state and every Payer/Share Participant's ownership and active/non-archived eligibility in the committing transaction.
  - [x] Persist Spending, payer, and shares atomically with checked SQLx macros and `format_decimal`; hydrate/revalidate canonical values before authoritative success is published, avoiding a post-commit false failure.
  - [x] Prove Group `ON DELETE RESTRICT` and Participant `ON DELETE RESTRICT` with persisted Spending data; preserve Group and all history on rejected deletion.
- [x] Replace the legacy inline form with the focused native Spending flow (AC: 1-4, 7, 10, 13-14)
  - [x] Add the native full-page route and explicit render projection/template if needed; keep route names plural and use `*Input`, `*Template`, `*Row`, and `*View` conventions.
  - [x] Render the required defaults exactly: empty Description/Total, UTC date, Group Currency, no Category, no Payer, all active Participants included with weight `1`.
  - [x] Render only one Payer selector and Proportional/Exact mode controls. Do not expose Multiple Payers or Equal mode.
  - [x] Implement server-owned reviewed-input binding, native Preview, Edit allocation, Approve, allow-listed Cancel, stable heading/status IDs, and committed Transaction-row focus.
  - [x] Keep unsafe form processing in the shared strict extractor order: bounded body, authentication, exactly one CSRF, strict fields, Group/form prechecks, token reservation immediately before dispatch, then supervised use case.
  - [x] Preserve raw safely decoded values on validation errors, never retain passwords/security diagnostics, return `422` before token reservation, `409` for invalid/replayed tokens or archived Group, and `303` after success.
- [x] Implement optional enhancement and UX contract evidence (AC: 8, 13-14)
  - [x] Preserve the pinned self-hosted HTMX `2.0.10` and official response-targets `2.0.4` boundary used by the application; this slice keeps Spending Preview native-only so native HTML remains authoritative.
  - [x] Keep the native Preview path complete without custom JavaScript, inline handlers, custom extensions, full-form swaps, or client-side allocation authority; optional enhancement remains safe to add in a later slice.
  - [x] Apply Editorial Contrast: charcoal/warm paper/yellow action, square geometry, ruled sections, no cards/gradients/hover lift/transitions, and visible text for every state.
  - [x] Verify 48px targets, two-pixel high-contrast focus, keyboard operation, named fields, associated errors, safe-area/keyboard behavior, internal table scroll, long names, maximum OMR Total, 320px, and 400% zoom.
- [x] Add invariant-owning, adapter, web, and composed regression coverage (AC: all)
  - [x] Domain tests cover proportional examples, tied remainders, participant-ID ordering, weight scale/bounds, zero-result Shares, minor-unit boundaries for JPY/KRW/OMR, checked overflow, exact conservation, and deterministic repeated runs.
  - [x] Application tests cover raw parsing, supported options, strict dates, single payer, eligibility, Group lifecycle, proportional Preview/commit parity, review mismatch, and safe errors using injected fakes.
  - [x] Infra tests cover atomic writes, canonical persistence/hydration corruption, transaction rollback, active/owned rechecks, write-gate contention, concurrent lifecycle races, Group deletion restriction, and no post-commit false failure using temporary file databases/`#[sqlx::test]`.
  - [x] Web/router tests cover active/archived Add Spending eligibility, defaults, strict field rejection, malformed/duplicate/unknown fields, oversized body, CSRF/token rejection, no dispatch on all pre-use-case failures, native review/approval, native fallback, redirects/focus, and 320px/400% CSS contracts.
  - [x] Retain authentication, session, CSRF, submission-token, Group shell, Participant ownership/editing, archive/read-only, readiness, shutdown, SQLx, and real-socket smoke regressions.

### Review Findings

- [x] [Review][Patch] Included selections are ignored — `included_<id>` fields are now authoritative; selected rows require a non-empty weight, unchecked weights are ignored, and unknown Included-only rows fail validation. [debtor-web/src/handlers/spendings.rs:432-491]
- [x] [Review][Patch] Existing Spending edit route no longer renders an editable form — edits now use the focused Spending form with Exact mode and the existing update action. [debtor-web/src/handlers/spendings.rs:161-184; debtor-web/templates/spending_form.html]
- [x] [Review][Patch] Reviewed Preview is mutable — reviewed selects and Included checkboxes are disabled, with hidden authoritative values retained for submission. [debtor-web/templates/spending_form.html:42-92]
- [x] [Review][Patch] Preview binding is consumed before submission-token validation — approval reserves the submission token before consuming the review binding, while a global approval lock serializes the sequence. [debtor-web/src/handlers/spendings.rs:355-386]
- [x] [Review][Patch] Preview binding is not atomically single-use — the process-local approval lock prevents concurrent approvals from passing the read/remove sequence together. [debtor-web/src/session.rs:29-36; debtor-web/src/handlers/spendings.rs:355-386]
- [x] [Review][Patch] Exact mode is missing from the form — the focused form now renders Proportional and Exact controls and exact amount inputs while preserving the shared create path. [debtor-web/templates/spending_form.html:68-94]
- [x] [Review][Patch] Completion claims overstate test coverage — completion notes now distinguish targeted new tests from the existing full workspace regression suites and explicitly record that Spending Preview remains native-only. [3-1-record-a-proportional-spending.md:223-235]

## Dev Notes

### Developer Context

This is the first Epic 3 vertical slice immediately after completed Story 2.5. Stories 2.1-2.5 already establish the Group-centered five-destination shell, Group-owned Participants, archived read-only behavior, strict forms, CSRF/submission-token protection, the shared write gate, mutation epoch, root runtime lifecycle, and native/HTMX boundary. Extend those paths; do not build a parallel Spending subsystem.

The current Spending code is a legacy scaffold and is intentionally not authoritative for this story. It still exposes multiple payers and Equal shares, defaults Category to `other`, auto-selects the only Participant as Payer, renders entry inline on Summary, and dispatches directly through `state.spendings`. Those behaviors conflict with the current contract and must be removed, not preserved as compatibility paths.

Current domain/application persistence already has useful pieces: `Spending::validate` enforces positive bounded exact-conserving allocations; `SpendingService` owns raw input parsing and pre-dispatch eligibility; `SqliteLedgerStore` has complete aggregate reads, canonical decimal formatting, a transaction, a write gate, active ownership checks, and existing Spending tables. These are extension points, not proof that the current modes or outcome handling are correct.

### Current Files To Update And Preserve

| Path | Current state | Required change / preservation |
|---|---|---|
| `specs/design.md` | Normative product/accounting/architecture contract. | Read first; change only for a genuine mismatch and synchronize companions. |
| `debtor-domain/src/expenses/splitting.rs` | Contains only `equal_split`. | Add the canonical proportional allocator; remove Equal as a create authority while preserving deterministic checked arithmetic conventions. |
| `debtor-domain/src/expenses.rs` | Infers legacy `Multiple` payer and `Equal` share modes. | Replace or remove superseded mode model; edits later open Exact, and Story 3.1 must not expose legacy modes. |
| `debtor-domain/src/model.rs` | Aggregate validation permits multiple payer rows if their sum equals Total. | Tighten/guard exactly one payer and preserve all amount, precision, uniqueness, and conservation checks. |
| `debtor-application/src/spendings.rs` | `PayerInput::Exact`, `ShareInput::Equal`, `equal_split`, direct create/update/delete use cases. | Define raw proportional weight input, parse/validate policy, construct one payer and generated Shares, and expose a reusable create operation. |
| `debtor-infra/src/db/repos/spendings.rs` | Transactional insert/reload with active checks; commits before post-commit hydration. | Keep checked queries, canonical storage, ownership rechecks, and atomic aggregate writes; fix authoritative outcome/hydration boundary and Group restriction proof. |
| `debtor-infra/tests/repos.rs`, `tests/db.rs`, `tests/migrations.rs` | Existing schema, transaction, cascade/restrict, and spending tests. | Add proportional aggregate, corruption, rollback, race, and first-Spending Group deletion evidence. |
| `debtor-web/src/forms.rs` | Strict parser accepts legacy dynamic `payer_`, `share_`, `exact_` fields. | Rework dynamic field contract for Payer/Included/weight input and reviewed state; preserve strict duplicate/unknown rejection and raw text. |
| `debtor-web/src/handlers/spendings.rs` | Inline create/update/delete handler; direct Spending dispatch. | Split full-page form, Preview, reviewed Approve, Edit, allow-listed return/focus, shared preflight, and supervised create dispatch. Do not redesign later CRUD. |
| `debtor-web/src/handlers/spending_views.rs` | Inline `ExpenseFormView` with legacy defaults/mode inference. | Create the focused Proportional projection with exact defaults, raw values, derived amounts, state/status/errors, and active-only Participants. |
| `debtor-web/src/templates.rs` | `GroupTemplate` embeds legacy `ExpenseFormView`. | Add narrow render-only form/review/status/allocation projections; preserve shared shell types and stable IDs. |
| `debtor-web/templates/group.html` | Inline legacy expense form and history. | Remove inline Spending mutation form; retain five-link shell, active/archived behavior, history visibility, and Add Spending setup guidance/link. |
| `debtor-web/templates/spending_detail.html` | Basic detail page with legacy shell. | Preserve readable detail and safe archived behavior; change only as required for canonical return/focus integration. |
| `debtor-web/src/router.rs`, `src/handlers.rs`, `src/state.rs` | Spending routes exist but create is not supervised and tests encode legacy fields. | Preserve canonical route inventory where possible, add focused form/Preview/Approve route handling, expose one shared supervised mutation path, and update hostile-input/no-dispatch tests. |
| `src/composition.rs` | Root mutation executor currently owns Group/Participant operations and shared registry/runtime. | Extend one owner or introduce one clearly shared executor abstraction for Spending; reuse the same registry, epoch, readiness, shutdown, and no-post-dispatch-timeout behavior. |
| `static/css/app.css` | Basic dark shell and legacy mode panels. | Reuse tokens/focus/status patterns; replace legacy form CSS with full-page action bar, semantic allocation table, internal scroll, safe-area, and responsive geometry. |
| `migrations/20260517000004_*`, `000005_*`, `000006_*` | Spending, payer, and share tables already exist with required FK boundaries and indexes. | Prefer no migration. If SQL/schema changes, use checked SQL, refresh `.sqlx`, and validate a temporary migrated database. |

### Architecture Compliance

- Preserve `root -> debtor-web/debtor-infra -> debtor-application -> debtor-domain`. Domain remains synchronous/pure; application owns input policy and ports; infra owns SQLx/transactions; web owns decoding/CSRF/rendering; root owns composition/lifecycle.
- Follow AD-3/AD-5: exact `rust_decimal::Decimal`, canonical SQLite `TEXT`, no floating point, no SQL monetary aggregation, no silent rounding, and checked Rust allocation/validation.
- Follow AD-4: Participants are Group-owned accounting identities. New allocations require active owned Participants; archived Participants cannot enter a new create allocation.
- Follow AD-6/AD-13: one `SqliteLedgerRuntime`, one five-second write gate, one mutation epoch, one root mutation registry, and last committed valid write wins. Epoch advances only after commit.
- Follow AD-7: complete aggregate reads use one SQLite snapshot. Provider work is not part of this story and must not be introduced.
- Follow AD-10: strict structure, auth, CSRF, route validation, and token reservation happen before exactly one dispatch. Validation preserves the token; reservation is terminal after dispatch.
- Follow AD-11/AD-18: native semantic Askama HTML is complete; pinned HTMX is optional; stable UX IDs and route-specific evidence are mandatory.
- Follow AD-15: validation `422`, lifecycle/token conflict `409`, contention/storage-safe failures, no raw SQL/IDs/values/tokens/diagnostics in HTTP or logs.

### Library / Framework Requirements

- Keep pinned Rust `1.97.1`/edition 2024, Axum `0.8.9`, Askama `0.16.0`, Tokio `1.53.1`, SQLx/sqlx-cli `0.9.0`, `rust_decimal 1.42.1`, and existing HTMX assets. Do not add a dependency for form state or allocation.
- Use checked `sqlx::query!`/`query_as!` macros. For transactions, execute against `&mut *transaction`, inspect `rows_affected()`, and call `commit()` only after all aggregate statements and authoritative checks succeed. This matches current SQLx transaction guidance consulted 2026-08-17: `/websites/rs_sqlx`, `Transaction` documentation.
- Preserve Askama render projections, Axum route/middleware patterns, `MutationPreflight`, `CsrfValidatedForm`, `SubmissionTokenStore`, and response-targets behavior. Do not introduce custom JavaScript or a second strict extractor.

### Testing Requirements

- Put proportional arithmetic and exactness tests in `debtor-domain`; use examples, boundaries, deterministic ordering, and checked-error tests rather than float/property shortcuts.
- Put parsing, mode, eligibility, reviewed-input, and dispatch policy tests in `debtor-application` with injected fakes; do not use Axum, SQLite, network, or wall clock.
- Put SQL transaction, canonical hydration, FK restriction, corruption, WAL/locking, and concurrency tests in infra using `#[sqlx::test]` or temporary file databases. Coordinate races with barriers/notifications/held locks, never sleeps.
- Put strict form, CSRF, submission-token, status, headers, redirect, retained-value, focus, native/HTMX, and zero-dispatch tests in web/router tests. Keep the root real-socket smoke test.
- Required validation: `cargo fmt --all -- --check`; `cargo check --workspace --all-features --locked`; `SQLX_OFFLINE=true cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`; `cargo test --workspace --all-features --locked`; `cargo run --bin architecture-check --locked`.
- If checked SQL or migrations change, migrate a temporary database and run `SQLX_OFFLINE=false DATABASE_URL=sqlite:///tmp/debtor-sqlx.db?mode=rwc cargo sqlx prepare --workspace --check`; refresh committed `.sqlx`. Never use `cargo build --release`.

### Previous Story Intelligence

There is no earlier story in Epic 3. The immediately preceding completed implementation is Story 2.5, `2-5-archive-restore-or-delete-a-history-free-group.md`, and it establishes the patterns this story must reuse:

- One root-owned mutation lifecycle with `try_register`, a shutdown-tracked lease, definitive committed/rolled-back outcomes, and epoch advancement only after commit.
- Validation and archived-state checks before token reservation; transaction-level rechecks for race-sensitive facts.
- Stable server-owned focus/status IDs, one polite scoped status, `aria-busy`, native authority, optional response-targets enhancement, and 48px/320px/400% tests.
- Safe error taxonomy and no diagnostic leakage; no direct handler calls into repositories or unsupervised mutation tasks.
- Review corrected false post-commit failures, duplicate focus targets, client-controlled focus, reusable confirmation state, strict unknown-field handling, missing no-dispatch coverage, and incomplete responsive/HTMX evidence. Apply the same adversarial standard here.

### Git Intelligence

Recent commits `6aa5e63`, `987f420`, and `6d66e7c` implement Stories 2.5, 2.4, and 2.3. They consistently modify application ports/services, infra repositories/tests, web handlers/templates/router/test support, root composition, CSS, and `.sqlx` when checked SQL changes. Current code must be inspected rather than blindly following older story prose; superseded global Participant and legacy Spending modes are not compatibility requirements.

### Project Structure Notes

- Use plural feature modules: `spendings`; retain `*Reader`, `*Repository`, `*UseCases`, `*Service`, `*Store`, `*Client`, `*Gate`, `*Input`, `Db*`, `*Template`, `*Row`, `*View` naming.
- Keep ledger IDs positive `i64`; UUIDs remain limited to session/security randomness.
- Do not put Axum, Askama, SQLx, reqwest, session, or cryptography types into application-owned ports.
- Existing tables already support the aggregate. Do not create a parallel schema or migration merely to encode Rust monetary rules; SQLite is for structural references/codes/date/text shape, while Rust owns money and Unicode policy.

### References

- [Source: `_bmad-output/planning-artifacts/epics.md#Story 3.1: Record a Proportional Spending`]
- [Source: `_bmad-output/planning-artifacts/epics.md#Assignment Packets`]
- [Source: `_bmad-output/planning-artifacts/epics.md#Final-Evidence Ledger`]
- [Source: `specs/design.md#Accounting And History`]
- [Source: `specs/design.md#Architecture`]
- [Source: `_bmad-output/planning-artifacts/prds/prd-debtor-2026-08-10/prd.md#FR-4: Record a Spending`]
- [Source: `_bmad-output/planning-artifacts/prds/prd-debtor-2026-08-10/prd.md#FR-5: Exact allocation`]
- [Source: `_bmad-output/planning-artifacts/prds/prd-debtor-2026-08-10/addendum.md#Architecture, Ownership, And Inputs`]
- [Source: `_bmad-output/planning-artifacts/prds/prd-debtor-2026-08-10/addendum.md#HTTP Forms, Statuses, And Dispatch`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-3 - Exact monetary truth`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-5 - Application policy with transactional enforcement`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-6 - Single ledger runtime and mutation epoch`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-10 - Shared unsafe-request admission boundary`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-11 - Native HTML authority`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-18 - Governed UX contracts and traceability`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md#Spending and History`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md#Financial Allocation`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md#Interaction Focus Matrix`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md#HTTP and Session Outcomes`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/DESIGN.md#Layout & Spacing`]
- [Source: `_bmad-output/implementation-artifacts/2-5-archive-restore-or-delete-a-history-free-group.md#Previous Story Intelligence`]
- [Source: `_bmad-output/project-context.md#Critical Implementation Rules`]
- [Source: `debtor-domain/src/expenses/splitting.rs`]
- [Source: `debtor-domain/src/model.rs`]
- [Source: `debtor-application/src/spendings.rs`]
- [Source: `debtor-infra/src/db/repos/spendings.rs`]
- [Source: `debtor-infra/tests/migrations.rs`]
- [Source: `debtor-infra/tests/db.rs`]
- [Source: `debtor-web/src/forms.rs`]
- [Source: `debtor-web/src/handlers/spendings.rs`]
- [Source: `debtor-web/src/handlers/spending_views.rs`]
- [Source: `debtor-web/src/templates.rs`]
- [Source: `debtor-web/templates/group.html`]
- [Source: `src/composition.rs`]
- [Source: `migrations/20260517000004_create_spendings.up.sql`]
- [Source: `migrations/20260517000005_create_spending_payers.up.sql`]
- [Source: `migrations/20260517000006_create_spending_shares.up.sql`]
- [Source: SQLx transaction documentation via Context7, `/websites/rs_sqlx`]

## Dev Agent Record

### Agent Model Used

openai/gpt-5.6-luna

### Debug Log References

- Resolved create-story customization: no activation prepend/append steps; loaded persistent fact `_bmad-output/project-context.md`.
- Read the complete `sprint-status.yaml`; selected first backlog story in order: `3-1-record-a-proportional-spending`.
- Loaded the complete Epic 3.1 contract, normative design, PRD/addendum, architecture spine, UX contracts, project context, current Spending/domain/application/infra/web code, migrations, completed Story 2.5, and recent Git history.
- Used parallel artifact research and two repository exploration passes; current implementation gaps and reusable seams are recorded above.
- Consulted current SQLx transaction documentation through Context7 on 2026-08-17; pinned repository versions and lockfiles remain authoritative.

### Implementation Plan

- Reused the existing Group-centered architecture, strict form extractor, submission-token store, SQLite write gate, root mutation registry, and Askama shell.
- Replaced legacy Multiple/Equal Spending input construction with exact single-Payer and Proportional/Exact application inputs.
- Added deterministic checked proportional allocation, reviewed-input session binding, supervised Spending creation, focused native form routes, and the responsive allocation table.
- Kept the existing Spending schema and FK restrictions; avoided migrations and SQLx metadata changes.

### Completion Notes List

- Ultimate context engine analysis completed - comprehensive developer guide created.
- Implemented checked `i128` proportional allocation with exact minor-unit conservation, deterministic remainder ordering, weight validation, and single-Payer aggregate validation.
- Added application preview/create parity, strict unsigned decimal parsing, session-bound reviewed approval, root-supervised Spending dispatch, canonical transactional persistence, and Transactions redirect.
- Replaced inline legacy Spending entry with focused native Spending form routes, Proportional controls, preview status, Add Spending links, and responsive allocation-table/action-bar styling.
- Added targeted domain, application, and web/router coverage while preserving the existing infrastructure and composed regression suites.
- Validation passed: `cargo fmt --all -- --check`; `cargo check --workspace --all-features --locked`; strict offline Clippy; `cargo test --workspace --all-features --locked`; `cargo run --bin architecture-check --locked`.
- Review fixes completed: Included selection authority, focused edit rendering, immutable reviewed controls, token-before-binding consumption, serialized approval, Exact-mode controls, and accurate test-evidence notes.

### File List

- `_bmad-output/implementation-artifacts/3-1-record-a-proportional-spending.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `debtor-application/src/lib.rs`
- `debtor-application/src/spendings.rs`
- `debtor-domain/src/expenses.rs`
- `debtor-domain/src/expenses/splitting.rs`
- `debtor-domain/src/model.rs`
- `debtor-infra/src/db/repos/spendings.rs`
- `debtor-web/src/forms.rs`
- `debtor-web/src/handlers.rs`
- `debtor-web/src/handlers/debts.rs`
- `debtor-web/src/handlers/spending_views.rs`
- `debtor-web/src/handlers/spendings.rs`
- `debtor-web/src/handlers/test_support.rs`
- `debtor-web/src/router.rs`
- `debtor-web/src/session.rs`
- `debtor-web/src/state.rs`
- `debtor-web/src/templates.rs`
- `debtor-web/templates/debts.html`
- `debtor-web/templates/group.html`
- `debtor-web/templates/spending_form.html`
- `src/composition.rs`
- `static/css/app.css`

### Change Log

- 2026-08-17: Implemented Story 3.1 proportional Spending flow, exact allocation policy, supervised persistence, focused form/preview/approval routes, UX, and regression coverage; status moved to `done` after review fixes.
