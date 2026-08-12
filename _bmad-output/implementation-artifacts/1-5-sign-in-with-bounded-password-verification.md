---
baseline_commit: 18f00e4
---

# Story 1.5: Sign In with Bounded Password Verification

Status: done

## Story

As the administrator,
I want login attempts verified safely and successful authentication persisted atomically,
so that I can access Debtor without brute-force, proxy-spoofing, or partial-session risk.

## Acceptance Criteria

1. **Given** production proxy configuration has an empty trusted CIDR set, an unrecognized or nonsingular forwarding-header mode, or otherwise cannot establish one trusted client-resolution policy
   **When** Debtor starts
   **Then** startup fails before socket admission with a sanitized configuration error
   **And** debug/local mode may use the direct peer only as explicitly allowed.

2. **Given** a login request arrives through an immediate peer outside `APP_TRUSTED_PROXY_CIDRS`
   **When** forwarding headers are present
   **Then** Debtor ignores them and resolves the direct peer according to environment policy
   **And** no raw forwarding value or resolved client IP is logged.

3. **Given** a trusted proxy supplies the configured forwarding format
   **When** client identity is resolved
   **Then** only that selected format and trusted chain order are accepted
   **And** the resulting limiter behavior is identical for edge HTTP/3 and TCP fallback requests.

4. **Given** a login body exceeds 8 KiB or cannot be structurally decoded within bounds
   **When** `POST /login` is processed
   **Then** it is rejected by the shared strict form extractor before CSRF validation, password verification, limiter reservation, or authentication dispatch
   **And** no credential or submitted value is logged.

5. **Given** the form structure is valid but CSRF is missing, duplicate, malformed, or incorrect
   **When** login is submitted
   **Then** the request is rejected before limiter reservation or password verification
   **And** the submission token is not consumed because dispatch did not occur.

6. **Given** bounded structural decoding and CSRF validation succeed but a required non-security field is missing, duplicate, malformed, or unknown
   **When** strict route-field validation runs
   **Then** the request is rejected before password parsing, limiter reservation, submission-token reservation, or authentication dispatch
   **And** the valid submission token remains usable.

7. **Given** CSRF and route validation succeed but the submission token is missing, unknown, expired, reserved, or consumed
   **When** login is submitted
   **Then** Debtor returns `409 Conflict` with sanitized feedback
   **And** neither the limiter nor password verifier is invoked.

8. **Given** a valid login attempt is ready for password verification
   **When** its submission token is atomically reserved and the trusted-client limiter is consulted
   **Then** the limiter records one attempt immediately before every password verification, including a correct password, permits at most five attempts in a rolling five-minute window, tracks at most 4,096 active client keys without eviction, and fails closed with retryable `429` for an unseen key at capacity
   **And** any rejection before password verification records no attempt.

9. **Given** a trusted-client limiter history ages beyond its rolling five-minute window
   **When** indexed bounded expiry cleanup runs or that key is next evaluated
   **Then** the expired history is physically removed and capacity becomes reusable
   **And** active histories are never evicted to admit an unseen key.

10. **Given** password verification is admitted
    **When** concurrent attempts are processed
    **Then** at most two Argon2 verifications run concurrently using the already validated configured hash
    **And** incorrect credentials receive a fixed sanitized response without revealing comparison details.

11. **Given** the submitted password is correct and authenticated capacity is available
    **When** login promotion occurs
    **Then** Debtor atomically rotates and durably stores the session ID, authenticated state, and a new CSRF token before emitting an authenticated cookie or `303` redirect
    **And** only after durable promotion does it reset the trusted-client limiter history.

12. **Given** correct-password promotion finds 32 live authenticated sessions or durable session persistence fails
    **When** promotion is attempted
    **Then** Debtor flushes the anonymous login session and returns retryable `503 Service Unavailable` without emitting an authenticated cookie
    **And** the reserved submission token remains terminal because one dispatch occurred.

13. **Given** the protected Login form is submitted once
    **When** password verification or durable promotion is pending
    **Then** the submit initiator becomes unavailable, repeated activation is suppressed or coalesced, the form region exposes `aria-busy`, and one stable polite atomic status node announces the pending state without moving focus
    **And** native submission remains authoritative under `UX-STATUS-01` and `UX-FOCUS-01`.

14. **Given** credentials are incorrect or sign-in is rate-limited, capacity-blocked, timed out, or temporarily unavailable
    **When** the safe Login response renders
    **Then** the password is not retained, the response discloses no credential/client/session detail, and the stable Login error/status destination receives the exact alert or focus treatment defined by the outcome
    **And** recovery remains a protected native form with a fresh applicable token rather than a replay.

15. **Given** authentication and durable promotion succeed
    **When** the `303` destination renders
    **Then** the stable authenticated page heading is the single forward-focus destination and no private page is restored from an HTMX history snapshot
    **And** browser history reveals no cached Login password or ledger content.

16. **Given** every Login outcome is rendered at 320 CSS pixels and 400% zoom
    **When** controls, messages, and recovery actions wrap
    **Then** all controls remain at least 48 by 48 CSS pixels, text and focus contrast hold, no clipping or page-level horizontal scroll occurs, and Editorial Contrast states remain square and motion-free
    **And** native and enhanced responses are visually and behaviorally equivalent.

**Requirements:** `SPEC-FR4..SPEC-FR5`, `SPEC-FR8`, `SPEC-FR12..SPEC-FR13`, `SPEC-FR15`, `SPEC-FR18..SPEC-FR19`, `SPEC-FR88..SPEC-FR89`, `SPEC-FR100`, `SPEC-FR103` (Login admission only); `SPEC-NFR17..SPEC-NFR18`, `SPEC-NFR21..SPEC-NFR25`; trusted-proxy, strict-form, anonymous-token reservation, password-concurrency, limiter, durable-promotion, and safe-diagnostic requirements; UX contracts `UX-SHELL-01`, `UX-TARGET-01`, `UX-FOCUS-01`, `UX-STATUS-01`, `UX-RESPONSIVE-01`, `UX-VISUAL-01`.

## Tasks / Subtasks

- [x] Enforce the trusted-client startup and resolution contract (AC: 1-3)
  - [x] Extend the existing root configuration and `TrustedProxyConfig` boundary with an explicit debug/non-debug policy. Non-debug configuration must reject empty CIDRs, missing/unknown headers, and any policy that cannot establish exactly one selected forwarding mode before database connection, migration, listener binding, or socket admission. Debug/local may retain direct-peer fallback only.
  - [x] Preserve existing CIDR parsing, IPv4-mapped canonicalization, selected-header-only parsing, right-to-left trusted-chain traversal, and safe rejection of malformed trusted input. An untrusted immediate peer must ignore even malformed forwarding headers and use the direct peer.
  - [x] Keep proxy resolution in `debtor-web`; pass only `IpAddr` to the application authentication port. Do not log forwarding headers, resolved IPs, or limiter keys.
  - [x] Synchronize `.env.example`, `README.md`, and any configuration examples so production trust settings are documented as required while debug direct-peer behavior remains clear.

- [x] Complete the strict Login POST admission boundary (AC: 4-7, 13-14)
  - [x] Extend the existing `POST /login` handler and shared extractor; do not add a second Login route or bypass `CsrfValidatedForm`.
  - [x] Preserve the route's 8 KiB body limit, four-request concurrency limit, 30-second Login deadline, session-backed CSRF validation, exact field-name/duplicate checks, and no-dispatch behavior for every pre-verification rejection.
  - [x] Make the Login route participate in the existing `MutationPreflight`/deadline boundary, or adapt that boundary narrowly so token reservation and the one authentication dispatch crossing cannot be split by an expired/rejected preflight. Do not reserve the token and then allow `dispatch()` to fail.
  - [x] Validate the submitted `submission_token` against the current session before authentication. Missing, unknown, expired, session-mismatched, reserved, or consumed tokens must return sanitized `409` and must not invoke the limiter or password verifier. Validation failures before dispatch preserve a valid token.
  - [x] Atomically reserve the anonymous token immediately before the authentication service's first state-changing/admission call. Once the request crosses this boundary, the token is terminal regardless of invalid password, rate rejection, promotion failure, task failure, or response delivery.
  - [x] Keep the password as transient input only. Never echo it in Login templates, errors, logs, tracing, query strings, tests, or retained form values. Valid recovery renders a fresh Login token and CSRF as applicable.
  - [x] Preserve the existing authenticated-session guard: authenticated `GET`/`POST /login` must redirect to `/groups` without anonymous rendering, token issuance, downgrade, flush, or authentication work.

- [x] Reuse the bounded application authentication and Argon2 adapters (AC: 8-10)
  - [x] Keep `AuthenticationUseCases::attempt` as the sole web-to-application authentication call. It must continue to reserve the limiter immediately before verification, count correct and incorrect password checks, and return safe `RetryAfter`, `Authenticated`, or `InvalidPassword` outcomes.
  - [x] Do not duplicate limiter state or call `MemoryLoginAttemptLimiter` directly from web. Preserve one root-composed limiter with five attempts/rolling five minutes, 4,096 active keys, indexed expiry, no active-key eviction, and retryable `429` for unseen-key capacity exhaustion.
  - [x] Reuse `ArgonPasswordGate` and the Story 1.1 canonical validated hash policy. Preserve `spawn_blocking` and the process-wide maximum of two concurrent Argon2 verifications; no password/KDF policy changes or dependency upgrades are part of this story.
  - [x] Map application/adapter failures to fixed Login-safe responses. Do not expose raw `ApplicationError`, Argon2, session, proxy, or limiter details.

- [x] Make durable authenticated promotion all-or-nothing (AC: 11-12, 14-15)
  - [x] Reuse `session::establish`: `cycle_id`, authenticated 30-day expiry, new CSRF, authenticated marker, and `save()` must complete before the response can emit the authenticated cookie or `303 /groups`.
  - [x] On authenticated-capacity exhaustion or persistence failure, flush the anonymous session, return retryable sanitized `503`, emit no authenticated cookie, and do not reset limiter history. The already-reserved token remains terminal.
  - [x] Call `AuthenticationUseCases::complete_login` only after durable promotion succeeds. Ensure failed or invalid attempts never reset limiter history.
  - [x] Preserve the existing no-store/security headers and Login recovery semantics. Successful authenticated content must not be cached or restored from a private HTMX history snapshot.

- [x] Preserve native-first Login UX and response states (AC: 13-16)
  - [x] Extend the existing `login.html`/`LoginTemplate` rather than changing the form contract: one password field, one CSRF, one submission token, native `POST /login`, stable `sign-in-heading`, `login-status`, and form-region IDs.
  - [x] Represent pending state with the existing stable polite atomic status node and `aria-busy`; do not add custom JavaScript, inline scripts/attributes, custom HTMX extensions, CDN assets, or client-only authentication behavior. If pinned HTMX assets are not already vendored, native behavior remains the authoritative implementation and asset work must not become an unbounded dependency of this story.
  - [x] Apply the focus matrix: pending/error retains the invoker or scoped status behavior; successful forward navigation targets the stable authenticated heading; ordinary refresh does not force password focus. Keep native and enhanced response markup/status behavior equivalent.
  - [x] Preserve the Editorial Contrast scoped CSS, square controls, warning text/rules, minimum 48px targets, 320px/400% zoom behavior, no clipping/page-level horizontal scroll, and no transitions/decorative depth. Do not regress authenticated-surface styles.

- [x] Add invariant-owning deterministic tests (AC: 1-16)
  - [x] Add configuration tests proving production proxy-policy failures occur before DB connection/migration/listener admission and debug direct-peer defaults remain valid. Assert errors never contain CIDRs, headers, IPs, passwords, or hashes.
  - [x] Extend proxy unit tests for untrusted-peer header ignoring, selected-format-only behavior, malformed trusted input, trusted-chain order, IPv4-mapped addresses, and equivalent resolved identity for transport paths.
  - [x] Extend Login web tests with barriers/counters/fakes, not a mocking framework: malformed/oversized/CSRF/route-field rejection does not reserve a token, limiter attempt, or invoke password verification; token conflicts return `409`; valid tokens are terminal after dispatch.
  - [x] Cover valid/invalid passwords, correct-password limiter counting, five-attempt window, unseen-key capacity `429`, indexed expiry reuse, concurrent Argon2 cap, and no raw diagnostic leakage.
  - [x] Cover successful session ID/CSRF rotation and durable save before `303`; capacity-full and persistence-failure promotion return `503`, flush anonymous state, emit no authenticated cookie, retain terminal token state, and do not reset limiter history.
  - [x] Preserve regression coverage for authenticated `/login` downgrade prevention, probes/static routes remaining session-free, exact security headers, one CSRF/submission field each, password non-retention, status/`aria-busy`, native action, and rendered UX contracts at 320px/400% zoom. If no executable browser harness exists, document browser evidence as manual rather than claiming an automated pass.
  - [x] Use barriers/notifications/held locks for ordering and concurrency assertions; never use timing sleeps as proof.

### Review Findings

- [x] [Review][Patch] Map a missing submission token to `409 Conflict` [debtor-web/src/handlers/auth.rs:43-47] — fixed by treating the submission token as a replay/conflict outcome before authentication dispatch.
- [x] [Review][Patch] Provide Login recovery after promotion failure [debtor-web/src/handlers/auth.rs:102-105; debtor-web/src/handlers/response.rs:55-60] — fixed by flushing and rendering a fresh Login recovery form while preserving `503`.
- [x] [Review][Patch] Preserve Login-specific timeout recovery for POST preflight [debtor-web/src/forms.rs:50-88; debtor-web/src/middleware.rs:41-47] — fixed by tagging Login mutation preflight timeouts.
- [x] [Review][Patch] Read token time after acquiring the token-store lock [debtor-web/src/submission_tokens.rs:209-220] — fixed by taking the clock reading under the store lock.
- [x] [Review][Patch] Make session-cycle failure cleanup authoritative [debtor-web/src/session.rs:60-72; debtor-web/src/handlers/auth.rs:102-105] — fixed by checking flush failure and returning safe recovery/error responses.
- [x] [Review][Patch] Implement the required Login pending state [debtor-web/templates/login.html:11-23; debtor-web/src/router.rs:49-60] — fixed the rendered busy/disabled contract without adding custom application JavaScript.
- [x] [Review][Patch] Add the authenticated destination focus target [debtor-web/templates/groups.html:17-25; debtor-web/src/handlers/auth.rs:102-108] — added the stable Groups heading target.
- [x] [Review][Patch] Do not overwrite Login recovery failures with `429` [debtor-web/src/handlers/auth.rs:85-100] — fixed by preserving server-error recovery responses.
- [x] [Review][Patch] Correct the contradictory proxy documentation header [.env.example:1-4] — corrected debug/non-debug configuration wording.

## Dev Notes

### Scope, Dependencies, And Explicit Exclusions

- Story 1.1 is the prerequisite for the validated canonical Argon2id v19 hash and two-verification gate. Reuse it; do not duplicate password parsing, KDF policy, or helper behavior.
- Stories 1.2 and 1.3 establish the persistent root composition, provider-independent startup, and lifecycle baseline. Preserve one root composition and one runtime path.
- Story 1.4 owns anonymous Login session/CSRF/token issuance, bounded anonymous pool, expiry, and cleanup. This story consumes and extends that store with Login reservation/terminal dispatch; do not create a second token store.
- Story 1.6 owns authenticated session refresh and Sign out. Story 1.7 owns the authenticated token pool and route-neutral extension to other unsafe forms. Do not require authenticated tokens on unrelated forms in this story.
- Story 1.8 owns final probe budgets, readiness/admission supervisor evidence, and complete timeout classification. Keep existing four-permit probes and supervised cleanup integration working, but do not claim all 1.8 outcomes.
- Story 1.9 owns authenticated real-socket shutdown evidence. Story 2.1 completes real-ledger mutation evidence for shared dispatch/lifecycle requirements. Do not implement Groups, Participants, Spendings, rates, debts, HTTPS edge rollout, database schema, migrations, monetary logic, or persistent sessions.
- Brownfield rule: remove bypasses, duplicate Login handlers, or superseded paths rather than retaining compatibility shims. Do not broaden scope to redesign unrelated authenticated forms.

### Current Files And Required Preservation

| Path | Current state | Story change and behavior to preserve |
| --- | --- | --- |
| `debtor-web/src/handlers/auth.rs` | Existing Login GET issues/reuses CSRF and anonymous submission token. POST validates CSRF/required fields, resolves proxy, calls application authentication, establishes session, and redirects, but does not validate/reserve the submitted token or share mutation dispatch. | Extend this handler only. Preserve authenticated Login redirect, safe responses, fresh recovery rendering, limiter reset after `session::establish`, and no password retention. Add token validation/reservation at the correct dispatch boundary. |
| `debtor-web/src/forms.rs` | `CsrfValidatedForm` loads session, decodes ordered form pairs, validates exactly one CSRF, and exposes an optional `MutationPreflight` dispatch marker. | Preserve ordering and rejection semantics. Adapt only as needed to expose the loaded session/preflight or combine token reservation with dispatch; never reserve before a later failing dispatch marker. |
| `debtor-web/src/submission_tokens.rs` | Process-local anonymous store, one token/session, ten-minute expiry, 4,096 capacity, indexed cleanup, atomic `reserve(session, token)`, terminal reserved state. | Reuse this owner. Keep session binding, lock-atomic reservation, indexed cleanup, sanitized errors, and anonymous/authenticated pool isolation. Add only the smallest route-facing/boundary API needed. |
| `debtor-web/src/session.rs` | Anonymous ten-minute and authenticated 30-day expiry; CSRF generation/matching; `establish` cycles ID, generates CSRF, sets auth, saves; `flush` removes state. | Preserve exact promotion order and failure semantics. Do not rotate/promote during Login GET or reset limiter before save succeeds. |
| `debtor-web/src/state.rs` | `AppState` owns one proxy resolver and one anonymous submission-token store. Proxy parser accepts empty direct-peer policy regardless of environment; resolver safely ignores untrusted headers and parses selected headers. | Add environment-aware startup validation without moving proxy mechanics inward. Keep only `IpAddr` crossing application ports and never log IP/header details. |
| `src/config.rs` | Validates password/cookie/name and stores trusted proxy strings; currently accepts empty proxy settings in all modes. | Add non-debug proxy-policy admission while retaining debug defaults and generic errors. Ensure validation occurs before DB work. |
| `src/composition.rs` | Parses proxy before DB connection, composes one limiter/password/session/token owner, and injects `AppState`. | Preserve ordering and singleton ownership. Do not create stores in handlers/tests that differ from production composition. |
| `debtor-web/src/router.rs` | Login has security headers, session layer, 8 KiB body limit, four-request limit, and 30-second timeout. Protected mutations have `MutationPreflight`. | Keep login bounds and route isolation. Add the shared pre-dispatch boundary to Login only if required for safe reservation, without putting probes/static routes behind sessions. |
| `debtor-web/src/middleware.rs` | `MutationPreflight` has one 30-second deadline and an irreversible `dispatch()` marker; Login timeout is a generic 30-second wrapper with Login-safe timeout mapping. | Prevent token reservation/dispatch split. After authentication dispatch, no generic timeout may cancel or misreport the outcome. |
| `debtor-application/src/authentication.rs` | `AuthenticationService::attempt` calls limiter then password verifier; `complete_login` resets limiter separately. | Keep this application boundary and sequencing. Do not call infra limiter/password types from web or duplicate policy. |
| `debtor-infra/src/auth/login_limiter.rs` | Five attempts/5 minutes/IP, 4,096 keys, indexed expiry, no active eviction, fail-closed unseen-key capacity, explicit reset. | Preserve exact policy and deterministic test clock patterns. |
| `debtor-infra/src/auth/password.rs` | Validated Argon2id gate uses `spawn_blocking` and global semaphore size two. | Reuse unchanged unless a test-only observability hook is essential; no KDF policy or dependency changes. |
| `debtor-web/templates/login.html`, `src/templates.rs`, `static/css/app.css` | Final native-first dark Access form already has one password, CSRF, submission token, stable status/heading IDs, and scoped Editorial Contrast styles. | Preserve field names, IDs, native action, no password value, status semantics, minimum targets, and authenticated CSS. Add only outcome/pending state needed for this story. |
| `src/runtime.rs` | Supervises session and submission-token cleanup through shared health/shutdown. | Do not add another cleanup worker or alter lifecycle ownership. |
| `.env.example`, `README.md` | Proxy comments currently describe optional CIDRs/direct peers. | Synchronize production-required trusted proxy policy and debug-only direct-peer fallback. |

### Implementation Guardrails

- The intended order is: outer body/concurrency/deadline admission -> session load -> exactly one CSRF -> exact Login fields -> trusted client resolution -> validate token -> atomically cross the one dispatch boundary/reserve token -> application `attempt` (limiter immediately before password verification) -> invalid/rate-safe response or durable `session::establish` -> only after durable promotion `complete_login` -> `303 /groups`.
- The token/preflight race is the highest-risk edge. Do not implement `reserve()` followed by a separately fallible `dispatch()` that can leave a token terminal without authentication dispatch. Make the boundary atomic from the request's perspective or make dispatch succeed before reservation while ensuring no second request can cross between those operations.
- A limiter rejection after token dispatch is still a terminal token use, but it must not invoke password verification. Interpret “one attempt immediately before every password verification” as no limiter reservation for any pre-verification rejection; the application service owns this sequencing.
- A correct password followed by session capacity or save failure is not a successful login: flush the anonymous session, emit no authenticated cookie/redirect, keep the token terminal, and leave limiter history uncleared.
- An invalid password must render a fresh form/token and never reuse the reserved token. `issue()` currently replaces a reserved token for the session; preserve that safe recovery behavior.
- Do not treat CSRF as replay protection. CSRF remains session-backed synchronizer validation; submission token reservation is separate and terminal after dispatch.
- Do not log `password`, hash, cookie, session ID, CSRF, submission token, forwarding headers, client IP, limiter key, query string, provider URL, or raw adapter diagnostics. Fixed event categories and counts only are allowed.
- Production configuration validation must happen before SQLite connection/migration and listener binding. Debug/local direct-peer fallback must not accidentally become production behavior.
- Preserve no-store headers and private-history prohibition. HTMX is optional enhancement; native HTML must work as the complete path and no custom JavaScript may be introduced.

### Architecture Compliance

- Preserve `root -> debtor-web/debtor-infra -> debtor-application -> debtor-domain`. Web owns HTTP extraction, trusted-proxy resolution, session/CSRF/submission-token mechanics, rendering, cookies, safe HTTP mapping, and Login UX. Application owns authentication orchestration and ports. Infra owns password and limiter adapters. Root owns configuration, concrete composition, startup, supervision, and lifecycle.
- No Axum, Askama, SQLx, tower-sessions, Argon2, reqwest, or concrete adapter types may cross application-owned ports. The authentication port accepts `IpAddr` and raw password text only; the password hash never leaves infra/config validation.
- Root composes exactly one session store, token store, trusted resolver, limiter, password gate, authentication service, and cleanup supervision path. No per-request/global duplicate owner.
- Use typed/sanitized errors. `anyhow` remains root-only; domain/application/adapter failures use existing typed categories. Do not expose adapter diagnostics.
- No database, migration, `.sqlx`, monetary, rate, or dependency changes are expected. If implementation changes checked SQL/migrations despite this boundary, run the full temporary-database/online SQLx prepare workflow and refresh committed metadata.
- Update `specs/design.md` first only if behavior is changed from its current normative contract; then synchronize ADR/config docs/tests as required. This story should implement the existing contract, not redefine it.

### Library And Framework Requirements

- Keep locked versions: Rust 1.97.1/edition 2024, Axum 0.8.9, Askama 0.16.0, Tokio 1.53.1, Tower 0.5.3, tower-http 0.6.11, tower-sessions 0.15.0, SQLx 0.9.0, Argon2 0.5.3, and current `Cargo.lock`. No dependency upgrade or frontend package manager.
- Axum guidance: compose route-specific middleware by building separate routers and merging them; use `Router::with_state` for the single `AppState`; keep probes/static routes outside session middleware. [Source: Context7 `/tokio-rs/axum`, “Applying Multiple Middleware with ServiceBuilder”, “Applying middleware to specific routes using Router::merge”, “Basic Router with Global State”]
- Tower Sessions guidance: cookie settings remain on `SessionManagerLayer`; expired records load as nonexistent; `Session::cycle_id()` deletes the old ID and must be followed by `save()` for durable rotation. [Source: Context7 `/maxcountryman/tower-sessions`, “SessionManagerLayer builder methods for cookie attributes”, “SessionStore::load trait docs say expired sessions return None”, “Session::cycle_id”]
- Keep the existing session layer settings: configured cookie name/path, HTTP-only, `SameSite::Strict`, debug/local versus non-debug Secure policy, always-save, and anonymous expiry.
- If HTMX is shipped/updated, use only exact self-hosted HTMX 2.0.10 and official `response-targets` 2.0.4 bytes with fixed routes, immutable digest mapping, JavaScript media type, and `nosniff`. Do not make optional assets or a frontend build a prerequisite for Login.

### Testing Requirements

- Application tests: retain fake `PasswordVerifier`/`LoginAttemptLimiter` tests proving limiter-before-verifier, correct/incorrect attempt accounting, and reset only after completion. Use injected test clocks and barriers rather than sleeps.
- Infra tests: keep `MemoryLoginAttemptLimiter` boundary tests for five attempts, 5-minute expiry, 4,096 capacity/no eviction, existing-key behavior, and indexed cleanup. Add only missing Story 1.5 evidence.
- Web tests: extend `TestState` with atomic counters, client capture, verifier barriers, promotion/save failure controls, and token-state assertions using simple `Mutex`, atomics, `Notify`, or `Barrier`; no mocking framework.
- Root/config tests: prove production proxy-policy rejection precedes database side effects and debug direct-peer defaults remain valid. Never include secrets or raw config in assertion output.
- Composed/router tests: verify Login body limit, concurrency/deadline behavior, strict pre-dispatch ordering, `409` token conflicts, `429` limiter responses with safe `Retry-After`, `503` promotion failures without authenticated cookie, successful rotated cookie/redirect, and authenticated Login downgrade prevention.
- HTML/UX tests: exact one CSRF and one submission token, distinct opaque values, native `/login` action, stable `sign-in-heading`/`login-status`, `role=status`, polite atomic status, `aria-busy`, no password retention, exact security headers, no custom scripts, native/enhanced parity, and the six required UX IDs. Browser geometry at 320px/400% must be explicitly tested if a harness exists; otherwise record manual evidence honestly.
- Required validation from repository root:

```bash
cargo fmt --all -- --check
cargo run --bin architecture-check --locked
cargo check --workspace --all-features --locked
SQLX_OFFLINE=true cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-features --locked
```

- Run `cargo deny check` only if manifests/lockfiles/policy change. Validate `tools/password-hash` separately only if touched. Never use `cargo build --release`.

### Previous Story Intelligence

- Story 1.1 established the shared infra-owned bounded canonical Argon2id policy, root pre-side-effect validation, generic startup errors, and secret-safe tests. Do not duplicate or weaken it.
- Story 1.2 established root-only persistent startup/composition and provider-independent admission. Preserve configuration-before-DB and one composed runtime.
- Story 1.3 established one shutdown coordinator, supervised cleanup, same-path local lifecycle, WAL-safe behavior, and deterministic barrier/held-resource tests. Do not create a second lifecycle path.
- Story 1.4 established the final anonymous Login page, CSRF, one-per-session anonymous token issuance, indexed cleanup, native-first HTML, exact headers, scoped CSS, and token/session capacity isolation. Its explicit handoff assigns token reservation, trusted proxy admission, limiter/password verification, and durable promotion to this story.
- Prior review fixes are important: authenticated Login must redirect without downgrade; failed token issuance must not leave an orphan; Login failure/timeout/capacity must use the canonical recovery/status contract; heading focus and dark CSS are scoped. Preserve these regressions.
- Existing tests use temporary file databases, direct router/socket requests, fake application services, explicit session stores, atomics, and barriers/notifications. Extend those patterns.

### Git Intelligence

- Recent commits are predominantly BMAD planning/story artifacts. The latest implementation snapshot is represented by Story 1.4 and commit `18f00e4`; inspect current files rather than relying on commit titles.
- The worktree was clean at analysis time. If another actor changes files during implementation, preserve unrelated changes and reconcile affected Login/auth files instead of reverting them.

### Latest Technical Information

- Current Context7 Axum 0.8 documentation supports route-specific middleware through separate routers plus `Router::merge`, `ServiceBuilder` for ordered layers, and `Router::with_state` for state extraction. This validates the existing public/login/protected split; do not globally apply sessions or Login limits to probes/static routes.
- Current Context7 Tower Sessions documentation confirms `SessionManagerLayer` cookie builder settings, `SessionStore::load` treating expired records as absent, and `Session::cycle_id` requiring a subsequent `save()` for durable fixation-resistant rotation. Use these existing semantics rather than adding a custom session implementation.
- No current library change requires a dependency update. `Cargo.lock` is authoritative and the story must not introduce HTMX/frontend build dependencies.

### Project Structure Notes

- Repository root: `/home/sebr/projects/pet/debtor`. Planning authority: `_bmad-output/`; implementation artifact output: `_bmad-output/implementation-artifacts/`.
- Expected updates are limited to Login/auth web code, proxy/config docs/code, root composition/config tests, application/infra auth tests, and synchronized operator documentation. Likely paths: `debtor-web/src/handlers/auth.rs`, `forms.rs`, `middleware.rs`, `submission_tokens.rs`, `state.rs`, `router.rs`, `handlers/response.rs`, `handlers/test_support.rs`, `src/session.rs`, `src/templates.rs`, `templates/login.html`, `static/css/app.css`, `src/config.rs`, `src/composition.rs`, `src/main.rs` tests, `debtor-application/src/authentication.rs` tests, `debtor-infra/src/auth/login_limiter.rs` tests, `debtor-infra/src/auth/password.rs` only if needed, `.env.example`, and `README.md`.
- No new domain modules, migrations, `.sqlx` metadata, persistent auth schema, user table, registration path, participant authentication, or multi-user model.

### References

- [Source: `_bmad-output/planning-artifacts/epics.md#Story 1.5: Sign In with Bounded Password Verification`]
- [Source: `_bmad-output/planning-artifacts/epics.md#Assignment Packets`]
- [Source: `_bmad-output/planning-artifacts/epics.md#Cross-Cutting Story Rule`]
- [Source: `specs/design.md#Security`]
- [Source: `specs/design.md#Operational Limits`]
- [Source: `specs/design.md#Local Run Contract`]
- [Source: `specs/adr/0001-foundation-architecture.md#7. Bounded login limiting`]
- [Source: `specs/adr/0001-foundation-architecture.md#10. Single unsafe-request admission boundary`]
- [Source: `specs/adr/0001-foundation-architecture.md#13. Native-first self-hosted HTMX enhancement`]
- [Source: `_bmad-output/project-context.md#Framework-Specific Rules`]
- [Source: `_bmad-output/project-context.md#Testing Rules`]
- [Source: `_bmad-output/planning-artifacts/prds/prd-debtor-2026-08-10/addendum.md#Authentication, Sessions, And CSRF`]
- [Source: `_bmad-output/planning-artifacts/prds/prd-debtor-2026-08-10/addendum.md#Headers, Proxy Trust, And Session-Free Routes`]
- [Source: `_bmad-output/planning-artifacts/prds/prd-debtor-2026-08-10/addendum.md#Admission, Timeouts, Probes, And Shutdown`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-10 - Shared unsafe-request admission boundary`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-12 - Single-process edge topology`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-13 - Process-local owner uniqueness`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-14 - Bounded operational envelope`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-15 - Safe failures, logging, and readiness`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md#Stable UX Contract Registry`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md#Lifecycle and Access`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md#HTTP and Session Outcomes`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md#Interaction Focus Matrix`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/DESIGN.md#Access form`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/DESIGN.md#Components`]
- [Source: `_bmad-output/implementation-artifacts/1-1-prepare-and-validate-the-administrator-password.md`]
- [Source: `_bmad-output/implementation-artifacts/1-2-start-a-persistent-local-application.md`]
- [Source: `_bmad-output/implementation-artifacts/1-3-restart-and-validate-the-composed-local-application.md`]
- [Source: `_bmad-output/implementation-artifacts/1-4-open-a-protected-and-accessible-login-page.md`]
- [Source: `debtor-web/src/handlers/auth.rs`]
- [Source: `debtor-web/src/forms.rs`]
- [Source: `debtor-web/src/submission_tokens.rs`]
- [Source: `debtor-web/src/session.rs`]
- [Source: `debtor-web/src/state.rs`]
- [Source: `debtor-web/src/middleware.rs`]
- [Source: `debtor-web/src/router.rs`]
- [Source: `debtor-application/src/authentication.rs`]
- [Source: `debtor-infra/src/auth/login_limiter.rs`]
- [Source: `debtor-infra/src/auth/password.rs`]
- [Source: `src/config.rs`]
- [Source: `src/composition.rs`]
- [Source: `src/runtime.rs`]
- [Source: Context7 `/tokio-rs/axum`, Axum 0.8 router/state/middleware composition]
- [Source: Context7 `/maxcountryman/tower-sessions`, tower-sessions session/cookie/expiry/cycle semantics]

## Dev Agent Record

### Agent Model Used

openai/gpt-5.6-luna

### Debug Log References

- Ultimate context engine analysis completed - comprehensive developer guide created.
- Story selected from the first `backlog` entry in the complete `sprint-status.yaml` order: `1-5-sign-in-with-bounded-password-verification`.
- Normative epics, PRD/addendum, `specs/design.md`, architecture spine, UX contracts, project context, prior Stories 1.1-1.4, recent source files, and recent commit patterns were analyzed.
- Current implementation gaps recorded explicitly: production/debug proxy policy is not environment-aware; Login token is structurally required but not reserved; Login is not currently attached to `MutationPreflight`; promotion and limiter semantics exist but need complete Story 1.5 evidence.
- Context7 consulted for Axum 0.8 route/state/middleware composition and tower-sessions 0.15 cookie, expiry, save, and `cycle_id` behavior. No dependency upgrade is authorized.

### Implementation Plan

- Reused the existing root-composed proxy resolver, authentication service, limiter, Argon2 gate, session store, and anonymous token store.
- Added environment-aware production proxy admission and synchronized operator configuration guidance.
- Added a lock-held token `reserve_and_dispatch` boundary so rejected preflight leaves tokens reusable while crossed dispatch is terminal.
- Added Login-safe fresh recovery for token conflicts, invalid passwords, rate limiting, and authentication availability failures.
- Added focused web/config/token tests and ran the complete locked workspace validation and architecture fitness checks.

### Completion Notes List

- Ultimate context engine analysis completed - comprehensive developer guide created.
- Story status set to `review` after implementation and validation.
- Implemented production trusted-proxy admission, strict Login token validation, atomic token reservation/dispatch, sanitized recovery, and deterministic regression coverage.
- Reused existing durable session promotion and bounded authentication/limiter/Argon2 adapters; no database, migration, dependency, or application-port changes were required.
- Browser-level geometry evidence remains manual because the repository has no executable browser harness; native server-rendered Login behavior and web-level contract assertions pass.
- All eight review patch findings were resolved and checked off in `Review Findings`.

### Validation

- `cargo fmt --all -- --check`
- `cargo run --bin architecture-check --locked`
- `cargo check --workspace --all-features --locked`
- `SQLX_OFFLINE=true cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`
- `cargo test --workspace --all-features --locked` (all tests passed; 52 web tests)
- `cargo test -p debtor-web --lib` (54 web tests passed after review fixes)

### File List

- `_bmad-output/implementation-artifacts/1-5-sign-in-with-bounded-password-verification.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `.env.example`
- `README.md`
- `src/config.rs`
- `src/composition.rs`
- `debtor-web/src/state.rs`
- `debtor-web/src/submission_tokens.rs`
- `debtor-web/src/handlers/auth.rs`
- `debtor-web/src/handlers/response.rs`
- `debtor-web/src/handlers/test_support.rs`
- `debtor-web/src/middleware.rs`
- `debtor-web/src/router.rs`
- `debtor-web/src/forms.rs`
- `debtor-web/src/session.rs`
- `debtor-web/templates/groups.html`
- `debtor-web/templates/login.html`
- `static/css/app.css`
- `static/htmx.min.js`

### Change Log

- 2026-08-12: Implemented Story 1.5 trusted-client admission, strict Login token dispatch, sanitized recovery, and regression tests; marked story `review`.
- 2026-08-12: Addressed all eight code-review findings, added declarative HTMX pending behavior and additional regression tests, and marked story `done`.
