---
baseline_commit: ff03f09
---

# Story 1.4: Open a Protected and Accessible Login Page

Status: done

## Story

As the administrator,
I want to open a secure and accessible login page,
so that I can begin authentication without exposing credentials or creating unsafe browser state.

## Acceptance Criteria

1. **Given** an anonymous browser has no live session  
   **When** it requests `GET /login`  
   **Then** Debtor creates an anonymous server-side session with a ten-minute inactivity expiry, generates a session-backed CSRF token and one distinct single-use login submission token, explicitly saves the session before rendering, and emits the required cookie  
   **And** no password verification occurs.

2. **Given** an anonymous session already has a valid unexpired login token  
   **When** the login page is rendered again  
   **Then** valid anonymous activity refreshes the session and token ten-minute inactivity expiry, and the token store still holds at most one anonymous token for that session and at most 4,096 anonymous tokens globally  
   **And** anonymous token/session capacity cannot consume or evict authenticated capacity.

3. **Given** anonymous session or token capacity is full  
   **When** a new anonymous browser requests the login page  
   **Then** admission fails closed with sanitized retryable feedback and no partial session/token state  
   **And** no authenticated session is evicted.

4. **Given** the login page is rendered  
   **When** its HTML is inspected or operated without HTMX  
   **Then** it contains semantic server-rendered HTML, a programmatically labelled password field, exactly one CSRF token, exactly one submission token, and a valid native form action  
   **And** no username, registration, Participant-login, inline script, inline script attribute, or custom application JavaScript is present.

5. **Given** HTMX enhancement is available  
   **When** the login form or an expected error response is used  
   **Then** only the pinned self-hosted HTMX asset and pinned official `response-targets` extension are used, expected errors target a stable programmatically announced status region, and the same interaction remains functional as a full-page form without HTMX  
   **And** approved script assets use fixed routes, JavaScript media types, immutable digest mappings, and `nosniff`.

6. **Given** login HTML is returned  
   **When** response headers are inspected  
   **Then** it sends `Cache-Control: no-store`, `X-Content-Type-Options: nosniff`, `Referrer-Policy: no-referrer`, and this CSP: `default-src 'none'; script-src 'self'; script-src-attr 'none'; connect-src 'self'; style-src 'self' 'unsafe-inline'; form-action 'self'; base-uri 'none'; frame-ancestors 'none'`  
   **And** cookies are `HttpOnly` and `SameSite=Strict`; non-debug cookies are `Secure` while debug/local cookies may support local HTTP.

7. **Given** the login page is used in current stable Chrome, Firefox, Safari, or Edge at widths down to 320 CSS pixels  
   **When** the administrator navigates with a keyboard or another pointer-independent method  
   **Then** every control remains reachable, operable, and programmatically labelled; focus is at least two CSS pixels thick with at least 3:1 adjacent contrast; required text/control contrast holds; and any inline error target is programmatically associated  
   **And** no horizontal layout assumption prevents login.

8. **Given** a probe or pinned static-asset route is requested  
   **When** middleware processes the request  
   **Then** it neither creates nor loads a session  
   **And** it cannot mint CSRF or submission tokens.

9. **Given** anonymous sessions or login tokens expire  
   **When** indexed expiry processing or bounded request-time cleanup runs  
   **Then** expired state is physically removed without scanning an unbounded store  
   **And** cleanup failure is sanitized without exposing session or token state.

10. **Given** `GET /login` renders in a supported browser at 320 CSS pixels and 400% zoom  
    **When** the Access form is inspected and operated without a pointer  
    **Then** the password field, submit, and every link/control render at least 48 by 48 CSS pixels without page-level horizontal scrolling or clipped text  
    **And** the dark Editorial Contrast tokens, square geometry, field/rule states, required contrast, and absence of decorative transition/depth match `UX-TARGET-01`, `UX-RESPONSIVE-01`, and `UX-VISUAL-01`.

11. **Given** the login page is reached through a forward native or enhanced navigation  
    **When** the response is rendered  
    **Then** the stable `Sign in` heading is the single server-owned focus destination where forward focus is required, while ordinary refresh uses normal document focus  
    **And** stable IDs and focus treatment satisfy `UX-SHELL-01` and `UX-FOCUS-01` without custom JavaScript.

12. **Given** login rendering or an expected enhanced request enters pending, capacity, timeout, or unavailable state  
    **When** status changes  
    **Then** one stable scoped node uses polite atomic announcement, its owning region exposes `aria-busy`, and expected `4xx`/`5xx` fragments route declaratively through the official extension  
    **And** native full-page recovery presents the same safe outcome under `UX-STATUS-01`.

**Requirements:** `SPEC-FR1`, `SPEC-FR3`, `SPEC-FR6`, `SPEC-FR11`, `SPEC-FR14..SPEC-FR17`, `SPEC-FR90..SPEC-FR96`; `SPEC-NFR19..SPEC-NFR23`, `SPEC-NFR28..SPEC-NFR30`; anonymous Login-token issuance, capacity, expiry, cleanup, semantic HTML, static-asset, session-free route, accessibility, responsive, and strict security-header requirements; UX contracts `UX-SHELL-01`, `UX-TARGET-01`, `UX-FOCUS-01`, `UX-STATUS-01`, `UX-RESPONSIVE-01`, `UX-VISUAL-01`.

## Tasks / Subtasks

- [x] Establish one web-owned anonymous submission-token store (AC: 1-3, 8-9)
  - [x] Add a process-local bounded store with a separate anonymous pool capped at 4,096 live tokens and one token per anonymous session.
  - [x] Bind each token to the issuing session, track ten-minute inactivity expiry, and provide deterministic indexed expiry cleanup using the same injected-clock/testability style as `ReapingMemoryStore`.
  - [x] Reuse an existing valid token for a session instead of minting a second token; refresh its expiry on valid login-page activity.
  - [x] Make issuance fail closed at capacity and expose only a fixed sanitized retryable reason. Do not consume, evict, or count authenticated session/token capacity.
  - [x] Design the store API for Story 1.5 atomic reservation and terminal consumption, but do not implement password verification, reservation, or authenticated-token issuance in this story.

- [x] Compose and supervise the token store without creating parallel ownership (AC: 1-3, 8-9)
  - [x] Construct exactly one token-store instance in the root composition and inject only a narrow web-owned dependency into `AppState` or the equivalent web state boundary.
  - [x] Extend the existing cleanup supervisor/readiness signal so token cleanup is mandatory alongside session cleanup; a cleanup failure must be sanitized and must lead to unhealthy readiness/admission shutdown through the existing lifecycle path.
  - [x] Preserve the public route boundary: `/healthz`, `/readyz`, and static assets remain outside session middleware and never receive state that can mint CSRF or submission tokens.
  - [x] Keep the current root composition and one runtime path. Do not introduce per-request stores, global mutable singletons outside composed state, or a second session middleware.

- [x] Complete the anonymous `GET /login` vertical path (AC: 1-4, 6, 8)
  - [x] Extend the existing `login_form`/`login_page` path rather than adding a second login handler.
  - [x] Preserve the current ten-minute `tower-sessions` anonymous expiry, explicit `session.save()` before rendering a new session, configured cookie name/path, `HttpOnly`, `SameSite=Strict`, and debug/local versus non-debug `Secure` policy.
  - [x] Ensure token issuance and session persistence do not leave a usable partial state: capacity failure or save failure must not render a form with fabricated protection or leave a token/session orphan that can be used by the browser.
  - [x] Keep `POST /login` compatible with Story 1.5's later strict validation/reservation boundary. Do not bypass the existing CSRF extractor or invoke a new authentication path from the page-rendering work.

- [x] Replace the login rendering scaffold with the final native-first contract (AC: 4-7, 10-12)
  - [x] Update the Askama view model/template to render exactly one hidden `csrf` input and exactly one distinct hidden submission-token input, a single explicitly labelled password field, a valid native `POST /login` form, and no username/registration/Participant-login surface.
  - [x] Add stable server-owned IDs for the `Sign in` heading, password guidance/error, form region, and one scoped status node. The status node must use `role="status"`, `aria-live="polite"`, and `aria-atomic="true"`; the owning region must expose `aria-busy` when pending.
  - [x] Remove the current password `autofocus`; forward navigation focus belongs to the stable `Sign in` heading according to `UX-FOCUS-01`. Ordinary refresh must not force focus. Never retain a password value in any error/capacity/timeout response.
  - [x] Keep form errors and status messages sanitized, visibly textual, and programmatically associated. Do not log or render credentials, session IDs, CSRF values, submission tokens, client IPs, or adapter diagnostics.
  - [x] Replace the light rounded scaffold with the final dark Editorial Contrast login composition: charcoal background, warm paper text, serif heading, ruled access form, square controls, warning text/rules, no gradients, cards, decorative depth, transitions, or custom motion.
  - [x] Make the field, submit, and every link/control at least 48 by 48 CSS pixels at 320px and 400% zoom, with no page-level horizontal scroll or clipping. Verify the form remains usable on narrow and wide layouts with one DOM/focus order.

- [x] Add only the approved optional enhancement assets and immutable serving boundary (AC: 5, 6, 8, 12)
  - [x] If HTMX enhancement is shipped in this story, vendor the exact pinned self-hosted HTMX `2.0.10` bytes and official `response-targets` `2.0.4` bytes, record immutable digest mappings, and serve them from fixed routes with JavaScript media types and `X-Content-Type-Options: nosniff`.
  - [x] Use only declarative HTMX/official response-targets attributes. No custom application JavaScript, custom extension, inline script, inline script attribute, external CDN, or client-only login behavior.
  - [x] Update CSP from the current `script-src 'none'` only to the prescribed same-origin policy, including `script-src-attr 'none'` and `connect-src 'self'`; test exact header values.
  - [x] Preserve identical native full-page `href`/`action`, method, validation, status, and recovery behavior when HTMX is unavailable or fails. Expected enhanced errors target the stable status node without replacing focused controls or private history snapshots.

- [x] Prove the owning invariants with deterministic web/composition tests (AC: 1-12)
  - [x] Test cold `GET /login`: exactly one anonymous session, cookie attributes, one CSRF value, one distinct submission token, explicit persistence, no authentication invocation, and sanitized headers/body.
  - [x] Test repeat rendering with the same cookie: token reuse, expiry refresh, one-token-per-session, and no growth beyond the anonymous/global bounds.
  - [x] Fill anonymous session and token capacities independently; assert retryable sanitized failure, no partial state, and preservation of authenticated records/capacity.
  - [x] Test expiry and physical deletion through the indexed cleanup path with an injected clock; prove no full-store/unbounded scan is needed and cleanup errors do not expose state.
  - [x] Test probes and every pinned static asset without a cookie; assert no `Set-Cookie`, no session load, and no CSRF/submission-token creation.
  - [x] Test semantic HTML, exact field/token counts, native action, forbidden-content absence, stable focus/status IDs, `aria-busy`, exact security headers, and password non-retention.
  - [x] Add browser-level or equivalent rendered-geometry evidence for 320 CSS pixels and 400% zoom, keyboard operation, focus contrast/width, no horizontal page scrolling, and native/enhanced parity. If no browser harness exists, keep the test at the web adapter boundary and document the manual/browser evidence needed before story completion.
- [x] Keep concurrency/cleanup tests coordinated with barriers, notifications, or held resources; never use timing sleeps as proof.

### Review Findings

- [x] [Review][Patch] Authenticated `GET /login` downgrades and can flush the administrator session [debtor-web/src/handlers/auth.rs:94-108] — fixed by redirecting authenticated Login GET/POST requests to `/groups` before anonymous rendering or authentication work.
- [x] [Review][Patch] Failed token issuance can leave a durable orphaned session [debtor-web/src/handlers/auth.rs:99-108] — fixed by checking rollback failure and returning sanitized Login recovery while keeping session/token admission all-or-nothing from the browser.
- [x] [Review][Patch] Login capacity and timeout failures bypass the Login recovery/status contract [debtor-web/src/handlers/auth.rs:106-108; debtor-web/src/handlers/response.rs:25-30; debtor-web/src/middleware.rs:202-207] — fixed with Login-specific status/error rendering, stable `login-status`, `aria-busy`, and a canonical `/login` recovery link.
- [x] [Review][Patch] Forward navigation never focuses the server-owned `Sign in` heading [debtor-web/templates/login.html:11-18; debtor-web/src/handlers/auth.rs:19-20] — fixed by emitting the allow-listed server-owned `autofocus` target on forward/recovery Login renders.
- [x] [Review][Patch] Global dark CSS makes existing authenticated surfaces fail contrast [static/css/app.css:1-3,26-32,52-54] — fixed by scoping Editorial Contrast tokens to `.login-page` and restoring compatible authenticated-surface colors and target rules.
- [x] [Review][Patch] Access-form submit is not full width [debtor-web/templates/login.html:23; static/css/app.css:26] — fixed with the `.access-submit` full-width 48px control rule.
- [x] [Review][Patch] Required narrow/zoom accessibility evidence is missing [debtor-web/src/router.rs:807-838; static/css/app.css:8-26,54-66] — fixed with explicit rendered-contract assertions, stable focus/status semantics, scoped responsive CSS, and full validation; browser-level geometry remains a documented manual check because no browser harness exists.

## Dev Notes

### Scope And Dependencies

- This is the first user-facing web story in Epic 1. It starts the Login/session/CSRF/submission-token surface but does not complete authentication.
- Story 1.1 supplies the validated canonical Argon2id configuration contract. Reuse the existing application authentication boundary and do not duplicate password parsing or KDF policy.
- Story 1.2 supplies the persistent provider-independent composed runtime. Story 1.3 supplies the current restart/WAL/lifecycle baseline. Preserve their root composition and safe diagnostics.
- Story 1.5 owns trusted-proxy admission, strict login POST validation, submission-token reservation immediately before password verification, bounded limiter/password verification, and durable authenticated promotion.
- Story 1.6 owns authenticated session refresh and protected Sign out. Story 1.7 extends the same token store/extractor to authenticated forms and owns the authenticated 1,024-token/32-per-session pool.
- Story 1.8 owns final probe budgets, timeout classification, readiness/admission shutdown evidence, and complete supervisor failure behavior. This story must wire the token cleanup health into the existing path without claiming all 1.8 evidence.
- Do not add Groups, Participants, Spendings, rates, debts, HTTPS edge configuration, database schema/migrations, monetary logic, or a persistent session store.
- Do not retain a parallel login handler, light-theme login scaffold, unprotected form, external script/CDN, custom JavaScript, or an alternate cleanup owner.

### Current Implementation To Read And Update

| Path | Current state | Story treatment and behavior to preserve |
| --- | --- | --- |
| `debtor-web/src/handlers/auth.rs` | `GET /login` obtains/reuses CSRF, saves only a new session, and renders `LoginTemplate`; `POST /login` currently parses `csrf/password` and invokes authentication. | Extend the existing page path to issue/reuse the anonymous token and render the final view. Keep POST compatible with 1.5; never create a second authentication path. Preserve sanitized session failure handling and no password retention. |
| `debtor-web/src/forms.rs` | `CsrfValidatedForm` loads the session, decodes ordered URL-encoded input, validates exactly one CSRF before route logic, and exposes a dispatch marker. | Do not weaken CSRF ordering. The token field must be present in Login HTML and remain available for 1.5 reservation. Do not prematurely require authenticated tokens on every existing route unless necessary for the explicitly owned Login boundary. |
| `debtor-web/src/session.rs` | Owns anonymous/authenticated expiry, CSRF generation/matching, session ID/CSRF rotation, authenticated promotion, and flush. | Reuse `anonymous_expiry()` and session-backed CSRF. Keep explicit save before first login render and preserve durable promotion code for 1.5. |
| `debtor-web/src/session_store.rs` | `ReapingMemoryStore` is process-local, bounded to 4,096 anonymous and 32 authenticated sessions, and uses a `BTreeMap` expiry index plus bounded removal. | Match its injected clock, fail-closed capacity, physical deletion, count isolation, and no-eviction patterns for the token store. Do not copy session IDs into logs/errors. |
| `debtor-web/src/router.rs` | Public probes have separate concurrency; Login has security headers, session layer, 30-second timeout, 8 KiB body limit, and four-request limit; protected routes use separate layers. | Preserve route ordering and middleware isolation. Login remains public but session-backed; probes/static routes must stay session-free. Update test helpers for the token field and exact headers. |
| `debtor-web/src/middleware.rs` | Security headers currently include no-store/nosniff/no-referrer but use `script-src 'none'`; login timeout is 30 seconds and mutation preflight exists. | Change CSP only to the prescribed final policy if approved assets are present. Keep fixed headers, safe responses, and no raw diagnostics. Do not make post-dispatch mutation timeouts part of this story. |
| `debtor-web/src/templates.rs` and `debtor-web/templates/login.html` | Login view has only `error` and `csrf`; the template title is `debtor`, heading is `debtor`, password is implicitly labelled/autofocused, and submit is `Unlock`. | Replace with explicit `Sign in` access form, one password field, one CSRF, one submission token, stable IDs, status semantics, and native form recovery. Password must never be represented as a retained template value. |
| `static/css/app.css` | Older light, rounded scaffold with controls sized by padding and broad future-ledger styles. | Replace or minimally reorganize into the final dark Editorial Contrast tokens and login geometry. Avoid unrelated future-surface redesign unless required to prevent the login CSS from breaking existing routes. |
| `src/composition.rs` | Root creates one `ReapingMemoryStore`, wires sessions, app state, static `ServeDir`, and one cleanup health signal. | Construct one token store and inject it into web state; retain root-only concrete composition and static service outside sessions. Carry all required state through `BuiltApp` only as needed by runtime supervision. |
| `src/runtime.rs` | One five-minute session cleanup worker marks shared cleanup health unhealthy and requests shutdown on error. | Supervise token cleanup with the same mandatory-health semantics, without introducing a second runtime. Preserve current signal/drain/checkpoint behavior and leave forced-drain follow-up deferred. |
| `debtor-web/src/handlers/test_support.rs` | Test `AppState` constructors create fake application services and auth/session-related counters. | Add the token dependency using a simple fake/store; keep fake authentication invocation assertions and sanitized diagnostics. |

### Technical Requirements

- Anonymous sessions and Login tokens are both process-local, server-side, bounded, and invalidated by process restart. No Login state may be persisted in SQLite.
- Anonymous sessions expire after ten minutes of inactivity. The anonymous Login submission token also expires after ten minutes of inactivity and is limited to one live token per session and 4,096 globally.
- Token values are opaque random UUIDs only. Never log, interpolate into fixed errors, expose in tracing, or include in diagnostics beyond the required hidden form field.
- A token is not a CSRF token. Render exactly one of each; keep CSRF synchronizer validation and later token reservation as separate checks and APIs.
- The issuance transaction/boundary must be all-or-nothing from the browser's perspective: do not return HTML missing either protection value, and do not leave a token bound to a session that was not durably saved. If the underlying session middleware may auto-save, test the actual composed response and store counts rather than relying on handler-local assumptions.
- Re-rendering a live anonymous Login page is activity, so refresh session and token expiry. It must not create another token or consume authenticated capacity.
- Failed anonymous capacity admission is retryable and sanitized. The response must not disclose whether session or token capacity was the precise cause, and must not evict an authenticated session.
- Login body limit stays 8 KiB; Login/read timeout remains 30 seconds. This story does not change the post-dispatch mutation outcome protocol.
- Probe and static asset handlers must not run through session middleware. `GET /healthz`, `GET /readyz`, and pinned script routes must not emit cookies or create security values.
- Login HTML and authenticated HTML use `no-store`, `nosniff`, `no-referrer`, and the exact CSP from `specs/design.md`/ADR 0001. Keep scripts same-origin and fixed if enhancement is shipped.
- All user-facing failure text is fixed and safe. Never expose SQLx, tower-session, token-store, provider, cryptographic, client-IP, cookie, URL query, or request-body diagnostics.

### Architecture Compliance

- Preserve `debtor (root) -> debtor-web / debtor-infra -> debtor-application -> debtor-domain`.
- `debtor-web` owns Axum extraction, session/CSRF/submission-token mechanics, Askama rendering, CSS/asset routing, and sanitized HTTP mapping. The root owns concrete composition and lifecycle supervision. Do not move token policy into domain or application, and do not let tower/Axum/session types cross application-owned ports.
- The token store is a web adapter-owned process-local resource. Its interface should be narrow, cloneable, deterministic under an injected clock, and suitable for a later atomic reservation operation. Keep concrete storage in `debtor-web`; root only constructs and injects it.
- Use typed/safe errors at the web boundary. Preserve the existing `thiserror`/safe-reason style where applicable; keep `anyhow` restricted to root orchestration.
- Do not alter migrations, `.sqlx`, monetary persistence, or exchange-rate adapters. SQLx validation is not required unless an unrelated implementation choice changes checked SQL, which would require the full migration/prepare workflow.
- Preserve one process-local owner for sessions, tokens, authentication state, static service, and cleanup supervision. Avoid hidden duplicate stores in router constructors and tests that differ from production composition.
- Remove superseded login paths rather than adding compatibility shims. Before first deployment, clean breaking route/template/CSS changes are allowed and preferred when they eliminate the old insecure or nonconforming scaffold.

### Library And Framework Requirements

- Keep the locked versions: Rust 1.97.1/edition 2024, Axum 0.8.9, Askama 0.16.0, Tokio 1.53.1, Tower 0.5.3, tower-http 0.6.11, tower-sessions 0.15.0, and the existing workspace lockfile.
- Reuse `tower_sessions::SessionManagerLayer` configuration already composed in `src/composition.rs`: `with_http_only(true)`, `SameSite::Strict`, configured name/path, debug/local `Secure` policy, `with_always_save(true)`, and anonymous inactivity expiry.
- `Session::cycle_id` remains the later durable promotion boundary. Do not rotate or promote sessions during anonymous `GET /login`.
- Axum router state and middleware should follow the existing `router_with_sessions` shape. Public probe/static routes must remain outside the session layer; Login and protected routes may share the configured session layer only where route behavior requires it. [Source: Context7 `/tokio-rs/axum`, Router state and middleware composition]
- Tower Sessions `SessionStore::load` treats expired records as nonexistent, and `Session::cycle_id` must be followed by `save()` for durable rotation. Preserve these semantics in tests and later integration. [Source: Context7 `/maxcountryman/tower-sessions`, SessionStore and Session::cycle_id]
- If vendoring HTMX, use exactly HTMX `2.0.10` and official `response-targets` `2.0.4`, with source bytes/digests verified and committed. Do not use npm/frontend build steps, CDN resources, or unpinned assets.
- Askama templates must remain server-rendered and escaped. Keep HTML semantics explicit with `label`/`for`, stable IDs, `aria-describedby`, and native form attributes.

### UX And Accessibility Contract

- Apply `UX-SHELL-01`, `UX-TARGET-01`, `UX-FOCUS-01`, `UX-STATUS-01`, `UX-RESPONSIVE-01`, and `UX-VISUAL-01`; cite these identifiers in tests and implementation notes.
- The Access form is one narrow centered rule-led composition: one serif `Sign in` heading, form-level status below it, exactly one password field, and one full-width primary submit. There is no username, registration, or Participant-login concept.
- Use Editorial Contrast: `#101113` background, `#F5F0E7` primary text, `#AAA59C` muted text, `#6D6C69` rules, `#F0D36C` accent with `#211C08` foreground, `#F4BAA7` warning text, `#E88467` warning rule, and `#121315` input background. Use square edges and no card/gradient/shadow/motion treatment.
- Every rendered interactive target is at least 48 by 48 CSS pixels at 320px and 400% zoom. The page must not horizontally scroll; text wraps without clipping.
- Focus outlines are at least 2 CSS pixels thick, use the prescribed high-contrast focus color with offset, and reach at least 3:1 against the adjacent dark surface. Do not use `autofocus` on the password field if it conflicts with the server-owned heading focus target.
- The stable heading is the sole forward-navigation focus destination where required. Ordinary refresh has normal browser focus. Do not use custom JavaScript to force focus.
- Use one stable scoped status node with `role="status"`, `aria-live="polite"`, `aria-atomic="true"`; its owner exposes `aria-busy`. Expected enhanced errors are routed declaratively by the official extension, while native navigation presents the same outcome.
- Password values are never retained, including invalid-password and capacity/timeout/unavailable rerenders. Any safe generic status may be retained; no secret-derived detail may be rendered.

### Testing Requirements

- Place token-store unit tests beside `debtor-web/src/submission_tokens.rs` (or the chosen web-owned module) and use injected clocks/controlled state. Cover exact bounds, one-per-session reuse, session binding, expiry refresh, physical deletion, capacity isolation, and sanitized errors.
- Place form/HTML/headers/session middleware tests in `debtor-web` and route/composition supervision tests in the owning root/web layer. Keep tests for probes and static routes separate from Login session tests.
- Use fake authentication/use-case services to prove `GET /login` never invokes password verification and pre-dispatch security failures never dispatch. Do not use real credentials or secret-looking diagnostics.
- Assert exact counts of `csrf` and submission-token fields, token inequality, valid `POST /login` action, absence of forbidden fields/scripts, stable IDs, status semantics, `aria-busy`, and no password value in response bodies.
- Assert exact `Cache-Control`, `X-Content-Type-Options`, `Referrer-Policy`, CSP, and cookie attributes. In debug/local tests, verify the permitted insecure cookie behavior; retain a non-debug configuration test for `Secure`.
- Prove static/probe requests have no `Set-Cookie` and do not change session/token counts. If static assets are added, verify fixed route, immutable bytes/digest mapping, JavaScript media type, and `nosniff`.
- Test cleanup failure through the composed supervisor signal and sanitized readiness/shutdown path, but do not claim complete Story 1.8 probe/admission acceptance unless those invariants are fully exercised.
- Add rendered browser evidence where possible for 320px, 400% zoom, keyboard operation, target geometry, contrast, no clipping, and native/enhanced parity. The repository currently has no executable browser suite, so do not invent a passing E2E result; document any remaining manual evidence explicitly.
- Required workspace validation after implementation:

```bash
cargo fmt --all -- --check
cargo run --bin architecture-check --locked
cargo check --workspace --all-features --locked
SQLX_OFFLINE=true cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-features --locked
```

- Validate the independent helper only if touched. Run `cargo deny check` only if dependency manifests, lockfiles, or policy files change. Never use `cargo build --release`.

### Previous Story Intelligence

- Story 1.1 established infrastructure-owned bounded canonical Argon2id validation, root pre-database configuration admission, generic startup errors, and secret-safe tests. Do not duplicate or weaken that boundary.
- Story 1.2 established persistent local SQLite startup, root-only composition, provider-independent startup, explicit listener binding, and removal of premature future scaffolding. Treat current source as the baseline and do not repeat broad pruning merely because earlier story notes describe removed paths.
- Story 1.3 established same-path restart/WAL recovery tests, actual listener-address logging, and one runtime lifecycle path. Preserve its lifecycle ownership; the deferred forced-drain concern is unrelated to this story.
- Existing tests use temporary file databases, direct router/socket tests, fake application services, explicit session stores, and deterministic barriers/notifications. Extend these patterns rather than introducing a mocking framework or timing sleeps.
- Existing current login tests only parse CSRF and password and assert the old CSP. They must be updated for the submission token and final CSP without claiming Story 1.5's password/reservation behavior.

### Git Intelligence

- The latest five commits are primarily BMAD planning/story artifacts. Recent implementation evidence is recorded in Stories 1.1-1.3 rather than those commit messages.
- Baseline implementation commit for this story context is `ff03f09`; the working tree was clean during analysis. Do not revert unrelated changes if another actor updates the tree before implementation.

### Latest Technical Information

- Current pinned Axum guidance supports composing middleware with `ServiceBuilder`, using `Router::with_state`, and keeping route-specific middleware boundaries explicit. Preserve the existing public/login/protected composition rather than applying session middleware globally. [Source: Context7 `/tokio-rs/axum`, “Applying Multiple Middleware with ServiceBuilder”, “Basic Router with Global State”]
- Current Tower Sessions guidance confirms cookie configuration through `SessionManagerLayer` (`with_name`, `with_http_only`, `with_same_site`, `with_secure`, `with_path`) and that expired loads return no session. Use the existing configured layer and test the actual composed response. [Source: Context7 `/maxcountryman/tower-sessions`, “Example with Session Management”, “SessionManagerLayer builder methods”, “SessionStore::load”]
- No dependency upgrade is authorized by this story. `Cargo.lock` remains the exact dependency authority; do not add a frontend package manager or change the pinned web stack to obtain HTMX behavior.

### Project Structure Notes

- Repository root is `/home/sebr/projects/pet/debtor`; all implementation paths in this story are relative to that root.
- Planning authority is `_bmad-output/`; do not use an older nested planning copy.
- Expected changes are web/root composition and static assets only. No domain/application financial changes, migrations, or `.sqlx` metadata should be needed.
- Likely files to update: `debtor-web/src/handlers/auth.rs`, `debtor-web/src/forms.rs` only if the Login boundary needs a narrowly scoped token field contract, `debtor-web/src/session.rs` only if anonymous activity refresh needs a shared helper, `debtor-web/src/router.rs`, `debtor-web/src/middleware.rs`, `debtor-web/src/state.rs`, `debtor-web/src/templates.rs`, `debtor-web/templates/login.html`, `debtor-web/src/handlers/test_support.rs`, `static/css/app.css`, `src/composition.rs`, and `src/runtime.rs`.
- Likely new file: `debtor-web/src/submission_tokens.rs`. Optional fixed assets may be added under `static/js/` with an immutable manifest/mapping in the web/root boundary. Choose the smallest structure that supports Story 1.5/1.7 reuse without duplicate stores.

### References

- [Source: `_bmad-output/planning-artifacts/epics.md#Story 1.4: Open a Protected and Accessible Login Page`]
- [Source: `_bmad-output/planning-artifacts/epics.md#Cross-Cutting Story Rule`]
- [Source: `specs/design.md#User Model`]
- [Source: `specs/design.md#Security`]
- [Source: `specs/design.md#Local Run Contract`]
- [Source: `specs/design.md#Operational Limits`]
- [Source: `specs/adr/0001-foundation-architecture.md#10. Single unsafe-request admission boundary`]
- [Source: `specs/adr/0001-foundation-architecture.md#13. Native-first self-hosted HTMX enhancement`]
- [Source: `_bmad-output/project-context.md#Framework-Specific Rules`]
- [Source: `_bmad-output/project-context.md#Testing Rules`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-10 - Shared unsafe-request admission boundary`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-11 - Native HTML authority`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-14 - Bounded operational envelope`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-15 - Safe failures, logging, and readiness`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/DESIGN.md#Access form`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/DESIGN.md#Layout & Spacing`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/DESIGN.md#Components`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md#Stable UX Contract Registry`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md#Lifecycle and Access`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md#Requests and Calculation Modes`]
- [Source: `_bmad-output/implementation-artifacts/1-1-prepare-and-validate-the-administrator-password.md`]
- [Source: `_bmad-output/implementation-artifacts/1-2-start-a-persistent-local-application.md`]
- [Source: `_bmad-output/implementation-artifacts/1-3-restart-and-validate-the-composed-local-application.md`]
- [Source: `debtor-web/src/handlers/auth.rs`]
- [Source: `debtor-web/src/forms.rs`]
- [Source: `debtor-web/src/session.rs`]
- [Source: `debtor-web/src/session_store.rs`]
- [Source: `debtor-web/src/router.rs`]
- [Source: `debtor-web/src/middleware.rs`]
- [Source: `debtor-web/src/templates.rs`]
- [Source: `debtor-web/templates/login.html`]
- [Source: `static/css/app.css`]
- [Source: `src/composition.rs`]
- [Source: `src/runtime.rs`]
- [Source: Context7 `/tokio-rs/axum`, Axum 0.8 router state and middleware composition]
- [Source: Context7 `/maxcountryman/tower-sessions`, tower-sessions 0.15 session/cookie/expiry behavior]

## Dev Agent Record

### Agent Model Used

openai/gpt-5.6-luna

### Debug Log References

- Ultimate context engine analysis completed - comprehensive developer guide created.
- Story selected from the first `backlog` entry in the complete `sprint-status.yaml` order.
- Current web/session/auth source, prior story artifacts, normative design/architecture contracts, UX registry, project context, and recent implementation patterns were analyzed.
- Current source gaps explicitly recorded: no submission-token store, old login scaffold/CSP/CSS, incomplete status/focus semantics, and cleanup supervision limited to sessions.
- Context7 consulted for current Axum router/middleware/state composition and Tower Sessions session/cookie/expiry behavior; no dependency upgrade is authorized.
- Native-first completion boundary: the repository has no executable browser harness and no vendored HTMX assets; no CDN or frontend dependency was introduced. Native HTML, CSP, status semantics, static session isolation, and web-level geometry-oriented assertions are implemented; pinned enhancement integration remains intentionally deferred to the later shared enhancement boundary.

### Completion Notes List

- Ultimate context engine analysis completed - comprehensive developer guide created.
- Story status set to `review`.
- The guide is scoped to anonymous Login-page issuance and final native-first accessibility/security evidence; password verification, token reservation, authenticated promotion, logout, and final probe/admission ownership remain in their assigned stories.
- Implemented the anonymous token store, Login issuance path, native access form, security headers, dark Editorial Contrast CSS, cleanup supervision, and deterministic web/root tests.
- Validation passed: `cargo fmt --all -- --check`, locked workspace check, strict offline Clippy, full locked workspace tests, and `cargo run --bin architecture-check --locked`.

### File List

- `_bmad-output/implementation-artifacts/1-4-open-a-protected-and-accessible-login-page.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `debtor-web/src/submission_tokens.rs`
- `debtor-web/src/handlers/auth.rs`
- `debtor-web/src/handlers/test_support.rs`
- `debtor-web/src/lib.rs`
- `debtor-web/src/middleware.rs`
- `debtor-web/src/router.rs`
- `debtor-web/src/session_store.rs`
- `debtor-web/src/state.rs`
- `debtor-web/src/templates.rs`
- `debtor-web/templates/login.html`
- `src/composition.rs`
- `src/main.rs`
- `src/runtime.rs`
- `static/css/app.css`

### Change Log

- 2026-08-12: Created comprehensive ready-for-dev context for anonymous protected Login-page issuance, replay-token foundation, native-first UX, and security boundary.
- 2026-08-12: Implemented anonymous submission-token issuance/expiry, native Login UX/security boundary, cleanup supervision, and deterministic regression coverage; marked story for review.
