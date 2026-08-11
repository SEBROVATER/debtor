---
stepsCompleted:
  - step-01-document-discovery
  - step-02-prd-analysis
  - step-03-epic-coverage-validation
  - step-04-ux-alignment
  - step-05-epic-quality-review
  - step-06-final-assessment
overallReadinessStatus: NEEDS WORK
assessmentDate: 2026-08-11
assessor: Kilo Implementation Readiness Review
filesIncluded:
  - _bmad-output/planning-artifacts/prds/prd-debtor-2026-08-10/prd.md
  - _bmad-output/planning-artifacts/prds/prd-debtor-2026-08-10/addendum.md
  - _bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md
  - _bmad-output/planning-artifacts/epics.md
  - _bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/DESIGN.md
  - _bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md
  - _bmad-output/planning-artifacts/sprint-change-proposal-2026-08-11.md
---

# Implementation Readiness Assessment Report

**Date:** 2026-08-11
**Project:** debtor

## Document Inventory

### PRD

- `prds/prd-debtor-2026-08-10/prd.md` (21,142 bytes; modified 2026-08-11 11:22)
- `prds/prd-debtor-2026-08-10/addendum.md` (27,214 bytes; modified 2026-08-11 11:28)

### Architecture

- `architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md` (34,154 bytes; modified 2026-08-11 19:07)

### Epics And Stories

- `epics.md` (141,874 bytes; modified 2026-08-11 19:12)

### UX Design

- `ux-designs/ux-debtor-2026-08-10/DESIGN.md` (26,436 bytes; modified 2026-08-10 21:48)
- `ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md` (53,649 bytes; modified 2026-08-11 19:07)

### Supplemental Planning Input

- `sprint-change-proposal-2026-08-11.md` (20,061 bytes; modified 2026-08-11 19:56)

### Discovery Notes

- All required document categories were found.
- No whole-document versus `index.md`-based sharded duplicates were found.
- Foldered PRD, Architecture, and UX packages have no `index.md`; the canonical files listed above were explicitly confirmed.
- Reconciliation notes, review records, validation records, and memory logs are excluded as normative assessment inputs.

## PRD Analysis

### Functional Requirements

**FR1: Password-gated access.** The Administrator can sign in with one configured password, remain authenticated during an active session, and sign out. Debtor provides no username, registration, Participant login, or multi-user authorization.

- Anonymous visitors cannot view Groups or ledger data.
- Restarting Debtor ends existing authenticated sessions.
- Login and all state-changing actions reject requests that lack valid request protection.
- Every unsafe form has one bounded, expiring, session-bound single-use submission token in addition to CSRF. Anonymous tokens use a separate 4,096-token pool with one per session and ten-minute inactivity expiry; authenticated tokens use a 1,024-token pool with 32 per session and 30-minute absolute expiry. Exactly one request can reserve a token for dispatch; reservation is terminal regardless of outcome, and duplicate or invalid use returns a clear conflict state without a second mutation.

**FR2: Group lifecycle.** The Administrator can create, edit, archive, and restore a Group. A Group with no Spendings can be deleted with its unreferenced Participants; a Group with any Spending cannot be deleted.

- Group names are trimmed, non-empty, and no longer than 100 Unicode characters.
- Group creation asks only for the name, assigns `USD` as Group Currency, and opens the new Group in Manage so currency and Participants can be configured. Established Groups open in Summary.
- Active Group and Participant lists exclude archived records; separate contextual archived views provide restoration access.
- Archived Groups remain readable but expose no mutation controls.
- Direct attempts to mutate an archived Group are rejected without changing state.

**FR3: Group-owned Participants.** The Administrator can add, edit, archive, and restore Participants inside a Group. Each Participant belongs to exactly one Group and is created independently if the same person appears in another Group.

- There is no separate global Participant-management surface.
- New Payers and Shares can use only active Participants owned by the selected Group.
- Archived Participants remain visible in referenced history, Balances, and Settlement Transfers.
- Archive is available and allowed only when one immutable all-time Historical-mode ledger/time/quote context gives the Participant an exact zero Group Currency Balance and remains eligible at commit. If the ledger epoch, UTC date, quote eligibility, or required rates invalidate the attempt, archive remains blocked with retryable feedback and no state change. Rate evidence is not persisted, so a later attempt may observe a provider revision.
- Restore remains available without a Balance check when a Participant returns to the Group.
- Participant names are trimmed, non-empty, and no longer than 100 Unicode characters.
- Participant colors use normalized `#RRGGBB` form. New Participant forms suggest a varied valid color that the Administrator can change.

**FR4: Record a Spending.** The Administrator can create a Spending with a description, date, category, positive Total, Source Currency, exactly one Payer, and either Proportional or Exact Shares.

- Source Currency and Group Currency options are `USD`, `EUR`, `RUB`, `KGS`, `TRY`, `KZT`, `UZS`, `CNY`, `KRW`, `JPY`, `OMR`, and `TJS`.
- Category options and current display labels are `food`, `transport`, `housing`, `fun`, `shopping`, `bills`, `health`, and `other`.
- Descriptions are trimmed, non-empty, and no longer than 200 Unicode characters.
- Dates use strict `YYYY-MM-DD` form and cannot precede `2025-01-01`.
- Description and Total start empty; Source Currency defaults to Group Currency; date defaults to the current UTC date; Category has no default.
- Payer selection and Share editing use one Participant allocation table. No Payer is initially selected; selecting one assigns the full Total, and selecting a different row replaces the Payer.
- Proportional and Exact are the only Share modes.
- Submitted values remain present when validation fails, with errors shown inline.

**FR5: Exact allocation.** Every accepted Spending preserves the Total exactly in Source Currency minor units: the single Payer pays the Total and Shares independently sum to the Total.

- Totals and Payer/Share amounts must be positive, cannot exceed `999_999_999_999`, and must satisfy Source Currency precision; zero values, excess precision, duplicate Participants, or mismatched totals are rejected.
- Proportional mode initially selects every active Participant with weight `1`, permits deselection, and accepts positive weights no greater than `1,000,000` with at most six fractional digits. Preview and commit use one checked integer-ratio operation and assign residual units by descending remainder with ascending Participant ID ties.
- Exact mode initially selects every active Participant, divides total minor units by Participant count, assigns residual units in ascending Participant ID order, permits deselection and amount editing, and displays the remaining or excess difference until the selected Shares equal the Total.
- A Spending update may retain an archived Participant only in the same existing Payer or Share role; it cannot introduce or change that archived Participant's role.

**FR6: Review and maintain history.** The Administrator can browse, inspect, edit, and delete Spendings in an active Group.

- History is ordered newest first and presented in pages of 25.
- Spending details remain readable when their Group or Participants are archived and display each Participant's current name after a rename.
- Editing may correct a Spending's Source Currency under the same validation as creation; subsequent historical calculations use the corrected stored Source Currency.
- Input modes and proportional weights are not persisted; every edit opens Exact with the stored single Payer and Share amounts.
- Each successful Spending change is applied completely or not at all.

**FR7: Source Currency summary.** The Administrator can see the selected Group's current-month Spending Total and each Payer's paid total, grouped by original Source Currency.

- Source Currency totals remain available without exchange-rate conversion.
- Spendings outside the current UTC calendar month are excluded.

**FR8: Group Currency summary.** The Administrator can see the same current-month Group and per-Payer totals converted to the Group Currency using the historical rate for each Spending date.

- Future-dated Spendings use the latest current rate and are marked provisional.
- A context-matching fixed past-date historical quote may be used without an age limit; current fallback selects the latest prior current-class quote for the pair, and future fallback also matches the original requested date. A stale current or future quote may be used inclusively through seven UTC calendar days after its prior fetch date. Every stale result carries a warning.
- Converted values accumulate exactly per Payer and are quantized together to Group Currency minor units by truncation and descending remainder with ascending Participant ID ties; the Group total is their exact sum. If a required quote or checked conversion/aggregation/quantization is unavailable, the entire converted summary reports one retryable failure with no partial totals; Source Currency totals, history, and ledger mutations remain usable.

**FR9: Select conversion mode.** The Administrator can calculate all-time Balances in historical mode or current mode. Historical mode is the default and converts each Spending at its Spending date; current mode converts every Spending at the UTC calculation date and is not persisted.

**FR10: Exact Balances.** Debtor calculates one exact Group Currency Balance per Participant and preserves an exact zero sum after currency quantization.

- Completion order of exchange-rate requests cannot alter results or warnings.
- Arithmetic or conversion failure returns no partial Balances or Settlement Transfers.

**FR11: Deterministic Settlement Transfers.** Debtor presents positive, deterministic Settlement Transfers that settle every Balance.

- A Participant pair appears at most once.
- No more than `n - 1` Settlement Transfers are produced for `n` Participants.
- Debtor does not claim globally minimal transfer count.

**FR12: Calculation disclosure and failure isolation.** The debts view identifies the conversion mode, calculation time, Group Currency, unique rates used, and stale or provisional warnings.

- If a required quote is unavailable without a valid stale fallback, the debts view reports a retryable failure.
- Exchange-rate failure never prevents Group, Participant, or Spending management.

**Total FRs: 12.**

### Non-Functional Requirements

**NFR1:** Core behavior must work through semantic server-rendered HTML and valid native links/forms. Pinned self-hosted HTMX may progressively enhance those interactions; custom application JavaScript and inline script attributes are forbidden.

**NFR2:** The single web experience must be mobile-friendly and remain usable on desktop without requiring a separate desktop design.

**NFR3:** The interface must remain usable in the latest stable versions of Chrome, Firefox, Safari, and Edge at viewport widths down to 320 CSS pixels.

**NFR4:** Every control must be reachable and operable without a pointer and must have a programmatic label and a visible focus indicator that is at least two CSS pixels thick and has at least 3:1 contrast against adjacent colors.

**NFR5:** Normal text must reach 4.5:1 contrast; large text, user-interface components, and meaningful graphics must reach 3:1. Inline errors must be programmatically associated with their fields. Formal accessibility certification is not required.

**NFR6:** Validation must identify errors inline and retain submitted values.

**NFR7:** Archived state and stale, provisional, or unavailable conversion results must be visibly distinguishable.

**NFR8:** Monetary input, storage, aggregation, conversion, and display must preserve exact decimal values and currency minor-unit rules without floating-point loss.

**NFR9:** Historical references must remain readable after Group or Participant archival.

**NFR10:** Every write of a complete Spending aggregate must validate Group ownership and Participant eligibility in the same atomic operation.

**NFR11:** Complete Spending and debt views must be calculated from internally consistent snapshots.

**NFR12:** Every state-changing request, including login, must be authenticated where applicable and protected against cross-site request forgery.

**NFR13:** Unsafe form replay must be suppressed server-side through a bounded, expiring, single-use session token that is atomically reserved before dispatch and distinct from CSRF.

**NFR14:** Authentication must resist repeated login attempts and use secure session cookies in production.

**NFR15:** Credentials, password hashes, session identifiers, request-protection tokens, and sensitive ledger or provider data must never appear in logs or user-facing errors.

**NFR16:** Authenticated pages must not be cached by browsers or intermediaries.

**NFR17:** Exchange-rate-provider availability must not gate startup, readiness, or ledger CRUD.

**NFR18:** User traffic, login, probes, database waits, exchange-rate calls, caches, and sessions must have bounded resource usage and wait times.

**NFR19:** Once an admitted mutation begins changing state, it must return a definitive success or rollback result rather than being cancelled by a generic timeout.

**NFR20:** Shutdown must stop new admission, allow in-flight work to finish within a defined bound, and leave the ledger recoverable.

**Total NFRs: 20.**

### Additional Requirements

#### Product And Authority Constraints

- `specs/design.md` remains normative and governs any conflict among the PRD, addendum, and project context. Downstream work must stop on divergence rather than silently choosing an interpretation.
- Debtor remains permanently a private single-Administrator workflow; Participants remain independent Group-owned accounting identities, with no global reusable Participants, Memberships, or global Participant-management surface.
- Arbitrary timeframe sums are deferred beyond v1; v1 includes only the fixed current UTC calendar month.
- The production topology is one Debtor process with one private local SQLite volume behind a sanitizing HTTPS reverse proxy. Multiple application instances and external SQLite writers are unsupported.
- Each Spending retains its stored Source Currency for historical interpretation; Group Currency remains a freely changeable display and settlement target.

#### Architecture, Ownership, And Input Contract

- Preserve inward dependencies: `debtor (root) -> debtor-web / debtor-infra -> debtor-application -> debtor-domain`.
- Domain owns synchronous deterministic rules; application owns use cases, input policy, authentication orchestration, and narrow ports; infrastructure owns SQLx, HTTP, cryptography, caching, and adapters; web owns Axum, proxy resolution, forms, CSRF/session mechanics, cookies, Askama, view models, and HTTP mapping; root owns configuration, composition, migrations, startup, lifecycle, and shutdown.
- Framework, persistence, HTTP, cryptography, session, and adapter types must not cross application-owned ports. External effects and clocks are injected, and use cases remain runnable with fakes.
- Transport adapters retain raw submitted text and decode field structure only. Application inputs parse and validate financial fields, construct allocations, and enforce financial invariants; web parsing must not construct allocations.
- Names, descriptions, dates, and Participant colors retain the exact bounds and formats stated by FR2-FR5. Application policy and transactional persistence guards remain distinct.

#### Monetary And Determinism Contract

- `rust_decimal::Decimal` is required for all money and rates. Floating point, lossy conversion, SQL monetary parsing/conversion/aggregation, and silent normalization are forbidden.
- Money persists as canonical decimal SQLite `TEXT`; repository decoding must reject malformed or noncanonical values as corruption.
- All monetary arithmetic, quantization, and settlement are checked. Failure must not panic, substitute zero, or produce partial results.
- Ledger IDs are positive `i64`; UUIDs are limited to session and CSRF randomness.
- Unordered inputs that can affect output must be sorted or stored in ordered collections, with Participant ID as the final tie-breaker.
- Balance quantization uses largest signed remainders with Participant-ID ties and exact zero-sum conservation. Settlement uses deterministic greedy matching by descending absolute Balance and Participant ID.

#### SQLite, Mutation, And Historical Integrity Contract

- SQLite uses WAL, `synchronous=FULL`, foreign keys, and a five-second busy timeout. A process-local five-second write gate serializes every ledger mutation before transaction creation.
- Group ownership, Participant eligibility, aggregate replacement, allocations, and commit remain in one transaction. Archived Participants may only retain an existing Payer or Share role during update.
- Referenced Group deletion is restricted; history-free Groups may delete with unreferenced owned Participants. Participants otherwise archive/restore and are never independently cascade-deleted through the application.
- Group creation persists `USD` and opens Manage. Archived identities are excluded from active lists and loaded through contextual archived views.
- Participant archive binds one immutable all-time Historical-mode ledger snapshot, UTC context, and quote bundle, then revalidates ledger epoch, date, and quote eligibility at final admission. Missing or invalidated evidence blocks mutation without state change; provider calls hold no database transaction.
- Among admitted valid mutations, the last committed write wins; optimistic revision columns and stale-edit conflicts are not used.
- SQLite structurally constrains supported codes, booleans, bounded text, color shape, ISO dates, relationships, and referenced-Group deletion, but does not duplicate Unicode trimming or Rust financial arithmetic.
- Checked SQLx macros and matching committed offline metadata are required; the fixed WAL-checkpoint PRAGMA is the sole verified unchecked-query exception.

#### Snapshot And Loading Contract

- Complete Spending aggregates load from one SQLite snapshot. Debt calculation materializes Group Currency and every complete Spending before releasing the read transaction; provider requests never hold a database transaction.
- History uses fixed 25-item keyset pages ordered by `(spent_date DESC, id DESC)`.
- Detail, edit, and delete load one complete aggregate directly while resolving Group ownership and current names for archived historical identities.

#### Safe Failure And Diagnostics Contract

- Application-facing failures use structured safe categories. Raw adapter errors must not cross inward ports or reach HTTP responses. `thiserror` is used by domain/application/adapters; `anyhow` is confined to root orchestration.
- Debt calculation errors map to one fixed sanitized reason and never yield partial calculations. Monthly missing quotes remain retryable `Unavailable`; checked arithmetic remains `Calculation`; rendering collapses either into whole-section unavailability while preserving Source Currency totals.
- SQLite log operations are restricted to `statement`, `pool_acquire`, `connection`, and `adapter`; result categories are restricted to `contention`, `statement`, `readonly`, `io`, `integrity`, `open`, `constraint`, `timeout`, `closed`, `protocol`, and `other`.
- Diagnostics exclude SQL, database messages, values, identifiers, query strings, provider URLs, credentials, hashes, cookies, session/CSRF data, limiter keys, and client IPs.

#### Rates And Provider Contract

- Provider JSON numbers are decoded lexically and at arbitrary precision directly into `Decimal`.
- For requested date `R` and UTC calculation date `C`, fetch date is `F = min(R, C)`. Deduplication, single-flight, and caching key on `(source, target, R, F)`; effective date remains quote evidence. Same-currency conversion is a disclosed synthetic exact rate of `1` without provider I/O.
- Historical and refreshable cache classes each cap at 4,096 deterministic-LRU entries. Past historical quotes may remain stale-eligible indefinitely; current/future quotes refresh on UTC rollover and remain eligible through seven UTC calendar days after prior `F`.
- Provider requests use five-second connect and 20-second total timeouts, a 64 KiB response limit, at most four global in-flight calls, and per-key single-flight.
- Each debt calculation deduplicates contexts and issues at most four concurrent provider requests; completion order cannot alter results, disclosures, or warnings.

#### HTTP, Forms, And Dispatch Contract

- Askama semantic HTML and vanilla CSS are required. Only pinned self-hosted HTMX and its official pinned `response-targets` extension are permitted; all interactions retain native full-page paths and no custom application JavaScript, inline scripts, inline script attributes, or custom HTMX extensions are allowed.
- A shared strict form/CSRF/submission-token extractor rejects malformed, missing, duplicate, and unknown fields before parsing or dispatch. Validation preserves the submission token; atomic reservation occurs immediately before dispatch and is terminal. Invalid token states return `409` without use-case invocation.
- Validation failures return `422` with inline errors and all raw values retained; successful mutations redirect with `303`.
- Archived Group form and mutation routes return `409` before use-case invocation.
- Mutation dispatch is marked immediately before the first state-changing use-case call. A 30-second pre-dispatch deadline applies, but no generic timeout may cancel work after dispatch.
- Debt rate unavailability returns retryable `503`; authenticated-session capacity returns retryable `503`; unseen login-limiter keys at capacity return retryable `429`.

#### Authentication, Session, CSRF, And Header Contract

- `APP_ADMIN_PASSWORD_HASH` is required, capped at 256 bytes, validated before database access, and constrained to bounded Argon2id v19 parameters. Password verification concurrency is capped at two.
- Sessions are process-local and server-side with HTTP-only `SameSite=Strict` cookies; secure cookies are mandatory outside debug builds, and restart invalidates all sessions.
- Anonymous sessions have ten-minute inactivity expiry and a 4,096-record non-evicting capacity; authenticated sessions have 30-day inactivity expiry and a 32-record non-evicting capacity.
- Successful login atomically rotates and durably persists session ID, authenticated state, and CSRF before response. Logout flushes the session. Session/token cleanup workers are mandatory supervisors whose failure fails readiness and initiates shutdown.
- Every unsafe request, including login, requires exactly one valid session-backed synchronizer token before parsing or dispatch.
- Login allows five attempts per trusted client IP in five rolling minutes, with 4,096 bounded active keys and fail-closed capacity behavior.
- Login and authenticated HTML responses send the exact no-store, nosniff, no-referrer, and restrictive CSP headers specified by the addendum.
- Probe and static routes do not create or load sessions.
- Forwarding headers are accepted only from configured trusted proxy CIDRs in one configured format; identity resolution must match across HTTP/3 and TCP fallback.

#### Admission, Probes, And Shutdown Contract

- Login request bodies cap at 8 KiB; other forms cap at 256 KiB.
- User traffic has 64 permits, login four permits, and probes a separate four-request budget.
- Safe dynamic reads and login time out after 30 seconds, Debts after 90 seconds, probes after two seconds, and SQLite readiness after one second.
- Write-gate acquisition and SQLite locking each cap at five seconds, separate from mutation pre-dispatch and post-dispatch rules.
- `/healthz` reports liveness; `/readyz` checks SQLite and mandatory supervisor health, never provider availability or ledger contents.
- Shutdown stops admission, drains HTTP for at most ten seconds, then waits without a fixed total deadline for dispatched mutations before checkpoint and pool close. The executor publishes authoritative `Committed` or `RolledBack`; an unestablished outcome is `Unknown`, triggers fatal shutdown, and is never retried or represented as rollback.

#### Edge And Rollout Contract

- A sanitizing HTTPS reverse proxy owns TLS, certificates, HTTP/3/QUIC, `Alt-Svc`, and TCP fallback; Debtor remains a private HTTP/1.1 backend.
- Edge forwarding configuration must match backend trusted-proxy CIDRs and mode.
- TLS/QUIC early data is disabled or unsafe early-data requests return `425`; only explicitly marked `GET`/`HEAD` paths may accept early data.
- Backend connections are reused, and edge timeouts cannot expire before an admitted post-dispatch mutation reaches definitive completion.
- Edge body limits match application limits.
- HTTP/3 rollout begins with short `Alt-Svc` lifetime and requires verification of UDP reachability, telemetry, TCP fallback, `425` handling, and cross-protocol identity before extension.

#### Local Operation, Maintenance, Toolchain, And Testing Contract

- With `.env` and a valid password hash, `cargo run` performs configuration, SQLite creation/connection, migration, composition, binding, safe URL logging, and graceful shutdown without Docker, frontend builds, manual migrations, metadata generation, or provider availability.
- Password hashes are generated by the independent helper; secrets never enter commands, logs, fixtures, or committed files. Pre-release local databases may require recreation after persistence or migration rewrites.
- `specs/design.md` is updated before behavior; affected ADRs, README, configuration, migrations, tests, and SQLx metadata synchronize in the same change. ADR supersession must be explicit.
- Before first deployment, clean breaking changes are allowed, compatibility shims are removed, and database compatibility is not promised, while security, accounting, and historical integrity remain mandatory.
- The exact pinned Rust and crate versions in the addendum are required; lockfiles and `--locked` are preserved, and current crate documentation is consulted before API changes.
- Production and password-helper workspaces remain independent and are validated separately.
- Domain financial tests cover examples, boundaries, and properties; application tests use injected clocks and simple fakes; infrastructure/web tests cover malformed input, persistence corruption, safe errors, bounded caches/concurrency, authentication/session capacities, strict forms, and CSRF.
- SQLx tests use temporary file databases where WAL and multi-connection behavior matter. Concurrency tests use deterministic coordination rather than sleeps. Web tests verify status, headers, redirects, retained values, rejection behavior, and non-dispatch. A root real-socket smoke test covers authentication, CSRF, an authenticated read, startup ordering, and bounded shutdown.
- Formatting, check, Clippy-with-denied-warnings, workspace tests, architecture fitness, dependency policy where changed, SQLx metadata checks where required, and separate helper checks use the exact debug-mode commands specified by the addendum. Routine `cargo build --release` is forbidden.
- Unsafe Rust is forbidden; production avoids `unwrap`/`expect`; broad lint suppression is forbidden. Naming, rustdoc, error documentation, and smallest-correct-change conventions are mandatory.

### PRD Completeness Assessment

The PRD and technical addendum are unusually detailed and jointly define the product behavior, accounting rules, failure semantics, security controls, UX acceptance criteria, deployment boundary, and validation contract. The PRD has 12 traceable FR identifiers, explicit success metrics, no declared open questions, and no unconfirmed assumptions.

The principal readiness dependency is authority synchronization: both documents explicitly defer to `specs/design.md` and require work to stop if the artifacts diverge. Therefore, apparent PRD completeness does not by itself prove implementation readiness; later coverage and alignment steps must verify that the selected Architecture, UX, Epics, and post-epics sprint-change proposal preserve this contract without contradiction or omission.

## Epic Coverage Validation

### Epic FR Coverage Extracted

- PRD-FR-1 is covered by Epic 1, Stories 1.1-1.9, with first real mutation evidence completed in Story 2.1. It decomposes to `SPEC-FR1..SPEC-FR19` and `SPEC-FR90..SPEC-FR105`.
- PRD-FR-2 is covered by Epic 2, Stories 2.1-2.2 and 2.5, with Spending-backed deletion proof completed in Story 3.3. It decomposes to `SPEC-FR20..SPEC-FR29`.
- PRD-FR-3 is covered by Epics 2 and 5, Stories 2.3-2.4 and 5.4-5.5, with historical-name projection in Story 3.4. It decomposes to `SPEC-FR30..SPEC-FR42`.
- PRD-FR-4 is covered by Epic 3, Stories 3.1-3.3. It decomposes to `SPEC-FR43..SPEC-FR59` and `SPEC-FR64`.
- PRD-FR-5 is covered by Epic 3, Stories 3.1-3.3 and 3.5. It decomposes to `SPEC-FR47..SPEC-FR61`.
- PRD-FR-6 is covered by Epic 3, Stories 3.4-3.6. It decomposes to `SPEC-FR42..SPEC-FR43` and `SPEC-FR60..SPEC-FR66`.
- PRD-FR-7 is covered by Epic 4, Story 4.1. It decomposes to `SPEC-FR67..SPEC-FR68`.
- PRD-FR-8 is covered by Epic 4, Stories 4.2-4.4. It decomposes to `SPEC-FR69..SPEC-FR77`.
- PRD-FR-9 is covered by Epic 5, Stories 5.1-5.2. It decomposes to `SPEC-FR75` and `SPEC-FR78..SPEC-FR79`.
- PRD-FR-10 is covered by Epic 5, Stories 5.1-5.2. It decomposes to `SPEC-FR78..SPEC-FR83`.
- PRD-FR-11 is covered by Epic 5, Story 5.3. It decomposes to `SPEC-FR84..SPEC-FR86`.
- PRD-FR-12 is covered by Epics 4 and 5, Stories 4.3-4.4 and 5.1-5.2. It decomposes to `SPEC-FR72..SPEC-FR83`.

**Total PRD FRs represented in epics: 12.**

### Coverage Matrix

| FR Number | PRD Requirement | Epic And Story Coverage | Status |
|---|---|---|---|
| PRD-FR-1 | Password-gated access | Epic 1, Stories 1.1-1.9; Story 2.1 mutation integration | Covered |
| PRD-FR-2 | Group lifecycle | Epic 2, Stories 2.1-2.2 and 2.5; Story 3.3 deletion restriction | Covered |
| PRD-FR-3 | Group-owned Participants | Epics 2 and 5, Stories 2.3-2.4 and 5.4-5.5; Story 3.4 name projection | Covered |
| PRD-FR-4 | Record a Spending | Epic 3, Stories 3.1-3.3 | Covered |
| PRD-FR-5 | Exact allocation | Epic 3, Stories 3.1-3.3 and 3.5 | Covered |
| PRD-FR-6 | Review and maintain history | Epic 3, Stories 3.4-3.6 | Covered |
| PRD-FR-7 | Source Currency summary | Epic 4, Story 4.1 | Covered |
| PRD-FR-8 | Group Currency summary | Epic 4, Stories 4.2-4.4 | Covered |
| PRD-FR-9 | Select conversion mode | Epic 5, Stories 5.1-5.2 | Covered |
| PRD-FR-10 | Exact Balances | Epic 5, Stories 5.1-5.2 | Covered |
| PRD-FR-11 | Deterministic Settlement Transfers | Epic 5, Story 5.3 | Covered |
| PRD-FR-12 | Calculation disclosure and failure isolation | Epics 4 and 5, Stories 4.3-4.4 and 5.1-5.2 | Covered |

### Missing Requirements

No PRD Functional Requirement is absent from the epic and story plan.

No unqualified or conflicting `PRD-FR-*` requirement appears in the epics without a PRD source. The additional `SPEC-FR1..SPEC-FR105` namespace is explicitly identified as a decomposition of the PRD plus its technical companions, not as 105 newly invented product requirements. Its coverage map assigns every `SPEC-FR` to at least one epic, and each story names its applicable `SPEC-FR` obligations.

This result establishes claimed traceability only. It does not yet establish that every acceptance criterion is internally sound, independently implementable, or aligned with UX and Architecture; those checks belong to later workflow steps.

### Coverage Statistics

- Total PRD FRs: 12
- PRD FRs covered in epics: 12
- Missing PRD FRs: 0
- Coverage percentage: 100%

## UX Alignment Assessment

### UX Document Status

Found. The confirmed final UX contract consists of:

- `ux-designs/ux-debtor-2026-08-10/DESIGN.md`, status `final`, which owns visual identity, tokens, component geometry, and responsive composition.
- `ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md`, status `final`, which owns information architecture, behavior, states, focus, announcements, and native/enhanced parity.

The conventional top-level `*ux*.md` and sharded `*ux*/index.md` patterns do not find these files because they are packaged under `ux-designs/...` with canonical filenames rather than an index. The broader artifact scan confirms that both final documents exist.

### UX To PRD Alignment

- The UX foundation preserves the permanent single-Administrator model and explicitly treats Participants as Group-owned accounting identities, matching the PRD vision, target user, FR1, and FR3.
- UX Flow 1 directly traces PRD FR4-FR12 through Spending entry, exact allocation, history, current-month summaries, conversion failure isolation, all-time Balances, Settlement Transfers, and disclosure.
- UX Flow 2 directly traces FR2-FR3 through name-only Group creation, USD default, Manage-first setup, Participant lifecycle, zero-Balance archive eligibility, contextual archived views, and history preservation.
- UX Flow 3 traces FR1 through password-only authentication, anonymous ledger denial, sign-out, expiry, and restart behavior.
- UX Flow 4 traces FR6 through fixed 25-item history, direct detail, Exact-mode correction, server-rendered deletion confirmation, and canonical return context.
- Currency/category sets, field defaults, limits, date floor, precision rules, one Payer, Proportional/Exact Shares, retained validation input, Source-Currency correction, and archived-role update restrictions match FR2-FR6.
- Source and converted summary hierarchy, Historical default, non-persisted Current mode, exact zero-sum Balances, deterministic Transfers, stale/provisional disclosure, and no-partial failure states match FR7-FR12.
- Native semantic HTML, optional pinned HTMX, no custom JavaScript, supported browsers down to 320 CSS pixels, pointer-independent operation, focus visibility, contrast, associated errors, and visible archived/conversion states match NFR1-NFR7.
- Unsafe-form submission tokens, `409` conflicts, `422` retained validation, `303` redirects, no-store pages, definitive post-dispatch outcomes, and safe retry language match the PRD/addendum security and availability contract.

### UX Requirements Beyond Explicit PRD Detail

The UX contracts add testable interaction detail not independently enumerated in the PRD:

- `UX-SHELL-01`: exact five-destination order, narrow lower-shell behavior, and wide normal-flow adaptation.
- `UX-TARGET-01`: every interactive target is at least 48 by 48 CSS pixels at 320px and 400% zoom, with no inline-link exceptions.
- `UX-ALLOC-01`: labeled focusable allocation-table scrolling, exact intrinsic column geometry, sticky Participant identity, and no page-level horizontal scrolling.
- `UX-FOCUS-01`: stable server-owned IDs, allow-listed return targets, forward-focus behavior, and explicit Back/Forward limits.
- `UX-STATUS-01`: stable polite atomic status nodes, `aria-busy`, transition ownership, and limited alert exceptions.
- `UX-PREVIEW-NATIVE-01` and `UX-PREVIEW-LATEST-01`: reviewed native Preview before Approve and revision-safe latest-input-wins enhanced previews that preserve focus, caret, keyboard, and scroll.
- `UX-CONFIRM-01`: dedicated server-rendered confirmation pages for Spending delete, Participant archive, Group archive, and history-free Group delete.
- `UX-RESPONSIVE-01`: dynamic-viewport/safe-area shell and keyboard-aware focused-form geometry.
- `UX-VISUAL-01`: dark Editorial Contrast identity, exact colors/type/rules/square geometry, and prohibition of decorative motion and card-heavy depth.

These are compatible refinements rather than product-scope expansion. They do not add users, financial modes, persistence state, or unsupported workflows.

### UX To Architecture Alignment

- AD-11 makes semantic Askama HTML and native links/forms authoritative, permits only pinned HTMX plus official `response-targets`, defines immutable self-hosted assets and security headers, and prohibits custom/inline JavaScript. This directly supports native/enhanced parity and request-status behavior.
- AD-18 explicitly adopts final `DESIGN.md` and `EXPERIENCE.md` within upstream product and architecture invariants, requires affected web stories/tests to cite stable `UX-*` IDs, and declares a story incomplete when it omits a required path, viewport, state, or verification dimension.
- AD-10 supports the UX submission-token lifecycle, strict extraction, pre-dispatch rejection, atomic reservation, exactly-one dispatch, and definitive outcome presentation.
- AD-14 provides the bounded request, session, admission, provider, and timeout envelope required for the UX pending, timeout, unavailable, and capacity states.
- AD-15 supports sanitized status/error rendering, no sensitive diagnostics, no-partial financial failures, provider-independent readiness, and session-free probes/static assets.
- AD-3 through AD-9 support every exact allocation, summary, Balance, Settlement, stale/provisional, snapshot, and archival-eligibility state presented by UX.
- AD-4 and AD-5 support contextual archived views, read-only archived Groups, current-name historical projections, active allocation eligibility, and Exact-on-edit behavior.
- AD-12 supports the UX's one responsive online web experience and native fallback across HTTP/3 edge and TCP backend paths without changing application interaction semantics.
- Architecture's deferred route inventory, template layout, breakpoint selection, and exact vendored asset digests are bounded implementation choices. AD-11 and AD-18 already constrain their required outcomes, so these deferrals do not leave UX behavior optional.

### Alignment Issues

No direct contradiction was found among the final PRD/addendum, final UX contracts, and final Architecture Spine.

The UX adds stronger interaction obligations than the PRD states explicitly, but AD-18 incorporates them and the epic crosswalk assigns all stable UX contracts to stories. Their status is therefore binding and traceable rather than advisory.

### Warnings

- Source authority remains a stop-work invariant: the UX and Architecture defer product/security/accounting authority to `specs/design.md` and accepted ADRs. Any later divergence must be reconciled rather than interpreted during implementation.
- The UX filenames do not match the workflow's narrow conventional search patterns and have no `index.md`. Future automated discovery depends on retaining the broader artifact scan or adding an index; this is a planning-process warning, not a product gap.
- Exact HTMX asset digests and implementation-selected responsive thresholds remain intentionally deferred. Their acceptance boundaries are fixed, but each implementing story must supply verification evidence before the affected enhanced route ships.

## Epic Quality Review

### Review Context

The current `epics.md` contains five epics and 29 stories and declares `validationStatus: pending-revalidation`. The approved `sprint-change-proposal-2026-08-11.md` identifies three implementation-gating defects and prescribes a coordinated correction, but the current epic artifact still has its old story structure and traceability. This review therefore assesses the actual current `epics.md`, not the proposed future state.

### Epic Compliance Summary

| Epic | User Value | Backward-Only Epic Dependencies | Story Sizing | No Forward Story Dependency | Persistence Timing | Acceptance Criteria | Traceability |
|---|---|---|---|---|---|---|---|
| Epic 1: Securely Operate and Access Debtor | Pass | Pass for delivered user/operator outcome | Fail: Story 1.2 oversized | Pass with explicitly staged later mutation evidence | Pass | Strong BDD overall | Fail: affected stories omit binding `UX-*` IDs/evidence |
| Epic 2: Organize Groups and Participants | Pass | Pass | Pass overall | Pass; Spending-backed deletion restriction is introduced when Spendings first exist | Pass | Gap in confirmation/focus criteria | Fail: affected stories omit binding `UX-*` IDs/evidence |
| Epic 3: Record and Maintain Exact Spendings | Pass | Pass | Pass overall | Pass | Pass: Spending schema appears at first Spending consumer | Gaps in reviewed-commit and deletion confirmation evidence | Fail: affected stories omit binding `UX-*` IDs/evidence |
| Epic 4: Understand Current-Month Spending | Pass at epic level | Pass at epic level | Fail: Story 4.2 is a technical slice | Fail: Story 4.2 requires Story 4.3 for Administrator value | Pass | Detailed but mis-sliced | Fail: affected stories omit binding `UX-*` IDs/evidence |
| Epic 5: Calculate Debts, Settle, and Safely Retire Identities | Pass | Pass; consumes completed Epic 4 capability | Pass overall | Pass | Pass: mutation epoch appears at first archival consumer | Gap in Participant archive confirmation evidence | Fail: affected stories omit binding `UX-*` IDs/evidence |

### Critical Violations

#### CQ-1: Epic 4 Contains A Technical Story With Forward-Dependent Value

**Affected:** Story 4.2, `Resolve Exact Historical Rate Evidence`.

The story's stated value is that "later Group Currency totals are reproducible and correctly contextualized." It delivers provider decoding, rate context, timeout, cache, and concurrency machinery, but no independently usable Administrator outcome. Visible converted totals arrive only in Story 4.3.

This violates two standards simultaneously:

- A story must deliver meaningful user or operator value rather than a technical milestone.
- A story must be independently complete without a future story supplying its value.

**Impact:** Story 4.2 cannot be accepted as a user-valued vertical increment. Estimation, demonstration, and acceptance would validate infrastructure in isolation while deferring the feature outcome that justifies it.

**Required remediation:** Apply approved proposal section 4.1. Merge the minimum exact historical/synthetic rate path into the first visible converted-total Story 4.2, and move advanced cache/stale/degraded behavior into visible resilience Story 4.3. Update all affected crosswalks from Stories 4.2-4.4 to 4.2-4.3.

#### CQ-2: The Approved Corrective Backlog Is Not The Current Implementation Backlog

**Affected:** `epics.md` as a whole.

The approved sprint-change proposal requires a Story 1.2 split, Epic 4 reslice, story-level UX traceability, regenerated owner mappings, mechanical requirement audit, epic validation, and a new readiness run. None of those structural edits appears in the current epic document, which still declares `pending-revalidation`.

**Impact:** Phase 4 would implement a backlog that the planning owner has already approved for replacement. Story numbers, ownership, acceptance evidence, and dependency boundaries would immediately diverge from the approved handoff.

**Required remediation:** Apply the approved proposal atomically to `epics.md`, retain `pending-revalidation` until validation succeeds, then rerun epic validation and this readiness workflow.

### Major Issues

#### MQ-1: Story-Level UX Traceability Is Missing Across All Affected Web Stories

The global UX owner table maps `UX-*` bundles to story numbers, but none of the 29 story-level `Requirements` lines cites a stable UX identifier. AD-18 requires each affected web story and acceptance test to cite applicable stable IDs; a global table does not replace route-specific proof.

Several criteria use generic phrases such as "accessible," "responsive," or "shared behavior" without proving the exact binding dimensions: 48-by-48 targets at 320px/400% zoom, stable focus destinations, status nodes and announcements, safe-area/keyboard geometry, visual tokens, allocation geometry, reviewed Preview, or confirmation return behavior.

**Impact:** Implementers can satisfy the textual story while missing final UX obligations. Review cannot mechanically distinguish applicable contracts or verify that native and enhanced paths, viewport states, focus states, and announcements are complete.

**Required remediation:** Add specialized `UX contracts:` and `UX acceptance evidence:` clauses to every affected web story. Regenerate the owner table from final story ownership after renumbering. Do not assign UX IDs to non-web operator stories merely for uniformity.

#### MQ-2: Story 1.2 Is Too Large For Reliable Independent Estimation And Review

Story 1.2 currently combines workspace/toolchain structure, dependency direction, bootstrap configuration, SQLite creation and migration, WAL/synchronous/busy settings, concrete adapter composition, socket bind, safe URL reporting, provider-independent startup, no-active-mutation shutdown, dependency pinning/policy, SQLx policy/metadata, full workspace validation, and architecture fitness.

These concerns have multiple independent failure modes and evidence surfaces. A single acceptance result cannot isolate whether failure belongs to startup, persistence, composition, lifecycle, dependency policy, SQLx metadata, or architecture validation.

**Impact:** The story is difficult to estimate, implement, review, and diagnose as one increment despite its valid operator-level goal.

**Required remediation:** Apply approved proposal section 4.2. Story 1.2 should end with a persistent runnable local process and minimum checked persistence/composition. New Story 1.3 should own clean shutdown, restart against the initialized database, recovery behavior, architecture fitness, and complete validation. Renumber current Stories 1.3-1.9 to 1.4-1.10 and update every reference atomically.

#### MQ-3: Story 2.5 Does Not Implement The Binding Confirmation Contract

Story 2.5 dispatches Group archive directly from a protected archive form. History-free Group deletion says only that the Administrator "confirms and dispatches" deletion. Neither path specifies the dedicated server-rendered confirmation page, named scope/reversibility, allow-listed cancel target, deterministic success focus, or announcement required by `UX-CONFIRM-01`.

**Required remediation:** Add separate Group archive and history-free delete confirmation scenarios. Name the Group and affected Participants, distinguish reversible archive from irreversible delete, prove one-shot protected submission, encode only allow-listed return/focus destinations, and verify deterministic post-success focus.

#### MQ-4: Story 3.3 Does Not Prove Commit Is Bound To Currently Reviewed Input

Story 3.3 correctly reparses raw commit input with the same allocation operation as Preview, but it does not prove the native reviewed non-editable state or that Approve is unavailable while Preview is pending, stale, invalid, or superseded. Consequently, the criteria do not prevent a commit path from bypassing `UX-PREVIEW-NATIVE-01` or `UX-PREVIEW-LATEST-01`.

**Required remediation:** Add native and enhanced acceptance criteria proving that only currently reviewed input can reach Approve, any field/revision change invalidates approval, superseded enhanced responses cannot re-enable approval, and commit always revalidates the approved raw input server-side.

#### MQ-5: Story 3.6 Has An Incomplete Spending Deletion Confirmation And Return Contract

Story 3.6 opens a confirmation page and verifies atomic deletion, but it omits the cancel path, allow-listed invoker return, one-shot activation behavior, page-boundary canonical return selection, and exact focus destination required by `UX-CONFIRM-01` and the focus matrix.

**Required remediation:** Add confirmation cancel/no-mutation criteria, token-protected single activation, and pagination-safe success behavior that targets the next summary, previous summary, or Transactions heading without returning an out-of-range page.

#### MQ-6: Story 5.4 Omits The Participant Archive Confirmation Surface

Story 5.4 calculates and revalidates exact-zero eligibility but proceeds from eligibility to archive commit without the required dedicated confirmation page. The final UX requires confirmation after complete eligibility is established without allowing the rendered page to cache or bypass server-owned final revalidation.

**Required remediation:** Add `UX-CONFIRM-01` acceptance criteria that name the Participant, explain archive effects/reversibility, preserve no rate evidence, use a one-shot protected submission, allow cancellation, and still perform authoritative ledger/date/quote revalidation inside the dispatched archive use case.

### Minor Concerns

#### NQ-1: Crosswalk And Owner Ranges Are Fragile During Approved Renumbering

The epic artifact repeats story-number ranges in the PRD crosswalk, UX owner table, epic coverage text, implementation notes, and story references. The approved Story 1.2 split and Epic 4 merge will invalidate several locations simultaneously.

**Required remediation:** Apply changes atomically and mechanically audit every `PRD-FR-*`, `SPEC-FR*`, `SPEC-NFR*`, `AD-*`, `UX-*`, and `Story X.Y` reference before changing validation status.

### Dependency Analysis

- Epic order is logical: secure runtime and access -> Groups/active Participants -> exact Spendings -> current-month summaries/rates -> all-time debts/settlement/archive.
- No epic requires a later epic to deliver its stated completed epic-level user outcome.
- Story 4.2 is the sole clear within-epic forward-value dependency: its Administrator value explicitly arrives in Story 4.3.
- Story 1.8's later real-mutation proof in Story 2.1 is staged cross-cutting evidence, not a blocker to its own no-active-mutation authenticated-runtime shutdown outcome.
- Story 2.5 can implement safe history-free Group deletion before Spending persistence; Story 3.3 adds and structurally proves the referenced-Group restriction when Spendings first become possible. This follows just-in-time schema evolution rather than creating a forward dependency for Story 2.5's current capability.
- Participant archive is correctly deferred to Epic 5 because its complete Historical Balance and immutable quote context are prerequisites. Epic 2 does not claim that rate-dependent lifecycle outcome.
- Epic 5 dependencies are backward-only: Historical Balances precede Current mode, complete Balances precede Settlement, and calculation/rate capability precedes Participant archive.

### Database And Brownfield Checks

- No architecture starter template is specified, so no mandatory starter-template Story 1.1 is missing.
- The project is brownfield. The plan retains existing valid crate/runtime boundaries while replacing superseded identity, payer, share-mode, route, and schema concepts incrementally.
- Database structures are introduced at first use: foundational SQLite/runtime in Epic 1, Group/Participant persistence in Epic 2, Spending aggregate persistence in Epic 3, and mutation-epoch support at the first archival consumer in Story 5.4.
- No up-front story creates all feature tables before their consumers.
- Clean breaking migrations are consistent with the approved pre-release policy; no unsupported compatibility story is needed.

### Acceptance Criteria Assessment

- All 29 stories use explicit As/I want/So that structure and Given/When/Then acceptance criteria.
- Happy paths, validation, failures, concurrency, deterministic ordering, safety, and test evidence are generally unusually specific and measurable.
- No vague one-line criterion such as "user can login" was found.
- The material completeness gaps are concentrated in the binding UX evidence listed in MQ-1 and the route-specific corrections MQ-3 through MQ-6.

### Quality Verdict

The five-epic product sequence is logical and all 12 PRD FRs have claimed owners, but the current story plan is not implementation-ready. It contains one technical forward-dependent story, one oversized foundational story, artifact-wide missing story-level UX traceability, and four material UX acceptance omissions. The approved correction proposal addresses these defects, but until it is applied and revalidated, `epics.md` must remain gated from Phase 4.

## Summary and Recommendations

### Overall Readiness Status

**NEEDS WORK**

Phase 4 implementation must not start from the current `epics.md`.

The product contract itself is strong:

- All required planning document categories exist.
- The PRD defines 12 FRs and 20 NFRs with no declared open questions or unconfirmed assumptions.
- All 12 PRD FRs have claimed epic/story coverage, for 100% functional traceability.
- Final UX contracts align with the PRD and are explicitly adopted by Architecture AD-18.
- The five-epic sequence is logical, user-oriented, and generally backward-dependent.
- Persistence structures are planned at first use rather than created speculatively.
- Story acceptance criteria are predominantly specific, measurable BDD scenarios.

The implementation backlog is nevertheless gated because its approved corrections remain unapplied and the current artifact explicitly remains `pending-revalidation`.

### Critical Issues Requiring Immediate Action

1. **Apply the approved sprint-change proposal to the actual implementation backlog.** The current `epics.md` is not the approved target backlog and must not be used for Phase 4.
2. **Eliminate technical, forward-dependent Story 4.2.** Merge minimum rate evidence into visible converted monthly totals and move advanced stale/cache resilience into the next user-valued story.
3. **Split oversized Story 1.2.** Separate minimum persistent local startup from restart, shutdown, recovery, architecture fitness, and complete validation while preserving runnable operator value in both stories.
4. **Restore binding story-level UX traceability and acceptance evidence.** Every affected web story must cite and objectively prove its applicable stable `UX-*` contracts.
5. **Correct material route-level UX omissions.** Group archive/delete, Spending deletion, and Participant archive need complete `UX-CONFIRM-01` behavior; Spending commit must be bound to currently reviewed input under native and enhanced Preview contracts.

### Recommended Next Steps

1. Apply approved proposal section 4.2: split current Story 1.2, add new Story 1.3, renumber current Stories 1.3-1.9 to 1.4-1.10, and synchronize all references.
2. Apply approved proposal section 4.1: merge current Stories 4.2 and 4.3 into a visible converted-total Story 4.2, move advanced resilience into Story 4.3, and update Epic 4 crosswalk ranges.
3. Add specialized stable UX contract citations and route-specific acceptance evidence to every affected web story, including exact viewport, zoom, target, focus, status, visual, native/enhanced, allocation, and confirmation obligations.
4. Correct Stories 2.5, 3.3, 3.6, and 5.4 with the specific confirmation, reviewed-input, cancellation, one-shot submission, canonical return, and deterministic focus scenarios documented in this report.
5. Regenerate the global UX owner index from final story ownership rather than editing it as an independent source.
6. Mechanically audit every `PRD-FR-*`, `SPEC-FR*`, `SPEC-NFR*`, `AD-*`, `UX-*`, and internal `Story X.Y` reference after restructuring.
7. Keep `validationStatus: pending-revalidation` until epic validation passes. Then update the status, rerun Implementation Readiness, and begin Phase 4 only if the new report removes CQ-1, CQ-2, and MQ-1 through MQ-6.

### Final Note

This assessment identified **9 issues across 4 planning-quality categories**: story decomposition/dependency, UX traceability, route-level acceptance completeness, and crosswalk maintenance. Two issues are critical, six are major, and one is minor.

The approved corrective direction is sufficient and does not require PRD, Architecture, UX, MVP, Rust, SQL, migration, or deployment changes. The remaining work is a coordinated backlog correction followed by epic validation and a fresh readiness assessment. Proceeding with implementation before that sequence completes would knowingly implement a superseded, unvalidated story plan.

**Assessment date:** 2026-08-11  
**Assessor:** Kilo Implementation Readiness Review
