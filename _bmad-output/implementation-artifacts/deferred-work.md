## Deferred from: code review of 1-3-restart-and-validate-the-composed-local-application (2026-08-12)

- [x] Forced-drain test does not verify server-side request cancellation [src/runtime.rs:328-335, src/main.rs:594-601] — pre-existing runtime/test behavior, not introduced by the Story 1.3 diff. The test aborts only the client task; the review noted that a stuck server handler could survive `drop(server)` and retain resources. Revisit with the authenticated-runtime shutdown work.

## Deferred from: code review of 1-9-shut-down-the-authenticated-runtime-safely.md (2026-08-14)

- Real mutation registration and definitive outcome publication remain unwired [src/runtime.rs:78, src/composition.rs:95, src/runtime.rs:588-612] — Story 1.9 intentionally adds only the empty root-owned lifecycle seam. Existing mutation routes still do not register leases or publish `Committed`/`RolledBack`/`Unknown`; Story 2.1 owns that integration and must avoid a second registry or generic post-dispatch cancellation path.
