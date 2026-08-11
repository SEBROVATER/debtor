# Security Architecture Review

## Verdict

**REJECT pending security closure.** The spine preserves the intended security posture at a policy level, but it does not yet constrain independently implemented epics tightly enough in several security-critical seams. The findings below are architecture holes, not requests for implementation detail: each permits an implementation to satisfy the current spine while weakening a control required or implied by `specs/design.md` and accepted ADRs.

## Review Basis

- Reviewed artifact: `ARCHITECTURE-SPINE.md`, draft dated 2026-08-10.
- Normative baseline: `specs/design.md`.
- Accepted decisions: `specs/adr/0001-foundation-architecture.md` and `specs/adr/0002-long-term-foundation-hardening.md`.
- Focus: authentication, sessions, CSRF, submission tokens, mutation cancellation, proxy trust, early data, CSP/HTMX, diagnostics, lifecycle, denial-of-service bounds, and singleton state.
- Excluded: implementation defects, general maintainability improvements, and security controls outside the accepted product boundary.

## Findings

### SEC-01 - CRITICAL - The accepted password-hash policy is not concretely bounded

**Evidence:** Spine AD-14 requires a "bounded Argon2id v19" hash and the stack delegates cryptography to an adapter, but neither the spine nor its security conventions define accepted lower and upper bounds for memory, iterations, parallelism, salt length, hash length, or encoded input length. ADR 0001 section 11 calls for validation of the "complete bounded Argon2id v19 policy" before database access; `specs/design.md:87` likewise requires a valid bounded hash.

**Security hole:** A cryptography epic can comply by accepting an Argon2id v19 hash with trivially weak cost parameters, or by selecting bounds that permit startup resource exhaustion. "Bounded" alone establishes neither a security floor nor a resource ceiling. Different adapters or the password helper can also implement incompatible policies while each claims compliance.

**Required architecture closure:** Establish one named password-hash policy, shared by startup validation, verification, tests, and the password helper, with explicit algorithm/version, parameter minima and maxima, encoded-length/salt/hash bounds, and rejection-before-expensive-work semantics. The root must validate that exact policy before database connection or migration.

### SEC-02 - HIGH - Application-side trusted client identity resolution is absent

**Evidence:** Spine AD-12 assigns forwarding sanitation to the edge but does not project the application-side requirements from `specs/design.md:89,93,98`: only an immediate peer within `APP_TRUSTED_PROXY_CIDRS` may supply one explicitly selected `APP_TRUSTED_PROXY_HEADER` format. AD-13 names the login limiter as a singleton but does not name a singleton trusted-client resolver shared by all client-IP consumers.

**Security hole:** A deployment epic can trust forwarding input unconditionally because the edge is described as sanitizing, accept multiple header formats, or resolve a different chain element than the limiter expects. A direct or misrouted backend client could then spoof limiter identity and bypass the five-attempt login bound. Independently composed resolvers could produce protocol- or route-dependent identities while all listed singleton owners remain unique.

**Required architecture closure:** Bind one root-composed client-identity resolver to the configured trusted CIDRs and exactly one header mode. It must derive trust from the socket peer, ignore forwarding input from untrusted peers, reject malformed or ambiguous chains fail closed, and be the sole identity source for login admission. Startup must reject invalid, empty-in-production, or contradictory trust configuration before binding.

### SEC-03 - HIGH - Production cookie and inactivity semantics are omitted

**Evidence:** Spine AD-14 specifies capacities, expiry durations, rotation, logout, and restart invalidation, but omits the mandatory `HttpOnly` and `SameSite=Strict` attributes, non-debug `Secure` requirement, authenticated inactivity refresh on every request, and explicit anonymous-session save before rendering login. These are explicit in `specs/design.md:87,89` and ADR 0001 section 9.

**Security hole:** A session epic can use a script-readable cookie, omit `SameSite=Strict`, permit a production cookie over plaintext HTTP, implement 30 days as absolute rather than sliding inactivity, or render a CSRF token before its anonymous server-side session is durably admitted. Those choices satisfy the spine's current lifecycle wording but weaken session theft and CSRF defenses or make login admission race with lazy persistence.

**Required architecture closure:** State the complete cookie profile and environment rule, define authenticated expiry as sliding inactivity refreshed on every admitted request, and require anonymous session allocation plus successful explicit persistence before login HTML is emitted. Cookie/session rotation must be atomic with authenticated promotion and CSRF rotation.

### SEC-04 - HIGH - The shared unsafe-request boundary does not fix security-check ordering

**Evidence:** Spine AD-10 lists "authentication or password-gate admission, strict session-backed CSRF validation, bounded body extraction, route validation" but does not require the single shared CSRF-validating form extractor mandated by `specs/design.md:91`, nor state that CSRF succeeds before password verification, route-specific parsing, asynchronous prechecks, or any use-case invocation. Its written order can be read as password-gate work preceding CSRF, while body extraction necessarily precedes validation of a form token.

**Security hole:** A login epic can perform expensive Argon2 verification before rejecting a missing or invalid CSRF token, giving cross-site traffic access to the scarce four-request login budget and password-verification work. Feature epics can implement route-local extractors with subtly different duplicate-field or malformed-token handling, or run asynchronous prechecks before the common defense.

**Required architecture closure:** Define one transport-level admission sequence: outer body/concurrency/deadline admission; bounded structural extraction that preserves duplicate detection; session load; CSRF validation; authentication/password use-case admission as applicable; route-specific parsing and prechecks; atomic submission-token reservation; exactly one dispatch. Require missing, duplicate, malformed, and incorrect CSRF values to fail before route-specific work and use-case invocation on every unsafe route, including login.

### SEC-05 - HIGH - Submission-token storage and terminal transitions are under-specified

**Evidence:** AD-10 calls tokens bounded, expiring, session-bound, and atomically reserved, but gives no per-session or global live-token capacity, admission behavior at capacity, expiry index/cleanup ownership, or post-dispatch state transition. AD-13 does not identify submission-token state/cleanup as a singleton owner. `specs/design.md:91` requires bounded, expiring, session-bound single-use tokens and exactly one mutation attempt.

**Security hole:** One anonymous session can repeatedly render login and accumulate an arbitrarily large number of live tokens even though anonymous session count is capped. Separate feature-owned token registries can bypass a global bound. An implementation may also release a reserved token after rollback or leave it indefinitely reserved after task failure, allowing either a second mutation attempt or persistent capacity exhaustion while still matching the current reservation wording.

**Required architecture closure:** Assign token state to one process-local owner, set explicit per-session and global capacities and expiry limits, require indexed bounded cleanup and fail-closed rendering/admission at capacity, and define an atomic state machine. Reservation must permit exactly one dispatch and become terminal regardless of commit, rollback, panic, or response-delivery failure; pre-dispatch validation must leave the token reusable or replace it deterministically without increasing bounds.

### SEC-06 - HIGH - Definitive mutation completion conflicts with the ten-second shutdown cap

**Evidence:** AD-10 and AD-14 prohibit cancellation after dispatch and require a definitive commit or rollback result. AD-14 also says shutdown drains for at most ten seconds, then checkpoints and closes the SQLite pool. No owner, cancellation-shield, join protocol, or bounded post-dispatch execution rule resolves what happens when a dispatched mutation is still running at ten seconds. ADR 0001 sections 10 and 11 contain both obligations but the spine is expected to make them implementable across epics.

**Security hole:** A lifecycle epic can close the pool or abort request tasks at the drain deadline while a mutation epic assumes post-dispatch cancellation is impossible. That can produce an unknown client outcome, interrupt multi-step orchestration, or cause a retry after a commit whose response was lost. Conversely, waiting forever preserves mutation semantics but violates bounded shutdown.

**Required architecture closure:** Define one root-owned mutation task registry and shutdown protocol. Either prove and enforce a post-dispatch upper bound below the shutdown budget, or distinguish HTTP drain from mutation-task completion and specify how the process remains alive until every dispatched mutation reaches a known commit/rollback terminal state. Pool close and WAL checkpoint must occur only after that terminal barrier; response disconnect and request-future cancellation must not own mutation execution.

### SEC-07 - HIGH - Early-data enforcement is too weak to be interoperable

**Evidence:** AD-12 says only "unsafe early-data rejection." It omits the concrete contract in `specs/design.md:99`: disable early data entirely or allow only `GET`/`HEAD` through an explicitly marked early-data path, and reject unsafe early data with `425 Too Early`. It also omits the required rollout verification across QUIC and TCP fallback from `specs/design.md:101,118` and ADR 0002 decision 8.

**Security hole:** A proxy epic can classify methods differently, return a generic rejection that a client does not handle as early-data replay protection, or enable early data globally while assuming application CSRF makes mutations safe. The spine's wording does not provide an acceptance test capable of detecting those deployments.

**Required architecture closure:** Choose one allowed policy shape, require `425` for every unsafe early-data request before backend forwarding, permit only explicitly marked `GET`/`HEAD` when early data is enabled, and require deployment tests proving no unsafe backend request across HTTP/3 and fallback plus identical client identity resolution.

### SEC-08 - MEDIUM - Startup does not establish a fail-closed admission barrier

**Evidence:** The spine assigns configuration, migrations, startup, supervision, and shutdown to root and validates the password hash before database access, but it does not order all mandatory configuration validation, migration/pragma establishment, singleton construction, supervisor startup, readiness health, and socket admission. `specs/design.md:103-107` requires configuration, database connection/migration, adapter composition, then binding; AD-14 makes cleanup supervisor failure readiness-fatal.

**Security hole:** Independently implemented startup and HTTP epics can bind and accept login or mutation traffic before trusted-proxy configuration is validated, WAL/foreign-key settings and migrations are established, or mandatory supervisors are known healthy. Readiness can remain false while a directly reachable backend still serves requests.

**Required architecture closure:** Define a root-owned startup state machine with no user socket admission until all configuration is validated, the database is migrated and required pragmas verified, all singleton owners are composed, and mandatory supervisors report healthy. Any failure before the admission barrier must exit without serving; supervisor failure after admission must first stop new user admission and then enter the defined shutdown protocol.

### SEC-09 - MEDIUM - Secret safety governs logs but not all diagnostic sinks

**Evidence:** AD-15 has a strong log deny-list and sanitized HTTP/error boundaries, while telemetry backend/export is deferred. It does not apply the same classification to metrics labels, traces/spans, panic hooks, crash reports, readiness details, or exporter error paths. The heading and prevention claim cover "diagnostics," but the binding rule is materially log-specific. `specs/design.md:38,89,116` requires safe reason categories and secret-safe diagnostics/logging.

**Security hole:** An observability epic can comply with the log rule while placing a session ID, CSRF/submission token, client identity, query string, SQL/provider error chain, identifier, or monetary value into span fields or metric labels. Exporter failures can then echo those values through otherwise safe logs.

**Required architecture closure:** Define one cross-sink diagnostic data policy and safe event schema covering logs, traces, metrics, panics, probes, crash reporting, and exporter self-diagnostics. Only fixed operation names, bounded reason categories, and explicitly approved low-cardinality fields may cross the diagnostic boundary; raw errors and request-derived fields must be reduced before instrumentation.

### SEC-10 - MEDIUM - Pinned HTMX versions are not pinned artifacts

**Evidence:** AD-11 fixes HTMX and `response-targets` versions and CSP, but the spine explicitly defers integrity hashes and provenance records for vendored assets. ADR 0001 section 13 requires pinned self-hosted official assets and forbids custom JavaScript/extensions. CSP permits same-origin scripts, so it cannot distinguish an official pinned asset from altered vendored content.

**Security hole:** A static-assets epic can ship modified, substituted, or accidentally rebuilt JavaScript under the pinned filenames and versions while satisfying self-hosting and CSP. Such code executes with authenticated origin privileges and can read form fields and issue same-origin requests; `HttpOnly` does not protect CSRF/submission values present in the DOM.

**Required architecture closure:** Make exact upstream artifact provenance and cryptographic digests binding before vendoring is implemented, verify those bytes in CI/build fitness, serve only the approved assets with fixed content types and `nosniff`, and fail tests if additional executable assets or inline executable attributes are introduced.

## Closure Gate

The spine is suitable for independent epic implementation only after SEC-01 through SEC-07 are resolved as binding invariants and SEC-08 through SEC-10 have enforceable ownership and verification criteria. Closure should update `specs/design.md` first wherever the accepted baseline itself lacks concrete bounds, then synchronize the accepted ADRs and spine as required by the project change-authority rule.
