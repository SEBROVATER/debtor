## Deferred from: code review of 1-3-restart-and-validate-the-composed-local-application (2026-08-12)

- [x] Forced-drain test does not verify server-side request cancellation [src/runtime.rs:328-335, src/main.rs:594-601] — pre-existing runtime/test behavior, not introduced by the Story 1.3 diff. The test aborts only the client task; the review noted that a stuck server handler could survive `drop(server)` and retain resources. Revisit with the authenticated-runtime shutdown work.
