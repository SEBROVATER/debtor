---
title: Debtor Sprint Change Proposal
date: 2026-08-22
project: debtor
status: approved-for-implementation
scope: moderate
mode: batch
trigger: implementation-readiness-report-2026-08-22.md
prepared_for: sebr
approved_by: sebr
approved_on: 2026-08-22
---

# Sprint Change Proposal

## 1. Issue Summary

The 2026-08-22 implementation-readiness assessment reports that the planning set is not ready to act as a conflict-free Phase 4 authority, despite complete PRD coverage and otherwise valid epic sequencing.

Two planning defects require correction:

1. `EXPERIENCE.md` gives incompatible focus instructions. Its generic Interaction Focus Matrix rule requires every forward full-page or enhanced response to render an `autofocus` target, while the Debts interaction explicitly requires enhanced success and expected errors to retain focus on the activated Historical or Current radio. An enhanced Debts fragment following the generic rule could steal focus.
2. Story 1.10 is recorded as a completed Epic 1 application story even though its own acceptance criteria and implementation record identify it as a completed pre-production operations gate, not a Phase 4 application increment. The Caddy 2.11.2 decision, edge template, verification runbook, and supporting documentation already exist; this is a classification and tracking defect, not unfinished edge work.

Evidence:

- `implementation-readiness-report-2026-08-22.md:146-152` identifies the focus contradiction and its keyboard-accessibility risk.
- `implementation-readiness-report-2026-08-22.md:173-175` identifies Story 1.10 as a non-Phase-4 operations milestone incorrectly included in Epic 1.
- `EXPERIENCE.md:131-132,259,265` contains the conflicting focus statements.
- `epics.md:1141-1189`, `implementation-artifacts/1-10-define-the-pre-production-https-edge-gate.md`, and `sprint-status.yaml:61` show the Story 1.10 classification mismatch.

## 2. Impact Analysis

| Area | Impact |
| --- | --- |
| Epic 1 | Remove Story 1.10 from the Phase 4 Epic 1 sequence and update the PRD-FR-1 crosswalk from Stories 1.1-1.10 to Stories 1.1-1.9. No remaining application story is removed or renumbered. |
| Completed edge work | Preserve the completed implementation record and its linked ADR, Caddy template, runbook, README, and configuration evidence. Reclassify the record and sprint tracker as a completed pre-production operations gate. |
| UX contract | Clarify generic focus ownership so only enhanced interactions whose matrix row specifies a forward destination render `autofocus`; interactions whose row requires retention, including Debts, render no forward autofocus target. |
| PRD and MVP | No change. The product boundary, all 12 PRD functional requirements, and release scope remain unchanged. |
| Architecture and deployment behavior | No change. `specs/design.md`, ADR 0003, the architecture spine, Caddy 2.11.2 selection, and existing edge verification obligations remain authoritative. |
| Sprint tracking | Remove Story 1.10 from `development_status`; add a separate completed operations-gate entry with the existing record path and owner. Refresh `epics.md` validation metadata after validation. |

## 3. Recommended Approach

**Direct adjustment, moderate scope.** Make two targeted planning corrections, preserve the existing delivered edge evidence, then revalidate the planning set.

- Effort: Low implementation effort; moderate planning and tracker synchronization effort.
- Risk before correction: Medium. The UX contradiction can produce an accessibility regression, and the tracker can incorrectly treat operations work as a Phase 4 application increment.
- Residual risk: Low after focused UX, epic-reference, and sprint-tracker validation.
- Timeline impact: One documentation/tracking correction and readiness revalidation. No code rollback, product replan, or MVP reduction is required.

### Alternatives Considered

| Option | Decision | Rationale |
| --- | --- | --- |
| Direct adjustment | Selected | Both defects are localized to planning authority and tracking. |
| Rollback | Not viable | Story 1.10's completed edge decision and deployment evidence are valid and should be retained. |
| MVP review | Not viable | No product requirement, feasibility assumption, or release boundary is invalidated. |

## 4. Detailed Change Proposals

### 4.1 Clarify Enhanced Focus Ownership

**Artifact:** `ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md`  
**Section:** Interaction Focus Matrix introduction and Accessibility Floor

**OLD:**

> Each forward full-page or HTMX response renders exactly one allow-listed destination with `autofocus` where the element permits it.

**NEW:**

> Each forward full-page response renders exactly one allow-listed destination with `autofocus` where the element permits it. An enhanced response follows its Interaction Focus Matrix row: it renders one allow-listed autofocus destination only when that row specifies forward focus; when the row specifies retained focus, it renders no autofocus target and retains the activated control. Focusable headings also receive `tabindex="-1"`.

**Companion update:** Replace the generic Accessibility Floor statement that all forward responses render one autofocus target with the same full-page/interaction-specific enhanced distinction.

**Rationale:** This makes `UX-FOCUS-01` internally consistent. It preserves the required native Debts heading focus while preventing enhanced Debts success and expected errors from stealing focus from the activated radio.

### 4.2 Remove the Operations Gate From Epic 1

**Artifact:** `epics.md`  
**Sections:** PRD-FR-1 crosswalk, Epic 1 story sequence, and frontmatter validation status

**OLD:**

> `PRD-FR-1` ... Epic 1, Stories 1.1-1.10

> ### Story 1.10: Define the Pre-Production HTTPS Edge Gate

**NEW:**

> `PRD-FR-1` ... Epic 1, Stories 1.1-1.9

> Story 1.10 is removed from the Epic 1 Phase 4 story sequence. Its completed evidence is tracked as the separate Pre-Production HTTPS Edge Gate in sprint operations tracking.

Set `validationStatus` to a pending-revalidation value during the edit, then replace it with the current revalidation date and status only after focused validation passes.

**Rationale:** Epic 1 remains a coherent Phase 4 application-delivery sequence. The already-complete edge decision is preserved but no longer misrepresented as an Administrator-facing implementation story.

### 4.3 Reclassify the Completed Story Record

**Artifact:** `implementation-artifacts/1-10-define-the-pre-production-https-edge-gate.md`  
**Section:** Frontmatter, title/status, and source references

**OLD:**

> `story_key: 1-10-define-the-pre-production-https-edge-gate`

> `story_id: 1.10`

> `epic: 1`

> `status: done`

**NEW:**

> `gate_key: pre-production-https-edge`

> `classification: operations-gate`

> `status: completed`

> `owner: Operations / Architect`

Retain the document body and all completion evidence, but identify it as a historical operations-gate record rather than an Epic 1 user story. Update its `epics.md` source reference to the dedicated operations-gate tracking entry.

**Rationale:** The completed delivery evidence remains auditable without retaining a false Phase 4 story identity.

### 4.4 Synchronize Sprint Tracking

**Artifact:** `implementation-artifacts/sprint-status.yaml`  
**Section:** `development_status`

**OLD:**

> `1-10-define-the-pre-production-https-edge-gate: done`

**NEW:**

> Remove the Story 1.10 line from `development_status`.

> Add `operations_gates.pre-production-https-edge` with status `completed`, owner `Operations / Architect`, record path `1-10-define-the-pre-production-https-edge-gate.md`, and a condition that production rollout requires its documented verification evidence.

**Rationale:** Separates Phase 4 application progress from deployment-readiness governance while retaining a clear release gate.

## 5. Implementation Handoff

**Classification:** Moderate. This requires planning, UX, and operations-tracker coordination; no application code change is authorized by this proposal.

| Recipient | Responsibility |
| --- | --- |
| Product Owner / planning owner | Apply the Epic 1 removal, crosswalk update, and validation metadata refresh. |
| UX owner | Apply the `UX-FOCUS-01` clarification and confirm native/enhanced focus-matrix consistency. |
| Operations / Architect | Reclassify the completed edge record and own the separate production-readiness gate. |
| Developer | Verify no application code, route, or deployment behavior changed; update only the approved planning/tracking artifacts. |
| Readiness assessor | Rerun focused UX/epic validation and the full implementation-readiness assessment. |

### Success Criteria

- `EXPERIENCE.md` has no generic autofocus rule that conflicts with enhanced Debts retained-radio focus.
- Native Debts results may autofocus their headings; enhanced Debts success and expected errors retain the activated radio and render no competing autofocus target.
- Epic 1 contains only Stories 1.1-1.9 in its Phase 4 sequence and crosswalks.
- The completed Caddy edge work remains preserved as a completed pre-production operations gate.
- `sprint-status.yaml` tracks the gate separately from Phase 4 development stories.
- `epics.md` validation metadata reflects a new successful revalidation.
- A follow-up readiness assessment reports the two blockers resolved.

## 6. Checklist Record

- `[x]` 1.1: Triggering sources are the 2026-08-22 readiness report, `EXPERIENCE.md`, Epic 1, and the completed Story 1.10 record.
- `[x]` 1.2: Issue type is a planning/contract contradiction plus an operations-work classification error.
- `[x]` 1.3: Evidence is documented in the issue summary.
- `[x]` 2.1-2.5: Epic impact assessed; Epic 1 is corrected without resequencing the remaining product epics.
- `[x]` 3.1: PRD and MVP conflict check completed; no change required.
- `[x]` 3.2: Architecture check completed; no behavioral or technology change required.
- `[x]` 3.3: UX conflict identified and targeted correction proposed.
- `[x]` 3.4: Secondary planning and sprint-tracking artifacts identified.
- `[x]` 4.1: Direct adjustment is viable; low effort, medium coordination risk.
- `[N/A]` 4.2: Rollback is not applicable.
- `[N/A]` 4.3: MVP review is not justified.
- `[x]` 4.4: Direct adjustment selected.
- `[x]` 5.1-5.5: Issue summary, impact, recommendation, artifact edits, and handoff are defined.
- `[x]` 6.3: User approved the proposal on 2026-08-22.
- `[x]` 6.4: Epic and Story 1.10 sprint-tracker references are synchronized; the operations gate is tracked separately.
- `[x]` 6.5: Moderate-scope handoff is routed to planning, UX, operations, and readiness review.

## 7. Workflow Execution Log

| Date | Event | Result |
| --- | --- | --- |
| 2026-08-22 | Correct Course initialized | Workflow configuration, project context, PRD, epics, architecture, UX, and current sprint tracker loaded. |
| 2026-08-22 | Trigger resolved | User identified the implementation-readiness check; latest report selected as evidence. |
| 2026-08-22 | Batch analysis completed | Two localized planning blockers confirmed; no PRD, MVP, architecture behavior, or code rollback required. |
| 2026-08-22 | Proposal drafted | Awaiting approval before applying the proposed artifact changes. |
| 2026-08-22 | Proposal approved and applied | UX focus wording, Epic 1 sequencing, operations-gate record, and sprint tracking were synchronized. |
