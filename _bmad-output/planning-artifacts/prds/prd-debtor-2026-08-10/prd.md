---
title: Debtor Product Requirements Document
status: final
created: 2026-08-10
updated: 2026-08-10
---

# Debtor Product Requirements Document

## Document Purpose

This PRD is the final product and UX handoff for downstream planning and implementation. `addendum.md` carries its technical companion contract. `specs/design.md` remains normative and governs any conflict. If the artifacts diverge, downstream work must stop until all three are reconciled; no reader may silently choose one interpretation.

## Vision

Debtor is a private shared-expense ledger for one administrator who wants dependable multi-currency accounting without the redundant collaboration features or deployment complexity of existing alternatives. It should make it straightforward to record shared Spendings and understand who owes whom while remaining deliberately small, self-operated, and trustworthy.

Debtor treats participants as accounting identities rather than application users. It calculates advisory settlement transfers on demand but does not manage repayment execution or settled state.

## Target User

Debtor is for one administrator who records both personal and shared Spendings in Groups. The administrator may use a Group for ongoing personal tracking or for shared activity such as travel with friends. Friends represented as Participants do not use Debtor themselves, and each Participant identity belongs to one Group.

### Jobs To Be Done

- Record who paid and who benefited without creating accounts for every Participant.
- Preserve original spending currencies while obtaining a coherent view in the Group Currency.
- Review the selected Group's current-month spending total and each Participant's paid total.
- Understand the Group's all-time balances and receive advisory Settlement Transfers.
- Correct or archive ledger information without losing referenced history.

### Lightweight User Journey

- **UJ-1. Sebr reviews a travel group's month and all-time debts.** Sebr signs in as the sole Administrator, opens the travel Group, and records shared Spendings paid by one or more friends. On the Group page, Sebr sees current-month totals by Source Currency and equivalent totals in the Group Currency, including how much each Participant paid. Sebr then opens the all-time debts view to see current balances and advisory Settlement Transfers. If exchange-rate conversion is temporarily unavailable, the Source Currency summary and ledger operations remain usable while the converted summary reports a retryable failure.

## Success Metrics

**Primary**
- **SM-1:** After four consecutive weeks of real use, the Administrator has recorded every intended Spending for at least one active Group and can answer that Group's current-month total, each Payer's current-month paid total, and all-time debts without an external ledger or calculation. Validates FR-4, FR-7, FR-8, FR-10, and FR-11.

**Quality**
- **SM-2:** Every accepted Spending conserves its Total exactly across both Payers and Shares; every completed debt calculation has zero-sum Balances and Settlement Transfers that fully settle those Balances. Validates FR-5, FR-10, and FR-11.
- **SM-3:** Exchange-rate failure leaves Source Currency summaries and ledger management usable. Validates FR-7, FR-8, and FR-12.

**Counter-metrics**
- **SM-C1:** Feature count must not increase merely to match competitor checklists.
- **SM-C2:** Deployment flexibility must not expand at the expense of simple operation.
- **SM-C3:** Transfer-count minimization must not displace deterministic, exact settlement.

## Experience Principles

- **Group-centered:** After login, the main page lists Groups. Spendings, monthly summaries, Participant management, and all-time debts remain within the selected Group.
- **Progressive disclosure:** Adding a Spending stays behind a compact disclosure or overlay rather than occupying the Group page permanently.
- **One web experience:** Debtor has one mobile-friendly web interface. It need not provide separate mobile and desktop designs or optimize its appearance for large desktop screens.
- **Minimal and modern:** The interface avoids redundant animation and decorative effects. Core interaction uses native HTML and CSS behavior and remains functional without custom JavaScript.

## UX Acceptance Requirements

- Core behavior must work through semantic server-rendered HTML without custom JavaScript.
- The single web experience must be mobile-friendly and remain usable on desktop without requiring a separate desktop design.
- The interface must remain usable in the latest stable versions of Chrome, Firefox, Safari, and Edge at viewport widths down to 320 CSS pixels.
- Every control must be reachable and operable without a pointer and must have a programmatic label and a visible focus indicator that is at least two CSS pixels thick and has at least 3:1 contrast against adjacent colors.
- Normal text must reach 4.5:1 contrast; large text, user-interface components, and meaningful graphics must reach 3:1. Inline errors must be programmatically associated with their fields. Formal accessibility certification is not required.
- Validation must identify errors inline and retain submitted values.
- Archived state and stale, provisional, or unavailable conversion results must be visibly distinguishable.

## MVP Scope

### In Scope

- Password-gated single-Administrator web access.
- Group lifecycle and Group-owned Participant lifecycle.
- Exact Spending CRUD with multiple Payers and equal or exact Shares.
- Historical records that survive archival.
- Dual Current-Month Summary by Source Currency and Group Currency.
- All-time multi-currency Balances and advisory Settlement Transfers.
- Mobile-friendly, server-rendered Group-centered UX.

### Out Of Scope

- Multi-user collaboration, Participant accounts, invitations, registration, usernames, tenants, or Participant authentication.
- Reusing one Participant identity across Groups.
- Repayment records, paid status, settlement checkpoints, payment initiation, or settlement date ranges.
- Arbitrary timeframe summaries, statistics beyond the fixed Current-Month Summary, search, exports, receipt capture, recurring Spendings, or bank integrations.
- Ratio, percentage, weighted, or itemized Share modes.
- Globally minimal Settlement Transfer count.
- Manual exchange-rate refresh or sessions that survive process restart.
- Native mobile or desktop applications, separate desktop UX, custom JavaScript, or decorative animation.
- Multiple application instances, external database writers, or broad deployment-topology flexibility.

### Deferred

- Simple sums of Spendings over arbitrary timeframes may be considered after v1. This must not silently expand into a general analytics product.

## Glossary

- **Administrator**: The single person authenticated to operate Debtor.
- **Group**: A private ledger that owns Participants, Spendings, one Group Currency, current-month summaries, balances, and Settlement Transfers.
- **Participant**: A Group-owned accounting identity. A Participant is not an application user and cannot be reused across Groups.
- **Spending**: A dated transaction with a positive Total, one Source Currency, one category, one or more Payers, and Participant Shares.
- **Payer**: A Participant who paid part or all of a Spending Total.
- **Share**: The exact portion of a Spending Total attributed to a Participant.
- **Source Currency**: The original currency retained by a Spending.
- **Group Currency**: The Group-selected currency used for converted summaries, balances, and Settlement Transfers.
- **Current-Month Summary**: Spending totals for the selected Group whose dates fall in the current UTC calendar month.
- **Balance**: A Participant's all-time net position in the Group Currency, derived on demand from all Group Spendings.
- **Settlement Transfer**: An advisory payment from one Participant to another that would settle all-time Balances. It is not a recorded repayment.

## Features

### Secure Administrator Access

Authentication protects all ledger information and every state-changing interaction.

#### FR-1: Password-gated access

The Administrator can sign in with one configured password, remain authenticated during an active session, and sign out. Debtor provides no username, registration, Participant login, or multi-user authorization.

**Consequences:**
- Anonymous visitors cannot view Groups or ledger data.
- Restarting Debtor ends existing authenticated sessions.
- Login and all state-changing actions reject requests that lack valid request protection.

### Group And Participant Management

The main page lists Groups. All Participant management occurs within a selected Group, realizing UJ-1.

#### FR-2: Group lifecycle

The Administrator can create, edit, archive, and restore a Group. A Group with no Spendings can be deleted with its unreferenced Participants; a Group with any Spending cannot be deleted.

**Consequences:**
- Group names are trimmed, non-empty, and no longer than 100 Unicode characters.
- Archived Groups remain readable but expose no mutation controls.
- Direct attempts to mutate an archived Group are rejected without changing state.

#### FR-3: Group-owned Participants

The Administrator can add, edit, archive, and restore Participants inside a Group. Each Participant belongs to exactly one Group and is created independently if the same person appears in another Group.

**Consequences:**
- There is no separate global Participant-management surface.
- New Payers and Shares can use only active Participants owned by the selected Group.
- Archived Participants remain visible in referenced history, Balances, and Settlement Transfers.
- Participant names are trimmed, non-empty, and no longer than 100 Unicode characters.
- Participant colors use normalized `#RRGGBB` form. New Participant forms suggest a varied valid color that the Administrator can change.

### Spending Ledger

The selected Group keeps one Spending form and its Spending history close together on the Group page. The form uses compact progressive disclosure rather than permanently occupying the page or becoming a separate management surface.

#### FR-4: Record a Spending

The Administrator can create a Spending with a description, date, category, positive Total, Source Currency, one or more Payers, and either equal or exact Shares.

**Consequences:**
- Source Currency and Group Currency options are `USD`, `EUR`, `RUB`, `KGS`, `TRY`, `KZT`, `UZS`, `CNY`, `KRW`, `JPY`, `OMR`, and `TJS`.
- Category options and current display labels are `food`, `transport`, `housing`, `fun`, `shopping`, `bills`, `health`, and `other`.
- Descriptions are trimmed, non-empty, and no longer than 200 Unicode characters.
- Dates use strict `YYYY-MM-DD` form and cannot precede `2025-01-01`.
- Payer mode and Share mode are independent.
- Equal and exact are the only Share modes.
- Submitted values remain present when validation fails, with errors shown inline.

#### FR-5: Exact allocation

Every accepted Spending preserves the Total exactly in Source Currency minor units: Payer amounts sum to the Total and Shares independently sum to the Total.

**Consequences:**
- Totals and Payer/Share amounts must be positive, cannot exceed `999_999_999_999`, and must satisfy Source Currency precision; zero values, excess precision, duplicate Participants, or mismatched totals are rejected.
- Equal splitting assigns indivisible residual minor units deterministically by ascending Participant ID.
- A Spending update may retain an archived Participant only in the same existing Payer or Share role; it cannot introduce or change that archived Participant's role.

#### FR-6: Review and maintain history

The Administrator can browse, inspect, edit, and delete Spendings in an active Group.

**Consequences:**
- History is ordered newest first and presented in pages of 25.
- Spending details remain readable when their Group or Participants are archived and display each Participant's current name after a rename.
- Editing may correct a Spending's Source Currency under the same validation as creation; subsequent historical calculations use the corrected stored Source Currency.
- Editing infers the closest single/multiple-Payer and equal/exact-Share input modes from stored allocations.
- Each successful Spending change is applied completely or not at all.

### Current-Month Spending Summary

The Group page keeps the current UTC calendar month separate from the all-time debt calculation, realizing UJ-1.

#### FR-7: Source Currency summary

The Administrator can see the selected Group's current-month Spending Total and each Payer's paid total, grouped by original Source Currency.

**Consequences:**
- Source Currency totals remain available without exchange-rate conversion.
- Spendings outside the current UTC calendar month are excluded.

#### FR-8: Group Currency summary

The Administrator can see the same current-month Group and per-Payer totals converted to the Group Currency using the historical rate for each Spending date.

**Consequences:**
- Future-dated Spendings use the latest current rate and are marked provisional.
- A context-matching fixed past-date historical quote may be used without an age limit; a stale current or future-date quote may be used only through seven UTC calendar days after its effective fetch date. Every stale result carries a warning.
- If a required quote is unavailable, only the converted summary reports a retryable failure; Source Currency totals, history, and ledger mutations remain usable.

### All-Time Balances And Advisory Settlements

Debtor derives all-time Group Balances and Settlement Transfers on demand. These results advise the Administrator but do not record payment, settlement, or completion state.

#### FR-9: Select conversion mode

The Administrator can calculate all-time Balances in historical mode or current mode. Historical mode is the default and converts each Spending at its Spending date; current mode converts every Spending at the UTC calculation date and is not persisted.

#### FR-10: Exact Balances

Debtor calculates one exact Group Currency Balance per Participant and preserves an exact zero sum after currency quantization.

**Consequences:**
- Completion order of exchange-rate requests cannot alter results or warnings.
- Arithmetic or conversion failure returns no partial Balances or Settlement Transfers.

#### FR-11: Deterministic Settlement Transfers

Debtor presents positive, deterministic Settlement Transfers that settle every Balance.

**Consequences:**
- A Participant pair appears at most once.
- No more than `n - 1` Settlement Transfers are produced for `n` Participants.
- Debtor does not claim globally minimal transfer count.

#### FR-12: Calculation disclosure and failure isolation

The debts view identifies the conversion mode, calculation time, Group Currency, unique rates used, and stale or provisional warnings.

**Consequences:**
- If a required quote is unavailable without a valid stale fallback, the debts view reports a retryable failure.
- Exchange-rate failure never prevents Group, Participant, or Spending management.

## Cross-Cutting Non-Functional Requirements

### Correctness And Historical Integrity

- Monetary input, storage, aggregation, conversion, and display must preserve exact decimal values and currency minor-unit rules without floating-point loss.
- Historical references must remain readable after Group or Participant archival.
- Every write of a complete Spending aggregate must validate Group ownership and Participant eligibility in the same atomic operation.
- Complete Spending and debt views must be calculated from internally consistent snapshots.

### Security And Privacy

- Every state-changing request, including login, must be authenticated where applicable and protected against cross-site request forgery.
- Authentication must resist repeated login attempts and use secure session cookies in production.
- Credentials, password hashes, session identifiers, request-protection tokens, and sensitive ledger or provider data must never appear in logs or user-facing errors.
- Authenticated pages must not be cached by browsers or intermediaries.

### Availability And Bounded Operation

- Exchange-rate-provider availability must not gate startup, readiness, or ledger CRUD.
- User traffic, login, probes, database waits, exchange-rate calls, caches, and sessions must have bounded resource usage and wait times.
- Once an admitted mutation begins changing state, it must return a definitive success or rollback result rather than being cancelled by a generic timeout.
- Shutdown must stop new admission, allow in-flight work to finish within a defined bound, and leave the ledger recoverable.

## Constraints And Guardrails

- Debtor remains permanently single-Administrator; Participants remain accounting identities.
- The supported production shape is one Debtor process with one private local ledger volume behind a sanitizing HTTPS reverse proxy.
- Each Spending retains its stored Source Currency for historical interpretation; Group Currency is a freely changeable display and settlement target.

## Open Questions

None.

## Assumptions Index

No unconfirmed assumptions remain in this draft.
