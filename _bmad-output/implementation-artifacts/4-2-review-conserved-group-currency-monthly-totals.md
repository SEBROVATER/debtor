---
story_key: 4-2-review-conserved-group-currency-monthly-totals
story_id: 4.2
epic: 4
status: done
baseline_commit: a5e0f6f23e776c04b7ceaba568f351cccb15d101
created: 2026-08-18
completion_note: Ultimate context engine analysis completed - comprehensive developer guide created
---

# Story 4.2: Review Conserved Group Currency Monthly Totals

Status: done

## Story

As the administrator,
I want current-month Payer totals converted with exact date-appropriate evidence into Group Currency,
so that I can see reproducible totals whose displayed Group amount reconciles exactly to the displayed Payer amounts.

## Acceptance Criteria

1. **Snapshot boundary:** A current-month conversion loads Group Currency and every complete included Spending from one `LedgerSnapshot`/SQLite snapshot, commits/releases the read transaction, and only then requests rates. No provider request may run while a database transaction is open.
2. **Historical context:** For Spending date `R` and UTC calculation date `C`, use `F = min(R, C)` and preserve the full context identity `(Source Currency, Group Currency, R, F)`. Provider effective date is separate evidence. Same-currency conversion returns exact `Decimal::ONE`, makes no provider call, and is still disclosed.
3. **Exact rate decoding:** Frankfurter JSON numeric rates are decoded lexically and directly into representable positive `Decimal` values. Malformed, nonpositive, oversized, excess-scale, or unrepresentable responses map to safe adapter/application reasons without floats or rounding.
4. **Bounded/deterministic provider work:** Fresh requests use the existing rustls client and five-second connect, 20-second total, and 64 KiB response bounds. Contexts deduplicate, identical uncached keys single-flight, and no more than four calls run globally or for one calculation. Completion order cannot change quote evidence, warnings, or output.
5. **Exact payer-paid conversion:** Include only Spendings in the current UTC calendar month. Convert each Spending's single Payer amount using that Spending's Historical context, then accumulate converted values per Payer with checked Decimal multiplication/addition and no per-Spending display rounding. Do not convert source nets or Share amounts.
6. **Conserved final quantization:** Quantize all final positive Payer totals together at Group Currency precision by truncation toward zero. Assign residual minor units by descending fractional remainder, breaking ties by ascending Participant ID. The displayed Group Currency total must be calculated as the exact sum of the displayed quantized Payer totals.
7. **Disclosure and identity:** The converted Group Currency total precedes converted Payer rows and deterministic unique rate evidence. Every amount includes symbol and ISO code. Same-currency, fixed-past, and future contexts are disclosed; future-dated Spendings explicitly say a current rate was used and are visibly provisional. Archived or renamed historical Payers remain included and display their current name plus visible `Archived` text.
8. **Whole converted failure boundary:** If fresh rate resolution, checked conversion, aggregation, or quantization fails, no converted Group/Payer row or zero substitution is rendered. The converted region uses one sanitized unavailable state while Story 4.1 Source Currency totals, Group navigation, Add Spending, and ledger CRUD remain usable. Do not expose provider diagnostics, IDs, amounts, SQL, or raw error causes.
9. **Accessible state parity:** Updating, Ready, Provisional, and Unavailable conversion states use one stable Group-level polite atomic status with `role="status"`, `aria-live="polite"`, `aria-atomic="true"`; the owning converted result toggles `aria-busy`; individual amount rows are not live regions. Summary heading focus and five-link shell behavior remain unchanged for native and optional HTMX-enhanced responses.
10. **Responsive native UI:** At 320 CSS pixels and 400% zoom, converted/source hierarchy, long names, rate evidence, warnings, and controls remain readable with no page-level horizontal interaction. Wide composition preserves DOM/focus order and Editorial Contrast. All controls remain at least 48 by 48 CSS pixels. There is no manual Retry control; native revisit is the retry path.

Requirements: `SPEC-FR67`, `SPEC-FR69..SPEC-FR74`; `SPEC-NFR4..SPEC-NFR5`, `SPEC-NFR10..SPEC-NFR15`, `SPEC-NFR25..SPEC-NFR30`, `SPEC-NFR32..SPEC-NFR34`; UX contracts `UX-SHELL-01`, `UX-TARGET-01`, `UX-FOCUS-01`, `UX-STATUS-01`, `UX-RESPONSIVE-01`, `UX-VISUAL-01`.

## Scope Boundary

- Extend the existing `/groups/{id}` Summary vertical slice with fresh/synthetic Historical Group Currency conversion and exact conserved payer-paid output.
- Reuse the complete snapshot, injected `Clock`, existing `ExchangeRateProvider`, and existing `FrankfurterClient`; do not create a second snapshot reader, clock, provider client, cache, or Summary route.
- Story 4.3 owns stable/refreshable stale fallback, seven-day eligibility, cache rollover, LRU behavior, and the final retryable unavailable semantics for missing eligible quotes. Story 4.2 must preserve the application cause boundary and the whole converted-section projection boundary so 4.3 can extend it without duplicating calculations.
- Do not add Current mode, all-time balances, settlement transfers, participant archival eligibility, arbitrary date ranges, statistics, charts, exports, manual refresh, or custom application JavaScript.
- Prefer no migration. If SQL or migrations genuinely change, update `specs/design.md` first, then refresh `.sqlx`, migrate a temporary SQLite database, and run online SQLx preparation.

## Tasks / Subtasks

- [x] Audit the accepted Story 4.1 Summary slice before editing (AC: all)
  - [x] Re-read `SummaryService`, `SummaryUseCases`, `GroupTemplate`, `build_group_template`, `group.html`, and Summary web tests.
  - [x] Preserve source totals as an independently available provider-free projection, including empty/error states, archived identities, month context, Add Spending, shell order, heading focus, and Manage/Spending-form `include_summary = false` behavior.
  - [x] Preserve the canonical `/groups/{id}` route. Do not use the bounded Transactions page as calculation input.
- [x] Define the application/domain conversion contract (AC: 1, 2, 5, 6, 8)
  - [x] Add the smallest typed converted-summary result alongside or as a deliberate extension of `SummaryUseCases`; keep provider and framework types out of inner crates.
  - [x] Capture one immutable calculation context containing calculation instant/date, Group Currency, current-month bounds, and deterministically ordered unique `(base, quote, requested, fetch)` contexts.
  - [x] Filter the complete snapshot before requesting rates. Historical requested date is each included Spending date; calculate `fetch = min(requested, today)` and retain provider effective date separately.
  - [x] Convert Payer allocations, not `Spending::source_nets`; aggregate by Participant ID with checked operations.
  - [x] Implement or reuse a pure positive-total joint quantizer. Do not reuse `add_converted_spending` or `quantize_balances` unchanged: the former computes debt nets and the latter requires an exact zero-sum input.
  - [x] Return no partial converted projection on any checked failure and map failures to safe `ApplicationError` categories.
- [x] Integrate the existing rate port and adapter (AC: 2, 3, 4, 7)
  - [x] Reuse `ExchangeRateProvider::rate` and `RateQuote` seams from `debtor-application/src/debts.rs`; add only the narrow context/evidence fields needed for the Summary contract.
  - [x] Reuse Frankfurter lexical Decimal decoding, same-currency synthesis, response bounds, global semaphore, single-flight, and cache owner. Do not add a second fallback/cache path or move Story 4.3 fallback ownership into the Summary handler.
  - [x] Validate returned quote identity against the requested context; sort disclosure deterministically independent of async completion order.
  - [x] Ensure all included historical contexts are deduplicated and fetched at most four concurrently. Tests must use barriers/notifications, not timing sleeps.
- [x] Compose the vertical slice (AC: 1, 5, 8)
  - [x] Inject the conversion-capable Summary service in `src/composition.rs` using the existing store snapshot reader, shared `UtcClock`, and one composed `FrankfurterClient`.
  - [x] Keep `AppState` application-facing; web must not receive `ExchangeRateProvider` or concrete infra types.
  - [x] Ensure Manage and Spending-form renders do not invoke conversion or provider I/O.
- [x] Build typed web projection and Summary markup (AC: 7, 8, 9, 10)
  - [x] Add explicit converted state/types, Group Currency total, payer rows, rate disclosure, provisional reason, and sanitized unavailable state to `templates.rs`/`GroupTemplate`.
  - [x] Keep arithmetic, rate orchestration, date policy, sorting, and error classification out of handlers and Askama templates. `build_group_template` may compose application results only.
  - [x] Extend `group.html` with source blocks first, then converted Group Currency total and payer rows, then rate disclosure. Keep one stable conversion status node and stable result IDs; do not make amount rows live regions.
  - [x] Add only minimal ruled Editorial Contrast CSS in `static/css/app.css`; preserve tabular numerals, wrapping, focus indicators, square geometry, no gradients/animation/card-heavy treatment, and no page-level horizontal dependency.
- [x] Add invariant-owning tests (AC: all)
  - [x] Application/domain tests cover current-month boundaries, same-currency no-call, fixed-past/future contexts, future provisional labeling, duplicate context requests, exact conversion, no intermediate rounding, all target precisions, positive joint quantization, remainder/Participant-ID ties, displayed-total conservation, overflow, invalid quote, and no partial result.
  - [x] Application tests use an injected clock, fake snapshot, and fake provider without Axum, SQLite, network, or wall clock. Assert provider completion-order independence and distinct unavailable/calculation causes.
  - [x] Infra tests retain complete snapshot/transaction release coverage and cover lexical rate decoding, invalid/oversized responses, same-currency no network, bounds, four-call concurrency, single-flight, and context/evidence correctness. Do not duplicate 4.3 fallback acceptance prematurely.
  - [x] Web/router/template tests cover source-before-converted hierarchy, symbol+ISO amounts, current names/Archived labels, rate evidence, provisional copy, whole converted unavailable state with source continuity, stable status/`aria-busy`, no live amount rows, no Retry control, canonical route, focus, shell, and native/HTMX parity.
  - [x] Preserve root real-socket smoke and architecture fitness coverage.
- [x] Run validation (AC: all)
  - [x] `cargo fmt --all -- --check`
  - [x] `cargo check --workspace --all-features --locked`
  - [x] `SQLX_OFFLINE=true cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`
  - [x] `cargo test --workspace --all-features --locked`
  - [x] `cargo run --bin architecture-check --locked`
  - [x] Password helper fmt, Clippy, and locked tests.
  - [x] No SQL or migration changes; SQLx preparation was not required.
  - [x] Never use `cargo build --release` for validation.

### Review Findings

- [x] [Review][Patch] Same-currency conversion still invokes the exchange-rate provider [debtor-application/src/summaries.rs:296-299]
- [x] [Review][Patch] Source and converted summaries can be built from different ledger snapshots [debtor-web/src/handlers/spending_views.rs:143-159]
- [x] [Review][Patch] Conversion has no Updating state or busy transition [debtor-web/templates/group.html:198-200]
- [x] [Review][Patch] Future provisional and effective-date evidence is trusted instead of validated [debtor-application/src/summaries.rs:300-303,377-392]
- [x] [Review][Patch] Required conversion precision, failure-boundary, ordering, and web-state coverage is missing [debtor-application/src/summaries.rs:749-844, debtor-web/src/router.rs:529-604]
- [x] [Review][Patch] Native/HTMX unavailable and accessibility parity is not test-backed [debtor-web/templates/group.html:198-230]

### Review Findings (Rerun)

- [x] [Review][Patch] The default `monthly_summary` implementation can still load two snapshots [debtor-application/src/summaries.rs:124-132]
- [x] [Review][Patch] Updating and HTMX conversion states are modeled but unreachable [debtor-web/templates/group.html:198-200, debtor-web/src/handlers/spending_views.rs:142-159]
- [x] [Review][Patch] Group display currency is read outside the conversion snapshot [debtor-web/src/handlers/spending_views.rs:142-159]
- [x] [Review][Patch] Snapshot ordering, provider completion ordering, metadata rejection, residual allocation, and conversion-only web failure tests remain incomplete [debtor-application/src/summaries.rs:823-960, debtor-web/src/router.rs:603-633]

## Dev Notes

### Developer Context

Story 4.1 is the accepted baseline at HEAD (`a5e0f6f`). It provides `SummaryService::source_summary`, current-month filtering, exact Source Currency payer totals, complete `LedgerSnapshot` hydration, typed source rendering, safe source failure, and the canonical `/groups/{id}` Summary shell. Story 4.2 is additive: source totals must remain available even if converted calculation fails.

The converted result is a payer-paid monthly projection. A Spending has exactly one Payer, and the displayed monthly converted rows represent what each Payer paid. `Spending::source_nets()` is debt arithmetic and must not be used. Shares are not monthly converted payer rows.

The calculation input must be the complete `LedgerSnapshot` from one database snapshot. `debtor-infra/src/db/repos/snapshots.rs` currently opens a transaction, loads Group, all Group-owned Participant projections, all complete Spending parents, payer rows, and share rows, validates them, commits, then returns. Rate requests must begin only after this future completes.

The existing debt service is useful for context orchestration patterns and provider bounds, but it is not the monthly implementation: `DebtService` converts all-time debt nets and calls the zero-sum quantizer. Reuse interfaces and exact-rate evidence where appropriate, not its financial aggregation.

### Current Files To Update And Preserve

| Path | Current state | Required change/preservation |
|---|---|---|
| `debtor-application/src/summaries.rs` | `SummaryService` loads a complete snapshot and aggregates current-month Payer amounts by Source Currency/Participant ID. `SummaryUseCases` exposes only `source_summary`. | Extend with a typed converted projection or deliberate second application operation. Preserve source calculation, month bounds, checked errors, deterministic ordering, and archived identity lookup. |
| `debtor-application/src/debts.rs` | Owns `Clock`, `LedgerSnapshot`, `LedgerSnapshotReader`, `ExchangeRateProvider`, `RateQuote`, `RateMode`, and all-time `DebtService`. | Reuse application-owned ports and quote shape. Add only neutral context/evidence needed by both consumers; do not couple monthly paid totals to debt balances or Current mode. |
| `debtor-application/src/errors.rs` | Safe `ApplicationError` taxonomy with `Unavailable`, storage, and calculation reasons. | Keep raw provider/SQLx diagnostics out of inward ports and HTTP. Preserve distinction between provider unavailability and checked calculation failure until the converted rendering boundary. |
| `debtor-domain/src/debts/balance.rs` | `add_converted_spending` converts source nets; `quantize_balances` enforces zero-sum debt balances. | Leave existing debt behavior intact. Add a separate pure positive-total conversion/quantization function only if the invariant belongs in domain. |
| `debtor-infra/src/db/repos/snapshots.rs` | Complete one-transaction snapshot read and canonical Decimal validation. | Reuse unchanged if possible. Never add SQL monetary aggregation or a second read for monthly conversion. |
| `debtor-infra/src/exchange_rates/frankfurter.rs` | Pinned reqwest/rustls adapter with lexical arbitrary-precision Decimal, 5s/20s/64KiB bounds, four permits, single-flight, cache classes, same-currency synthesis, and quote metadata. | Add only missing 4.2 validation/evidence tests or narrow adapter fixes. Do not create duplicate provider/cache logic or claim 4.3 stale fallback work in this story. |
| `debtor-web/src/state.rs` | `AppState` exposes application use-case traits including `summaries`, `debts`, and shared clock. | Inject the application Summary capability, never concrete provider/store types. |
| `debtor-web/src/handlers/spending_views.rs` | `build_group_template` composes Group, source Summary, members, bounded history, forms, and shell; `include_summary` prevents Manage/form calculations. | Compose converted result only when Summary is included. Do not put Decimal/rate/date policy here. Keep source result on conversion failure and preserve `include_summary = false`. |
| `debtor-web/src/templates.rs` | `GroupTemplate` contains `source_summary: SourceSummaryView`; `RateRow` already models debt rate disclosure. | Add explicit typed converted Summary state/rows/evidence. Keep template fields display-ready and compile-time aligned. Avoid loosely typed maps or financial logic in Askama. |
| `debtor-web/templates/group.html` | Existing five-link shell, Summary heading/focus, source hierarchy, status node, and Transactions link. | Preserve shell/order/focus and source-first hierarchy. Add a sibling converted region with one status, total-before-payers, disclosure, provisional text, and no Retry. Native HTML remains authoritative. |
| `static/css/app.css` | Existing Editorial Contrast, financial result rules, tabular amounts, focus, wrapping, and responsive layout. | Extend minimally for converted rows/disclosure/warnings. Preserve 48px targets, 2px focus, contrast, no motion, and no page horizontal scrolling. |
| `src/composition.rs` | Composes one store, one clock, one Frankfurter client, DebtService, SummaryService, and AppState. | Share the existing store/clock/provider instances; do not instantiate parallel rate clients or snapshot abstractions. |
| `debtor-web/src/handlers/test_support.rs`, `debtor-web/src/router.rs` | Existing shared fakes and protected Group route tests cover Story 4.1 source rendering/security. | Extend fakes/fixtures for converted ready, provisional, unavailable, and deterministic disclosure cases while retaining all security/header/auth tests. |
| `migrations/*`, `.sqlx/*` | Existing schema stores source amounts as canonical TEXT and supports complete aggregate reads. | No change expected. Any change requires design-first synchronization and SQLx prepare verification. |

### Architecture Compliance

- Preserve `root -> debtor-web/debtor-infra -> debtor-application -> debtor-domain` (AD-1/AD-2). Domain owns synchronous deterministic Decimal conversion/quantization; application owns the use case, calculation context, date policy, provider port, and safe taxonomy; infra owns SQLite/HTTP adapters; web owns rendering and HTTP mapping; root owns composition.
- Apply AD-3: exact `rust_decimal::Decimal`, checked `checked_mul`/`checked_add`, canonical SQLite TEXT, currency precision validation, no float, lossy conversion, SQL monetary aggregation, silent rounding, zero substitution, or partial output.
- Apply AD-4/AD-5: historical Participant identities remain included after archive/rename; display current names and visible `Archived`; never filter converted history to active Participants.
- Apply AD-7: complete snapshot first, transaction released before provider I/O. The 25-item Transactions reader is not a calculation input.
- Apply AD-9: one immutable `CalculationContext`, context key `(base, quote, R, F)`, deterministic context/evidence ordering, same-currency synthetic one, lexical provider Decimal, bounded concurrency, and completion-order-independent output.
- Apply AD-11/AD-18: Askama semantic HTML and native links are authoritative; HTMX is optional enhancement only. No custom JavaScript, custom extensions, inline scripts, manual Retry, or client-side aggregation.
- Apply AD-15/AD-16: safe bounded errors, secret-safe diagnostics, injected clock/provider/snapshot fakes, and no raw adapter details at application/web boundaries.

### Library / Framework Requirements

- Preserve pinned project versions: Rust 1.97.1/edition 2024, Axum 0.8.9, Askama 0.16.0, Tokio 1.53.1, SQLx 0.9.0, reqwest 0.13.4 with rustls, `rust_decimal` 1.42.1, HTMX 2.0.10, and response-targets 2.0.4. Add no dependency unless a concrete existing contract requires it.
- `Decimal::checked_add` and `Decimal::checked_mul` return `Option`; map `None` to the safe calculation boundary. Never use `f32`/`f64`, `to_f64`, saturating arithmetic, or fallback-to-zero. Context7 current documentation: `/websites/rs_rust_decimal`, consulted 2026-08-18.
- Existing `#[serde(with = "rust_decimal::serde::arbitrary_precision")]` is the correct adapter boundary for lexical arbitrary-precision JSON numbers. Reject invalid values before constructing a successful `RateQuote`.
- Askama `#[derive(Template)]` requires Rust context fields to match template variables at compile time; keep display strings and booleans in typed `*View`/`*Row` projections. Context7 current documentation: `/askama-rs/askama`, consulted 2026-08-18.
- Existing Axum route/extractor and Askama patterns are established by Story 4.1. Do not alter route shape or introduce provider access into handlers.

### Testing Requirements

- Domain/application examples must prove month inclusion/exclusion, date contexts, same-currency synthesis, exact accumulation before rounding, target precisions (JPY/KRW 0, OMR 3, others 2), multiple residual units, equal-remainder ID ties, exact displayed conservation, overflow, nonrepresentable results, and whole-result failure.
- Use fake `LedgerSnapshotReader`, fake `ExchangeRateProvider`, and fixed `Clock`. Assert the provider never runs before snapshot completion, duplicate contexts make one request, and reversed completion produces identical rows/evidence/warnings. Coordinate with barriers/notifications, never sleeps.
- Infra tests must retain complete snapshot hydration, canonical-decimal corruption rejection, group scoping, and transaction release proof. Test lexical rates, malformed/nonpositive/oversized/excess-scale/unrepresentable responses, same-currency no network, provider bounds, single-flight, and returned context/effective-date metadata.
- Web tests must assert source sections precede converted sections; Group total precedes payer rows; every amount includes symbol and ISO; provisional copy names the current rate/future Spending; rate evidence is deterministic; converted failure leaves source totals and CRUD links; one stable status has `role=status`, `aria-live=polite`, `aria-atomic=true`, owning `aria-busy`; amount rows are not live; no Retry exists; and native/HTMX responses preserve semantics/focus.
- Preserve root real-socket smoke, architecture fitness, security headers, authenticated route, and archived read-only coverage. Browser geometry/contrast can remain manual if no browser harness exists, but tests must preserve the CSS/markup contracts and 48px/focus rules.

### Previous Story Intelligence

- Story 4.1 review fixed the snapshot/Participant projection boundary, avoided calculating summaries for Manage/Spending-form paths, added stable Summary focus, tied fallback month to the calculation attempt, and added multi-currency/precision/archived/accessibility tests. Do not regress these fixes.
- Stories 3.3-3.5 established complete aggregate reads, current Participant identity projection, archived labels, native full-page authority, stable focus/status IDs, direct aggregate loading, restrictive historical integrity, and deterministic concurrency tests. Extend these seams instead of rebuilding them.
- Story 4.1 explicitly forbids `source_nets`, paginated history, SQL monetary aggregation, provider coupling in source totals, and duplicate snapshot abstractions. These remain hard constraints.

### Git Intelligence

- Recent implementation commits are story-oriented: `a5e0f6f feat: implement 4-1 bmad`, `4c0989e feat: implement 3-5 bmad`, `02d53b4 feat: implement 3-4 bmad`, `0fc4380 feat: implement 3-3 bmad`, `120c4ca feat: implement 3-2 bmad`.
- Build on the accepted HEAD implementation and current worktree; do not overwrite unrelated changes or revive superseded pre-Epic-3 APIs.

### UX Guardrails

- `UX-SHELL-01`: keep five native Group destinations in order (`Groups`, `Summary`, `Transactions`, `Debts`, `Manage`), mark Summary current, preserve stable URL and persistent Add Spending placement/guidance.
- `UX-FOCUS-01`: native/enhanced forward Summary navigation keeps the stable Summary heading focus contract. Conversion state changes do not steal focus.
- `UX-STATUS-01`: one stable Group-level Conversion notice announces one state transition and owns `aria-busy`; no amount row is live-announced. Use visible text for provisional/unavailable, never color alone.
- `UX-TARGET-01`: every link/control remains at least 48 by 48 CSS pixels at 320px/400% with visible 2px focus and required contrast.
- `UX-RESPONSIVE-01`: one-column narrow layout, governed wide reading measure, preserved DOM/focus order, no page-level horizontal dependency.
- `UX-VISUAL-01`: dark Editorial Contrast, warm paper, serif major totals, ruled sections, square geometry, explicit warning/provisional text, no cards/gradients/transitions/motion.
- Use product copy from the experience contract: `Updating converted values.`, `Provisional: a current rate was used for a future Spending.`, and `Converted values are unavailable. Reopen this section to retry.` Do not add a manual Retry control.

### Latest Technical Information

- The pinned versions in `_bmad-output/project-context.md` and `Cargo.lock` remain authoritative; this story should not upgrade Axum, Askama, reqwest, rust_decimal, or HTMX.
- Current rust_decimal documentation confirms checked arithmetic is the required overflow boundary and arbitrary-precision serde support must be enabled/used for exact JSON number decoding. See `/websites/rs_rust_decimal`.
- Current Askama documentation confirms compile-time template context checking and `#[template(path = "...")]` relative to the crate template directory. See `/askama-rs/askama`.
- Existing Frankfurter adapter behavior already supplies rustls, 5s connect/20s total timeout constants, 64 KiB body bound, four-request semaphore, single-flight, cache classes, same-currency synthesis, and requested/effective date evidence. Treat these as reusable seams; do not duplicate them. Story 4.3 owns correcting/accepting stale fallback behavior.

## Project Structure Notes

- Feature modules use plural nouns. Application ports use `*Reader`, `*Provider`, or `*UseCases`; implementations use `*Service`, `*Store`, or `*Client`; rendering uses `*Template`, `*View`, and `*Row`.
- Expected files are existing Summary/application/domain/rate/web/composition files listed above. No new route, migration, `.sqlx` metadata, dependency, global participant surface, or JavaScript is expected.
- If a new pure quantizer is added, keep it in the domain financial module with checked errors and focused tests. If conversion orchestration remains in `summaries.rs`, keep it local and avoid speculative shared abstractions unless Epic 5 will genuinely consume the same contract.

## References

- [Source: `_bmad-output/planning-artifacts/epics.md#Story 4.2: Review Conserved Group Currency Monthly Totals`]
- [Source: `_bmad-output/planning-artifacts/epics.md#Story 4.3: Preserve Source Totals When Conversion Is Unavailable`]
- [Source: `specs/design.md#Accounting And History`]
- [Source: `specs/design.md#Rates And Settlements`]
- [Source: `specs/design.md#Architecture`]
- [Source: `specs/design.md#Operational Limits`]
- [Source: `_bmad-output/project-context.md#Technology Stack & Versions`]
- [Source: `_bmad-output/project-context.md#Critical Implementation Rules`]
- [Source: `_bmad-output/implementation-artifacts/4-1-review-exact-source-currency-monthly-totals.md#Previous Story Intelligence`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-3 - Exact monetary truth`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-7 - Snapshot-complete calculation reads`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-9 - Deterministic rate and settlement processing`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-11 - Native HTML authority`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-15 - Safe failures, logging, and readiness`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md#Financial Presentation`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md#Rate and Debt States`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md#Interaction Focus Matrix`]
- [Source: `debtor-application/src/summaries.rs`]
- [Source: `debtor-application/src/debts.rs`]
- [Source: `debtor-domain/src/debts/balance.rs`]
- [Source: `debtor-infra/src/db/repos/snapshots.rs`]
- [Source: `debtor-infra/src/exchange_rates/frankfurter.rs`]
- [Source: `debtor-web/src/handlers/spending_views.rs`]
- [Source: `debtor-web/src/templates.rs`]
- [Source: `debtor-web/templates/group.html`]
- [Source: `src/composition.rs`]
- [Source: `static/css/app.css`]
- [Source: Context7 `/websites/rs_rust_decimal` and `/askama-rs/askama`, consulted 2026-08-18]

## Dev Agent Record

### Agent Model Used

openai/gpt-5.6-luna

### Debug Log References

- Resolved `bmad-create-story` customization: no activation prepend/append or completion terminal steps configured.
- Loaded `_bmad/bmm/config.yaml`, `_bmad-output/project-context.md`, normative `specs/design.md`, ordered sprint status, Epic 4, architecture spine/reviews, UX contracts, Story 4.1, current Summary/rate/snapshot/domain/web files, and recent Git history.
- Auto-selected first backlog story `4-2-review-conserved-group-currency-monthly-totals`; Epic 4 was already `in-progress`.
- Used parallel artifact and code exploration passes to identify reusable seams, regressions, and Story 4.3 ownership boundaries.
- Consulted current rust_decimal and Askama documentation through Context7 on 2026-08-18.
- Captured baseline commit `a5e0f6f23e776c04b7ceaba568f351cccb15d101` before implementation and moved sprint status to `in-progress`.
- Red/green-tested the new positive-total quantizer, then added application and web conversion tests before the corresponding implementation paths.

### Implementation Plan

- Added `quantize_positive_totals` to the domain with checked arithmetic, target-precision aggregate truncation, largest-remainder residual assignment, and Participant-ID ties.
- Extended `SummaryUseCases`/`SummaryService` with injected rate-backed current-month conversion. The service filters the complete snapshot before rate requests, deduplicates `(base, quote, requested, fetch)` contexts, validates quote identity/positivity, converts Payer amounts exactly, and returns deterministic evidence.
- Composed the existing shared Frankfurter client into Summary without exposing infra types to web or creating a duplicate cache/provider.
- Added typed converted web rows/state, source-first Group Summary markup, rate evidence/provisional copy, unavailable conversion state, responsive Editorial Contrast styles, and native route coverage.
- Preserved Story 4.1 source behavior and excluded Manage/Spending-form renders from conversion calls.

### Completion Notes List

- Ultimate context engine analysis completed - comprehensive developer guide created.
- Implemented exact current-month Group Currency payer-paid conversion with fresh/synthetic Historical contexts and deterministic evidence.
- Added positive joint quantization that conserves the target-precision displayed aggregate without reusing debt-net/zero-sum logic.
- Preserved source totals and ordinary ledger behavior when converted calculation is unavailable; Story 4.3 cache fallback ownership remains isolated.
- Added application/domain tests for exact conversion, context deduplication, future provisional evidence, nonpositive provider rejection, fractional aggregate truncation, conservation, and Participant-ID ties.
- Added web coverage for converted hierarchy, archived identity rendering, rate evidence, stable status semantics, and no manual Retry.
- Validation passed: workspace fmt check, workspace check, offline Clippy with warnings denied, full workspace tests, architecture fitness, and password-helper fmt/Clippy/tests.
- No SQL, migration, dependency, or `.sqlx` changes were required.
- Applied all six code-review patches: synthetic same-currency conversion, one-snapshot monthly composition, explicit conversion state enum with busy rendering, quote metadata validation, expanded precision/failure tests, and unavailable/accessibility assertions.
- Code review outcome: all findings resolved; story is ready to close.
- Applied the rerun fixes: mandatory one-snapshot `monthly_summary` contract, snapshot-captured display currency, protected converted HTMX fragment refresh, reachable Updating/Busy state, and additional snapshot/conversion/web assertions.

### File List

- `_bmad-output/implementation-artifacts/4-2-review-conserved-group-currency-monthly-totals.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `debtor-application/src/summaries.rs`
- `debtor-domain/src/debts.rs`
- `debtor-domain/src/debts/balance.rs`
- `debtor-web/src/handlers/spending_views.rs`
- `debtor-web/src/router.rs`
- `debtor-web/src/templates.rs`
- `debtor-web/templates/group.html`
- `src/composition.rs`
- `static/css/app.css`
- `debtor-web/templates/converted_summary.html`
- `debtor-web/src/handlers.rs`
- `debtor-web/src/handlers/groups.rs`
- `debtor-web/src/handlers/test_support.rs`

### Change Log

- 2026-08-18: Implemented Story 4.2 exact current-month Group Currency conversion, conserved payer totals, rate evidence, provisional rendering, safe unavailable boundary, and invariant-owning tests; status moved to review.
- 2026-08-18: Addressed code review findings - 6 items resolved; status moved to done.
- 2026-08-18: Addressed rerun code review findings - 4 items resolved; status remains done.
