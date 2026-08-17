---
story_key: 3-2-record-a-spending-with-exact-shares
story_id: 3.2
epic: 3
status: done
baseline_commit: 7be36b17358de12e4263f80f9b69dbe5b39692ad
created: 2026-08-17
completion_note: Ultimate context engine analysis completed - comprehensive developer guide created
---

# Story 3.2: Record a Spending with Exact Shares

Status: done

## Story

As the administrator,
I want to preview, edit, and record exact Participant Shares,
so that I can complete a precisely allocated shared transaction.

## Acceptance Criteria

1. When Exact mode is selected for a new Spending with a valid Total and active Group Participants, every active Participant is initially selected. Total minor units are divided equally and residual units are assigned one at a time in ascending Participant ID order. The candidate conserves the Total exactly; if any initialized Share would be zero, it remains visibly invalid and no aggregate can be accepted.
2. Deselecting a Participant or editing an exact Share recalculates the allocation table. Each selected Participant's exact amount is shown, with a visible `Remaining: [amount]` or `Excess: [amount]` difference until selected Shares equal the Total. Payer selection remains exactly one independent control in the same table.
3. Preview rejects duplicate, deselected/empty, zero, negative, excess-precision, over-limit, cross-Group, inactive, archived, or otherwise invalid exact Share input with `422 Unprocessable Entity`, retained safely decoded values, row-specific errors where applicable, and no aggregate or submission-token consumption.
4. Exact Shares are parsed with checked Rust `Decimal` rules. Values are not rounded, converted with floating point, calculated in SQL, substituted with zero, or accepted when their Source Currency minor-unit sum differs from the Total. Results are deterministic regardless of submitted field order.
5. A valid exact allocation has unique positive precision-valid Shares whose minor-unit sum equals Total, and one active Group-owned Payer whose amount equals Total. Native and enhanced Preview use the same application/domain operation and produce the same ordered aggregate.
6. Switching between Proportional and Exact before commit uses only the submitted active mode. Stale or hidden fields from the other mode are not silently interpreted; malformed mode-specific fields are rejected by the strict known-field contract. Allocation mode and proportional weights are not persisted.
7. Native Preview stores the exact raw submitted input server-side and renders a non-editable reviewed state. Approve is available only for that reviewed input; Edit allocation restores editable controls. Approval reparses and revalidates the same raw input, and stale, changed, mismatched, invalid, or replayed review state cannot dispatch or create an aggregate. Native review focus targets the review status/heading.
8. If enhanced Exact requests overlap, latest revision wins. Superseded responses are ignored; only derived amount cells, status, and approval state may swap. Focus, caret, selection, keyboard, table/page scroll, and active row remain unchanged. Approve stays disabled while preview is pending, stale, invalid, or superseded. The native full-page path remains complete without HTMX.
9. Approval reuses Story 3.1's reviewed-input and complete aggregate create path. It uses the existing shared submission-token reservation ordering, root-owned mutation executor, SQLite write gate, mutation epoch, definitive commit/rollback result, and atomic Spending/Payer/Shares persistence. No second Exact persistence or mutation path is introduced.
10. Active Group and Participant ownership/lifecycle checks remain enforced in application policy and rechecked transactionally by infrastructure. Archived Groups reject form, Preview, and mutation requests with pre-use-case `409 Conflict`; invalid ownership/lifecycle input is sanitized and does not invoke a state-changing use case.
11. The allocation table remains semantic and labeled inside a keyboard-focusable internal horizontal scroll region. At 320 CSS pixels and 400% zoom, including long names and maximum OMR Total, it retains the 520px intrinsic width and `116/76/76/92/160` columns, sticky Participant identity, programmatic selection/Payer state, 48px controls, and no page-level horizontal scroll.
12. Exact status text is programmatically associated with both the allocation region and Approve, is one polite atomic status with `aria-busy` on its owning region, and never relies on color alone. Validation errors use stable labels/guidance/IDs and attach row errors only to the affected exact inputs.

Requirements: `SPEC-FR49..SPEC-FR59`, `SPEC-FR62..SPEC-FR64`; `SPEC-NFR5..SPEC-NFR10`, `SPEC-NFR14..SPEC-NFR16`, `SPEC-NFR22`, `SPEC-NFR25`, `SPEC-NFR28..SPEC-NFR34`; UX contracts `UX-TARGET-01`, `UX-ALLOC-01`, `UX-FOCUS-01`, `UX-STATUS-01`, `UX-PREVIEW-NATIVE-01`, `UX-PREVIEW-LATEST-01`, `UX-RESPONSIVE-01`, `UX-VISUAL-01`.

## Scope Boundary

- Complete Exact create-mode projection, selection, equal initialization, exact amount editing, difference display, native Preview/review/Approve behavior, and focused regression coverage.
- Reuse Story 3.1's `SpendingInput`, `ShareInput::Exact`, preview/create parity, reviewed session binding, approval lock, `SpendingMutationExecutor`, transactional repository, and runtime lifecycle.
- Do not create a second repository, migration, mutation executor, write gate, epoch, review store, or financial allocation algorithm.
- Do not implement Spending history pagination, correction redesign, deletion redesign, monthly summaries, exchange rates, debts, settlements, or Participant archive/restore.
- Do not reintroduce Multiple Payers, Equal mode, client-side allocation authority, inline/modal Spending entry, custom JavaScript, custom HTMX extensions, inline scripts/attributes, stale-edit conflicts, or compatibility shims.
- Existing persistence tables are sufficient. If SQL or migrations genuinely change, update `specs/design.md` first when required, refresh `.sqlx`, and run the temporary-database/online SQLx preparation checks.

## Tasks / Subtasks

- [x] Complete the application/domain Exact policy seam (AC: 1, 3-6)
  - [x] Reuse `debtor_domain::expenses::splitting::equal_split` for deterministic equal initialization; do not copy minor-unit arithmetic into web code.
  - [x] Ensure Exact raw inputs are parsed through `SpendingService::parse_input`/`ShareInput::Exact`, then validated by `Spending::validate` for positivity, precision, bounds, uniqueness, and exact minor-unit conservation.
  - [x] Ensure selected exact IDs are explicit and deterministic; empty exact values mean deselection only when that is the established form contract, and an empty selection remains invalid.
  - [x] Add/retain application tests for unsorted input, under/over allocation, zero/negative/excess precision/over-limit amounts, duplicate IDs, inactive/cross-Group IDs, and Preview/create vector parity.

- [x] Complete the server-rendered Exact form projection (AC: 1-3, 6-8, 11-12)
  - [x] Update `ExpenseFormView` and `expense_view`/`apply_submitted_expense` so new Exact state has equal exact defaults based on the submitted/known Total and active Participants, while preserving Proportional's existing defaults and edit-as-Exact behavior.
  - [x] Make mode-specific fields unambiguous. Do not let stale proportional weights or hidden exact fields influence the active mode; preserve wire-order review binding separately from sorted financial construction.
  - [x] Render Exact selection controls and exact amount inputs in the existing allocation table with explicit labels, header associations, retained raw values, disabled/hidden reviewed replacements exactly once, and no client-side calculation authority.
  - [x] Calculate/render Remaining or Excess as a render projection using exact `Decimal`/minor-unit rules or the shared application/domain result; never use JavaScript, floating point, or SQL aggregation.
  - [x] Preserve stable heading/status IDs, status ownership, `aria-describedby`, `aria-invalid`, error-summary behavior, focus matrix, 48px targets, table geometry, sticky identity, safe-area action bar, and native fallback.

- [x] Preserve and verify the existing route/admission/review flow (AC: 7-10)
  - [x] Reuse `GET /groups/{id}/spendings/new`, `POST /groups/{id}/spendings/preview`, and the existing create action unless a route change is necessary for native mode switching; any new route must remain a valid full-page path.
  - [x] Keep shared `CsrfValidatedForm` structural rejection before route parsing/use-case work, validation before token reservation, and reservation immediately before dispatch.
  - [x] Keep `set_spending_preview` and `take_matching_spending_preview` exact-field binding plus the serialized approval lock. Never weaken matching by normalizing, sorting, or dropping submitted fields.
  - [x] Keep archived Group rejection before use-case invocation and preserve authenticated session/token behavior.

- [x] Add invariant-owning tests (AC: all)
  - [x] Domain tests cover equal residual assignment by ascending Participant ID, JPY/KRW zero-minor-unit boundaries, OMR three-decimal precision, insufficient minor units/zero Shares, duplicate/invalid IDs, exact conservation, and deterministic repeated runs.
  - [x] Application tests cover exact parsing, field-order independence, selection/eligibility, under/over closure, invalid values before repository access, and Preview/commit parity with fakes.
  - [x] Web/router tests cover exact defaults, mode-specific field handling, explicit deselection, retained exact drafts, Remaining/Excess semantics and associations, row errors, native review immutability, stale review rejection, token/CSRF/unknown/duplicate/oversized input rejection, archived Group no-dispatch, and native fallback.
  - [x] Add or retain infra tests only where the shared aggregate path needs exact-row proof: atomic create/reload, canonical `TEXT` amounts, rollback/no partial aggregate, lifecycle rechecks, and snapshot loading. Do not duplicate Story 3.1 persistence coverage unnecessarily.
  - [x] Preserve the root real-socket smoke test and existing authentication, CSRF, submission-token, Group shell, readiness, shutdown, SQLx, and architecture regressions.

### Review Findings

- [x] [Review][Patch] Exact default Preview cannot be approved because generated `exact_*` fields are rendered but the original, pre-generation ordered fields are stored for review binding [debtor-web/src/handlers/spendings.rs:62-129]
- [x] [Review][Patch] Exact allocation status sums user-controlled Decimal values with unchecked arithmetic, allowing an overflow panic during validation rendering [debtor-web/src/handlers/spending_views.rs:451-468]
- [x] [Review][Patch] Inactive-mode `weight_*` and `exact_*` fields are accepted and silently ignored instead of being rejected by the active-mode strict form contract [debtor-web/src/forms.rs:328-332; debtor-web/src/handlers/spendings.rs:490-493]
- [x] [Review][Patch] Allocation status is not fully associated with the allocation region and Approve, and the page exposes two independent polite status nodes instead of the governed allocation status contract [debtor-web/templates/spending_form.html:31-32,79-96,102]
- [x] [Review][Patch] Selected Exact rows with blank or malformed amounts receive only a generic form message; no row-specific `aria-invalid`, guidance, or stable error association is rendered [debtor-web/src/handlers/spendings.rs:516-523; debtor-web/templates/spending_form.html:84-90]
- [x] [Review][Patch] Equal-split initialization failure for totals with fewer minor units than selected Participants drops the candidate amounts and renders blank rows rather than visibly retaining invalid zero-share candidates [debtor-web/src/handlers/spending_views.rs:486-505]
- [x] [Review][Patch] Required web/router coverage for Exact default Preview-to-Approve binding, mode switching, retained invalid drafts, row errors, and accessibility associations is absent [debtor-web/src/handlers/spendings.rs:576-610; debtor-web/src/handlers/spending_views.rs:521-566]

## Dev Notes

### Developer Context

Story 3.1 is the immediately preceding Story in Epic 3 and intentionally built the shared create path while exposing only Proportional behavior as authoritative. Exact support is now the next consumer of that seam. The current code is partially Exact-capable at the domain/application/persistence layers, but the web projection is incomplete: new forms default to Proportional, Exact inputs are not reliably usable after a radio change without a server round trip, no Exact Remaining/Excess projection is rendered, and the template has parallel `share_rows`/`exact_rows` state that can diverge.

The developer must inspect current code before editing. The task is not to redesign the ledger. Extend the existing Spending flow and remove/avoid superseded legacy behavior. The current `equal_split` function already implements the required Exact initialization and should remain the single source of equal-allocation arithmetic.

### Current Files To Update And Preserve

| Path | Current state | Required change / preservation |
|---|---|---|
| `debtor-domain/src/expenses/splitting.rs` | Contains checked deterministic `equal_split` and `proportional_split`. | Reuse `equal_split`; add tests or only minimal domain changes if a real Exact invariant is missing. Preserve checked arithmetic and ID ordering. |
| `debtor-domain/src/expenses.rs` | Persisted inputs infer `ShareMode::Exact`; legacy modes were removed/restricted by Story 3.1. | Keep edits/loaded Spendings Exact and do not add persisted mode/weight state. |
| `debtor-domain/src/model.rs` | `Spending::validate` enforces positive bounded exact-conserving allocations and one payer. | Preserve exact validation and safe errors; do not move policy into templates. |
| `debtor-application/src/spendings.rs` | `ShareInput::Exact(Vec<(EntityId, String)>)`, exact raw parsing, Preview/create parity, and eligibility checks already exist. | Extend only if Exact selection/mode semantics require a narrow transport-neutral change. Keep all parsing and financial policy here, not in web. |
| `debtor-infra/src/db/repos/spendings.rs` | One transactional aggregate loader/writer with canonical decimal persistence and lifecycle rechecks. | Reuse unchanged unless a focused test exposes a real defect. No second Exact write path or SQL monetary arithmetic. |
| `debtor-web/src/forms.rs` | Strict parser accepts scalar fields and `included_`, `weight_`, `exact_` dynamic fields, retaining order before route parsing. | Preserve duplicate/unknown/malformed rejection. Enforce active-mode semantics after structural parsing rather than making the parser construct allocations. |
| `debtor-web/src/handlers/spendings.rs` | Parses active mode into `ShareInput`, stores exact ordered review fields, requires matching review on create, and dispatches through shared executor. | Preserve exact raw review matching, token ordering, approval lock, archived precheck, and sorted financial IDs. Fix only Exact mode/selection behavior. |
| `debtor-web/src/handlers/spending_views.rs` | Builds the form projection; new forms use proportional weights; edits load stored Exact amounts; `exact_rows` is not fully rendered. | Make Exact projection authoritative and calculate a safe difference/status projection. Avoid duplicate row models or stale legacy payer restoration. |
| `debtor-web/src/templates.rs` | `ExpenseFormView` contains payer, share, and exact row projections. | Add only render-state fields needed for exact difference/errors/status; use `*View`/`*Row` conventions. Keep Askama types free of financial policy. |
| `debtor-web/templates/spending_form.html` | Native full-page form, reviewed immutable controls, Proportional/Exact radios, fixed allocation table, no Exact difference status. | Add usable Exact selection/amount rendering and accessible Remaining/Excess state without custom JS or full-form HTMX authority. Keep hidden replacements exactly once in reviewed state. |
| `static/css/app.css` | Fixed 520px table and responsive Spending action bar/field layout, plus obsolete legacy rules. | Preserve `520px` and `116/76/76/92/160` geometry, internal-only overflow, focus, 48px controls, safe-area behavior, and Editorial Contrast. Remove only superseded rules that conflict. |
| `debtor-web/src/router.rs`, `debtor-web/src/handlers/test_support.rs` | Existing focused Spending routes and test state/composition. | Reuse route inventory and fakes; update tests for Exact rather than adding parallel routes. |
| `src/composition.rs` | Root owns the shared mutation executor and lifecycle wiring. | No second executor/gate/epoch/review store. Change only if a compile-time seam is genuinely required. |
| `migrations/*spending*`, `.sqlx/*` | Existing Spending/Payer/Share schema and checked metadata. | Prefer no change. If changed, refresh metadata and validate a temporary migrated SQLite database. |

### Architecture Compliance

- Preserve `debtor -> debtor-web/debtor-infra -> debtor-application -> debtor-domain`. Domain remains synchronous/pure; application owns raw input parsing, mode/selection policy, use cases, and ports; infra owns SQLx and transaction enforcement; web owns decoding, CSRF/session mechanics, rendering, and safe HTTP mapping; root owns composition/lifecycle.
- Apply AD-3: exact `rust_decimal::Decimal`, canonical SQLite `TEXT`, source-currency minor-unit equality, checked arithmetic, no floats, no silent rounding, no SQL monetary aggregates, and no zero substitution.
- Apply AD-4/AD-5: Participants are Group-owned accounting identities. New exact allocations require active, owned, non-archived Participants. Payer and Share roles are independent. Do not introduce a participant through a hidden/stale mode field.
- Apply AD-6: reuse one `SqliteLedgerRuntime`, five-second write gate, mutation epoch, root mutation registry, and last-committed-valid-write-wins behavior.
- Apply AD-10: strict bounded form, authentication, exactly one CSRF, strict fields, Group prechecks, token reservation immediately before dispatch, then exactly one mutation dispatch. Validation before reservation preserves the token; post-dispatch generic timeout must not cancel or obscure the result.
- Apply AD-11/AD-18: native semantic Askama HTML is authoritative. Pinned HTMX `2.0.10` and official response-targets `2.0.4` are optional enhancement only; no custom JavaScript, inline scripts/attributes, custom extensions, full-form swaps, or client-side financial authority.
- Apply AD-15: validation maps to `422`, lifecycle/token conflicts to `409`, retryable runtime/storage conditions to safe bounded responses, and no raw SQL, identifiers, values, tokens, provider diagnostics, or request-derived secrets in responses/logs.

### Library / Framework Requirements

- Keep the pinned toolchain and dependencies: Rust `1.97.1` edition 2024, Axum `0.8.9`, Askama `0.16.0`, Tokio `1.53.1`, SQLx/sqlx-cli `0.9.0`, `rust_decimal 1.42.1`, HTMX `2.0.10`, and response-targets `2.0.4`. Do not add a form-state or allocation dependency.
- Use Askama render projections and existing Axum form/router patterns. Do not place Axum, Askama, SQLx, session, or cryptography types in application-owned ports.
- SQLx guidance consulted 2026-08-17 via Context7 `/websites/rs_sqlx`: execute all transaction statements through the mutable transaction, check results, and call `commit()` only after all aggregate writes and authoritative checks succeed. Continue using checked `query!`/`query_as!` macros and committed offline metadata.

### Testing Requirements

- Domain tests own allocation arithmetic and exactness. Cover examples, boundaries, deterministic order, checked failures, and conservation; no float/property shortcut.
- Application tests use injected fakes without Axum, SQLite, network, or wall clock. Assert invalid Exact input does not reach the repository.
- Web tests verify native/enhanced parity, `422` retained values, `409` token/archived rejection, no dispatch for pre-use-case rejection, focus/status associations, and exact mode-switch behavior.
- Use temporary SQLite/`#[sqlx::test]` only for repository contracts, corruption, transaction rollback, FK/lifecycle checks, and concurrency. Coordinate races with barriers/notifications/held locks, never sleeps.
- Required validation: `cargo fmt --all -- --check`; `cargo check --workspace --all-features --locked`; `SQLX_OFFLINE=true cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`; `cargo test --workspace --all-features --locked`; `cargo run --bin architecture-check --locked`.
- If checked SQL or migrations change: migrate a temporary database and run `SQLX_OFFLINE=false DATABASE_URL=sqlite:///tmp/debtor-sqlx.db?mode=rwc cargo sqlx prepare --workspace --check`; refresh committed `.sqlx`. Never use `cargo build --release`.

### Previous Story Intelligence

Story 3.1 established these patterns and review corrections that Exact must preserve:

- `SpendingService::preview_input` and `create_input` share the same parse/validate operation; create is dispatched through `SpendingMutationExecutor` rather than directly from a handler.
- Preview stores raw submitted fields in the session. Approval reserves the submission token before consuming matching review state, and a process-local approval lock serializes the sequence. Do not sort/normalize the review binding.
- Reviewed controls are disabled/readonly with hidden authoritative values retained exactly once. A reviewed form must not remain editable or approvable after its input changes.
- Included selection is authoritative in Proportional mode; selected rows require a non-empty weight, unchecked weights are ignored, and unknown Included-only rows fail validation. Exact mode needs equally explicit selection semantics.
- The existing flow has no Exact field-change endpoint and no custom JavaScript. If mode switching needs a request, use an approved native/HTMX path with full-page fallback; never pretend a browser radio alone changes server-rendered fields.
- Review fixes emphasized no false post-commit failure, no duplicate focus targets, no client-controlled focus, strict unknown-field rejection, no-dispatch coverage, and accurate test claims. Apply the same standard.

### Git Intelligence

- HEAD `7be36b1` implements Story 3.1 after `6aa5e63` (Story 2.5), `987f420` (Story 2.4), and `6d66e7c` (Story 2.3). These changes consistently extend application services/ports, infra repositories/tests, web handlers/templates/router/test support, root composition, CSS, and `.sqlx` only when checked SQL changes.
- Story 3.1's packet explicitly requires Story 3.2 to reuse the create aggregate path. The current worktree was clean at analysis time; inspect current files rather than trusting historical story prose over repository reality.

### UX Guardrails

- `UX-ALLOC-01`: label the allocation region as “Participant allocation table; scroll horizontally for Payer, Included, and Weight/Share”; preserve semantic headers, sticky identity, long-name wrapping, exact column geometry, and internal-only scrolling.
- `UX-STATUS-01`: one stable `role="status"`, `aria-live="polite"`, `aria-atomic="true"` status owns Remaining/Excess/pending/error; owning region exposes `aria-busy`; do not make every amount a live region.
- `UX-PREVIEW-NATIVE-01`: native Preview is reviewed, non-editable, server-bound, and has Approve/Edit allocation; Approve is limited to the current reviewed input.
- `UX-PREVIEW-LATEST-01`: enhanced previews are latest-input-wins, derived-only swaps, and preserve focus/caret/selection/keyboard/scroll. Enhancement never changes native `action`, method, or full-page outcome.
- `UX-FOCUS-01`: native successful Preview focuses the review status/heading; validation focuses the linked error summary or sole invalid control; enhanced Preview retains the active control.
- `UX-TARGET-01`, `UX-RESPONSIVE-01`, `UX-VISUAL-01`: all controls are at least 48px at 320px/400% zoom; the focused form is one `100dvh` document scroll owner with safe-area action bar; use dark Editorial Contrast, ruled sections, square geometry, no cards/gradients/transitions/hover lift.

### Project Structure Notes

- Feature modules remain plural (`spendings`); use `*Input`, `*View`, `*Row`, `*Template`, `*Repository`, `*Service`, and `*Store` naming.
- Ledger IDs remain positive `i64`; UUIDs remain limited to session/security randomness.
- Existing `equal_split` currently sorts IDs and assigns residual units ascending. Preserve that behavior and its checked error mapping.
- `equal_split` rejects totals with fewer minor units than recipients because zero Shares are invalid. The UI must show that invalid state rather than silently deselecting or substituting an amount.
- Do not add a schema or migration to represent transient mode/weights. Stored edits reopen Exact from persisted Payer/Share amounts.

### References

- [Source: `_bmad-output/planning-artifacts/epics.md#Story 3.2: Record a Spending with Exact Shares`]
- [Source: `_bmad-output/planning-artifacts/epics.md#Assignment Packets`]
- [Source: `_bmad-output/planning-artifacts/epics.md#Final-Evidence Ledger`]
- [Source: `_bmad-output/planning-artifacts/prds/prd-debtor-2026-08-10/prd.md#FR-4: Record a Spending`]
- [Source: `_bmad-output/planning-artifacts/prds/prd-debtor-2026-08-10/prd.md#FR-5: Exact allocation`]
- [Source: `_bmad-output/planning-artifacts/prds/prd-debtor-2026-08-10/addendum.md#HTTP Forms, Statuses, And Dispatch`]
- [Source: `specs/design.md#Accounting And History`]
- [Source: `specs/design.md#Architecture`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-3 - Exact monetary truth`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-5 - Application policy with transactional enforcement`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-6 - Single ledger runtime and mutation epoch`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-10 - Shared unsafe-request admission boundary`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-11 - Native HTML authority`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-18 - Governed UX contracts and traceability`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md#Spending and History`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md#Financial Allocation`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md#Interaction Focus Matrix`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/DESIGN.md#Layout & Spacing`]
- [Source: `_bmad-output/implementation-artifacts/3-1-record-a-proportional-spending.md#Previous Story Intelligence`]
- [Source: `_bmad-output/project-context.md#Critical Implementation Rules`]
- [Source: `debtor-domain/src/expenses/splitting.rs`]
- [Source: `debtor-application/src/spendings.rs`]
- [Source: `debtor-web/src/forms.rs`]
- [Source: `debtor-web/src/handlers/spendings.rs`]
- [Source: `debtor-web/src/handlers/spending_views.rs`]
- [Source: `debtor-web/src/templates.rs`]
- [Source: `debtor-web/templates/spending_form.html`]
- [Source: `static/css/app.css`]
- [Source: `debtor-infra/src/db/repos/spendings.rs`]
- [Source: SQLx transaction/query macro documentation via Context7, `/websites/rs_sqlx`]

## Dev Agent Record

### Agent Model Used

openai/gpt-5.6-luna

### Debug Log References

- Resolved create-story customization: no activation prepend/append steps; loaded persistent fact `_bmad-output/project-context.md`.
- Read the complete ordered sprint status and selected first backlog story: `3-2-record-a-spending-with-exact-shares`.
- Epic 3 transitioned from `backlog` to `in-progress` because this is its first created story.
- Loaded the complete Story 3.2 contract, Epic 3 assignment boundary, normative design, relevant PRD/addendum sections, architecture spine, UX contracts, project context, Story 3.1, current Exact-related code, migrations, and recent Git history.
- Used parallel repository exploration to identify reusable Exact domain/application/infra seams and web projection gaps.
- Consulted current SQLx transaction and checked-query documentation through Context7 on 2026-08-17; pinned versions and lockfiles remain authoritative.

### Implementation Plan

- Reuse the existing domain `equal_split`, application `ShareInput::Exact`, reviewed-input session binding, approval lock, root mutation executor, and transactional repository.
- Sort Exact allocations by Participant ID in application parsing so persisted aggregate order is independent of form field order.
- Extend the focused Spending projection with explicit Exact selection, mode-specific inputs, equal initialization during Exact Preview, closure status, and native CSS mode presentation without custom JavaScript.
- Keep the existing routes and admission pipeline unchanged; add only focused application and web regression tests. No schema, migration, dependency, or SQLx metadata changes were needed.

### Completion Notes List

- Ultimate context engine analysis completed - comprehensive developer guide created.
- Story is deliberately scoped to Exact form/projection and shared-path reuse; persistence redesign is excluded unless repository evidence proves a defect.
- No user clarification is required to begin implementation.
- Targeted validation passed: `cargo fmt --all`; `cargo test -p debtor-domain -p debtor-application -p debtor-web`.
- Full validation passed: `cargo fmt --all -- --check`; `cargo check --workspace --all-features --locked`; `SQLX_OFFLINE=true cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`; `cargo test --workspace --all-features --locked`; `cargo run --bin architecture-check --locked`.
- Review fixes applied: generated Exact defaults now bind to the reviewed wire payload; status arithmetic is checked; inactive mode values are rejected when non-empty; the allocation status is a single associated polite status; row-level Exact errors are projected; insufficient-minor-unit defaults retain visible zero candidates; and focused review regression tests were added.
- Code review completed: 7 patch findings resolved, with no deferred or dismissed findings.

### File List

- `_bmad-output/implementation-artifacts/3-2-record-a-spending-with-exact-shares.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `debtor-application/src/spendings.rs`
- `debtor-web/src/forms.rs`
- `debtor-web/src/handlers/spendings.rs`
- `debtor-web/src/handlers/spending_views.rs`
- `debtor-web/src/templates.rs`
- `debtor-web/templates/spending_form.html`
- `static/css/app.css`

### Change Log

- 2026-08-17: Implemented Exact Share selection, deterministic ordering, equal initialization on Exact Preview, Remaining/Excess status, native mode presentation, and focused regression coverage; all required validation gates passed and status moved to review.
- 2026-08-17: Addressed code review findings - 7 items resolved; final status set to done.
