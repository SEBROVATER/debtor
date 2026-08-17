---
story_key: 3-3-browse-and-inspect-spending-history
story_id: 3.3
epic: 3
status: done
baseline_commit: 120c4ca32dd4d4efbc08d7a40779d4b0576fb0d4
created: 2026-08-17
completion_note: Ultimate context engine analysis completed - comprehensive developer guide created
---

# Story 3.3: Browse and Inspect Spending History

Status: done

## Story

As the administrator,
I want to browse Transactions and open one complete Spending,
so that I can review exact history without loading the entire ledger.

## Acceptance Criteria

1. Given a Group has more than 25 Spendings, when Transactions is opened or paged, each page contains at most 25 rows ordered by `(spent_date DESC, id DESC)` using keyset cursors. No offset pagination, infinite scroll, or SQL monetary aggregation is used.
2. When multiple Spendings share a date, consecutive pages use descending positive Spending ID as the stable tie-breaker with no duplicate or skipped rows. Cursor input is bounded, strictly parsed, canonical, and safely rejected when malformed.
3. Every displayed row decodes and revalidates canonical monetary `TEXT` in Rust, shows Source Currency, and resolves the current Payer name. Corrupt persisted data withholds the affected aggregate behind sanitized failure; it is never normalized, partially rendered, replaced with zero, or exposed through raw diagnostics.
4. The Spending detail route loads one complete aggregate from one SQLite snapshot: Spending fields, owning Group context, Payer, all ordered Shares, and current Participant names/identity state by ID. It must not materialize all Group history.
5. If a referenced Participant was renamed or archived after the Spending committed, history/detail shows the current name while preserving the historical Payer/Share role, Participant ID, and exact stored amount. Archived identities are not filtered out.
6. Archived Groups remain readable through the same accessible responsive native path. Their shell, history, and detail remain available, but settings, mutation, Edit, and Delete controls are absent.
7. A safe history/detail read timeout or persistence failure returns bounded sanitized feedback with no raw SQL, values, identifiers, cursor contents, or partial aggregate. No session or submission-token state changes beyond normal authenticated-session refresh.
8. Each Transactions item is a native `<details>` row. Its 48px-minimum `<summary>` places disclosure plus Description/date on the left and an unbroken Source Currency Total on the right. Expanded content uses the approved definition layout for Description, Total, Source Currency, date, category, Payer, and Shares, followed by equal Edit/Delete actions for mutable active-Group rows. Participant markers supplement, never replace, visible current names.
9. When more than 25 Spendings exist, Previous and Next are equal 48px outlined native links with readable disabled endpoints and page context between/above them at 320 CSS pixels and 400% zoom. There is no clipping, page-level horizontal scroll, or infinite-scroll affordance.
10. A successful native or enhanced page change focuses the stable Transactions heading and renders exactly one scoped polite atomic page-context status. While pending or on expected error, the activated link retains focus, the owning region is `aria-busy`, and current rows remain until the outcome is known.
11. When the Group or referenced Participant is archived, visible “Archived” text is associated with the relevant identity, the five-link Group shell remains readable, and financial facts retain normal contrast.

Requirements: `SPEC-FR42..SPEC-FR43`, `SPEC-FR65..SPEC-FR66`; `SPEC-NFR2`, `SPEC-NFR5`, `SPEC-NFR10`, `SPEC-NFR14..SPEC-NFR16`, `SPEC-NFR25`, `SPEC-NFR28..SPEC-NFR34`; UX contracts `UX-SHELL-01`, `UX-TARGET-01`, `UX-FOCUS-01`, `UX-STATUS-01`, `UX-RESPONSIVE-01`, `UX-VISUAL-01`.

## Scope Boundary

- Implement the vertical Transactions browse/detail experience and its invariant-owning read projections, persistence reads, templates, CSS, and tests.
- Reuse the existing `SpendingReader`, `SpendingUseCases`, authenticated shell, safe GET timeout/admission, session handling, and optional pinned HTMX enhancement. Do not create a second authentication, session, CSRF, submission-token, timeout, or read pipeline.
- Preserve existing Spending creation and its shared mutation path. Do not redesign creation, correction, deletion, monthly summaries, exchange rates, debts, settlements, or Participant archival/restore.
- Edit/Delete links may be rendered for active Groups as navigation targets, but this story does not implement or expand correction/deletion behavior. Archived Groups must suppress both links.
- Prefer no migration. If checked SQL changes, update `specs/design.md` first if behavior/architecture changes, refresh committed `.sqlx`, migrate a temporary SQLite database, and run online SQLx preparation.
- Remove superseded table-row/full-history/detail projections rather than maintaining two divergent Transactions implementations.

## Tasks / Subtasks

- [x] Define transport-neutral read models for history rows and complete detail (AC: 3-5, 8, 11)
  - [x] Keep SQLx/Askama/Axum types out of `debtor-application`; add a dedicated projection only if extending `SpendingSummary` would mix row and detail concerns.
  - [x] Carry current Payer and Share Participant display data, color marker, archived state, role, ID, exact amount, Group name/currency/archive state, and category/date fields needed by web rendering.
  - [x] Preserve deterministic Participant-ID ordering and exact `Decimal` values; templates only format already validated values.
- [x] Extend the infra reader with bounded page and direct detail loading (AC: 1-7)
  - [x] Keep keyset predicates and ordering `(spent_date DESC, id DESC)`; fetch at most 26 rows to determine the next cursor, then return at most 25.
  - [x] Make cursor traversal directionally correct for older/newer pages, including same-date boundaries, and derive cursors from returned rows rather than offsets.
  - [x] Load detail parent, Group context, Payer, Shares, and Participant identity projections in one SQLite transaction/snapshot; do not call a full history reader to construct detail.
  - [x] Validate canonical decimal text, currency/category/date/description, aggregate completeness, participant ownership, and archived identity rows. Convert malformed persisted content to safe invalid-storage failure.
  - [x] Add checked SQLx macros only; preserve `idx_spendings_group_date` unless query evidence proves a required index change.
- [x] Build a dedicated Transactions/detail web projection (AC: 4-8, 11)
  - [x] Make `/groups/{id}/transactions` the canonical Transactions URL and retain `/groups/{group_id}/spendings/{spending_id}` as a direct, group-scoped detail path with the same data and archived/read-only rules.
  - [x] Stop using the current broad `build_group_template` projection to make Transactions and stop rendering history through the existing plain table.
  - [x] Render the existing five-link Group shell, `aria-current="page"`, Group archived text, empty state, history rows, detail disclosure, and safe read errors consistently for native and enhanced requests.
  - [x] Use explicit view booleans for `archived`, `show_actions`, `has_older`, `has_newer`, `focused`, and status/focus behavior. Do not infer mutability or Group emptiness from a bounded page.
  - [x] Render current names and visible “Archived” labels for archived Participants without filtering historical allocations.
- [x] Implement strict cursor and pagination behavior (AC: 1, 2, 9, 10)
  - [x] Preserve the existing `older|newer:YYYY-MM-DD:positive-id` direction/anchor semantics unless repository evidence requires a cleaner canonical format.
  - [x] Add a small maximum cursor length, strict segment count, exact ISO date, supported minimum date, positive bounded ID, and no trailing/unknown query fields. Reject malformed input with sanitized `400` behavior and no raw cursor echo.
  - [x] Use native Previous/Next links with disabled readable endpoint state and page context; do not add offset counts solely to imitate mockup page numbers.
  - [x] If HTMX enhances links, keep the same `href`, full-page response, URL semantics, and server-owned focus target. Swap only the intended Transactions region/status; use the pinned official `response-targets` extension where expected errors need routing.
  - [x] Preserve current rows and invoker focus during pending/error states; successful forward pagination renders exactly one `autofocus`/stable heading target and one scoped polite atomic status.
- [x] Replace the history/detail markup and CSS (AC: 8-11)
  - [x] Replace plain `<table>` history rows with semantic native `<details>`/`<summary>` rows. Use a two-column summary and an expanded two-column definition list; keep monetary total and ISO date readable at narrow widths.
  - [x] Keep summary, Edit, Delete, Previous, and Next controls at least 48px; make Edit/Delete equal columns and hide them on archived Groups.
  - [x] Apply Editorial Contrast: dark charcoal canvas, warm paper text, rules/whitespace, square geometry, no cards, gradients, hover lift, authored transitions, or decorative shadows.
  - [x] Ensure long descriptions/names wrap, amounts remain unbroken, and Transactions never creates page-level horizontal scrolling. Only a specifically labeled allocation region may scroll, and this story's detail should not need horizontal scrolling.
  - [x] Preserve existing sign-out and Group shell behavior while adding stable IDs for Transactions heading, owning region, page status, rows, summaries, and detail facts.
- [x] Add invariant-owning tests and run validation (AC: all)
  - [x] Application tests use fakes to verify page/detail forwarding, Group scoping, deterministic projections, and safe error propagation without Axum, SQLite, network, or wall clock.
  - [x] Infra tests cover 25-row limits, same-date multi-page traversal with no duplicates/skips, malformed/corrupt canonical data, current renamed Payer/Share names, archived identity retention, complete ordered direct detail, Group scoping, and unrelated malformed history not breaking a targeted detail.
  - [x] Web/router tests cover authenticated native Transactions rendering, native detail parity, empty state, archived shell/action suppression, visible archived labels, exact amounts/currency, malformed/oversized cursors, focus/status/`aria-busy`, no token reservation on GET, sanitized timeout/storage failures, and anonymous redirect/security headers.
  - [x] Add template/CSS assertions for semantic `<details>`, definition facts, stable IDs, 48px classes/attributes, readable disabled pagination, and no legacy table path. Do not claim automated geometry testing unless an executable browser harness exists.
  - [x] Run `cargo fmt --all -- --check`, `cargo check --workspace --all-features --locked`, `SQLX_OFFLINE=true cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`, `cargo test --workspace --all-features --locked`, and `cargo run --bin architecture-check --locked`.

### Review Findings

- [x] [Review][Patch] Legacy Summary still renders a divergent table-based Spending history [debtor-web/templates/group.html:171-215; debtor-web/src/handlers/groups.rs:100-115] — replaced the Summary table with a canonical Transactions link and updated the route regression test.
- [x] [Review][Patch] Missing or cross-Group Participant references are misclassified as NotFound [debtor-infra/src/db/repos/spendings.rs:36-62, 109-110] — missing identity rows now map to invalid persisted storage.
- [x] [Review][Patch] Missing Payer references disappear from history [debtor-infra/src/db/repos/spendings.rs:397-400] — history uses left joins and validates absent payer data instead of silently omitting the Spending.
- [x] [Review][Patch] Unknown query parameters are silently accepted [debtor-web/src/handlers.rs:40-43] — `SpendingQuery` now denies unknown fields and has route coverage.
- [x] [Review][Patch] Transaction amounts omit the required currency symbol [debtor-web/templates/transactions.html:46,51-56; debtor-web/templates/spending_detail.html:29-34] — added currency symbols to domain currency display projections and all affected facts.
- [x] [Review][Patch] Group archive state is read outside the history snapshot [debtor-web/src/handlers/spending_views.rs:34-40] — the history page now carries Group state from the same persistence snapshot as its rows.
- [x] [Review][Patch] Acceptance-critical Transactions behavior lacks route-level verification [debtor-web/src/handlers/test_support.rs:443-455; debtor-web/src/router.rs:1282-1298] — added native Transactions rendering and unknown-query route tests.
- [x] [Review][Patch] Empty paginated pages announce and display the wrong state [debtor-web/src/handlers/spending_views.rs:66-71; debtor-web/templates/transactions.html:36-40] — the visible empty copy now uses the page-specific status.
- [x] [Review][Patch] `autofocus` on the Transactions heading is not a reliable heading focus mechanism [debtor-web/templates/transactions.html:31] — the stable heading now contains a native focusable anchor target for successful pagination.
- [x] [Review][Patch] Transaction disclosure summaries have no explicit focus-visible indicator [debtor-web/templates/transactions.html:43-47; static/css/app.css:80] — added a two-pixel high-contrast `:focus-visible` outline for summaries.

## Dev Notes

### Developer Context

Epic 3 already delivered exact Spending creation and the shared aggregate persistence path. This story is the first historical read consumer and must prove that stored ledger history remains exact and readable after Participant rename/archive without loading the entire ledger. The current implementation is a brownfield scaffold: `group.html` renders a plain table, `build_group_template` loads the Group, all members, and a bounded page for several sections, `SpendingSummary` contains only parent fields, `load_spending` returns only the domain `Spending`, and `spending_detail.html` separately fetches context and renders a divergent standalone card/list page. Replace those superseded read projections with one coherent Transactions/detail contract.

Do not trust existing infrastructure merely because it has a keyset query. The current page query only decodes parent summaries; it does not resolve current Payer names or validate the complete aggregate. The current detail path performs separate Group/Participant loads after `load_spending`, which does not satisfy the one-snapshot complete-context requirement. Read the current files before editing and preserve unrelated Manage, Summary, Add Spending, sign-out, and mutation behavior.

### Current Files To Update And Preserve

| Path | Current state | Required change / preservation |
|---|---|---|
| `debtor-application/src/spendings.rs` | `SpendingCursor`, `SpendingPage`, `SpendingSummary`, `SpendingReader`, and forwarding use cases already exist; summary has only parent fields. | Extend or add read models for current identity/detail context without leaking adapters. Keep `SpendingReader` as the narrow application port and preserve cursor semantics. |
| `debtor-infra/src/db/repos/spendings.rs` | `load_spending` reads parent/payer/share rows in a transaction; `spending_page` uses keyset `LIMIT 26` but only parent summaries. | Reuse checked query and transaction patterns. Add complete direct read/context and current-name resolution; validate all affected canonical data; preserve no SQL monetary aggregation. |
| `debtor-infra/src/db/repos/decoding.rs` | `DbSpendingSummary` and `spending_summary` decode parent row fields and canonical amounts. | Add only the decoding needed for new read models; map malformed persisted data to safe storage failure and never normalize. |
| `debtor-web/src/handlers/groups.rs` | `group_transactions` calls broad `build_group_template`; `group_detail` also accepts cursor. | Route Transactions through a dedicated projection; retain auth, safe-read pipeline, canonical URL, and group scoping. Decide/remove cursor acceptance from Summary if it conflicts with the five-section contract. |
| `debtor-web/src/handlers/spendings.rs` | Direct detail separately loads Group, Spending, and all members; cursor parser has direction/date/ID checks but no length/minimum-date bound. | Use complete detail projection, preserve `404` scoping and safe error mapping, and strengthen cursor admission without logging/echoing input. Do not implement edit/delete redesign. |
| `debtor-web/src/handlers/spending_views.rs` | `build_group_template` always loads members/history and maps parent rows; `named_allocations` has only name/amount. | Add dedicated history/detail builders and explicit identity/action flags. Do not make templates calculate financial data or infer mutability from empty pages. |
| `debtor-web/src/templates.rs` | `GroupTemplate` owns history fields; `SpendingRow` has parent summary only; `SpendingDetailTemplate` has basic allocation rows. | Add focused `TransactionsTemplate`/detail view types or minimally extend them. Keep render projections typed and stable; do not leave divergent old/new detail paths. |
| `debtor-web/templates/group.html` | Plain table rows, separate links, “Newest/Newer/Older”, broad Group shell, and legacy card classes. | Replace only history/Transactions markup with native details, definition facts, paired actions, page context, stable status/region IDs, and archived suppression. Preserve shell, Manage, Summary, Add Spending, sign-out, and native fallback. |
| `debtor-web/templates/spending_detail.html` | Standalone Back/detail page with unordered Payer/Share lists and mutation links. | Make direct detail use the same shell-consistent, current-name, archived-readable projection as inline detail, or remove its divergent visual role if route composition makes inline detail canonical. |
| `static/css/app.css` | Legacy cards, white table, and no transaction disclosure/pagination styles; existing form CSS must remain usable. | Add ruled Editorial Contrast transaction/detail/pagination styles, responsive wrapping, focus/status styles, and 48px targets. Remove only superseded history/table rules; do not regress Spending form geometry. |
| `debtor-infra/tests/repos.rs` | Basic page traversal and direct-load tests exist. | Extend with boundary, corruption, identity, archive, complete-detail, and snapshot/scoping assertions. |
| `debtor-web/src/router.rs` and relevant handler tests | Authenticated route composition and broad history tests already exist. | Extend route-level native/enhanced, safe-error, focus/status, archived, and no-token-read coverage without introducing a second pipeline. |
| `specs/design.md` | Normative product/architecture source of truth. | Do not change for implementation-only behavior. If implementation reveals a genuine contract change, update it before code and synchronize affected artifacts. |

### Architecture Compliance

- Preserve `root -> debtor-web/debtor-infra -> debtor-application -> debtor-domain`. Domain remains pure; application owns read ports/projections and safe reason categories; infra owns SQLx/snapshot/canonical persistence reads; web owns HTTP/query parsing, rendering, accessibility, and safe mapping; root composition remains unchanged unless a compile-time seam requires it.
- Apply AD-3: exact `rust_decimal::Decimal`, canonical SQLite `TEXT`, checked parsing/hydration, no float/SQL arithmetic/normalization/zero substitution. A read must validate enough of the complete aggregate to avoid rendering a false financial record.
- Apply AD-4 and AD-7: preserve Group-owned identity history, resolve current Participant names, retain archived referenced identities, load direct detail from one snapshot, and reserve full-history snapshots only for calculations that require them.
- Apply AD-11: Askama semantic HTML and native links are authoritative. HTMX `2.0.10` and response-targets `2.0.4` are optional enhancement only; no custom JavaScript, custom extension, inline scripts, or client-only disclosure/pagination authority.
- Apply AD-14/AD-15: safe dynamic reads use the existing 30-second bounded timeout; map missing data to `404`, malformed cursor to safe `400`, invalid persisted data/unexpected storage to sanitized `500`, contention/availability to bounded retryable feedback. Never expose raw SQL, database messages, IDs, values, query strings, cursor content, sessions, tokens, or adapter diagnostics.
- Apply AD-18: cite and test all applicable UX contracts, including shell parity, targets, focus, status/`aria-busy`, responsive geometry, and Editorial Contrast. Mockups are illustrative; `DESIGN.md`, `EXPERIENCE.md`, and `specs/design.md` are authoritative.

### Library / Framework Requirements

- Keep pinned versions and lockfiles: Rust `1.97.1` edition 2024, Axum `0.8.9`, Askama `0.16.0`, Tokio `1.53.1`, SQLx/sqlx-cli `0.9.0`, `rust_decimal 1.42.1`, HTMX `2.0.10`, response-targets `2.0.4`. Do not add a pagination, template, form-state, or JavaScript dependency.
- Current Axum documentation confirms `Path`/`Query` extraction and route handlers for typed path/query values. Keep cursor parsing in the web boundary and convert it into the application-owned `SpendingCursor`; do not make application ports depend on Axum extractors.
- Current Askama documentation confirms compile-time typed `#[derive(Template)]` structs with external template paths. Keep all display flags and already validated strings in render structs; do not put financial policy or SQL queries in templates.
- Current HTMX documentation confirms `hx-get`/boosted links can progressively enhance navigation while native `href` remains the no-JavaScript path. Preserve the full-page response and URL semantics; do not rely on HTMX history snapshots or custom event JavaScript for focus/status correctness.
- Consult current crate documentation again before changing framework APIs. SQL changes require checked macros, committed offline metadata, temporary migration, and online `cargo sqlx prepare --workspace --check`.

### Testing Requirements

- Domain changes are not expected. If any are introduced, test only pure exactness/determinism there and do not duplicate web/persistence tests.
- Application tests use simple fakes and assert Group-scoped page/detail forwarding, current identity data, deterministic ordering, and safe error propagation without Axum, SQLite, network, or wall clock.
- Infra tests use `#[sqlx::test]`/temporary databases for keyset boundaries, 25-row limit, same-date IDs, direct snapshot context, current names after rename, archived identity retention, canonical corruption, and Group isolation. Use barriers/notifications for concurrency only; do not use sleeps.
- Web tests assert native/enhanced parity, authenticated access, safe cursor rejection, exact current-name/archived rendering, no partial detail, no submission-token reservation on safe GET, stable heading/status IDs, one polite atomic status, `aria-busy`, retained rows/invoker on pending/error, archived action suppression, and safe headers/errors.
- Template/CSS tests can assert markup and classes, but do not claim 320px/400% geometry or contrast verification without a real browser harness. Manual evidence must cover long descriptions/names, unbroken maximum OMR totals, 48px controls, no page-level horizontal scroll, and Editorial Contrast.
- Required validation commands are listed in Tasks. Never use `cargo build --release`.

### Previous Story Intelligence

Story 3.2 (commit `120c4ca`) completed Exact creation and resolved seven review findings. Preserve these patterns:

- Reuse the existing aggregate/repository seams; do not add parallel Spending readers, mutation paths, financial algorithms, write gates, epochs, review stores, or lifecycle wiring.
- Application owns raw input/financial policy; infra owns canonical persistence and transactionality; web owns strict transport decoding/rendering/safe mapping; handlers remain thin.
- Review fixes specifically protected strict field handling, stable focus/status associations, checked arithmetic, exact binding, no raw diagnostics, and no false completion. History must meet the same standard for cursor and read failures.
- Native HTML is authoritative. Any enhanced response must be the same contract as the full-page response and must not make browser focus, URL, or data correctness depend on JavaScript.
- Current names are projections, not creation-time snapshots. Do not persist/render stale Participant names.
- Existing tests and routes are valuable seams, but story prose never overrides current repository reality; inspect each file before editing.

### Git Intelligence

- Recent commits are story-oriented and extend existing layers rather than introducing broad dependencies: `120c4ca feat: implement 3-2 bmad`, `7be36b1 feat: implement 3-1 bmad`, followed by Stories 2.5, 2.4, and 2.3.
- Story 3.2 touched application input seams and web form/projection/templates/CSS but intentionally left Spending persistence and migration structure shared. Story 3.3 should similarly make focused read-model/web changes and only alter infra SQL where the complete read contract requires it.
- The worktree was clean during analysis. The developer must inspect current state again and must not overwrite unrelated concurrent changes.

### UX Guardrails

- `UX-SHELL-01`: Transactions is one of five persistent Group destinations in the approved order; active and archived Groups keep the shell, and Transactions is `aria-current="page"`.
- `UX-TARGET-01`: `<summary>`, row actions, pagination, navigation, and every other control are at least 48 by 48 CSS pixels at 320px/400% zoom.
- `UX-FOCUS-01`: Successful pagination focuses the stable Transactions heading; pending/error retains the activated link; only one server-owned forward focus target is rendered.
- `UX-STATUS-01`: One scoped stable `role="status"`, `aria-live="polite"`, `aria-atomic="true"` announces page context/request outcomes; owning Transactions region exposes `aria-busy`; do not make every amount a live region.
- `UX-RESPONSIVE-01`: Long names/descriptions wrap, totals stay unbroken, disabled endpoints remain readable, and there is no page-level horizontal scroll at 320px/400% zoom.
- `UX-VISUAL-01`: Use dark Editorial Contrast with rules, whitespace, square controls, normal readable financial contrast, visible archived text, no card-heavy decoration, gradients, transitions, hover lift, or color-only status.
- Details must show Description, Total, Source Currency, date, category, Payer, and Shares. Participant marker is supplemental. Amounts include symbol and ISO code in visual/accessible text, and dates remain ISO `YYYY-MM-DD` outside forms.

### Project Structure Notes

- Feature modules remain plural (`spendings`); interfaces use `*Reader`/`*Repository`, adapters use `*Store`, and rendering types use `*Template`/`*Row`/`*View`.
- Ledger IDs remain positive `i64`; UUIDs remain limited to session/security state.
- No schema change is expected. Existing `spending_payers` and `spending_shares` restrictive foreign keys and `idx_spendings_group_date` should support this story.
- A dedicated `TransactionsTemplate`/`transactions.html` is architecturally cleaner than expanding `GroupTemplate` indefinitely, but either choice is valid only if Summary/Manage and direct detail do not retain divergent history projections. No new file is mandatory.
- Do not implement page-number counts by switching to offsets. If the UX needs context, use bounded server-owned cursor context such as “Showing newest”, “Older Spendings”, or an established count query that does not alter keyset ordering.

### References

- [Source: `_bmad-output/planning-artifacts/epics.md#Story 3.3: Browse and Inspect Spending History`]
- [Source: `_bmad-output/planning-artifacts/epics.md#Epic 3: Record and Maintain Exact Spendings`]
- [Source: `_bmad-output/planning-artifacts/epics.md#Cross-Cutting Story Rule`]
- [Source: `specs/design.md#Architecture`]
- [Source: `specs/design.md#Accounting And History`]
- [Source: `specs/design.md#Security`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-2 - Layer responsibility ownership`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-3 - Exact monetary truth`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-4 - Group-owned identity and history`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-7 - Snapshot-complete calculation reads`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-11 - Native HTML authority`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-15 - Safe failures, logging, and readiness`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-18 - Governed UX contracts and traceability`]
- [Source: `_bmad-output/planning-artifacts/prds/prd-debtor-2026-08-10/prd.md#FR-6: Review and maintain history`]
- [Source: `_bmad-output/planning-artifacts/prds/prd-debtor-2026-08-10/addendum.md#Snapshot, Pagination, And Direct Loading`]
- [Source: `_bmad-output/planning-artifacts/prds/prd-debtor-2026-08-10/addendum.md#Safe Failures And Diagnostic Allowlists`]
- [Source: `_bmad-output/planning-artifacts/prds/prd-debtor-2026-08-10/addendum.md#HTTP Forms, Statuses, And Dispatch`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md#Information Architecture`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md#Spending and History`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md#Interaction Focus Matrix`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md#Spending`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/DESIGN.md#Layout & Spacing`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/DESIGN.md#Components`]
- [Source: `_bmad-output/implementation-artifacts/3-2-record-a-spending-with-exact-shares.md#Previous Story Intelligence`]
- [Source: `_bmad-output/project-context.md#Critical Implementation Rules`]
- [Source: `debtor-application/src/spendings.rs`]
- [Source: `debtor-infra/src/db/repos/spendings.rs`]
- [Source: `debtor-web/src/handlers/groups.rs`]
- [Source: `debtor-web/src/handlers/spendings.rs`]
- [Source: `debtor-web/src/handlers/spending_views.rs`]
- [Source: `debtor-web/src/templates.rs`]
- [Source: `debtor-web/templates/group.html`]
- [Source: `debtor-web/templates/spending_detail.html`]
- [Source: `static/css/app.css`]
- [Source: Axum 0.8 routing/extractor documentation via Context7 `/tokio-rs/axum`]
- [Source: Askama typed template documentation via Context7 `/askama-rs/askama`]
- [Source: HTMX progressive enhancement documentation via Context7 `/bigskysoftware/htmx`]

## Dev Agent Record

### Agent Model Used

openai/gpt-5.6-luna

### Debug Log References

- Resolved create-story customization: no activation prepend/append steps; loaded `_bmad-output/project-context.md`.
- Read the complete ordered `_bmad-output/implementation-artifacts/sprint-status.yaml`; selected first backlog story `3-3-browse-and-inspect-spending-history`.
- Epic 3 was already `in-progress`; no epic transition was required.
- Loaded Epic 3 context, Story 3.3 contract, normative design, PRD/addendum, architecture spine, UX experience/design, previous Story 3.2, current read paths, and recent git history.
- Consulted current Axum, Askama, and HTMX documentation through Context7 on 2026-08-17. Pinned versions and lockfiles remain authoritative.

### Implementation Plan

- Added application-owned history/detail read projections while preserving existing Spending mutation and summary paths.
- Added checked SQLite queries that load complete detail and bounded history identity/share data from one transaction, with canonical decoding and aggregate validation.
- Added a dedicated native Transactions template and direct-detail projection with current Participant names, archived labels, disclosure rows, pagination/status targets, and responsive Editorial Contrast styling.
- Added cursor-boundary and renamed/archived identity regression coverage; refreshed committed SQLx metadata.

### Completion Notes List

- Ultimate context engine analysis completed - comprehensive developer guide created.
- No clarification is required before implementation; the story resolves the keyset/detail/archived/native-path boundaries against the current repository and normative artifacts.
- Full implementation completed and validated; story and sprint status are now `done`.
- Validation passed: `cargo fmt --all -- --check`; offline locked workspace check; offline locked Clippy with warnings denied; full locked workspace tests; architecture fitness; and online SQLx prepare/check against a migrated temporary SQLite database.
- Code review resolved all 10 patch findings; story and sprint status are now `done`.

### File List

- `_bmad-output/implementation-artifacts/3-3-browse-and-inspect-spending-history.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `.sqlx/query-2c45d16288085f7d15a1981e4287dc27ea5ec64929a8f3d252da8bbc85de7cdb.json`
- `.sqlx/query-090a387f014deaf17a0ee628e0e1e7915668600ff872859d99aa948443aa671e.json`
- `.sqlx/query-1eedca64c877bc1062cb364844af459dd4e6ba0e619489ffd329ed94f30dff47.json`
- `.sqlx/query-ad632a35506f6c718dfe64935e6ee54c13f8f2ab9ef3021eaa37e4002bc168c5.json`
- `debtor-application/src/spendings.rs`
- `debtor-domain/src/currency.rs`
- `debtor-infra/src/db/repos/decoding.rs`
- `debtor-infra/src/db/repos/spendings.rs`
- `debtor-infra/tests/repos.rs`
- `debtor-web/src/handlers/groups.rs`
- `debtor-web/src/handlers/spending_views.rs`
- `debtor-web/src/handlers/spendings.rs`
- `debtor-web/src/handlers/test_support.rs`
- `debtor-web/src/handlers.rs`
- `debtor-web/src/router.rs`
- `debtor-web/src/templates.rs`
- `debtor-web/templates/spending_detail.html`
- `debtor-web/templates/group.html`
- `debtor-web/templates/transactions.html`
- `static/css/app.css`

### Change Log

- 2026-08-17: Implemented bounded keyset Transactions history, complete one-snapshot Spending detail, current/archived Participant identity projections, semantic disclosure rows, pagination/status/focus markup, responsive styling, cursor bounds, regression tests, and SQLx metadata; status moved to review.
- 2026-08-17: Resolved all 10 adversarial review findings covering canonical history routing, corruption classification, strict query parsing, currency symbols, snapshot Group state, route coverage, empty-page messaging, focus targeting, and disclosure focus styling; status moved to done.
