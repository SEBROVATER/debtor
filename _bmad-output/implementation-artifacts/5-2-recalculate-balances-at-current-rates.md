---
story_key: 5-2-recalculate-balances-at-current-rates
story_id: 5.2
epic: 5
status: done
created: 2026-08-19
baseline_commit: 15f4b48a5a3c4dfae151cba014d5b33ce0c40b56
completion_note: Ultimate context engine analysis completed - comprehensive developer guide created
---

# Story 5.2: Recalculate Balances at Current Rates

Status: done

## Story

As the administrator,
I want to recalculate all-time Balances using the current UTC rate context,
so that I can compare historical obligations with what settlement means today.

## Acceptance Criteria

1. Given Debts defaults to Historical mode, when Current is selected through its native form, every Spending uses one captured UTC calculation date as its requested and fetch context regardless of Spending date. `rate_mode=current` remains in that result URL, while a later request without it defaults to Historical. Current is never persisted.
2. Given multiple Spendings share a Source/Target pair in Current mode, when contexts are assembled, they deduplicate to one current calculation-date context. Same-currency pairs synthesize exact rate `1` without a provider request. Existing provider concurrency and per-key single-flight bounds remain enforced, and one immutable quote bundle serves the calculation.
3. Given a fresh Current quote fails after UTC rollover, when refreshable evidence exists for the same Source/Target pair and current class, only the latest prior current-class quote is eligible through the inclusive seven-UTC-day boundary after its prior effective fetch date. It remains disclosed as stale with its original evidence.
4. Given prior evidence is older than seven UTC days, belongs to another pair/class, or is fixed-past Historical evidence, when fresh Current resolution fails, no fallback is used. Debts returns retryable `503 Service Unavailable` with no partial Balances or Settlement Transfers.
5. Given complete Current evidence, when all-time positions are converted and quantized, the existing checked exact `Decimal` arithmetic, deterministic ordering, signed largest-remainder quantization, Participant-ID tie-breaking, and exact-zero-sum invariant apply. Every Group-owned Participant, including archived, inactive, and zero-activity identities, remains present.
6. Given Current succeeds, when its result renders, it discloses Current mode, calculation UTC time, target Group Currency, unique current/synthetic rates, and one Group-level stale warning when applicable. No Spending Source Currency, allocation, Participant identity, or persisted setting changes.
7. Given the provider later revises a rate, when ordinary Debts has no persisted rate evidence, a later calculation may reflect that revision. Each result remains internally immutable and reproducible from displayed context; no manual refresh or saved Current preference is introduced.
8. Given Current is selected through the safe native form, when the native response renders, Current remains in the URL and the result heading may receive focus. Enhanced success and expected enhanced errors keep the activated radio mounted and focused outside the result replacement while one scoped polite atomic server-rendered status announces the complete result or no-partial failure; individual amounts are never live regions.
9. Given an enhanced Current calculation is pending, HTMX's request class hides prior financial content and shows one scoped Updating placeholder. The completed server-rendered replacement restores complete results or the scoped no-partial failure. This transition does not dynamically change `aria-busy`, retain client-side financial data, use manually authored application JavaScript, inline scripts or event handlers, application-owned HTMX event handlers, a custom HTMX extension, or imperative post-swap behavior. Other official extensions require explicit design and security approval before addition.
10. Given Current succeeds with fresh or eligible stale evidence, Current is visibly and programmatically selected; Balances preserve exact direction and currency presentation; disclosure identifies current/synthetic evidence and stale dates; and one Group-level warning announces once.
11. Given Current is unavailable, native or enhanced output contains no partial Balance or Transfer. Enhanced expected failures replace only the result region, retain the activated mode-control focus, and announce the server-rendered scoped status; native full-page failures may autofocus their heading. Revisit/reselection recalculates safely, with no manual Retry control, persisted preference, client-side financial state, or imperative post-swap behavior.

Requirements: `SPEC-FR75..SPEC-FR83`, `SPEC-NFR4..SPEC-NFR5`, `SPEC-NFR10`, `SPEC-NFR12..SPEC-NFR13`, `SPEC-NFR25..SPEC-NFR30`, `SPEC-NFR32..SPEC-NFR34`; UX contracts `UX-SHELL-01`, `UX-TARGET-01`, `UX-FOCUS-01`, `UX-STATUS-01`, `UX-RESPONSIVE-01`, `UX-VISUAL-01`.

## Scope Boundary

- Extend Story 5.1's exact Historical calculation with request-scoped Current rate context; do not create a second calculator.
- Reuse the existing `LedgerSnapshotReader`, injected `Clock`, `ExchangeRateProvider`, `RateQuote`, Frankfurter refreshable cache, single-flight, provider semaphore, balance aggregation, quantization, and no-partial error path.
- Story 5.3 owns settlement semantics. Preserve existing complete-result behavior but do not add repayment, paid, settled, checkpoint, date-range, or persistence state.
- Do not add a database preference, migration, `.sqlx` change, manual refresh endpoint/control, manually authored application JavaScript, inline scripts/event handlers, application-owned HTMX event handlers, custom or unapproved official HTMX extensions, client-side financial state, imperative post-swap behavior, new route, new provider/cache, or a separate Group/Participant financial read.
- Do not change stored Spending Source Currency, allocations, dates, names, archive state, or Group Currency.
- The enhanced pending projection is CSS-only: HTMX's built-in request class displays an Updating placeholder. Do not reintroduce `hx-on`, other inline event attributes, application-owned HTMX event handlers, custom or unapproved official extensions, CSP relaxation, dynamic `aria-busy`, client-side financial retention, or imperative post-swap behavior.

## Tasks / Subtasks

- [x] Audit the existing 5.1 debt path and Current compatibility before editing (AC: 1-5, 9-11)
  - [x] Read `debtor-application/src/debts.rs`, domain balance/settlement rules, snapshot repository, Frankfurter adapter, Debts handler/template/CSS, response mapping, router, composition, and current tests.
  - [x] Preserve one complete snapshot and release it before any provider request.
  - [x] Confirm one shared process-local clock, snapshot reader, provider/cache, and debt service remain composed in `src/composition.rs`.
- [x] Harden Current application orchestration (AC: 1-5, 7)
  - [x] Capture one injected UTC instant/date before snapshot/rate work; pass that date to every Current context and provider call.
  - [x] Keep Historical behavior unchanged: each Spending date remains its requested date and future Historical contexts remain provisional.
  - [x] Deduplicate and deterministically order `(base, target, requested_date, fetch_date)` contexts before fetching; retain at most four concurrent requests through the existing stream bound.
  - [x] Synthesize same-currency `RateQuote` with exact `Decimal::ONE`, no provider access, and explicit disclosure metadata.
  - [x] Validate quote base/target/requested date, positive rate, fetch/effective dates, Current stale metadata, and truthful provisional/stale flags. Current stale evidence retains its original fetch/effective evidence while exposing the Current requested context.
  - [x] Ensure Current never accepts stable fixed-past evidence; only the provider-owned current-class pair fallback may satisfy it.
  - [x] Reuse the existing exact payer-minus-share source-net aggregation, joint quantization, deterministic ordering, zero-sum validation, and no-partial failure mapping.
  - [x] Seed all snapshot Participants at zero, including archived/inactive and zero-activity identities.
- [x] Verify or correct provider Current fallback policy (AC: 2-4, 7)
  - [x] Keep cache/fallback policy in `debtor-infra/src/exchange_rates/frankfurter.rs`, not application or web.
  - [x] After refresh failure, select only the latest prior current-class quote for the same Source/Target pair, with inclusive seven-day eligibility from prior fetch/effective date.
  - [x] Reject wrong pairs, fixed-past stable entries, future-class entries, day-eight evidence, and invalid/future effective dates.
  - [x] Preserve revised provider evidence on later successful calculations and existing provider bounds, lexical Decimal decoding, response-size limit, timeout, LRU, and single-flight behavior.
- [x] Complete Current web projection and native/enhanced parity (AC: 1, 6, 8-11)
  - [x] Keep absent/`historical` query values Historical; parse only `current` as Current; unknown values remain `400` and enhanced failures replace only `#debts-results`.
  - [x] Render mode-specific heading/status/context instead of hardcoded Historical copy. Keep `?rate_mode=current` in the native result URL while a new request without it is Historical.
  - [x] Preserve Balance -> Settlement -> rate disclosure order, current Participant labels, archived labels, exact currency/sign/direction text, and no partial state.
  - [x] Disclose Current calculation time, target currency, unique rates, synthetic same-currency evidence, and one Group-level stale warning without making each amount live.
  - [x] Keep the mode form outside the replaceable result region so enhanced replacement keeps the selected radio mounted. Use only pinned HTMX and its official response-targets extension.
  - [x] Render stable result/status IDs with `role="status"`, `aria-live="polite"`, and `aria-atomic="true"`; retain static final-state `aria-busy="false"` and native heading focus.
  - [x] Use HTMX's request class and CSS to hide financial content and show the scoped Updating placeholder while pending. Do not attempt dynamic `aria-busy`, compatible-result retention, inline handlers, custom JavaScript, or client state.
  - [x] Keep the 90-second Debts safe-read timeout and include sanitized attempted mode/time context in unavailable and timeout responses without raw causes, IDs, provider URLs, query strings, or target-currency leakage.
- [x] Add invariant-owning regression tests (AC: all)
  - [x] Application tests use injected clock, snapshot, and provider fakes: fixed Current date across differing Spending dates, context deduplication, same-currency isolation, immutable quote metadata, completion-order independence, archived/inactive/zero-activity identities, stale days 0-7 accepted/day 8 rejected, fixed-past isolation, revisions, and no partial results.
  - [x] Infra tests cover current rollover fallback, latest prior current-class selection, pair/class isolation, inclusive boundary, revised quote evidence, and preservation of existing bounds/cache/single-flight tests.
  - [x] Web tests cover Historical default, Current URL/selection, subsequent default reset, native/enhanced parity, final status semantics, CSS-only Updating placeholder, absence of inline handlers/custom scripts, unavailable no-partial output, and sanitized timeout/error context.
  - [x] Add or retain responsive checks for 320 CSS pixels and 400% zoom: controls remain 48 by 48, long names/rates/warnings/amounts wrap, and no page-level horizontal scroll exists.
  - [x] Preserve domain exactness tests, root real-socket smoke, architecture fitness, security headers, readiness, Summary, Transactions, and Story 5.1 regressions.
- [x] Run required validation (AC: all)
  - [x] `cargo fmt --all -- --check`
  - [x] `cargo check --workspace --all-features --locked`
  - [x] `SQLX_OFFLINE=true cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`
  - [x] `cargo test --workspace --all-features --locked`
- [x] `cargo run --bin architecture-check --locked`
- [x] Never use `cargo build --release` for validation.

### Review Findings

- [x] [Review][Patch] Preserve rate-mode control focus in enhanced debt results [debtor-web/templates/debts.html:43]
- [x] [Review][Patch] Announce the group-level stale-rate warning with the final enhanced result status [debtor-web/templates/debts.html:44]
- [x] [Review][Patch] Reject overflowing stale-current quote expiry dates without panicking [debtor-application/src/debts.rs:290]

## Dev Notes

### Developer Context

Story 5.1 supplies the vertical slice. `DebtService` must use its injected clock once, map every Current Spending to that date, sort and deduplicate contexts, synthesize identity rates, fetch at most four non-identity quotes concurrently, seed every snapshot Participant, quantize exact balances, and reuse the existing settlement calculation. Extend that path; do not fork it.

Current fallback belongs to the Frankfurter adapter. It is pair/current-class only and cannot borrow fixed-past Historical evidence. Stale Current quotes retain prior fetch/effective evidence while exposing the new requested date.

The Debts form is always natively valid. HTMX may replace `#debts-results`; its built-in request class is sufficient to show the Updating placeholder and hide old financial values. CSS cannot mutate ARIA attributes. Do not use `hx-on`, inline handlers, custom script, or a custom extension to toggle `aria-busy`. Completed fragments render `aria-busy="false"`, the stable polite atomic status reports the final outcome, and enhanced errors replace the same result region without partial rows.

### Architecture Compliance

- Preserve `root -> debtor-web/debtor-infra -> debtor-application -> debtor-domain`. Domain owns pure arithmetic; application owns Current context policy and ports; infra owns provider/cache; web owns HTTP/rendering; root only composes.
- Use exact checked `rust_decimal::Decimal`; never use floats, SQL monetary aggregates, per-Spending rounding, saturating arithmetic, or failed-conversion-to-zero.
- Keep `RateQuote` and `CalculationContext` immutable per result. Sort contexts, rates, Participants, and visible rows explicitly; provider completion order must not change output.
- Materialize a complete snapshot and release its transaction before network I/O. Preserve all historical identities.
- Missing eligible Current quote is retryable `Unavailable`/HTTP `503`; calculation/storage failures are sanitized and never display partial data.
- Current is request state only. Do not persist it or add manual refresh, mutation epoch/client retention, rate preferences, repayment state, multi-user abstractions, or compatibility shims.

### Files To Update And Preserve

| Path | Ownership and guardrail |
|---|---|
| `debtor-application/src/debts.rs` | `RateMode`, contexts, quote validation, snapshot orchestration, exact balances, and transfers. Keep one calculation path. |
| `debtor-infra/src/exchange_rates/frankfurter.rs` | Refreshable current/future cache, rollover, provider bounds, single-flight, and stale fallback. Keep fallback policy here. |
| `debtor-web/src/handlers/debts.rs` | Strict query parsing, calculation projection, and no separate financial identity read. |
| `debtor-web/src/handlers/response.rs` | Sanitized timeout/failure mapping and HX-aware scoped no-partial fragments. |
| `debtor-web/src/templates.rs` and `debtor-web/templates/debts.html` | Typed mode-aware projection, stable result/status semantics, final static `aria-busy`, and native/HTMX parity. |
| `static/css/app.css` | CSS-only `htmx-request` Updating placeholder, focus, 48px targets, wrapping, and no page-level horizontal overflow. |
| `debtor-web/src/router.rs` | Existing authenticated GET route, 90-second timeout, and route-level parity regressions. |
| `src/composition.rs` | One shared snapshot reader, provider/cache, clock, and `DebtService`; no duplicate process-local owner. |

No database, migration, `.sqlx`, dependency, route, or composition change is expected.

### Library And Framework Requirements

- Preserve Rust 1.97.1 edition 2024, Axum 0.8.9, Askama 0.16.0, Tokio 1.53.1, SQLx 0.9.0, reqwest 0.13.4, `rust_decimal` 1.42.1, HTMX 2.0.10, and response-targets 2.0.4. Add no dependency.
- `Decimal::checked_add`, `checked_sub`, `checked_mul`, and `checked_div` return `Option`; map failure to the fixed safe calculation reason.
- Askama templates remain typed and escaped. Keep financial policy out of templates.
- HTMX `hx-indicator`/its request class supports CSS request presentation only; it does not dynamically set `aria-busy`. The pinned response-targets extension is the only permitted extension.

### Testing Requirements

- Use injected `Clock`, `LedgerSnapshotReader`, and `ExchangeRateProvider` fakes for application policy tests; no Axum, SQLite, network, wall clock, or sleeps.
- Infra owns provider/cache rollover, pair/class isolation, stale boundary, response bounds, lexical Decimal, LRU, global bound, and single-flight tests.
- Web owns strict query parsing, mode URL/default reset, mode selection, final result/status markup, CSS-only Updating state, no-inline-handler contract, no-partial failures, and sanitized context.
- Domain owns exact aggregation, signed largest remainder, precision, tie ordering, overflow, and zero-sum invariants.
- Preserve root smoke, architecture fitness, strict offline Clippy, security/readiness, Summary, Transactions, and Story 5.1 regressions.

### Previous Story Intelligence

- Story 5.1 established snapshot-owned Participant projection, exact payer-minus-share aggregation, zero identity seeding, no unrelated identity reads, and no-partial enhanced failures.
- Story 4.3 owns refreshable rollover, seven-day eligibility, fixed-past isolation, lexical Decimal handling, bounded provider concurrency, and source continuity.
- The prior Story 5.2 follow-up removed CSP-blocked inline HTMX handlers. Do not restore them; the CSS-only Updating projection is now the normative design.

### Anti-Patterns To Avoid

- Do not create a Current-specific calculator, provider, cache, route, persistence table, refresh button, JavaScript state machine, or separate identity read.
- Do not use Historical fixed-past evidence as a Current fallback, accept stale evidence after day seven, or accept wrong pair/class evidence.
- Do not round individual conversions, use floats, aggregate in SQL, iterate unordered maps for visible output, or return partial Balances/Transfers.
- Do not use `hx-on`, event attributes, custom JavaScript, custom HTMX extensions, CSP relaxation, or dynamic ARIA mutation to model the Updating state.

### References

- [Source: `specs/design.md#Rates And Settlements`]
- [Source: `_bmad-output/planning-artifacts/epics.md#Story 5.2: Recalculate Balances at Current Rates`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md#Financial Presentation`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md#Requests and Calculation Modes`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-11 - Native HTML authority`]
- [Source: `_bmad-output/project-context.md#Critical Don't-Miss Rules`]
- [Source: `_bmad-output/implementation-artifacts/5-1-calculate-exact-historical-balances.md`]
- [Source: `debtor-application/src/debts.rs`]
- [Source: `debtor-infra/src/exchange_rates/frankfurter.rs`]
- [Source: `debtor-web/src/handlers/debts.rs`]
- [Source: `debtor-web/src/handlers/response.rs`]
- [Source: `debtor-web/templates/debts.html`]
- [Source: `static/css/app.css`]
- [Source: Context7 `/bigskysoftware/htmx/v2.0.4`, consulted 2026-08-19]

## Dev Agent Record

### Agent Model Used

openai/gpt-5.6-terra

### Debug Log References

- Resolved `bmad-create-story` customization: no activation prepend/append or completion terminal steps configured.
- Loaded complete sprint status, project context, normative design, Epic 5.2, UX contracts, architecture spine, Story 5.1, current debt/rate/web seams, deferred work, and recent git history.
- Used parallel planning-artifact and implementation analysis to retain established Current semantics while removing the CSP-incompatible dynamic `aria-busy` and client-retention requirement.
- Consulted HTMX 2.0.4 documentation through Context7: `hx-indicator` exposes request CSS classes and does not set `aria-busy`.

### Completion Notes List

- Recreated the Story 5.2 implementation guide with a CSS-only enhanced Updating state, stable final status semantics, and explicit no-custom-JavaScript/CSP guardrails.
- Implemented Current-mode rate context reuse, strict stale-evidence validation, mode-aware Debts rendering, scoped enhanced no-partial errors, and CSS-only Updating presentation.
- Added regressions for the inclusive seven-day Current stale boundary, archived/inactive zero-activity Current identities, enhanced mode/error responses, and the no-inline-handler Updating contract.
- Hardened review follow-ups: stale Current evidence must be strictly prior to the calculation date, and timeout mode decoding now rejects duplicate or invalid query values.
- Validation passed: format, workspace check, strict offline Clippy, full workspace tests, and architecture fitness.

### File List

- `_bmad-output/implementation-artifacts/5-2-recalculate-balances-at-current-rates.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `_bmad-output/implementation-artifacts/deferred-work.md`
- `_bmad-output/planning-artifacts/epics.md`
- `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md`
- `debtor-application/src/debts.rs`
- `debtor-web/src/handlers/debts.rs`
- `debtor-web/src/handlers.rs`
- `debtor-web/src/handlers/response.rs`
- `debtor-web/src/handlers/test_support.rs`
- `debtor-web/src/middleware.rs`
- `debtor-web/src/router.rs`
- `debtor-web/src/templates.rs`
- `debtor-web/templates/debts.html`
- `specs/design.md`
- `static/css/app.css`

### Change Log

- 2026-08-19: Recreated Story 5.2 for CSS-only enhanced Updating behavior; dynamic `aria-busy` and client-side financial retention are out of scope.
- 2026-08-19: Implemented and validated Current-mode Debts with CSS-only enhanced Updating behavior.
- 2026-08-19: Addressed review findings for strictly prior stale Current evidence and timeout mode-decoding drift.
