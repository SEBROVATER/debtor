---
stepsCompleted:
  - step-01-document-discovery
  - step-02-prd-analysis
  - step-03-epic-coverage-validation
  - step-04-ux-alignment
  - step-05-epic-quality-review
  - step-06-final-assessment
status: READY FOR IMPLEMENTATION ASSIGNMENT
reassessmentCompletedAt: 2026-08-12T11:54:07+06:00
revalidatedAt: 2026-08-12T11:14:13+06:00
correctCourseResolvedAt: 2026-08-12T12:00:12+06:00
reassessmentStartedAt: 2026-08-12T12:08:40+06:00
latestReassessmentCompletedAt: 2026-08-12T12:08:40+06:00
includedFiles:
  prd:
    - prds/prd-debtor-2026-08-10/prd.md
    - prds/prd-debtor-2026-08-10/addendum.md
  architecture:
    - architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md
  epics:
    - epics.md
  ux:
    - ux-designs/ux-debtor-2026-08-10/DESIGN.md
    - ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md
supportingContext:
  - prds/prd-debtor-2026-08-10/reconcile-design.md
  - prds/prd-debtor-2026-08-10/reconcile-project-context.md
  - prds/prd-debtor-2026-08-10/review-rubric.md
  - architecture/architecture-debtor-2026-08-10/reviews/
  - ux-designs/ux-debtor-2026-08-10/reconcile-prd.md
  - ux-designs/ux-debtor-2026-08-10/reconcile-addendum.md
  - ux-designs/ux-debtor-2026-08-10/review-accessibility.md
  - ux-designs/ux-debtor-2026-08-10/review-one-handed.md
  - ux-designs/ux-debtor-2026-08-10/review-rubric.md
  - ux-designs/ux-debtor-2026-08-10/validation-report.md
---

# Implementation Readiness Assessment Report

**Date:** 2026-08-12
**Project:** debtor

## Document Inventory

### PRD

- `prds/prd-debtor-2026-08-10/prd.md`
- `prds/prd-debtor-2026-08-10/addendum.md`

### Architecture

- `architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md`

### Epics and Stories

- `epics.md`

### UX Design

- `ux-designs/ux-debtor-2026-08-10/DESIGN.md`
- `ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md`

### Discovery Notes

- All four required artifact families were found.
- No whole-versus-sharded duplicate conflict was found.
- Supporting reconciliation, review, validation, mockup, and working files remain available as corroborating context.

## PRD Analysis

### Functional Requirements

**FR-1: Password-gated access.** The Administrator can sign in with one configured password, remain authenticated during an active session, and sign out. Debtor provides no username, registration, Participant login, or multi-user authorization. Anonymous visitors cannot view Groups or ledger data; restart ends authenticated sessions; login and all state-changing actions require valid request protection. Every unsafe form also carries a bounded, expiring, session-bound, single-use submission token distinct from CSRF, with separate anonymous and authenticated capacity/expiry rules and terminal atomic reservation before dispatch.

**FR-2: Group lifecycle.** The Administrator can create, edit, archive, and restore a Group. A Group with no Spendings can be deleted with its unreferenced Participants; a Group with any Spending cannot be deleted. Names are trimmed, non-empty, and at most 100 Unicode characters. Creation asks only for name, assigns `USD`, and opens Manage; established Groups open Summary. Active lists exclude archived records, contextual archived views permit restoration, archived Groups remain readable without mutation controls, and direct archived mutations are rejected without state change.

**FR-3: Group-owned Participants.** The Administrator can add, edit, archive, and restore Participants inside a Group. Each Participant belongs to exactly one Group and is recreated independently in another Group. New allocations use only active Participants owned by the Group; archived Participants remain visible in history, Balances, and Settlement Transfers. Archive requires one immutable all-time Historical-mode ledger/time/quote context showing exact zero Group Currency Balance and successful eligibility revalidation at commit; invalidated context or missing rates blocks archive with retryable feedback and no state change. Restore needs no Balance check. Names are trimmed, non-empty, and at most 100 Unicode characters; colors use normalized `#RRGGBB`, with a varied valid suggestion on creation.

**FR-4: Record a Spending.** The Administrator can create a Spending with description, date, category, positive Total, Source Currency, exactly one Payer, and Proportional or Exact Shares. Supported currencies are `USD`, `EUR`, `RUB`, `KGS`, `TRY`, `KZT`, `UZS`, `CNY`, `KRW`, `JPY`, `OMR`, and `TJS`; categories are `food`, `transport`, `housing`, `fun`, `shopping`, `bills`, `health`, and `other`. Description is trimmed, non-empty, and at most 200 Unicode characters. Date is strict `YYYY-MM-DD` and no earlier than `2025-01-01`. Description and Total start empty, Source Currency defaults to Group Currency, date defaults to current UTC date, and Category and Payer have no default. Payer and Shares share one allocation table; selecting a Payer assigns the full Total and replaces any prior Payer. Validation retains submitted values and renders inline errors.

**FR-5: Exact allocation.** Every accepted Spending preserves Total exactly in Source Currency minor units: the single Payer pays Total and Shares independently sum to Total. Total and Payer/Share amounts are positive, at most `999_999_999_999`, and precision-valid; zero, excess precision, duplicate Participants, and mismatched totals are rejected. Proportional mode initially selects all active Participants at weight `1`, permits deselection, accepts positive weights up to `1,000,000` with at most six fractional digits, and uses one checked integer-ratio operation for Preview and commit with descending remainders and ascending Participant-ID ties. Exact mode initially selects all active Participants, divides minor units equally, assigns residual units by ascending Participant ID, permits deselection/editing, and displays the difference until exact closure. Updates may retain an archived Participant only in the same existing Payer or Share role.

**FR-6: Review and maintain history.** The Administrator can browse, inspect, edit, and delete Spendings in an active Group. History is newest-first in pages of 25. Details remain readable for archived Groups/Participants and show current Participant names. Editing may correct Source Currency under creation validation, and later historical calculations use the corrected value. Input mode and proportional weights are not persisted; edit opens Exact with stored Payer and Shares. Every successful change is all-or-nothing.

**FR-7: Source Currency summary.** The Administrator can see the selected Group's current-month Spending Total and each Payer's paid total grouped by original Source Currency. These totals require no conversion, and Spendings outside the current UTC calendar month are excluded.

**FR-8: Group Currency summary.** The Administrator can see the same current-month Group and per-Payer totals converted to Group Currency using each Spending date's historical rate. Future Spendings use the latest current rate and are provisional. Context-matching fixed past quotes have no age limit; current fallback uses the latest prior current-class quote; future fallback also matches original requested date; current/future stale quotes remain eligible through seven UTC calendar days and carry warnings. Values accumulate exactly per Payer and are quantized together by truncation, descending remainder, and ascending Participant-ID ties; Group total is their exact sum. Missing quote or checked calculation failure makes the whole converted section retryably unavailable without partial totals, while Source Currency totals, history, and mutations remain usable.

**FR-9: Select conversion mode.** The Administrator can calculate all-time Balances in Historical or Current mode. Historical is default and converts each Spending at its date; Current converts every Spending at the UTC calculation date and is not persisted.

**FR-10: Exact Balances.** Debtor calculates one exact Group Currency Balance per Participant and preserves exact zero sum after currency quantization. Rate-request completion order cannot alter results or warnings, and arithmetic/conversion failure returns no partial Balances or Settlement Transfers.

**FR-11: Deterministic Settlement Transfers.** Debtor presents positive, deterministic Settlement Transfers that settle every Balance. A Participant pair appears at most once, no more than `n - 1` transfers are produced for `n` Participants, and global transfer-count minimality is not claimed.

**FR-12: Calculation disclosure and failure isolation.** The debts view identifies conversion mode, calculation time, Group Currency, unique rates, and stale/provisional warnings. Missing required quotes without valid stale fallback produce a retryable failure; exchange-rate failure never prevents Group, Participant, or Spending management.

**Total functional requirements: 12.** Their consequence clauses above are normative acceptance detail, not optional commentary.

### Non-Functional Requirements

**NFR-1:** Core behavior must work through semantic server-rendered HTML and valid native links/forms. Pinned self-hosted HTMX may progressively enhance them; custom application JavaScript and inline script attributes are forbidden.

**NFR-2:** One mobile-friendly web experience must remain usable on desktop without a separate desktop design.

**NFR-3:** The interface must work in current stable Chrome, Firefox, Safari, and Edge down to 320 CSS pixels.

**NFR-4:** Every control must be pointer-independent, programmatically labeled, and have a visible focus indicator at least two CSS pixels thick with at least 3:1 adjacent-color contrast.

**NFR-5:** Normal text must reach 4.5:1 contrast; large text, UI components, and meaningful graphics must reach 3:1; inline errors must be programmatically associated. Formal accessibility certification is not required.

**NFR-6:** Validation must identify errors inline and retain submitted values.

**NFR-7:** Archived state and stale, provisional, or unavailable conversion results must be visibly distinguishable.

**NFR-8:** Monetary input, storage, aggregation, conversion, and display must preserve exact decimal values and currency minor-unit rules without floating-point loss.

**NFR-9:** Historical references must remain readable after Group or Participant archival.

**NFR-10:** Every complete Spending write must validate Group ownership and Participant eligibility atomically.

**NFR-11:** Complete Spending and debt views must use internally consistent snapshots.

**NFR-12:** Every state-changing request, including login, must be authenticated where applicable and protected against CSRF.

**NFR-13:** Unsafe form replay must be suppressed server-side with a bounded, expiring, session-bound, single-use token distinct from CSRF and atomically reserved before dispatch.

**NFR-14:** Authentication must resist repeated login attempts and use secure production session cookies.

**NFR-15:** Credentials, password hashes, session IDs, request-protection tokens, and sensitive ledger/provider data must never appear in logs or user-facing errors.

**NFR-16:** Authenticated pages must not be cached by browsers or intermediaries.

**NFR-17:** Exchange-rate-provider availability must not gate startup, readiness, or ledger CRUD.

**NFR-18:** User traffic, login, probes, database waits, exchange-rate calls, caches, and sessions must have bounded resource usage and wait times.

**NFR-19:** Once an admitted mutation starts changing state, it must return definitive success or rollback rather than be cancelled by a generic timeout.

**NFR-20:** Shutdown must stop new admission, drain in-flight work within its defined HTTP bound, and leave the ledger recoverable.

**Total explicit PRD NFRs: 20** (seven UX acceptance requirements and thirteen cross-cutting quality requirements).

### Additional Requirements

The PRD addendum is normative technical acceptance scope. The following numbered constraint groups preserve all of its implementation obligations for traceability:

**AR-1 Product boundaries:** Permanent single Administrator; Participants are independent Group-owned accounting identities; no global Participants/Memberships; v1 has only fixed current-UTC-month summaries.

**AR-2 Layering and ownership:** Preserve `root -> web/infra -> application -> domain`; domain owns synchronous deterministic rules, application owns use cases/input policy/ports, infra owns concrete adapters, web owns HTTP/session/forms/rendering, and root owns configuration/composition/migrations/lifecycle.

**AR-3 Port purity and testability:** Axum, SQLx, reqwest, Argon2, tower-sessions, and other outer types cannot cross application ports; external effects and clocks are injected and use cases run with fakes without frameworks, SQLite, network, or wall clock.

**AR-4 Input ownership:** Web decodes structure and preserves raw text; application parses values, validates allocation/ownership/lifecycle/mode, constructs allocations, and enforces financial invariants. Application policy and transactional race guards remain distinct.

**AR-5 Exact money:** Use `Decimal`; forbid float/lossy conversion and SQL monetary parsing/conversion/aggregation. Values are positive, at most `999_999_999_999`, and precision-valid: JPY/KRW 0, OMR 3, all other supported currencies 2 minor units. Persist canonical decimal SQLite `TEXT` and reject malformed/noncanonical stored values as corruption.

**AR-6 Allocation determinism:** Exactly one active Group-owned Payer; nonempty unique positive Shares exactly conserve Total. Proportional and Exact allocation follow the FR-5 limits and Participant-ID tie rules. Checked failures never panic, substitute zero, or return partial results.

**AR-7 IDs and deterministic finance:** Ledger IDs are positive `i64`; UUID is session/CSRF-only. Sort output-affecting unordered inputs; Participant ID is final tie-breaker. Balances use largest signed-remainder quantization and exact zero-sum conservation. Settlement uses deterministic greedy matching by descending absolute Balance then Participant ID and remains positive, complete, pair-unique, and at most `n - 1`.

**AR-8 SQLite topology and durability:** One process and one local SQLite volume only; use WAL, `synchronous=FULL`, foreign keys, five-second busy timeout, and one five-second process-local ledger write gate whose timeout precedes transaction creation.

**AR-9 Transactional history integrity:** Ownership, eligibility, aggregate replacement, allocations, and commit occur in one transaction. Archived allocation roles cannot be introduced or changed. Referenced Group deletion is restricted; history-free Group deletion may remove unreferenced owned Participants; Participants otherwise archive/restore. Last committed valid write wins; no optimistic revisions.

**AR-10 Participant archive admission:** Bind one immutable all-time Historical ledger snapshot, UTC context, and quote bundle; provider calls hold no transaction; commit revalidates ledger epoch/date/quote eligibility; mismatch or missing quote blocks without mutation; rate evidence is not persisted; restore needs no Balance check.

**AR-11 Persistence constraints and SQLx:** SQLite structurally constrains supported codes, flags, bounded text, color shape, dates, relationships, and referenced deletion without duplicating Unicode trimming or Rust money arithmetic. Use compile-time checked SQLx and matching committed `.sqlx` metadata; only the fixed WAL-checkpoint PRAGMA is an unchecked exception.

**AR-12 Snapshot and loading:** Load complete Spending aggregates from one snapshot. Materialize all debt inputs before releasing the read transaction and making provider calls. Use 25-item keyset history ordered `(spent_date DESC, id DESC)`; detail/edit/delete directly load one aggregate and resolve archived identities/current names.

**AR-13 Safe failures and logging:** Ports and HTTP expose structured safe categories, never raw adapter errors. Use `thiserror` by layer and root-only `anyhow`. Calculation errors are sanitized and nonpartial. SQLite logs use only the fixed operation/category allowlists and exclude SQL/messages/values/IDs/query strings/provider URLs/request data/secrets/tokens/IP-derived data.

**AR-14 Rate precision and identity:** Decode provider JSON numbers lexically into arbitrary-precision `Decimal`. For requested date `R` and UTC calculation date `C`, fetch `F=min(R,C)` and key dedupe/single-flight/cache by `(source,target,R,F)` while retaining effective-date evidence. Fixed-past fallback matches exact key; current fallback uses prior current-class pair; future fallback also matches `R`; same-currency is disclosed synthetic exact `1` without I/O.

**AR-15 Rate cache and request bounds:** Stable historical and refreshable current/future caches each cap at 4,096 with deterministic LRU. Fixed-past quotes remain eligible without age limit; current/future refresh on UTC rollover and remain stale-eligible through seven UTC days. Provider connect timeout is five seconds, total timeout 20 seconds, body limit 64 KiB, global concurrency four, and identical misses single-flight. A debt calculation deduplicates contexts and issues at most four requests concurrently without completion-order effects.

**AR-16 Web rendering and forms:** Askama semantic HTML and vanilla CSS; only pinned self-hosted HTMX and official `response-targets`; no custom extension/application JS/inline script attributes; every core path works natively. One strict extractor rejects malformed/missing/duplicate/unknown fields and invalid CSRF before route parsing/verification/dispatch.

**AR-17 Submission-token protocol:** Separate anonymous 4,096-token pool (one/session, ten-minute inactivity) and authenticated 1,024-token pool (32/session, 30-minute absolute); indexed cleanup supervisor; validation preserves token; atomic pre-dispatch reservation is terminal; missing/unknown/expired/reserved/consumed returns `409` without dispatch.

**AR-18 HTTP mutation semantics:** Validation returns `422` with inline errors and all raw values; success redirects `303`; archived form/mutation routes return `409` pre-use-case. Mark dispatch immediately before first state-changing call. A 30-second absolute pre-dispatch deadline covers extraction/auth/CSRF/prechecks; no generic post-dispatch timeout; response reports definitive commit/rollback. Missing debt quote maps retryable `503`; session promotion capacity maps `503`; new-key limiter exhaustion maps `429`.

**AR-19 Password contract:** `APP_ADMIN_PASSWORD_HASH` is required, at most 256 encoded bytes, canonical Argon2id v19 PHC with exactly `m/t/p`, memory `19,456..=65,536` KiB, time `2..=5`, parallelism `1..=4`, salt 16..64 bytes, output 32..64 bytes, validated before DB connection/migration. Helper emits `m=19,456,t=2,p=1`, 16-byte OS salt, 32-byte output. Verification concurrency is two.

**AR-20 Sessions and login:** In-memory server-side sessions with HTTP-only `SameSite=Strict`; secure cookies outside debug; restart invalidates sessions. Anonymous inactivity ten minutes/cap 4,096/no authenticated eviction; authenticated inactivity 30 days/cap 32/no eviction. Promotion at capacity flushes anonymous state and returns `503`. Successful login atomically rotates and durably stores ID/auth/CSRF before cookie; limiter reserves each post-CSRF verification and resets only after durable promotion; logout flushes. Mandatory indexed cleanup failure fails readiness, admission, and process health. Every unsafe request requires exactly one valid synchronizer token. Login permits five attempts per trusted IP per rolling five minutes, cap 4,096 active keys, no active eviction, fail-closed `429` at capacity.

**AR-21 Headers, proxy, and session-free routes:** Login/authenticated HTML sends the specified no-store, nosniff, no-referrer, and restrictive CSP headers. Probe/static routes do not create/load sessions. Forwarded identity is trusted only from configured immediate-peer CIDRs in one configured format; production requires valid nonempty settings; identity must match across HTTP/3 and TCP fallback.

**AR-22 Admission and timeouts:** Login body 8 KiB, other forms 256 KiB; user permits 64, login permits four, separate probe permits four. Safe dynamic reads/login timeout 30 seconds, Debts 90, probes two outer, SQLite readiness one inner, write gate and DB lock five each. Health is process liveness; readiness checks SQLite and mandatory supervisors, never provider or ledger content.

**AR-23 Shutdown outcome:** Stop admission, drain HTTP at most ten seconds, then wait without fixed total deadline for dispatched mutations before checkpoint/pool close. Executor publishes authoritative `Committed`/`RolledBack`; task failure is `RolledBack` only when established, otherwise `Unknown` causes fatal shutdown and no retry. Checkpoint failure preserves WAL sidecars.

**AR-24 Edge transport:** Sanitizing HTTPS proxy owns TLS/certificates/HTTP3/QUIC/Alt-Svc/client fallback; app is private HTTP/1.1 TCP. Edge sanitizes forwarding, aligns trusted settings, disables early data or returns `425` for unsafe requests, allows only marked GET/HEAD early-data paths, reuses backend connections, never times out admitted post-dispatch mutations, enforces matching body limits, and rolls out HTTP/3 with short Alt-Svc plus UDP/fallback/425/identity verification.

**AR-25 Local operation:** With `.env` and valid hash, `cargo run` loads config, creates/connects DB, migrates, enables foreign keys, composes, binds, logs a nonsecret `http://` URL, and shuts down gracefully. Local startup needs no Docker/frontend build/manual migration/metadata generation/provider availability. Hash helper is independent; secrets stay out of commands/logs/fixtures/repo. Pre-release monetary DBs may require recreation.

**AR-26 Maintenance policy:** `specs/design.md` is normative and changes first; synchronize ADRs, README, config, migrations, tests, and SQLx metadata. ADR supersession is explicit and synchronized. Before deployment, clean breaking API/config/route/schema changes and migration rewrites are allowed without compatibility shims, while security/accounting/history invariants remain mandatory.

**AR-27 Toolchain:** Pin Rust 1.97.1/edition 2024/MSRV 1.97/resolver 3 and the listed crate versions; preserve lockfiles and `--locked`; consult current crate docs for API changes. Production and password-helper workspaces remain independent.

**AR-28 Testing:** Domain tests cover examples/boundaries/properties for exact finance and determinism; application tests use injected clocks/simple fakes without outer systems; infra/web adapters cover malformed/oversized input, corruption, safe errors, cache/single-flight/concurrency, auth/session/limiter capacity, and strict forms/CSRF. Use temporary file `#[sqlx::test]` for WAL/locking/multi-connection/migration constraints. Concurrency tests use deterministic coordination, not sleeps. Web tests verify statuses/headers/redirects/retention/rejection/no-dispatch. Retain root real-socket auth/CSRF/read/startup/shutdown smoke test. Tests live at invariant-owning layer; broad lint weakening is forbidden.

**AR-29 Quality and validation:** Keep fmt clean, Clippy pedantic warning-free, no unsafe, avoid production `unwrap`/`expect`, and avoid broad suppression. Follow module/interface/implementation/input/DB/view naming rules and public rustdoc `# Errors`. Prefer minimal local changes. Run the specified locked workspace fmt/check/clippy/test, architecture fitness, conditional `cargo deny`, SQLx prepare after checked SQL/migrations, and independent helper fmt/clippy/test. Never use release builds for routine validation.

### PRD Completeness Assessment

The product requirements are unusually complete and testable: all 12 product capabilities have explicit consequences, financial invariants are quantitative, failure isolation is defined, and no open questions or unconfirmed assumptions remain. The addendum supplies detailed architecture, security, capacity, timeout, persistence, rollout, and testing acceptance contracts.

The principal traceability risk is document layering rather than missing intent: `specs/design.md` is explicitly normative, while `prd.md`, `addendum.md`, architecture, UX, project context, and epics must remain synchronized. Readiness therefore depends on proving that epics cover both the 12 product FRs and the 29 grouped technical constraint families, not merely the headline FR list.

## Epic Coverage Validation

### Epic FR Coverage Extracted

- **FR-1:** Epic 1, Stories 1.1-1.10; first real mutation evidence completes in Story 2.1.
- **FR-2:** Epic 2, Stories 2.1-2.2 and 2.5; Spending-backed deletion proof completes in Story 3.3.
- **FR-3:** Epics 2 and 5, Stories 2.3-2.4 and 5.4-5.5; historical-name projection is proved in Story 3.4.
- **FR-4:** Epic 3, Stories 3.1-3.3.
- **FR-5:** Epic 3, Stories 3.1-3.3 and 3.5.
- **FR-6:** Epic 3, Stories 3.4-3.6.
- **FR-7:** Epic 4, Story 4.1.
- **FR-8:** Epic 4, Stories 4.2-4.3.
- **FR-9:** Epic 5, Stories 5.1-5.2.
- **FR-10:** Epic 5, Stories 5.1-5.2.
- **FR-11:** Epic 5, Story 5.3.
- **FR-12:** Epics 4 and 5, Stories 4.2-4.3 and 5.1-5.2.

**Total PRD FRs claimed in epics: 12.**

### Coverage Matrix

| FR | PRD requirement | Epic and story coverage | Status |
|---|---|---|---|
| FR-1 | Password-gated access | Epic 1, Stories 1.1-1.10; Story 2.1 completes real-mutation evidence | Covered |
| FR-2 | Group lifecycle | Epic 2, Stories 2.1-2.2, 2.5; Story 3.3 completes Spending-backed deletion restriction | Covered |
| FR-3 | Group-owned Participants | Epics 2/5, Stories 2.3-2.4, 5.4-5.5; Story 3.4 proves current-name history | Covered |
| FR-4 | Record a Spending | Epic 3, Stories 3.1-3.3 | Covered |
| FR-5 | Exact allocation | Epic 3, Stories 3.1-3.3, 3.5 | Covered |
| FR-6 | Review and maintain history | Epic 3, Stories 3.4-3.6 | Covered |
| FR-7 | Source Currency summary | Epic 4, Story 4.1 | Covered |
| FR-8 | Group Currency summary | Epic 4, Stories 4.2-4.3 | Covered |
| FR-9 | Select conversion mode | Epic 5, Stories 5.1-5.2 | Covered |
| FR-10 | Exact Balances | Epic 5, Stories 5.1-5.2 | Covered |
| FR-11 | Deterministic Settlement Transfers | Epic 5, Story 5.3 | Covered |
| FR-12 | Calculation disclosure and failure isolation | Epics 4/5, Stories 4.2-4.3, 5.1-5.2 | Covered |

### Missing Requirements

No PRD functional requirement is missing from the epics and stories artifact.

The epics document additionally decomposes the combined product and technical contract into `SPEC-FR1..SPEC-FR105` and `SPEC-NFR1..SPEC-NFR34`. These are source-qualified implementation requirements with an explicit coverage map and are not unapproved product features absent from the PRD.

### Coverage Statistics

- Total PRD FRs: 12
- PRD FRs covered in epics: 12
- Missing PRD FRs: 0
- Coverage: 100%

## UX Alignment Assessment: 2026-08-12 Reassessment

### UX Document Status

**Found and final.** The selected contracts are `ux-designs/ux-debtor-2026-08-10/DESIGN.md` (visual) and `EXPERIENCE.md` (structure, interaction, state, and accessibility).

### PRD Alignment

- UX Flow 1 covers PRD UJ-1 and FR-4 through FR-12: native Spending entry, exact allocation Preview/approval, Transactions, conversion-independent source totals, converted totals, Balances, Settlement, disclosure, and complete-or-no-result failure handling.
- UX Flow 2 covers UJ-2 and FR-2 through FR-3: name-only USD Group creation into Manage, Group-local Participants, archive eligibility, contextual restore, history-preserving lifecycle, and active versus archived separation.
- UX Flow 3 covers FR-1: password-only single-Administrator access, protected Sign in/out, private data, and session loss behavior.
- UX Flow 4 completes FR-6 correction and deletion behavior. The contract preserves all PRD boundaries: one Payer, Proportional/Exact modes, no repayment state, no manual rate retry, no extra desktop IA, no custom application JavaScript, and no HTMX dependency.

### Architecture Alignment

- AD-11 provides the required Askama semantic HTML, native links/forms, pinned self-hosted HTMX and official `response-targets` only, restrictive CSP, static-asset rules, and native fallback.
- AD-18 makes the final stable `UX-*` registry binding for every affected route, template, projection, CSS rule, enhancement, and web acceptance test; the epics attach the required UX IDs to the relevant stories.
- AD-3 through AD-10 support exact allocation, archived historical identity, snapshot calculations, deterministic rate disclosure, protected mutations, and definitive outcomes required by UX states.
- AD-12 through AD-16 support responsive online delivery, safe failures, bounded admission/timeouts, session-free probes/assets, provider-independent readiness/CRUD, and testable implementation.

### Alignment Issues

No PRD-to-UX or UX-to-Architecture contradiction was found.

### Warnings

- Authority is intentionally layered: `specs/design.md` and accepted ADRs govern product, security, accounting, and architecture; final UX contracts govern visual and interaction detail within that envelope. Divergence must halt work for reconciliation.
- Architecture defers physical route/template layout, responsive content-fit thresholds, and verified HTMX asset digests to first implementation. These are bounded implementation decisions, and the owning story must close them before the relevant route/enhancement is accepted.

## Epic Quality Review: 2026-08-12 Reassessment

### Epic Structure

The five epics deliver ordered Administrator outcomes rather than detached technology milestones:

1. Secure local operation and access.
2. Group and active-Participant setup.
3. Exact Spending recording and maintenance.
4. Current-month understanding.
5. All-time debts, settlement, and rate-dependent Participant retirement.

The product ordering is coherent. Epic 2 can operate after secure access; Epic 3 has the Group and active Participant context it needs; Epic 4 uses committed Spendings; Epic 5 reuses completed history and rate capability. Persistence is introduced just in time: runtime foundation in 1.2, Groups in 2.1, Participants in 2.3, and Spendings/allocations in 3.1. No starter template is specified, and brownfield replacement/removal is explicitly assigned without compatibility shims.

### Critical Violations

None found.

### Major Issues

#### M1: Epic 1 has a forward dependency on Story 1.7 for its login and logout outcomes

**Evidence:**

- Story 1.4 issues an anonymous submission token and requires its capacity/expiry behavior.
- Story 1.5 requires atomic token reservation immediately before Login verification.
- Story 1.6 requires token reservation and replay rejection for Sign out.
- Story 1.7, which appears after all three, is defined as the owner of the shared submission-token lifecycle, strict unsafe-form boundary, reservation race handling, cleanup, and no-dispatch contract.

**Impact:** Stories 1.4-1.6 cannot independently meet their own acceptance criteria from preceding work. Implementing token functionality early would either pre-implement Story 1.7 out of order or duplicate/rework its shared boundary, violating the no-forward-dependency and first-consumer principles.

**Recommendation:** Recut or reorder the work so the first Login-rendering story owns the minimal anonymous token issuance/store path, Login submission owns its reservation path, and Sign out reuses it. Move Story 1.7 before the first unsafe route only if it becomes a runnable protected user outcome with its own initial route; otherwise reduce it to the cross-route extension/verification work that follows the already-delivered first consumers. Update the packet and requirement ownership to make the first consumer unambiguous.

### Minor Concerns

#### m1: Acceptance criteria remain highly compound

Many stories combine route behavior, domain rules, persistence, concurrency, operational lifecycle, UX geometry, and all validation commands in the same BDD chains. The Assignment Packets reduce this risk for the ten previously high-complexity stories, but the criteria remain difficult to review atomically.

**Recommendation:** Keep user-outcome scenarios at route/use-case level and reference the shared packet checklist for recurring validation and cross-cutting proof. Preserve route-specific security, financial, and UX evidence in each story.

### Dependency Assessment

- No circular epic dependency was found.
- The Final-Evidence Ledger correctly distinguishes introduction from completion for `SPEC-FR29`, `SPEC-FR30`, `SPEC-FR40`, `SPEC-FR103`, and `SPEC-FR104`.
- Stories 3.1 and 3.2 are now vertical committed Spending outcomes. Story 3.1 is the sole shared aggregate-persistence owner and Story 3.2 explicitly reuses it; the former duplicate integrity ownership is removed.
- Story 1.10 is correctly marked as a pre-production operations gate, not a Phase 4 application assignment.
- The only forward execution dependency found is the submission-token sequencing issue in M1.

### Best-Practice Checklist

| Epic | User value | Independent ordering | Story sizing | No forward dependency | Just-in-time persistence | Clear/testable AC | Traceability |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Epic 1 | Pass | Needs work: M1 | Packets mitigate risk | Fail: M1 | Pass | Pass, compound | Pass |
| Epic 2 | Pass | Pass | Packets mitigate risk | Pass | Pass | Pass, compound | Pass |
| Epic 3 | Pass | Pass | Packets mitigate risk | Pass | Pass | Pass, compound | Pass |
| Epic 4 | Pass | Pass | Packets mitigate risk | Pass | Pass | Pass, compound | Pass |
| Epic 5 | Pass | Pass | Packets mitigate risk | Pass | Pass | Pass, compound | Pass |

### Quality Verdict

The revised epics resolve the prior Spending vertical-slice, duplicate persistence ownership, sizing, deferred-evidence, and edge-scope blockers. One material sequencing defect remains: shared submission-token behavior is assigned after the Login and Sign out stories that require it. Correct that dependency before implementation assignment; the remaining concern is documentation/review granularity rather than missing product or architecture scope.

## Summary and Recommendations: 2026-08-12 Reassessment

### Overall Readiness Status

**SUPERSEDED: READY FOR IMPLEMENTATION ASSIGNMENT**

This assessment section is retained as the original finding record. The Sprint Change Proposal and the Epic 1 first-consumer correction recorded below resolved the remaining sequencing defect. The PRD packet is complete and reconciled; all 12 functional requirements have 100% epic/story coverage; UX and Architecture align; the revised Spending path is vertical; high-complexity stories have bounded Assignment Packets; and the pre-production edge contract is correctly separated from Phase 4 application implementation.

The earlier Phase 4 hold is lifted: the Epic 1 first-consumer correction is recorded in `epics.md` and the approved Sprint Change Proposal.

### Critical Issues Requiring Immediate Action

No critical artifact, requirement, UX, architecture, or epic-level dependency issue was found.

### Major Issue Requiring Resolution

1. **Correct the Epic 1 submission-token sequence.** Stories 1.4-1.6 require token issuance, reservation, expiry/capacity behavior, and replay rejection, but Story 1.7 is the planned owner of that shared functionality. Reassign first-consumer ownership or reorder/recut Stories 1.4-1.7 so no story depends on later work to meet its own criteria.

### Recommended Next Steps

1. Update `epics.md` to resolve M1: put each minimal token capability in its first runnable consumer, or position a genuinely vertical protected-form story before Login/Sign out.
2. Synchronize the `SPEC-FR15..SPEC-FR19` coverage map, Story 1.4-1.7 requirement lists, Assignment Packets, and Final-Evidence Ledger so shared token completion cannot be misreported.
3. Rerun implementation readiness with emphasis on Epic 1 dependency order after the corrected story plan is finalized.

### Final Note

This reassessment identified **1 major planning issue and 1 minor documentation concern across 2 categories**. It found **0 missing PRD FRs, 0 PRD/UX/Architecture contradictions, 0 technical-only epics, 0 circular dependencies, and 0 remaining Spending-persistence ownership conflicts**.

**Assessment date:** 2026-08-12

**Assessor:** Kilo, Product Requirements and Traceability Review

## Document Inventory: 2026-08-12 Reassessment

### PRD

- `prds/prd-debtor-2026-08-10/prd.md`
- `prds/prd-debtor-2026-08-10/addendum.md`

### Architecture

- `architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md`

### Epics and Stories

- `epics.md`

### UX Design

- `ux-designs/ux-debtor-2026-08-10/DESIGN.md`
- `ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md`

### Discovery Notes

- All four required artifact families are present.
- No whole-versus-sharded duplicate conflict was found.
- The current `epics.md` modification postdates the prior assessment record, so the current artifacts will be reassessed.

## PRD Analysis: 2026-08-12 Reassessment

### Functional Requirements

- **FR-1 Password-gated access:** One configured-password Administrator can sign in, maintain an active session, and sign out; no usernames, registration, Participant login, or multi-user authorization. Anonymous ledger access is denied, restart ends sessions, unsafe requests require request protection, and every unsafe form has one bounded, expiring, session-bound, single-use submission token distinct from CSRF with atomic terminal reservation and replay conflict handling.
- **FR-2 Group lifecycle:** The Administrator can create, edit, archive, restore, and, only when history-free, delete Groups with unreferenced Participants. Names are constrained; creation accepts only a name, sets USD, and opens Manage; established Groups open Summary; archived records are separated contextually, remain readable, and reject mutation.
- **FR-3 Group-owned Participants:** The Administrator can add, edit, archive, and restore Group-local Participants. Allocation permits only active Group-owned identities; historical identity remains visible. Archive requires an immutable all-time Historical exact-zero calculation context that remains eligible on commit, while restore needs no Balance check. Names and normalized colors are constrained.
- **FR-4 Record a Spending:** The Administrator can create a Spending with constrained description, date, category, positive total, source currency, one Payer, and Proportional or Exact Shares. Defaults, allocation-table behavior, permitted codes, and retained inline validation are specified.
- **FR-5 Exact allocation:** A Spending's Payer and independently summed Shares conserve its Total exactly in source-currency minor units. Positive/precision/bound/uniqueness rules, deterministic Proportional allocation, Exact initialization and closure, and archived-role update restrictions are specified.
- **FR-6 Review and maintain history:** The Administrator can browse, inspect, edit, and delete active-Group Spendings. History uses newest-first 25-item pages; details survive archival and use current names; edit corrects source currency and opens Exact; changes are atomic.
- **FR-7 Source Currency summary:** The Administrator sees selected-Group current-UTC-month total and Payer totals grouped by source currency without conversion dependency.
- **FR-8 Group Currency summary:** The Administrator sees equivalent converted current-month totals using historical/current/future rate rules, exact accumulation and deterministic quantization. A missing quote or checked calculation failure makes the complete converted section retryably unavailable without disabling source totals, history, or mutations.
- **FR-9 Select conversion mode:** The Administrator can calculate all-time Balances using default Historical or non-persisted Current mode.
- **FR-10 Exact Balances:** Debtor calculates one Group-Currency Balance per Participant with exact zero sum after quantization, deterministic despite rate-request completion order, and no partial calculation result.
- **FR-11 Deterministic Settlement Transfers:** Debtor produces positive, complete, deterministic, pair-unique Transfers of at most `n - 1`, without claiming global minimum count.
- **FR-12 Calculation disclosure and failure isolation:** Debts disclose mode, calculation time, Group Currency, unique rates, and stale/provisional warnings; unavailable quotes cause retryable failure without preventing ledger management.

**Total FRs: 12.**

### Non-Functional Requirements

- **NFR-1:** Core behavior uses semantic server-rendered HTML and native links/forms; only pinned self-hosted HTMX may progressively enhance; custom JavaScript and inline script attributes are forbidden.
- **NFR-2:** One mobile-friendly web experience remains usable on desktop.
- **NFR-3:** Support current stable Chrome, Firefox, Safari, and Edge down to 320 CSS pixels.
- **NFR-4:** Controls are pointer-independent, programmatically labeled, and have a two-CSS-pixel, 3:1 visible focus indicator.
- **NFR-5:** Text/component contrast meets 4.5:1/3:1 and inline errors are programmatically associated.
- **NFR-6:** Validation is inline and retains submitted values.
- **NFR-7:** Archived, stale, provisional, and unavailable states are visually distinct.
- **NFR-8:** Monetary input, persistence, aggregation, conversion, and display preserve exact decimals and minor-unit rules without floats.
- **NFR-9:** Historical references remain readable after identity archival.
- **NFR-10:** Complete Spending writes atomically validate Group ownership and Participant eligibility.
- **NFR-11:** Complete Spending and debt views use internally consistent snapshots.
- **NFR-12:** State-changing requests, including login, use applicable authentication and CSRF protection.
- **NFR-13:** Unsafe form replay is server-side suppressed through bounded, expiring, session-bound, atomically reserved single-use tokens distinct from CSRF.
- **NFR-14:** Authentication resists repeated login attempts and production uses secure session cookies.
- **NFR-15:** Credentials, hashes, session/request-protection identifiers, and sensitive ledger/provider data are excluded from logs and user errors.
- **NFR-16:** Authenticated pages are not cacheable by browsers or intermediaries.
- **NFR-17:** Provider availability does not gate startup, readiness, or ledger CRUD.
- **NFR-18:** Traffic, login, probes, database waits, rate calls, caches, and sessions have bounded resource use and waits.
- **NFR-19:** An admitted state-changing mutation reaches definitive success or rollback, not generic-timeout cancellation.
- **NFR-20:** Shutdown stops admission, drains in-flight work within its defined HTTP bound, and leaves a recoverable ledger.

**Total explicit NFRs: 20.**

### Additional Requirements

The normative addendum introduces 29 implementation constraint families: product boundaries; inward layering; port purity; input ownership; exact canonical Decimal money; allocation and deterministic-finance rules; SQLite durability/history/transaction integrity; archival admission; checked SQLx and schema constraints; snapshot/direct loading/keyset pagination; safe diagnostics; lexical rate parsing/context identity/cache/concurrency; native rendering and strict form semantics; submission-token protocol; password/session/CSRF/proxy/header contracts; admission/timeouts/probes/shutdown; HTTPS edge rollout; local operation; maintenance policy; pinned toolchain; testing; and quality validation.

### PRD Completeness Assessment

The PRD is final, contains no open questions or unconfirmed assumptions, and specifies measurable business, accounting, history, usability, security, availability, and operational outcomes. `specs/design.md` remains normative: divergence among it, the PRD, and addendum is an explicit stop condition. Epic validation must therefore cover the 12 FRs, 20 NFRs, and all 29 addendum constraint families.

## Epic Coverage Validation: 2026-08-12 Reassessment

### Epic FR Coverage Extracted

- **PRD-FR-1:** Epic 1, Stories 1.1-1.10; Story 2.1 supplies final real-ledger-mutation lifecycle evidence.
- **PRD-FR-2:** Epic 2, Stories 2.1-2.2 and 2.5; Story 3.1 supplies Spending-backed deletion-restriction evidence.
- **PRD-FR-3:** Epics 2 and 5, Stories 2.3-2.4 and 5.4-5.5; Story 3.3 supplies current-name history evidence.
- **PRD-FR-4:** Epic 3, Stories 3.1-3.2.
- **PRD-FR-5:** Epic 3, Stories 3.1-3.2 and 3.4.
- **PRD-FR-6:** Epic 3, Stories 3.3-3.5.
- **PRD-FR-7:** Epic 4, Story 4.1.
- **PRD-FR-8:** Epic 4, Stories 4.2-4.3.
- **PRD-FR-9:** Epic 5, Stories 5.1-5.2.
- **PRD-FR-10:** Epic 5, Stories 5.1-5.2.
- **PRD-FR-11:** Epic 5, Story 5.3.
- **PRD-FR-12:** Epics 4 and 5, Stories 4.2-4.3 and 5.1-5.2.

### Coverage Matrix

| FR | PRD requirement | Epic and story coverage | Status |
|---|---|---|---|
| FR-1 | Password-gated access | Epic 1, Stories 1.1-1.10; final mutation evidence 2.1 | Covered |
| FR-2 | Group lifecycle | Epic 2, Stories 2.1-2.2 and 2.5; final deletion evidence 3.1 | Covered |
| FR-3 | Group-owned Participants | Stories 2.3-2.4, 3.3, 5.4-5.5 | Covered |
| FR-4 | Record a Spending | Stories 3.1-3.2 | Covered |
| FR-5 | Exact allocation | Stories 3.1-3.2 and 3.4 | Covered |
| FR-6 | Review and maintain history | Stories 3.3-3.5 | Covered |
| FR-7 | Source Currency summary | Story 4.1 | Covered |
| FR-8 | Group Currency summary | Stories 4.2-4.3 | Covered |
| FR-9 | Select conversion mode | Stories 5.1-5.2 | Covered |
| FR-10 | Exact Balances | Stories 5.1-5.2 | Covered |
| FR-11 | Deterministic Settlement Transfers | Story 5.3 | Covered |
| FR-12 | Calculation disclosure and failure isolation | Stories 4.2-4.3 and 5.1-5.2 | Covered |

### Missing Requirements

No PRD functional requirement is missing. The plan expands the source requirements into `SPEC-FR1..SPEC-FR105` and `SPEC-NFR1..SPEC-NFR34`; these are traceable decompositions of the approved PRD/addendum/architecture/UX contract, not unapproved features. The Final-Evidence Ledger appropriately distinguishes enabling work from completion for shared cross-epic requirements.

### Coverage Statistics

- Total PRD FRs: 12
- FRs covered in epics: 12
- Missing PRD FRs: 0
- Coverage: 100%

## UX Alignment Assessment: 2026-08-12 Reassessment

### UX Document Status

**Found and final.** `ux-designs/ux-debtor-2026-08-10/DESIGN.md` governs visual identity, tokens, geometry, and responsive composition. `EXPERIENCE.md` governs information architecture, route behavior, state, focus, announcements, and native/enhanced parity. Both cite the selected PRD and addendum.

### PRD Alignment

- UX Flows 1 and 4 cover Spending entry/allocation, history, current-month source and converted summaries, debts, settlement, disclosure, correction, and deletion under PRD FR-4 through FR-12.
- UX Flow 2 covers name-only USD Group creation, Group-local Participant setup, active-versus-archived lifecycle, archive eligibility, restoration, and history preservation under FR-2 and FR-3.
- UX Flow 3 covers password-only single-Administrator access, private ledger data, Sign out, and session loss under FR-1.
- The UX preserves all product exclusions: no Participant accounts, collaboration, repayments, extra share modes, manual rate retry, separate desktop IA, custom application JavaScript, or HTMX dependency.
- Final UX contracts operationalize the PRD NFRs through semantic native baseline, 320px support, 48px targets, focus/contrast/error association, retained validation input, and distinct archived/stale/provisional/unavailable states.

### Architecture Alignment

- AD-11 supports Askama semantic HTML, native links/forms, pinned self-hosted HTMX plus official `response-targets`, strict CSP, immutable asset treatment, and full native fallback.
- AD-18 makes final `UX-*` contracts binding on affected routes, templates, projections, CSS, enhancements, and web acceptance tests; `epics.md` assigns route-specific UX IDs to relevant stories.
- AD-3 through AD-10 provide exact allocation, historical identity, calculation snapshots, deterministic rate/settlement behavior, protected mutation outcomes, and no-partial results required by UX states.
- AD-12 through AD-16 provide the bounded responsive online runtime: safe errors, session-free static/probe routes, request admission/timeouts, provider-independent readiness/CRUD, and testable interfaces.

### Alignment Issues

No blocking PRD-to-UX or UX-to-Architecture contradiction was found.

### Warnings

- Authority is deliberately layered: `specs/design.md` and accepted ADRs govern product, security, accounting, and architecture; final UX contracts govern visual and interaction detail within that envelope. Any divergence is a stop condition.
- Physical route/template layout, content-fit responsive thresholds, and exact verified HTMX asset digests remain implementation decisions. The architecture bounds each; owning stories must close them before relevant route/enhancement acceptance.

## Epic Quality Review: 2026-08-12 Reassessment

### Epic Structure

The plan has five Administrator outcomes rather than detached technical milestones:

1. Secure local operation and private access.
2. Group and active-Participant organization.
3. Exact Spending recording and maintenance.
4. Current-month spending understanding.
5. All-time debts, settlement, and rate-dependent Participant retirement.

The order is sound. Epic 2 uses the secure shell from Epic 1; Epic 3 uses Groups and active Participants; Epic 4 consumes committed Spendings; Epic 5 reuses completed snapshots and rate capability. Persistence arrives with first consumers: runtime in 1.2, Groups in 2.1, Participants in 2.3, and Spending aggregates in 3.1. No starter template is specified. The brownfield plan identifies retained/replaced paths and explicitly forbids compatibility shims.

### Critical Violations

None found.

### Major Issues

None found.

### Minor Concerns

#### m1: Acceptance criteria remain highly compound

Many stories necessarily join a route outcome with security, accounting, persistence, concurrency, UX, and validation evidence. Assignment Packets cap ten complex stories to one primary route/use case and a 3-7 day estimate, substantially reducing the prior implementation-context risk, but review of individual BDD chains will remain demanding.

**Recommendation:** Preserve the current packet boundaries and Final-Evidence Ledger. During implementation, keep route-specific behavior in each story and record recurring validation proof through the shared checklist rather than duplicating generic claims.

### Dependency Assessment

- The former Epic 1 forward-dependency defect is resolved: Story 1.4 owns anonymous Login token issuance/expiry/capacity/cleanup; Story 1.5 owns Login reservation; Story 1.6 reuses the established path for Sign out; Story 1.7 extends it to authenticated forms and the route-neutral extractor.
- Stories 3.1 and 3.2 are complete vertical create-Spending outcomes. Story 3.1 solely owns aggregate persistence; Story 3.2 reuses it without duplicate ownership.
- The Final-Evidence Ledger correctly separates introduction from completion for `SPEC-FR29`, `SPEC-FR30`, `SPEC-FR40`, `SPEC-FR103`, and `SPEC-FR104`; earlier stories do not require later work to satisfy their stated outcomes.
- Story 1.10 is explicitly a pre-production operations gate, not a Phase 4 application assignment, resolving the prior vendor/substrate ambiguity.
- No circular epic dependency or story-level forward execution dependency was found.

### Best-Practice Checklist

| Epic | User value | Independent ordering | Story sizing | No forward dependency | Just-in-time persistence | Clear/testable AC | Traceability |
|---|---|---|---|---|---|---|---|
| Epic 1 | Pass | Pass | Packets bound complex work | Pass | Pass | Pass, compound | Pass |
| Epic 2 | Pass | Pass | Packets bound complex work | Pass | Pass | Pass, compound | Pass |
| Epic 3 | Pass | Pass | Packets bound complex work | Pass | Pass | Pass, compound | Pass |
| Epic 4 | Pass | Pass | Packets bound complex work | Pass | Pass | Pass, compound | Pass |
| Epic 5 | Pass | Pass | Packets bound complex work | Pass | Pass | Pass, compound | Pass |

### Quality Verdict

The current epic/story plan meets the readiness standard. It has no technical-only delivery epic, no unresolved forward dependency, no circular dependency, no duplicate Spending persistence owner, and no unbounded Phase 4 edge implementation story. The remaining concern is review granularity, not a planning blocker.

## Summary and Recommendations: 2026-08-12 Final Reassessment

### Overall Readiness Status

**READY FOR IMPLEMENTATION ASSIGNMENT**

The planning artifacts are complete and aligned. All four artifact families are present without duplicate-format ambiguity. The current plan covers all 12 PRD functional requirements, preserves the 20 explicit PRD non-functional requirements and 29 addendum constraint families through decomposed `SPEC-*` requirements, aligns final UX contracts with architecture, and orders five user-value epics without execution-blocking forward dependencies.

### Critical Issues Requiring Immediate Action

None.

### Recommended Next Steps

1. Assign implementation in epic/story order and enforce the Assignment Packet boundary before beginning each complex story.
2. Keep the Final-Evidence Ledger current so shared requirements are not marked complete before their named final-evidence story.
3. Close route/template layout, content-fit responsive thresholds, and verified HTMX digests in the owning implementation stories before acceptance.

### Final Note

This final reassessment identified **1 non-blocking documentation/review-granularity concern across 1 category**. It found **0 missing PRD FRs, 0 PRD/UX/Architecture contradictions, 0 critical planning defects, 0 technical-only epics, 0 circular dependencies, and 0 unresolved forward dependencies**.

**Assessment date:** 2026-08-12

**Assessor:** Kilo, Product Requirements and Traceability Review

## UX Alignment Assessment

### UX Document Status

**Found and final.** The assessment uses:

- `ux-designs/ux-debtor-2026-08-10/DESIGN.md` as the visual design contract.
- `ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md` as the structural, behavioral, state, and interaction contract.

Both documents explicitly cite the PRD and addendum as sources. Architecture AD-18 establishes their authority within the normative product/security/accounting envelope and makes the stable `UX-*` registry binding on affected web stories and tests.

### UX to PRD Alignment

- PRD UJ-1 is preserved as UX Flow 1: sign in, select Group, create and exactly allocate a Spending, return to Transactions, review Source and Group Currency monthly totals, and calculate complete all-time Balances and Settlement Transfers with conversion disclosure and no-partial failure behavior.
- PRD UJ-2 is preserved as UX Flow 2: create-by-name, default USD, open Manage, configure Group and Participants, enable Add Spending after an active Participant exists, archive only from exact-zero Historical eligibility, restore contextually, and preserve history.
- FR-1 is separately covered by UX Flow 3, including password-only login, private ledger access, sign-out, session loss, and secret-safe failures.
- FR-6 receives a dedicated correction/deletion Flow 4 with fixed 25-item history, complete details, Exact-on-edit, full-page confirmation, canonical return focus, and atomic no-partial outcomes.
- The UX retains the PRD's single responsive web experience, semantic native baseline, optional pinned HTMX enhancement, no custom JavaScript, browser support down to 320 CSS pixels, keyboard operation, contrast/focus/error association, retained values, and visibly distinct archived/stale/provisional/unavailable states.
- Product boundaries remain aligned: one Administrator; Participants are accounting identities; no collaboration/account/tenant model; no repayment state; no arbitrary timeframe analytics; one Payer; Proportional/Exact only; no manual rate Retry, infinite scroll, offline queue, separate desktop IA, or decorative motion.
- UX-specific elaborations such as 48-by-48 targets, stable focus destinations, server-rendered confirmations, one stable status region, 520px internally scrolling allocation geometry, latest-input-wins enhanced Preview, and Editorial Contrast styling refine the experience without contradicting PRD behavior.

### UX to Architecture Alignment

- AD-11 supplies the exact rendering substrate: Askama semantic server HTML, native links/forms, self-hosted HTMX 2.0.10 and official `response-targets` 2.0.4 only, immutable verified assets, native fallback, restrictive CSP, and no custom/inline JavaScript.
- AD-18 explicitly binds every web route, template, projection, CSS rule, native interaction, enhancement, and web test to the final UX contracts and requires route-specific geometry, focus, announcement, state, zoom, and parity evidence.
- AD-10 and AD-14 support the UX's definitive mutation states, retained pre-dispatch validation, single-use token conflicts, bounded admission, and no generic post-dispatch cancellation.
- AD-3 through AD-9 support exact financial display, deterministic allocation/quantization/settlement, complete snapshots, historical identity, conversion context, stale/provisional disclosure, and no-partial derived results.
- AD-12 through AD-15 support responsive online delivery, secure proxy/session behavior, bounded body/concurrency/timeouts, safe errors, session-free static/probe routes, and provider-independent readiness/CRUD.
- The architecture capability map assigns native/enhanced HTML to web routes, Askama templates, rendering projections, CSS, static assets, and web acceptance tests under AD-11/AD-18.
- Performance and responsiveness have bounded architectural support: 64 user/4 login/4 probe permits, 30-second ordinary reads/login, 90-second Debts, bounded provider calls and caches, and 320px/400%-zoom acceptance criteria in stories. UX does not impose an unsupported real-time or client-runtime dependency.

### Alignment Issues

No blocking PRD-to-UX or UX-to-Architecture contradiction was found.

### Warnings

- Source authority is distributed. `specs/design.md` remains normative; accepted ADRs govern architecture; final UX governs visual/interaction detail only within that envelope. Any later divergence must stop implementation rather than be resolved by silently selecting one artifact.
- Architecture intentionally defers exact route inventory, template/source layout, responsive breakpoint, and HTMX asset digests to implementation. These are bounded decisions, not missing UX support, but each must be closed by its owning story before the affected route or enhancement ships.
- Mockups and `.working` assets are illustrative only. Implementers must use `DESIGN.md`, `EXPERIENCE.md`, and stable `UX-*` IDs rather than copying superseded artifact behavior or CSS.

## Epic Quality Review

### Overall Structure

The five-epic sequence is logically ordered:

1. Securely operate and access the application.
2. Create Groups and active Participant identities.
3. Record and maintain exact Spendings.
4. Understand current-month spending.
5. Calculate all-time debts and perform rate-dependent Participant lifecycle work.

All epic titles and goals describe Administrator/operator outcomes rather than database, API, or model milestones. Epic 1 is infrastructure-heavy, but its stated outcome is runnable private access with a secure usable shell; it is not merely “set up infrastructure.” Epic 2 can deliver Group setup without future Spendings or debts, Epic 3 uses only prior Group/Participant capability, Epic 4 uses completed Spendings, and Epic 5 uses completed snapshots/rate capability. No circular epic dependency was found.

### Critical Violations

No critical violation was found. There is no purely technical epic, circular dependency, or story that directly requires a later story in order to satisfy its own narrowly stated acceptance criteria.

### Major Issues

#### M1: Stories 3.1 and 3.2 are horizontal Preview slices, not independently complete user outcomes

**Evidence:**

- Story 3.1 lets the Administrator preview Proportional allocation but cannot save the Spending until Story 3.3.
- Story 3.2 lets the Administrator preview Exact Shares but likewise cannot save the Spending until Story 3.3.
- Story 3.3 supplies the first end-to-end “record a Spending” outcome for both modes.

**Impact:** The two Preview stories can be implemented and tested but do not independently deliver the stated product job. They leave UI and domain behavior that the user cannot use to complete the transaction, contrary to the workflow rule that every story leave a meaningful runnable vertical increment.

**Recommendation:** Recut Epic 3 around end-to-end outcomes. Preferred options are: make one story deliver complete Proportional create (Preview plus commit), make the next deliver complete Exact create using the established persistence path, then retain browse/edit/delete stories; or combine both Preview modes and commit into one implementation story and split persistence/corruption/concurrency verification into tightly scoped enabling tasks within that story rather than separate user stories.

#### M2: Multiple stories exceed a realistic single-developer context

**Evidence:**

- Story 1.2 combines workspace/toolchain setup, configuration, SQLite connection/migration/durability policy, root composition, provider-independent startup, dependency pinning, SQLx metadata, schema-constraint policy, and brownfield removal.
- Story 1.5 combines trusted-proxy parsing, body/field/CSRF/token admission, rate limiting, Argon2 concurrency, durable session promotion, capacity failure, responsive UX, focus, status, and history/cache behavior.
- Story 1.7 combines token-store design, capacity/expiry, strict extraction, dispatch boundaries, concurrency races, every rejection class, cleanup, cross-route contract tests, and responsive conflict/validation/pending UX.
- Story 2.1 combines first Group vertical slice with mutation executor semantics, write gate, epoch ownership, pre-dispatch deadline, task-failure outcome protocol, schema constraints, hostile-input testing, shutdown-with-real-mutation proof, and full responsive UX.
- Story 3.3 combines server revalidation, both allocation modes, canonical persistence, corruption handling, complete schema/migration constraints, Group deletion restriction, SQLx metadata, reviewed-input binding/revisions, dispatch semantics, and post-commit UX.
- Stories 4.2, 4.3, 5.1, and 5.4 each combine substantial financial algorithms, persistence snapshots, provider/cache behavior, failure taxonomy, concurrency/determinism, complete UI state handling, and extensive test obligations.

**Impact:** These stories are likely to overflow one implementation context, complicate review, and produce partial completion despite their detailed criteria. Their breadth hides sequencing and ownership risk even though each criterion is individually testable.

**Recommendation:** Split by runnable behavior and owning boundary while preserving vertical value. A story should have one primary route/use case and its necessary domain/persistence/web evidence. Move reusable operational primitives into the first consumer but avoid simultaneously completing unrelated lifecycle, deployment, schema-governance, and full UX matrices in that same story. At minimum, re-estimate and recut Stories 1.2, 1.5, 1.7, 2.1, 3.3, 4.2, 4.3, 5.1, and 5.4 before implementation assignment.

#### M3: Story 1.10 requires a deployable HTTPS edge without selecting its implementation substrate

**Evidence:** Story 1.10 requires reproducible verification for TLS/HTTP3, forwarding sanitation, early-data rejection, body limits, backend reuse/timeouts, fallback, and staged `Alt-Svc`. Architecture explicitly defers reverse-proxy vendor and vendor-specific configuration.

**Impact:** The behavioral edge contract is clear, but the story cannot produce concrete deployable configuration and executable verification without choosing a proxy/vendor or narrowing its deliverable to vendor-neutral acceptance documentation. Different edge products expose materially different HTTP/3, early-data, forwarding, timeout, and telemetry controls.

**Recommendation:** Before Story 1.10 enters implementation, select and record the edge product/version and deployment verification environment, or explicitly reclassify the story as pre-production operations work outside Phase 4 application implementation. Synchronize the architecture deferred-decision table and story acceptance criteria with that decision.

#### M4: Cross-epic completion claims depend on later evidence and can mislead status tracking

**Evidence:**

- Epic 1 establishes lifecycle primitives, but real-mutation `SPEC-FR103..SPEC-FR104` evidence completes in Story 2.1.
- Story 1.3 points to Story 1.9 for authenticated-runtime shutdown and Story 2.1 for real-mutation evidence.
- Epic 2/Story 2.5 defer Spending-backed Group deletion restriction to Story 3.3.
- Story 2.4 defers current-name historical projection proof to Story 3.4.
- Participant lifecycle requirements begun in Epic 2 complete in Epic 5 because archive depends on debts/rates.

**Impact:** These are not executable forward dependencies for the earlier narrow stories, but they create split acceptance ownership. A tracker could mark a shared requirement or epic complete before its final evidence exists.

**Recommendation:** Keep each earlier story's `Requirements` list limited to the evidence it actually closes. Represent later proof as an explicit dependency/verification ledger with status owned by the later story. Do not mark `SPEC-FR29`, `SPEC-FR30`, `SPEC-FR40`, `SPEC-FR103`, or `SPEC-FR104` complete at the earlier partial owner. Consider renaming Epic 2 to make “active Participant setup” explicit if full Participant lifecycle remains intentionally owned by Epic 5.

### Minor Concerns

#### m1: Acceptance criteria are testable but excessively compound

Most scenarios use valid Given/When/Then structure and include unhappy paths, bounds, statuses, side-effect exclusions, responsive states, and test evidence. However, many `Then`/`And` chains assert several independent concerns across layers. A single failed criterion can obscure whether the story's user outcome, architecture rule, UX rule, or verification gate is incomplete.

**Recommendation:** Keep BDD scenarios outcome-focused and move repeated cross-cutting validation commands, brownfield disposition, and generic contract evidence into a story Definition of Done/checklist referenced by stable IDs. Retain route-specific criteria in the story.

#### m2: Epic 1 contains several operator/technical stories under one user-facing epic

Stories 1.1-1.3, 1.8-1.10 primarily serve operation, security, and deployment rather than an in-app end-user interaction. They still provide real Administrator/operator value and therefore do not constitute a technical-epic violation, but their concentration makes Epic 1 unusually broad.

**Recommendation:** Keep the epic only if the Administrator and operator are intentionally the same product actor. Otherwise separate pre-production edge rollout from the runnable secure-access outcome without creating a technology-only milestone.

### Dependency Analysis

- Within Epic 1, the order is valid: password validation precedes startup; startup precedes restart; login page precedes sign-in; authentication precedes authenticated session/logout; shared replay protection and admission build on those paths. Story 1.10 depends on prior trusted-client and mutation-timeout contracts, not future stories.
- Within Epic 2, Group creation precedes settings and Participant management; lifecycle follows established Group persistence. No story requires Epic 3 to perform its own stated behavior, although final Spending-backed deletion proof is deferred.
- Within Epic 3, Proportional Preview precedes Exact Preview and common commit; history follows persisted Spendings; edit/delete follow direct complete loads. The order is technically valid, but Preview independence is deficient as noted in M1.
- Within Epic 4, provider-independent source totals precede converted totals; stale/failure behavior extends the completed converted path. No forward dependency exists.
- Within Epic 5, Historical calculation precedes Current-mode reuse, Settlement, archive eligibility, and archived restoration. No forward dependency exists.
- Database/entity timing is sound: runtime/database foundation appears in Story 1.2; Group schema when Groups first appear in Story 2.1; Participant persistence in Story 2.3; Spending/allocation persistence in Story 3.3. No “create all future tables up front” instruction was found.
- No starter template is specified by Architecture, so no missing starter-template story exists.
- Brownfield handling is explicit in every story through retained/replaced/removed paths; compatibility shims are prohibited consistently.

### Best-Practice Checklist

| Epic | User value | Independent from future epics | Story sizing | No forward execution dependency | Just-in-time schema | Clear/testable AC | Traceability |
|---|---|---|---|---|---|---|---|
| Epic 1 | Pass | Pass, with later shared-evidence closure | Fail: several oversized stories | Pass with tracking warning | Pass | Pass, compound | Pass |
| Epic 2 | Pass | Pass for stated active setup/lifecycle subset | Fail: Stories 2.1 and 2.5 are oversized | Pass with deferred proof warning | Pass | Pass, compound | Pass |
| Epic 3 | Pass | Pass | Fail: horizontal Preview slices and oversized commit | Pass | Pass | Pass, compound | Pass |
| Epic 4 | Pass | Pass | Concern: Stories 4.2-4.3 are large | Pass | Pass | Pass, compound | Pass |
| Epic 5 | Pass | Pass | Concern: Stories 5.1 and 5.4 are large | Pass | Pass | Pass, compound | Pass |

### Quality Verdict

The epic architecture and requirement traceability are strong, but story decomposition is not yet consistently implementation-ready. The major issues are recutting the Epic 3 Preview/commit path into independently valuable increments, reducing oversized stories, resolving Story 1.10's deployment substrate, and making deferred acceptance ownership impossible to misreport.

## Summary and Recommendations

### Overall Readiness Status

**SUPERSEDED: READY FOR IMPLEMENTATION ASSIGNMENT**

The product contract is complete enough to plan implementation: all required artifacts exist, the PRD has no open questions, all 12 PRD FRs are covered, UX aligns with PRD and Architecture, epic ordering is coherent, and no circular or direct forward execution dependency exists.

This original assessment is superseded by the approved Sprint Change Proposal and the later Correct Course correction. Phase 4 may begin from the corrected story plan; the remaining edge implementation choice stays explicitly outside Phase 4 application assignment.

### Critical Issues Requiring Immediate Action

No critical requirements or architecture gap was found. Four major planning issues require resolution before implementation assignment:

1. **Recut Epic 3 Stories 3.1-3.3 into vertical user outcomes.** Preview-only stories do not let the Administrator record a Spending. Each accepted story must end with a usable committed behavior or be explicitly treated as a non-story task within a vertical story.
2. **Split or formally re-estimate oversized stories.** Stories 1.2, 1.5, 1.7, 2.1, 3.3, 4.2, 4.3, 5.1, and 5.4 currently combine too many application, domain, infrastructure, web, UX, concurrency, migration, and verification obligations for reliable single-context execution.
3. **Resolve Story 1.10's deployment substrate.** Select the reverse proxy/product version and verification environment, or move vendor-neutral edge acceptance to a pre-production operations plan outside Phase 4 application stories.
4. **Make deferred acceptance ownership explicit.** Track `SPEC-FR29`, `SPEC-FR30`, `SPEC-FR40`, `SPEC-FR103`, and `SPEC-FR104` at their final-evidence stories; partial earlier owners must not close them.

### Recommended Next Steps

1. Run story correction on `epics.md`, preserving the five coherent epic boundaries unless recutting proves a real epic-level dependency.
2. Rebuild Epic 3 first around complete Proportional and Exact Spending creation outcomes, then revalidate FR-4/FR-5 coverage and dependency order.
3. Recut the nine oversized stories into smaller route/use-case-centered vertical increments, preserving just-in-time schema creation and first-consumer infrastructure ownership.
4. Add an explicit requirement-evidence status table that distinguishes `introduced`, `partially evidenced`, and `complete` for cross-story requirements.
5. Decide the Story 1.10 edge implementation boundary and synchronize Architecture, story criteria, deployment artifacts, and verification ownership.
6. Rerun implementation readiness after the revised epics/stories document is final.

### Final Note

This assessment identified **6 quality issues across 3 categories**: four major story-planning issues, two minor structural concerns, and three non-blocking UX/authority warnings. It found **0 missing PRD FRs, 0 critical violations, and 0 PRD/UX/Architecture contradictions**.

The project vision and technical contract are strong. Address the four major planning issues before Phase 4 implementation so that the existing depth becomes executable rather than overwhelming individual stories.

**Assessor:** Kilo, Product Requirements and Traceability Review

## Document Discovery: 2026-08-12 Reassessment

### Confirmed Artifact Selection

- PRD packet: `prds/prd-debtor-2026-08-10/prd.md` and `addendum.md`.
- Architecture: `architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md`.
- Epics and Stories: `epics.md`.
- UX packet: `ux-designs/ux-debtor-2026-08-10/DESIGN.md` and `EXPERIENCE.md`.

### Discovery Result

- All required artifact families are present.
- No whole-versus-sharded duplicate conflict was found.
- Reconciliation, review, and validation files remain supporting context and are not selected as primary source artifacts.

## PRD Analysis: 2026-08-12 Reassessment

### Functional Requirements

- **PRD-FR-1:** Password-gated access: one configured-password Administrator may sign in, stay authenticated in an active session, and sign out; there are no usernames, registration, Participant login, or multi-user authorization. Anonymous data access is denied, restart ends sessions, unsafe requests require CSRF, and unsafe forms require separately bounded, expiring, session-bound single-use tokens.
- **PRD-FR-2:** Group lifecycle: Groups can be created, edited, archived, restored, and deleted only when they contain no Spendings; creation asks only for a valid name, initializes USD, and opens Manage. Archived Groups remain readable but immutable, with contextual restoration views.
- **PRD-FR-3:** Group-owned Participants: Group-local Participants can be added, edited, archived, and restored; allocation uses active Group-owned identities only. Archive requires a revalidated all-time Historical zero balance and eligible quote context, while restore has no balance check.
- **PRD-FR-4:** Record a Spending: a valid description, date, category, positive total, supported source currency, one Payer, and Proportional or Exact Shares can be submitted with stated defaults and inline retained-value validation.
- **PRD-FR-5:** Exact allocation: the Payer and Shares each exactly conserve the source-currency total under positive, bounded, minor-unit-valid values. Proportional and Exact modes have specified initialization, validation, rounding, and archived-role retention rules.
- **PRD-FR-6:** Review and maintain history: active Groups permit paginated browsing, direct detail, edit, and deletion of Spendings; history survives archival, resolves current names, edits use stored Exact allocations, and every change is atomic.
- **PRD-FR-7:** Source Currency summary: current UTC-month Group and per-Payer paid totals are grouped by original Source Currency without conversion.
- **PRD-FR-8:** Group Currency summary: the same current-month totals convert by date-sensitive context, exact accumulation and deterministic quantization; future/stale cases disclose warnings and a quote/calculation failure makes only the converted section retryably unavailable.
- **PRD-FR-9:** Select conversion mode: all-time Balances support default Historical and non-persistent Current modes.
- **PRD-FR-10:** Exact Balances: per-Participant Group Currency Balances preserve exact zero sum after quantization and return no partial calculation on arithmetic or conversion failure.
- **PRD-FR-11:** Deterministic Settlement Transfers: positive, deterministic, pair-unique, complete transfers settle Balances with at most `n - 1` transfers, without claiming global minimality.
- **PRD-FR-12:** Calculation disclosure and failure isolation: debts disclose mode, time, Group Currency, rate evidence, and stale/provisional warnings; missing quotes are retryable and do not block ledger management.

**Total functional requirements: 12.**

### Non-Functional Requirements

- **PRD-NFR-1 to NFR-7 (UX):** semantic native server-rendered baseline; optional pinned HTMX only; responsive one-web-experience operation through 320 CSS pixels; current stable browser support; keyboard/labels/focus/contrast/error association; retained inline validation; visibly distinct archived and conversion states.
- **PRD-NFR-8 to NFR-11 (correctness):** exact decimal/minor-unit money; historical readability; atomic Spending ownership and eligibility validation; internally consistent Spending and debt snapshots.
- **PRD-NFR-12 to NFR-16 (security):** authenticated, CSRF-protected unsafe requests; server-side single-use replay suppression; throttled authentication and secure production cookies; safe diagnostics; no caching of authenticated pages.
- **PRD-NFR-17 to NFR-20 (availability):** provider-independent startup/readiness/CRUD; bounded resources and waits; definitive post-dispatch mutation result; admission-stopping, recoverable shutdown.

**Total explicit non-functional requirements: 20.**

### Additional Requirements

The technical addendum defines 29 mandatory acceptance families: product boundaries; layering and port purity; input ownership; exact canonical money; deterministic allocation/IDs/settlement; SQLite durability, transactions, structural constraints, snapshots, pagination, and checked SQLx; safe failures/logging; precision-preserving rate lookup, cache/single-flight, and bounded provider behavior; native-first HTTP/forms/status/dispatch; password/session/CSRF/proxy/header rules; admission, probes, shutdown, HTTPS edge rollout, and local operation; source-first maintenance; pinned toolchain/workspace separation; and layer-owned testing and validation.

### PRD Completeness Assessment

The PRD package is final, complete, internally reconciled, and testable. It declares `specs/design.md` normative, requires reconciliation rather than silent conflict resolution, and has no open questions or unconfirmed assumptions. The traceability review must therefore test story coverage against all 12 FRs plus the UX, NFR, and addendum constraint families rather than against headline capabilities alone.

## Epic Coverage Validation: 2026-08-12 Reassessment

### Epic FR Coverage Extracted

- **PRD-FR-1:** Epic 1, Stories 1.1-1.10; Story 2.1 provides final real-mutation lifecycle evidence.
- **PRD-FR-2:** Epic 2, Stories 2.1, 2.2, and 2.5; Story 3.1 provides final Spending-backed deletion evidence.
- **PRD-FR-3:** Epic 2 Stories 2.3-2.4 and Epic 5 Stories 5.4-5.5; Story 3.3 proves current-name historical views.
- **PRD-FR-4:** Epic 3, Stories 3.1-3.2.
- **PRD-FR-5:** Epic 3, Stories 3.1, 3.2, and 3.4.
- **PRD-FR-6:** Epic 3, Stories 3.3-3.5.
- **PRD-FR-7:** Epic 4, Story 4.1.
- **PRD-FR-8:** Epic 4, Stories 4.2-4.3.
- **PRD-FR-9:** Epic 5, Stories 5.1-5.2.
- **PRD-FR-10:** Epic 5, Stories 5.1-5.2.
- **PRD-FR-11:** Epic 5, Story 5.3.
- **PRD-FR-12:** Epic 4 Stories 4.2-4.3 and Epic 5 Stories 5.1-5.2.

### FR Coverage Analysis

| PRD FR | Requirement | Coverage | Status |
| --- | --- | --- | --- |
| 1 | Password-gated access | Epic 1; final mutation proof 2.1 | Covered |
| 2 | Group lifecycle | Epic 2; final deletion proof 3.1 | Covered |
| 3 | Group-owned Participants | Epics 2, 3, and 5 | Covered |
| 4 | Record a Spending | Stories 3.1-3.2 | Covered |
| 5 | Exact allocation | Stories 3.1, 3.2, 3.4 | Covered |
| 6 | Spending history maintenance | Stories 3.3-3.5 | Covered |
| 7 | Source Currency summary | Story 4.1 | Covered |
| 8 | Group Currency summary | Stories 4.2-4.3 | Covered |
| 9 | Conversion mode | Stories 5.1-5.2 | Covered |
| 10 | Exact Balances | Stories 5.1-5.2 | Covered |
| 11 | Settlement Transfers | Story 5.3 | Covered |
| 12 | Disclosure and failure isolation | Stories 4.2-4.3, 5.1-5.2 | Covered |

### Missing Requirements

No PRD functional requirement is missing. The `SPEC-FR1..SPEC-FR105` and `SPEC-NFR1..SPEC-NFR34` registers add source-qualified technical traceability rather than unapproved product scope. The Final-Evidence Ledger prevents premature completion of shared requirements `SPEC-FR29`, `SPEC-FR30`, `SPEC-FR40`, `SPEC-FR103`, and `SPEC-FR104`.

### Coverage Statistics

- Total PRD FRs: 12
- PRD FRs covered in epics: 12
- Missing PRD FRs: 0
- Coverage: 100%

## Correct Course Revalidation

**Revalidation time:** 2026-08-12T11:41:58+06:00

The approved Sprint Change Proposal corrected the two planning blockers in `epics.md` without changing product behavior, architecture, UX, or MVP scope:

1. Story 3.1 is now the sole first-consumer owner of reviewed-input validation, complete create-Spending aggregate persistence, canonical hydration, and the first persisted-Spending proof for `SPEC-FR29`. Story 3.2 delivers Exact creation by reusing that path. The duplicate integrity milestone was removed; history, edit, and delete are now Stories 3.3, 3.4, and 3.5, with crosswalk and UX-owner references synchronized.
2. The prior generic sizing promise is now concrete Assignment Packets for Stories 1.2, 1.4, 1.5, 1.7, 2.1, 3.1, 3.2, 4.2, 4.3, 5.1, and 5.4. Each has a one-developer boundary, a 3-7 day estimate, a mandatory split threshold, and an ordered implementation checklist.
3. Story 1.4 now owns anonymous Login-token issuance, expiry, capacity, and cleanup; Story 1.5 owns Login-token reservation; Story 1.6 reuses the completed path for Sign out; Story 1.7 is limited to authenticated/general-route extension and cross-route verification. Login and Sign out therefore have no forward dependency.

The Final-Evidence Ledger remains authoritative for `SPEC-FR29`, `SPEC-FR30`, `SPEC-FR40`, `SPEC-FR103`, and `SPEC-FR104`. Story 1.10 remains a pre-production operations gate rather than a Phase 4 application assignment.

### Corrected Readiness Status

**READY FOR IMPLEMENTATION ASSIGNMENT**

All 12 PRD functional requirements remain covered. No PRD/architecture/UX contradiction, forward dependency, duplicate Spending-persistence owner, or unbounded pre-assignment story scope remains. Each implementation assignment must stay within its approved packet; any expansion beyond that packet requires a split and targeted revalidation before work starts.
**Assessment date:** 2026-08-12

## Revalidation Note

The full PRD, technical addendum, architecture spine, final UX contracts, normative design contract, and all five epics with their stories were reread on 2026-08-12. This revalidation confirms the findings and verdict above: all 12 PRD functional requirements have traceable story coverage, but Phase 4 must not begin from the current story plan unchanged.

The blocking planning work remains: recut Preview-only Spending stories into end-to-end outcomes, split or formally re-estimate oversized stories, select or defer the concrete HTTPS edge substrate for Story 1.10, and track final evidence for requirements shared across epics.

## Revalidation Run: Document Discovery And PRD Analysis

**Run time:** 2026-08-12T11:35:40+06:00

The document inventory was reconfirmed with the Administrator: the PRD is the sharded `prds/prd-debtor-2026-08-10/` packet, Architecture is `architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md`, UX is the sharded `ux-designs/ux-debtor-2026-08-10/` packet, and the stories are in `epics.md`. No whole-versus-sharded duplicate conflict exists. Primary folders lack `index.md`, but their primary documents are unambiguous.

The complete `prd.md` and `addendum.md` packet was reread, with its reconciliation, review, and decision records. It retains 12 functional requirements, 20 explicit cross-cutting non-functional requirements, and 29 grouped additional technical acceptance families. It declares `specs/design.md` normative, requires a stop-and-reconcile response to divergence, has no open questions or unconfirmed assumptions, and remains internally consistent with its reconciliation evidence.

## Revalidation Run: Epic Coverage

All 12 product functional requirements have explicit traceability in the complete `epics.md` crosswalk and story acceptance criteria:

| PRD FR | Final story ownership |
| --- | --- |
| `PRD-FR-1` Password-gated access | Epic 1, Stories 1.1-1.10; real-mutation lifecycle evidence: Story 2.1 |
| `PRD-FR-2` Group lifecycle | Epic 2, Stories 2.1-2.2 and 2.5; Spending-backed deletion proof: Story 3.1 |
| `PRD-FR-3` Group-owned Participants | Stories 2.3-2.4 and 5.4-5.5; historical-name projection: Story 3.4 |
| `PRD-FR-4` Record a Spending | Stories 3.1-3.3 |
| `PRD-FR-5` Exact allocation | Stories 3.1-3.3 and 3.5 |
| `PRD-FR-6` Review and maintain history | Stories 3.4-3.6 |
| `PRD-FR-7` Source Currency summary | Story 4.1 |
| `PRD-FR-8` Group Currency summary | Stories 4.2-4.3 |
| `PRD-FR-9` Select conversion mode | Stories 5.1-5.2 |
| `PRD-FR-10` Exact Balances | Stories 5.1-5.2 |
| `PRD-FR-11` Deterministic Settlement Transfers | Story 5.3 |
| `PRD-FR-12` Calculation disclosure and failure isolation | Stories 4.2-4.3 and 5.1-5.2 |

**Coverage:** 12 of 12 PRD functional requirements, 100%. No PRD FR is missing. The plan also maintains a source-qualified decomposition of `SPEC-FR1..SPEC-FR105` and `SPEC-NFR1..SPEC-NFR34`; these are traceability requirements, not unapproved scope.

The prior Preview-only weakness is resolved: Stories 3.1 and 3.2 now each deliver preview plus reviewed approval and atomic committed Spending creation. Story 1.10 is explicitly a pre-production operations gate, not a Phase 4 application story. The Final-Evidence Ledger now prevents premature closure of `SPEC-FR29`, `SPEC-FR30`, `SPEC-FR40`, `SPEC-FR103`, and `SPEC-FR104`.

## Revalidation Run: UX Alignment

`ux-designs/ux-debtor-2026-08-10/DESIGN.md` and `EXPERIENCE.md` are present and final. They define a complete responsive server-rendered UX: Group-centered information architecture, native link/form completion paths, optional pinned HTMX and official `response-targets`, the focused Spending form, native Preview/review/approval, transparent enhanced-preview parity, lifecycle confirmations, state/failure presentation, and accessibility criteria through 320 CSS pixels and 400% zoom.

No PRD-to-UX or UX-to-architecture contradiction was found. UX preserves the single-Administrator boundary, Group-owned Participants, exact allocation, rate-failure isolation, archived history, and all visible state semantics. Architecture AD-11 provides the semantic Askama/native-first rendering, asset, CSP, and security-header substrate; AD-18 binds final UX contracts to each affected route, template, CSS rule, enhancement, and acceptance test. The epics' stable `UX-*` ownership matrix and story-level evidence clauses provide planned traceability.

**Warnings:** Source authority is deliberately layered: `specs/design.md` and accepted ADRs govern product/security/accounting architecture; the final UX spines govern visual and interaction detail within that envelope. Architecture leaves the physical route/template layout, responsive content-fit threshold, and fixed HTMX asset digests to the first implementing story. These are bounded implementation decisions, not alignment gaps; an unreconciled divergence must stop work.

## Revalidation Run: Epic Quality

### Structure and dependency results

The five epics remain outcome-oriented and ordered by runnable Administrator value: secure access, Group/active-Participant setup, Spending ledger, current-month understanding, then debts/settlement/rate-dependent Participant retirement. No circular or direct forward execution dependency was found. The revised Stories 3.1 and 3.2 are complete vertical Spending outcomes, while Story 1.10 is correctly scoped as a pre-production operations gate. The Final-Evidence Ledger is a material improvement: it accurately distinguishes enabling work from final proof for cross-epic requirements.

### Major findings

**M1 - Spending persistence has overlapping ownership.** Stories 3.1 and 3.2 each promise server revalidation and atomic persistence of a complete Spending, while Story 3.3, "Preserve Complete Spending Integrity," separately owns the same commit revalidation, transaction, canonical persistence, corruption, schema, and reviewed-input binding behavior. A story that follows already-committed Proportional and Exact flows cannot be the first owner of their required integrity. This risks two persistence paths, duplicated acceptance evidence, or a false claim that Stories 3.1/3.2 are independently complete.

**Remediation:** Make Story 3.1 the sole first-consumer owner of shared complete-aggregate persistence, transaction eligibility checks, canonical persistence, reviewed-input binding, and first Spending-backed Group-deletion proof. Make Story 3.2 reuse that established path for Exact creation. Distribute any remaining distinct integrity tests to their owning behavior, or replace Story 3.3 with a narrowly defined user-visible outcome that does not duplicate creation acceptance.

**M2 - The high-complexity stories have not yet been concretely recut or estimated.** `epics.md` recognizes Stories 1.2, 1.5, 1.7, 2.1, 3.3, 4.2, 4.3, 5.1, and 5.4 as high complexity, but delegates their estimate and pull-request checklist to a future pre-assignment action. This is not a completed planning artifact. Several still combine a primary route/use case with infrastructure, migrations, concurrency, operational lifecycle, full UX-state matrices, and cross-layer verification.

**Remediation:** Before any implementation assignment, record the promised one-developer estimate and ordered checklist for each named story, and split any story that exceeds one implementation context. Apply the M1 disposition first, so Story 3.3 is not estimated as a duplicated technical milestone.

### Minor concern

Acceptance criteria are highly specific and testable, but frequently compound route outcome, domain invariant, persistence rule, operational behavior, UX geometry, and test command expectations in the same scenario. Preserve route-specific acceptance conditions, but place repetitive cross-cutting validation in the referenced Definition of Done/checklist so reviewers can isolate incomplete behavior.

### Quality checklist

| Epic | User value | Independent ordering | Story sizing | No forward execution dependency | Just-in-time persistence | Testable ACs | Traceability |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Epic 1 | Pass | Pass | Needs work | Pass | Pass | Pass, compound | Pass |
| Epic 2 | Pass | Pass | Needs work | Pass | Pass | Pass, compound | Pass |
| Epic 3 | Pass | Pass | Needs work: M1/M2 | Pass | Needs ownership correction | Pass, compound | Pass |
| Epic 4 | Pass | Pass | Needs work | Pass | Pass | Pass, compound | Pass |
| Epic 5 | Pass | Pass | Needs work | Pass | Pass | Pass, compound | Pass |

## Final Revalidation Assessment

**Assessment time:** 2026-08-12T11:35:40+06:00

### Overall Readiness Status

**SUPERSEDED: READY FOR IMPLEMENTATION ASSIGNMENT**

The product, technical, architecture, UX, and traceability foundations are ready. All required artifact families exist; the PRD packet has no open questions; all 12 PRD functional requirements have explicit epic/story ownership; UX and Architecture are aligned; native fallback, accessibility, security, accounting, lifecycle, and rate-failure contracts are planned.

Phase 4 implementation assignment must not start from the current story plan. The remaining blockers are planning-quality defects, not missing product intent: duplicated Spending-persistence ownership across Stories 3.1-3.3 and an unfulfilled commitment to size/split high-complexity stories before assignment.

### Required Actions

1. Resolve Stories 3.1-3.3 ownership. Assign shared atomic Spending persistence and its first evidence to one first-consumer story, have Exact creation reuse it, and eliminate or repurpose the duplicate integrity story.
2. Record one-developer estimates and ordered implementation checklists for Stories 1.2, 1.5, 1.7, 2.1, 4.2, 4.3, 5.1, and 5.4, plus the recut Spending path. Split any work that exceeds one implementation context before it enters Phase 4.
3. Retain and use the Final-Evidence Ledger in sprint tracking. Do not mark `SPEC-FR29`, `SPEC-FR30`, `SPEC-FR40`, `SPEC-FR103`, or `SPEC-FR104` complete until their listed final-evidence story succeeds.
4. Preserve Story 1.10 as an operations gate. Select the reverse-proxy product, version, configuration, and verification environment before production rollout, rather than assigning vendor-specific edge work to Phase 4 application implementation.
5. Rerun readiness after the corrected `epics.md` is finalized.

### Final Note

This revalidation identified **2 major planning issues and 1 minor documentation concern**. It found **0 missing PRD functional requirements, 0 PRD/UX/Architecture contradictions, and 0 critical forward-dependency or technical-epic violations**. The plan is close, but it is not yet implementation-ready because duplicate ownership and unbounded story scope will undermine reliable execution.

**Assessor:** Kilo, Product Requirements and Traceability Review
