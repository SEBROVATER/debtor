## Deferred from: code review of 1-3-restart-and-validate-the-composed-local-application (2026-08-12)

- [x] Forced-drain test does not verify server-side request cancellation [src/runtime.rs:328-335, src/main.rs:594-601] — pre-existing runtime/test behavior, not introduced by the Story 1.3 diff. The test aborts only the client task; the review noted that a stuck server handler could survive `drop(server)` and retain resources. Revisit with the authenticated-runtime shutdown work.

## Deferred from: code review of 1-9-shut-down-the-authenticated-runtime-safely.md (2026-08-14)

- Real mutation registration and definitive outcome publication remain unwired [src/runtime.rs:78, src/composition.rs:95, src/runtime.rs:588-612] — Story 1.9 intentionally adds only the empty root-owned lifecycle seam. Existing mutation routes still do not register leases or publish `Committed`/`RolledBack`/`Unknown`; Story 2.1 owns that integration and must avoid a second registry or generic post-dispatch cancellation path.

## Deferred from: code review of 2-1-create-and-select-a-group (2026-08-14)

- Add contextual Group navigation to the existing Debts page [debtor-web/templates/debts.html:12-20] — pre-existing separate Debts template remains outside the Story 2.1 Group shell change; address it with the broader debts/context-shell work.

## Deferred from: code review of 2-3-add-group-owned-participants (2026-08-17)

- Unsupported Group Currency is not represented in the submitted option list [debtor-web/src/handlers/spending_views.rs:175-178; debtor-web/templates/group.html:73-78] — pre-existing behavior, not introduced by the Story 2.3 diff.

## Deferred from: code review of 5-2-recalculate-balances-at-current-rates (2026-08-19)

- Pending HTMX state does not expose `aria-busy="true"` or context-compatible Updating retention [debtor-web/templates/debts.html:42-45] — pre-existing behavior, not introduced by the Story 5.2 diff.
- Native and enhanced result/error flows do not focus the result heading or selected mode control [debtor-web/templates/debts.html:27,43; debtor-web/src/handlers/response.rs:48-52] — pre-existing behavior, not introduced by the Story 5.2 diff.
- Unknown `rate_mode` values in enhanced requests return a full-page error instead of a scoped result fragment [debtor-web/src/handlers/debts.rs:43-46] — pre-existing behavior, not introduced by the Story 5.2 diff.
- Web regression coverage omits Historical reset, focus parity, pending state, and incompatible-result retention [debtor-web/src/router.rs:505-526; debtor-web/src/handlers/response.rs:329-353] — pre-existing coverage gap, not introduced by the Story 5.2 diff.

## Deferred from: code review of 5-2-recalculate-balances-at-current-rates (2026-08-19 follow-up)

- Fresh fixed-past Historical rate evidence is labeled `current` [debtor-web/templates/debts.html:80] — pre-existing behavior, not introduced by the Story 5.2 diff.
- Resolved by the 2026-08-19 Debts UX redesign: enhanced mode recalculation uses HTMX's request class for the scoped Updating placeholder and final server-rendered status announcement; dynamic `aria-busy` and retained client-side financial state are not required under the no-custom-JavaScript CSP contract.
