---
story_key: 3-5-delete-a-spending-atomically
story_id: 3.5
epic: 3
status: done
baseline_commit: 02d53b48dd5aac297306ae17e4506d43c1eb4d1b
created: 2026-08-18
completion_note: Ultimate context engine analysis completed - comprehensive developer guide created
---

# Story 3.5: Delete a Spending Atomically

Status: done

## Story

As the administrator,
I want to delete an incorrect Spending as one complete aggregate,
so that no orphaned Payer or Share data remains in the ledger.

## Acceptance Criteria

1. **Complete confirmation scope:** From an expanded active-Group Transaction row, Delete direct-loads one complete aggregate and renders a full server confirmation naming the Spending, exact Source Currency Total, Payer, Shares, date, category, description, irreversible effect, and one-shot protection.
2. **Allow-listed return state:** Confirmation state carries only an allow-listed canonical Transactions return URL plus stable invoking Delete-control/disclosure ID. It must not accept arbitrary URLs, DOM selectors, cursor text, or untrusted focus targets.
3. **Cancel behavior:** Cancel performs no mutation, returns to the canonical Transactions page/disclosure state, and restores focus to the invoking Delete control in the expanded row. Invalid or missing return state falls back to a safe canonical Transactions destination.
4. **Protected pending dispatch:** Confirm can be activated once only. The initiator becomes unavailable, repeated activation is suppressed, one scoped status with `aria-busy` reports pending, the submission token is reserved immediately before dispatch and remains terminal after dispatch, and the shared write-gated mutation reaches a definitive commit or rollback result.
5. **Atomic deletion:** A valid delete removes the Spending row and its Payer/Share allocations as one complete transaction. Any storage constraint, eligibility, or transaction failure leaves the complete aggregate unchanged or reports an authoritative failure; no partial allocation deletion is visible.
6. **Sanitized failures:** Raw SQLx diagnostics, database messages, monetary values, entity IDs, request data, and adapter errors never reach HTTP or logs. Map safe application reasons to bounded responses and preserve definitive mutation outcome semantics.
7. **Canonical post-delete focus:** After commit, return with `303 See Other` to Transactions on the same page when still valid. Focus the next row summary, otherwise the previous row summary, otherwise the Transactions heading when the page is empty or the page boundary changed. Do not redirect unconditionally to Group Summary, show an out-of-range page, leave an orphaned disclosure target, or add a completion badge.
8. **Concurrency:** Concurrent delete attempts, or an edit/delete race for the same Spending, serialize through the existing mutation registry/write gate/SQLite transaction. At most one delete commits; later work observes committed state or a safe not-found/conflict outcome. No automatic retry or optimistic stale-edit conflict API is introduced.
9. **Archived boundary:** Archived Groups remain readable, but direct delete confirmation and mutation routes return `409 Conflict` before invoking any use case, and Transactions exposes no Delete control. Read-only detail/history behavior remains unchanged.
10. **History-free Group eligibility:** Deleting the last Spending leaves no payer/share rows and makes the owning Group eligible for the existing history-free Group deletion flow. Referenced Group/Participant history remains protected while the Spending exists.
11. **Responsive accessible confirmation:** At 320 CSS pixels and 400% zoom, long details and actions wrap without clipping or page-level horizontal scroll. Confirm and Cancel remain at least 48 by 48 CSS pixels, focus remains visible, and coral destructive treatment is paired with explicit text.
12. **Native/enhanced parity:** Native server-rendered HTML is complete and authoritative. Optional pinned HTMX enhancement preserves the same action/href semantics, status codes, return state, focus contract, and failure recovery; no custom JavaScript, inline script, modal, browser-only confirmation, or duplicate destructive route is added.

Requirements: `SPEC-FR43`, `SPEC-FR62..SPEC-FR66`, `SPEC-FR87..SPEC-FR90`; `SPEC-NFR3`, `SPEC-NFR14..SPEC-NFR16`, `SPEC-NFR22`, `SPEC-NFR25`, `SPEC-NFR31..SPEC-NFR34`; UX contracts `UX-TARGET-01`, `UX-FOCUS-01`, `UX-STATUS-01`, `UX-CONFIRM-01`, `UX-RESPONSIVE-01`, `UX-VISUAL-01`.

## Scope Boundary

- Implement the existing Spending Delete confirmation and mutation vertical slice, including complete detail projection, allow-listed return/focus state, supervised dispatch, atomic persistence, canonical Transactions response, and invariant-owning tests.
- Reuse Stories 3.1-3.4: `SpendingReader`/`SpendingUseCases`, direct complete aggregate loading, existing `delete_spending` repository port, shared strict CSRF/form/submission-token pipeline, `SpendingMutationExecutor`/root mutation registry, SQLite write gate and mutation epoch, Transactions keyset projection, stable row/disclosure IDs, and existing `ConfirmTemplate` only where its contract can be extended safely.
- Do not redesign Spending creation, Exact allocation, edit/review behavior, history pagination, summaries, rates, debts, settlements, Participant archival/restore, or Group lifecycle beyond proving that deleting the last Spending restores history-free Group eligibility.
- Do not add a second repository, reader, mutation executor, write gate, epoch, token/review store, confirmation protocol, financial algorithm, schema compatibility shim, optimistic revision, stale-edit conflict, custom JavaScript, modal, inline/browser-only confirmation, or unconditional page-one/Group redirect.
- Prefer no migration. Existing `spending_payers` and `spending_shares` foreign keys use `ON DELETE CASCADE` from Spending while Participant and Group references remain restrictive. If SQL/migrations genuinely change, update `specs/design.md` first, refresh `.sqlx`, migrate a temporary SQLite database, and run online SQLx preparation.

## Tasks / Subtasks

- [x] Audit the accepted 3.1-3.4 seams before editing (AC: all)
  - [x] Read current delete handlers, `ConfirmTemplate`, Transactions template/projection, strict form extractor, session state, root executor, repository, migrations, and tests; preserve unrelated create/edit/history behavior.
  - [x] Confirm current partial path is replaced rather than duplicated: `delete_spending_form` currently loads only `Spending`, uses generic copy, and loses Payer/Shares/return state; `delete_spending` currently dispatches directly through `state.spendings.delete` and redirects to Group Summary.
- [x] Define server-owned confirmation/return context (AC: 1-4, 7, 9, 11)
  - [x] Extend the web-owned session/confirmation state or an existing route-neutral mechanism with the Group ID, Spending ID, allow-listed Transactions URL/page cursor, disclosure ID, invoking Delete-control ID, and a bounded protected confirmation binding. Never persist arbitrary URLs/selectors or echo untrusted IDs in diagnostics.
  - [x] Ensure the confirmation GET direct-loads `SpendingDetail` from one snapshot, includes current names and visible Archived labels where applicable, and rejects archived Group before rendering mutation controls.
  - [x] Render named object facts, exact symbol plus ISO currency amount, Payer, ordered Shares, category/date/description, irreversible wording, and one-shot protected Confirm/Cancel actions.
  - [x] Make Cancel clear/expire confirmation state safely and return to the encoded canonical Transactions state with the invoking delete control focused; retain a safe fallback when state is absent or invalid.
- [x] Route delete through shared unsafe admission and supervised mutation dispatch (AC: 4, 6, 8, 9)
  - [x] Preserve strict field/CSRF/unknown/duplicate validation and validate route/group/confirmation binding before submission-token reservation where possible.
  - [x] Mark dispatch immediately before the first state-changing use-case call; use the existing `SpendingMutationExecutor`/root registry and definitive outcome publication, not a direct `SpendingUseCases::delete` call from the handler.
  - [x] Reserve the submission token atomically immediately before dispatch; invalid/replayed/consumed tokens return `409` with no use-case invocation, while validation before dispatch does not consume the token.
  - [x] Preserve post-dispatch no-generic-timeout semantics, write-gate contention behavior, mutation epoch advancement only after commit, and Unknown/fatal readiness handling already established by Story 3.4.
- [x] Strengthen atomic repository deletion (AC: 5, 8, 10)
  - [x] Reuse `SpendingRepository::delete_spending` and the existing gate/transaction seam. Recheck Group ownership and active Group state inside the authoritative delete operation.
  - [x] Delete only the parent Spending row through checked SQLx so the existing payer/share `ON DELETE CASCADE` removes both allocation sets atomically. Do not issue independent allocation deletes or SQL monetary operations.
  - [x] Map zero affected rows to the existing safe Group-scoped not-found/conflict behavior; rollback all state on constraint/transaction failure and keep the complete aggregate readable.
  - [x] Verify delete of the final Spending leaves payer/share counts at zero and permits the existing history-free Group confirmation/delete path without weakening restrictive Group/Participant history rules.
- [x] Implement canonical Transactions return and focus (AC: 2, 3, 7, 11, 12)
  - [x] Extend `TransactionsTemplate`/view state only as needed for stable focus and one scoped status. Keep `spending-{id}-summary`, Transactions heading, region, and status IDs stable.
  - [x] Derive the destination after authoritative commit from a bounded server-owned page/disclosure context, selecting the stored next row, previous row, or heading deterministically at page boundaries.
  - [x] Preserve current rows during enhanced pending/error states, use one polite atomic status and owning-region `aria-busy`, and keep native full-page navigation complete without HTMX.
  - [x] Preserve Editorial Contrast, square controls, explicit destructive text, 48px targets, visible focus, and no page-level horizontal scroll.
- [x] Add invariant-owning tests (AC: all)
  - [x] Application tests cover Group/Spending scope, safe delete forwarding, no delete use-case on archived/pre-dispatch rejection, and definitive error propagation using fakes.
  - [x] Infrastructure tests cover complete aggregate deletion, payer/share cascade, archived Group rejection, concurrent delete races, and last-Spending Group eligibility.
  - [x] Web/router tests cover complete confirmation facts/current identity projections, allow-listed Cancel and focus binding, token replay/admission, canonical `303` destination, and archived action suppression.
  - [x] Migration tests retain direct SQLite cascade coverage and restrictive Group/Participant history coverage. Browser geometry/contrast remains manual because no browser harness exists.
- [x] Run validation (AC: all)
  - [x] `cargo fmt --all -- --check`
  - [x] `cargo check --workspace --all-features --locked`
  - [x] `SQLX_OFFLINE=true cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`
  - [x] `cargo test --workspace --all-features --locked`
  - [x] `cargo run --bin architecture-check --locked`
  - [x] No checked SQL/migrations changed; SQLx metadata refresh was not required.
  - [x] Independent password-helper fmt, Clippy, and locked tests.
  - [x] Never use `cargo build --release`.

### Review Findings

- [x] [Review][Patch] **HIGH: Validate the delete lifecycle form before dispatch** [debtor-web/src/handlers/spendings.rs:490-520] — AC 4 and the strict-form constraint require exactly `csrf` and `submission_token`; the handler now calls the shared lifecycle parser before confirmation lookup and token reservation.
- [x] [Review][Patch] **HIGH: Bind the confirmation to its rendered submission token** [debtor-web/src/handlers/spendings.rs:421-475, 502-520] — The typed session binding now stores the rendered token and POST rejects a different valid session token before reservation or mutation.
- [x] [Review][Patch] **HIGH: Restore focus to the invoking Delete control** [debtor-web/src/session.rs:115-175; debtor-web/templates/transactions.html:43-58] — The session stores the canonical control identity, Cancel carries a validated `focus_delete` state, and Transactions renders the matching Delete control with focus.
- [x] [Review][Patch] **HIGH: Reconcile post-delete focus with the committed page** [debtor-web/src/handlers/spendings.rs:408-419, 532-537] — Post-commit history is re-read to validate next/previous focus and resolve empty cursor pages through the newer boundary before falling back to the heading.
- [x] [Review][Patch] **MEDIUM: Treat invalid confirmation return context as safe fallback** [debtor-web/src/handlers/spendings.rs:386-392] — Delete confirmation now treats malformed cursor/focus values as absent and uses the canonical Transactions fallback rather than returning a context error.
- [x] [Review][Patch] **MEDIUM: Make the concurrent-delete regression deterministic** [debtor-infra/tests/repos.rs:463-472] — The regression now shares one runtime write gate and coordinates both contenders with a Tokio barrier before the authoritative operations.

## Dev Notes

### Developer Context

Epic 3 already has exact create, bounded Transactions history/detail, and supervised complete Spending correction. The current repository contains delete scaffolding, but it does not satisfy this story: the confirmation uses a generic message and empty detail list, binds no invoking row context, the mutation handler calls the application service directly after token reservation, and success redirects to `/groups/{group_id}`. Treat those as known gaps against the final contract, not as a reason to create a second deletion flow.

The authoritative aggregate boundary is `Spending`. A delete must remove the parent and both allocation tables as one storage operation. The database schema already encodes the intended cascade from `spendings` to `spending_payers` and `spending_shares`; the Participant foreign keys are restrictive, and Group deletion is restrictive. Do not manually delete Payer/Share rows because that creates partial-deletion risk and duplicates the schema's aggregate boundary.

The confirmation page must use `SpendingDetail` rather than the smaller `Spending` read. This is required to display the complete named scope and to resolve current Participant names after rename/archive. The Group archived state must come from the same direct snapshot and must be checked before any delete confirmation or mutation use-case dispatch.

### Current Files To Update And Preserve

| Path | Current state | Story-specific change/preservation |
|---|---|---|
| `debtor-application/src/spendings.rs` | Owns `SpendingUseCases::delete`, `SpendingRepository::delete_spending`, reader projections, and raw input policy. | Keep the narrow port and safe application errors. Add only a delete-specific confirmation/use-case contract if required; no Axum/SQLx/session types inward. Preserve exact creation/edit/read behavior. |
| `debtor-infra/src/db/repos/spendings.rs` | `delete_spending` acquires the shared write gate and executes one checked parent `DELETE` with active Group predicate; zero rows maps via `group_write_failure`. | Strengthen tests and any transactional/error mapping needed. Preserve parent-only cascade, Group ownership/archived guard, restrictive history, checked SQLx, and no monetary SQL. |
| `debtor-web/src/handlers/spendings.rs` | Delete GET loads only `Spending` and renders generic `ConfirmTemplate`; Delete POST reserves a token then calls `state.spendings.delete` directly and redirects to Summary. | Build complete confirmation, bind allow-listed return/focus state, validate before reservation, use `state.spending_mutations`, and return canonical Transactions focus state. Keep detail/edit/create routes unchanged. |
| `debtor-web/src/handlers/spending_views.rs` | Builds Transactions rows with stable IDs/focus and complete current identity projections. | Extend only the focused return/status projection needed after deletion. Do not alter keyset ordering or make templates infer page validity. |
| `debtor-web/src/session.rs` | Stores Group deletion confirmation and Spending Preview binding. | Reuse the session-owned pattern for bounded Spending delete confirmation/return state, or add the smallest dedicated typed state. Clear it on Cancel/consumption; never store arbitrary return data. |
| `debtor-web/src/forms.rs` | Shared strict CSRF and submission-token extractor; `reserve_and_dispatch` marks preflight dispatch immediately before mutation. | Preserve exact field validation/order and token semantics. Delete forms must use the shared extractor, not a route-specific security path. |
| `debtor-web/src/templates.rs` | Generic `ConfirmTemplate` has heading/message/action/cancel/details/security values; `TransactionsTemplate` and `TransactionRow` have stable row/focus fields. | Extend typed projections for complete Spending facts, invoking focus, status, and scoped busy state rather than putting policy or URL parsing in Askama. |
| `debtor-web/templates/confirm.html` | Generic confirmation has Back header, heading, message, optional list, Confirm/Cancel, and one status node, but no full aggregate facts or explicit scoped return behavior. | Make Spending delete confirmation meet `UX-CONFIRM-01`; preserve Group confirmation behavior and native fallback. Avoid a modal or browser `confirm()`. |
| `debtor-web/templates/transactions.html` | Native `<details>` rows with stable summary IDs, current names/Archived labels, Edit/Delete actions, keyset pagination, scoped status, and focus flags. | Preserve row semantics and action suppression for archived Groups; add only canonical post-delete focus/status behavior. |
| `debtor-web/src/router.rs` / `debtor-web/src/handlers.rs` | Delete GET/POST route exists at `/groups/{group_id}/spendings/{spending_id}/delete`. | Keep one canonical route pair, wire the existing handler through shared admission/supervision, and do not add duplicate destructive routes. |
| `src/composition.rs` | Root `RootGroupMutationExecutor` supervises create/update Spending and Group mutations through one registry, runtime, and epoch. | Add a narrow delete executor method if absent, reusing the same lease/guard/definitive outcome path. Unknown task outcomes must not be reported as rollback. |
| `static/css/app.css` | Accepted Editorial Contrast Transactions and confirmation-related base styles. | Add minimal delete confirmation/pending/focus/long-detail styling; preserve form/table geometry and prohibit page-level horizontal scrolling. |
| `debtor-infra/tests/repos.rs`, `debtor-infra/tests/migrations.rs`, web/router/application tests | Existing archived-delete rejection, direct cascade, and basic route/security seams exist. | Extend invariant-owning tests for complete atomic delete, races, token/order, confirmation facts, canonical return/focus, and final Group eligibility. |
| `.sqlx/*`, `migrations/*`, `specs/design.md` | Existing schema and checked metadata already support parent delete cascades and restrictive history. | Prefer no changes. If a query or migration changes, update normative design first and refresh metadata against a migrated temporary database. |

### Architecture Compliance

- Preserve `root -> debtor-web/debtor-infra -> debtor-application -> debtor-domain`. Domain remains pure; application owns use cases/ports; infra owns SQLx and transactional guards; web owns HTTP/session/CSRF/rendering/safe mapping; root owns composition and mutation lifecycle.
- Apply AD-3/AD-5: no floating point, SQL monetary aggregation, normalization of corrupted values, or client-side accounting. Delete does not recalculate or rewrite amounts; it must remove the validated aggregate as a unit.
- Apply AD-4: historical identities remain protected while referenced. Archived Groups are read-only and reject delete before use-case invocation. Group deletion remains separately restricted and history-free.
- Apply AD-6: use exactly one process-local mutation registry, mutation epoch, five-second ledger write gate, SQLite WAL/` synchronous=FULL`, and transaction. Gate timeout starts no transaction or guarded side effect; epoch advances only after commit.
- Apply AD-7: detail/delete direct-load one complete aggregate; do not load all history. Confirmation identity data must be snapshot-consistent.
- Apply AD-10/AD-14: shared strict unsafe admission, authentication, CSRF, route checks, bounded pre-dispatch deadline, token reservation immediately before dispatch, and no generic timeout after dispatch.
- Apply AD-11/AD-18: semantic Askama/native HTML is authoritative; pinned HTMX/response-targets are enhancement only. Cite and test `UX-CONFIRM-01`, `UX-FOCUS-01`, `UX-STATUS-01`, `UX-TARGET-01`, `UX-RESPONSIVE-01`, and `UX-VISUAL-01`.
- Apply AD-15: `Validation` is `422`, submission/lifecycle/archive conflicts are `409`, contention/unknown storage uses safe bounded failure mapping, and raw diagnostics, IDs, values, query strings, tokens, and session data never leave adapters.

### Library / Framework Requirements

- Keep pinned Rust `1.97.1` edition 2024, Axum `0.8.9`, Askama `0.16.0`, Tokio `1.53.1`, SQLx `0.9.0`, `rust_decimal 1.42.1`, HTMX `2.0.10`, response-targets `2.0.4`, and lockfiles. Add no dependency.
- Axum's typed `Path`/`Query` extractors belong at the web boundary. Parse only bounded route/query values there and convert them into application-owned types; do not expose Axum extractors through ports. Current official docs consulted 2026-08-18: `/tokio-rs/axum`, extractors/routing and conditional response handling.
- Askama `#[derive(Template)]` with `#[template(path = "...")]` remains the compile-time typed rendering boundary. Keep URL allow-listing, financial policy, SQL, and request parsing in Rust handlers/services, not templates. Current docs consulted 2026-08-18: `/askama-rs/askama`.
- HTMX `hx-post`/`hx-target`/`hx-target-4xx` may enhance the valid native form, but `action`, `href`, status codes, full-page responses, and recovery must remain correct without HTMX. Use the pinned official response-targets extension only; no custom events or JavaScript. Current docs consulted 2026-08-18: `/bigskysoftware/htmx`.
- SQLx mutations must use checked `query!`/`query_as!` macros. Execute through the transaction/authoritative connection where required, commit only after the complete parent delete succeeds, and refresh committed `.sqlx` metadata if SQL changes.

### Testing Requirements

- Application: use fakes and injected effects only. Assert Group/Spending scope, archived/conflict policy, safe delete forwarding, no repository call for pre-dispatch rejection, and safe error propagation.
- Infrastructure: use `#[sqlx::test]`/temporary SQLite. Assert complete parent/payer/share removal, zero orphan rows, rollback/no change on failure, restrictive Participant/Group history, archived Group rejection, missing target mapping, and final-Spending history-free Group eligibility. Use barriers, notifications, or held locks for races; never sleeps.
- Root/runtime: assert delete runs through the existing dispatched-mutation lease and publishes authoritative Committed/RolledBack/Unknown behavior consistently with create/update. Do not add a generic post-dispatch timeout.
- Web/router: assert complete confirmation facts, current Participant names and visible Archived labels, allow-listed return/focus state, Cancel no mutation, token preservation before dispatch, duplicate/replay `409` with no dispatch, archived precheck `409`, pending disabled/status/`aria-busy`, canonical `303`, page-boundary next/previous/heading focus, no completion badge, safe errors, and native/enhanced parity.
- Templates/CSS: assert semantic confirmation content, explicit irreversible wording, stable IDs, one scoped status node, 48px target classes/attributes, destructive text plus coral treatment, and no page-level horizontal-scroll rules. Do not claim 320px/400% geometry or contrast without a browser harness; manual evidence must cover long descriptions/names, maximum currency text, keyboard focus, and safe-area behavior.
- Preserve root real-socket authentication/CSRF/read/shutdown smoke coverage and architecture fitness. Place each regression in the layer owning the invariant.

### Previous Story Intelligence

From Story 3.4:

- Reuse `SpendingMutationExecutor`, root mutation registry, write gate, epoch, complete aggregate loader, strict form extractor, token store, review/session conventions, and Transactions projection. Do not call the use-case delete directly from a web handler.
- Story 3.4's review fixes established pre-dispatch validation before token reservation, unknown mutation outcome preservation, current-name/Archived rendering, stable focus IDs, allow-listed return context, and native-preserving HTMX enhancement. Apply those same standards to delete.
- Stored Spending history is complete and exact; current Participant names are projections. Confirmation must not reconstruct names from stale stored fields or omit archived identities.
- Successful edit returns to an authoritative canonical detail/Transactions context rather than a generic Group page. Delete must go further and choose next/previous/heading based on the post-commit page boundary.
- Review feedback repeatedly rejected arbitrary return targets, direct mutation bypasses, partial allocation paths, raw diagnostics, false completion, and duplicate routes. Treat these as explicit anti-patterns.

From Story 3.3:

- Preserve native `<details>` Transactions rows, fixed 25-item `(spent_date DESC, id DESC)` keyset ordering, stable `spending-{id}-summary` IDs, current/archived identity projections, scoped status, and archived read-only action suppression.
- Direct detail loads Group, Spending, Payer, Shares, and current Participant identity data from one snapshot. Reuse it for confirmation rather than separately loading a small Spending then unrelated members.

### Git Intelligence

- Recent commits are story-oriented and extend existing seams: `02d53b4 feat: implement 3-4 bmad`, `0fc4380 feat: implement 3-3 bmad`, `120c4ca feat: implement 3-2 bmad`, `7be36b1 feat: implement 3-1 bmad`.
- HEAD is the accepted Story 3.4 implementation. Its modified files establish the current supervised update, review/session, template, router, and root composition patterns. The worktree was clean during analysis; inspect current state again and do not overwrite unrelated concurrent changes.
- Story 3.3 added the complete direct-read and Transactions projection; Story 3.4 added supervised update and canonical focus. Story 3.5 should be a focused continuation, not a rewrite of either vertical slice.

### Project Structure Notes

- Feature modules remain plural (`spendings`); interfaces use `*Reader`, `*Repository`, `*UseCases`, `*MutationExecutor`; implementations use `*Service`, `*Store`, `*Gate`; transport inputs use `*Input`; Askama projections use `*Template`, `*View`, and `*Row`.
- Ledger IDs are positive `i64`; UUIDs remain limited to security/session randomness. Debtor remains permanently single-administrator; Participants are accounting identities, never users.
- Existing migrations are sufficient: `spendings.group_id` is `ON DELETE RESTRICT`; payer/share `spending_id` references are `ON DELETE CASCADE`; payer/share Participant references are `ON DELETE RESTRICT`. Parent-only deletion is the intended aggregate operation.
- Keep Summary, Manage, Add Spending, Transactions, direct detail, sign-out, authentication headers, and archived read-only behavior working end-to-end.

### UX Guardrails

- `UX-CONFIRM-01`: Spending deletion is a dedicated server-rendered confirmation page, not an inline/browser confirmation. Name object, exact scope, irreversible effect, and one-shot action; Cancel returns to the allow-listed invoker.
- `UX-FOCUS-01`: Cancel targets the invoking Delete control; successful deletion targets next/previous Transaction summary or Transactions heading. Render exactly one server-owned forward focus destination and encode only canonical page/disclosure state.
- `UX-STATUS-01`: one stable scoped `role="status"`, `aria-live="polite"`, `aria-atomic="true"` announces pending/error/commit; owning region uses `aria-busy`; do not make individual facts live regions.
- `UX-TARGET-01`: Confirm, Cancel, Back, Delete, disclosure, and pagination targets are at least 48 by 48 CSS pixels at 320px/400% zoom with a visible two-pixel focus indicator.
- `UX-RESPONSIVE-01`: confirmation and Transactions have one document scroll owner, safe-area-aware normal flow, wrapping long content, and no page-level horizontal scrolling.
- `UX-VISUAL-01`: retain dark Editorial Contrast, warm paper text, ruled sections, square controls, explicit destructive/reversible words, and coral destructive treatment; no cards, gradients, hover lift, authored transitions, completion badges, or color-only status.

## References

- [Source: `_bmad-output/planning-artifacts/epics.md#Story 3.5: Delete a Spending Atomically`]
- [Source: `_bmad-output/planning-artifacts/epics.md#Epic 3: Record and Maintain Exact Spendings`]
- [Source: `specs/design.md#Accounting And History`]
- [Source: `specs/design.md#HTTP Forms, Statuses, And Dispatch`]
- [Source: `specs/design.md#Testing Contract`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-2 - Layer responsibility ownership`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-4 - Group-owned identity and history`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-6 - Single ledger runtime and mutation epoch`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-7 - Snapshot-complete calculation reads`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-10 - Shared unsafe-request admission boundary`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-11 - Native HTML authority`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-15 - Safe failures, logging, and readiness`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-18 - Governed UX contracts and traceability`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md#Spending and History`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md#Lifecycle and Access`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md#Interaction Focus Matrix`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/DESIGN.md#Components`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/DESIGN.md#Layout & Spacing`]
- [Source: `_bmad-output/implementation-artifacts/3-3-browse-and-inspect-spending-history.md#Previous Story Intelligence`]
- [Source: `_bmad-output/implementation-artifacts/3-4-correct-an-existing-spending.md#Previous Story Intelligence`]
- [Source: `_bmad-output/project-context.md#Critical Implementation Rules`]
- [Source: `debtor-application/src/spendings.rs`]
- [Source: `debtor-infra/src/db/repos/spendings.rs`]
- [Source: `debtor-infra/tests/repos.rs`]
- [Source: `debtor-infra/tests/migrations.rs`]
- [Source: `debtor-web/src/handlers/spendings.rs`]
- [Source: `debtor-web/src/handlers/spending_views.rs`]
- [Source: `debtor-web/src/forms.rs`]
- [Source: `debtor-web/src/session.rs`]
- [Source: `debtor-web/src/templates.rs`]
- [Source: `debtor-web/templates/confirm.html`]
- [Source: `debtor-web/templates/transactions.html`]
- [Source: `debtor-web/src/router.rs`]
- [Source: `src/composition.rs`]
- [Source: `migrations/20260517000004_create_spendings.up.sql`]
- [Source: `migrations/20260517000005_create_spending_payers.up.sql`]
- [Source: `migrations/20260517000006_create_spending_shares.up.sql`]
- [Source: Context7 `/tokio-rs/axum`, `/askama-rs/askama`, `/bigskysoftware/htmx`, consulted 2026-08-18]

## Dev Agent Record

### Agent Model Used

openai/gpt-5.6-luna

### Debug Log References

- Resolved create-story customization for `bmad-create-story`; no activation prepend/append or completion terminal steps were configured.
- Loaded `_bmad-output/project-context.md` as the persistent project fact and `_bmad/bmm/config.yaml` with English communication/document output.
- Read the complete ordered `_bmad-output/implementation-artifacts/sprint-status.yaml`; selected first backlog story `3-5-delete-a-spending-atomically`. Epic 3 was already `in-progress`.
- Loaded Epic 3, Stories 3.3-3.4, normative `specs/design.md`, architecture spine, UX `EXPERIENCE.md`/`DESIGN.md`, PRD addendum, deferred-work ledger, current delete/read/mutation files, migrations, tests, and recent Git history.
- Identified current partial delete gaps: generic confirmation, incomplete aggregate facts, no allow-listed return/focus binding, direct service deletion bypassing root supervision, unconditional Summary redirect, and insufficient race/atomicity/UI coverage.
- Consulted current Axum, Askama, and HTMX documentation through Context7 on 2026-08-18; pinned project versions and lockfiles remain authoritative.
- Added a root-supervised Spending delete operation, typed session-bound confirmation context, complete Spending facts, canonical return/focus links, and native confirmation markup.
- Preserved parent-only checked SQL deletion so existing payer/share foreign-key cascades remain the aggregate cleanup mechanism.
- Added web confirmation/redirect coverage, complete cascade/Group eligibility coverage, and deterministic concurrent-delete coverage.

### Implementation Plan

- Extend the existing `SpendingMutationExecutor` rather than bypassing root lifecycle supervision.
- Bind delete confirmation to typed Group/Spending/cursor/focus state in the authenticated session; keep URL construction server-owned and bounded.
- Load complete detail from the existing snapshot reader, render explicit facts, reserve the shared token immediately before supervised dispatch, and redirect to the canonical Transactions context.
- Reuse the existing SQLite parent-delete cascade and validate exact cleanup, archived rejection, safe losing races, and history-free Group eligibility in invariant-owning tests.

### Completion Notes List

- Ultimate context engine analysis completed - comprehensive developer guide created.
- Implemented complete Spending delete confirmation with current Payer/Share identity facts, explicit irreversible scope, and allow-listed Transactions return state.
- Added typed session confirmation state and cleared it on Cancel/return or successful confirmation; invalid/missing state falls back to canonical Transactions recovery.
- Routed delete through the root-supervised `SpendingMutationExecutor`, preserving submission-token reservation, mutation registry, definitive outcomes, write-gate serialization, and mutation epoch semantics.
- Preserved checked parent-only SQL deletion and existing `ON DELETE CASCADE` cleanup for Payers/Shares; no migrations or SQLx metadata changed.
- Added web, repository, cascade, Group eligibility, and concurrent-delete regression coverage.
- Validation passed: workspace tests, locked workspace Clippy with warnings denied, formatting, architecture fitness, and independent password-helper fmt/Clippy/tests.
- Resolved all six code-review patch findings; no decision-needed or deferred findings remain.
- Final post-review validation passed: full workspace tests, workspace Clippy with warnings denied, and focused web/infrastructure regression suites.

### File List

- `_bmad-output/implementation-artifacts/3-5-delete-a-spending-atomically.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `debtor-application/src/spendings.rs`
- `debtor-infra/tests/repos.rs`
- `debtor-web/src/handlers/groups.rs`
- `debtor-web/src/handlers/spending_views.rs`
- `debtor-web/src/handlers/spendings.rs`
- `debtor-web/src/handlers/test_support.rs`
- `debtor-web/src/router.rs`
- `debtor-web/src/session.rs`
- `debtor-web/src/templates.rs`
- `debtor-web/templates/confirm.html`
- `debtor-web/templates/transactions.html`
- `src/composition.rs`
- `static/css/app.css`

### Change Log

- 2026-08-18: Implemented supervised atomic Spending deletion, complete confirmation scope, session-bound canonical return/focus state, cascade/eligibility tests, and concurrent-delete coverage; status moved to review.
- 2026-08-18: Resolved six adversarial review findings covering lifecycle validation, confirmation-token binding, Delete-control focus, page-boundary reconciliation, safe return fallback, and deterministic concurrency coverage.
