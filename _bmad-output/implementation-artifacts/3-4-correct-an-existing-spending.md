---
story_key: 3-4-correct-an-existing-spending
story_id: 3.4
epic: 3
status: done
baseline_commit: 0fc43805c72317e99b111cba43e2f89c3d2dc663
created: 2026-08-17
completion_note: Ultimate context engine analysis completed - comprehensive developer guide created
---

# Story 3.4: Correct an Existing Spending

Status: done

## Story

As the administrator,
I want to edit a complete Spending and its allocations,
so that corrections preserve exact accounting and archived historical identities.

## Acceptance Criteria

1. **Exact-on-edit loading:** Given an active Group owns a Spending, the edit GET direct-loads one complete aggregate, renders stored Payer and Share amounts in Exact mode, and never reconstructs or persists Proportional mode or weights.
2. **Archived role retention:** Given the stored Payer or Share Participant is archived, the edit form retains that identity and stored amount only in its existing role; active, Group-owned Participants remain available for otherwise valid changes. Archived identities have visible `Archived` text and are not presented as unrestricted new-role choices.
3. **Archived-role validation:** Introducing an archived identity, moving an archived Payer to Shares, moving an archived Share to Payer, or adding an archived identity to another role returns `422`, retains safely decoded raw input, performs no replacement, and does not consume the submission token before dispatch.
4. **Shared exact validation:** Preview and commit reparse description, date, category, Source Currency, precision, Payer, and Shares through the same exact validation/conservation path as creation. The Payer amount and Share amounts each equal the Total exactly in Source Currency minor units.
5. **Atomic replacement:** A dispatched valid update rechecks Group ownership, active eligibility for newly introduced roles, and archived-role retention under the write gate/transaction, then atomically replaces scalar Spending fields, Payer, and all Shares. Any failure leaves the old complete aggregate intact.
6. **Last committed write wins:** Two admitted valid edits produce either the old or new complete aggregate; the last commit is authoritative. Do not add revision columns, optimistic stale-edit conflicts, or mixed scalar/allocation reads.
7. **Protected success:** Successful commit publishes the authoritative result and returns `303 See Other` to the canonical Transactions/Spending context showing the corrected aggregate. A replayed or consumed token returns `409` without dispatching another update.
8. **Archived Group boundary:** Direct edit GET, edit Preview, and update requests for an archived Group return `409` before use-case invocation. Read-only detail remains available.
9. **Edit UX:** Edit opened from a Transaction row focuses the stable form `h1`; Exact mode shows stored Payer/Shares; retained archived identities are visibly labeled; Cancel uses only an allow-listed canonical row return target; existing full-page form, allocation-table, action-bar, responsive, and accessibility geometry remains intact.
10. **Native reviewed Preview:** Native edit Preview renders non-editable reviewed input. Approve is bound to the reviewed Spending ID and exact ordered raw fields, including Source Currency. Edit allocation restores editable controls. Any field/revision change invalidates approval; stale, changed, mismatched, invalid, or superseded review state cannot dispatch.
11. **Enhanced Preview:** Overlapping enhanced edit Preview requests are latest-input-wins. Superseded responses cannot swap content or re-enable Approve; only derived cells/status/approval state may change. Focus, caret, selection, keyboard, active row, and scroll remain unchanged, and stale enhanced state cannot alter retained archived rows. Native HTML remains complete without HTMX.
12. **Canonical post-commit focus:** After a successful edit, including a date change that reorders history, the response encodes the authoritative Transactions page and disclosure ID and focuses the committed Transaction `<summary>` without a completion badge. Failures focus the form heading or linked error and retain safely decoded values.

Requirements: `SPEC-FR43..SPEC-FR44`, `SPEC-FR46..SPEC-FR51`, `SPEC-FR56..SPEC-FR63`, `SPEC-FR66`, `SPEC-FR87..SPEC-FR90`; `SPEC-NFR3`, `SPEC-NFR5..SPEC-NFR10`, `SPEC-NFR14..SPEC-NFR16`, `SPEC-NFR22`, `SPEC-NFR25`, `SPEC-NFR28..SPEC-NFR34`; UX contracts `UX-TARGET-01`, `UX-ALLOC-01`, `UX-FOCUS-01`, `UX-STATUS-01`, `UX-PREVIEW-NATIVE-01`, `UX-PREVIEW-LATEST-01`, `UX-RESPONSIVE-01`, `UX-VISUAL-01`.

## Scope Boundary

- Implement the existing Spending edit form, edit Preview/Approve binding, archived-role retention/validation, complete atomic replacement, supervised update dispatch, and canonical Transactions return/focus behavior.
- Reuse Stories 3.1-3.3: `SpendingService`, `SpendingReader`/`SpendingRepository`, Exact parsing/conservation, complete aggregate loading, shared strict form/CSRF/submission-token pipeline, root mutation executor, write gate, mutation epoch, Transactions projections, and stable row IDs.
- Do not implement new Spending creation, delete behavior (Story 3.5), history/detail redesign (Story 3.3), summaries, rates, debts, settlements, Participant archival/restore, or Group lifecycle changes.
- Do not add a second repository, mutation executor, write gate, epoch, review store, token store, financial algorithm, schema compatibility shim, optimistic revision, stale-edit conflict, custom JavaScript, inline script, custom HTMX extension, modal, or client-side accounting authority.
- Prefer no migration. If checked SQL or migrations genuinely change, update `specs/design.md` first when behavior changes, refresh `.sqlx`, migrate a temporary SQLite database, and run online SQLx preparation/check.

## Tasks / Subtasks

- [x] Audit and preserve the accepted 3.1-3.3 seams before editing (AC: all)
  - [x] Read current files and tests listed in Dev Notes; do not trust this packet over repository reality.
  - [x] Preserve create-mode Preview/Approve, Transactions browse/detail, current-name projections, archived read-only behavior, sign-out, and native fallback.
- [x] Complete application update policy and mutation boundary (AC: 3-7)
  - [x] Keep raw parsing and Exact construction in `debtor-application`; validate original-vs-updated Payer and Share role sets so archived identities may only retain their exact existing role.
  - [x] Ensure invalid input is rejected before submission-token reservation and repository dispatch where possible; validation never consumes the token.
  - [x] Extend the existing `SpendingMutationExecutor`/root composition path for updates rather than calling `update_input` directly from web handlers. Preserve definitive commit/rollback publication, write-gate/epoch behavior, and last-commit-wins semantics.
- [x] Implement edit-specific reviewed Preview (AC: 1, 4, 9-11)
  - [x] Add an edit Preview route or shared route variant while retaining the valid native full-page path; bind review state to Group, Spending ID, and ordered raw fields.
  - [x] Reuse create review locking and exact matching; consume matching review only for Approve, not for Preview or pre-dispatch validation.
  - [x] Restrict existing edits to Exact presentation and reject stale/inactive mode-specific fields under the strict known-field contract.
  - [x] Preserve raw values, row errors, `aria-invalid`/`aria-describedby`, one polite status, and `aria-busy` behavior.
- [x] Fix form projection and archived-role UX (AC: 1-3, 9-11)
  - [x] Direct-load the complete aggregate for edit and retain stored amounts.
  - [x] Make archived retained identities visible only in their original Payer/Share role, with explicit `Archived` text; active owned identities remain selectable for valid changes.
  - [x] Keep the single semantic allocation table, required `520px` intrinsic width and `116/76/76/92/160` columns, sticky identity, 48px controls, internal-only scrolling, safe-area action bar, and no page-level horizontal scroll.
- [x] Replace the complete aggregate atomically and safely (AC: 3-8)
  - [x] Reuse `save_spending`/`update_spending` transaction structure unless tests expose a defect; recheck ownership, active eligibility, role retention, and archived Group state authoritatively inside the transaction.
  - [x] Preserve canonical Decimal `TEXT`, checked SQLx macros, restrictive foreign keys, rollback on any scalar/allocation failure, and sanitized error mapping.
- [x] Implement canonical return and focus behavior (AC: 7, 9, 12)
  - [x] Carry only allow-listed Transactions page/disclosure/focus context from the invoking row; never accept arbitrary URLs, selectors, cursor text, or IDs for logging/error output.
  - [x] After commit reload the canonical page and focus the committed `<summary>` even if date/currency changes reorder the row; on failure focus the form error/heading and retain safe input.
  - [x] Preserve no completion badge and native/enhanced response parity.
- [x] Add invariant-owning tests (AC: all)
  - [x] Application tests cover Exact update parity, Source Currency correction, invalid input before repository update, archived Payer/Share retention, role changes, new archived identities, active additions, Group scoping, and no dispatch on validation failure.
  - [x] Infrastructure tests cover complete scalar/Payer/Share replacement, rollback/no orphan rows, canonical persistence, archived-role and Group checks, ownership races, and complete old/new snapshots under concurrent updates using barriers/notifications, never sleeps.
  - [x] Web/router tests cover Exact-on-edit, visible archived labels/role restrictions, edit-ID-bound review, Source Currency and field invalidation, `422` retained values/no token consumption, `409` replay/archived Group, native/enhanced parity, latest-input-wins, stable status/focus IDs, reorder-aware canonical row focus, and no completion badge.
  - [x] Preserve root real-socket smoke and architecture/security regressions; do not claim geometry/contrast verification without a browser harness.
- [x] Run validation (AC: all)
  - [x] `cargo fmt --all -- --check`
  - [x] `cargo check --workspace --all-features --locked`
  - [x] `SQLX_OFFLINE=true cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`
  - [x] `cargo test --workspace --all-features --locked`
  - [x] `cargo run --bin architecture-check --locked`
  - [x] Independent password-helper fmt, Clippy, and locked tests.
  - [x] No SQL/migrations changed; SQLx metadata refresh was not required.

### Review Findings

- [x] [Review][Patch] Edit Preview omits the required `split_mode` field [debtor-web/templates/spending_form.html:71-77] — fixed by retaining the Exact mode field in edit forms.
- [x] [Review][Patch] Preserve submission tokens until the edit review binding matches [debtor-web/src/handlers/spendings.rs:565-578] — fixed by checking review binding before token reservation and consuming the review only after reservation.
- [x] [Review][Patch] Use the edit route for edit approval-conflict recovery [debtor-web/src/handlers/spendings.rs:572-576] — fixed with an edit-specific recovery target.
- [x] [Review][Patch] Return to a canonical page containing the committed Spending [debtor-web/src/handlers/spendings.rs:587-595] — fixed by redirecting successful corrections to the authoritative Spending detail route.
- [x] [Review][Patch] Retain invalid newly introduced Participant fields in edit errors [debtor-web/src/handlers/spending_views.rs:153-185,513-562] — fixed by projecting submitted Participant rows and preserving unmapped dynamic fields as hidden raw inputs.
- [x] [Review][Patch] Show `Archived` text for retained Share identities [debtor-web/templates/spending_form.html:84-90] — fixed with visible Archived text in the allocation identity column.
- [x] [Review][Patch] Implement the specified enhanced edit Preview concurrency behavior [debtor-web/templates/spending_form.html:32-104] — fixed with native-preserving HTMX edit Preview enhancement and `hx-sync` latest-request replacement.
- [x] [Review][Patch] Preserve unknown mutation outcomes in the root update executor [src/composition.rs:123-140] — fixed with a distinct `StorageReason::Unknown` response and service-unavailable mapping after readiness failure.
- [x] [Review][Patch] Add end-to-end correction-path regression coverage [debtor-web/src/router.rs:113-118, debtor-web/src/handlers/spendings.rs:171-623] — fixed by extending review-binding and safe-outcome regression coverage and passing the complete router/application/infrastructure/runtime suite.

## Dev Notes

### Developer Context

Epic 3 already has exact create and history/detail foundations. This story is the correction vertical slice. The current repository has a useful but incomplete update path: `SpendingService::update_input` parses Exact input and `save_spending(..., update = true)` transactionally replaces scalar fields/Payer/Shares, but the web path commits directly without edit Preview, and the root mutation executor exposes only create. Extend those seams; do not redesign Spending accounting.

The current handler loads the aggregate with `state.spendings.spending`, then builds a broad Group projection. It initially renders Exact values but still exposes both mode controls, does not visibly label archived retained identities in the form, has no Spending-ID-bound review state, reserves the update token before application validation, redirects successful updates to Summary, and does not preserve the invoking Transactions page/disclosure. These are implementation gaps to fix, not reasons to create parallel flows.

### Current Files To Update And Preserve

| Path | Current state | Story-specific change/preservation |
|---|---|---|
| `debtor-application/src/spendings.rs` | `SpendingInput`, Exact parsing, `update_input`, role-set eligibility, `SpendingRepository::update_spending`, and `SpendingMutationExecutor` with create only. | Preserve exact checked parsing and role policy; add only the narrow update-dispatch contract/review-facing data needed. Keep adapters out of ports and prevent repository update on validation failure. |
| `debtor-infra/src/db/repos/spendings.rs` | Complete aggregate loader and transactional replacement with write gate, scalar update, allocation delete/reinsert, and lifecycle checks. | Reuse and strengthen tests/role checks if needed; preserve canonical TEXT, checked SQLx, rollback, ownership, and archived-role retention. Do not add monetary SQL. |
| `debtor-web/src/handlers/spendings.rs` | Edit GET loads `Spending`; update POST parses and reserves token before direct `update_input`; create has shared review flow; success redirects to `/groups/{id}`. | Add edit Preview/Approve binding, pre-dispatch validation ordering, supervised update dispatch, safe return context, retained validation, and canonical redirect/focus. Keep create behavior unchanged. |
| `debtor-web/src/handlers/spending_views.rs` | `expense_view` infers Exact for stored edits; `build_group_template` adds historical IDs to a common member list; Transactions rows already have `focused` but builder sets it false. | Make edit Exact/role projection authoritative, visibly label archived rows, and build canonical focused Transactions context without making templates calculate policy. |
| `debtor-web/src/forms.rs` | Shared strict ordered parser for scalar and dynamic payer/included/weight/exact fields. | Preserve duplicate/unknown/malformed rejection and wire order; enforce Exact-only edit semantics and avoid silently accepting inactive mode fields. |
| `debtor-web/src/session.rs` | Existing create-oriented ordered review state and approval lock. | Bind reviewed input to edit Group/Spending ID and invalidate on any raw field change; reuse the same store/lock, no second review store. |
| `debtor-web/src/templates.rs` | `SpendingFormTemplate`, `ExpenseFormView`, `MemberRow`, `TransactionRow` and stable IDs. | Add minimal edit review/return/focus/role flags; preserve typed Askama projections and stable status/error associations. |
| `debtor-web/templates/spending_form.html` | Full-page form with native Preview/Approve; reviewed action currently points to create flow; edit still renders mode controls. | Support edit review and Approve, Exact-only edit, archived labels/role restrictions, allow-listed Cancel, one status node, and native fallback. Do not add JS or modal behavior. |
| `debtor-web/templates/transactions.html` | Native `<details>` rows, stable summary IDs, pagination, optional Edit/Delete, but no committed-row focus input. | Preserve Story 3.3 semantics; add only canonical focused-row/disclosure behavior required after correction. |
| `debtor-web/src/router.rs`, `debtor-web/src/handlers.rs`, `debtor-web/src/handlers/test_support.rs` | Existing edit/update routes and authenticated fake composition. | Add route wiring/tests for edit Preview if needed; reuse auth, strict form, CSRF, token, timeout, and fake seams. |
| `src/composition.rs`, `debtor-web/src/state.rs` | Root-owned create mutation executor is composed into state; update currently bypasses it. | Extend the same executor/registry/lifecycle path for update; no second executor/gate/epoch. |
| `static/css/app.css` | Accepted focused Spending form and Transactions Editorial Contrast geometry. | Preserve form/table/action-bar dimensions, focus, contrast, and responsive behavior; only add minimal edit/review/archived/focus styling. |
| `debtor-infra/tests/repos.rs` and web/application tests | Existing creation/history and basic update coverage. | Add invariant-owning correction, rollback, role, concurrency, review, response, focus, and native/enhanced tests. |
| `.sqlx/*`, `migrations/*`, `specs/design.md` | Existing schema and checked metadata are sufficient by current evidence. | Prefer no changes. If behavior/schema changes, update normative design first and refresh metadata with the required temporary database check. |

### Architecture Compliance

- Preserve `debtor (root) -> debtor-web/debtor-infra -> debtor-application -> debtor-domain` and AD-2 ownership. Domain stays pure; application owns raw input/policy/use cases; infra owns SQLx and authoritative transaction checks; web owns HTTP/session/CSRF/rendering; root owns composition/lifecycle.
- Apply AD-3: exact checked `rust_decimal::Decimal`, canonical plain decimal SQLite `TEXT`, currency precision (JPY/KRW 0, OMR 3, all others 2), positive bounded amounts, no float, rounding, zero substitution, SQL monetary arithmetic, or raw diagnostics.
- Apply AD-4/AD-5: Participants are Group-owned accounting identities. New roles require active owned Participants. An archived identity may only remain in the same stored Payer or Share role; persistence rechecks this under the committing transaction.
- Apply AD-6/AD-7: use one complete aggregate read, the existing five-second write gate, mutation epoch, and root mutation registry. Last committed valid write wins; no optimistic conflict. No provider I/O or database transaction spanning provider I/O is relevant here.
- Apply AD-10/AD-14: strict bounded form extraction, authentication, CSRF and route checks precede application dispatch; pre-dispatch validation preserves the token; reserve atomically immediately before one dispatch; after dispatch do not cancel the mutation with a generic timeout.
- Apply AD-11/AD-18: semantic Askama/native HTML is authoritative. HTMX `2.0.10` and official response-targets `2.0.4` are optional enhancement only; enhanced responses must preserve native URL/action semantics, latest-input-wins interaction state, and scoped derived/status swaps. No custom JavaScript, inline scripts, custom extensions, or client-side financial authority.
- Apply AD-15: `422` for validation, `409` for token/lifecycle conflicts, safe bounded storage/runtime responses, and no logging or response of credentials, tokens, IDs, amounts, cursor/return input, SQL, or adapter diagnostics.

### Library / Framework Requirements

- Keep Rust `1.97.1` edition 2024, Axum `0.8.9`, Askama `0.16.0`, Tokio `1.53.1`, SQLx `0.9.0`, `rust_decimal 1.42.1`, HTMX `2.0.10`, response-targets `2.0.4`, lockfiles, and existing features. Add no pagination, form-state, or JavaScript dependency.
- Axum's typed `Path`/`Query` extractors and route handlers should remain at the web boundary; convert validated values into application-owned types, never expose Axum extractors through ports. [Context7: `/tokio-rs/axum`, Axum extractors/routing documentation, consulted 2026-08-17]
- Askama `#[derive(Template)]` with `#[template(path = "...")]` remains the typed compile-time rendering boundary. Keep financial policy, SQL, and request parsing out of templates. [Context7: `/askama-rs/askama`, template struct/path documentation, consulted 2026-08-17]
- HTMX may enhance valid native links/forms using `hx-post`/`hx-get`/boosted navigation, but native `action`/`href` and full-page responses remain authoritative. Do not use private history snapshots or custom event JavaScript for review correctness/focus. [Context7: `/bigskysoftware/htmx`, progressive enhancement and `hx-boost` documentation, consulted 2026-08-17]
- For SQLx changes, use checked `query!`/`query_as!` macros, execute all transaction statements through the mutable transaction, commit only after complete aggregate writes/checks, and refresh committed offline metadata.

### Testing Requirements

- Application: use injected fakes, no Axum/SQLite/network/wall clock. Assert Preview/update parity, Exact conservation, Source Currency correction, Group scope, archived role retention/rejection, active new-role eligibility, and no repository update on validation failure.
- Infrastructure: use `#[sqlx::test]` or temporary SQLite. Assert complete replacement, canonical Decimal hydration, rollback/no orphan rows, Group/Participant ownership and archived-role checks, archived Group rejection, and concurrent old-or-new complete snapshots. Coordinate concurrency with barriers/notifications/held locks, never sleeps.
- Web/router: assert edit GET Exact values, visible archived labels and role restrictions, edit-ID-bound ordered review, field/Source Currency invalidation, `422` retained values and token preservation, `409` replay/archived Group/no dispatch, supervised update path, native/enhanced parity, latest-input-wins, stable `h1`/status/`aria-busy` associations, allow-listed return targets, reorder-aware summary focus, and no completion badge.
- Template/CSS: assert semantic allocation table, required IDs/labels, one status node, reviewed hidden values exactly once, 48px target classes/attributes, archived text, and no page-level horizontal scroll declarations. Do not claim 320px/400% geometry or contrast without a browser harness; manual verification must cover long names, maximum OMR values, keyboard focus, and safe-area action bar.
- Preserve root real-socket startup/authentication/CSRF/read/shutdown smoke coverage and architecture fitness. Validate with the commands in Tasks; never use `cargo build --release`.

### Previous Story Intelligence

From Story 3.2 and Story 3.3:

- Reuse the single aggregate path, shared root mutation lifecycle, Exact parsing/conservation, review binding, approval lock, strict form extractor, and complete history/detail projections. Do not create duplicate repositories, readers, mutation executors, token/review stores, or allocation algorithms.
- Native HTML is the source of truth. HTMX must not determine correctness, URL semantics, focus, or financial output. Reviewed approval must be bound to exact ordered raw input; do not normalize/sort/drop fields for review matching.
- Stored edits are Exact because mode/weights are transient. Current Participant names are projections; never persist or render creation-time names. Historical archived identities remain readable and retain their original role/amount.
- Recent review fixes established strict unknown/duplicate field rejection, one polite associated status, checked allocation arithmetic, no raw diagnostics, stable focus IDs, and no false completion. Apply the same standards to correction failures and post-commit redirects.
- Story 3.3's Transactions route is canonical: preserve `<details>` rows, stable `spending-{id}-summary` IDs, fixed 25-item keyset ordering, current-name projections, archived-readable detail, `UX-SHELL-01`, and action suppression for archived Groups.

### Git Intelligence

- Recent commits are story-oriented and extend existing layers: `0fc4380 feat: implement 3-3 bmad`, `120c4ca feat: implement 3-2 bmad`, `7be36b1 feat: implement 3-1 bmad`, followed by the Group/Participant stories.
- Story 3.3 added complete read projections, current/archived identity resolution, Transactions templates, cursor/status/focus markup, and SQLx metadata. Story 3.4 should extend those focused seams rather than reintroduce the old Group table or broad history projection.
- Story 3.2 added Exact form/review behavior and fixed review-binding, checked arithmetic, mode-field, status-association, row-error, and insufficient-minor-unit issues. Do not regress create Preview/Approve while adding edit review.

### Project Structure Notes

- Feature modules remain plural (`spendings`); use `*Input`, `*Reader`, `*Repository`, `*UseCases`, `*Service`, `*Store`, `*Template`, `*View`, and `*Row` naming.
- Ledger IDs are positive `i64`; UUIDs remain limited to session/security randomness. No participant/user/tenant abstraction may be introduced.
- No schema or migration is expected. Existing spending payer/share foreign keys and canonical persistence support complete replacement. If a query changes, refresh `.sqlx` and run online preparation against a migrated temporary SQLite database.
- Keep Summary, Manage, Add Spending, Transactions, direct detail, sign-out, authenticated headers, and archived read-only behavior working end-to-end.

### UX Guardrails

- `UX-TARGET-01`: every edit field, selector, allocation control, action, link, disclosure, and navigation target is at least 48 by 48 CSS pixels at 320px/400% zoom.
- `UX-ALLOC-01`: keep the labeled, keyboard-focusable semantic allocation table, sticky Participant identity, explicit associations, exact `520/116/76/76/92/160` geometry, long-name wrapping, and internal-only horizontal scrolling.
- `UX-FOCUS-01`: edit GET focuses stable `spending-heading`; Preview/error uses one allow-listed forward target; Cancel uses an allow-listed return target; committed correction focuses the canonical row summary, including after reorder.
- `UX-STATUS-01`: use one stable polite atomic status with owning-region `aria-busy`; announce pending/error/commit once; never make individual amounts live regions.
- `UX-PREVIEW-NATIVE-01`: reviewed edit is non-editable and server-bound; Approve is only for the current review; Edit allocation returns to editable state; changing any field invalidates it.
- `UX-PREVIEW-LATEST-01`: enhanced previews are latest-input-wins, derived-only/status-only swaps, and preserve focus/caret/selection/keyboard/scroll. Archived retained rows cannot be changed by stale responses.
- `UX-RESPONSIVE-01`: preserve one full-page document scroll owner, safe-area/dynamic-viewport action bar, keyboard scroll margin, and no page-level horizontal scrolling.
- `UX-VISUAL-01`: retain dark Editorial Contrast, warm paper text, ruled sections, square controls, explicit Archived/status text, and no cards, gradients, authored transitions, hover lift, or completion badges.

### References

- [Source: `_bmad-output/planning-artifacts/epics.md#Story 3.4: Correct an Existing Spending`]
- [Source: `_bmad-output/planning-artifacts/epics.md#Epic 3: Record and Maintain Exact Spendings`]
- [Source: `_bmad-output/planning-artifacts/epics.md#Final-Evidence Ledger`]
- [Source: `specs/design.md#Architecture`]
- [Source: `specs/design.md#Accounting And History`]
- [Source: `specs/design.md#Security`]
- [Source: `specs/design.md#Operational Limits`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-2 - Layer responsibility ownership`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-3 - Exact monetary truth`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-4 - Group-owned identity and history`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-5 - Application policy with transactional enforcement`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-6 - Single ledger runtime and mutation epoch`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-7 - Snapshot-complete calculation reads`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-10 - Shared unsafe-request admission boundary`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-11 - Native HTML authority`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-15 - Safe failures, logging, and readiness`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-18 - Governed UX contracts and traceability`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md#Spending and History`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md#Financial Allocation`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md#HTTP and Session Outcomes`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md#Interaction Focus Matrix`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/DESIGN.md#Layout & Spacing`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/DESIGN.md#Components`]
- [Source: `_bmad-output/implementation-artifacts/3-1-record-a-proportional-spending.md#Previous Story Intelligence`]
- [Source: `_bmad-output/implementation-artifacts/3-2-record-a-spending-with-exact-shares.md#Previous Story Intelligence`]
- [Source: `_bmad-output/implementation-artifacts/3-3-browse-and-inspect-spending-history.md#Previous Story Intelligence`]
- [Source: `_bmad-output/project-context.md#Critical Implementation Rules`]
- [Source: `debtor-application/src/spendings.rs`]
- [Source: `debtor-infra/src/db/repos/spendings.rs`]
- [Source: `debtor-web/src/handlers/spendings.rs`]
- [Source: `debtor-web/src/handlers/spending_views.rs`]
- [Source: `debtor-web/src/forms.rs`]
- [Source: `debtor-web/src/session.rs`]
- [Source: `debtor-web/src/templates.rs`]
- [Source: `debtor-web/templates/spending_form.html`]
- [Source: `debtor-web/templates/transactions.html`]
- [Source: `src/composition.rs`]
- [Source: `static/css/app.css`]
- [Source: Context7 `/tokio-rs/axum`, `/askama-rs/askama`, `/bigskysoftware/htmx`, consulted 2026-08-17]

## Dev Agent Record

### Agent Model Used

openai/gpt-5.6-luna

### Debug Log References

- Resolved customization with no activation prepend/append steps and loaded `_bmad-output/project-context.md` as a persistent fact.
- Read the complete ordered `_bmad-output/implementation-artifacts/sprint-status.yaml`; selected first backlog story `3-4-correct-an-existing-spending`. Epic 3 was already `in-progress`.
- Loaded Epic 3, PRD, architecture spine, UX `DESIGN.md`/`EXPERIENCE.md`, normative `specs/design.md`, project context, Stories 3.1-3.3, current edit/update files, and recent Git history.
- Parallel repository audit identified direct-update, missing edit-review, archived-role presentation, token-order, root-dispatch, and reorder-focus gaps.
- Consulted current Axum, Askama, and HTMX documentation through Context7 on 2026-08-17; pinned project versions and lockfiles remain authoritative.

### Implementation Plan

- Extend the application Spending port with side-effect-free update validation and extend the existing root mutation executor so updates use the same supervised lifecycle, write gate, epoch, and definitive outcome path as creates.
- Bind edit Preview state to Group ID, Spending ID, and ordered raw fields in the existing session review store; reserve the submission token only after validation and require a matching review for Approve.
- Reuse the Exact form and aggregate replacement path, constrain edits to Exact mode, expose archived identities only in retained roles, and preserve native HTML/optional HTMX behavior.
- Return to Transactions with an allow-listed focused Spending ID, retain the accepted Transactions projection, and verify formatting, Clippy, workspace tests, architecture fitness, and the independent password helper.

### Completion Notes List

- Ultimate context engine analysis completed - comprehensive developer guide created.
- Implemented Exact-only existing Spending correction with edit Preview, Spending-ID-bound review state, retained raw-field matching, and replay-safe Approve dispatch.
- Added side-effect-free update validation before token reservation and routed updates through the root-supervised mutation executor.
- Preserved atomic scalar/Payer/Share replacement, archived-role retention rules, canonical Decimal validation, native form fallback, and Transactions focus redirects.
- Added regression coverage for session review identity binding and retained all existing application, infrastructure, web, runtime, and architecture tests.
- Validation passed: workspace check, offline locked Clippy with warnings denied, full locked workspace tests, architecture fitness, formatting, and independent password-helper formatting/Clippy/tests.
- No migrations or checked SQL changed; no SQLx metadata refresh was required.
- Resolved all 9 adversarial review findings; story is complete.

### File List

- `_bmad-output/implementation-artifacts/3-4-correct-an-existing-spending.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `debtor-application/src/spendings.rs`
- `debtor-application/src/errors.rs`
- `debtor-web/src/handlers.rs`
- `debtor-web/src/handlers/groups.rs`
- `debtor-web/src/handlers/spending_views.rs`
- `debtor-web/src/handlers/spendings.rs`
- `debtor-web/src/handlers/test_support.rs`
- `debtor-web/src/handlers/response.rs`
- `debtor-web/src/router.rs`
- `debtor-web/src/session.rs`
- `debtor-web/src/templates.rs`
- `debtor-web/templates/spending_form.html`
- `debtor-web/templates/transactions.html`
- `src/composition.rs`

### Change Log

- 2026-08-17: Implemented supervised existing-Spending correction, Exact-only edit Preview/Approve, Spending-ID-bound review state, archived-role-aware form projection, and canonical Transactions focus; status moved to review.
- 2026-08-18: Resolved 9 code-review findings covering Exact edit form binding, token/review ordering, recovery targets, canonical post-correction detail, archived raw-input retention, visible archive labels, HTMX synchronization, unknown mutation outcomes, and regression coverage; status moved to done.
