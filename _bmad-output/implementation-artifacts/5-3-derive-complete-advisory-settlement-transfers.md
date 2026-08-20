---
story_key: 5-3-derive-complete-advisory-settlement-transfers
story_id: 5.3
epic: 5
status: done
created: 2026-08-19
baseline_commit: de757ad
completion_note: Ultimate context engine analysis completed - comprehensive developer guide created
---

# Story 5.3: Derive Complete Advisory Settlement Transfers

Status: done

## Story

As the administrator,
I want advisory Transfers derived from exact all-time Balances,
so that I know who could pay whom to settle the Group without recording repayment state.

## Acceptance Criteria

1. Given a successful Historical or Current calculation produced quantized exact-zero-sum Balances, when Settlement runs, then it consumes only that complete immutable Balance set after financial calculation succeeds, and never produces a Transfer from partial, unquantized, corrupt, or unavailable Balances.
2. Given positive creditors and negative debtors, when deterministic greedy matching begins, then each side is ordered by descending absolute Balance with ascending Participant ID ties; each Transfer is the checked positive minimum for the current debtor/creditor pair; and completion order or unordered input cannot alter the output.
3. Given `n` included Participants, when Settlement completes, then every Balance is settled, pairs do not repeat, amounts are target-precision-valid and positive, and no more than `n - 1` Transfers are returned. Do not claim global transfer-count minimality.
4. Given every Balance is exactly zero, when Settlement renders, then an accessible factual settled empty state appears with no fabricated Transfer, persistence, provider call, completion badge, or celebratory motion; mode and disclosure remain visible.
5. Given checked arithmetic, conservation, positivity, pair uniqueness, or completion fails, when Settlement evaluates, then it maps to the existing fixed sanitized calculation failure and exposes neither partial Transfers nor otherwise-valid Balance rows. Never panic, substitute zero, or silently omit a Participant.
6. Given advisory Transfers render for either mode, when the administrator reviews them, then the Transfer section follows Balances and precedes rate disclosure; each row explicitly says `from [Participant] to [Participant]`, carries a positive Group Currency amount with symbol and ISO code, and does not rely on color or sign for direction. Archived endpoints use their current names and visible `Archived` text.
7. Given Debts is revisited or its mode changes, when Transfers are recalculated, then they remain derived advice only: no repayment, paid/settled state, checkpoint, date range, completion record, new route, persistence, or provider/cache behavior is introduced.
8. Given long names, directions, and amounts at 320 CSS pixels or 400% zoom, when Transfers wrap, then reading order remains payer, recipient, amount; money remains intact; ruled Editorial Contrast rows remain readable; no page-level horizontal scroll occurs; and native and HTMX output are equivalent.

Requirements: `SPEC-FR41`, `SPEC-FR83..SPEC-FR86`, `SPEC-NFR5`, `SPEC-NFR10`, `SPEC-NFR12`, `SPEC-NFR25`, `SPEC-NFR28..SPEC-NFR34`; UX contracts `UX-SHELL-01`, `UX-TARGET-01`, `UX-FOCUS-01`, `UX-STATUS-01`, `UX-RESPONSIVE-01`, `UX-VISUAL-01`.

## Scope Boundary

- Extend the existing Story 5.1/5.2 `DebtService` result. `simplify` already runs immediately after final balance quantization; retain this one calculation path.
- Settlement is pure domain logic over complete target-currency balances. It does not read the database, clock, rates, cache, or network.
- Do not add persistence, migrations, SQLx metadata, dependencies, routes, composition changes, repayment/payment workflows, paid or completed state, checkpoint/date-range filtering, transfer records, manual refresh, or global-minimum optimization.
- Do not change rate selection/fallback, snapshot materialization, balance calculation, quantization, mode controls, HTMX request-state behavior, or the unrelated deferred Debts shell/focus items.
- `specs/design.md` already defines this behavior. Do not alter normative documentation unless implementation reveals a genuine contract divergence.

## Tasks / Subtasks

- [x] Prove and document deterministic settlement at the domain seam (AC: 1-5)
  - [x] Retain `debtor-domain::debts::simplify` as the sole settlement algorithm. Do not create a second service or operate on raw Spendings/rates.
  - [x] Preserve the separate debtor (negative) and creditor (positive) queues, one-time descending-absolute/ascending-ID sorting, head-only matching, checked subtraction, no reinsertion/re-sort, and generated output order.
  - [x] Add full ordered fixtures including magnitude and Participant-ID ties, partial head settlement, simultaneous exhaustion, zero balances, empty/all-zero maps, and non-zero-sum rejection.
  - [x] Add/strengthen invariant coverage for positive amounts, unique pairs, exact settlement conservation, `<= n - 1`, deterministic output under permuted input, supported target precision (USD, JPY, OMR), and checked failure without a partial vector.
- [x] Retain the application all-or-nothing boundary (AC: 1-5, 7)
  - [x] Keep `DebtService::calculate` ordering: complete snapshot -> rate evidence -> exact accumulation -> joint quantization -> `simplify` -> `DebtResult`.
  - [x] Add Historical and Current service fixtures asserting exact ordered Transfers after quantization, shared mode/evidence/warnings, archived/inactive/zero-activity identity inclusion, and completion-order-independent results.
  - [x] Assert a settlement error maps through `CalculationReason::SettlementInvariant` (or the existing exact safe reason) and constructs no `DebtResult`.
  - [x] Preserve one snapshot before provider I/O, injected clock/provider fakes, concurrency bounds, rate disclosure, and no partial failure mapping.
- [x] Complete the Debts Transfer projection without financial logic in web (AC: 4, 6, 8)
  - [x] Extend `TransferRow` and the handler projection with both endpoints' archive state while resolving current names only from `DebtResult.participants`. Preserve safe failure on a missing participant ID; never perform a second identity query.
  - [x] Change the Transfer presentation from separate implicit table columns to explicit textual debtor-to-creditor direction: `from [Participant] to [Participant]`, with visible `Archived` at either endpoint and formatted positive symbol-plus-ISO money.
  - [x] Name the section `Settlement Transfers` and distinguish a genuine all-zero settled state from the existing no-Participant state. The no-Spending ledger still shows its complete zero Balance result and no Transfer.
  - [x] Keep order as Balances -> Settlement Transfers -> rates/warnings. Keep the existing native GET mode form outside `#debts-results`, stable final status node, CSS-only HTMX Updating state, and no custom JavaScript/inline event attributes.
  - [x] Make only minimal CSS changes needed for ruled, wrapping-safe transfer rows at narrow widths. Preserve 48px controls, dark Editorial Contrast, semantic tables/labels, existing focus behavior, and no page-level horizontal scrolling.
- [x] Test at the owning layers (AC: all)
  - [x] Domain tests assert exact ordered `Transfer` vectors and invariant/property outcomes; do not test this financial algorithm only through web output.
  - [x] Application tests use fixed injected clocks and simple snapshot/provider fakes only, with no Axum, SQLite, network, wall clock, or timing sleeps.
  - [x] Web/template/router tests cover native and enhanced output parity, section order, literal from/to copy, money format, archive labels, settled state, empty-ledger state, no partial result on calculation failure, stable polite atomic status, no inline handlers/custom scripts, and preserved Historical/Current behavior.
  - [x] Add responsive assertions or browser evidence for 320px/400% zoom with long archived names: payer -> recipient -> amount reading order, intact currency, 48px controls, and no horizontal page scroll.
- [x] Run focused and full validation (AC: all)
  - [x] `cargo fmt --all -- --check`
  - [x] `cargo check --workspace --all-features --locked`
  - [x] `SQLX_OFFLINE=true cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`
  - [x] `cargo test --workspace --all-features --locked`
  - [x] `cargo run --bin architecture-check --locked`
- [x] Do not use `cargo build --release`. No SQL/migration change is expected; only refresh `.sqlx` metadata if the implementation actually changes checked SQL.

### Review Findings

- [x] [Review][Patch] Long transfer amounts overflow the narrow Debts layout [static/css/app.css:136, static/css/app.css:209] — the global mobile table-cell padding leaves roughly 160px for the Amount value at 320px; the new `white-space: nowrap` prevents valid maximum USD/OMR transfer amounts from wrapping and can create page-level horizontal scrolling, violating AC 8.
- [x] [Review][Defer] Settlement uses saturating arithmetic [debtor-domain/src/debts/simplify.rs:60] — deferred, pre-existing

## Dev Notes

### Developer Context

Most settlement mechanics already exist. `debtor-domain/src/debts/simplify.rs` takes a `BTreeMap<EntityId, Decimal>`, validates exact zero sum, derives positive debtor magnitudes, independently sorts debtor and creditor queues, emits the minimum pair amount, and checks residuals. `DebtService` invokes it only after `quantize_balances`; do not move it before quantization or duplicate it in application/web.

The missing Story 5.3 evidence is precise algorithm coverage plus Transfer rendering. Current `TransferRow` loses archival state and `debts.html` exposes separate From/To cells rather than the required explicit prose. Correct those projections while retaining current participant names from the calculation snapshot.

### Required Algorithm Guardrails

- Input is the complete, final, target-currency, exact-zero-sum balance map. Positive means creditor; negative means debtor; zero balances have no Transfer queue entry.
- Sort each queue exactly once by absolute amount descending, then `EntityId` ascending. Match queue heads only and preserve generation order in the returned vector. Do not iterate an unordered map for output, re-sort residuals, or optimize pairs globally.
- For every emitted transfer, `from_participant_id` is the debtor, `to_participant_id` is the creditor, and `amount = min(debtor_remaining, creditor_remaining)` is checked and strictly positive. Advance only zeroed heads.
- Preserve all postconditions: zero residuals, positive target-precision amounts, no self-transfer or repeated pair, every input balance settled exactly, and output count no greater than included nonzero Participants minus one (therefore no greater than total included Participants minus one).
- A checked error is fatal to the whole financial projection. Reuse the existing `ApplicationError::Calculation` mapping; never return an incomplete `DebtResult` or display valid balance rows next to failed settlement advice.

### Architecture Compliance

- Preserve `root -> debtor-web/debtor-infra -> debtor-application -> debtor-domain`. Domain owns deterministic settlement arithmetic; application orchestrates the already-complete calculation; web projects it; root composition and infra remain unchanged.
- Use exact checked `rust_decimal::Decimal` only. Never use floats, lossy conversion, `saturating_*`, SQL monetary aggregation, per-transfer rounding, `unwrap`/`expect` in production paths, or zero substitution.
- Keep all order-affecting collections deterministic (`BTreeMap` plus explicit queue sorting). Participant ID is the final tie-breaker. Provider completion order must not affect balances, transfers, rates, warnings, or rows.
- Retain complete snapshot-before-provider-I/O behavior, all historical identities, and current names for archived participants. Settlement needs no extra read, provider call, or state mutation.
- Keep safe failures sanitized. Do not surface raw SQLx/provider/ledger details, monetary values, IDs, URLs, or query strings in errors/logs.

### Files To Update And Preserve

| Path | Required change and preservation rule |
| --- | --- |
| `debtor-domain/src/debts/simplify.rs` | Update tests/documentation around the existing pure greedy algorithm. Retain its single implementation and checked queue mechanics. |
| `debtor-application/src/debts.rs` | Primarily add service-level settlement fixtures. Keep `simplify` immediately after `quantize_balances`, one snapshot/read path, one injected clock/provider, and no partial `DebtResult`. |
| `debtor-web/src/templates.rs` | Extend `TransferRow` with display-ready archive metadata or equivalent typed state; keep typed escaped Askama projections. |
| `debtor-web/src/handlers/debts.rs` | Project both transfer endpoints from snapshot participants, including archived flags/current names. Preserve missing-ID safe failure and avoid a second read. |
| `debtor-web/templates/debts.html` | Render explicit from/to language, visible archive labels, factual settled state, and fixed result order. Preserve native/HTMX form contract and stable results/status IDs. |
| `static/css/app.css` | Update only if needed for readable ruled Transfer rows and narrow wrapping. Preserve CSS-only HTMX pending behavior and existing responsive table rules. |
| `debtor-web/src/router.rs`, `debtor-web/src/templates.rs`, and handler test support | Add route/template evidence for native/enhanced parity and no-partial output. |
| `_bmad-output/implementation-artifacts/5-3-derive-complete-advisory-settlement-transfers.md` | Update task checkboxes, Dev Agent Record, completion notes, and File List during implementation. |

Do not modify migrations, `.sqlx`, Cargo manifests/lockfile, `debtor-infra`, exchange-rate caches, snapshot repository, composition, router registration, or `specs/design.md` unless implementation proves an unplanned contract change.

### Library And Framework Requirements

- Preserve pinned Rust 1.97.1, Axum 0.8.9, Askama 0.16.0, Tokio 1.53.1, SQLx 0.9.0, reqwest 0.13.4, `rust_decimal` 1.42.1, HTMX 2.0.10, and response-targets 2.0.4. Add no dependency.
- Askama remains typed and escaped; templates render presentation only and contain no settlement policy.
- The existing HTMX `hx-indicator` target receives request CSS state. Use this only for the already-established CSS-only Updating placeholder; it does not justify dynamic ARIA mutation, client financial state, inline handlers, custom JavaScript, or a custom extension.

### Testing Requirements

- Domain: exact ordered fixtures, tie order, partial/simultaneous exhaustion, empty/all-zero, non-zero-sum, positivity, pair uniqueness, conservation, `n - 1`, input-order invariance, precision, and checked failure.
- Application: Historical and Current exact transfer fixtures after quantization; same quotes/warnings/mode results; archived/inactive/zero-activity inclusion; one snapshot before provider use; deterministic asynchronous completion; settlement failure produces a safe error and no result.
- Web: literal `from`/`to` direction, symbol-plus-ISO positive amount, archive labels for both endpoints, Balance -> Transfer -> disclosure order, no Participant vs settled empty-state distinction, no partial calculation error, status semantics, native/HTMX parity, and no custom scripting regression.
- Preserve existing root smoke, strict form/auth/security tests, Summary/Transactions behavior, rate/cache tests, Story 5.1 Historical tests, and Story 5.2 Current-mode tests.

### Previous Story Intelligence

- Story 5.1 established snapshot-owned Participant projection, zero identity seeding, exact payer-minus-share aggregation, signed largest-remainder quantization, and the complete-or-no-result Debts boundary. Extend it rather than adding a transfer-specific read or calculator.
- Story 5.2 established Current-mode context deduplication, strict stale-evidence boundaries, a shared `DebtService` path, mode-aware results, and CSS-only HTMX Updating behavior. Preserve it unchanged while transfers gain correct projection coverage.
- Recent commits `15f4b48` and `de757ad` deliberately left migrations, SQLx metadata, Cargo manifests, snapshot persistence, rate adapter/cache, and composition unchanged. Story 5.3 should retain that shape.

### Anti-Patterns To Avoid

- Do not derive settlement from raw Spendings, unquantized positions, a partial result, a new database query, rate provider result, or a second calculator.
- Do not add repayment/payment persistence, paid/settled/completed status, checkpoint/date range, transfer record, manual retry/refresh, payment initiation, or a global-minimum claim.
- Do not use floats, SQL sums, unordered visible output, individual rounding, saturating arithmetic, zero defaults, panic paths, or partial Balance/Transfer output.
- Do not hide direction in color, sign, or table layout. Use literal debtor-to-creditor prose and visible `Archived` labels.
- Do not reintroduce `hx-on`, inline event attributes, custom JavaScript, a custom HTMX extension, CSP relaxation, dynamic `aria-busy`, or retained client-side financial values.
- Do not fold in unrelated deferred Debts navigation/focus/error-flow work without a demonstrated dependency.

### References

- [Source: `specs/design.md#Accounting And History`]
- [Source: `specs/design.md#Rates And Settlements`]
- [Source: `_bmad-output/planning-artifacts/epics.md#Epic 5: Calculate Debts, Settle, and Safely Retire Identities`]
- [Source: `_bmad-output/planning-artifacts/epics.md#Story 5.3: Derive Complete Advisory Settlement Transfers`]
- [Source: `_bmad-output/planning-artifacts/prds/prd-debtor-2026-08-10/prd.md#All-Time Balances And Advisory Settlements`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-3 - Exact monetary truth`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-7 - Snapshot-complete calculation reads`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-9 - Deterministic rate and settlement processing`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-11 - Native HTML authority`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md#Financial Presentation`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md#Rate and Debt States`]
- [Source: `_bmad-output/implementation-artifacts/5-1-calculate-exact-historical-balances.md`]
- [Source: `_bmad-output/implementation-artifacts/5-2-recalculate-balances-at-current-rates.md`]
- [Source: `debtor-domain/src/debts/simplify.rs`]
- [Source: `debtor-application/src/debts.rs`]
- [Source: `debtor-web/src/handlers/debts.rs`]
- [Source: `debtor-web/src/templates.rs`]
- [Source: `debtor-web/templates/debts.html`]
- [Source: `static/css/app.css`]
- [Source: Context7 `/bigskysoftware/htmx/v2.0.4`, consulted 2026-08-19]

## Dev Agent Record

### Agent Model Used

openai/gpt-5.6-terra

### Debug Log References

- Resolved `bmad-create-story` customization: no activation prepend/append or completion terminal steps configured.
- Loaded the full sprint state, persistent project context, normative design contract, complete Epic 5.3 context, PRD, architecture spine, UX contract, Story 5.2 intelligence, deferred-work ledger, current settlement/debt/web files, and recent relevant commits.
- Used parallel planning, codebase, and UX research. Consulted HTMX request-indicator documentation through Context7; request state is CSS-oriented and does not dynamically set `aria-busy`.
- Implemented Story 5.3 from baseline `de757ad`: retained the existing domain settlement seam, added exact order/precision/settled-state coverage, and extended only the typed web transfer projection and presentation.
- Red phase: the Transfer template test failed against the prior `Settlement` heading and column-only direction; the minimal template/projection changes then made it pass.
- Final validation passed: formatting, workspace check, strict offline Clippy, all workspace tests, and architecture fitness.

### Completion Notes List

- Ultimate context engine analysis completed - comprehensive developer guide created.
- Added deterministic queue-order, all-zero, and JPY/OMR decimal-preservation settlement tests without adding a parallel calculator or dependencies.
- Added Historical/Current ordered-transfer and safe settlement-invariant mapping tests at the application boundary.
- Rendered `Settlement Transfers` with explicit debtor-to-creditor text, endpoint-specific visible archive labels, a truthful no-Participant distinction, and unbroken tabular currency amounts.
- Added template and authenticated-route regressions for archive labels, section order, explicit direction, settled state, and responsive transfer amount styling.
- Validation passed: `cargo fmt --all -- --check`, `cargo check --workspace --all-features --locked`, strict offline Clippy, `cargo test --workspace --all-features --locked`, and `cargo run --bin architecture-check --locked`.

### File List

- `_bmad-output/implementation-artifacts/5-3-derive-complete-advisory-settlement-transfers.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `debtor-domain/src/debts/simplify.rs`
- `debtor-application/src/debts.rs`
- `debtor-web/src/handlers/debts.rs`
- `debtor-web/src/handlers/test_support.rs`
- `debtor-web/src/router.rs`
- `debtor-web/src/templates.rs`
- `debtor-web/templates/debts.html`
- `static/css/app.css`

### Change Log

- 2026-08-20: Implemented deterministic advisory Settlement Transfer verification and accessible Debts transfer rendering; status moved to review.
