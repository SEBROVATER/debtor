---
title: Debtor Sprint Change Proposal
date: 2026-08-12
project: debtor
status: approved-for-implementation
scope: moderate
mode: batch
trigger: implementation-readiness-report-2026-08-12.md
prepared_for: sebr
approved_by: sebr
approved_on: 2026-08-12
---

# Sprint Change Proposal

## 1. Issue Summary

The 2026-08-12 revalidation found two remaining planning blockers before Phase 4 assignment:

1. Stories 3.1, 3.2, and 3.3 duplicate ownership of complete Spending commit behavior. Stories 3.1 and 3.2 promise usable committed outcomes, but Story 3.3 separately claims the same revalidation, transactional aggregate persistence, canonical persistence, reviewed-input binding, and first Spending-backed Group-deletion proof.
2. Stories 1.2, 1.5, 1.7, 2.1, 4.2, 4.3, 5.1, and 5.4 remain unbounded despite the existing sizing commitment. They combine unrelated route, infrastructure, persistence, concurrency, operational, UX, and verification work without concrete one-developer estimates or ordered implementation packets.

The trigger is a planning-quality failure. No product requirement, accounting invariant, architecture decision, UX contract, database schema, or deployed code is being changed.

## 2. Impact Analysis

| Area | Impact |
| --- | --- |
| Epic 3 | Correct first-consumer ownership: Story 3.1 owns the shared create aggregate path and first `SPEC-FR29` proof; Story 3.2 reuses it for Exact creation. Replace duplicate Story 3.3 with the existing user-visible browse/inspect outcome and renumber later maintenance stories accordingly. |
| Epics 1, 2, 4, and 5 | Replace the generic future sizing promise with concrete, sequenced one-developer implementation packets. Split a packet before assignment if it cannot meet its declared boundary. |
| Final-Evidence Ledger | Retain it. Update all Story 3 references so only the corrected first persistence owner can close `SPEC-FR29`. |
| PRD and MVP | No change. All 12 functional requirements remain covered; no scope reduction or deferral is proposed. |
| Architecture and UX | No behavior or contract change. AD-3 through AD-18 and all applicable `UX-*` contracts continue to govern the recut stories. |
| Sprint tracking | No `sprint-status.yaml` exists. The corrected epic document becomes the handoff source and readiness must be rerun after it is updated. |

## 3. Recommended Approach

**Direct adjustment, moderate scope.** Preserve the five approved epics, revise story ownership, and make sizing decisions executable before implementation assignment.

- Planning effort: Medium.
- Implementation risk before correction: High for Spending duplication; Medium to High for unbounded stories.
- Residual risk after correction: Medium, controlled by first-consumer ownership, ordered packets, and the Final-Evidence Ledger.
- Timeline impact: One story-plan correction and readiness revalidation before Phase 4 assignments.

Rollback is not applicable because implementation has not started. An MVP review is not recommended because the issue is story decomposition rather than scope or feasibility.

## 4. Detailed Change Proposals

### 4.1 Story 3.1: First Consumer Owns Complete Proportional Creation

**Section:** Acceptance Criteria and Requirements

**OLD:**

> Story 3.1 approves a Proportional Spending and atomically persists it, while Story 3.3 independently owns server-side commit revalidation, write-gated aggregate persistence, canonical decimal persistence, reviewed-input binding, and the first referenced-Group deletion proof.

**NEW:**

> Story 3.1 is the sole first-consumer owner of the create-Spending aggregate path. It owns reviewed-input binding, server reparse/revalidation, application/domain allocation reuse, write-gated transaction eligibility rechecks, canonical decimal persistence, and the first persisted-Spending proof that a referenced Group cannot be deleted. It remains a complete Proportional preview-to-Transactions outcome.

**Rationale:** A user-visible create story cannot be complete while its mandatory atomic persistence path belongs to a later story. One first-consumer owner prevents parallel write paths and establishes the reusable create contract before Exact mode.

### 4.2 Story 3.2: Exact Creation Reuses the Established Aggregate Path

**Section:** Acceptance Criteria and Requirements

**OLD:**

> Story 3.2 independently claims atomic complete-Spending persistence and all generic create integrity behavior.

**NEW:**

> Story 3.2 owns Exact-mode initialization, editing, difference display, Preview, review, and approved creation. Its commit acceptance explicitly reuses Story 3.1's established complete-aggregate command and transaction path; it adds no second repository, reviewed-input, or create-persistence implementation.

**Rationale:** This leaves Story 3.2 as a vertical Exact user outcome while keeping shared integrity behavior singular.

### 4.3 Epic 3 Story Sequence: Remove Duplicate Integrity Story

**Section:** Story titles, numbering, crosswalk, and Final-Evidence Ledger

**OLD:**

| Story | Current purpose |
| --- | --- |
| 3.3 | Preserve Complete Spending Integrity, duplicating create persistence and reviewed-input behavior. |
| 3.4 | Browse and Inspect Spending History. |
| 3.5 | Correct an Existing Spending. |
| 3.6 | Delete a Spending Atomically. |

**NEW:**

| Story | Proposed purpose |
| --- | --- |
| 3.3 | Browse and Inspect Spending History, including canonical hydration/corruption rejection and direct complete aggregate reads. |
| 3.4 | Correct an Existing Spending, reusing the established aggregate replacement path. |
| 3.5 | Delete a Spending Atomically. |

All crosswalk, ledger, dependency, UX-owner, and later-story references must be renumbered in the same edit. `SPEC-FR29` final evidence remains Story 3.1.

**Rationale:** History inspection is a distinct Administrator outcome. The current Story 3.3 is a duplicate technical milestone with no independent user job and should be removed rather than retained as a parallel integrity owner.

### 4.4 Replace Generic Sizing Promise With Assignment Packets

**Section:** `Sizing Commitments`

**OLD:**

> Before assignment, the planning owner records a one-developer estimate and an ordered implementation checklist for high-complexity stories.

**NEW:**

| Story | One-developer packet boundary | Estimate | Assignment rule |
| --- | --- | --- | --- |
| 1.2 | Validated config, SQLite startup/migration, root composition, and provider-independent local socket admission. | 3-5 days | Move restart, shutdown, architecture/dependency governance, and broad SQL verification to their existing owners. |
| 1.4 | Anonymous Login session, CSRF, and one-per-session submission-token issuance with bounded capacity, expiry, and cleanup. | 3-5 days | Establish only the anonymous Login first-consumer path; do not add authenticated page-scoped tokens here. |
| 1.5 | Trusted-client resolution, strict Login admission, anonymous-token reservation, bounded Argon2 verification, limiter, and durable promotion through the Login route. | 4-6 days | Extract proxy-policy/config validation or session-store internals only if the route packet exceeds estimate. |
| 1.7 | Authenticated page-scoped token issuance and non-Login route-neutral extractor extension, reusing the completed Login/Sign-out token path. | 4-6 days | Prove authenticated cross-route races and cleanup; do not reimplement Login or Sign-out issuance/reservation. |
| 2.1 | Create Group from Groups through a real dispatched ledger mutation and Manage redirect. | 4-6 days | Keep `SPEC-FR103`/`SPEC-FR104` final evidence here; split reusable mutation-executor plumbing only into a first-consumer subtask within this packet. |
| 3.1 | Proportional Preview, reviewed approval, and the sole shared create aggregate path. | 5-7 days | If persistence/migration evidence cannot fit, split an explicitly named prerequisite packet that still lands with this story before Exact work begins. |
| 3.2 | Exact Preview, reviewed approval, and reuse of 3.1's create aggregate path. | 3-5 days | Do not reimplement shared persistence. |
| 4.2 | Fresh/synthetic Historical conversion plus exact monthly Group-Currency projection and disclosure. | 5-7 days | Keep cache fallback/rollover behavior in 4.3; do not combine it with fresh conversion. |
| 4.3 | Stable/refreshable cache admission, stale fallback, rollover, and whole-section unavailable behavior. | 4-6 days | Reuse 4.2 conversion projection; do not create a second summary calculation. |
| 5.1 | Historical snapshot, quote orchestration, exact zero-sum Balance calculation, and Debts result/disclosure. | 5-7 days | Keep Current mode, Settlement, and archival admission in their designated later stories. |
| 5.4 | Confirmed zero-Balance Participant archive attempt with snapshot/epoch/date/quote revalidation and lifecycle commit. | 5-7 days | Reuse Story 5.1's Historical engine; do not create a parallel calculator. |

Each packet must have an ordered checklist that names its one route/use case, application/domain work, adapter work, route-specific UX evidence, tests, and validation commands. Any packet estimated above seven developer days, or containing unrelated deliverables after decomposition, must be split before assignment and revalidated.

**Rationale:** The existing generic rule records intent but does not make an assignment safe. The packets create immediate scope boundaries while preserving first-consumer ownership and existing epic order.

### 4.5 Definition of Done Boundary

**Section:** Cross-Cutting Story Rule

**OLD:**

> Repeated cross-cutting concerns appear in compound acceptance criteria throughout each story.

**NEW:**

> Keep route- and invariant-specific Given/When/Then criteria in each story. Move repeated workspace validation commands, generic diagnostic policy, broad brownfield dispositions, and global UX matrix repetition into a stable story Definition of Done/checklist referenced by the story. The checklist cannot close a requirement ahead of the Final-Evidence Ledger.

**Rationale:** This reduces review ambiguity without weakening any acceptance requirement or removing route-specific proof.

### 4.6 Epic 1 Submission-Token First-Consumer Sequence

**Section:** Stories 1.4-1.7, Assignment Packets, and Final-Evidence Ledger

**OLD:**

> Story 1.7 owns the shared submission-token lifecycle and strict unsafe-form boundary, while Stories 1.4-1.6 require issuance, reservation, expiry/capacity behavior, and replay rejection.

**NEW:**

> Story 1.4 owns anonymous Login-token issuance, one-per-session capacity, ten-minute expiry, and cleanup. Story 1.5 owns Login-token validation, atomic reservation immediately before password-verification dispatch, and terminal replay behavior. Story 1.6 reuses the established path for Sign out. Story 1.7 extends that established implementation to authenticated page-scoped token issuance, the non-Login route-neutral extractor, and cross-route concurrency/cleanup verification. It must not become a prerequisite for Login or Sign out.

**Rationale:** Each unsafe outcome has the functionality it needs in a preceding runnable story. The extension story remains a bounded cross-route outcome rather than retrospectively owning already-required Login and Sign-out behavior.

## 5. Implementation Handoff

**Classification:** Moderate, requiring planning-owner and Developer coordination.

| Recipient | Responsibility |
| --- | --- |
| Planning owner / Product Owner | Apply the approved Epic 3 renumbering, maintain traceability, create the concrete assignment packets, and update the Final-Evidence Ledger references. |
| Developer | Implement only an approved packet at a time. Story 3.1 establishes the sole create aggregate path; Story 3.2 and later flows reuse it. |
| Readiness assessor | Rerun implementation readiness after `epics.md` is corrected, validating coverage, ownership, packet boundaries, and final-evidence tracking. |
| Operations / Architect | Keep Story 1.10 as the separate pre-production edge gate; select the edge product, version, configuration, and verification environment before production rollout. |

The planning owner must also apply the Epic 1 first-consumer sequence in Section 4.6 and keep `SPEC-FR15..SPEC-FR19` evidence aligned with its first runnable consumer.

### Success Criteria

- Exactly one create-Spending persistence path is owned by Story 3.1.
- Story 3.2 is a complete Exact outcome that reuses Story 3.1's path.
- No duplicate Story 3.3 integrity milestone remains; all Story 3 references and IDs are synchronized.
- Each named high-complexity story has a concrete packet, estimate, ordered checklist, and pre-assignment split rule.
- `SPEC-FR29`, `SPEC-FR30`, `SPEC-FR40`, `SPEC-FR103`, and `SPEC-FR104` remain unclosable until their ledgered final evidence succeeds.
- A follow-up readiness assessment reports no duplicate ownership or unbounded pre-assignment story scope.
- Login and Sign out have no dependency on later Story 1.7; authenticated-form replay protection extends, rather than duplicates, their established path.

## 6. Checklist Record

- `[x]` 1.1-1.3: Readiness report establishes the trigger, issue type, and evidence.
- `[x]` 2.1-2.5: Epic impact assessed; five epic boundaries and order remain valid.
- `[x]` 3.1: PRD/MVP conflict checked; no product requirement change.
- `[x]` 3.2: Architecture checked; first-consumer aggregate ownership and no parallel paths are required by AD-2, AD-3, AD-5, and AD-6.
- `[x]` 3.3: UX checked; all existing `UX-*` route contracts remain applicable with no visual or flow change.
- `[x]` 3.4: Secondary artifacts identified: `epics.md`, Final-Evidence Ledger, crosswalks, readiness report, and proposal history.
- `[x]` 4.1: Direct adjustment is viable; medium planning effort and controlled risk.
- `[N/A]` 4.2: Rollback is not applicable; no implementation is being reverted.
- `[N/A]` 4.3: MVP reduction is not justified; requirements and feasibility remain intact.
- `[x]` 4.4: Direct adjustment selected.
- `[x]` 5.1-5.5: Proposal, handoff roles, dependencies, and success criteria defined.
- `[x]` 6.3: User approved correction of every identified planning issue on 2026-08-12.
- `[N/A]` 6.4: No `sprint-status.yaml` exists.

## 7. Workflow Execution Log

| Date | Event | Result |
| --- | --- | --- |
| 2026-08-12 | Correct Course initialized | Workflow customization, project context, and all required planning artifacts loaded. |
| 2026-08-12 | Trigger confirmed | User selected both readiness blockers and batch review mode. |
| 2026-08-12 | Impact analysis completed | Planning-only moderate change; no PRD, architecture, UX, MVP, or code change required. |
| 2026-08-12 | Batch proposal drafted | Awaited explicit approval before artifact changes. |
| 2026-08-12 | Proposal approved | `epics.md` corrected: sole Story 3.1 aggregate ownership, Story 3.2 reuse, duplicate integrity story removal, and bounded assignment packets. |
| 2026-08-12 | Epic 1 sequencing correction approved | Stories 1.4-1.6 now own their required first-consumer token behavior; Story 1.7 is an authenticated/general-route extension. |
