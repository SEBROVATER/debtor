---
stepsCompleted: ['document-discovery', 'prd-analysis', 'epic-coverage-validation', 'ux-alignment', 'epic-quality-review']
assessmentRun: 2026-08-22T14:51:07+06:00
previousAssessmentSteps: ['document-discovery', 'prd-analysis', 'epic-coverage-validation', 'ux-alignment', 'epic-quality-review', 'final-assessment']
selectedDocuments:
  prd: prds/prd-debtor-2026-08-10/prd.md
  architecture: architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md
  epics: epics.md
  ux:
    - ux-designs/ux-debtor-2026-08-10/DESIGN.md
    - ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md
---

# Implementation Readiness Assessment Report

**Date:** 2026-08-22
**Project:** debtor

## Document Inventory

### PRD

- `prds/prd-debtor-2026-08-10/prd.md`
- `prds/prd-debtor-2026-08-10/addendum.md`

### Architecture

- `architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md`

### Epics

- `epics.md`

### UX

- `ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md`
- `ux-designs/ux-debtor-2026-08-10/DESIGN.md`

### Discovery Decision

No duplicate whole-versus-sharded canonical documents were found. Reconciliation, review,
mockup, import, and validation files are supporting artifacts, not canonical assessment inputs.

## PRD Analysis

### Functional Requirements

FR-1: The Administrator can sign in with one configured password, remain authenticated during an active session, and sign out. Debtor provides no username, registration, Participant login, or multi-user authorization. Anonymous visitors cannot view ledger data; restart ends authenticated sessions; unsafe requests require request protection; every unsafe form also has a bounded, expiring, session-bound, single-use submission token.

FR-2: The Administrator can create, edit, archive, and restore a Group. A Group with no Spendings can be deleted with its unreferenced Participants; a Group with any Spending cannot be deleted. Creation requires only a valid name, defaults Group Currency to USD, and opens Manage; active lists omit archived records; archived Groups remain readable with no mutation controls and reject direct mutation attempts.

FR-3: The Administrator can add, edit, archive, and restore Participants inside a Group. Each Participant belongs to exactly one Group. New allocations use only active owned Participants; archived identities remain in history, Balances, and Transfers; archive requires an immutable exact-zero Historical calculation context and remains blocked on invalidated eligibility or missing rates; restore has no Balance check; names and colors have defined validation.

FR-4: The Administrator can create a Spending with description, date, category, positive Total, Source Currency, exactly one Payer, and Proportional or Exact Shares. Supported currency and category rosters are fixed; description/date boundaries apply; initial form defaults and the unified Payer/Share allocation table are prescribed; validation retains submitted values and renders inline errors.

FR-5: Every accepted Spending preserves the Total exactly in Source Currency minor units: the single Payer pays the Total and Shares independently sum to the Total. Amounts are positive, bounded, precision-valid, unique by Participant, and exact; proportional weights use one checked integer-ratio operation with deterministic residuals; exact shares initialize deterministically and must close the difference; archived identities may only remain in their existing update role.

FR-6: The Administrator can browse, inspect, edit, and delete Spendings in an active Group. History is newest-first in 25-item pages; archived history remains readable with current Participant names; edits may correct Source Currency under creation validation; edits reopen Exact mode; every Spending change is atomic.

FR-7: The Administrator can see the selected Group's current-UTC-month Spending Total and each Payer's paid total grouped by original Source Currency. Source totals remain available without conversion and exclude Spendings outside the current month.

FR-8: The Administrator can see the same current-month Group and per-Payer totals converted to Group Currency using historical spending-date rates. Future dates are provisional; stale eligibility is context-specific; converted values use exact accumulation and deterministic joint quantization; quote or checked-calculation failure makes the whole converted section retryably unavailable without partial totals while Source totals and CRUD remain usable.

FR-9: The Administrator can calculate all-time Balances in Historical or non-persisted Current mode. Historical is the default and uses each Spending date; Current uses the UTC calculation date for all Spendings.

FR-10: Debtor calculates one exact Group Currency Balance per Participant and preserves an exact zero sum after currency quantization. Exchange-rate completion order cannot change results or warnings; arithmetic or conversion failure returns no partial Balances or Settlement Transfers.

FR-11: Debtor presents positive, deterministic Settlement Transfers that settle every Balance. A pair appears at most once; at most `n - 1` transfers are produced for `n` Participants; global transfer-count minimality is not claimed.

FR-12: The debts view identifies conversion mode, calculation time, Group Currency, unique rates, and stale/provisional warnings. Missing eligible quotes return retryable failure and never block Group, Participant, or Spending management.

Total FRs: 12

### Non-Functional Requirements

NFR-1 Correctness and historical integrity: monetary input, storage, aggregation, conversion, and display preserve exact decimals and minor-unit rules without floating point; archived historical references remain readable; complete Spending writes validate ownership and eligibility atomically; Spending and debt views use internally consistent snapshots.

NFR-2 Security and privacy: every unsafe request, including login, is authenticated where applicable and CSRF-protected; replay uses a bounded, expiring, atomically reserved, distinct single-use session token; authentication resists repeated attempts and production uses secure cookies; credentials, hashes, session IDs, request-protection tokens, and sensitive ledger/provider data never reach logs or user-facing errors; authenticated pages are not cached.

NFR-3 Availability and bounded operation: rate-provider availability never gates startup, readiness, or ledger CRUD; traffic, login, probes, database waits, provider calls, caches, and sessions have bounded resources and waits; admitted state-changing mutations return definitive success or rollback rather than generic-timeout cancellation; shutdown stops admission, completes in-flight work under defined bounds, and leaves the ledger recoverable.

NFR-4 UX/accessibility: the experience is semantic server-rendered HTML with valid native links/forms; only pinned HTMX core and official response-targets may enhance it; it is usable on current Chrome, Firefox, Safari, and Edge down to 320 CSS pixels; controls are keyboard-operable, labelled, and visibly focused at two CSS pixels/3:1; text and components meet 4.5:1/3:1 contrast; inline errors are associated; submitted values are retained; archived and conversion states are distinguishable.

Total NFR groups: 4

### Additional Requirements And Constraints

- Product boundaries: permanently single Administrator; Participants are Group-owned identities; one process and one private local ledger volume behind a sanitizing HTTPS proxy; Source Currency is stored while Group Currency is changeable.
- Addendum technical contract: inward crate dependency direction; injected ports/effects; exact `Decimal`, canonical SQLite TEXT, no SQL monetary operations; deterministic ordering; WAL/FULL/foreign-key SQLite with a five-second write gate and lock bounds; checked SQLx metadata.
- HTTP/security/operations: strict shared form extraction; defined `422`, `409`, and `303` behavior; bounded request and provider timeouts; session/CSRF/token pools and supervised cleanup; restrictive security headers and trusted-proxy policy; readiness/liveness separation; definitive mutation shutdown semantics; reverse proxy/HTTP3 responsibilities.
- Delivery quality: locked pinned toolchain, independent password-helper workspace, layer-specific tests and root smoke test, pedantic Clippy with denied warnings, architecture fitness, dependency-policy checks when applicable, and update-first synchronization of the normative design contract.

### PRD Completeness Assessment

The PRD contains 12 contiguous, explicit functional requirements and four cross-cutting NFR groups. The companion addendum supplies the technical precision needed to make each requirement testable. No open questions or unconfirmed assumptions are recorded. Requirement-to-epic coverage and UX/architecture alignment remain to be validated in subsequent steps.

## Epic Coverage Validation

### Epic FR Coverage Extracted

- PRD-FR-1: Epic 1, Stories 1.1-1.10, with real mutation evidence completed in Story 2.1.
- PRD-FR-2: Epic 2, Stories 2.1-2.2 and 2.5, with Spending-backed deletion proof in Story 3.1.
- PRD-FR-3: Epics 2 and 5, Stories 2.3-2.4 and 5.4-5.5, with historical-name evidence in Story 3.3.
- PRD-FR-4: Epic 3, Stories 3.1-3.2.
- PRD-FR-5: Epic 3, Stories 3.1-3.2 and 3.4.
- PRD-FR-6: Epic 3, Stories 3.3-3.5.
- PRD-FR-7: Epic 4, Story 4.1.
- PRD-FR-8: Epic 4, Stories 4.2-4.3.
- PRD-FR-9: Epic 5, Stories 5.1-5.2.
- PRD-FR-10: Epic 5, Stories 5.1-5.2.
- PRD-FR-11: Epic 5, Story 5.3.
- PRD-FR-12: Epics 4 and 5, Stories 4.2-4.3 and 5.1-5.2.

Total PRD FRs claimed in epics: 12

### Coverage Matrix

| FR Number | PRD Requirement | Epic Coverage | Status |
| --- | --- | --- | --- |
| PRD-FR-1 | Password-gated single-administrator access | Epic 1; Stories 1.1-1.10 | Covered |
| PRD-FR-2 | Group lifecycle | Epic 2; Stories 2.1, 2.2, 2.5; Story 3.1 final deletion proof | Covered |
| PRD-FR-3 | Group-owned Participant lifecycle | Epics 2 and 5; Stories 2.3, 2.4, 5.4, 5.5; Story 3.3 name history | Covered |
| PRD-FR-4 | Record a Spending | Epic 3; Stories 3.1-3.2 | Covered |
| PRD-FR-5 | Exact allocation | Epic 3; Stories 3.1, 3.2, 3.4 | Covered |
| PRD-FR-6 | Review and maintain history | Epic 3; Stories 3.3-3.5 | Covered |
| PRD-FR-7 | Source Currency summary | Epic 4; Story 4.1 | Covered |
| PRD-FR-8 | Group Currency summary | Epic 4; Stories 4.2-4.3 | Covered |
| PRD-FR-9 | Select conversion mode | Epic 5; Stories 5.1-5.2 | Covered |
| PRD-FR-10 | Exact Balances | Epic 5; Stories 5.1-5.2 | Covered |
| PRD-FR-11 | Deterministic Settlement Transfers | Epic 5; Story 5.3 | Covered |
| PRD-FR-12 | Calculation disclosure and failure isolation | Epics 4 and 5; Stories 4.2-4.3, 5.1-5.2 | Covered |

### Missing Requirements

No PRD functional requirement is missing from the explicit epic crosswalk. The `SPEC-FR-*` and `SPEC-NFR-*` entries are a separate decomposed implementation namespace, not extra untraced PRD functional requirements.

### Coverage Statistics

- Total PRD FRs: 12
- FRs covered in epics: 12
- Coverage percentage: 100%

## Reassessment Run: UX Alignment Assessment

### UX Document Status

Found: final `DESIGN.md` and `EXPERIENCE.md` contracts define visual identity, responsive composition, information architecture, native/enhanced interaction, financial states, focus, accessibility, and route-specific acceptance references.

### Alignment Confirmed

- The three UX flows cover the PRD's secure access, Group/Participant lifecycle, Spending entry and maintenance, monthly summaries, Debts, Settlement Transfers, archive/restore, and conversion-unavailable outcomes.
- The UX preserves every product boundary: one Administrator, Group-owned accounting identities, one Payer with Proportional/Exact Shares, advisory-only Settlement Transfers, no persisted Current mode, and no manual rate retry.
- Native HTML is authoritative with only the approved HTMX core and `response-targets` extension as optional enhancement, matching PRD and AD-11. The architecture supplies server-rendering, strict forms, session/CSRF/token admission, safe statuses, static assets, and the no-imperative-post-swap constraint.
- The architecture explicitly supports the UX's exact-money displays, immutable full calculation results, rate states, no-partial failure rendering, direct aggregate history reads, confirmation pages, responsive full-page Spending form, and Debts retained-radio focus behavior through AD-3 through AD-11 and AD-18.
- The prior generic autofocus conflict is resolved: the Interaction Focus Matrix now limits automatic focus to full-page responses or enhanced rows that specify forward focus. Enhanced Debts success and expected errors explicitly retain the activated radio without an autofocus target.

### Alignment Issues

No current PRD-to-UX or UX-to-architecture behavior conflict was found.

### Warnings

- `DESIGN.md`, `EXPERIENCE.md`, and `ARCHITECTURE-SPINE.md` frontmatter dates predate their current filesystem modification times. Refresh source-artifact update metadata in the next controlled planning-document revision so the audit trail reflects the latest contract changes.

## Reassessment Run: Epic Quality Review

### Epic Structure

| Epic | User value | Independence and sequencing | Assessment |
| --- | --- | --- | --- |
| Epic 1: Securely Operate and Access Debtor | Administrator can safely start, sign in, use the protected shell, and sign out. | Establishes runnable access, bounded admission, and lifecycle primitives before ledger work. | Valid user-value epic. |
| Epic 2: Organize Groups and Participants | Administrator can create the ledger context and active accounting identities. | Depends only on Epic 1; rate-dependent Participant retirement is explicitly deferred. | Valid. |
| Epic 3: Record and Maintain Exact Spendings | Administrator can create, inspect, correct, and delete exact Spendings. | Depends only on active Groups/Participants from Epics 1-2. | Valid. |
| Epic 4: Understand Current-Month Spending | Administrator can understand Source and Group Currency monthly totals. | Depends on Spending history from Epic 3; Source totals form an independently useful first slice. | Valid. |
| Epic 5: Calculate Debts, Settle, and Safely Retire Identities | Administrator can calculate Balances, derive advisory Transfers, and perform rate-safe Participant archival. | Builds on Spending/rate work; Historical calculation precedes Current, Settlement, and archival consumers. | Valid. |

### Dependency and Acceptance-Criteria Assessment

- Stories use specific Given/When/Then acceptance criteria with success, failure, lifecycle, security, concurrency, responsive, and native/enhanced cases.
- The Final-Evidence Ledger prevents false completion claims for shared requirements. It explicitly separates enabling work from final evidence for referenced-Group deletion, Participant lifecycle, real-mutation dispatch, and real-mutation shutdown.
- First-consumer timing is sound: Group persistence begins in Story 2.1, Participant persistence in Story 2.3, Spending aggregates in Story 3.1, rate-derived monthly summaries in Epic 4, and all-time debt/archival work in Epic 5.
- The corrected first-consumer sequence removes the Login/Sign-out forward dependency: Stories 1.4-1.6 establish the required token behavior and Story 1.7 extends it to authenticated/general routes.
- The approved one-developer packet table gives high-complexity stories explicit boundaries and a seven-day split rule. The densest packets, especially Stories 1.5, 3.1, 4.2, 5.1, and 5.4, still require that estimate/split gate immediately before any future reassignment.

### Critical Violations

None found. The former Epic 1 operations-gate problem is resolved: the HTTPS edge record is no longer in the Phase 4 epic sequence.

### Major Issues

None found. No story requires a future story to achieve its stated usable outcome, and no technical-only epic is present.

### Minor Concerns

1. `epics.md:7` remains `pending-revalidation-2026-08-22` while this reassessment is in progress. Final assessment must set the result to the current verified status rather than leave an obsolete pending marker.
2. Story 4.3 repeats the fixed-past/future stale-fallback acceptance cases at `epics.md:2010-2058`. The duplicate criteria are consistent, but one canonical set would reduce maintenance and review ambiguity.
3. Sprint tracking still marks Epics 1 through 5 `in-progress` although every listed story is `done`; this does not alter requirement coverage, but it obscures actual delivery state and should be reconciled under the tracker’s documented transition rules.

### Quality Checklist

- [x] Every epic delivers a user or operator outcome.
- [x] Epic sequencing contains no direct forward dependency.
- [x] Stories have defined packet boundaries and estimates.
- [x] Database and shared primitives are introduced by their first runnable consumer.
- [x] Acceptance criteria are testable and use Given/When/Then form.
- [x] PRD-FR traceability is maintained.

## UX Alignment Assessment

### UX Document Status

Found: `ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md` defines information architecture, interaction, states, focus, announcements, native/enhanced parity, accessibility, and responsive behavior. `ux-designs/ux-debtor-2026-08-10/DESIGN.md` defines the visual system, component geometry, target size, responsive composition, contrast, and motion constraints.

### Alignment Confirmed

- PRD journeys and all 12 PRD FRs have corresponding UX surfaces, flows, states, or component contracts: protected Sign in; Group/Participant lifecycle; focused Spending entry and history; current-month Summary; Debts; settlement; archive/restore; and rate-unavailable outcomes.
- The UX native-first, server-rendered, HTMX allowlist, accessibility, 320px, focus, status, retained-validation, and no-partial-financial-output requirements align with the PRD and technical addendum.
- Architecture AD-11 provides the native HTML, asset, CSP, and enhanced-error boundary; AD-18 makes the final UX contracts binding on routes, templates, CSS, enhancements, and acceptance tests. AD-2, AD-5 through AD-10, and AD-14 through AD-16 supply the application, persistence, financial, unsafe-form, bounded-operation, and test ownership required by the UX flows.

### Alignment Issues

- High: `EXPERIENCE.md` has an internal focus-contract contradiction. Its generic rule says each forward full-page or HTMX response renders an autofocus destination (`EXPERIENCE.md:259`), but its Debts-specific contract requires enhanced success and expected errors to retain the activated Historical/Current radio (`EXPERIENCE.md:131-132`, `:265`). The latter aligns with the normative Debts policy and architecture's prohibition on imperative post-swap focus behavior. Clarify the generic rule to exempt enhanced responses that intentionally retain focus, especially Debts, before using the UX contract as an implementation authority.

### Warnings

- The UX package is complete and architecture-supported, but the unresolved focus contradiction can cause a story to emit an enhanced-fragment `autofocus` attribute and steal keyboard focus. This is a readiness blocker for any feature relying on the affected generic focus language.

## Epic Quality Review

### Epic Structure

| Epic | User value | Independence and sequencing | Assessment |
| --- | --- | --- | --- |
| Epic 1: Securely Operate and Access Debtor | Administrator can safely start, access, and operate the private ledger. | Establishes startup, authentication, shell, bounded admission, and shutdown before ledger work. | Valid user-value epic, except Story 1.10. |
| Epic 2: Organize Groups and Participants | Administrator can establish and manage a ledger context and active identities. | Depends only on Epic 1; lifecycle requirements whose proof needs Spendings are explicitly deferred. | Valid. |
| Epic 3: Record and Maintain Exact Spendings | Administrator can record, inspect, correct, and delete exact Spendings. | Depends on active Groups/Participants from Epics 1-2, not later work. | Valid. |
| Epic 4: Understand Current-Month Spending | Administrator can understand source and converted current-month totals. | Depends on Spending history from Epic 3; source totals precede rate-dependent conversion. | Valid. |
| Epic 5: Calculate Debts, Settle, and Safely Retire Identities | Administrator can calculate debts, receive settlement advice, and safely archive/restore identities. | Depends on prior Spending/rate work; Historical calculation precedes Current, Settlement, and archival consumers. | Valid. |

### Dependency and Acceptance-Criteria Assessment

- All story acceptance criteria use Given/When/Then format and specify observable positive, failure, lifecycle, security, and responsive outcomes.
- The `Final-Evidence Ledger` correctly handles shared requirements: Group deletion restriction completes after the first Spending; archive/restore completes with debt capabilities; real mutation lifecycle proof completes with Group creation. These are sequenced final-evidence dependencies, not forward implementation dependencies.
- Database/persistence capability is introduced with the first consuming vertical slice: Groups in Story 2.1, Participants in Story 2.3, Spendings in Story 3.1, rate-derived summaries in Epic 4, and all-time debts/archival in Epic 5.
- Most stories retain a user or operator outcome and explicitly identify prior capabilities they reuse. Story packet estimates and single-route/use-case boundaries are stated, although Stories 1.5, 3.1, 4.2, and 5.1 warrant estimate review before assignment because their dense cross-layer acceptance sets sit at the upper approved limit.

### Critical Violations

- Story 1.10, `Define the Pre-Production HTTPS Edge Gate` (`epics.md:1141-1189`), is explicitly a pre-production operations gate and "not a Phase 4 application implementation story." It is not an independently shippable Administrator-facing increment and depends on a future edge-product/environment choice. Remove it from Epic 1's Phase 4 story sequence and track it as a separate operations/deployment readiness work item with its own owner and gate criteria.

### Major Issues

None found beyond the UX focus contradiction recorded in the UX alignment assessment.

### Minor Concerns

- The epic document frontmatter says `validationStatus: revalidated-2026-08-12` (`epics.md:7`) although policy and story text changed on 2026-08-22. Refresh the validation date/status after resolving this readiness report's blockers so the artifact audit trail represents the current planning set.

### Quality Checklist

- Epic user value: pass for Epics 1-5; exception: Story 1.10 is an operations milestone.
- Epic independence: pass.
- Story sizing: pass with upper-bound estimate risk noted for Stories 1.5, 3.1, 4.2, and 5.1.
- Forward dependencies: pass; final-evidence sequencing is explicit.
- Entity/schema timing: pass.
- Acceptance criteria: pass.
- FR traceability: pass.

## Summary and Recommendations

**Assessor:** Kilo, Product Manager
**Assessment date:** 2026-08-22

### Overall Readiness Status

NOT READY

The planning set has complete PRD FR coverage, coherent product epics, explicit acceptance criteria, and strong architecture/UX traceability. It is not ready to be treated as a conflict-free Phase 4 implementation authority because the final UX contract gives incompatible focus instructions and Epic 1 includes an acknowledged non-Phase-4 operations gate as an implementation story.

### Critical Issues Requiring Immediate Action

1. Resolve the `EXPERIENCE.md:259` generic HTMX-autofocus rule against the Debts retained-radio contract at `EXPERIENCE.md:131-132` and `:265`. Preserve the Debts-specific requirement: enhanced success and expected enhanced errors retain focus on the activated radio; native full-page responses may autofocus a heading.
2. Remove or reclassify Story 1.10 (`epics.md:1141-1189`) outside the Phase 4 Epic 1 implementation sequence. Keep its edge verification obligations as an owned pre-production operations gate rather than a user story.

### Recommended Next Steps

1. Amend the generic UX focus rule with an explicit retained-focus exception, synchronize any affected UX cross-references, and rerun focused UX/epic validation.
2. Move Story 1.10 to an operations/deployment tracker or dedicated pre-production gate, then update Epic 1 coverage, sprint tracking, and validation metadata.
3. Revalidate the updated epics document and refresh `validationStatus` from its stale 2026-08-12 value after both blockers are resolved.
4. Recheck the estimates for Stories 1.5, 3.1, 4.2, and 5.1 immediately before assignment; split any packet that exceeds its seven-developer-day limit without changing its dependency sequence.

### Final Note

This assessment identified 3 issues across critical, high, and minor categories. Address the two blocking issues before treating the planning artifacts as a single implementation authority. The report also confirms that all 12 PRD functional requirements have explicit epic coverage and that no forward dependency was found in the intended product-delivery sequence.

---

## Reassessment Run: Document Discovery

### Selected Canonical Inputs

- PRD: `prds/prd-debtor-2026-08-10/prd.md`
- Architecture: `architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md`
- Epics: `epics.md`
- UX: `ux-designs/ux-debtor-2026-08-10/DESIGN.md` and `ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md`

### Discovery Decision

No whole-versus-sharded canonical duplicates were found. Reconciliation, review, mockup, validation, working, and memlog files are supporting artifacts and are excluded from the canonical assessment set. The earlier assessment content above is retained as historical evidence; this reassessment uses the selected current files.

## Reassessment Run: PRD Analysis

### Functional Requirements

- **PRD-FR-1: Password-gated access.** The Administrator signs in with one configured password, remains authenticated during an active session, and signs out; no username, registration, Participant login, or multi-user authorization exists. Anonymous visitors cannot view ledger data; restart ends sessions; unsafe requests require request protection; each unsafe form has a bounded, expiring, session-bound single-use token in addition to CSRF, with distinct anonymous/authenticated pools and terminal reservation.
- **PRD-FR-2: Group lifecycle.** The Administrator creates, edits, archives, and restores Groups. History-free Groups may be deleted with unreferenced Participants; Groups with Spendings cannot be deleted. Names are trimmed/nonempty/at most 100 Unicode characters; creation accepts only a name, defaults to USD, and opens Manage; established Groups open Summary; active and archived views remain separate; archived Groups are readable but mutation-disabled.
- **PRD-FR-3: Group-owned Participants.** The Administrator adds, edits, archives, and restores Participants only within their owning Group. Identities are non-reusable across Groups and have no global management surface. New allocations require active owned Participants; referenced archived identities remain visible in history, Balances, and Transfers. Archival requires an unchanged immutable Historical exact-zero Balance/rate context; missing or invalidated rate evidence blocks it; restore has no Balance check. Names are bounded and colors are normalized `#RRGGBB` values with a server suggestion for new Participants.
- **PRD-FR-4: Record a Spending.** The Administrator creates a Spending with description, date, category, positive Total, Source Currency, exactly one Payer, and Proportional or Exact Shares. Currency/category options are closed; description/date limits apply; form defaults are prescribed; Payer and Share editing share one allocation table; validation retains values and shows inline errors.
- **PRD-FR-5: Exact allocation.** Every accepted Spending conserves its Total independently across Payer and Shares in Source Currency minor units. Amounts are positive, bounded, and precision-valid; duplicate Participants, zero/excess-precision amounts, and mismatches are rejected. Proportional allocation uses bounded decimal weights, checked integer ratios, and deterministic residual assignment; Exact allocation initializes deterministic equal minor-unit Shares, supports editing/deselection, and requires zero Remaining/Excess. Archived identities may only retain their existing role on edit.
- **PRD-FR-6: Review and maintain history.** The Administrator browses, inspects, edits, and deletes Spendings in an active Group. History is newest-first in 25-item pages; referenced archived identities remain readable by current name; Source Currency corrections follow create validation and become historical truth; edits open Exact with stored Payer/Shares; every change is atomic.
- **PRD-FR-7: Source Currency summary.** The Administrator sees current UTC-month Spending Group and per-Payer paid totals grouped by original Source Currency, independently of conversion; out-of-month Spendings are excluded.
- **PRD-FR-8: Group Currency summary.** The Administrator sees the same current-month totals in Group Currency using Spending-date historical rates. Future dates are provisional; stale eligibility is context-bound; per-Payer values accumulate exactly then quantize jointly and deterministically; unavailable quotes or checked calculation failure withhold the entire converted section without blocking Source totals, history, or mutations.
- **PRD-FR-9: Select conversion mode.** The Administrator calculates all-time Balances in Historical mode by Spending date or non-persisted Current mode by calculation date; Historical is the default.
- **PRD-FR-10: Exact Balances.** Debtor calculates one Group Currency Balance per Participant with an exact zero sum after quantization; rate-completion order cannot alter results/warnings; conversion or arithmetic failures expose no partial Balances or Transfers.
- **PRD-FR-11: Deterministic Settlement Transfers.** Debtor presents positive, deterministic Transfers that settle all Balances, never repeat a pair, contain at most `n - 1` Transfers for `n` Participants, and do not claim global minimum count.
- **PRD-FR-12: Calculation disclosure and failure isolation.** The Debts view discloses mode, calculation time, Group Currency, unique rates, and stale/provisional warnings. Missing eligible quotes produce retryable failure and never block Group, Participant, or Spending management.

**Total FRs: 12**

### Non-Functional Requirements

- **NFR-1: Correctness and historical integrity.** Decimal money/rates and currency-minor-unit rules preserve exactness without floating point; referenced history survives archival; complete Spending writes validate ownership/eligibility atomically; Spending and debt calculations use internally consistent snapshots.
- **NFR-2: Security and privacy.** Unsafe requests are authenticated where applicable and CSRF-protected; a distinct, bounded, expiring, atomically reserved single-use token suppresses replay; authentication is rate-limited with secure production cookies; secrets, identifiers, sensitive ledger/provider data, and raw diagnostics never reach logs or responses; private pages are not cached.
- **NFR-3: Availability and bounded operation.** Provider availability never gates startup, readiness, or CRUD; traffic, login, probes, locks, provider calls, caches, and sessions are bounded; admitted mutations resolve definitively rather than by generic cancellation; shutdown closes admission, drains safely, completes dispatched work, and preserves recoverability.
- **NFR-4: UX and accessibility.** Semantic server-rendered HTML/native forms are authoritative; only approved pinned HTMX infrastructure may enhance them. Current browsers are supported to 320 CSS pixels; controls are keyboard-operable, labelled, and visibly focused; contrast and error association requirements apply; validation retains values; archived and rate states are distinguishable.

**Total NFR groups: 4**

### Additional Requirements And Constraints

- Product scope is permanently single-Administrator, Group-owned Participant identity, fixed current-UTC-month summaries, twelve currencies, eight categories, and no repayment, multi-user, reusable identity, unsupported split mode, manual refresh, persistent session, custom JavaScript, multiple-instance, or external-writer feature.
- Architecture preserves `root -> web/infra -> application -> domain`, constructor-injected effects, pure deterministic domain rules, application-owned raw-input policy, checked SQLx, canonical SQLite `TEXT` money, Rust-owned financial aggregation, snapshot reads, WAL/FULL/five-second gate and lock bounds, last-commit-wins mutation semantics, and restrictive history-preserving lifecycle rules.
- Rates require lexical arbitrary-precision JSON decoding, `(source, target, requested, fetch)` context keys, deterministic bounded caches/single-flight/concurrency, defined stale/provisional rules, and synthetic same-currency rate disclosure.
- HTTP requires strict structure/CSRF/token handling before dispatch; `422` retained validation, `303` successful mutations, `409` lifecycle/token conflict, sanitised `503` unavailable outcomes, bounded pre-dispatch mutation work, and no generic timeout after dispatch.
- Authentication requires a bounded canonical Argon2id v19 configuration, process-local session/token stores and cleanup supervision, trusted-proxy-only identity, required security headers, session-free probes/static assets, and bounded admission/timeouts/readiness/shutdown.
- Edge deployment owns TLS, HTTP/2/3, forwarding sanitation, early-data prevention, body limits, backend reuse, mutation-compatible timeouts, and staged `Alt-Svc` verification; Debtor remains a private HTTP/1.1 backend.
- Delivery requires the pinned toolchain/lockfiles, independent password-helper validation, layer-owned tests, architecture fitness, dependency policy checks where applicable, refreshed SQLx metadata for SQL/migration changes, and normative-design-first synchronization.

### PRD Completeness Assessment

The current PRD is explicit, internally structured, and has 12 numbered FRs plus four cross-cutting NFR groups. The technical companion makes accounting, security, operational, deployment, and verification constraints testable. It declares no open questions or unconfirmed assumptions. Coverage, architecture, UX, and story-quality alignment remain the next assessment stages.

## Reassessment Run: Epic Coverage Validation

### Coverage Matrix

| PRD FR | Requirement | Current epic/story ownership | Status |
| --- | --- | --- | --- |
| PRD-FR-1 | Password-gated access | Epic 1, Stories 1.1-1.9; real-mutation lifecycle evidence in Story 2.1 | Covered |
| PRD-FR-2 | Group lifecycle | Epic 2, Stories 2.1, 2.2, and 2.5; Spending-backed deletion proof in Story 3.1 | Covered |
| PRD-FR-3 | Group-owned Participant lifecycle | Epics 2 and 5, Stories 2.3, 2.4, 5.4, and 5.5; historical current-name proof in Story 3.3 | Covered |
| PRD-FR-4 | Record a Spending | Epic 3, Stories 3.1-3.2 | Covered |
| PRD-FR-5 | Exact allocation | Epic 3, Stories 3.1, 3.2, and 3.4 | Covered |
| PRD-FR-6 | Review and maintain history | Epic 3, Stories 3.3-3.5 | Covered |
| PRD-FR-7 | Source Currency summary | Epic 4, Story 4.1 | Covered |
| PRD-FR-8 | Group Currency summary | Epic 4, Stories 4.2-4.3 | Covered |
| PRD-FR-9 | Select conversion mode | Epic 5, Stories 5.1-5.2 | Covered |
| PRD-FR-10 | Exact Balances | Epic 5, Stories 5.1-5.2 | Covered |
| PRD-FR-11 | Deterministic Settlement Transfers | Epic 5, Story 5.3 | Covered |
| PRD-FR-12 | Calculation disclosure and failure isolation | Epics 4 and 5, Stories 4.2-4.3 and 5.1-5.2 | Covered |

### Missing Requirements

No PRD functional requirement is missing from the current epic crosswalk. The Epic 1 range is now correctly limited to Stories 1.1-1.9; the completed HTTPS edge work is separately tracked as an operations gate and is not required to establish PRD-FR-1 coverage.

### Coverage Statistics

- Total PRD FRs: 12
- FRs covered in epics: 12
- Coverage percentage: 100%
