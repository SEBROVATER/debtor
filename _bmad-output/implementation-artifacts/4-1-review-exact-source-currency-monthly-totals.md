---
story_key: 4-1-review-exact-source-currency-monthly-totals
story_id: 4.1
epic: 4
status: done
baseline_commit: 4c0989e69b402b1c6730a50d966a6697f981c402
created: 2026-08-18
completion_note: Ultimate context engine analysis completed - comprehensive developer guide created
---

# Story 4.1: Review Exact Source Currency Monthly Totals

Status: done

## Story

As the administrator,
I want to see this month's exact totals in each original Source Currency,
so that I can understand current spending even when exchange rates are unavailable.

## Acceptance Criteria

1. **Current UTC month only:** Given a selected Group has Spendings across multiple dates, Summary includes only Spendings whose `spent_date` falls in the current UTC calendar month. No arbitrary date-range control, all-time statistic, or extra analytics surface is added.
2. **Exact source grouping:** Given included Spendings use one or more Source Currencies, Summary shows one Group total and per-Payer paid totals for each original Source Currency. Aggregation uses checked `Decimal` arithmetic in Rust; SQLite performs no monetary parsing, conversion, or `SUM`/monetary aggregation.
3. **Historical identity preservation:** A current-month Spending still contributes when its Payer was renamed or archived. Summary resolves and displays the current Participant name and visibly marks archived identity; active-only filtering must not change historical totals.
4. **Deterministic exact output:** Successfully decoded stored amounts retain valid Source Currency precision. Currency blocks and Payer rows have deterministic ordering, with Participant ID as the final Payer tie-breaker, and output is independent of database row order.
5. **Empty month:** A Group with no current-month Spendings renders an accessible source-summary empty state and no fabricated zero-valued currencies. Group navigation and Add Spending remain available.
6. **Provider independence:** Frankfurter availability, rate-cache state, and later converted-summary availability do not affect source totals. Summary source calculation performs no exchange-rate provider call, and ledger CRUD remains usable when rates are unavailable.
7. **Safe failure boundary:** Stored corruption or checked aggregation failure produces sanitized feedback, no partial affected source summary, and no zero substitution. Logs and user-facing responses contain no amounts, identities, SQL, row values, IDs, or adapter diagnostics.
8. **Summary navigation and focus:** Native Summary navigation returns the existing canonical `/groups/{id}` response with the stable Summary heading as the forward focus target. Enhanced navigation, if retained, uses the same URL and response contract; the five-link shell marks Summary current.
9. **Source result hierarchy:** When data exists, the Summary shows an explicit `YYYY-MM` month title and `YYYY-MM · UTC` context. Each Source Currency block shows its Group total before indented per-Payer rows; every amount includes currency symbol plus ISO code and uses tabular numerals.
10. **Responsive accessible rendering:** At 320 CSS pixels, 400% zoom, keyboard-only use, and wide composition, source blocks, long Participant names, currencies, amounts, and shell controls remain readable without page-level horizontal interaction. Controls remain at least 48 by 48 CSS pixels, focus remains visible, and DOM/focus order is not changed between narrow and wide layouts.
11. **Scoped status parity:** Pending, empty, and unavailable source-summary states use one stable scoped polite atomic status and `aria-busy`; individual financial amounts are not live regions. Native full-page and optional HTMX-enhanced responses are equivalent, and no custom JavaScript, manual rate Retry, or provider-coupled source projection is introduced.

Requirements: `SPEC-FR41`, `SPEC-FR67..SPEC-FR68`, `SPEC-FR72`; `SPEC-NFR2`, `SPEC-NFR5..SPEC-NFR7`, `SPEC-NFR10`, `SPEC-NFR14..SPEC-NFR16`, `SPEC-NFR25..SPEC-NFR30`, `SPEC-NFR32..SPEC-NFR34`; UX contracts `UX-SHELL-01`, `UX-TARGET-01`, `UX-FOCUS-01`, `UX-STATUS-01`, `UX-RESPONSIVE-01`, `UX-VISUAL-01`.

## Scope Boundary

- Implement the first runnable Summary financial projection: current UTC-month exact Source Currency Group and Payer totals in the existing `/groups/{id}` Summary route.
- Reuse the existing complete ledger snapshot and current Participant identity patterns. Keep the source projection provider-independent.
- Keep Group Currency conversion, rate contexts, Frankfurter calls, stale/provisional evidence, converted quantization, and whole converted-region degradation for Stories 4.2 and 4.3.
- Do not add arbitrary date filters, all-time statistics, search, exports, charts, dashboards, manual refresh, a new Summary route, a global Participant surface, or custom application JavaScript.
- Do not use the paginated Transactions reader for Summary totals, `DebtService`/`RateMode` for source aggregation, `Spending::source_nets` for Payer-paid totals, SQL monetary aggregation, floats, or duplicate snapshot/rate/repository abstractions.
- Prefer no migration. If SQL or migrations genuinely change, update `specs/design.md` first, then refresh `.sqlx`, migrate a temporary SQLite database, and run online SQLx preparation as required by `AGENTS.md`.

## Tasks / Subtasks

- [x] Audit and preserve the existing Summary vertical slice before editing (AC: all)
  - [x] Re-read current `group_detail`, `build_group_template`, `GroupTemplate`, `group.html`, shell/focus markup, and test fakes against repository reality.
  - [x] Preserve existing Group settings, Participant setup, Add Spending availability, Transactions link/history, archived read-only behavior, authenticated headers, and native/HTMX shell semantics.
  - [x] Confirm no existing Summary conversion or SQL aggregate path is silently retained as a competing projection.
- [x] Define the application-owned source-summary contract (AC: 1-7)
  - [x] Add the smallest plural summary module/types and narrow reader/use-case port consistent with `*Reader`, `*UseCases`, `*Service`, `*Input`, `*View`, and `*Row` naming.
  - [x] Accept an injected `Clock` or equivalent application-owned UTC date source so month-boundary tests do not use wall clock.
  - [x] Materialize the Group and complete Spendings from one consistent read snapshot before filtering/aggregation; do not hold a database transaction while doing unrelated work.
  - [x] Filter by `NaiveDate` current UTC year/month in Rust. Define inclusive first day and exclusive first day of next month; cover December/January rollover.
  - [x] Aggregate each Spending's single Payer allocation by `(Source Currency, Participant ID)` using checked `Decimal::checked_add`. Derive each currency Group total from the exact Payer buckets or independently checked totals, but ensure displayed Group total equals displayed Payer rows exactly.
  - [x] Use explicit deterministic ordering: supported Currency code/order, then Payer Participant ID ascending. Do not rely on SQL row order or hash-map iteration.
  - [x] Validate/retain canonical persisted decimals and currency precision already enforced by snapshot hydration. Map invalid stored data and arithmetic overflow to safe application reasons without returning partial buckets.
- [x] Reuse or extend infrastructure snapshot seams (AC: 2-4, 7)
  - [x] Prefer `debtor-infra/src/db/repos/snapshots.rs` and its `LedgerSnapshotReader`/complete aggregate path. If the existing port is insufficient for current Participant identity projection, add the narrowest snapshot-consistent identity contract rather than separately loading inconsistent Group/Participant data.
  - [x] Keep SQLx statements compile-time checked, money as canonical SQLite `TEXT`, and all monetary parsing/aggregation in Rust.
  - [x] Preserve `spendings.group_id` ownership and restrictive Participant references. Archived Participants remain hydratable because history references are protected.
  - [x] Do not add a provider call, rate cache dependency, or transaction spanning provider I/O.
- [x] Build typed Summary rendering projection (AC: 3, 5, 8-11)
  - [x] Extend `GroupTemplate` with typed source-summary state/rows. Templates render fields only; date filtering, Decimal arithmetic, sorting policy, and error mapping remain in Rust.
  - [x] Source rows carry validated display data: currency code/symbol, exact formatted amount, current Participant name, Participant ID ordering result, color as supplementary marker, and visible `Archived` text where applicable.
  - [x] Render the Summary heading with stable focus semantics and explicit `YYYY-MM` plus `YYYY-MM · UTC` context. Keep Summary current in the existing five-link shell.
  - [x] Render Source Currency Group total before per-Payer rows with semantic definition-list relationships. Include symbol and ISO code on every amount.
  - [x] Render empty and failure states with one scoped status node and `aria-busy`; no amount rows are live regions and no currencies/zeros are fabricated.
  - [x] Preserve Add Spending as a native link for active Groups with active Participants and retain the disabled guidance/recovery link when none exist. Archived Groups remain readable and mutation-free.
  - [x] Preserve dark Editorial Contrast: charcoal/warm paper, serif major totals, ruled sections, square geometry, no cards/gradients/animation/hover lift, and text paired with status color.
  - [x] Keep native HTML authoritative; no HTMX-specific Summary enhancement or custom JavaScript was needed.
- [x] Add invariant-owning tests (AC: all)
  - [x] Application tests cover current-month inclusion/exclusion, UTC month/year rollover, multiple currencies, multiple Payers, renamed/archived Payers, deterministic ordering, checked overflow, and no partial result/provider dependency.
  - [x] Existing snapshot/infra tests continue to cover complete hydration, canonical-decimal corruption rejection, Group scoping, archived identity retention, and row-order-independent reads.
  - [x] Web/router tests cover Summary hierarchy, month/context labels, symbol+ISO amounts, current shell state, stable heading focus, empty state, provider-independent rendering, sanitized failure, Add Spending preservation, status semantics, and no partial totals.
  - [x] Template/CSS assertions and implementation preserve stable IDs/classes, 48px target/focus rules, tabular numerals, wrapping names, and no page-level horizontal-scroll dependency; browser geometry/contrast remains manual because no browser harness exists.
  - [x] Root real-socket authentication/read/shutdown smoke coverage and architecture fitness remain passing.
- [x] Run validation (AC: all)
  - [x] `cargo fmt --all -- --check`
  - [x] `cargo check --workspace --all-features --locked`
  - [x] `SQLX_OFFLINE=true cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`
  - [x] `cargo test --workspace --all-features --locked`
  - [x] `cargo run --bin architecture-check --locked`
  - [x] Independent password-helper fmt, Clippy, and locked tests.
  - [x] Never used `cargo build --release`.

### Review Findings

- [x] [Review][Patch] Keep source amounts and Participant identity projections on one consistent read snapshot [debtor-application/src/summaries.rs:100-105] — moved Participant identity hydration into `LedgerSnapshot`, loaded by the same SQLite read transaction. Sources: blind+edge+auditor. Severity: medium. Resolved.
- [x] [Review][Patch] Avoid calculating the unbounded monthly snapshot for Manage and Spending-form requests [debtor-web/src/handlers/spending_views.rs:140-157] — added an explicit Summary inclusion flag; Manage and Spending-form renders skip the source calculation. Severity: medium. Resolved.
- [x] [Review][Patch] Add a visible focus indicator for the programmatic Summary heading [debtor-web/templates/group.html:172; static/css/app.css:80] — added a dedicated `.financial-results h2:focus` outline rule. Severity: medium. Resolved.
- [x] [Review][Patch] Keep the unavailable fallback month tied to the calculation attempt [debtor-web/src/handlers/spending_views.rs:141-144] — capture and reuse the fallback date around the source calculation. Severity: low. Resolved.
- [x] [Review][Patch] Test December-to-January filtering through `source_summary` [debtor-application/src/summaries.rs:337-352] — added an actual January aggregation test with December and January fixtures. Severity: low. Resolved.
- [x] [Review][Patch] Add supported-currency precision and multi-currency display assertions [debtor-application/src/summaries.rs:166-173; debtor-web/src/router.rs:500-520] — added JPY/OMR formatter coverage and EUR/USD rendered symbol-plus-ISO assertions. Severity: low. Resolved.
- [x] [Review][Patch] Add rendering assertions for archived identity and Summary accessibility attributes [debtor-web/src/router.rs:500-580] — added archived-name, `Archived`, `aria-busy`, `aria-live`, and `aria-atomic` assertions. Severity: low. Resolved.

## Dev Notes

### Developer Context

Epic 3 is complete and provides exact Spending CRUD, complete aggregate hydration, current Participant-name projections, fixed keyset Transactions history, supervised mutations, and the established Summary shell. The current `/groups/{id}` page has Participants and an Expense history link, but no financial Summary. Story 4.1 is the first Summary financial vertical slice and must end in visible source totals, not merely an infrastructure milestone.

The source summary is a paid-total projection. It must use each Spending's one Payer allocation, not `Spending::source_nets`, because source nets combine payer and Share amounts for debt arithmetic. Shares are not displayed or aggregated by this story. Archived or renamed Payers remain part of the historical aggregate; resolve their current Participant projection and mark archival visibly.

The existing `LedgerSnapshotReader` loads the Group and all complete Spendings from one SQLite read transaction, validates canonical Decimal values and complete aggregates, commits the read, and returns `LedgerSnapshot`. This is the strongest existing seam. Reuse it unless current-name projection requires a narrowly extended consistent read. The current `SpendingReader::spending_history_page` is deliberately bounded to 25 rows and is not valid for monthly totals.

The source section must be independently usable when Frankfurter is offline or conversion is not implemented. Do not instantiate or call `ExchangeRateProvider`, `DebtService`, or any rate cache from the source-summary path. Later Stories 4.2 and 4.3 consume the same complete snapshot/projection foundation for Group Currency conversion and unavailable behavior; do not pre-implement those later responsibilities here.

### Current Files To Update And Preserve

| Path | Current state | Story-specific change/preservation |
|---|---|---|
| `debtor-application/src/lib.rs` | Re-exports `debts`, `spendings`, groups, participants, and errors. | Add/re-export the smallest summary module if needed; keep framework and SQLx types out of application ports. |
| `debtor-application/src/debts.rs` | Owns `Clock`, `LedgerSnapshot`, `LedgerSnapshotReader`, rate orchestration, and all-time debt calculation. | Reuse `Clock` and snapshot contract where appropriate, but do not couple source totals to `DebtService`, rates, conversion, or debt balances. |
| `debtor-application/src/errors.rs` | Safe `ApplicationError`, `StorageReason::InvalidData`, and calculation reasons currently named for debts. | Extend safe reason taxonomy only if necessary; preserve sanitized mapping and avoid leaking Decimal values, IDs, or adapter diagnostics. |
| `debtor-infra/src/db/repos/snapshots.rs` | Loads Group, all spending parents, payer/share rows, validates canonical decimals and complete `Spending`, then commits. | Reuse/extend only for a consistent source-summary read. Preserve checked SQLx, canonical corruption rejection, restrictive history, and no SQL monetary aggregation. |
| `debtor-web/src/handlers/groups.rs` | `group_detail` authenticates, parses optional Transactions cursor, calls `build_group_template`, renders or maps errors. | Keep route `/groups/{id}` and cursor behavior. Keep handlers thin; map source-summary errors to safe scoped/full-page output without rate calls. |
| `debtor-web/src/handlers/spending_views.rs` | `build_group_template` loads Group, memberships, bounded spending page, forms, shell, and constructs `GroupTemplate`. | Add source-summary orchestration/projection here only as composition. No Decimal arithmetic, filtering policy, SQL, or provider logic in the handler/projection builder. Preserve Manage and spending-form paths. |
| `debtor-web/src/templates.rs` | `GroupTemplate` contains shell, members, bounded spending rows, forms, lifecycle/settings state. | Add typed Summary state and rows. Keep Askama compile-time fields aligned with `group.html`; do not expose domain policy to templates. |
| `debtor-web/templates/group.html` | Existing five-link shell; Summary renders Participants and Transactions link; archived branch is read-only. | Add semantic source-summary sections to Summary only. Preserve stable `group-heading`, `aria-current`, Add Spending, archived warning/readability, and native/HTMX shell behavior. |
| `debtor-web/src/state.rs` | App state owns Groups, Participants, Spendings, Debts, Clock, token/session/runtime controls. | Inject the application-owned summary use case/reader through state if a separate port is introduced; do not expose infra concrete types. |
| `src/composition.rs` | Composes one SQLite store as Group/Participant/Spending readers and `LedgerSnapshotReader`; composes `DebtService`, root mutation executor, and web state. | Compose the Summary service against the existing store/snapshot reader and shared clock. Do not create a second store, snapshot owner, clock, or provider. |
| `static/css/app.css` | Existing Editorial Contrast, 48px targets/focus, responsive breakpoint, transaction and form styles; Summary currently has only generic/card rules. | Add minimal ruled source-summary styles, tabular amount alignment, archived/status/empty states, and narrow/wide composition. Remove/avoid card-heavy treatment for the new financial section. |
| `debtor-web/src/handlers/test_support.rs` | Fake Group/Participant/Spending/Debt use cases and test AppState. | Extend fakes with summary data/errors and provider-call observation if needed. Keep tests simple and no mocking framework. |
| `debtor-web/src/router.rs` | Protected `/groups/{id}` Summary route and large shared auth/security/admission suite. | Extend Summary rendering/focus/native-enhancement tests; route shape should not change. Preserve security headers, body limits, timeout, and auth middleware. |
| `migrations/20260517000004_create_spendings.up.sql` | Stores `spent_date` as validated ISO TEXT and has `(group_id, spent_date DESC, id DESC)` index; total/currency are TEXT/closed codes. | Prefer no change. Existing schema supports a complete read; do not add SQL aggregates or monetary columns. |
| `.sqlx/*` | Committed compile-time query metadata. | Refresh only if SQL changes. Any refresh requires temporary migration and online `cargo sqlx prepare --workspace --check`. |
| `specs/design.md` | Normative product/architecture contract. | If implementation changes behavior or contracts beyond already specified Story 4.1 behavior, update it first and synchronize affected artifacts. Do not silently resolve divergence. |

### Architecture Compliance

- Preserve `debtor -> debtor-web/debtor-infra -> debtor-application -> debtor-domain`. Domain owns pure deterministic financial rules; application owns the use case, UTC month policy, and ports; infra owns SQLx snapshot adapters; web owns HTTP/rendering/safe mapping; root owns composition.
- Apply AD-3: exact `rust_decimal::Decimal`, checked addition, canonical SQLite TEXT hydration, currency precision validation, no floating point, rounding, zero substitution, or SQL monetary aggregation.
- Apply AD-4/AD-5: historical Participant identities remain included after archive/rename. Do not filter by active membership, create reusable/global identity abstractions, or make Participants application users.
- Apply AD-7: use a complete consistent ledger read for financial calculation. Do not use fixed 25-row history pages, mixed-version reads, or a database transaction for future provider work.
- Apply AD-9 only by exclusion in this story: no rate context, provider call, cache, stale/provisional classification, conversion, or quantization. Those belong to 4.2/4.3.
- Apply AD-11/AD-18: semantic Askama/native HTML is authoritative; pinned HTMX is enhancement only. Cite `UX-SHELL-01`, `UX-TARGET-01`, `UX-FOCUS-01`, `UX-STATUS-01`, `UX-RESPONSIVE-01`, and `UX-VISUAL-01` in tests/evidence.
- Apply AD-15: invalid persisted data and checked aggregation failure are safe bounded failures; raw SQLx, IDs, values, identities, provider data, and request data never reach responses or logs.
- Apply AD-16: use injected `Clock`/fakes in application tests. Keep domain/application tests independent of Axum, SQLite, network, and wall clock; test infrastructure and web adapters separately.

### Library / Framework Requirements

- Keep pinned Rust `1.97.1` edition 2024, Axum `0.8.9`, Askama `0.16.0`, Tokio `1.53.1`, SQLx `0.9.0`, `rust_decimal 1.42.1`, HTMX `2.0.10`, and response-targets `2.0.4`. Add no dependency for this story.
- Axum `Path`, `Query`, and `State` remain at the web boundary. The current official Axum guidance supports extracting typed route/query/state values in handlers and returning a unified `Response`; convert to application-owned values before calling ports. Consulted via Context7 `/tokio-rs/axum` on 2026-08-18.
- Askama `#[derive(Template)]` with `#[template(path = "...")]` remains the compile-time typed rendering boundary. Struct fields must match template variables; keep aggregation and error policy in Rust. Consulted via Context7 `/askama-rs/askama` on 2026-08-18.
- `rust_decimal::Decimal::checked_add` returns `Option<Decimal>` on overflow. Map `None` to the safe calculation/storage failure path; never use saturating arithmetic or floating-point conversion. Consulted via Context7 `/websites/rs_rust_decimal` on 2026-08-18.
- Existing pinned HTMX/response-targets assets may enhance the Summary section only if native navigation remains complete. Do not add custom events, custom extensions, inline scripts, or client-side aggregation.

### Testing Requirements

- Application/domain: test the pure aggregation result with fixtures containing dates before/current/after the UTC month, month/year boundaries, at least two currencies, equal and unequal Payer IDs, archived/renamed identity metadata, Decimal overflow, and invalid stored precision/corruption boundary. Assert exact strings/Decimals and deterministic ordering.
- Snapshot/infra: assert all complete Spendings are available to the source calculation, payer allocations hydrate canonically, archived Participant foreign-key history remains readable, Group scope is enforced, and the result does not depend on SQL row order. Use temporary SQLite/`#[sqlx::test]` for adapter behavior.
- Web/router: assert Summary contains explicit month/UTC context, Group total before Payer rows, symbol plus ISO code, current names and Archived text, empty state without fabricated currencies, stable heading/autofocus and Summary `aria-current`, Add Spending/guidance, safe failure with no partial amounts, and provider-independent behavior.
- Accessibility/native parity: assert stable status node has `role="status"`, `aria-live="polite"`, `aria-atomic="true"`, owning `aria-busy`, and no amount row is a live region. Verify native full-page response and any enhanced fragment use the same semantic content and recovery.
- CSS/template: assert source-summary classes/IDs and semantic structure, 48px controls/focus rules, tabular numerals, wrapping long names, no page-level horizontal overflow rule, and Editorial Contrast tokens. Treat actual 320px/400% geometry/contrast as manual evidence unless a browser harness exists.
- Keep concurrency tests deterministic with barriers/notifications if any read/write snapshot behavior is added; never use timing sleeps. Preserve existing root smoke, architecture, formatting, Clippy, and locked test coverage.

### Previous Story Intelligence

- Stories 3.3-3.5 established the complete aggregate/read seam, current Participant-name projection, visible Archived labels, stable heading/row focus IDs, native full-page authority, scoped status nodes, and safe error mapping. Extend those seams; do not rebuild Transactions or introduce a second snapshot reader.
- Story 3.4 review fixed direct mutation bypasses, token ordering, unknown mutation outcomes, retained archived roles, stable focus, and latest-input-wins behavior. Summary is a read, but it must preserve the same no-raw-error/no-false-success standards.
- Story 3.5 review fixed complete aggregate confirmation, parent-only cascade deletion, allow-listed return state, deterministic concurrency tests, and archived mutation prechecks. Preserve restrictive historical integrity and do not let Summary filtering make archived records disappear.
- Existing story packets explicitly warn that current Summary has no financial projection. This is an additive first consumer, not a reason to retain parallel legacy statistics or provider-coupled scaffolding.

### Git Intelligence

- Recent commits are story-oriented and extend existing seams: `4c0989e feat: implement 3-5 bmad`, `02d53b4 feat: implement 3-4 bmad`, `0fc4380 feat: implement 3-3 bmad`, `120c4ca feat: implement 3-2 bmad`, `7be36b1 feat: implement 3-1 bmad`.
- HEAD is the accepted Story 3.5 implementation. Build on the current supervised runtime, complete snapshot reader, Summary shell, and Transactions projections. Do not overwrite unrelated worktree changes or revive superseded pre-Epic-3 APIs.
- Epic 4 is the next backlog epic; Story 4.1 must establish a reusable exact source projection that Stories 4.2/4.3 can extend without duplicating aggregation or changing source totals when conversion is unavailable.

### Project Structure Notes

- Feature modules are plural (`summaries` if a module is needed); interfaces use `*Reader`/`*UseCases`; implementations use `*Service`/`*Store`; rendering types use `*Template`/`*View`/`*Row`.
- Ledger IDs are positive `i64`; UUIDs remain limited to session/security randomness. No user, tenant, registration, Participant-authentication, or multi-administrator concept may be introduced.
- Existing `spendings` date index and payer/share schema are sufficient. No migration is expected. If SQL changes, update normative design first and regenerate checked metadata.
- Keep `/groups/{id}` as canonical Summary, preserve five destinations in fixed order, and keep all Group/Participant/Spending management and archived read-only paths working.

### UX Guardrails

- `UX-SHELL-01`: retain the five native Group destinations in order, Summary current, stable URL, and persistent Add Spending placement/guidance.
- `UX-TARGET-01`: every link/control/navigation target remains at least 48 by 48 CSS pixels at 320px and 400% zoom; no inline-link exception.
- `UX-FOCUS-01`: native Summary forward response focuses stable Summary heading; enhanced navigation preserves the same focus contract and does not invent arbitrary selectors.
- `UX-STATUS-01`: one scoped polite atomic status and owning `aria-busy` represent pending/empty/unavailable transitions; individual amounts are never live-announced.
- `UX-RESPONSIVE-01`: one-column narrow Summary, governed wide reading measure, safe lower shell, no page horizontal dependency, no DOM/focus reordering.
- `UX-VISUAL-01`: dark Editorial Contrast, warm paper, serif primary totals, rules/whitespace, square geometry, explicit text for Archived/error/empty state, no cards, gradients, transitions, motion, or color-only meaning.

## References

- [Source: `_bmad-output/planning-artifacts/epics.md#Story 4.1: Review Exact Source Currency Monthly Totals`]
- [Source: `_bmad-output/planning-artifacts/epics.md#Epic 4: Understand Current-Month Spending`]
- [Source: `specs/design.md#Accounting And History`]
- [Source: `specs/design.md#Rates And Settlements`]
- [Source: `specs/design.md#Architecture`]
- [Source: `specs/design.md#Operational Limits`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-2 - Layer responsibility ownership`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-3 - Exact monetary truth`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-4 - Group-owned identity and history`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-7 - Snapshot-complete calculation reads`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-11 - Native HTML authority`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-15 - Safe failures, logging, and readiness`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-18 - Governed UX contracts and traceability`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md#Information Architecture`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md#Financial Presentation`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md#Interaction Focus Matrix`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md#Responsive & Platform`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/DESIGN.md#Layout & Spacing`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/DESIGN.md#Components`]
- [Source: `_bmad-output/implementation-artifacts/3-3-browse-and-inspect-spending-history.md#Previous Story Intelligence`]
- [Source: `_bmad-output/implementation-artifacts/3-4-correct-an-existing-spending.md#Previous Story Intelligence`]
- [Source: `_bmad-output/implementation-artifacts/3-5-delete-a-spending-atomically.md#Previous Story Intelligence`]
- [Source: `_bmad-output/project-context.md#Critical Implementation Rules`]
- [Source: `debtor-application/src/debts.rs`]
- [Source: `debtor-application/src/spendings.rs`]
- [Source: `debtor-application/src/errors.rs`]
- [Source: `debtor-infra/src/db/repos/snapshots.rs`]
- [Source: `debtor-web/src/handlers/groups.rs`]
- [Source: `debtor-web/src/handlers/spending_views.rs`]
- [Source: `debtor-web/src/templates.rs`]
- [Source: `debtor-web/templates/group.html`]
- [Source: `debtor-web/src/state.rs`]
- [Source: `src/composition.rs`]
- [Source: `static/css/app.css`]
- [Source: `migrations/20260517000004_create_spendings.up.sql`]
- [Source: Context7 `/tokio-rs/axum`, `/askama-rs/askama`, `/websites/rs_rust_decimal`, consulted 2026-08-18]

## Dev Agent Record

### Agent Model Used

openai/gpt-5.6-luna

### Debug Log References

- Resolved `bmad-create-story` customization: no activation prepend/append or completion terminal steps were configured.
- Loaded `_bmad-output/project-context.md` as the persistent project fact and `_bmad/bmm/config.yaml`; communication and document output language are English.
- Read the complete ordered `_bmad-output/implementation-artifacts/sprint-status.yaml`; selected first backlog story `4-1-review-exact-source-currency-monthly-totals`. Epic 4 was backlog at selection time.
- Loaded the complete Epic 4 story context, PRD, architecture spine, UX `DESIGN.md`/`EXPERIENCE.md`, normative `specs/design.md`, project context, migrations, current Summary/snapshot/application/web/root files, and recent Git history.
- Audited the repository with a parallel exploration pass: Summary currently has no financial projection; the complete snapshot reader and current identity projections are the primary reusable seams; Transactions pagination is not valid for totals.
- Consulted current Axum, Askama, and rust_decimal documentation through Context7 on 2026-08-18. Pinned project versions and lockfiles remain authoritative.

### Implementation Plan

- Added an application-owned `SummaryService` using the existing complete `LedgerSnapshotReader`, `ParticipantReader`, and injected UTC `Clock`.
- Aggregated current-month single-Payer allocations with checked Decimal addition into deterministic Source Currency and Participant-ID buckets; formatted exact currency display values in the application projection.
- Injected the service through root composition and web state, then extended the canonical Group Summary template with typed source blocks, current/archived identity labels, month/UTC context, empty state, unavailable state, stable status, and responsive ruled styling.
- Added application tests for month boundaries, filtering, ordering, archived identity projection, missing identity corruption, overflow, and empty results; added web tests for exact hierarchy/focus, empty state, and sanitized failure.

### Completion Notes List

- Ultimate context engine analysis completed - comprehensive developer guide created.
- Story is intentionally limited to exact current UTC-month Source Currency totals and the first visible Summary financial result.
- Converted Group Currency totals, exchange-rate evidence/fallback, and whole converted-section degradation remain explicitly assigned to Stories 4.2 and 4.3.
- Implemented exact current UTC-month Source Currency totals without provider access or SQL monetary aggregation.
- Preserved archived/renamed Payer history through current Participant projections and deterministic Participant-ID ordering.
- Added safe whole-summary unavailable handling with no partial totals or raw storage diagnostics.
- Full workspace tests, formatting, locked Clippy, architecture fitness, and independent password-helper validation passed.
- Resolved all seven code-review patch findings; SQLx migration and offline metadata verification also passed.

### File List

- `_bmad-output/implementation-artifacts/4-1-review-exact-source-currency-monthly-totals.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `debtor-application/src/lib.rs`
- `debtor-application/src/debts.rs`
- `debtor-application/src/summaries.rs`
- `debtor-infra/src/db/repos/snapshots.rs`
- `debtor-web/src/handlers/groups.rs`
- `debtor-web/src/handlers/spending_views.rs`
- `debtor-web/src/handlers/spendings.rs`
- `debtor-web/src/handlers/test_support.rs`
- `debtor-web/src/router.rs`
- `debtor-web/src/state.rs`
- `debtor-web/src/templates.rs`
- `debtor-web/templates/group.html`
- `src/composition.rs`
- `static/css/app.css`
- No migration or `.sqlx` change was required.

### Change Log

- 2026-08-18: Implemented Story 4.1 exact current UTC-month Source Currency Summary, typed web projection, safe empty/unavailable states, and invariant-owning tests; status moved to review.
- 2026-08-18: Resolved all seven adversarial code-review findings and moved story status to done.
