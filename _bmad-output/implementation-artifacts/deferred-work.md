## Deferred from: code review of 1-3-restart-and-validate-the-composed-local-application (2026-08-12)

- [x] Forced-drain test does not verify server-side request cancellation [src/runtime.rs:328-335, src/main.rs:594-601] — pre-existing runtime/test behavior, not introduced by the Story 1.3 diff. The test aborts only the client task; the review noted that a stuck server handler could survive `drop(server)` and retain resources. Revisit with the authenticated-runtime shutdown work.

## Deferred from: code review of 1-9-shut-down-the-authenticated-runtime-safely.md (2026-08-14)

- Real mutation registration and definitive outcome publication remain unwired [src/runtime.rs:78, src/composition.rs:95, src/runtime.rs:588-612] — Story 1.9 intentionally adds only the empty root-owned lifecycle seam. Existing mutation routes still do not register leases or publish `Committed`/`RolledBack`/`Unknown`; Story 2.1 owns that integration and must avoid a second registry or generic post-dispatch cancellation path.

## Deferred from: code review of 2-1-create-and-select-a-group (2026-08-14)

- Add contextual Group navigation to the existing Debts page [debtor-web/templates/debts.html:12-20] — pre-existing separate Debts template remains outside the Story 2.1 Group shell change; address it with the broader debts/context-shell work.

## Deferred from: code review of 2-3-add-group-owned-participants (2026-08-17)

- Unsupported Group Currency is not represented in the submitted option list [debtor-web/src/handlers/spending_views.rs:175-178; debtor-web/templates/group.html:73-78] — pre-existing behavior, not introduced by the Story 2.3 diff.

## Deferred from: code review of 5-2-recalculate-balances-at-current-rates (2026-08-19)

- Pending HTMX state does not expose `aria-busy="true"` or context-compatible Updating retention [debtor-web/templates/debts.html:42-45] — superseded by the approved Debts exception: HTMX's request class provides the scoped Updating placeholder without dynamic `aria-busy` or client-side financial retention.
- [x] Native and enhanced result/error flows do not focus the result heading or selected mode control [debtor-web/templates/debts.html; debtor-web/src/handlers/debts.rs; debtor-web/src/handlers/response.rs] — closed 2026-08-22: native results and error documents autofocus headings; enhanced responses replace only `#debts-results` without autofocus, preserving the activated radio outside the swap while its scoped status announces the outcome. Verified by `cargo fmt --all -- --check`, `cargo test -p debtor-web`, `cargo check --workspace --all-features --locked`, `SQLX_OFFLINE=true cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`, and `cargo test --workspace --all-features --locked`.
- [x] Unknown `rate_mode` values in enhanced requests return a full-page error instead of a scoped result fragment [debtor-web/src/handlers/debts.rs] — closed 2026-08-22: unknown and duplicate mode errors return a `400` scoped `#debts-results` fragment with safe status text and no form, document, table, or autofocus.
- [x] Web regression coverage omits Historical reset, retained-radio parity for enhanced success/expected errors, the CSS-only pending state, and no-partial result replacement [debtor-web/src/router.rs; debtor-web/src/handlers/response.rs] — closed 2026-08-22: response-boundary tests cover native focus, enhanced result/error scope, invalid and duplicate modes, unavailable and unmapped failures, and percent-decoded/duplicate timeout modes.

## Deferred from: code review of 5-2-recalculate-balances-at-current-rates (2026-08-19 follow-up)

- Fresh fixed-past Historical rate evidence is labeled `current` [debtor-web/templates/debts.html:80] — pre-existing behavior, not introduced by the Story 5.2 diff.
- [x] Policy resolved by the 2026-08-22 JavaScript reconciliation: enhanced mode recalculation uses HTMX's request class for the scoped Updating placeholder and final server-rendered status announcement; dynamic `aria-busy`, retained client-side financial state, application-owned HTMX event handlers, and imperative post-swap behavior are not permitted. Focus/error parity coverage was closed on 2026-08-22 with the verified scoped-fragment implementation.

## Deferred from: code review of 5-3-derive-complete-advisory-settlement-transfers (2026-08-20)

- Settlement uses saturating arithmetic [debtor-domain/src/debts/simplify.rs:60] — pre-existing use of `saturating_sub` conflicts with the project’s checked-arithmetic preference but was not introduced by Story 5.3.

- source_spec: `_bmad-output/implementation-artifacts/spec-close-deferred-debts-parity-coverage.md`
  summary: Mark completed epics done in sprint tracking when all stories are done.
  evidence: `epic-5` and prior epics remain `in-progress` despite every story being done, contradicting the sprint-status definition.
