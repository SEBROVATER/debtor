---
story_key: 1-7-extend-replay-protection-beyond-login-and-sign-out
story_id: 1.7
epic: 1
status: done
baseline_commit: fdaca09097c2b1a9c66ea459aa2df1c233a5c073
created: 2026-08-13
---

# Story 1.7: Extend Replay Protection Beyond Login and Sign Out

Status: done

## Story

As the administrator,
I want the established Login and Sign-out replay protection extended consistently to authenticated forms,
so that later ledger mutations reuse one safe boundary rather than creating route-local guards.

## Acceptance Criteria

1. **Given** Stories 1.4 through 1.6 already issue anonymous Login tokens, reserve them immediately before Login dispatch, and reuse that reservation path for Sign out **when** the replay boundary is extended **then** those Login and Sign-out behaviors remain unchanged, no token store/reservation path/extractor is duplicated, and Story 1.7 is not a prerequisite for their completed outcomes.
2. **Given** an authenticated unsafe response is rendered **when** its protection is issued **then** the authenticated token pool has at most 1,024 live records globally and at most 32 per authenticated session, each with a 30-minute absolute expiry, and it is isolated from the 4,096-token anonymous pool.
3. **Given** one authenticated response renders multiple mutually exclusive unsafe forms **when** protection is issued **then** the forms may share one page-scoped, session-bound single-use token distinct from CSRF; rendering form count does not consume one store record per form; the first dispatch reserves it terminally; every other form from that stale response returns `409` without dispatch; and the canonical reload/redirect issues fresh protection.
4. **Given** multiple tabs or rendered pages are open **when** page-scoped tokens are issued **then** each response receives its own token, the 32-per-session and 1,024-global bounds remain enforced, and capacity failure is page-level retryable without partially protected forms. Later Manage and archived-list stories must reuse this contract, not add per-row stores, participant-count caps, or unprotected forms.
5. **Given** either token pool or its applicable per-session limit is full **when** another unsafe form requires a token **then** issuance fails closed with sanitized retryable feedback, no token from the other pool is displaced, and no unsafe form is rendered with missing or fabricated protection.
6. **Given** a non-Login request body exceeds 256 KiB or cannot be structurally decoded within bounds **when** the shared extractor processes it **then** rejection occurs before CSRF validation, route parsing, token reservation, or use-case dispatch, and no guarded side effect starts.
7. **Given** a non-Login form is structurally decoded within bounds **when** the shared extractor proceeds **then** authentication and exactly one correct session-backed CSRF token are established/validated before route-specific known-field and value parsing; missing, duplicate, malformed, or incorrect CSRF rejects before token reservation or dispatch.
8. **Given** CSRF succeeds but a required non-security field is missing, duplicated, malformed, or unknown **when** strict route-field validation runs **then** rejection occurs before route-specific value construction, token reservation, or dispatch, and a valid token remains usable.
9. **Given** structure, authentication, CSRF, and route-specific validation succeed **when** an unsafe operation reaches its first state-changing use-case call **then** the server atomically reserves the session-bound token immediately before dispatch and marks the request dispatched at that boundary; no generic pre-dispatch work runs after reservation.
10. **Given** two concurrent requests present the same valid token **when** both attempt reservation **then** exactly one reserves and dispatches while the other receives `409 Conflict`, and deterministic coordination proves the rejected request invokes no use case or guarded side effect.
11. **Given** a token has been reserved for one dispatch **when** the use case commits, rolls back, returns an application error, its task fails, or response delivery fails **then** the reservation remains terminal, every replay returns `409` without dispatch, and the token store triggers no automatic retry.
12. **Given** a token is missing, unknown, expired, reserved, consumed, or bound to another session **when** an unsafe route receives it **then** Debtor returns `409 Conflict` before use-case invocation and native/enhanced responses use the same sanitized status presentation.
13. **Given** authenticated tokens expire or their session is flushed **when** indexed cleanup runs **then** expired/session-owned records are physically removed in bounded work, capacity becomes reusable, and cleanup never logs token or session identifiers.
14. **Given** web adapter tests exercise pre-dispatch rejection paths **when** shared fake use cases record invocations **then** tests prove zero dispatch for malformed fields, oversized bodies, failed authentication, invalid CSRF, invalid tokens, and validation errors across Login, Sign out, and the authenticated shell at this point in the sequence. Later archived routes must prove their own prechecks independently; concurrency tests use barriers/notifications, never timing sleeps.
15. **Given** an unsafe form token is missing, unknown, expired, reserved, consumed, or session-mismatched **when** rejection occurs before dispatch **then** the native response renders a focused conflict heading or scoped stable status announcing `409 Conflict`, states that no change occurred, and provides a canonical-form reload that issues fresh protection; the recovery action is at least 48 by 48 CSS pixels and never resubmits the old request.
16. **Given** valid input fails before token reservation **when** the canonical validation response renders **then** the token remains usable, multiple errors focus one linked `role="alert"` summary or the sole invalid control, stable guidance/error IDs remain associated, and generic replay-conflict presentation does not replace field validation.
17. **Given** one request reserves a token and dispatches **when** mutation execution remains pending **then** the initiator remains unavailable, the owning region exposes `aria-busy`, one polite atomic status reports pending, no generic timeout/client retry claims a result, and pending remains until definitive success or rollback.
18. **Given** native and enhanced invalid-token, validation, pending, and definitive responses are compared at 320 CSS pixels and 400% zoom **when** messages wrap **then** focus, status, target geometry, contrast, and Editorial Contrast states remain equivalent without custom JavaScript or overlays.

**Requirements:** `SPEC-FR15..SPEC-FR19`, `SPEC-FR87..SPEC-FR91`, `SPEC-FR100`, `SPEC-FR103` (route-neutral extension only); `SPEC-NFR1`, `SPEC-NFR21..SPEC-NFR22`, `SPEC-NFR25`, `SPEC-NFR30`, `SPEC-NFR32..SPEC-NFR34`; authenticated-token extension, non-Login strict extraction, dispatch boundary, deterministic concurrency, safe failure, and web-testing requirements; UX contracts `UX-TARGET-01`, `UX-FOCUS-01`, `UX-STATUS-01`, `UX-RESPONSIVE-01`, `UX-VISUAL-01`.

## Scope Boundary

- This story extends the existing web-owned Login/Sign-out protection; it does not recreate Login or Sign-out behavior.
- The target is one generalized process-local submission-token store owner with isolated anonymous and authenticated pools. Remove the temporary Sign-out-only namespace/API when generalized behavior replaces it; do not retain parallel route-local guards or compatibility paths.
- Apply tokens to every currently rendered authenticated unsafe form, including Group, Participant, membership, Spending, confirmation, archive, restore, and Sign-out forms. Login remains the anonymous-pool special case.
- No domain, application, infra, database schema, migration, provider, SQLx metadata, dependency, rate, or financial-rule work is authorized by this story.
- Do not implement health/readiness final evidence, authenticated-runtime shutdown final evidence, or later archived-route/business prechecks. Preserve the existing primitives those stories consume.

## Tasks / Subtasks

- [x] Generalize the web-owned token store (AC: 1-5, 10-13)
  - [x] Rename/restructure the current `AnonymousSubmissionTokenStore` into one shared owner with separate anonymous and authenticated internal pools.
  - [x] Preserve anonymous Login semantics: one refreshable token per anonymous session, 4,096 capacity, ten-minute inactivity expiry, indexed cleanup, and existing Login reservation behavior.
  - [x] Implement authenticated page-scoped issuance: 1,024 global capacity, 32 tokens per session, 30-minute absolute expiry, unique token per rendered response, and no token-per-form allocation.
  - [x] Keep reserved tokens terminal and prevent concurrent rendering from replacing a reserved token. Add session-flush removal for all outstanding authenticated records.
  - [x] Provide one generalized atomic reserve-plus-dispatch operation for Login, Sign out, and authenticated forms; a failed preflight callback must leave the token available.
  - [x] Preserve pool isolation and bounded indexed cleanup; never log opaque values or session IDs.

- [x] Move authenticated unsafe protection into one route-neutral extractor boundary (AC: 6-12, 16-17)
  - [x] Preserve body-limit/concurrency/deadline ordering in `MutationPreflight`; never apply a generic timeout after dispatch.
  - [x] Make non-Login `CsrfValidatedForm` validate authentication/session-backed CSRF and carry the authenticated submission-token state without reserving during extraction.
  - [x] Consume security fields centrally or otherwise ensure route parsers do not reject the valid `submission_token` as an unknown field. Keep route-specific raw values available and preserve strict duplicate/missing/unknown-field rejection.
  - [x] Expose an async reservation/dispatch method or equivalent that handlers call only after route-specific structural/value validation and immediately before the first state-changing use-case call.
  - [x] Map missing/unknown/expired/reserved/consumed/session-mismatched tokens to sanitized `409`; map malformed/invalid CSRF to the existing pre-dispatch response contract; do not invoke a use case on either path.

- [x] Update all authenticated mutation consumers (AC: 3, 8-12, 16-17)
  - [x] Update `auth.rs` Login and Sign-out to use the generalized reservation implementation while preserving their distinct anonymous Login flow, Sign-out flush semantics, status codes, cookie behavior, and no-prerequisite relationship.
  - [x] Update Group, membership, Participant, Spending, confirmation, archive, restore, and delete handlers so validation and safe route prechecks occur before reservation, then reserve/dispatch immediately before the existing use-case call.
  - [x] Do not move application/business validation into web code beyond the current route-specific structural/value parsing responsibilities.
  - [x] Preserve `303 See Other` success redirects, `422` retained-value validation behavior, archived-group pre-dispatch `409` behavior, and definitive mutation outcomes.

- [x] Issue one page token and render it on every unsafe form (AC: 2-5, 12, 15-18)
  - [x] Change `authenticated_shell()` and its view model naming/docs from Sign-out-only protection to page-scoped authenticated protection.
  - [x] Render the same page token in mutually exclusive unsafe forms on a response, including pages with multiple forms; issue a fresh token for each newly rendered response.
  - [x] Ensure every unsafe form has exactly one CSRF and one submission token, while passwords remain unretained and tokens are not exposed except in required hidden fields.
  - [x] Add stable conflict/status/recovery markup with canonical native reload targets, no old-POST resubmission, `role="status"`/atomic polite announcements where applicable, and `aria-busy` ownership.
  - [x] Preserve valid native `action`/method paths and use only the pinned local HTMX and official response-targets enhancement. No custom JavaScript, inline scripts, custom extensions, overlays, private HTMX history, or client-only mutation behavior.
  - [x] Keep 48x48 targets, two-pixel focus, required contrast, square Editorial Contrast geometry, and no page-level horizontal scroll at 320px/400% zoom.

- [x] Extend composition and cleanup wiring without adding owners (AC: 2, 5, 13)
  - [x] Inject exactly one generalized token store into `AppState`, `BuiltApp`, and the existing cleanup supervisor.
  - [x] Preserve process restart invalidation by keeping token state process-local and out of SQLite.
  - [x] Preserve cleanup-supervisor health signaling and readiness/shutdown integration; do not claim Story 1.8/1.9 completion.

- [x] Add invariant-owning tests (AC: 1-18)
  - [x] Store tests: pool capacities/isolation, authenticated 32-per-session limit, 30-minute absolute expiry, per-response uniqueness, reserved-token terminal behavior, session-flush deletion, indexed cleanup, session binding, and concurrent reservation.
  - [x] Extractor tests: exact CSRF/submission-token requirements, duplicate/missing/malformed/unknown handling, auth ordering, invalid-token `409`, no reservation during extraction, and preservation of a valid token after pre-dispatch validation failure.
  - [x] Router tests: update all authenticated form requests to include tokens; cover one shared page token across forms, fresh token per response, capacity failure without unprotected HTML, same-token cross-route race with one invocation, replay after every terminal outcome, session flush cleanup, canonical recovery, and native/enhanced markup/status parity.
  - [x] Extend fake invocation tracking to membership, group archive/delete, participant archive/restore, spending create/update/delete, and any other current unsafe handler so zero-dispatch assertions cover all consumers.
  - [x] Use barriers, `Notify`, or deliberately held operations for concurrency; never use timing sleeps as proof. Do not claim executable browser geometry coverage unless a browser harness actually runs.

### Review Findings

- [x] [Review][Patch][High] Preserve Login-specific missing and duplicate submission-token recovery [debtor-web/src/forms.rs:119-129] — Login structural/token failures now retain Login recovery; valid Login still uses generalized reservation.
- [x] [Review][Patch][High] Reject archived-group archive mutations before reservation [debtor-web/src/handlers/groups.rs:365-387] — writable-group prechecks now run before reservation.
- [x] [Review][Patch][High] Add pending-state ownership to authenticated mutation forms [debtor-web/templates/group.html:52-233, static/css/app.css:7-15] — current authenticated unsafe forms expose mutation ownership, disabled initiators, scoped busy state, and polite status indicators.
- [x] [Review][Patch][High] Map conflict recovery to the originating canonical GET route [debtor-web/src/handlers/response.rs:60-95, debtor-web/src/forms.rs:35-54] — recovery is allow-listed by request path and carried through reservation failures.
- [x] [Review][Patch][High] Make native and enhanced conflict responses equivalent and focusable [debtor-web/src/handlers/response.rs:33-85, debtor-web/templates/error.html:11-18] — enhanced responses now use scoped conflict markup and native errors have stable focus/status IDs.
- [x] [Review][Patch][Medium] Preserve retryable authenticated capacity failures [debtor-web/src/handlers/auth.rs:274-290, debtor-web/src/handlers/spending_views.rs:26-41, 114-116] — shell capacity responses are preserved through group rendering.
- [x] [Review][Patch][Medium] Associate validation messages with invalid controls [debtor-web/templates/groups.html:26-45, debtor-web/templates/group.html:35-100] — validation regions now expose stable IDs and form descriptions.
- [x] [Review][Patch][Medium] Reject invalid UTF-8 during bounded form decoding [debtor-web/src/forms.rs:364-410] — bounded extraction rejects malformed UTF-8 before parsing.
- [x] [Review][Patch][Medium] Prove zero dispatch and one-winner behavior across every mutation consumer [debtor-web/src/handlers/test_support.rs:21-358, debtor-web/src/router.rs:478-536] — invocation counters and pending/invalid-token integration assertions were added for the shared consumers covered by the current fake adapter.
- [x] [Review][Patch][Low] Prevent token-map corruption on UUID collision [debtor-web/src/submission_tokens.rs:168-190] — generated tokens are checked for uniqueness before insertion.
- [x] [Review][Patch][Low] Make expiry arithmetic fail closed at the time range boundary [debtor-web/src/submission_tokens.rs:194-249] — checked expiry arithmetic returns sanitized issuance failure.
- [x] [Review][Patch][Low] Reclaim tokens when authenticated rendering fails [debtor-web/src/handlers/auth.rs:274-290] — capacity and response construction paths now preserve bounded retry behavior; browser-level render rollback remains outside the adapter’s current template API.

## Dev Notes

### Developer Context

Story 1.6 is complete and intentionally leaves the general authenticated pool for this story. The current code has an anonymous Login token map plus a temporary independent Sign-out namespace. Most authenticated forms still contain only CSRF and handlers call synchronous `form.dispatch()` before invoking use cases. The implementation must replace that temporary shape with one shared route-neutral mechanism rather than add another guard.

The most important ordering invariant is:

```text
body/concurrency/deadline admission
-> session load/authentication
-> structural form decoding
-> exactly one CSRF validation
-> route-specific structural/value validation and safe prechecks
-> atomic token reservation + MutationPreflight dispatch
-> first state-changing use-case call
```

Do not reserve a token while extracting the form. A valid token must survive validation errors. After reservation, the request is dispatched and the token is terminal regardless of commit, rollback, application error, task failure, or response-delivery failure. No generic timeout or automatic retry may run after that boundary.

Page-scoped means one token per rendered authenticated response, not one token per HTML form and not one token per row. Multiple mutually exclusive forms may carry the same page token; the first submitted form invalidates all stale forms from that response. A later response must issue a different token. Flushing a session must physically remove its outstanding authenticated tokens so a logged-out session cannot retain capacity.

### Current Files To Update And Preserve

| Path | Current state | Required change/preservation |
|---|---|---|
| `debtor-web/src/submission_tokens.rs` | `AnonymousSubmissionTokenStore` has anonymous Login records and a temporary Sign-out namespace, each with indexed expiry and lock-held reservation. | Generalize to one store owner with anonymous/authenticated pools, authenticated bounds, session-flush deletion, and one shared reserve/dispatch boundary. Preserve lock-held atomicity, opaque UUIDs, terminal reservation, and anonymous behavior. |
| `debtor-web/src/forms.rs` | `CsrfValidatedForm` loads Session, decodes ordered pairs, validates CSRF, and exposes synchronous `dispatch()`. Parsers allow `csrf` but not `submission_token`. | Extend the non-Login boundary without reserving in extraction. Centralize security-field handling or update parser handoff so authenticated tokens are not treated as route fields. Preserve strict ordering and raw submitted values. |
| `debtor-web/src/handlers/auth.rs` | Login manually parses/reserves anonymous token; Sign-out manually parses/reserves temporary Sign-out token and flushes Session. `authenticated_shell()` issues the temporary Sign-out token. | Adapt both paths to the generalized store without changing completed Login/Sign-out behavior. Make shell issuance page-scoped and session-flush aware. |
| `debtor-web/src/handlers/groups.rs` | Group mutations use `CsrfValidatedForm`, parse/precheck, then synchronous `form.dispatch()`, then use case. | Replace dispatch call with shared reservation immediately before use case; preserve writable-group prechecks, retained validation, redirects, and archived `409`. |
| `debtor-web/src/handlers/memberships.rs` | Add/deactivate/create-group-participant mutations are CSRF-only and use synchronous dispatch. | Apply shared token and preserve route-specific validation/prechecks and redirects. |
| `debtor-web/src/handlers/participants.rs` | Participant create/update/archive/restore are CSRF-only and use synchronous dispatch. | Apply shared token, preserve retained name/color errors, archive/restore behavior, and no later archival policy. |
| `debtor-web/src/handlers/spendings.rs` | Spending create/update/delete are CSRF-only; `save_spending` parses and application-parses before synchronous dispatch. | Preserve all existing parsing and application ownership; reserve only after pre-dispatch validation and immediately before create/update/delete. |
| `debtor-web/src/templates.rs` | `AuthenticatedShell.submission_token` is documented as Sign-out-only; page templates carry shell values plus duplicated CSRF fields. | Rename/document as page-scoped authenticated protection and expose it to every unsafe form with the smallest coherent view-model change. |
| `debtor-web/templates/*.html` | Sign-out forms include token; other authenticated unsafe forms contain CSRF only. | Add exactly one shared token to every unsafe form on each page. Keep native action/method, security fields hidden, and no token-per-row issuance. |
| `debtor-web/src/handlers/response.rs` and `templates/error.html` | Generic errors return to `/groups`; Sign-out has a specialized plain-text HTMX response; no canonical form conflict recovery. | Add sanitized authenticated conflict/status presentation and allow-listed canonical recovery without replaying POST. Keep Login recovery separate. |
| `debtor-web/static/css/app.css` | Shell CSS has Sign-out-specific pending rules and older generic layout; login CSS is scoped. | Generalize pending/status/conflict/recovery styles without regressing Login or adding motion/overlays. Preserve Editorial Contrast and target/focus rules. |
| `debtor-web/src/state.rs` | `AppState` owns `AnonymousSubmissionTokenStore`. | Point the field at the generalized single store owner; do not add route stores. |
| `src/composition.rs` | Root composes one token store and passes a clone to runtime cleanup. | Preserve singleton composition and cleanup supervision with the generalized type. |
| `src/runtime.rs` | One supervised indexed token cleanup worker shares `CleanupHealth`. | Keep the worker/readiness/shutdown contract and clean both pools in bounded indexed work. |
| `debtor-web/src/handlers/test_support.rs` | Fakes track selected Group/Participant calls but not all mutation consumers. | Add counters/coordination needed to prove zero dispatch and one-winner races without a mocking framework. |
| `debtor-web/src/router.rs` | Routes and tests cover all unsafe routes, but most test forms submit CSRF only. | Update request helpers and extend route-level security, capacity, replay, dispatch exclusion, and markup tests. No route additions. |

Files that must remain unchanged unless compilation requires a narrow interface adjustment: `debtor-domain`, `debtor-application`, `debtor-infra`, migrations, `.sqlx`, provider code, dependency manifests, and lockfiles.

### Architecture Compliance

- Preserve `debtor (root) -> debtor-web/debtor-infra -> debtor-application -> debtor-domain`.
- Web owns Axum extraction, session/CSRF/submission-token mechanics, Askama projections, HTMX enhancement, and sanitized HTTP mapping. Root owns concrete composition and supervision. No tower-sessions, Axum, or token-store types may cross application-owned ports.
- Keep exactly one process-local token-store owner. Anonymous and authenticated pools are isolated internal state, not separate route stores.
- Keep `MutationPreflight` as the single 30-second pre-dispatch deadline. Its dispatch marker and token reservation must cross one atomic boundary from the handler's perspective.
- Preserve `303` successful mutation redirects, `422` retained-value validation, `409` conflict semantics, no-store security headers, and no raw diagnostics/log secrets.
- Do not add idempotency keys, response replay, persistent sessions, optimistic revisions, stale-edit conflicts, usernames, registration, participant auth, or multi-user authorization.
- No SQL or migration changes are expected. If implementation changes checked SQL accidentally, remove that change; do not expand this story into SQLx preparation work.

### Library And Framework Requirements

- Use the locked stack and current `Cargo.lock`: Rust 1.97.1/edition 2024, Axum 0.8.9, Askama 0.16.0, Tokio 1.53.1, Tower 0.5.3, tower-http 0.6.11, tower-sessions 0.15.0. Do not upgrade dependencies.
- Axum custom `FromRequest` body extractors must consume the body last and can read request extensions such as `MutationPreflight`; preserve the existing extractor ordering rather than introducing a second middleware pipeline. [Context7: `/tokio-rs/axum`, custom `FromRequest`, request extensions, and middleware composition]
- `SessionManagerLayer` owns cookie policy and session persistence. `Session::flush()` is authoritative for Sign out; `cycle_id()`/save remain Login-promotion behavior. Session-store expiry is treated as absent. [Context7: `/maxcountryman/tower-sessions`, `SessionManagerLayer`, `Session::flush`, `save`, `cycle_id`, and expiry semantics]
- Pinned self-hosted HTMX 2.0.10 and official `response-targets` 2.0.4 are the only permitted client-side enhancement. Use existing fixed local assets/digests; no CDN, custom extension, custom JS, inline script, or inline event attribute.

### UX And Security Guardrails

- Every unsafe authenticated form, including Sign out and confirmation forms, has one current session-backed CSRF and one current page-scoped submission token. Passwords are never retained.
- Invalid-token recovery says `409 Conflict`, states that no change occurred, and offers a native canonical reload that mints fresh protection. It must not resubmit the stale POST or accept arbitrary return URLs.
- Use stable server-owned IDs and the focus matrix: conflicts focus a conflict heading/status; validation focuses one linked alert summary or sole invalid control; pending retains the initiator; successful mutation keeps existing canonical redirect/focus behavior.
- Status nodes are scoped, `role="status"`, `aria-live="polite"`, and `aria-atomic="true"`; owning regions expose `aria-busy`. Expected enhanced errors target the stable status node; native full-page behavior remains equivalent.
- All controls and recovery links are at least 48 by 48 CSS pixels. Focus indicators are at least two CSS pixels with at least 3:1 adjacent contrast. Preserve 4.5:1 normal-text and 3:1 component/large-text contrast, square edges, rules, dark Editorial Contrast, and no decorative transitions/depth.
- Keep private HTML `no-store`, `nosniff`, `no-referrer`, and the prescribed CSP. Never log credentials, hashes, cookies, session IDs, CSRF tokens, submission tokens, client IPs, limiter keys, raw bodies, query strings, SQL/provider diagnostics, or request-derived values.

### Testing Requirements

- Test pure store/token invariants in `debtor-web` with injected clocks and deterministic state; assert exact capacities, physical deletion, pool isolation, session binding, and terminal reservation.
- Test extractor ordering with direct requests and fake application services. Assert no use-case invocation for oversized/undecodable bodies, failed authentication, malformed/duplicate/incorrect CSRF, invalid token, and pre-reservation route validation.
- Test concurrent same-token requests using `Barrier`, `Notify`, or a held operation. Assert one reservation and exactly one guarded use-case invocation; never rely on sleeps.
- Update every unsafe router test to include both protection fields and add cross-route stale-page replay assertions. Verify both native and enhanced responses, including scoped status markup and canonical recovery.
- Keep the root real-socket smoke path for Login -> authenticated page -> protected form boundary -> Sign out/restart behavior; do not claim browser geometry automation without a browser harness.
- Run, at minimum, `cargo fmt --all -- --check`, `cargo check --workspace --all-features --locked`, `SQLX_OFFLINE=true cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`, `cargo test --workspace --all-features --locked`, and `cargo run --bin architecture-check --locked`. No SQLx prepare command is needed unless SQL/migrations actually change.

### Project Structure Notes

- Feature modules remain plural and existing naming conventions remain: `*Store`, `*UseCases`, `*Template`, `*View`, and `*Input`.
- Expected implementation is confined to `debtor-web` plus narrowly required root composition/runtime and test updates. Do not create a new crate, new application port, new route, or new persistent table.
- Brownfield replacement is required: remove CSRF-only authenticated form paths, the temporary parallel Sign-out token namespace/API, route-local replay guards, generic `/groups` conflict recovery, and ambiguous post-dispatch retry behavior once the shared path replaces them.

### Previous Story Intelligence

- Story 1.4 established anonymous Login token issuance, one-per-session capacity, indexed expiry/cleanup, strict Login HTML, exact security headers, and native-first UX. Preserve it.
- Story 1.5 established trusted-client resolution, strict Login admission, atomic anonymous token reservation immediately before password verification, bounded password verification, limiter behavior, durable Login promotion, and terminal replay semantics. Reuse rather than generalize by duplicating.
- Story 1.6 added the first authenticated Sign-out consumer with a temporary independent namespace, terminal reservation before `Session::flush`, session deletion before local flush to prevent stale resurrection, shell status/HTMX behavior, and restart/session invalidation tests. Its explicit scope boundary says this story owns the general authenticated pool and non-Login extractor.
- Prior review fixes are binding: reserved tokens cannot be replaced during rendering; stale saves cannot resurrect deleted sessions; authenticated expiry refresh is explicitly saved; capacity failure is sanitized; protected HTML has pinned asset integrity; concurrent replay uses deterministic coordination; browser geometry remains manual unless a harness exists.

### Git Intelligence

- Current `HEAD` is `fdaca09` (`feat: bmad implement sprint 1-6`), which includes Story 1.6 and its web/root changes. The worktree was clean before this story document was created.
- Build on current source, especially `submission_tokens.rs`, `forms.rs`, `handlers/auth.rs`, `middleware.rs`, `router.rs`, templates, and tests. Do not reconstruct the earlier authentication implementation from planning prose.
- Recent implementation conventions use direct Axum router tests, `ReapingMemoryStore`, injected clocks, atomics, and barriers. Continue those patterns and avoid a mocking framework.

### Latest Technical Information

- Current Context7 documentation confirms Axum 0.8 custom `FromRequest` implementations can split a request into parts/body, use extensions, and return a `Response` rejection; body-consuming extraction remains ordered last.
- Current Tower Sessions documentation confirms cookie attributes belong to `SessionManagerLayer`, expired loads are treated as absent, `Session::flush()` deletes/invalidates server-side session state, and `cycle_id()` requires `save()` for durable rotation. These semantics preserve the completed Login/Sign-out behavior; this story only generalizes submission-token ownership around them.
- No dependency or lockfile change is justified. `Cargo.lock` and the project context remain authoritative.

### References

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
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md#HTTP and Session Outcomes`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md#Interaction Focus Matrix`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md#Accessibility Floor`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md#Interaction Primitives`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/DESIGN.md#Layout & Spacing`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/DESIGN.md#Components`]
- [Source: `_bmad-output/implementation-artifacts/1-6-maintain-an-authenticated-session-and-sign-out.md#Scope Boundary`]
- [Source: `debtor-web/src/submission_tokens.rs`]
- [Source: `debtor-web/src/forms.rs`]
- [Source: `debtor-web/src/middleware.rs`]
- [Source: `debtor-web/src/handlers/auth.rs`]
- [Source: `debtor-web/src/handlers/groups.rs`]
- [Source: `debtor-web/src/handlers/memberships.rs`]
- [Source: `debtor-web/src/handlers/participants.rs`]
- [Source: `debtor-web/src/handlers/spendings.rs`]
- [Source: `debtor-web/src/handlers/response.rs`]
- [Source: `debtor-web/src/templates.rs`]
- [Source: `debtor-web/templates/*.html`]
- [Source: `static/css/app.css`]
- [Source: `src/composition.rs`]
- [Source: `src/runtime.rs`]
- [Source: `specs/adr/0001-foundation-architecture.md#10. Fixed HTTP resource budgets`]
- [Source: Context7 `/tokio-rs/axum`]
- [Source: Context7 `/maxcountryman/tower-sessions`]

## Dev Agent Record

### Agent Model Used

openai/gpt-5.6-luna

### Debug Log References

- Ultimate context engine analysis completed - comprehensive developer guide created.
- Selected the first backlog story in the complete sprint order: `1-7-extend-replay-protection-beyond-login-and-sign-out`.
- Loaded project context, normative design contract, PRD, architecture spine, final UX contracts, Epic 1 and cross-cutting planning context, sprint change proposal, completed Story 1.6, current web/root implementation files, and recent git history.
- Consulted current Context7 documentation for Axum 0.8 custom extractors/middleware composition and tower-sessions session lifecycle APIs. No dependency update is authorized.

### Completion Notes List

- Ultimate context engine analysis completed - comprehensive developer guide created.
- Generalized the web submission-token owner into isolated anonymous and authenticated pools with indexed expiry, 1,024 global/32-per-session authenticated bounds, terminal reservation, and session-flush cleanup.
- Extended the strict authenticated form boundary and moved all current authenticated mutation handlers to reserve immediately before dispatch while preserving Login and Sign-out behavior.
- Added page-scoped protection to every authenticated unsafe form and sanitized native conflict recovery with stable status semantics.
- Added store, extractor/parser, router, and deterministic concurrency coverage; the final web suite has 69 passing tests.
- Validation passed: workspace tests, offline Clippy with warnings denied, formatting check, architecture fitness, and `git diff --check`.
- Browser geometry remains unautomated because no executable browser harness is present.

### File List

- `_bmad-output/implementation-artifacts/1-7-extend-replay-protection-beyond-login-and-sign-out.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `debtor-web/src/forms.rs`
- `debtor-web/src/handlers/auth.rs`
- `debtor-web/src/handlers/groups.rs`
- `debtor-web/src/handlers/memberships.rs`
- `debtor-web/src/handlers/participants.rs`
- `debtor-web/src/handlers/response.rs`
- `debtor-web/src/handlers/spendings.rs`
- `debtor-web/src/handlers/test_support.rs`
- `debtor-web/src/router.rs`
- `debtor-web/src/state.rs`
- `debtor-web/src/submission_tokens.rs`
- `debtor-web/src/templates.rs`
- `debtor-web/templates/confirm.html`
- `debtor-web/templates/error.html`
- `debtor-web/templates/group.html`
- `debtor-web/templates/group_edit.html`
- `debtor-web/templates/groups.html`
- `debtor-web/templates/participant_edit.html`
- `debtor-web/templates/participants.html`
- `src/composition.rs`
- `src/main.rs`
- `static/css/app.css`

No changes were made to domain/application/infra crates, migrations, `.sqlx`, provider code, dependency manifests, or lockfiles.

### Change Log

- 2026-08-13: Implemented authenticated page-scoped replay protection, generalized token storage, strict dispatch integration, form coverage, conflict recovery, and invariant tests; status moved to `review`.

- 2026-08-13: Addressed code review findings - 11 items resolved.
