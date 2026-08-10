# Final Closure Reconciliation: PRD Addendum Against Polished UX Spines

## Inputs Re-read

- Confirmed source: `/home/sebr/projects/pet/debtor/_bmad-output/planning-artifacts/prds/prd-debtor-2026-08-10/addendum.md`
- Decision history: `/home/sebr/projects/pet/debtor/_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/.memlog.md`
- Polished visual spine: `/home/sebr/projects/pet/debtor/_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/DESIGN.md`
- Polished experience spine: `/home/sebr/projects/pet/debtor/_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md`
- Previous reconciliation: overwritten by this report

## Verdict

**CLOSED.** The latest confirmed addendum, UX memlog, polished `DESIGN.md`, and polished `EXPERIENCE.md` are reconciled. The later progressive-HTMX, official `response-targets`, native fallback, CSP, full-page Spending form, and single-use submission-token contracts are synchronized and technically coherent. All prior critical, high, medium, and low findings remain closed. No new finding was introduced by polishing.

The UX spines contain the source contracts that affect visible behavior, states, accessibility, rendering, privacy, and interaction. Internal architecture, persistence, provider/cache implementation, deployment, toolchain, testing, and operational mechanics remain correctly inherited through source references.

## Later Contract Verification

### HTMX Is Progressive, Not Required

**Status: Closed and aligned.**

The current addendum and spines consistently supersede the earlier required-HTMX decision:

- Semantic server-rendered HTML and valid native links/forms are the complete interaction baseline.
- Pinned self-hosted HTMX may enhance Group section navigation and allocation previews, but no task depends on it.
- Native `href`, form `action`, method, return destination, validation response, and full-page response remain authoritative.
- Group destinations are native links; enhanced navigation uses the same URL and server response.
- Add Spending is a native link to a focused full-page form, not an HTMX-only action.
- Spending Preview has an explicit native submit that rerenders a reviewed full-page state.
- Pagination and Debt mode have native links/forms and optional enhanced replacement.
- Native navigation presents server, transport, runtime, and validation failures when enhancement is absent or fails.
- HTMX history snapshots are disabled for private ledger content; Back/Forward relies on encoded URL state and browser-native restoration.

`DESIGN.md` reinforces the same boundary: the Spending form remains a full page at every width, enhanced fragments are visually identical to native responses, and no core action may depend on HTMX.

### Official `response-targets` Extension

**Status: Closed and aligned.**

The addendum and `EXPERIENCE.md` consistently allow only:

- One pinned self-hosted HTMX asset.
- Its pinned official `response-targets` extension.
- Declarative routing of expected `4xx`/`5xx` fragments into stable scoped status targets.

The UX consequences are explicit:

- One stable **Request status** node uses `role="status"`, `aria-live="polite"`, and `aria-atomic="true"`.
- The owning region toggles `aria-busy`.
- Expected enhanced errors retain the invoker's focus and announce once.
- Enhanced response errors clear stale pending/Updating state and preserve native recovery.
- Validation summaries and urgent session loss use their stronger alert/heading treatments rather than being flattened into routine status.
- No custom extension, custom response handler, external library, inline script, or inline script attribute is permitted.

The extension changes response placement only; it does not replace native full-page error behavior.

### Native Spending And Allocation Fallback

**Status: Closed and aligned.**

The current spines remove every stale modal/overlay requirement:

- Spending create/edit is a focused full-page form with one document scroll owner.
- No modal semantics, focus trap, scrim, sheet shadow, or side-sheet transformation remains.
- Cancel is a native allow-listed return link.
- Native Preview submits the complete form and rerenders reviewed, non-editable input with Approve and Edit allocation.
- Native approval cannot outlive the reviewed input.
- Optional HTMX preview swaps only derived amount cells, approval state, and one status node.
- Focused controls stay outside swaps; latest input wins and superseded responses do not update the page.
- Focus, caret, selection, software keyboard, allocation scroll, page scroll, and active row remain stable during enhanced previews.
- Approve remains disabled while preview is pending, stale, invalid, or superseded.
- At 320px, the action bar provides Cancel, Preview/Edit allocation, and Approve as three equal 48px-minimum controls below a separate Total/status row.
- At 320px and 400% zoom, the allocation table remains semantic inside a labeled, focusable horizontal scroll region while page-level horizontal scrolling stays absent.

The polished visual and behavioral contracts therefore preserve a complete native path while allowing a bounded enhancement.

### CSP And Asset Privacy

**Status: Closed and aligned.**

The addendum and `EXPERIENCE.md` use the same fixed CSP:

`default-src 'none'; script-src 'self'; script-src-attr 'none'; connect-src 'self'; style-src 'self' 'unsafe-inline'; form-action 'self'; base-uri 'none'; frame-ancestors 'none'`

This policy matches the later interaction contract:

- `script-src 'self'` permits pinned self-hosted HTMX and the pinned official extension.
- `connect-src 'self'` permits only same-origin enhanced requests.
- `script-src-attr 'none'` forbids inline event/script attributes without blocking declarative `hx-*` attributes.
- No CDN, external script, external HTMX connection, custom application JavaScript, inline script block, or custom extension is allowed.
- Every task remains available when the permitted scripts do not execute.
- Login and authenticated HTML remain `no-store`, unframeable, no-referrer, and protected from MIME guessing.
- Static assets, including both pinned client assets, neither create nor load sessions.

The prior `script-src 'none'` contradiction and the superseded JavaScript-required `<noscript>` state are absent from the current source and spines.

### Single-Use Submission Token

**Status: Closed and aligned.**

The addendum and `EXPERIENCE.md` consistently define a submission token separate from CSRF for every unsafe form:

- The token is bounded, expiring, session-bound, and single-use.
- Every rendered unsafe form receives one token, including login, Group/Participant create/edit, Spending Approve, confirmations, restore, and Sign out.
- Valid validation errors preserve the token because no mutation dispatch has begun.
- Immediately before the first state-changing use-case call, the server atomically reserves the token.
- After reservation, the request remains pending until definitive commit or rollback; no generic timeout produces an ambiguous outcome.
- Missing, unknown, expired, reserved, or consumed tokens return announced `409 Conflict` before use-case invocation.
- Conflict recovery is a native canonical-form reload that issues a fresh token, not automatic resubmission.
- The token prevents duplicate dispatch but never replays an earlier response and is not presented as an idempotency key.
- First activation disables the unsafe initiator; repeated activations are suppressed or coalesced while pending.
- Before dispatch, rejected work states that no change occurred and retry is safe; after dispatch, the pending state remains until the definitive response.
- Definitive mutation failure retains safely decoded form values and renders a usable unsafe form under the every-rendered-form token rule.

The token is never conflated with the session-backed CSRF synchronizer token. CSRF still protects request authenticity; the submission token separately prevents duplicate dispatch.

## Prior Finding Verification

### Input And Financial Validation

**Status: Closed.**

- Group/Participant names and Spending Description have visible trimming, non-empty, Unicode-length, label, guidance, retained-value, and associated-error behavior.
- Date format, earliest date, UTC default, Participant color normalization, amount maximum, positivity, and currency precision remain fixed.
- Exactly one Payer, nonempty Participant-unique Shares, no zero Share, positive proportional weights, exact minor-unit conservation, and no rounding-to-pass remain explicit.
- Proportional/Exact behavior, Exact edit reopening, archived-role restrictions, and editable corrected Source Currency remain aligned.
- Multiple errors use one focused linked alert summary; row errors remain attached to the owning input.

### HTTP, Session, Privacy, And Timeout Outcomes

**Status: Closed.**

- Route-owned validation retains safely decoded non-password values and uses the upstream `422` contract.
- Successful mutations use canonical redirect behavior and cannot be replayed by refresh.
- Archived mutation routes and invalid submission tokens produce pre-dispatch `409` outcomes with no use-case call.
- Debts rate failure and authenticated-session capacity preserve retryable `503` behavior; login limiting preserves retryable `429` behavior.
- Malformed, missing, duplicate, unknown, and invalid-CSRF input is rejected before route parsing or dispatch.
- Anonymous/authenticated expiry, restart invalidation, rotation, CSRF rotation, save-before-redirect, Sign out flush, cookie privacy, no-store, no-referrer, no-framing, and session-free assets remain inherited with their UX consequences explicit.
- Oversized requests, safe-read timeouts, Debts timeout, mutation pre-dispatch deadline, and no generic timeout after dispatch remain represented by safe native/scoped outcomes.
- Every unsafe result is definitive: pre-dispatch non-mutation, commit, or rollback.

### History And Concurrent Writes

**Status: Closed.**

- Transactions remain fixed 25-item keyset pages ordered by `(spent_date DESC, id DESC)`.
- Native and enhanced pagination preserve the same URL/page contract.
- Create/edit returns to the canonical row page; deletion avoids an empty out-of-range page.
- Among admitted valid concurrent mutations, the last committed result is displayed.
- No optimistic revision or stale-edit-conflict state exists.

### Lifecycle And Historical Integrity

**Status: Closed.**

- New Groups take a valid name, persist USD, and enter Manage; established Groups enter Summary.
- Archived collections remain separate and archived Groups retain readable navigation with mutation controls suppressed.
- Participant archival requires a complete all-time Historical exact-zero Group Currency Balance.
- Missing eligible rates block archival with retryable non-mutation feedback.
- Confirmation and focus-return behavior are explicit; restore has no Balance eligibility requirement.
- Referenced Group deletion remains unavailable; history-free deletion names affected unreferenced Participants.
- Current Participant names and archived identity remain visible throughout history and calculations.
- The three earlier source-synchronizing overrides remain aligned: archive eligibility, Group creation/archive views, and single-Payer Proportional/Exact allocation.

### Rates, Summary, And Debts

**Status: Closed.**

- Summary preserves current-month Group and per-Payer Source Currency totals independently of conversion.
- Group Currency totals remain separate.
- Ready, Updating, stale, provisional, unavailable, timeout, and invalid-calculation states remain complete-or-no-result.
- Prior values remain during Updating only when their full visible calculation context matches.
- Historical/Current behavior, stale eligibility windows, complete rate identity, future provisional state, disclosure, and no manual Retry remain explicit or inherited with visible consequences fixed.
- Completed Debts contain one Balance per Participant, exact zero sum, deterministic positive complete Transfers, no repeated pair, and at most `n - 1` Transfers.
- Completion order cannot alter values, transfer order, rates, or warnings.
- Rate, arithmetic, quantization, settlement, or consistency failure displays no partial calculation.

### Accessibility, Responsive, And Visual Contracts

**Status: Closed.**

- All controls, including links, summaries, fields, row actions, radio/checkbox labels, navigation, and recovery links, render at least 48 by 48 CSS pixels at 320px and 400% zoom.
- Focus outlines, boundaries, text, warnings, meaningful graphics, and action colors retain their documented contrast floors.
- The narrow Group shell is intrinsically measured, respects safe areas, does not overlay content, and permits wrapped labels.
- The focused Spending form, sticky in-flow action bar, software-keyboard behavior, scroll margins, and maximum OMR Total remain specified.
- The allocation table has semantic headings, a labeled keyboard-focusable scroll region, sticky Participant identity, long-name handling, and no page-level horizontal scroll.
- Native and enhanced focus behavior is owned by one stable-ID matrix; private HTMX snapshots remain disabled.
- Request and derived statuses are scoped, polite, atomic, and announce once; validation and urgent session loss retain stronger semantics.
- Archived state uses visible associated text rather than invented ARIA.
- Participant color remains supplemental and never carries identity or state alone.
- Polished mock references are subordinate to the spines and are linked only where they illustrate already-extracted contracts.

## Memlog Supersession Verification

The memlog chronology is coherent when later overrides are applied:

- The original compact Group shell, persistent Add Spending reachability, Editorial Contrast identity, allocation rules, rate states, and lifecycle overrides remain active.
- The prior native-overlay decision is superseded by the focused full-page Spending form.
- The prior required-HTMX/unsupported-`noscript` decision is superseded by progressive enhancement and mandatory native links/forms.
- The later 320px allocation/action-bar, latest-input-wins preview, focus preservation, response-targets, and single-use submission-token decisions are present in the polished spines.
- The addendum reflects the same latest overrides, so no UX-only exception remains.

No dropped qualitative idea, stale active decision, or unsynchronized later override was found.

## Inherited Implementation-Only Contracts

The following remain correctly inherited rather than duplicated:

- Crate boundaries, dependency direction, ports, adapters, clocks, and use-case testability.
- Decimal implementation, canonical SQLite text storage, corruption decoding, exact Rust aggregation, IDs, and internal ordered collections.
- SQLite WAL, `synchronous=FULL`, foreign keys, write gate, lock timeout, schema checks, SQLx macros, and offline metadata.
- Snapshot lifetime, direct repository loading, provider-I/O separation, race-safety implementation, and transaction construction.
- Submission-token storage structure, bound/capacity values, expiry index, and atomic reservation mechanism beyond their stated interaction consequences.
- Error-library selection, diagnostic allowlists, and adapter sanitization implementation; visible privacy and safe-error behavior is fixed.
- Provider JSON decoding, cache topology, LRU bounds, single-flight, provider request limits, and concurrency caps.
- Argon2 profile, verification concurrency, session cleanup/index mechanics, and limiter storage structures.
- Proxy trust resolution, TLS/HTTP transport, early-data enforcement, admission permit counts, probes, readiness, shutdown, and checkpointing.
- Local run, pre-release compatibility, toolchain, dependencies, workspaces, testing, linting, documentation, and validation commands.

## Final Disposition

- **Critical findings:** 0.
- **High findings:** 0.
- **Medium findings:** 0.
- **Low findings:** 0.
- **Later HTMX/response-targets/native fallback/CSP contracts closed:** Yes.
- **Single-use submission-token consequences closed:** Yes.
- **All earlier UX-significant findings closed:** Yes.
- **Contradictions with latest memlog decisions:** 0.
- **Dropped qualitative ideas:** 0.
