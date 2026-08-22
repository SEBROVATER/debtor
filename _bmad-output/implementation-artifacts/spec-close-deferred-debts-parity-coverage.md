---
title: 'Close Deferred Debts Focus and Error-Parity Coverage'
type: 'bugfix'
created: '2026-08-21T16:58:34+06:00'
status: 'done'
review_loop_iteration: 1
baseline_commit: 'd7c5064b5bedf5f06950642288767d48efee0a7d'
context:
  - '{project-root}/specs/design.md'
  - '{project-root}/_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md'
---

<frozen-after-approval reason="superseded by approved 2026-08-22 Sprint Change Proposal; do not modify unless human renegotiates">

## Intent

**Problem:** The Debts page needs focus/error parity coverage that preserves the native and enhanced paths without inventing imperative browser behavior. Native full-page responses may autofocus their result or error heading. Enhanced success and expected enhanced errors must retain focus on the activated rate-mode radio while the server-rendered scoped status announces the complete result or no-partial failure.

**Approach:** Preserve native full-page and enhanced progressive paths. Define and test the focus contract: native calculations and failures may autofocus their result/error headings; enhanced successful mode changes and expected enhanced errors retain the activated radio outside the swapped result region; the server-rendered scoped status announces one final outcome. Do not use an event handler or imperative post-swap focus repair.

## Boundaries & Constraints

**Always:** Retain server-rendered Askama/HTMX behavior; keep the stable `#debts-results` target and polite atomic `#debts-status`; preserve selected rate mode, HTTP status, safe error text, and no-partial-financial-results policy. Keep full-page native errors intact, retain the activated radio for enhanced success and expected errors, and keep tests at the web/response boundary. Update `specs/design.md` before the behavior change and synchronize the deferred-work and sprint action tracking when this scope is completed.

**Ask First:** Stop for approval before expanding this work beyond Debts focus/error parity, changing the HTMX dependency/CSP policy, or altering calculation/rate semantics.

**Never:** Do not add manually authored application JavaScript, inline scripts or event handlers, custom HTMX extensions, application-owned HTMX event handlers, client-retained financial data, dynamic `aria-busy`, imperative post-swap behavior, new routes, payment state, or partial Balance/Settlement Transfer rendering. Do not change enhanced-success or expected-enhanced-error focus away from the activated Historical/Current radio. Other official extensions require explicit design and security approval before addition.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Native calculation | Omitted or valid `rate_mode`; normal Debts GET | Full Debts document selects Historical by default or requested mode and autofocuses `#debts-results-heading` | Existing safe full-page result remains intact |
| Enhanced calculation | `HX-Request` with valid Current or Historical mode | Successful response preserves selected control outside swapped results; result fragment has no autofocus | Browser retains focus on activated radio; one status announces completion |
| Native invalid mode | Unknown, empty, or duplicate `rate_mode` | `400` full error document autofocuses `#error-heading` | Shows only sanitized `Unknown rate mode.` guidance |
| Enhanced invalid mode | Same invalid mode with `HX-Request` | `400` `#debts-results` fragment has no autofocus; the activated radio remains focused outside the replacement; no document, form, table, or financial rows | Stable `#debts-status` announces one safe error |
| Enhanced calculation failure/timeout | Mapped application failure or Debts timeout with `HX-Request` | Original `404`/`503`/`500`/`504` status and safe attempted-mode context; no result-heading autofocus and activated radio remains focused | No Balance or Settlement Transfer table; response stays a scoped fragment |

</frozen-after-approval>

## Code Map

- `specs/design.md` -- normative Debts enhanced/native focus and safe-error contract.
- `debtor-web/templates/debts.html` -- stable mode controls outside `#debts-results`, announced status, native-only heading autofocus, and no enhanced-fragment autofocus.
- `debtor-web/src/handlers/debts.rs` -- selects native result autofocus and preserves activated-radio focus for enhanced success and expected errors.
- `debtor-web/src/handlers/response.rs` -- maps calculation, mode, and timeout failures to native documents or scoped enhanced Debts fragments without imperative focus behavior.
- `debtor-web/src/router.rs` -- authenticated Debts route tests for mode selection and full-page/scoped response parity.
- `_bmad-output/implementation-artifacts/deferred-work.md` -- records the source deferred findings to close only after implementation and validation evidence.
- `_bmad-output/implementation-artifacts/sprint-status.yaml` -- contains the open Epic 5 coverage action item to mark done after verification.

## Tasks & Acceptance

**Execution:**
- [x] `specs/design.md` -- state the route-specific Debts focus contract for native success/failure and enhanced success/failure -- makes the behavior authoritative before code changes.
- [x] `debtor-web/src/handlers/response.rs` -- render all enhanced Debts failure variants through one equivalent scoped result fragment without heading autofocus; retain the native error-document path -- preserves activated-radio focus for calculation, invalid-mode, and timeout failures without divergent markup.
- [x] `debtor-web/src/handlers/debts.rs` -- retain native-only result autofocus and preserve activated-radio focus for enhanced success and expected errors -- prevents all expected enhanced outcomes from stealing focus.
- [x] `debtor-web/src/router.rs` -- extend authenticated route tests for default/Current native result focus, enhanced success/error radio retention without result autofocus, and native/enhanced unknown and duplicate mode parity -- proves the progressive paths have their intended focus target and response shape.
- [x] `debtor-web/src/handlers/response.rs` -- extend mapper tests for enhanced unavailable, unmapped application failures, invalid mode, and timeouts -- prove each status-preserving fragment has the announced status, no autofocus, no full document/form/table, and no partial financial output; retain percent-decoded and duplicate-query timeout assertions.
- [x] `_bmad-output/implementation-artifacts/deferred-work.md` -- close only the verified Debts focus/error coverage entries after implementation evidence -- preserves the append-only review ledger.
- [x] `_bmad-output/implementation-artifacts/sprint-status.yaml` -- mark the Epic 5 Debts coverage action item done only after all checks pass -- synchronizes retrospective tracking with the delivered evidence.

**Acceptance Criteria:**
- Given an authenticated native Debts request with omitted or valid `rate_mode`, when calculation completes, then the selected mode is visible and `#debts-results-heading` has `tabindex="-1" autofocus`.
- Given an authenticated enhanced Debts request with a valid mode, when calculation completes, then the activated selected radio remains focused outside the response replacement, `#debts-results` is returned, its scoped status announces one outcome, and no result heading autofocus is emitted.
- Given native unknown or duplicate `rate_mode`, when the route rejects it, then it returns `400` full-page safe error HTML with autofocus on `#error-heading`.
- Given enhanced unknown or duplicate `rate_mode`, when the route rejects it, then it returns `400` and only a non-autofocus `#debts-results` failure fragment with `Unknown rate mode.`, announced status, activated-radio focus retention, and no `<html>`, `<form>`, `<table>`, Balances, or Settlement Transfers.
- Given an enhanced Debts application failure or timeout, when the failure is rendered, then its original HTTP status and sanitized attempted-mode message are preserved, the activated radio retains focus, the scoped status announces the failure, and no partial financial rows are rendered.

## Design Notes

The Debts mode radios remain outside the `#debts-results` outer-HTML swap. Enhanced success and expected enhanced failures therefore retain focus on the activated radio. The replacement result keeps the stable server-rendered status region, which announces one final outcome. Native full-page result/error responses may autofocus their headings. No application-owned HTMX event handler, client-side financial state, or imperative post-swap behavior is used to manage focus or status.

## Verification

**Commands:**
- `cargo fmt --all -- --check` -- expected: workspace formatting passes.
- `cargo test -p debtor-web` -- expected: Debts router and response-mapping regressions pass.
- `cargo check --workspace --all-features --locked` -- expected: workspace compiles with all features.
- `SQLX_OFFLINE=true cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` -- expected: no denied warnings.
- `cargo test --workspace --all-features --locked` -- expected: all workspace tests pass.

## Suggested Review Order

**Interaction Contract**

- Defines the native/enhanced focus boundary and complete browser-behavior allowlist.
  [`design.md:87`](../../specs/design.md#L87)

- Keeps mode controls mounted while replacing only the financial results region.
  [`debts.html:32`](../../debtor-web/templates/debts.html#L32)

- Shares the result markup while allowing native-only forward autofocus.
  [`debts_results.html:1`](../../debtor-web/templates/debts_results.html#L1)

**Response Boundaries**

- Returns a scoped template fragment for enhanced success without rendering the shell.
  [`debts.rs:163`](../../debtor-web/src/handlers/debts.rs#L163)

- Centralizes enhanced error fragments while retaining native error documents.
  [`response.rs:82`](../../debtor-web/src/handlers/response.rs#L82)

**Regression Evidence**

- Exercises scoped enhanced success and validates no document or autofocus leaks.
  [`router.rs:748`](../../debtor-web/src/router.rs#L748)

- Covers malformed enhanced modes and native autofocus error responses.
  [`router.rs:849`](../../debtor-web/src/router.rs#L849)

**Policy Synchronization**

- Makes the explicit approved browser policy available to all epic stories.
  [`epics.md:301`](../planning-artifacts/epics.md#L301)
