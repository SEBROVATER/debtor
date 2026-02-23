# Feature Specification: Shared Expenses Manager

**Feature Branch**: `001-shared-expenses`
**Created**: 2026-02-23
**Status**: Draft
**Input**: User description: "Create web-app that will help me to manage shared expenses with my roommates or friends..."

---

## User Scenarios & Testing *(mandatory)*

### User Story 1 — Secure Access (Priority: P1)

As the sole owner, I need to log in securely before accessing any part of the
application, so that my financial data is not exposed to others on the network.

**Why this priority**: Without authentication, no other feature is safe to use.
This is the gating requirement for the entire application.

**Independent Test**: Navigate to the app without a session — all pages MUST
redirect to the login screen. After successful login, the dashboard is
accessible.

**Acceptance Scenarios**:

1. **Given** I am not logged in, **When** I visit any page, **Then** I am
   redirected to the login page.
2. **Given** I am on the login page, **When** I submit correct credentials,
   **Then** I am taken to the main dashboard.
3. **Given** I am on the login page, **When** I submit incorrect credentials,
   **Then** I see an error message and remain on the login page.
4. **Given** I am logged in, **When** I click "Log out", **Then** my session
   ends and I am redirected to the login page.

---

### User Story 2 — Group Management (Priority: P1)

As the owner, I need to create independent expense-sharing groups (e.g.,
"Apartment", "Road Trip") and manage their members, so I can track separate
pools of debts without them interfering with each other.

**Why this priority**: Groups are the central organising unit; all other
features depend on them.

**Independent Test**: Create a group, add members, rename a member, remove a
member, and delete the group — all without any expenses needed.

**Acceptance Scenarios**:

1. **Given** I am on the dashboard, **When** I create a new group with a name
   and a target currency, **Then** the group appears in my group list.
2. **Given** I am in a group, **When** I add a friend by name, **Then** the
   friend appears in the group's member list.
3. **Given** a friend exists in a group, **When** I rename them, **Then** the
   new name is reflected throughout the group's history.
4. **Given** a friend exists in a group, **When** I remove them, **Then** they
   are no longer listed as an active member.
5. **Given** a group exists, **When** I change its target currency, **Then**
   all debt summaries are recalculated using current exchange rates in the new
   currency.

---

### User Story 3 — Expense Recording (Priority: P1)

As the owner, I need to record an expense by specifying the payer, the
participants sharing the cost, the amount, and the currency (which may differ
from the group's target currency), so that the system can accurately track who
owes what.

**Why this priority**: Expense recording is the core value of the application.

**Independent Test**: Add a single expense to a group with multiple members,
verify it appears in the expense list with correct details.

**Acceptance Scenarios**:

1. **Given** I am in a group, **When** I record an expense with a payer,
   participants, amount, and currency, **Then** the expense appears in the
   group's expense list.
2. **Given** I select a currency different from the group's target currency,
   **Then** the system stores the original currency and amount without
   converting.
3. **Given** an expense exists, **When** I edit its amount, payer, participants,
   or currency, **Then** the updated values are saved and debts recalculated.
4. **Given** an expense exists, **When** I delete it, **Then** it is removed
   and debts are recalculated.

---

### User Story 4 — Debt Summary & Smart Recalculation (Priority: P2)

As the owner, I need to see a simplified, minimised list of who owes whom in
each group (expressed in the group's target currency), so I can quickly
understand and settle debts without navigating a complex web of transactions.

**Why this priority**: This is the main insight the app provides; without it
the expense list is just a ledger.

**Independent Test**: Add several cross-connecting expenses in a group and
verify the debt summary is the minimum number of transactions needed to settle
all balances.

**Acceptance Scenarios**:

1. **Given** a group has multiple expenses, **When** I view the debt summary,
   **Then** I see a minimal set of directed debts that fully settles all balances.
2. **Given** expenses are recorded in multiple currencies, **When** I view the
   summary, **Then** amounts are converted to the group's target currency using
   the most recently cached exchange rate.
3. **Given** all debts in a group are settled (balances zero), **When** I view
   the summary, **Then** I see a "no outstanding debts" indication.

---

### User Story 5 — Multi-Currency & Exchange Rates (Priority: P2)

As the owner, I need the system to automatically fetch and cache exchange rates
(at most once per day, on demand), so that multi-currency expenses are converted
correctly without me having to enter rates manually.

**Why this priority**: Multi-currency support is only useful if rates are
accurate and automatic.

**Independent Test**: Record an expense in a non-target currency, view the debt
summary, and verify a converted amount appears. Verify that a second request
within the same day does not trigger a new rate fetch.

**Acceptance Scenarios**:

1. **Given** a group debt summary is requested, **When** cached rates are
   stale (older than 24 hours) or absent, **Then** the system fetches fresh
   rates before rendering the summary.
2. **Given** rates were fetched within the last 24 hours, **When** a summary
   is requested, **Then** the cached rates are used without a new network
   request.
3. **Given** the rate service is unavailable, **When** a summary is requested,
   **Then** the system falls back to the last known cached rates and displays a
   warning that rates may be outdated.

---

### Edge Cases

- What happens when only one person is in a group? Expenses can be added but
  debts are trivially zero (person cannot owe themselves).
- What happens when a removed member had expenses? Historical expenses remain;
  the member appears as "inactive" in expense history.
- What happens when the exchange rate service is completely unreachable and no
  cache exists? Debt conversion is blocked; user sees an informative error
  with the raw currency amounts still displayed.
- What happens with rounding during currency conversion? Amounts are rounded to
  2 decimal places using standard half-up rounding; minor rounding errors are
  accepted as a known limitation.
- What happens after 5 consecutive failed login attempts? The account is locked
  for 15 minutes; the user sees the remaining lockout duration.

---

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST require authentication before granting access to
  any page or data.
- **FR-002**: The system MUST support exactly one administrator account; there
  MUST be no registration flow.
- **FR-003**: Users MUST be able to create, rename, and delete expense groups.
- **FR-004**: Each group MUST have a configurable target currency.
- **FR-005**: Users MUST be able to add, rename, and remove members within a group.
- **FR-006**: Users MUST be able to record an expense specifying: payer (one
  member), participants (one or more members), amount, and currency. An
  optional free-text description (note) MAY be provided but is not required.
- **FR-006a**: Expense amounts MUST be split equally among participants by
  default. The owner MUST be able to override the split per participant using
  either a percentage share or an absolute amount. The sum of participant
  shares MUST equal the total expense amount.
- **FR-007**: The expense currency MAY differ from the group's target currency;
  the raw original currency and amount MUST be persisted.
- **FR-008**: Users MUST be able to edit and delete existing expenses.
- **FR-009**: The system MUST calculate a minimal set of directed debt
  transactions that settles all balances within a group (debt simplification).
- **FR-010**: Debt summaries MUST be expressed in the group's target currency,
  converting multi-currency expenses using cached exchange rates.
- **FR-011**: Exchange rates MUST be fetched from a free external service at
  most once per calendar day, triggered lazily on demand (not on a schedule).
- **FR-012**: Fetched exchange rates MUST be cached and reused for subsequent
  requests within the same calendar day.
- **FR-013**: If rate fetching fails and a cache exists, the system MUST fall
  back to cached rates and display a staleness warning.
- **FR-014**: The target currency of a group MUST be changeable at any time;
  debt summaries MUST recalculate immediately using the new currency.
- **FR-015**: The UI MUST be minimalistic: no unnecessary chrome, no
  decorative elements beyond what aids readability.
- **FR-016**: The system MUST NOT provide a mechanism to mark debts as
  settled or paid. Debts exist as long as the underlying expenses exist;
  removing or editing expenses is the only way to change balances.
- **FR-017**: A login session MUST remain valid for 30 days, with the
  expiry reset on each authenticated page visit (sliding expiry). Sessions
  MUST be invalidated immediately on explicit logout.
- **FR-018**: After 5 consecutive failed login attempts, the system MUST
  lock the account for 15 minutes. The owner MUST be shown a clear message
  indicating the lockout duration. The attempt counter MUST reset after a
  successful login.

### Key Entities

- **Group**: Named collection of members and expenses, with a target currency.
- **Member**: Named participant within a group; may be active or inactive
  (after removal).
- **Expense**: An amount in a specific currency, attributed to a payer, shared
  among a set of members within a group; has an optional free-text description
  and a date. Each participant's share is recorded as either an equal portion
  (default), a percentage, or an absolute amount.
- **Debt**: A derived, directional obligation (A owes B amount in target
  currency) calculated from the group's expenses; not stored — always computed.
- **ExchangeRate**: A cached snapshot of currency conversion rates fetched from
  an external service, with a timestamp used to enforce the 24-hour TTL.

---

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: The owner can log in and reach the dashboard within 5 seconds on
  a local network connection.
- **SC-002**: Adding a new expense, including selecting payer, participants, and
  currency, takes under 60 seconds end-to-end.
- **SC-003**: The debt summary for a group with up to 20 members and 200
  expenses is displayed in under 3 seconds.
- **SC-004**: Exchange rates are never fetched more than once per calendar day
  per currency pair, regardless of how many summary requests are made.
- **SC-005**: The simplified debt list for any group contains the minimum
  possible number of transactions to settle all balances (provably optimal
  simplification).
- **SC-006**: 100% of application pages redirect to the login screen when
  accessed without a valid session.
- **SC-007**: Changing a group's target currency causes all debt amounts to
  update within the same page response.

---

## Assumptions

- The application is deployed and accessed on a private/local network; HTTPS is
  recommended but not enforced by the spec.
- "Friends" and "roommates" are represented as named members within a group;
  they do not have individual accounts or logins.
- The free exchange rate service is assumed to support common world currencies;
  the specific service is a technical decision deferred to planning.
- The admin's own name may appear as a member within groups (to participate in
  shared expenses).
- Group deletion is a destructive action and removes all associated members and
  expenses; the user is warned before confirming.
- The minimum debt simplification algorithm is based on net-balance settlement
  (greedy or equivalent optimal approach); the exact algorithm is deferred to
  planning.
- **Out of scope**: Debt settlement / payment tracking. Debts are always
  derived from expenses; balances can only be adjusted by editing or deleting
  expenses.

---

## Clarifications

### Session 2026-02-23

- Q: When an expense is shared between participants, how are amounts split? → A: Equal split is the default; the owner may override per participant using either a percentage share or an absolute amount. Both override modes must be supported simultaneously.
- Q: Should the system allow marking a debt as settled/paid? → A: No. No settlement tracking. Debts exist until expenses are deleted or edited.
- Q: How long should a login session remain valid? → A: 30 days, sliding expiry reset on each page visit; invalidated immediately on logout.
- Q: Is a description (note) for an expense required or optional? → A: Optional free-text note; can be left blank.
- Q: Should repeated failed login attempts be rate-limited? → A: Yes — temporary lockout: 5 consecutive failures trigger a 15-minute lockout; counter resets on successful login.
