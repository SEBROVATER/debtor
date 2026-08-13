---
baseline_commit: 18f00e4
---

# Story 1.6: Maintain an Authenticated Session and Sign Out

Status: done

## Story

As the administrator,
I want authenticated access to persist safely until I sign out or the session expires,
so that I can use Debtor without repeated login while retaining reliable revocation.

## Acceptance Criteria

1. **Given** login promotion completed durably
   **When** the administrator follows the `303` redirect
   **Then** Debtor renders an authenticated no-store home shell using the rotated process-local session and CSRF token
   **And** the response exposes no session identifier or security token except the protection values required by rendered unsafe forms.

2. **Given** an anonymous, expired, flushed, or otherwise invalid session requests any protected ledger route
   **When** authentication middleware evaluates it
   **Then** access is denied before any ledger use case is invoked
   **And** native navigation redirects to `/login` with sanitized handling and no ledger content.

3. **Given** a valid authenticated session is used
   **When** an authenticated request completes
   **Then** its 30-day inactivity expiry is refreshed and durably persisted through the indexed session store
   **And** refresh does not create a 33rd authenticated session or evict another authenticated session.

4. **Given** the administrator opens an authenticated page
   **When** the page is rendered without HTMX
   **Then** it uses the shared semantic responsive shell, exact authenticated security headers, and a protected Sign out form
   **And** the Sign out form contains exactly one current session-backed CSRF token and one distinct session-bound single-use submission token.

5. **Given** a valid authenticated session submits Sign out with exactly one valid CSRF token and one valid Sign out submission token
   **When** the request crosses the dispatch boundary
   **Then** the submission token is atomically reserved before session flush, the server-side session is flushed, the browser cookie is expired, and the response redirects with `303 See Other` to `/login`
   **And** the reserved token remains terminal if flush or response delivery fails.

6. **Given** Sign out form structure, authentication, CSRF, or submission-token validation fails before dispatch
   **When** the request is rejected
   **Then** the session remains authenticated, no session flush occurs, and no guarded side effect is invoked
   **And** a valid token remains usable when rejection occurred before reservation.

7. **Given** the same Sign out token is replayed or presented concurrently by more than one request
   **When** token reservation is attempted
   **Then** exactly one request can reserve and flush the session
   **And** every losing or later request receives sanitized `409 Conflict` without a second flush or dispatch.

8. **Given** an authenticated session reaches 30 days of inactivity
   **When** indexed expiry cleanup runs or the session is next presented
   **Then** the record is physically deleted and protected access is denied
   **And** the expired record no longer consumes authenticated capacity.

9. **Given** Debtor restarts
   **When** a previously issued anonymous or authenticated cookie is presented to the new process
   **Then** no corresponding process-local session exists, the administrator is logged out, and protected access is denied
   **And** restart does not restore authentication state from SQLite.

10. **Given** the shared authenticated shell is rendered at 320 CSS pixels, 400% zoom, or a wide composition
    **When** the administrator operates it without a pointer
    **Then** header content remains in reading order, every control including Sign out is at least 48 by 48 CSS pixels, no private content is clipped or page-level horizontally scrolled, and wide layout preserves DOM/focus order
    **And** Editorial Contrast tokens, square controls, double-rule hierarchy, focus geometry, and motion prohibitions satisfy `UX-SHELL-01`, `UX-TARGET-01`, `UX-RESPONSIVE-01`, and `UX-VISUAL-01`.

11. **Given** Sign out is activated
    **When** the protected request is pending
    **Then** the initiator becomes unavailable, repeated activation is suppressed or coalesced, the owning header/form region exposes `aria-busy`, and one stable polite atomic status reports pending or failure without moving focus
    **And** native form submission remains authoritative.

12. **Given** logout commits successfully
    **When** the `303` Sign in page renders
    **Then** the stable `Sign in` heading is the single forward-focus destination, authenticated history exposes no cached ledger page, and no private HTMX history snapshot is restored
    **And** a failure focuses the Sign out control or scoped status according to `UX-FOCUS-01` and `UX-STATUS-01`.

**Requirements:** `SPEC-FR1`, `SPEC-FR7`, `SPEC-FR9..SPEC-FR11`, `SPEC-FR14..SPEC-FR16`, `SPEC-FR18..SPEC-FR19`, `SPEC-FR88`, `SPEC-FR90..SPEC-FR95`; `SPEC-NFR19..SPEC-NFR23`, `SPEC-NFR25`, `SPEC-NFR28..SPEC-NFR30`; authenticated access, sliding expiry refresh, process-local restart invalidation, protected Sign out, terminal Sign out replay protection, safe diagnostics, and shared web-policy requirements; UX contracts `UX-SHELL-01`, `UX-TARGET-01`, `UX-FOCUS-01`, `UX-STATUS-01`, `UX-RESPONSIVE-01`, `UX-VISUAL-01`.

### Scope Boundary

This story is the first authenticated consumer of the existing submission-token reservation primitive, but it is not the general authenticated replay-protection story. Implement the smallest Sign out-only authenticated token issuance/reservation path needed to satisfy the criteria above. Story `1.7` owns the general authenticated pool of 1,024 tokens, 32-per-session/30-minute limits, page-scoped tokens, non-Login route-neutral extraction, cross-route races, and applying tokens to all other authenticated forms. Do not make `1.7` a prerequisite for this runnable Sign out outcome.

## Tasks / Subtasks

- [x] Establish authenticated session refresh and protected-route behavior (AC: 1-3, 8-9)
  - [x] Preserve `session::authenticated_expiry()` as 30-day sliding inactivity expiry and ensure the authenticated middleware refresh is saved by the composed `SessionManagerLayer`.
  - [x] Keep authentication middleware before protected handlers and preserve the existing defense-in-depth handler checks without introducing a second authentication model.
  - [x] Ensure anonymous, expired, flushed, and restart-invalidated sessions cannot invoke any ledger use case or mint authenticated protection values.
  - [x] Prove indexed physical expiry removal, capacity reuse, and process restart invalidation with deterministic store/composition tests.

- [x] Add the minimal protected Sign out token path (AC: 4-7)
  - [x] Extend the existing web-owned `AnonymousSubmissionTokenStore` reservation boundary only as necessary for a Sign out token bound to the authenticated session; do not create a second store or general route-local replay guard.
  - [x] Issue exactly one opaque Sign out token for the rendered Sign out form and keep it distinct from CSRF. Never log or expose token/session identifiers outside the required hidden field.
  - [x] Validate exact field structure, exactly one CSRF, authenticated session state, and the Sign out token before reservation. Missing, malformed, duplicate, expired, session-mismatched, reserved, or consumed token returns sanitized `409` where applicable and never flushes.
  - [x] Reserve the token atomically immediately before `session::flush`; do not call `reserve()` and then perform a separately fallible dispatch marker that can consume a token without dispatch.
  - [x] Treat reservation as terminal after dispatch regardless of flush failure, task failure, or response delivery. Do not automatically retry or re-enable a stale form as if logout were known to be undone.
  - [x] On successful flush, return `303 See Other` to `/login` and preserve the configured cookie name/path, `HttpOnly`, `SameSite=Strict`, and secure-cookie policy so the browser receives cookie-expiration semantics from tower-sessions.

- [x] Extend authenticated shell rendering without duplicating policy (AC: 4, 10-12)
  - [x] Add the Sign out form and stable status/`aria-busy` contract to every authenticated page/header that is in this story's shell scope; use native `POST /logout` as the authoritative path.
  - [x] Prefer a small Askama partial/view projection or shared rendering helper rather than copying divergent Sign out markup into every template. Preserve each page's existing route-specific content and current authenticated behavior.
  - [x] Ensure every Sign out form receives the current CSRF and the Sign out-only token. Do not add authenticated submission tokens to Group, Participant, Spending, confirmation, restore, or other forms; those are Story `1.7` consumers.
  - [x] Keep `Cache-Control: no-store`, `X-Content-Type-Options: nosniff`, `Referrer-Policy: no-referrer`, and the exact restrictive CSP on authenticated HTML and logout outcomes. Do not enable private HTMX history snapshots.
  - [x] Implement pending state declaratively with existing pinned self-hosted HTMX only if the response/asset contract is already complete; no custom JavaScript, inline scripts, custom extensions, CDN, animation, or client-only logout behavior.
  - [x] Apply Editorial Contrast consistently to the authenticated shell without regressing the completed Login surface: charcoal canvas, warm paper text, ruled sections, square controls, high-contrast focus, minimum 48px targets, and no decorative depth or transitions.

- [x] Add invariant-owning tests (AC: 1-12)
  - [x] Web tests cover valid Sign out, `303 /login`, expired cookie/session behavior, physical server-side flush, cookie deletion, and protected access denial after logout.
  - [x] Web tests cover missing, duplicate, malformed, wrong, expired, reserved, consumed, and session-mismatched Sign out protection; assert the session remains authenticated and no flush/use-case side effect occurs before dispatch.
  - [x] Use barriers, notifications, or a deliberately held session-store operation to prove concurrent replay has exactly one winner and no second flush. Do not use timing sleeps as proof.
  - [x] Test authenticated expiry refresh with an injected clock/store and verify persistence/index updates without creating or evicting sessions at the 32-session boundary.
  - [x] Extend the composed/root real-socket smoke coverage to authenticate, read a protected page, sign out, verify the cookie/session is invalid, and confirm restart invalidates both anonymous and authenticated cookies while SQLite data remains.
  - [x] Assert exact authenticated security headers, no session/CSRF/submission identifiers in rendered content except required form fields, no password retention, native `/logout` action, stable heading/status IDs, `role="status"`, `aria-live="polite"`, `aria-atomic="true"`, and `aria-busy` behavior.
  - [x] Add rendered-contract evidence for 320px/400% zoom, keyboard operation, target geometry, no page-level horizontal scrolling, wide normal-flow adaptation, and native/enhanced parity. If no executable browser harness exists, record manual evidence honestly rather than claiming automated geometry coverage.

### Review Findings

- [x] [Review][Patch] Concurrent authenticated reads can resurrect a logged-out session [debtor-web/src/middleware.rs:98-100; debtor-web/src/session_store.rs:189-213] — Fixed by rejecting stale saves for deleted session IDs and adding `stale_save_cannot_resurrect_a_deleted_session`.
- [x] [Review][Patch] A reserved Sign out token can be replaced by a concurrent page render [debtor-web/src/submission_tokens.rs:243-264] — Fixed by making reserved Sign out tokens terminal and adding the reserved-token assertion.
- [x] [Review][Patch] Sign out pending and failure states do not satisfy the accessibility contract [debtor-web/templates/groups.html:11-21] — Fixed with stable status targets, official response-targets enhancement, request indicators, and shell markup coverage.
- [x] [Review][Patch] Authenticated keyboard focus uses an undefined CSS variable [static/css/app.css:52] — Fixed by defining the authenticated Editorial Contrast focus token.
- [x] [Review][Patch] Authenticated shell does not implement the required Editorial Contrast and target geometry [static/css/app.css:1-52] — Fixed by applying dark Editorial Contrast, square controls, and 48px minimum control geometry to the shared shell.
- [x] [Review][Patch] Sign-out token capacity exhaustion is mapped to a generic 500 session error [debtor-web/src/handlers/auth.rs:248-260] — Fixed with sanitized retryable session-unavailable feedback.
- [x] [Review][Patch] Authenticated HTMX assets omit the integrity contract used by Login [debtor-web/templates/groups.html:7-8] — Fixed by adding SRI/crossorigin attributes and the pinned official response-targets asset to authenticated templates.
- [x] [Review][Patch] HTTP-level concurrent replay is not tested deterministically [debtor-web/src/router.rs:424-467] — Fixed with a barrier-based concurrent request test asserting one success and one conflict.
- [x] [Review][Patch] Flush-failure terminal behavior is not tested [debtor-web/src/handlers/auth.rs:198-201; debtor-web/src/router.rs:313-511] — Fixed by making deletion precede local flush and covering stale-save/deletion behavior.
- [x] [Review][Patch] Restart invalidation coverage is overclaimed [tests/restart.rs:106-138; src/main.rs:358-430] — Fixed by extending the real-socket smoke test to replay anonymous and authenticated cookies after restart.
- [x] [Review][Patch] Flush failure can leave the old authenticated record usable [debtor-web/src/handlers/auth.rs:198-201; tower-sessions 0.15 service semantics] — Fixed by deleting server-side state before clearing the session locally.
- [x] [Review][Patch] Sliding authenticated expiry is not persisted for 5xx responses [debtor-web/src/middleware.rs:98-100; tower-sessions 0.15 service semantics] — Fixed by explicitly saving authenticated sessions after protected requests.

## Dev Notes

### Developer Context

- The previous completed stories establish the root-composed runtime, process-local session store, anonymous Login/CSRF/token issuance, strict Login admission, atomic Login token reservation, bounded password verification, limiter, trusted-proxy policy, durable session promotion, and Login recovery UX. Reuse these paths; do not duplicate them.
- The current implementation already has `authenticated_expiry()` and `require_authenticated`, and the protected router applies session middleware plus `security_headers`. The missing first-consumer behavior is authenticated Sign out token issuance/reservation, session flush/cookie expiry proof, shell-wide Sign out UX, and explicit refresh/restart tests.
- Current `logout` validates only CSRF, calls `form.dispatch()`, flushes the session, and redirects. It must be changed so token validation and atomic reservation occur before the flush dispatch boundary.
- Current authenticated templates mostly expose only CSRF and duplicate their own headers. `partials.rs` is a placeholder. Consolidate only the shell/Sign out concern needed here; do not redesign future ledger surfaces or add a separate application shell architecture prematurely.

### Architecture Compliance

- Preserve `root -> debtor-web/debtor-infra -> debtor-application -> debtor-domain`.
- Session, CSRF, submission-token mechanics, Askama rendering, cookies, strict form extraction, and sanitized HTTP mapping belong in `debtor-web`. Root owns concrete session/token composition and cleanup/lifecycle supervision. No tower-sessions, Axum, or web token types may cross application-owned ports.
- Keep exactly one process-local session store and one web-owned token store. Restart must invalidate all session/token state by reconstructing process-local owners; SQLite must not become a session store.
- Keep `anyhow` at root orchestration only. Preserve fixed safe response categories; never expose raw session-store, cookie, token, SQLx, or runtime errors.
- No domain, application financial, exchange-rate, migration, `.sqlx`, dependency, or database schema changes are expected. If checked SQL or migrations are changed accidentally, stop and either remove the change or run the full temporary migration and online SQLx prepare workflow.
- Preserve last-committed-write semantics and do not add revisions, stale-edit conflicts, persistent sessions, usernames, registration, participant authentication, or multi-user authorization.

### Current Files And Required Preservation

| Path | Current state | Required treatment |
| --- | --- | --- |
| `debtor-web/src/session.rs` | Defines 10-minute anonymous and 30-day authenticated inactivity expiry, CSRF access, durable Login rotation, and `flush`. | Reuse the existing expiry/flush semantics. Add only a narrowly reusable refresh or test helper if needed. Do not rotate during ordinary requests or Sign out. |
| `debtor-web/src/session_store.rs` | Process-local bounded store with 4,096 anonymous and 32 authenticated records, indexed expiry, no eviction, and physical deletion. | Preserve count isolation, indexed cleanup, save/load expiry behavior, and restart invalidation. Test refresh at capacity without creating a new record. |
| `debtor-web/src/submission_tokens.rs` | Anonymous-only store with UUID tokens, one/session, indexed 10-minute expiry, and atomic `reserve_and_dispatch`. | Generalize minimally for Sign out or add a clearly bounded Sign out-only capability. Preserve lock-held atomicity, session binding, terminal reservation, and sanitized errors. Do not implement Story 1.7's general authenticated pool here. |
| `debtor-web/src/forms.rs` | `CsrfValidatedForm` loads the session, decodes ordered form pairs, validates exactly one CSRF, and exposes `MutationPreflight`. | Preserve body/deadline/session/CSRF ordering. Extend only the Sign out path or a minimal helper; do not turn this into the Story 1.7 route-neutral extractor. |
| `debtor-web/src/handlers/auth.rs` | Login uses `reserve_and_dispatch`; logout currently requires only CSRF, marks dispatch, flushes, and redirects. | Make logout exact-field/token protected and reserve immediately before flush. Preserve authenticated Login redirect, safe errors, and no private content after logout. |
| `debtor-web/src/middleware.rs` | `require_authenticated` checks session before protected handlers and sets authenticated expiry; security headers apply to protected HTML; preflight marks unsafe requests. | Preserve ordering, refresh semantics, no generic post-dispatch cancellation, and exact headers. Ensure anonymous protected requests cannot reach handlers/use cases. |
| `debtor-web/src/router.rs` | `/logout` is a protected POST; protected routes use security headers, auth, sessions, mutation preflight, safe read timeout, body limit, and user concurrency. | Keep `/logout` protected and within existing 256KiB/body and pre-dispatch boundaries. Keep probes/static assets outside sessions. |
| `debtor-web/templates/*.html` | Authenticated pages have duplicated basic headers; only Groups currently renders Log out; unsafe forms generally have CSRF only. | Add a consistent Sign out form/status to the authenticated shell surfaces required by the story while preserving route-specific content. Do not add general authenticated tokens to later forms. |
| `debtor-web/src/templates.rs` / `templates/partials.rs` | View models are page-specific; partial module is empty. | Add the smallest shared projection/helper needed to avoid divergent Sign out markup and carry CSRF plus the Sign out token. |
| `static/css/app.css` | Generic light authenticated styling; dark Editorial Contrast is scoped to Login. | Extend styles carefully to the authenticated shell and Sign out states. Avoid breaking existing Login CSS and do not redesign future allocation/debt UI beyond this shell contract. |
| `src/composition.rs` | Builds one session store and one anonymous token store, injects web state, and returns both stores in `BuiltApp`. | Preserve singleton ownership. Carry any generalized token store through `BuiltApp` only for existing runtime supervision; do not add per-request stores. |
| `src/runtime.rs` | Supervises session/token cleanup and shared cleanup health; shutdown currently stops supervisors before checkpoint/pool close. | Preserve one supervisor path and health signaling. Do not claim Story 1.8's final admission/readiness packet or Story 1.9's authenticated mutation shutdown evidence. |
| `tests/restart.rs` / `src/main.rs` tests | Existing process/composition tests prove database restart, real-socket Login/authenticated read, and shutdown. | Extend with session invalidation/sign-out evidence without logging secrets or relying on SQLite for sessions. Keep cleanup deterministic and bounded. |

### Technical Guardrails

- Required request order for Sign out: body/concurrency/deadline admission -> session load -> authenticated check -> ordered structural extraction -> exactly one CSRF validation -> exact Sign out fields/token validation -> atomic token reservation plus `MutationPreflight::dispatch()` immediately before `session::flush()` -> definitive flush response -> `303 /login`.
- Do not reserve a token before route structure/CSRF/authentication validation. Do not mark dispatch separately after token reservation. The reservation boundary must not leave a terminal token when the later dispatch marker can still fail.
- A valid pre-dispatch rejection preserves the token and the authenticated session. After dispatch, token consumption is terminal even if flush fails; report a safe failure and never claim that logout definitely succeeded unless the session flush result is known.
- `session::flush()` must remove the server-side record and clear session state. The response must use tower-sessions' configured cookie policy so the browser receives the normal expired/deletion cookie; do not invent a parallel cookie implementation.
- Authenticated expiry is sliding inactivity, not a 30-day absolute lifetime. Updating expiry on an existing session must save the same record and update its expiry index; it must not call `cycle_id`, create a second record, or evict another authenticated session.
- Session ID and CSRF rotate only on successful Login promotion. Sign out does not rotate or persist a new authenticated session; it flushes the current one.
- Protected-route denial must occur before route-specific use-case calls, including protected POST requests. Anonymous requests must not mint authenticated Sign out tokens or expose ledger content.
- Do not log credentials, hashes, cookies, session IDs, CSRF values, submission tokens, client IPs, limiter keys, raw headers, query strings, or adapter diagnostics. Use fixed operation/reason categories only.
- Native HTML is authoritative. HTMX may disable the Sign out initiator and target a stable status region only through declarative pinned assets already present. No custom JS, inline scripts, inline event attributes, custom extensions, CDN, or private history snapshots.

### Library And Framework Requirements

- Keep the locked stack: Rust 1.97.1/edition 2024, Axum 0.8.9, Askama 0.16.0, Tokio 1.53.1, Tower 0.5.3, tower-http 0.6.11, tower-sessions 0.15.0, and the current `Cargo.lock`. No dependency upgrade or frontend build is authorized.
- Tower Sessions `SessionManagerLayer` owns cookie attributes and always-save behavior. `Session::flush()` is the authoritative logout operation; use the configured layer rather than manually setting a replacement cookie. Existing `Session::cycle_id()`/`save()` semantics remain Login-only.
- Axum route-specific middleware should remain composed through separate routers merged into the final router. Keep probes/static routes outside session middleware, and preserve the existing `Router::with_state`/`router_with_sessions` composition.
- Context7 references consulted: `/maxcountryman/tower-sessions` for `SessionManagerLayer`, expiry, `Session::flush`, `SessionStore`, and `cycle_id`; `/tokio-rs/axum` for route-specific middleware, `ServiceBuilder`, router merge, and state composition.

### UX Requirements

- Apply `UX-SHELL-01`, `UX-TARGET-01`, `UX-FOCUS-01`, `UX-STATUS-01`, `UX-RESPONSIVE-01`, and `UX-VISUAL-01` as stable test references.
- Sign out appears in the Page header in normal flow, not as a fixed overlay or hidden menu. It is a protected native form and at least a 48-by-48 target.
- Authenticated pages use one responsive DOM from 320px through wide layouts. Preserve reading/focus order, safe-area behavior where the shell uses it, no page-level horizontal scroll, and no clipped private content at 400% zoom.
- Use dark Editorial Contrast: `#101113` canvas, `#F5F0E7` primary text, `#AAA59C` muted text, `#6D6C69` rules, `#F0D36C` primary action, `#211C08` action text, `#E88467` warning rule, `#F4BAA7` warning text, square geometry, and no decorative depth or authored transitions.
- The Sign out form owner exposes `aria-busy` while pending. One stable `role="status"`, `aria-live="polite"`, `aria-atomic="true"` node announces pending/failure once. Pending/error retains the initiator; successful forward navigation focuses `sign-in-heading`.
- Session expiry/restart returns to Sign in without ledger content and without claiming unsaved-form recovery. Logout errors state only safe outcome information.

### Testing Requirements

- Keep store/session tests at `debtor-web` for indexed expiry, count isolation, refresh persistence, flush deletion, capacity behavior, token binding, and terminal reservation.
- Keep web route tests in `debtor-web/src/router.rs` or the owning handler test module using fake application services, atomic counters, barriers, `Notify`, and temporary stores. Assert no application dispatch on invalid protected requests.
- Add a composition/root test for restart invalidation and a real-socket flow for Login -> protected read -> Sign out -> denied protected read. Do not use real secrets in fixtures or output.
- Verify exact statuses/headers: protected anonymous redirect to `/login`; valid logout `303` with `/login` location and cookie expiration; invalid/replayed token `409`; malformed structure/CSRF rejection before flush; safe session/storage failures without raw diagnostics.
- Verify HTML contracts: one Sign out CSRF, one Sign out token, distinct opaque values, native `POST /logout`, stable shell heading/status IDs, `aria-busy`, no session/security values except required hidden fields, no custom scripts, no private HTMX history.
- Browser geometry evidence is manual unless an executable harness is introduced. Never report automated 320px/400% geometry or cross-browser parity as passed without an actual browser test.

### Project Structure Notes

- Planning authority is `_bmad-output/`; implementation artifact is this file under `_bmad-output/implementation-artifacts/`.
- Expected implementation is primarily in `debtor-web` handlers/forms/session/token/store/templates/CSS, with narrowly scoped root composition/runtime/test updates. No domain, application financial, migration, `.sqlx`, provider, or dependency work should be necessary.
- Preserve plural feature naming and existing `*Store`, `*UseCases`, `*Template`, `*View` conventions. Keep shared shell logic local until reuse is real; avoid speculative abstractions.
- Brownfield disposition: remove the current unprotected/CSRF-only logout path and divergent shell markup when replaced. Do not retain parallel logout routes, persistent-session paths, alternate auth shells, route-local replay guards, or compatibility shims.

### Previous Story Intelligence

- Story 1.4 established the final native-first Login page, anonymous session/CSRF issuance, one-per-session anonymous token store, indexed cleanup, exact security headers, and dark access styling. Preserve its Login behavior while extending the authenticated shell.
- Story 1.5 established the atomic token reservation/dispatch boundary, safe Login recovery, trusted proxy validation, limiter/password gate, and durable Login promotion. Reuse `reserve_and_dispatch` semantics and do not regress authenticated Login redirect or session rotation.
- Prior review fixes specifically require authenticated `/login` not to downgrade sessions, failed session/token issuance not to leave orphans, exact status/heading focus contracts, scoped dark CSS, and no password retention. Add Sign out without undoing these fixes.
- Existing tests use direct router requests, explicit `ReapingMemoryStore`, fake use cases, atomics, and barriers/notifications. Extend those patterns rather than introducing a mocking framework or timing-based concurrency assertions.

### Git Intelligence

- Recent repository commits are predominantly BMAD planning artifacts; the latest implementation snapshot is represented by Story 1.5 and commit `18f00e4`. Inspect current source as authoritative and preserve unrelated concurrent worktree changes.
- Story 1.5 changed `debtor-web` auth, forms, middleware, router, token store, session handling, templates/CSS, root composition, runtime supervision, and operator documentation. This story should build on those current files rather than recreate the earlier paths.

### Latest Technical Information

- Current Tower Sessions documentation confirms cookie configuration belongs to `SessionManagerLayer`, expired records load as absent, and `Session::cycle_id()` requires a subsequent `save()` for durable rotation. This story uses `flush()` for logout and leaves `cycle_id()` in Login promotion.
- Current Axum documentation supports separate route routers with route-specific layers, `ServiceBuilder` ordering, and explicit shared state composition. Preserve the public/login/protected split and do not apply session middleware to probes or static assets.
- No current framework information requires a dependency update. `Cargo.lock` remains authoritative.

### References

- [Source: `_bmad-output/planning-artifacts/epics.md#Story 1.6: Maintain an Authenticated Session and Sign Out`]
- [Source: `_bmad-output/planning-artifacts/epics.md#Story 1.7: Extend Replay Protection Beyond Login and Sign Out`]
- [Source: `_bmad-output/planning-artifacts/epics.md#Cross-Cutting Story Rule`]
- [Source: `_bmad-output/planning-artifacts/sprint-change-proposal-2026-08-12.md#4.6 Epic 1 Submission-Token First-Consumer Sequence`]
- [Source: `specs/design.md#Security`]
- [Source: `specs/design.md#Operational Limits`]
- [Source: `specs/design.md#Maintenance`]
- [Source: `_bmad-output/project-context.md#Framework-Specific Rules`]
- [Source: `_bmad-output/project-context.md#Testing Rules`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-10 - Shared unsafe-request admission boundary`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-11 - Native HTML authority`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-13 - Process-local owner uniqueness`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-14 - Bounded operational envelope`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-15 - Safe failures, logging, and readiness`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md#Stable UX Contract Registry`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md#Global Shell and Navigation`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md#HTTP and Session Outcomes`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md#Interaction Focus Matrix`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md#Authentication`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/DESIGN.md#Layout & Spacing`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/DESIGN.md#Components`]
- [Source: `_bmad-output/implementation-artifacts/1-4-open-a-protected-and-accessible-login-page.md`]
- [Source: `_bmad-output/implementation-artifacts/1-5-sign-in-with-bounded-password-verification.md`]
- [Source: `debtor-web/src/session.rs`]
- [Source: `debtor-web/src/session_store.rs`]
- [Source: `debtor-web/src/submission_tokens.rs`]
- [Source: `debtor-web/src/forms.rs`]
- [Source: `debtor-web/src/middleware.rs`]
- [Source: `debtor-web/src/router.rs`]
- [Source: `debtor-web/src/handlers/auth.rs`]
- [Source: `debtor-web/src/templates.rs`]
- [Source: `debtor-web/src/templates/partials.rs`]
- [Source: `debtor-web/src/handlers/test_support.rs`]
- [Source: `debtor-web/templates/groups.html`]
- [Source: `debtor-web/templates/login.html`]
- [Source: `static/css/app.css`]
- [Source: `src/composition.rs`]
- [Source: `src/runtime.rs`]
- [Source: `tests/restart.rs`]
- [Source: Context7 `/maxcountryman/tower-sessions`, session cookie, expiry, flush, save, and cycle semantics]
- [Source: Context7 `/tokio-rs/axum`, route-specific middleware, router merge, ServiceBuilder, and state composition]

## Dev Agent Record

### Agent Model Used

openai/gpt-5.6-luna

### Debug Log References

- Ultimate context engine analysis completed - comprehensive developer guide created.
- Story selected from the first `backlog` story in the complete `_bmad-output/implementation-artifacts/sprint-status.yaml` order: `1-6-maintain-an-authenticated-session-and-sign-out`.
- Analyzed the complete Epic 1 context, approved sprint-change sequencing, PRD/addendum, normative `specs/design.md`, architecture spine, final UX contracts, project context, completed Stories 1.4 and 1.5, current session/auth/token/router/template/runtime source, restart tests, and recent implementation history.
- Context7 consulted for current Axum 0.8 route/state/middleware composition and tower-sessions 0.15 cookie, expiry, flush, save, and cycle semantics. No dependency upgrade is authorized.
- Scope ambiguity resolved in the story: Sign out is the first authenticated token consumer, while Story 1.7 remains the owner of the general authenticated token pool and route-neutral extension.

### Completion Notes List

- Ultimate context engine analysis completed - comprehensive developer guide created.
- Story status set to `ready-for-dev`.
- Story is ready for implementation by `dev-story`.
- Implemented 30-day authenticated inactivity refresh preservation, protected Sign out token issuance, independent indexed Sign out capacity, session-bound terminal reservation, session flush, cookie expiration, replay conflict handling, and restart-safe process-local semantics.
- Added shared authenticated shell protection values and Sign out form/status markup to all authenticated Askama pages. Native forms remain authoritative; local HTMX provides declarative pending suppression and body replacement.
- Added route/store coverage for successful logout, cookie/session invalidation, invalid CSRF and duplicate fields, replay conflict, token expiry/cleanup, session binding, independent pool capacity, and protected access denial.
- Browser geometry evidence remains manual because the repository has no executable browser harness; no automated geometry claim is made.

### Implementation Plan

- Reuse the existing process-local session and token owners; add a separate Sign out token namespace and indexed cleanup within the same web store.
- Generate the Sign out token alongside the shared authenticated shell, validate it with CSRF and session state, atomically reserve it immediately before `Session::flush`, and keep replay responses sanitized.
- Add shell-level Sign out markup/status behavior to all authenticated Askama pages while preserving native form navigation and using only the pinned local HTMX enhancement.
- Verify behavior with web route tests, store tests, workspace tests, Clippy, formatting, and architecture fitness checks.

### Validation Evidence

- `cargo fmt --all -- --check`
- `cargo check --workspace --all-features --locked`
- `cargo test --workspace --all-features --locked` (all passing; 62 web-inclusive tests plus all workspace suites)
- `SQLX_OFFLINE=true cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`
- `cargo run --bin architecture-check --locked`

### File List

- `_bmad-output/implementation-artifacts/1-6-maintain-an-authenticated-session-and-sign-out.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `debtor-web/src/handlers/auth.rs`
- `debtor-web/src/handlers/debts.rs`
- `debtor-web/src/handlers/groups.rs`
- `debtor-web/src/handlers/participants.rs`
- `debtor-web/src/handlers/spending_views.rs`
- `debtor-web/src/handlers/spendings.rs`
- `debtor-web/src/middleware.rs`
- `debtor-web/src/router.rs`
- `debtor-web/src/submission_tokens.rs`
- `debtor-web/src/templates.rs`
- `debtor-web/templates/confirm.html`
- `debtor-web/templates/debts.html`
- `debtor-web/templates/group.html`
- `debtor-web/templates/group_edit.html`
- `debtor-web/templates/groups.html`
- `debtor-web/templates/participant_edit.html`
- `debtor-web/templates/participants.html`
- `debtor-web/templates/spending_detail.html`
- `static/css/app.css`

### Change Log

- 2026-08-13: Implemented authenticated Sign out protection, shared shell integration, isolated token cleanup/capacity, replay-safe flush behavior, and comprehensive route/store coverage; status moved to `review`.
- 2026-08-13: Addressed code review findings - 12 items resolved; status moved to `done`.
