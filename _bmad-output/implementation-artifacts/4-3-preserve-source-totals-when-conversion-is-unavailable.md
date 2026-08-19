---
story_key: 4-3-preserve-source-totals-when-conversion-is-unavailable
story_id: 4.3
epic: 4
status: done
baseline_commit: a9bd767
created: 2026-08-18
completion_note: Ultimate context engine analysis completed - comprehensive developer guide created
---

# Story 4.3: Preserve Source Totals When Conversion Is Unavailable

Status: done

## Story

As the administrator,
I want the monthly Summary to degrade safely when fresh exchange rates cannot be obtained,
so that exact Source Currency totals and ledger operations remain usable without misleading partial conversion.

## Acceptance Criteria

1. Fixed-past Historical fallback uses only an exact stable-cache key `(source, target, requested R, fetch F)`, where `F = min(R, C)`. If the fresh request fails, an exact prior quote is eligible without an age limit, returns `is_stale = true`, and no quote from another `R` or `F` is substituted.
2. Current/future refreshable fallback retries the current fetch context first. Current fallback may select the latest prior current-class quote for the pair; future fallback must match the same original future `R`. Both are eligible inclusively through seven UTC calendar days after the prior fetch date `F`; day eight is unavailable.
3. Refreshable rollover uses the new UTC calculation/fetch date. Stable past entries may survive rollover. Rollover, pruning, and deterministic LRU eviction must never cross contexts or make an expired quote eligible. Stable and refreshable classes remain independently capped at 4,096 entries with per-key single-flight and at most four provider calls in flight globally.
4. Fallback metadata is preserved and validated: stale fallback is marked stale; future fallback remains provisional; returned base, target, requested date, positive rate, and effective date remain consistent with the requested context. Summary disclosure must expose the actual prior quote evidence, not imply it was fetched on the current date.
5. If any required context lacks fresh or eligible stale evidence, the application returns retryable `ApplicationError::Unavailable(UnavailableReason::ExchangeRates)`. If checked conversion, accumulation, or quantization fails, it returns `ApplicationError::Calculation`. Neither cause produces partial converted rows or zero substitutions.
6. `MonthlySummary` continues to return successful Source Currency totals independently when conversion is unavailable or calculation fails. Source totals, Group navigation, Add Spending, Transactions, and Spending CRUD remain usable; provider availability never gates startup, readiness, or ordinary ledger requests.
7. The converted Summary renders one whole-section sanitized unavailable state on either inward failure cause. Any prior converted total, payer row, or rate evidence is replaced, no raw cause/diagnostic/identifier/amount is exposed, and revisiting Summary is the retry path. No manual Retry control is added.
8. Converted state is explicit and truthful: Updating, Ready, Stale, Provisional, and Unavailable. Stale results have one Group-level stale notice; future stale results are both stale and provisional in visible text and rate evidence. One stable scoped status has `role="status"`, `aria-live="polite"`, and `aria-atomic="true"`; the owning result controls `aria-busy`; amount rows are not live regions.
9. Native and optional HTMX Summary responses retain equivalent source-first semantics, stable focus/navigation, safe failure recovery, and the Editorial Contrast responsive contract at 320 CSS pixels and 400% zoom. Controls remain at least 48 by 48 CSS pixels, no page-level horizontal interaction is required, and no custom JavaScript is introduced.

Requirements: `SPEC-FR72`, `SPEC-FR74`, `SPEC-FR76..SPEC-FR77`; `SPEC-NFR4..SPEC-NFR5`, `SPEC-NFR10`, `SPEC-NFR13`, `SPEC-NFR25..SPEC-NFR34`; UX contracts `UX-SHELL-01`, `UX-TARGET-01`, `UX-FOCUS-01`, `UX-STATUS-01`, `UX-RESPONSIVE-01`, `UX-VISUAL-01`.

## Scope Boundary

- Extend Story 4.2's accepted rate/cache and monthly Summary seams; do not create a second provider, cache, snapshot reader, clock, route, or calculation path.
- Primary implementation is the Frankfurter adapter's stable/refreshable cache admission, stale fallback, eligibility, rollover, and deterministic LRU behavior.
- Correct the web conversion projection so stale and unavailable states are visible and truthful while preserving the accepted source/converted whole-section boundary.
- Preserve `MonthlySummary` one-snapshot behavior and distinct inward `Unavailable` versus `Calculation` causes.
- Do not add Current mode, all-time Balances, Settlement Transfers, participant archival, arbitrary date ranges, analytics, manual rate refresh, migrations, or custom application JavaScript.
- No migration or `.sqlx` change is expected. If SQL/migrations genuinely change, update `specs/design.md` first and perform the full temporary-database and online SQLx preparation workflow.

## Tasks / Subtasks

- [x] Audit and lock the accepted Story 4.2 behavior before editing (AC: 5-9)
  - [x] Re-read `SummaryService`, `MonthlySummary`, `RateQuote`, `LedgerSnapshot`, `build_group_template`, converted fragment handling, templates, CSS, and existing Summary tests.
  - [x] Preserve one complete snapshot for source and converted projections; provider calls begin only after snapshot completion and no provider call holds a database transaction.
  - [x] Preserve source totals on both provider-unavailable and checked-calculation failure, including archived/renamed Payers, shell order, Add Spending, Transactions, CRUD, and `include_summary = false` Manage/Spending-form behavior.

- [x] Implement exact stable fixed-past fallback in `FrankfurterClient` (AC: 1, 3, 4)
  - [x] On fresh failure for `R < C`, consult only the stable cache's exact `(base, quote, R, F)` key. Do not use pair-only, latest-date, current-class, or differently fetched evidence.
  - [x] Keep fixed-past evidence stale-eligible without an age limit, set `is_stale = true`, and preserve requested/effective metadata safely.
  - [x] Keep same-currency conversion synthetic at exact rate `1` with no cache/provider access.
  - [x] Define deterministic cache recency for successful stale fallback access if changing cache reads; do not introduce nondeterministic HashMap iteration or expose cache keys in logs.

- [x] Enforce refreshable current/future eligibility and rollover (AC: 2-4)
  - [x] Retain current fallback selection as latest prior current-class quote for the pair, and future fallback selection as same pair plus exact original future `R`.
  - [x] Enforce the inclusive UTC calendar rule `C <= prior_F + 7 days`; reject `prior_F + 8` and older. Use date arithmetic, not elapsed wall-clock durations.
  - [x] Ensure the eligibility predicate is checked independently of rollover pruning and LRU retention. Eviction or retained expired values must never become eligible.
  - [x] Preserve separate stable and refreshable 4,096-entry caps, deterministic LRU behavior, per-key single-flight keyed by `(base, quote, R, F)`, and global four-request concurrency.
  - [x] Preserve provider bounds: five-second connect timeout, 20-second total/read timeout, and 64 KiB response limit. Do not add a new HTTP client or dependency.

- [x] Preserve application cause and evidence boundaries (AC: 4-7)
  - [x] Keep cache/fallback policy in `debtor-infra`; `SummaryService` validates returned quote identity and checked arithmetic but does not implement fallback lookup.
  - [x] Ensure stale quote evidence contains enough information for accurate requested/fetch/effective disclosure. If the existing `RateQuote` shape cannot represent prior fetch evidence, make the smallest application-owned extension and update all consumers, including `DebtService`, without leaking adapter types.
  - [x] Keep `MonthlySummary` source and converted results independently typed and all-or-nothing for converted output. Never use `source_nets`, Shares, floats, SQL aggregates, or fallback-to-zero.
  - [x] Retain raw provider/storage diagnostics inside adapters; map only fixed safe application reasons outward.

- [x] Correct typed web states and native/HTMX parity (AC: 7-9)
  - [x] Add an explicit `Stale` state or equivalent typed aggregate state to `ConvertedSummaryState`; derive Ready/Stale/Provisional from all quote evidence, not only future evidence. A future stale result must communicate both conditions.
  - [x] Do not mark completed converted values as `Updating` merely because an HTMX fragment exists. Native full-page output must accurately represent its actual final state; enhanced Updating must be an intentional pending state.
  - [x] Render one Group-level stale/provisional/unavailable status and retain detailed rate evidence. Final unavailable state must have `aria-busy="false"`, no converted amounts/evidence, and no manual Retry.
  - [x] Keep currency and failure projection tied to the application calculation context; do not perform an unrelated Group read that can disagree with the conversion snapshot.
  - [x] Preserve stable Summary heading focus, five-link shell (`Groups`, `Summary`, `Transactions`, `Debts`, `Manage`), native URLs, source-before-converted hierarchy, and no page-level horizontal scrolling.

- [x] Add invariant-owning tests (AC: all)
  - [x] Infra tests: exact stable fixed-past fallback; rejection for different `R`/`F`; unlimited fixed-past age; current and future fallback at exactly seven days; rejection at eight days; same future `R` enforcement; stale/provisional flags; rollover; LRU caps/recency; cross-context isolation; single-flight and four-call bound.
  - [x] Application tests: source success plus converted `Unavailable`; source success plus converted `Calculation`; no partial converted result; stale and stale-plus-provisional evidence; exact metadata validation; one snapshot; deterministic output/evidence regardless of provider completion order.
  - [x] Web/router/template tests: source continuity with conversion-only failure; explicit stale and stale-provisional status; final unavailable replacement clears old converted rows; stable status/`aria-busy`; no live amount rows; no Retry; archived identity labels; native/HTMX semantic parity and focus behavior.
  - [x] Coordinate asynchronous tests with barriers, notifications, or controlled fakes, not timing sleeps. Keep existing root real-socket smoke, architecture fitness, security-header, and archived read-only coverage.

- [x] Run required validation (AC: all)
  - [x] `cargo fmt --all -- --check`
  - [x] `cargo check --workspace --all-features --locked`
  - [x] `SQLX_OFFLINE=true cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`
  - [x] `cargo test --workspace --all-features --locked`
  - [x] `cargo run --bin architecture-check --locked`
  - [x] `cargo fmt --manifest-path tools/password-hash/Cargo.toml -- --check`
  - [x] `cargo clippy --manifest-path tools/password-hash/Cargo.toml --all-targets --all-features --locked -- -D warnings`
  - [x] `cargo test --manifest-path tools/password-hash/Cargo.toml --locked`
  - [x] Never use `cargo build --release` for validation.

### Review Findings

- [x] [Review][Patch] Fixed-past stale fallback is rejected by application validation [debtor-application/src/summaries.rs:372-374] — `FrankfurterClient` marks an exact stable fixed-past quote stale while retaining its exact context fetch date, but `SummaryService` rejects every stale quote whose `fetch_date >= context.fetch_date`. Provider failure therefore makes valid unlimited-age fixed-past fallback unavailable, violating AC 1 and AC 4.
- [x] [Review][Patch] Converted fragment fallback performs an unrelated Group read after monthly-summary failure [debtor-web/src/handlers/spending_views.rs:564-572] — the HTMX fragment calls `groups.group(id)` when `monthly_summary` fails, so its currency can come from a different read than the calculation context. This violates the one-context conversion projection constraint in AC 9 and the application snapshot boundary.

## Dev Notes

### Developer Context

Story 4.2 is the accepted baseline at commit `a9bd767`. It already provides exact current-month Source Currency totals, fresh/synthetic Historical conversion, payer-paid aggregation, joint target quantization, deterministic rate evidence, and an independent `MonthlySummary` source/converted result boundary. Story 4.3 must finish the deferred cache fallback and degraded-state behavior rather than duplicate that work.

The critical current defect is in `FrankfurterClient::stale_or_error`: fixed-past requests immediately return unavailable, despite stable cache support. Refreshable fallback currently has no seven-day eligibility predicate and can accept indefinitely old current/future evidence. The web projection currently has no aggregate stale state and can label already-rendered complete values as Updating. These are implementation targets, not permission to redesign the Summary architecture.

The Summary is a payer-paid projection. Convert each Spending's single Payer amount. Shares and `Spending::source_nets()` belong to debt arithmetic and must not be used. All converted output is atomic: if one context, multiplication, addition, quantization, or evidence check fails, return no converted rows or total.

### Current Files To Update And Preserve

| Path | Current state | Required change / preservation |
|---|---|---|
| `debtor-infra/src/exchange_rates/frankfurter.rs` | Owns exact lexical Decimal decoding, reqwest bounds, same-currency synthesis, stable/refreshable caches, single-flight, global semaphore, rollover, and fallback. | Add exact stable fallback, seven-day refreshable eligibility, correct fallback metadata, deterministic recency/rollover tests. Keep adapter diagnostics safe and bounds unchanged. |
| `debtor-application/src/debts.rs` | Owns `RateQuote`, `Clock`, `ExchangeRateProvider`, snapshot ports, and all-time debt orchestration. | Reuse application-owned quote/port seams. Any quote metadata extension must remain transport-neutral and preserve DebtService behavior. |
| `debtor-application/src/summaries.rs` | Owns one-snapshot `MonthlySummary`, source/converted separation, quote validation, payer conversion, exact quantization, and evidence projection. | Preserve cause separation, one snapshot, checked Decimal operations, no partial output, and source continuity. Do not move cache fallback here. |
| `debtor-web/src/templates.rs` | Owns typed `ConvertedSummaryState` and display projections. | Model Stale and truthful final states with compile-time Askama fields; keep amounts/rates display-ready. |
| `debtor-web/src/handlers/spending_views.rs` | Composes Group Summary and converted fragment; currently forces successful results to Updating and separately reads Group for fragment fallback. | Derive state from evidence, preserve source on conversion-only errors, and keep currency/context tied to application result. Handlers must not perform rate/date/arithmetic policy. |
| `debtor-web/templates/group.html` | Renders source-first Summary, converted section, status, HTMX load, and shell. | Keep source-first hierarchy, stable status/result IDs, correct `aria-busy`, native authority, no Retry, and five-link/focus contract. |
| `debtor-web/templates/converted_summary.html` | Renders converted fragment. | Keep fragment semantically equivalent to native output and clear prior converted rows on unavailable result. |
| `debtor-web/src/router.rs` and `debtor-web/src/handlers/test_support.rs` | Contain protected route tests and Summary fakes/fixtures. | Add source-success/conversion-failure, stale, stale-provisional, unavailable replacement, and native/HTMX assertions without weakening auth/security coverage. |
| `src/composition.rs` | Composes one store, clock, Frankfurter client, SummaryService, DebtService, and web state. | Keep one shared provider/cache/snapshot reader/clock. No web exposure of infra types. |
| `static/css/app.css` | Owns Editorial Contrast, financial result rules, focus, responsive layout, and target sizing. | Only minimal stale/unavailable state styling if required; preserve square geometry, contrast, wrapping, no animation/cards/gradients, and 320px behavior. |
| `specs/design.md` | Normative contract already defines fallback, stale eligibility, source continuity, and provider independence. | Do not silently diverge. Update first only if implementation reveals a genuine behavior-contract change, then synchronize companions. |

### Architecture Compliance

- Preserve `root -> debtor-web/debtor-infra -> debtor-application -> debtor-domain` (AD-1/AD-2). Domain remains synchronous/deterministic; application owns use-case/evidence policy; infra owns cache/provider; web owns HTTP/rendering; root owns composition.
- Apply AD-3: checked `Decimal`, canonical persistence, no floats, SQL monetary aggregation, silent rounding, zero substitution, or partial results.
- Apply AD-7: calculation reads use one complete snapshot, release it before provider work, and do not use the fixed 25-item Transactions page.
- Apply AD-9: exact context key `(source, target, R, F)`, deterministic evidence ordering, synthetic same-currency rate `1`, immutable per-calculation quote set, bounded concurrency, and completion-order independence.
- Apply AD-11/AD-18: semantic Askama/native HTML is authoritative; pinned HTMX and official response-targets are optional enhancement only. No custom JS, extension, inline script, manual Retry, or client-side financial calculation.
- Apply AD-15/AD-16: preserve safe `Unavailable`/`Calculation` categories, injected clock/provider/snapshot fakes, and no raw adapter diagnostics in logs or responses.

### Library / Framework Requirements

- Keep pinned versions from the project context: Rust 1.97.1/edition 2024, Axum 0.8.9, Askama 0.16.0, Tokio 1.53.1, SQLx 0.9.0, reqwest 0.13.4 with rustls, `rust_decimal` 1.42.1, HTMX 2.0.10, and response-targets 2.0.4. Add no dependency.
- `Decimal::checked_add` and `Decimal::checked_mul` return `Option`; map `None` to the fixed safe calculation reason. Never use `f32`/`f64`, saturating arithmetic, or fallback-to-zero. Arbitrary-precision serde is the existing lexical JSON boundary.
- Preserve reqwest `ClientBuilder` connect/total/read timeout bounds and chunk-by-chunk response-size enforcement. The request timeout covers connection through response-body completion; do not add a shorter timeout that changes the contract.
- Askama template structs must provide fields matching template variables at compile time. Keep cache/provider policy out of templates and expose only typed display projections.
- Current API guidance consulted through Context7 on 2026-08-18: `/websites/rs_rust_decimal`, `/seanmonstar/reqwest`, and `/askama-rs/askama`.

### Testing Requirements

- Test exact date boundaries using fixed `NaiveDate` values: fixed past has no age limit; refreshable fallback accepts `F + 7` inclusively and rejects `F + 8`.
- Test stable exact-key isolation across source, target, requested date, and fetch date. Test current pair-only selection and future same-original-requested-date selection separately.
- Test fallback flags and evidence: fixed-past stale; current stale; future stale plus provisional; effective date never later than fetch date; malformed metadata is unavailable.
- Test source-success/converted-failure at the application and web boundaries. Assert no converted partials, no zero, no raw reason, and continued source totals/CRUD.
- Test cache rollover and LRU without assuming HashMap iteration order. If sequence rebasing or fallback access changes, assert deterministic recency. Use barriers/notifications rather than sleeps for concurrency.
- Preserve tests for lexical Decimal decoding, 64 KiB body limit, 5s/20s bounds, same-currency no network, single-flight, max four provider calls, root smoke, architecture fitness, and security headers.

### Previous Story Intelligence

- Story 4.2 review fixed one-snapshot monthly composition, captured Group Currency in the snapshot, reachable HTMX Updating/Busy state, quote metadata validation, source continuity, and expanded precision/failure/accessibility tests. Do not regress these fixes.
- Story 4.1 established provider-independent source totals, complete snapshot identity projection, archived labels, stable Summary focus/status, and `include_summary = false` for Manage/Spending forms.
- Stories 3.3-3.5 established complete aggregate reads, current Participant names, archived history, native full-page authority, stable focus/status IDs, and deterministic concurrency testing.

### Git Intelligence

- Recent story commits are `a9bd767 feat: implement 4-2 bmad`, `a5e0f6f feat: implement 4-1 bmad`, `4c0989e feat: implement 3-5 bmad`, `02d53b4 feat: implement 3-4 bmad`, and `0fc4380 feat: implement 3-3 bmad`.
- Worktree was clean at story creation. Build on the accepted 4.2 implementation; do not restore superseded pre-Epic-3 APIs or overwrite unrelated changes.

### UX Guardrails

- `UX-SHELL-01`: preserve five native destinations in order, Summary current, stable route, and persistent Add Spending placement/guidance.
- `UX-STATUS-01`: one Group-level polite atomic status announces Updating, Ready, Stale, Provisional, or Unavailable; converted result owns `aria-busy`; amount rows are not live regions.
- `UX-FOCUS-01`: Summary navigation retains stable heading focus; conversion refresh does not steal focus. Native and enhanced responses use allow-listed stable targets.
- `UX-TARGET-01` and `UX-RESPONSIVE-01`: all controls stay 48px minimum at 320px/400% with no page horizontal dependency and preserved DOM/focus order.
- `UX-VISUAL-01`: dark Editorial Contrast, ruled sections, explicit text for stale/provisional/unavailable, square geometry, no cards/gradients/transitions/motion, and no color-only meaning.
- Required copy includes `Converted values are unavailable. Reopen this section to retry.` and `Provisional: a current rate was used for a future Spending.` Add one concise explicit stale notice; do not add a manual Retry control.

### Project Structure Notes

- Feature modules remain plural. Interfaces use `*Reader`, `*Provider`, `*UseCases`; implementations use `*Service`, `*Store`, `*Client`; rendering uses `*Template`, `*View`, `*Row`.
- No new route, migration, dependency, global Participant surface, or JavaScript is expected. Prefer the smallest local change in the existing cache and Summary projection seams.
- Do not expose cache keys, provider URLs, dates/IDs as diagnostics, monetary values, credentials, or raw adapter errors in logs or user-facing errors.

## References

- [Source: `_bmad-output/planning-artifacts/epics.md#Story 4.3: Preserve Source Totals When Conversion Is Unavailable`]
- [Source: `specs/design.md#Rates And Settlements`]
- [Source: `specs/design.md#Accounting And History`]
- [Source: `specs/design.md#Operational Limits`]
- [Source: `_bmad-output/project-context.md#Critical Don't-Miss Rules`]
- [Source: `_bmad-output/implementation-artifacts/4-2-review-conserved-group-currency-monthly-totals.md#Scope Boundary`]
- [Source: `_bmad-output/implementation-artifacts/4-2-review-conserved-group-currency-monthly-totals.md#Review Findings (Rerun)`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-7 - Snapshot-complete calculation reads`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-9 - Deterministic rate and settlement processing`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-11 - Native HTML authority`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-15 - Safe failures, logging, and readiness`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md#Financial Presentation`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md#Rate and Debt States`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md#Interaction Focus Matrix`]
- [Source: `debtor-infra/src/exchange_rates/frankfurter.rs`]
- [Source: `debtor-application/src/summaries.rs`]
- [Source: `debtor-application/src/debts.rs`]
- [Source: `debtor-web/src/handlers/spending_views.rs`]
- [Source: `debtor-web/src/templates.rs`]
- [Source: `debtor-web/templates/group.html`]
- [Source: `debtor-web/templates/converted_summary.html`]
- [Source: Context7 `/websites/rs_rust_decimal`, `/seanmonstar/reqwest`, `/askama-rs/askama`, consulted 2026-08-18]

## Dev Agent Record

### Agent Model Used

openai/gpt-5.6-luna

### Debug Log References

- Resolved `bmad-create-story` customization: no activation prepend/append or completion terminal steps configured.
- Loaded ordered sprint status, persistent project context, normative design contract, Epic 4, PRD/addendum, architecture spine, UX contracts, Stories 4.1/4.2, current cache/application/web files, and recent Git history.
- Auto-selected first backlog story `4-3-preserve-source-totals-when-conversion-is-unavailable`; Epic 4 was already `in-progress`.
- Used parallel repository analysis passes to identify missing fixed-past fallback, missing seven-day eligibility, stale-state projection gaps, and source-continuity regression risks.
- Consulted current `rust_decimal`, `reqwest`, and Askama documentation through Context7 on 2026-08-18.

### Implementation Plan

- Extend the application-owned `RateQuote` with cache fetch-date evidence, retaining provider effective date separately.
- Implement exact stable fixed-past fallback and bounded refreshable current/future fallback in the existing Frankfurter cache owner.
- Derive typed web states from complete rate evidence and reuse `MonthlySummary` for converted fragment composition.
- Add adapter and web projection tests, then run workspace, architecture, and helper validation.

### Completion Notes List

- Ultimate context engine analysis completed - comprehensive developer guide created.
- Added exact fixed-past cache fallback, inclusive seven-day refreshable stale eligibility, future-context isolation, and stale fetch-date evidence.
- Preserved source totals and safe error-cause boundaries while making converted Summary states truthful for fresh, stale, provisional, and unavailable results.
- Added fallback boundary tests and typed stale/provisional web projection coverage.
- No migration, SQLx metadata, dependency, or JavaScript changes were required.
- Validation passed: workspace formatting, check, strict Clippy, full workspace tests, architecture fitness, and password-helper formatting/Clippy/tests.
- Code review resolved both patch findings: fixed-past stale validation now accepts exact historical context dates, and converted-fragment errors no longer perform an unrelated Group read.

### File List

- `_bmad-output/implementation-artifacts/4-3-preserve-source-totals-when-conversion-is-unavailable.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `debtor-application/src/debts.rs`
- `debtor-application/src/summaries.rs`
- `debtor-infra/src/exchange_rates/frankfurter.rs`
- `debtor-web/src/handlers/spending_views.rs`
- `debtor-web/src/router.rs`
- `debtor-web/src/templates.rs`
- `debtor-web/templates/converted_summary.html`
- `debtor-web/templates/group.html`

### Change Log

- 2026-08-18: Implemented Story 4.3 stale fallback, eligibility, evidence, and Summary state handling; validation passed.
- 2026-08-19: Applied and verified both code-review patches; full post-review validation passed.
