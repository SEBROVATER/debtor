---
title: Debtor Sprint Change Proposal
date: 2026-08-11
project: debtor
status: approved
scope: moderate
mode: incremental
trigger: implementation-readiness-report-2026-08-11.md
prepared_for: sebr
approved_by: sebr
approved_on: 2026-08-11
supersedes: earlier same-day proposal based on superseded artifact findings
---

# Sprint Change Proposal

## 1. Issue Summary

The 2026-08-11 implementation-readiness assessment found the Debtor planning set **NEEDS WORK** before Phase 4. The PRD, SPEC, normative design, architecture, and final UX contracts are complete and aligned, but `epics.md` has three implementation-gating planning defects:

1. **MQ-1:** Story 4.2 is a technical rate-evidence milestone whose Administrator value arrives only in Story 4.3.
2. **MQ-2:** A global `UX-*` owner table exists, but affected story `Requirements` lines and planned acceptance evidence do not cite the binding stable UX contracts required by AD-18.
3. **MQ-3:** Story 1.2 combines too many independently failing startup, persistence, composition, lifecycle, dependency, SQLx, and validation concerns for reliable estimation and review.

The readiness assessment discovered these defects before implementation. They are planning decomposition and traceability defects, not new product requirements, technical feasibility failures, stakeholder scope changes, or strategic pivots.

### Evidence

- The readiness report rates PRD functional coverage at 12/12 and finds no blocking PRD-to-UX or UX-to-architecture conflict.
- Story 4.2 explicitly says its work enables "later Group Currency totals" and exposes no independently usable Administrator outcome.
- None of the 29 current story-level `Requirements` lines cites a stable `UX-*` identifier despite AD-18 requiring affected stories and acceptance tests to do so.
- Story 1.2 owns workspace/toolchain architecture, dependencies, configuration, SQLite initialization, migrations, composition, startup, provider isolation, shutdown, SQLx policy, architecture fitness, and complete validation.
- `epics.md` already declares `validationStatus: pending-revalidation`, so Phase 4 remains correctly gated.

## 2. Impact Analysis

### Epic Impact

All five epic goals remain valid and retain their current order. No epic is added, removed, deferred, or redefined.

#### Epic 1

- Replace current Story 1.2 with two backward-only, runnable operator increments.
- Renumber current Stories 1.3-1.9 to 1.4-1.10.
- Update every crosswalk, owner-table entry, and story reference affected by the renumbering.
- Keep real-mutation lifecycle proof in Story 2.1 and authenticated-runtime shutdown proof in the later Epic 1 lifecycle story.

#### Epics 2 And 3

- Keep story boundaries and order.
- Add story-level stable UX citations and route-specific acceptance evidence.
- Strengthen the existing confirmation and reviewed-input criteria where final UX contracts are currently under-specified.

#### Epic 4

- Keep Story 4.1 as the provider-independent Source Currency summary.
- Merge the minimum exact historical/synthetic rate path with the first visible converted-total result in new Story 4.2.
- Renumber old Story 4.4 to 4.3 and attach advanced cache, stale-fallback, warning, and degraded-state work to that visible resilience outcome.
- Update converted-summary crosswalk ranges from `4.2-4.4` to `4.2-4.3`.

#### Epic 5

- Keep the Historical -> Current -> Settlement -> Participant archive -> restore progression.
- Preserve its backward dependency on the completed Epic 4 rate capability.
- Add story-level stable UX citations and exact acceptance evidence.

### Story Impact

The final story count remains 29:

- Splitting Story 1.2 adds one story.
- Merging old Stories 4.2 and 4.3 removes one story.

The affected numbering is:

| Current | Proposed |
|---|---|
| 1.2 Start the Complete Local Application | 1.2 Start a Persistent Local Application |
| none | 1.3 Restart and Validate the Composed Local Application |
| current 1.3-1.9 | proposed 1.4-1.10 |
| 4.2 Resolve Exact Historical Rate Evidence | merged into proposed 4.2 |
| 4.3 Review Conserved Group Currency Monthly Totals | proposed 4.2 with minimum rate capability |
| 4.4 Preserve Source Totals When Conversion Is Unavailable | proposed 4.3 with advanced cache/fallback behavior |

### Artifact Conflicts

| Artifact | Required action |
|---|---|
| PRD and addendum | None; requirements and MVP remain valid |
| Canonical SPEC and glossary | None; no contract or terminology change |
| `specs/design.md` | None; no behavior or architecture invariant changes |
| Architecture Spine | None; AD-11 and AD-18 already govern the required behavior |
| Final `DESIGN.md` and `EXPERIENCE.md` | None; the stable registry is complete |
| `epics.md` | Restructure, renumber, recrosswalk, and add story-level UX traceability/evidence |
| Readiness report | Preserve as historical evidence; rerun after epic validation |
| Sprint status | Synchronize only if a story-key tracking file exists when the approved edits are applied |

### Technical Impact

This proposal changes planning artifacts only. It triggers no immediate Rust, SQL, migration, SQLx metadata, deployment, configuration, README, or CI change.

Later implementation sequencing changes, but all existing boundaries remain mandatory: exact Decimal arithmetic, preserved history, strict request protection, inward crate dependencies, provider isolation, native-first Askama/HTMX behavior, and deterministic testing.

## 3. Recommended Approach

### Selected Path: Direct Adjustment

Apply a coordinated, moderate backlog reorganization to `epics.md`:

1. Split Story 1.2 into two runnable operator increments.
2. Renumber later Epic 1 stories and synchronize references.
3. Reslice Epic 4 into three independently useful stories.
4. Add stable UX IDs to every affected story's `Requirements` line.
5. Add route-specific acceptance evidence carrying the same IDs.
6. Rebuild the global UX owner index from finalized story ownership.
7. Audit `PRD-FR-*`, `SPEC-FR*`, `SPEC-NFR*`, `AD-*`, and `UX-*` coverage mechanically.
8. Rerun epic validation and implementation readiness before Phase 4.

### Alternatives Considered

#### Potential Rollback

Not viable. No completed implementation caused the findings, and reverting valid product, architecture, or UX decisions would not repair story decomposition or traceability.

#### PRD MVP Review

Not needed. MVP scope remains achievable and fully covered. Reducing scope would solve none of MQ-1 through MQ-3.

### Estimate And Risk

- **Planning effort:** Medium.
- **Incremental product-engineering scope:** Low to neutral; already-required work is redistributed and made explicit.
- **Risk during change:** Medium because requirement ownership, story numbering, and UX mappings can drift during movement.
- **Risk after validated change:** Low to Medium, with improved estimation, reviewability, and implementation diagnostics.
- **Timeline impact:** One focused epic-reconciliation pass, epic validation, and a new readiness assessment before Phase 4.

## 4. Detailed Change Proposals

### 4.1 Epic 4 Vertical Reslice

Artifact: `_bmad-output/planning-artifacts/epics.md`

#### Story Structure

**OLD**

```text
4.1 Review Exact Source Currency Monthly Totals
4.2 Resolve Exact Historical Rate Evidence
4.3 Review Conserved Group Currency Monthly Totals
4.4 Preserve Source Totals When Conversion Is Unavailable
```

**NEW**

```text
4.1 Review Exact Source Currency Monthly Totals
4.2 Review Conserved Group Currency Monthly Totals
4.3 Preserve Source Totals When Conversion Is Unavailable
```

#### Proposed Story 4.2

**OLD**

> As the administrator, I want monthly conversion to use exact date-appropriate rate evidence, so that later Group Currency totals are reproducible and correctly contextualized.

**NEW**

> As the administrator, I want current-month Payer totals converted with exact date-appropriate evidence into Group Currency, so that I can see reproducible totals whose displayed Group amount reconciles exactly to the displayed Payer amounts.

Proposed 4.2 owns only the provider behavior needed for its visible result:

- Historical context identity `(source, target, R, F)`.
- Synthetic same-currency evidence.
- Lexical arbitrary-precision Decimal decoding.
- Provider size, timeout, safe-diagnostic, single-flight, and concurrency bounds.
- Complete monthly snapshot release before provider I/O.
- Exact per-Payer accumulation, joint quantization, conserved Group total, and deterministic evidence disclosure.
- Whole-section withholding when fresh resolution or checked calculation fails.

#### Proposed Story 4.3

Old Story 4.4 becomes Story 4.3 and additionally owns:

- Stable-past and refreshable cache classes.
- Deterministic bounded LRU behavior and UTC rollover.
- Fixed-past stale eligibility without an age limit.
- Future-context seven-day stale eligibility.
- Stale/provisional warnings and whole-section degraded behavior.
- Source-summary and CRUD availability during conversion failure.
- Distinct inward `Unavailable` and `Calculation` causes.

#### Crosswalk Changes

**OLD**

```text
PRD-FR-8: Epic 4, Stories 4.2-4.4
PRD-FR-12 monthly ownership: Stories 4.3-4.4
```

**NEW**

```text
PRD-FR-8: Epic 4, Stories 4.2-4.3
PRD-FR-12 monthly ownership: Stories 4.2-4.3
```

**Justification:** Both rate-capability stories end with an observable Administrator outcome. No standalone diagnostic screen or new behavior is introduced, and Epic 5 still consumes a complete prior rate capability.

### 4.2 Story 1.2 Split

Artifact: `_bmad-output/planning-artifacts/epics.md`

#### Current Story

**OLD**

```markdown
### Story 1.2: Start the Complete Local Application

As the administrator,
I want Debtor to start locally from one validated command,
So that I can run a dependable private ledger without external build or provider prerequisites.
```

#### Proposed Story 1.2

**NEW**

```markdown
### Story 1.2: Start a Persistent Local Application

As the administrator,
I want one validated command to start a persistent local Debtor process,
So that I can reach my private ledger without external build or provider prerequisites.
```

Story 1.2 owns the minimum complete startup outcome:

- Pinned workspace, toolchain, dependency direction, and required configuration.
- SQLite create/connect, migrations, foreign keys, WAL, `synchronous=FULL`, and busy timeout.
- Concrete composition, bind, and non-secret local URL reporting.
- Provider-independent startup.
- Checked SQLx policy needed by the runnable application.

#### Proposed Story 1.3

**NEW**

```markdown
### Story 1.3: Restart and Validate the Composed Local Application

As the administrator,
I want Debtor to stop cleanly and restart against the same initialized local database,
So that I can trust the composed application and its local ledger lifecycle.
```

Story 1.3 owns:

- Basic no-active-mutation shutdown and storage-resource closure.
- Restart against the same initialized local database without manual migration or recreation.
- WAL sidecar recovery behavior available at this stage.
- Architecture fitness and locked workspace validation.
- SQLx metadata and dependency-policy validation.

#### Renumbering

Current Stories 1.3-1.9 become 1.4-1.10. Update the PRD-FR-1 crosswalk, UX owner table, requirements ownership, internal story references, and subsequent validation output atomically.

`SPEC-FR9` remains proved by the renumbered authenticated-session story. `SPEC-FR105` is shared by the two startup/lifecycle stories without losing coverage.

**Justification:** Both stories end with runnable operator value. The split does not create workspace-only, database-only, or validation-only scaffolding and does not weaken startup ordering.

### 4.3 Story-Level UX Traceability

Artifact: `_bmad-output/planning-artifacts/epics.md`

#### Requirements Pattern

**OLD**

```markdown
**Requirements:** ... accessibility, responsive, and strict security-header requirements.
```

**NEW**

```markdown
**Requirements:** ...; UX contracts: `UX-SHELL-01`, `UX-TARGET-01`,
`UX-FOCUS-01`, `UX-STATUS-01`, `UX-RESPONSIVE-01`, `UX-VISUAL-01`.

**UX acceptance evidence:** Route-specific tests cite the same contracts and
verify their exact native/enhanced, focus, status, geometry, viewport, zoom,
and visual consequences for the routes and states introduced by this story.
```

The evidence block must be specialized per story. Merely asserting compliance with a bundle is insufficient.

#### Story Mapping Rules

- Non-web operator stories receive no artificial UX IDs.
- Login, Group, summary, debt, and lifecycle pages cite their applicable shell, target, focus, status, responsive, and visual bundles.
- Allocation stories additionally cite `UX-ALLOC-01`, `UX-PREVIEW-NATIVE-01`, and `UX-PREVIEW-LATEST-01` where they render, review, or commit allocation state.
- Destructive Group, Spending, and Participant flows cite `UX-CONFIRM-01`.
- The owner index is regenerated after Epic 1 and Epic 4 renumbering rather than maintained as a divergent source.

#### Objective Evidence Required

Where applicable, story criteria must prove:

- 48 by 48 CSS-pixel controls at 320 CSS pixels and 400% zoom.
- Exact stable focus destinations for success, error, cancellation, pagination, and enhanced updates.
- Stable scoped status nodes, `aria-busy`, atomic polite announcements, and alert exceptions.
- Narrow `100dvh`, safe-area, wide normal-flow, focused-form, and one-scroll-owner behavior.
- Dark Editorial Contrast visual tokens, square geometry, component states, and prohibited decorative motion/depth.
- Allocation table geometry `520/116/76/76/92/160`, labeled internal scrolling, sticky identity, and no page-level horizontal scroll.
- Native reviewed non-editable Preview and enhanced latest-input-wins revisions with superseded-response rejection and interaction-state preservation.
- Full server-rendered confirmation pages with named scope, reversibility, protected one-shot submission, allow-listed cancel targets, and deterministic success focus.

#### Material Story Corrections

- **Story 2.5:** Replace direct Group archive/delete submission and vague contextual returns with `UX-CONFIRM-01` confirmation and exact focus destinations.
- **Story 3.3:** Prove native and enhanced commit cannot bypass the currently reviewed input; disable approval for pending, stale, invalid, or superseded Preview state.
- **Story 3.6:** Complete Spending deletion confirmation, cancel, one-shot submit, pagination-safe return, and focus criteria.
- **Story 5.4:** Add the Participant archive confirmation surface without caching or bypassing server-owned eligibility and final revalidation.

**Justification:** This satisfies AD-18 at both story and acceptance-evidence level while preserving the final UX contracts as upstream authority.

## 5. Implementation Handoff

### Scope Classification

**Moderate:** Product and architecture direction remain stable, but backlog boundaries, identifiers, crosswalks, and acceptance traceability require coordinated reorganization.

### Recipients

| Recipient | Responsibility |
|---|---|
| Product Owner / planning owner | Apply approved story decomposition, numbering, and crosswalk changes |
| Developer / technical reviewer | Verify runnable vertical slices, backward-only dependencies, and complete technical requirement preservation |
| UX acceptance reviewer | Verify each affected story and planned test cites and proves the applicable stable UX contracts |
| Readiness assessor | Rerun epic validation and implementation readiness before Phase 4 |

### Execution Order

1. Split Story 1.2 and renumber Epic 1.
2. Reslice Epic 4 and update its crosswalks.
3. Add story-level UX contract citations and route-specific evidence.
4. Regenerate the global UX owner index.
5. Audit all source-qualified requirements and internal references.
6. Keep `validationStatus: pending-revalidation` while edits are unvalidated.
7. Rerun epic validation.
8. Rerun implementation readiness.
9. Synchronize sprint status if a tracked story-key file exists.

### Success Criteria

- Every story delivers independently usable user or operator value using only earlier capabilities.
- Epic order remains unchanged and no story depends on future implementation for its completion value.
- All 12 PRD FRs and every applicable `SPEC-FR*`, `SPEC-NFR*`, `AD-*`, and `UX-*` obligation retain an explicit owner.
- Story 4.2 ends with visible exact converted monthly totals rather than technical evidence for a later story.
- Story 1.2 no longer owns restart/lifecycle and complete validation as one inseparable acceptance surface.
- Every affected web story and planned acceptance test cites and objectively proves applicable stable UX IDs.
- PRD, SPEC, normative design, architecture, and final UX contracts remain unchanged.
- Epic validation passes before `validationStatus` changes.
- A new implementation-readiness assessment removes MQ-1, MQ-2, and MQ-3 before Phase 4 starts.

## 6. Checklist Record

### Understand The Trigger And Context

- `[x] 1.1` Triggering stories identified: 4.2 for MQ-1 and 1.2 for MQ-3; MQ-2 is artifact-wide.
- `[x] 1.2` Core problem classified as planning decomposition and traceability defects.
- `[x] 1.3` Readiness evidence and initial implementation-gate impact recorded.

### Epic Impact Assessment

- `[x] 2.1` All five epics remain completable.
- `[!] 2.2` Epics 1 and 4 require approved story restructuring; all affected web stories require traceability edits.
- `[x] 2.3` Later dependencies remain valid after synchronized renumbering.
- `[N/A] 2.4` No new or obsolete epic.
- `[x] 2.5` Epic order remains unchanged; corrections precede Phase 4.

### Artifact Conflict And Impact Analysis

- `[x] 3.1` PRD and MVP remain valid.
- `[x] 3.2` Architecture remains valid; revised stories must obey existing AD-11 and AD-18.
- `[x] 3.3` Final UX contracts remain valid; only downstream story traceability changes.
- `[!] 3.4` `epics.md` requires edits and subsequent epic/readiness validation.

### Path Forward Evaluation

- `[x] 4.1` Direct Adjustment is viable: Medium planning effort, Low-to-Medium validated risk.
- `[N/A] 4.2` Rollback is not useful.
- `[N/A] 4.3` PRD MVP review is not needed.
- `[x] 4.4` Direct Adjustment with Moderate backlog reorganization selected.

### Proposal And Handoff

- `[x] 5.1` Issue summary created.
- `[x] 5.2` Epic, story, artifact, and technical impacts documented.
- `[x] 5.3` Recommended path and alternatives documented.
- `[x] 5.4` MVP impact and sequenced action plan defined.
- `[x] 5.5` Moderate-scope handoff defined.
- `[x] 6.1` Applicable checklist sections reviewed; remaining action items are captured in the handoff.
- `[x] 6.2` Proposal checked for consistency and actionability.
- `[x] 6.3` Final explicit approval received from sebr on 2026-08-11.
- `[N/A] 6.4` No `sprint-status.yaml` or `sprint-status.yml` exists in the project.
- `[x] 6.5` Moderate-scope handoff activated with recipients, sequence, and success criteria defined.

## 7. Workflow Execution Log

| Date | Event | Result |
|---|---|---|
| 2026-08-11 | Correct Course activation and artifact discovery | Required PRD and epics found; architecture, UX, SPEC, normative design, project context, and readiness evidence loaded |
| 2026-08-11 | Change trigger confirmed | MQ-1, MQ-2, and MQ-3 selected as one coordinated correction |
| 2026-08-11 | Review mode selected | Incremental |
| 2026-08-11 | Checklist Sections 1-4 completed | Direct Adjustment selected; Moderate scope |
| 2026-08-11 | Epic 4 vertical reslice reviewed | Approved by sebr |
| 2026-08-11 | Story 1.2 split reviewed | Approved by sebr |
| 2026-08-11 | Story-level UX traceability reviewed | Approved by sebr |
| 2026-08-11 | Complete proposal reviewed | Continued without revision |
| 2026-08-11 | Final implementation approval | Approved by sebr |
| 2026-08-11 | Handoff | Routed as Moderate to planning owner, Developer/technical reviewer, UX acceptance reviewer, and readiness assessor |
