# Final Closure Reconciliation

## Scope And Sources

This report performs the final closure reconciliation of the current artifacts:

- Normative product and architecture contract: `specs/design.md`
- Capability-focused requirements: `prd.md`
- Technical and downstream requirements: `addendum.md`
- Supplied implementation context: `_bmad-output/project-context.md`
- User decisions and dispositions: `.memlog.md`

The reconciliation extracts omissions, distortions, contradictions, and qualitative intent loss. It does not rewrite the PRD. Product capabilities and visible behavior are expected primarily in `prd.md`; technical mechanisms may live in `addendum.md` when their exact normative values and semantics remain preserved.

Per the final reconciliation instructions, the following are treated as supplied decisions rather than unsupported additions:

- Personal-spending positioning is user-supplied (`.memlog.md:13`, `.memlog.md:38`).
- Compact progressive disclosure or overlay for Spending entry is user-supplied (`.memlog.md:23`).
- Archived-Participant role retention policy is supplied by project context (`project-context.md:96`) and is now also normative in the design contract (`design.md:54`).

## Verdict

**Closed - fully reconciled.** No critical, high, medium, or low findings remain. The PRD and addendum preserve the supplied product intent, capability boundaries, user-visible validation behavior, historical-integrity rules, and exact technical contract without a remaining omission, distortion, or contradiction.

## Residual Disposition Verification

### Group-Name Validation - Resolved

The source requires Group and Participant names to be trimmed, non-empty, and at most 100 Unicode characters (`design.md:50`). FR-2 now states the complete Group-name boundary (`prd.md:77`), FR-3 states the matching Participant-name boundary (`prd.md:89`), and the addendum preserves both (`addendum.md:11`).

### Archived-Participant Role Policy - Resolved And Sourced

The design now explicitly permits a Spending update to retain an archived Participant only in the same existing Payer or Share role and prohibits introducing or changing that archived Participant's role (`design.md:54`). The policy is also supplied in project context (`project-context.md:96`) and preserved consistently in FR-5 (`prd.md:116`) and transactional persistence requirements (`addendum.md:32`). It is no longer an inferred or unsupported requirement.

### One Group-Page Spending Form - Resolved

The source requires one expense form on the Group page with independent Payer and Share choices (`design.md:59`). The PRD now states that the selected Group keeps one Spending form on the Group page and that progressive disclosure does not create a separate management surface (`prd.md:94`). Independent modes remain explicit (`prd.md:105-106`).

### Personal-Spending Positioning - Accepted User Decision

The PRD positions Groups for personal and shared Spendings (`prd.md:18`). The user explicitly supplied personal-spending tracking as a job (`.memlog.md:13`) and directed retention of that positioning during residual closure (`.memlog.md:38`). It is therefore valid qualitative product intent, not scope drift.

### Progressive Disclosure - Accepted User Decision

Compact disclosure or overlay is stated as an experience principle (`prd.md:35`) and concretized as one form on the Group page rather than a separate management surface (`prd.md:94`). The interaction was explicitly selected by the user (`.memlog.md:23`) and remains constrained by server rendering and no custom JavaScript (`prd.md:37`, `prd.md:248`; `addendum.md:60`). It is neither unsupported nor contradictory.

### Current-Name Historical Resolution - Resolved

Historical detail must resolve current Participant names (`design.md:62`). FR-6 now states that details remain readable through archival and show each Participant's current name after rename (`prd.md:124`). The direct-load contract preserves the same behavior (`addendum.md:41`).

## Core Product Reconciliation

### Goal, User Model, And Scope - Preserved

- Debtor remains a private password-gated ledger for exactly one Administrator (`design.md:5`, `design.md:21`; `prd.md:10-18`, `prd.md:55-66`).
- Participants remain Group-owned accounting identities, not users, and are never reused across Groups (`design.md:13`, `design.md:21`, `design.md:54`; `prd.md:41-48`, `prd.md:81-90`).
- The release includes Groups, Participant lifecycle, Spending CRUD, multiple Payers, equal/exact Shares, current-month summaries, and advisory Settlements (`design.md:13`; `prd.md:53-181`, `prd.md:195-204`).
- The twelve currencies and eight category codes/current labels match exactly (`design.md:15`; `prd.md:101-102`).
- Non-goals preserve no multi-user collaboration, repayment state, settlement ranges, unsupported split modes, global transfer minimization, persistent sessions, manual refresh, custom JavaScript, multiple instances, or external writers (`design.md:17`, `design.md:41`; `prd.md:183-207`).

### Group And Participant Lifecycle - Preserved

- Groups support create, edit, archive, and restore; history-free Groups may be deleted with unreferenced Participants, while Groups with Spendings are archive-only (`design.md:55`; `prd.md:72-79`; `addendum.md:32`).
- Participant add/edit/archive/restore remains Group-local with no global management surface (`design.md:54-59`; `prd.md:81-90`).
- Archived Groups are readable and mutation-disabled, with direct archived form/mutation requests rejected before use-case invocation (`design.md:55`, `design.md:63`; `prd.md:78-79`; `addendum.md:63`).
- Archived identities remain visible in history, Balances, and Transfers, and historical detail resolves current names (`design.md:54`, `design.md:62`; `prd.md:88`, `prd.md:124`; `addendum.md:41`).

### Spending And Validation - Preserved

- A Spending includes description, date, category, positive Total, Source Currency, one or more Payers, and equal or exact Shares (`design.md:13`, `design.md:49-53`; `prd.md:96-116`).
- Group and Participant names have the exact 100-Unicode-character trimmed/non-empty boundary; descriptions have the exact 200-character boundary; colors normalize to `#RRGGBB`; dates are strict `YYYY-MM-DD` on or after `2025-01-01` (`design.md:50`; `prd.md:77`, `prd.md:89-90`, `prd.md:103-104`; `addendum.md:11`).
- Totals and persisted Payer/Share amounts are positive, precision-valid, and at most `999_999_999_999`; JPY/KRW use zero minor units, OMR three, and all other supported currencies two (`design.md:49-50`; `prd.md:111-115`; `addendum.md:15-17`).
- Payer and Share totals each conserve the Spending Total exactly; allocations are nonempty and Participant-unique; equal-split residuals use ascending Participant ID (`design.md:51-53`; `prd.md:109-116`; `addendum.md:17`).
- Validation errors preserve submitted values and render inline with `422`; successful mutations redirect with `303` (`design.md:61-63`; `prd.md:107`, `prd.md:252`; `addendum.md:62`).

### Source Currency - Preserved And Editable

- Each Spending retains its currently stored Source Currency for conversion and historical interpretation (`design.md:70`; `prd.md:47`, `prd.md:259`).
- Spending edit may correct Source Currency under creation validation, after which calculations use the corrected stored value (`prd.md:125`; `.memlog.md:35`).
- Group Currency remains freely changeable as the converted-summary and settlement target (`design.md:70`; `prd.md:48`, `prd.md:259`).

### History And Summaries - Preserved

- Ordinary history uses fixed 25-item keyset pages ordered by `(spent_date DESC, id DESC)`, while detail/edit/delete directly load one complete aggregate (`design.md:59`; `prd.md:123`; `addendum.md:37-41`).
- The Group page shows the current UTC month's Group Total and per-Payer totals by original Source Currency and in converted Group Currency using each Spending date's historical rate (`design.md:60`; `prd.md:129-148`).
- Source Currency totals and ordinary ledger operations survive rate failure; only converted summary content becomes retryably unavailable (`design.md:60`, `design.md:77`; `prd.md:137-148`).
- All-time Balances remain separate from the current-month summary (`design.md:60`; `prd.md:150-181`).

### Rates, Balances, And Settlements - Preserved

- Historical mode is default; current mode uses the UTC calculation date and is not persisted; future historical dates use current rates and are provisional (`design.md:72-74`; `prd.md:145-156`).
- Balances quantize with largest remainder and Participant-ID tie-breaking while preserving exact zero sum (`design.md:78`; `prd.md:158-164`; `addendum.md:25`).
- Settlement is deterministic greedy, ordered by descending absolute balance then Participant ID, positive, complete, pair-unique, bounded by `n - 1`, and not claimed globally minimal (`design.md:79`; `prd.md:166-173`; `addendum.md:26`).
- The debts view discloses mode, calculation time, Group Currency, unique rates, and stale/provisional warnings, with no partial result on failure (`design.md:76-80`; `prd.md:158-181`).

### Experience And Accessibility - Preserved

- The information architecture is Group-centered with one mobile-friendly web experience (`prd.md:32-37`; `.memlog.md:21`, `.memlog.md:23`).
- Core behavior is semantic server-rendered HTML with vanilla CSS and no custom JavaScript; optional HTMX cannot be required (`design.md:17`; `prd.md:37`, `prd.md:248`; `addendum.md:60`).
- Stable Chrome, Firefox, Safari, and Edge remain supported down to 320 CSS pixels, with semantic structure, keyboard operation, visible focus, labels, contrast, and clear errors (`design.md:64`; `prd.md:246-253`).
- Minimal, modern, low-animation qualitative intent and deliberately narrow self-operated positioning remain intact (`prd.md:10-12`, `prd.md:32-37`; `.memlog.md:10`, `.memlog.md:21`).

## Exact Technical Contract Reconciliation

### Architecture And Ownership - Preserved

The inward dependency direction, crate responsibilities, application-owned input policy and ports, transport limitations, injected effects and clock, handler thinness, structured safe failures, and no outer-type leakage are all retained (`design.md:25-44`; `addendum.md:5-11`, `addendum.md:43-48`).

### Money And Persistence - Preserved

Exact Decimal use, canonical SQLite `TEXT`, Rust-only monetary parsing/formatting/aggregation, no floating point or SQL monetary aggregation, checked failures, WAL, `synchronous=FULL`, foreign keys, five-second busy/write-gate limits, pre-transaction gate timeout, transactional eligibility checks, latest-commit semantics, no optimistic revisions, enumerated structural checks, and compile-time SQLx metadata are retained (`design.md:42`, `design.md:48-58`, `design.md:65-66`; `addendum.md:13-19`, `addendum.md:28-35`).

### Snapshots And Loading - Preserved

Complete Spending aggregates load from one snapshot; debt calculation materializes Group Currency and all complete Spendings before releasing the transaction; no provider request holds a database transaction (`design.md:57`; `addendum.md:39`). Pagination and direct-load constraints are retained exactly (`design.md:59`; `addendum.md:40-41`).

### Safe Failures And Logs - Preserved

Application-facing safe categories, fixed sanitized calculation failure, no panic/zero substitution/partial result, and strict SQLite operation/category allowlists and diagnostic exclusions are retained (`design.md:38-40`, `design.md:113`; `addendum.md:43-48`). Secret and identity logging exclusions remain at least as strict as the source (`design.md:86`, `design.md:113`; `addendum.md:48`).

### Rate Caches And Provider Limits - Preserved

Lexical arbitrary-precision decoding; complete rate-context keys; context-matching-only stale fallback; separate 4,096-entry deterministic-LRU stable and refreshable caches; UTC rollover; five-second connect, 20-second total, and 64 KiB response limits; global concurrency four; per-key single-flight; and per-calculation deduplication/concurrency four are all exact (`design.md:75-77`, `design.md:114`; `addendum.md:50-56`).

### HTTP And Dispatch - Preserved

The shared strict form/CSRF extractor, pre-route rejection, exact `422`/`409`/`303` outcomes, retained raw values, 30-second mutation pre-dispatch deadline, dispatch boundary, no generic post-dispatch cancellation, and definitive commit/rollback response are retained (`design.md:62-63`, `design.md:88`, `design.md:111`; `addendum.md:58-65`).

### Authentication, Sessions, And CSRF - Preserved

Startup password-hash validation before database work; bounded Argon2id v19; process-local server-side sessions; exact anonymous/authenticated inactivity periods and capacities; authenticated no-eviction; rotation, save, flush, promotion-capacity behavior, indexed five-minute cleanup; exactly one CSRF token; exact rejection cases; and exact five-attempt/five-minute/4,096-key fail-closed limiter behavior are retained (`design.md:84-88`; `addendum.md:67-75`).

### Headers And Proxy Trust - Preserved

The exact no-store, nosniff, no-referrer, and no-script CSP headers; session-free probes/static routes; configured trusted CIDRs and one forwarding format; forwarding-chain sanitation; and protocol-independent client identity are retained (`design.md:88-90`, `design.md:95`; `addendum.md:77-81`).

### Admission, Probes, And Shutdown - Preserved

The 8 KiB/256 KiB body limits; 64 user, four login, and independent four probe permits; 30-second read/login, 90-second debt, two-second probe, and one-second SQLite readiness timeouts; five-second storage waits; liveness/readiness semantics; ten-second shutdown drain; bounded WAL checkpoint; and WAL-sidecar recovery behavior are retained (`design.md:108-114`; `addendum.md:83-90`).

### Edge Contract - Preserved

Edge ownership of TLS, certificates, HTTP/3/QUIC, `Alt-Svc`, and fallback; private HTTP/1.1 backend reuse; forwarding sanitation; disabled or `425` unsafe early data; GET/HEAD-only marked early data; mutation-safe timeout relationships; exact edge body limits; staged `Alt-Svc` rollout; UDP fallback; and cross-protocol identity verification are retained (`design.md:92-98`, `design.md:115`; `addendum.md:92-99`).

### Local Run, Maintenance, And Validation - Preserved

The complete `cargo run` startup contract and independence from Docker, frontend build, manual migrations, SQLx preparation, and Frankfurter are retained, as are the password helper and pre-release database consequence (`design.md:100-104`; `addendum.md:101-106`). Source-first maintenance, ADR supersession, synchronized artifacts, clean pre-release breaking changes, no compatibility shims, rewritable migrations, and no database compatibility promise are retained (`design.md:9`, `design.md:17`, `design.md:118-120`; `addendum.md:108-113`). Pinned/locked validation, architecture fitness, dependency policy, SQLx metadata, and independent helper checks remain explicit (`design.md:116`; `addendum.md:115-142`).

## Decision Audit

All relevant residual decisions are represented in an artifact:

- Personal-spending job and positioning: `prd.md:18`; sourced by `.memlog.md:13`, `.memlog.md:38`.
- Current-month summary override and exact shape: `design.md:60`, `prd.md:129-148`; sourced by `.memlog.md:15-18`.
- Minimal/mobile/no-animation UX: `prd.md:32-37`, `prd.md:246-253`; sourced by `.memlog.md:21`, `.memlog.md:30-31`.
- Group-centered information architecture and progressive disclosure: `prd.md:34-35`, `prd.md:94`; sourced by `.memlog.md:23`.
- Group-owned Participant model and lifecycle: `design.md:54-55`, `prd.md:81-90`; sourced by `.memlog.md:22`, `.memlog.md:25`.
- Source Currency editability: `prd.md:125`, `prd.md:259`; sourced by `.memlog.md:35`.
- Exact visible and technical boundaries: `design.md:50`, `prd.md:77`, `prd.md:89-116`, and `addendum.md`; sourced by `.memlog.md:36-37`.
- Final residual dispositions: synchronized across all three artifacts; recorded by `.memlog.md:38-39`.

No relevant decision remains only in memory, and no decision conflicts with the current normative design contract.

## Findings

- Critical: 0
- High: 0
- Medium: 0
- Low: 0

## Closure

The reconciliation gate is closed. `prd.md` is capability-focused without losing visible product boundaries; `addendum.md` retains the exact technical mechanisms and supplied project context; and `specs/design.md`, project context, PRD, addendum, and recorded user decisions are mutually consistent.
