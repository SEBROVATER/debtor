---
story_key: 5-1-calculate-exact-historical-balances
story_id: 5.1
epic: 5
status: done
baseline_commit: 4495a1ddd8571675ca6bcca2893004fe19995668
created: 2026-08-19
completion_note: Ultimate context engine analysis completed - comprehensive developer guide created
---

# Story 5.1: Calculate Exact Historical Balances

Status: done

## Story

As the administrator,
I want all-time Participant Balances calculated at each Spending's historical context,
so that I can see an exact zero-sum picture of who is owed and who owes.

## Acceptance Criteria

1. Given a Group has all-time Spending history, when Historical Debts calculation begins, one SQLite snapshot materializes Group Currency, every Group-owned Participant, every complete Spending, and all Payer/Shares before committing the read transaction. No network request holds the database transaction.
2. Given the immutable ledger snapshot is materialized at UTC calculation date/time `C`, when quote contexts are assembled, Historical is selected by default, each Spending uses original date `R` with `F = min(R, C)`, future dates use provisional current evidence, same-currency contexts synthesize exact `1`, and unique contexts are deduplicated into one immutable quote bundle.
3. Given complete exact quote evidence is available, each Payer receives its converted paid Total and each Participant is charged its converted Share. Every Group-owned Participant, including inactive or archived identities and identities with no activity, is present; zero-activity identities remain exact zero. All operations use checked `Decimal`, and provider completion order cannot change pre-quantized positions.
4. Final Balances are quantized together at Group Currency precision using largest signed remainder allocation with Participant-ID tie-breaking and exact zero-sum preservation. No individual rounding step may create or destroy value.
5. Repeating the same immutable snapshot/date/quote bundle with different row or provider completion order produces identical ordered Balances, evidence, and warnings. Domain tests cover exact zero sum, checked boundaries, currency precision, and deterministic ties.
6. Corrupt stored aggregates, missing or ineligible quote evidence, conversion/aggregation failure, and quantization failure produce no partial financial result. Missing quote evidence maps to retryable `503`; checked calculation failure maps to one sanitized calculation failure. Source monthly Summary and ledger CRUD remain available.
7. A successful Historical Debts view discloses Historical mode, UTC calculation time, target Group Currency, deterministically ordered unique rates, and explicit stale/provisional/synthetic warnings. It shows one Balance per Participant before the later Settlement section and disclosure; each amount includes symbol plus ISO code and explicit direction/sign text.
8. An empty ledger retains the mode control and calculation context, shows one exact zero Group Currency Balance for every Group-owned Participant, states that no Spendings exist, and shows no Settlement Transfer.
9. Debts uses the shared accessible native full-page path without custom JavaScript. Historical is the checked default; native forward navigation focuses the stable result heading and announces Updating in one scoped polite atomic status. Enhanced replacement retains focus on the selected mode control. The result owns `aria-busy`; individual amounts are not live regions.
10. On unavailable, timeout, or calculation failure, prior financial results are replaced by one sanitized no-partial state with attempted context and no raw cause, IDs, provider details, or partial Balances/Transfers. Revisiting Debts retries automatically; no manual Retry control is added.
11. At 320 CSS pixels and 400% zoom, mode controls, long names, rates, warnings, amounts, and disclosure wrap without page-level horizontal scrolling. Controls remain at least 48 by 48 CSS pixels, reading order remains Balance then Settlement then disclosure, Editorial Contrast remains readable and motion-free, and native/enhanced paths are equivalent.

Requirements: `SPEC-FR41`, `SPEC-FR74`, `SPEC-FR78..SPEC-FR83`, `SPEC-FR102`; `SPEC-NFR2`, `SPEC-NFR4..SPEC-NFR5`, `SPEC-NFR10`, `SPEC-NFR12..SPEC-NFR16`, `SPEC-NFR25..SPEC-NFR30`, `SPEC-NFR32..SPEC-NFR34`; UX contracts `UX-SHELL-01`, `UX-TARGET-01`, `UX-FOCUS-01`, `UX-STATUS-01`, `UX-RESPONSIVE-01`, `UX-VISUAL-01`.

## Scope Boundary

- Complete the Historical Balance vertical slice using the accepted Epic 4 snapshot, quote, cache, and fallback seams.
- Do not create a second provider, cache, snapshot reader, clock, debt route, or financial calculation path.
- Do not make Story 5.1 depend on new Settlement behavior; Story 5.3 owns advisory transfer semantics. Existing settlement primitives may remain reusable, but no partial transfer output may accompany a failed balance calculation.
- Do not implement Current mode beyond preserving existing route/API compatibility; Story 5.2 owns Current orchestration and its UX.
- Do not implement Participant archival/restore, repayment state, paid state, settlement checkpoints, date-range filters, analytics, manual refresh, persisted results, migrations, or custom JavaScript.
- No SQL, migration, or `.sqlx` change is expected. If one becomes necessary, update `specs/design.md` first and refresh checked SQLx metadata with the required temporary-database workflow.

## Tasks / Subtasks

- [x] Audit accepted Epic 4 debt/rate seams before editing (AC: 1-6)
  - [x] Read the current `DebtService`, domain balance rules, snapshot repository, Frankfurter adapter, Debts handler/template, CSS, composition, and existing tests.
  - [x] Preserve one complete snapshot and release it before provider I/O.
  - [x] Reuse Story 4.3 `RateQuote`, cache, stale fallback, lexical Decimal, single-flight, four-request limit, and safe failure boundaries.
- [x] Complete application Historical orchestration (AC: 1-6)
  - [x] Capture one injected UTC instant/date and build Historical contexts `(base, target, requested R, fetch F)` with `F = min(R, C)`.
  - [x] Deduplicate contexts in deterministic order, fetch at most four concurrently, and materialize an immutable quote bundle independent of completion order.
  - [x] Validate quote base/target/requested date, positive exact rate, fetch/effective metadata, stale eligibility, and truthful provisional/synthetic flags before arithmetic.
  - [x] Seed a deterministic balance map with every snapshot Participant at `Decimal::ZERO`, then add each Spending's payer-minus-share source nets multiplied by its quote with checked arithmetic and no intermediate rounding.
  - [x] Map missing/ineligible rates to `ApplicationError::Unavailable(UnavailableReason::ExchangeRates)` and checked/corrupt calculation failures to fixed sanitized `ApplicationError::Calculation` reasons. Never substitute zero or return partial output.
- [x] Preserve or harden domain balance invariants (AC: 3-5)
  - [x] Keep domain code synchronous, I/O-free, and framework-free; use `Decimal`, checked operations, deterministic `BTreeMap`/sorting, and Participant ID as final tie-breaker.
  - [x] Quantize signed balances jointly at target minor-unit precision: truncate toward zero, assign residual units by the accepted signed largest-remainder ordering, and verify exact zero sum.
  - [x] Retain and extend tests for USD/EUR-like two-decimal currencies, JPY/KRW zero minor units, OMR three minor units, positive/negative residuals, equal ties, overflow, non-integral residuals, and non-zero input sums.
- [x] Preserve complete snapshot integrity (AC: 1, 3, 6)
  - [x] Keep `debtor-infra/src/db/repos/snapshots.rs` transactionally loading Group, all owned Participants including archived/inactive identities, and complete Spending aggregates with payer/share allocations.
  - [x] Revalidate canonical persisted Decimal values and map corruption safely; do not use the paginated Transactions reader or SQL monetary aggregation.
- [x] Build the Historical Debts projection (AC: 7-11)
  - [x] Project Participant names, colors, and archive state from the calculation-owned snapshot/result; do not issue separate Group/Participant reads after calculation to fill financial output.
  - [x] Add typed Balance rows with display-ready sign/direction, symbol, ISO code, exact amount, and visible archived identity labeling. Never fabricate labels containing internal IDs.
  - [x] Preserve the five-link shell order, active Debts destination, native Historical mode form/URL, Add Spending behavior for active Groups, and read-only archived Group behavior.
  - [x] Render Balance results before the reserved Settlement section and rates/disclosure. Empty all-time history still renders exact zero rows for all owned Participants.
  - [x] Implement stable status/result IDs, `role="status"`, `aria-live="polite"`, `aria-atomic="true"`, correct `aria-busy`, forward heading focus, retained enhanced radio focus, no live amount rows, and no manual Retry.
  - [x] Keep the existing 90-second Debts safe-read timeout and map it to the same sanitized no-partial retryable state.
- [x] Add invariant-owning tests (AC: all)
  - [x] Domain tests cover zero-activity, archived/inactive identities, conversion conservation, quantization, ties, precision, and checked failures.
  - [x] Application tests cover snapshot-before-provider, Historical/future/same-currency contexts, deduplication, metadata validation, completion-order independence, empty ledgers, unavailable rates, calculation failure, and no partial balances.
  - [x] Infra tests retain snapshot atomicity/corruption coverage and all Story 4.3 provider fallback, bounds, LRU, rollover, lexical Decimal, single-flight, and concurrency tests.
  - [x] Web/router tests cover Historical default, successful balance ordering, archived labels, empty state, status/focus/`aria-busy`, timeout/unavailable/calculation no-partial output, native/HTMX parity, and 320px/400% behavior.
  - [x] Preserve root real-socket smoke, architecture fitness, security-header, readiness, and existing Summary regression coverage.
- [x] Run required validation (AC: all)
  - [x] `cargo fmt --all -- --check`
  - [x] `cargo check --workspace --all-features --locked`
  - [x] `SQLX_OFFLINE=true cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`
  - [x] `cargo test --workspace --all-features --locked`
  - [x] `cargo run --bin architecture-check --locked`
  - [x] Never use `cargo build --release` for validation.

### Review Findings

- [x] [Review][Patch] Enhanced recalculation failures leave prior financial results visible [debtor-web/templates/debts.html:35; debtor-web/src/handlers/debts.rs:47-49] — Added response-targets error routing and debt-specific sanitized error responses so enhanced failures replace the financial region.
- [x] [Review][Patch] Debt results never expose Updating or `aria-busy` state [debtor-web/templates/debts.html:32-35; static/css/app.css:201-204] — Added the result indicator target and visible Updating state while preserving the no-custom-JavaScript constraint.
- [x] [Review][Patch] Enhanced mode replacement loses radio focus [debtor-web/templates/debts.html:27,35-40] — Moved the native mode form outside the replaceable result region so the selected radio remains mounted and retains focus.
- [x] [Review][Patch] Debt failures omit attempted calculation context [debtor-web/src/handlers/debts.rs:47-49; debtor-web/src/handlers/response.rs:158-163,198-201; debtor-web/src/middleware.rs:266-274] — Added mode and UTC attempt context to debt calculation and timeout responses with sanitized target-currency wording.
- [x] [Review][Patch] Fixed-past stale quotes accept an invalid later fetch date [debtor-application/src/debts.rs:280-284] — Historical stale evidence now rejects any fetch date later than the requested context fetch date.
- [x] [Review][Patch] Synthetic-rate disclosure is suppressed by other warnings [debtor-web/src/handlers/debts.rs:91-101; debtor-web/templates/debts.html:78] — Combined stale, provisional, and synthetic warnings and labeled synthetic rate rows explicitly.
- [x] [Review][Patch] Snapshot Group identity is not checked against the requested route [debtor-application/src/debts.rs:195-196] — Mismatched snapshot Group IDs now map to safe invalid-storage failure.
- [x] [Review][Patch] Epic 5 sprint metadata remains backlog while its first story is in review [_bmad-output/implementation-artifacts/sprint-status.yaml:82-83] — Epic 5 is now tracked as in-progress.

## Dev Notes

### Developer Context

This is a brownfield vertical slice. `DebtService` already reads `LedgerSnapshot`, deduplicates rate contexts, fetches up to four rates concurrently, converts Spending source nets, quantizes, and currently invokes `simplify`. The key defects are that it does not seed all snapshot Participants at zero, quote validation is weaker than the accepted Summary validation, and the web handler performs unrelated Group/Participant reads after the calculation. The current Debts page renders transfers only and cannot satisfy the Balance-first UX. Correct these seams rather than introducing parallel debt infrastructure.

The domain function `add_converted_spending` intentionally uses `Spending::source_nets()`: Debts must calculate payer-paid amount minus each participant's share. This differs from the monthly payer-paid Summary, where shares must not be used. Apply no rounding per Spending, participant, or rate; aggregate exact Decimal values first, then quantize once.

### Architecture Compliance

- Preserve `root -> debtor-web/debtor-infra -> debtor-application -> debtor-domain` (AD-1/AD-2). Domain owns pure financial rules; application owns mode/context orchestration, quote validation, ports, and safe mapping; infra owns SQLx snapshot/provider/cache; web owns HTTP/rendering; root only composes.
- Apply AD-3 and AD-7: canonical Decimal/TEXT rules, checked arithmetic, one complete snapshot, no SQL monetary work, no paginated history for all-time calculation, and no network request inside the read transaction.
- Apply AD-9: immutable `CalculationContext`, exact `(source,target,R,F)` semantics, deterministic evidence ordering, synthetic same-currency rate `1`, completion-order independence, bounded concurrency, and truthful stale/provisional evidence.
- Apply AD-11/AD-18: native server-rendered Askama HTML is authoritative; pinned HTMX/response-targets are optional; no custom JS, inline scripts, client-side calculation, manual Retry, cards, gradients, animation, or color-only direction/state.
- Apply AD-15/AD-16: raw storage/provider diagnostics never cross application ports or user responses; use injected clocks/providers/snapshots and simple fakes in tests.

### Current Files To Update And Preserve

| Path | Current state | Story guardrail |
|---|---|---|
| `debtor-domain/src/debts/balance.rs` | Exact converted source-net accumulation and joint signed quantization already exist. | Preserve checked arithmetic and tie rules; add missing participant/edge tests, do not use floats or per-row rounding. |
| `debtor-application/src/debts.rs` | Owns `RateMode`, `RateQuote`, snapshot/provider ports, context deduplication, `DebtResult`, and `DebtService`. | Make Historical balances complete, seed every Participant, validate evidence, and do not make settlement a prerequisite. |
| `debtor-infra/src/db/repos/snapshots.rs` | Loads Group, full Spending aggregates, and owned Participants in one transaction. | Preserve complete archived/inactive identity hydration and commit before provider calls. |
| `debtor-infra/src/exchange_rates/frankfurter.rs` | Owns Story 4.3 exact cache/fallback/provider policy. | Reuse it; do not add provider/cache or move fallback policy inward. |
| `debtor-web/src/handlers/debts.rs` | Calculates then separately loads Group and Participants; fabricates `Participant {id}` labels if absent. | Use calculation-owned snapshot identity/projection; no second financial read and no ID fallback labels. |
| `debtor-web/src/templates.rs` | `DebtsTemplate` has transfer/rate fields but no Balance projection. | Add typed display projections for balances, states, context, and accessible status. |
| `debtor-web/templates/debts.html` | Transfer-only, link-based mode UI without Balance/status/no-partial states. | Implement native Historical-default control, Balance-first output, empty/unavailable states, disclosure, focus, and status contract. |
| `debtor-web/src/router.rs` | Existing `/groups/{id}/debts` route and generic safe-read middleware. | Keep route; preserve 90-second timeout and add route-specific tests, not an alternate route. |
| `src/composition.rs` | Composes one snapshot reader, shared rate provider/cache, clock, and `DebtService`. | Keep exactly one shared instance of each process-local owner. |
| `static/css/app.css` | Owns Editorial Contrast, focus, responsive tables, and target sizing. | Make only minimal Balance/rate/status layout changes; preserve 320px/400% no-horizontal-scroll behavior. |
| `specs/design.md` | Normative contract already covers Historical debts. | Do not change unless implementation exposes a genuine contract defect; synchronize all companions if changed. |

### Library / Framework Requirements

- Preserve pinned project versions: Rust 1.97.1/edition 2024, Axum 0.8.9, Askama 0.16.0, Tokio 1.53.1, SQLx 0.9.0, reqwest 0.13.4, `rust_decimal` 1.42.1, HTMX 2.0.10, response-targets 2.0.4. Add no dependency.
- `Decimal::checked_add`, `checked_sub`, `checked_mul`, and `checked_div` return `Option`; map `None` to safe calculation failure. `trunc_with_scale` truncates without rounding. Never use `f32`/`f64`, saturating arithmetic, or fallback-to-zero.
- Preserve reqwest connect/total/read timeout bounds and response-size enforcement in the existing adapter. Do not add a shorter request timeout that changes provider semantics.
- Askama derives templates at compile time; every new template field/method must exist on the context type and compile. Keep rate/cache policy out of templates.
- Current docs consulted 2026-08-19: Context7 `/websites/rs_rust_decimal`, `/seanmonstar/reqwest`, and `/askama-rs/askama`.

### Previous Story Intelligence

- Story 4.3 fixed exact fixed-past fallback, inclusive seven-day refreshable stale eligibility, future-context isolation, stale fetch-date evidence, typed Summary states, source continuity, and a one-context converted fragment. Do not regress these fixes.
- Story 4.2 established one-snapshot monthly composition, Group Currency captured in the snapshot, deterministic quote metadata, and bounded conversion failure behavior.
- Stories 3.3-3.5 established complete aggregate reads, current names for archived identities, native full-page authority, stable focus/status IDs, and deterministic concurrency tests.
- Story 4.3 review specifically caught stale quote validation rejecting valid fixed-past fallback and a converted fragment performing an unrelated Group read. Apply the same scrutiny here: calculation-owned snapshot identity is authoritative.

### Testing Requirements

- Use injected `Clock`, `LedgerSnapshotReader`, and `ExchangeRateProvider` fakes; no Axum, SQLite, network, wall clock, or sleeps in application tests.
- Use barriers/notifications for concurrency ordering; assert provider completion order cannot alter balances, rate disclosure, or warnings.
- Keep malformed/corrupt persistence tests in infra, pure arithmetic tests in domain, policy tests in application, rendering/status/HTTP tests in web, and composition behavior in root smoke tests.
- Assert no partial Balances/Transfers and no raw diagnostics for every unavailable, timeout, corrupt-data, arithmetic, and quantization failure.

### Anti-Patterns To Avoid

- Do not calculate only Participants encountered in allocations; initialize every Group-owned identity, including archived/inactive and zero-activity Participants.
- Do not issue separate Group/Participant reads after calculation to fill names, currency, or archive state.
- Do not use the 25-item Transactions page for all-time debts, SQL monetary aggregates, floats, per-Spending rounding, or zero substitution.
- Do not move rate fallback/cache logic into application or web, use unordered map iteration for visible order, or expose provider URLs, IDs, raw errors, dates, rates, or internal diagnostics in failures.
- Do not add Current mode, settlement persistence, repayment state, archival checks, manual Retry, custom JavaScript, or an alternate Debts route.

## References

- [Source: `_bmad-output/planning-artifacts/epics.md#Story 5.1: Calculate Exact Historical Balances`]
- [Source: `specs/design.md#Accounting And History`]
- [Source: `specs/design.md#Rates And Settlements`]
- [Source: `specs/design.md#Operational Limits`]
- [Source: `_bmad-output/project-context.md#Critical Don't-Miss Rules`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-3 - Exact monetary truth`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-7 - Snapshot-complete calculation reads`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-9 - Deterministic rate and settlement processing`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-11 - Native HTML authority`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-15 - Safe failures, logging, and readiness`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md#Financial Presentation`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md#Rate and Debt States`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md#Interaction Focus Matrix`]
- [Source: `debtor-domain/src/debts/balance.rs`]
- [Source: `debtor-domain/src/debts/simplify.rs`]
- [Source: `debtor-application/src/debts.rs`]
- [Source: `debtor-infra/src/db/repos/snapshots.rs`]
- [Source: `debtor-infra/src/exchange_rates/frankfurter.rs`]
- [Source: `debtor-web/src/handlers/debts.rs`]
- [Source: `debtor-web/src/templates.rs`]
- [Source: `debtor-web/templates/debts.html`]
- [Source: Context7 `/websites/rs_rust_decimal`, `/seanmonstar/reqwest`, `/askama-rs/askama`, consulted 2026-08-19]

## Dev Agent Record

### Agent Model Used

openai/gpt-5.6-luna

### Debug Log References

- Resolved `bmad-create-story` customization: no activation prepend/append or completion terminal steps configured.
- Loaded ordered sprint status, project context, normative design contract, Epic 5.1, architecture spine, UX contracts, prior Story 4.3, current debt/rate/snapshot/web code, and recent Git history.
- Used a parallel repository analysis pass to identify missing zero-activity Participants, transfer coupling, post-calculation identity reads, incomplete Debts UX, and no-partial/error gaps.
- Consulted current `rust_decimal`, `reqwest`, and Askama documentation through Context7 on 2026-08-19.

### Implementation Plan

- Preserve the existing snapshot reader and Frankfurter provider; extend the application result with snapshot-owned Group/Participant identity data.
- Validate snapshot ownership and quote metadata before checked financial aggregation. Seed all owned identities at zero, aggregate payer-minus-share nets exactly, and quantize once with existing domain rules.
- Replace the transfer-only Debts template with a native Historical-default Balance-first projection, retaining the existing route, timeout, shell, and optional HTMX enhancement.
- Validate in red-green order with a zero-activity Participant regression, same-currency provider isolation, application/web tests, full workspace tests, Clippy, architecture fitness, and password-helper checks.

### Completion Notes List

- Implemented snapshot-owned Historical Balance calculation with exact Decimal aggregation, zero-activity identity seeding, same-currency synthesis, quote metadata validation, and corrupt ownership rejection.
- Added calculation-owned Participant identity projection so Debts no longer performs unrelated Group/Participant reads or exposes identifier fallback labels.
- Reworked Debts HTML/CSS for Balance-first results, Historical/Current native mode controls, archived labels, exact currency/direction text, rate fetch-date disclosure, status/`aria-busy`, focus, empty, and no-partial-safe states.
- Added regression coverage for zero-activity identities and same-currency provider isolation; existing domain, application, infra, web, root smoke, and provider tests remain green.
- Validation passed: `cargo fmt --all -- --check`, workspace check, strict offline Clippy, full workspace tests, architecture fitness, and password-helper fmt/Clippy/tests.
- Story implementation is complete and ready for code review.
- Code review resolved 7 patch findings: enhanced failure replacement, debt status/focus behavior, attempted-context errors, stale quote bounds, synthetic disclosure, snapshot identity validation, and sprint metadata.

### File List

- `_bmad-output/implementation-artifacts/5-1-calculate-exact-historical-balances.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `debtor-application/src/debts.rs`
- `debtor-web/src/handlers/debts.rs`
- `debtor-web/src/templates.rs`
- `debtor-web/templates/debts.html`
- `static/css/app.css`

### Review Files

- `debtor-web/src/handlers/response.rs`
- `debtor-web/src/middleware.rs`

### Change Log

- 2026-08-19: Implemented exact Historical Balance calculation, snapshot identity projection, and Balance-first Debts UI; validation passed.
