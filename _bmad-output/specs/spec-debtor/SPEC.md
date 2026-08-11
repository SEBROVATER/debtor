---
id: SPEC-debtor
companions:
  - glossary.md
  - ../../planning-artifacts/prds/prd-debtor-2026-08-10/addendum.md
  - ../../planning-artifacts/ux-designs/ux-debtor-2026-08-10/DESIGN.md
  - ../../planning-artifacts/ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md
  - ../../../specs/design.md
  - ../../project-context.md
sources:
  - ../../planning-artifacts/prds/prd-debtor-2026-08-10/prd.md
---

> **Canonical contract.** This SPEC and the files in `companions:` are the complete, preservation-validated contract for what to build, test, and validate. Source documents listed in frontmatter are for traceability only; consult them only for narrative rationale or prose color this contract intentionally omits.

# Debtor

## Why

Debtor realizes a dependable, deliberately small, self-operated multi-currency shared-expense ledger for one administrator who needs exact current spending and debt answers without collaboration features or complex deployment.

## Capabilities

- **CAP-1**
  - **intent:** The administrator can sign in with one configured password, maintain an active session, and sign out while all ledger access and mutations remain protected.
  - **success:** Anonymous access is denied, valid login enables ledger use, logout and restart invalidate sessions, and unsafe requests without valid CSRF and single-use submission protection cause no mutation.
- **CAP-2**
  - **intent:** The administrator can create, edit, archive, restore, and, when history permits, delete Groups.
  - **success:** New Groups default to USD and open in Manage, archived Groups remain readable but immutable, and only Groups without Spendings can be deleted with unreferenced Participants.
- **CAP-3**
  - **intent:** The administrator can manage Group-owned Participant accounting identities without creating application users.
  - **success:** Participants belong to exactly one Group, referenced identities survive archival, restoration is unconditional, and archival commits only from an unchanged all-time Historical context with an exact zero Group Currency Balance.
- **CAP-4**
  - **intent:** The administrator can record and correct a dated Spending in its Source Currency with one Payer and Proportional or Exact Shares.
  - **success:** Accepted Spendings satisfy all field, ownership, lifecycle, precision, positivity, and upper-bound rules; Payer and Shares each conserve the Total exactly; failed submissions retain input with inline errors.
- **CAP-5**
  - **intent:** The administrator can browse, inspect, edit, and delete Spendings while preserving referenced history.
  - **success:** History is newest-first in fixed 25-item pages, details remain readable for archived identities using current names, direct item actions load complete aggregates, and every change commits atomically or not at all.
- **CAP-6**
  - **intent:** The administrator can review the selected Group's current UTC month Total and each Payer's paid Total by original Source Currency.
  - **success:** Only current-month Spendings contribute, totals remain exact, and the summary remains available without exchange-rate conversion.
- **CAP-7**
  - **intent:** The administrator can review the same current-month Group and per-Payer totals converted to Group Currency using date-appropriate rates.
  - **success:** Final Payer totals quantize together exactly and sum to the Group total; stale or provisional evidence is disclosed; any missing quote or checked calculation failure withholds the entire converted section while source totals and ledger CRUD remain usable.
- **CAP-8**
  - **intent:** The administrator can calculate all-time Participant Balances in historical or current conversion mode.
  - **success:** Historical mode defaults to each Spending date, current mode uses the calculation date, results are deterministic and exact-zero-sum, disclosures identify context and rates, and any failure returns no partial Balances or Transfers.
- **CAP-9**
  - **intent:** The administrator can receive advisory Settlement Transfers that settle all-time Balances without recording repayment state.
  - **success:** Transfers are positive, deterministic, pair-unique, complete, and at most `n - 1` for `n` included Participants, without claiming global count minimality.

## Constraints

- Debtor is permanently single-Administrator; Participants are Group-owned accounting identities, never users, memberships, tenants, or reusable global identities.
- All money and rates use exact Decimal, canonical SQLite TEXT persistence, currency minor-unit validation, checked Rust arithmetic, and Rust aggregation; floating point, SQL monetary parsing/conversion/aggregation, silent rounding, zero substitution, and partial financial results are forbidden.
- Referenced history survives archival; destructive Participant deletion is unavailable, referenced Group deletion is restricted, complete Spending writes are atomic, and calculations read internally consistent snapshots.
- Preserve inward dependencies `root -> web/infra -> application -> domain`; outer-library types cannot cross application-owned ports. Detailed ownership and testing boundaries are defined by the adopted companions.
- The Group-centered web experience uses semantic server-rendered Askama HTML, vanilla CSS, and native links/forms; only pinned self-hosted HTMX progressive enhancement is allowed, with no custom application JavaScript, and usability/accessibility holds across current major browsers down to 320 CSS pixels.
- Every unsafe route requires applicable authentication, session-backed CSRF, and a bounded, expiring single-use submission token reserved immediately before dispatch; secrets and sensitive ledger/provider diagnostics never enter logs or user-facing errors.
- Production supports one Debtor process and one private local WAL SQLite volume behind a sanitizing HTTPS reverse proxy; provider availability cannot gate startup, readiness, or ledger CRUD, and resource use, waits, and shutdown remain bounded as defined by the adopted companions.
- `specs/design.md` is normative and is updated before behavior changes; conflicts among contract artifacts must be reconciled rather than silently interpreted.

## Non-goals

- Multi-user collaboration, Participant accounts or authentication, invitations, registration, usernames, tenants, and cross-Group Participant reuse.
- Repayment records, paid or settled state, settlement checkpoints or date ranges, payment initiation, and globally minimal transfer count.
- Arbitrary-timeframe analytics in v1, search, exports, receipt capture, recurring Spendings, bank integrations, manual rate refresh, and sessions that survive restart.
- Multiple Payers, direct percentage or itemized Share modes, native applications, separate desktop UX, custom application JavaScript, multiple application instances, external database writers, and broad deployment-topology flexibility.

## Success signal

- After four consecutive weeks of real use, the administrator has recorded every intended Spending for at least one active Group and can answer its exact current-month source and converted totals, each Payer's paid total, all-time zero-sum Balances, and complete advisory Transfers without an external ledger. Exchange-rate failure still leaves source summaries and ledger management usable.
