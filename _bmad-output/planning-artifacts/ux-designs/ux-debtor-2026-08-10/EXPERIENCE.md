---
title: Debtor Experience Contract
name: Debtor
status: final
created: 2026-08-10
updated: 2026-08-10
sources:
  - /home/sebr/projects/pet/debtor/_bmad-output/planning-artifacts/prds/prd-debtor-2026-08-10/prd.md
  - /home/sebr/projects/pet/debtor/_bmad-output/planning-artifacts/prds/prd-debtor-2026-08-10/addendum.md
---

# Debtor Experience Contract

## Foundation

`DESIGN.md` is the visual identity reference. This spine owns structure, behavior, states, and interaction. `DESIGN.md` and `EXPERIENCE.md` take precedence over mockups, wireframes, imports, and `.working` artifacts when conflicts arise.

Debtor is one responsive, authenticated web experience for one Administrator. Participants are Group-owned accounting identities, never application users. Semantic server-rendered HTML and native links/forms are the complete interaction baseline. Pinned self-hosted HTMX and its pinned official `response-targets` extension may progressively enhance section navigation and allocation previews; neither is required to complete a task. No other extension, custom application JavaScript, inline script, or inline script attribute is permitted. Operation is online only.

Private rendering, authentication, request protection, HTTP outcomes, limits, and runtime bounds are inherited from the [PRD security requirements](../../prds/prd-debtor-2026-08-10/prd.md#security-and-privacy) and [technical addendum](../../prds/prd-debtor-2026-08-10/addendum.md#http-forms-statuses-and-dispatch). Their UX consequences are fixed: anonymous visitors see no ledger data; expired or restarted sessions return to Sign in; unsafe requests are protected and produce either definitive success or definitive non-mutation; submitted non-password values survive valid form errors; private pages are not cached; errors disclose no secrets; native navigation presents server and browser transport/runtime failures when HTMX is absent or fails.

Every unsafe form explicitly requires one submission token distinct from CSRF. [HTTP and Session Outcomes](#http-and-session-outcomes) owns its bounded, expiring, session-bound, single-use lifecycle, reservation, conflict, and recovery contract.

The experience is mobile-first and supports one-handed use from 320 CSS pixels through desktop. It is one layout adapting to available width, not separate mobile and desktop products. It serves both ongoing personal tracking and shared activity without assuming every Group represents travel or friends.

Debtor stays deliberately small: every destination or control serves a confirmed Administrator job. Trust comes from committed history, preserved identity, complete-or-no-result calculations, and visible calculation context.

## Information Architecture

| Surface | Reached from | Purpose |
|---|---|---|
| Sign in | Anonymous app open | Authenticate the single Administrator with password and CSRF-protected submission. |
| Groups | Successful sign in; **Group navigation** | List active Groups through the **Group list** and create a Group by name. Each row shows only Group name, Group Currency, and active Participant count. |
| Archived Groups | Groups contextual link | Find and restore archived Groups; archived Groups open read-only. |
| Summary | Established Group open; **Group navigation** | Through **Financial results**, immediately show current-month Group and per-Payer totals grouped by Source Currency, plus equivalent Group Currency totals. [Group Summary mock](mockups/group-summary.html) illustrates Source/Group Currency hierarchy, provisional notice, and phone/wide shell adaptation. |
| Transactions | **Group navigation**; successful Spending mutation | Browse 25 newest-first Spendings and expand one **Transaction row** in place. [Transactions mock](mockups/transactions.html) illustrates expanded facts/actions, pagination context, and post-create focused-row placement. |
| Debts | **Group navigation** | Select Historical or Current and calculate complete Balances/Transfers with disclosure. [Debts mock](mockups/debts.html) illustrates ready-with-stale and unavailable-with-no-partial-result states. |
| Manage | New Group open; **Group navigation** | Edit the Group and Participants and perform Group lifecycle work. [Manage mock](mockups/manage.html) illustrates ordered settings, Participant eligibility/color controls, lifecycle, and rate-blocked archive state. |
| Archived Participants | Manage contextual link | Restore archived Participants while preserving their historical visibility. |
| Spending form | Persistent **Add Spending action**; edit action | Create or edit one Spending on a focused full page. [Add Spending mock](mockups/add-spending.html) illustrates reviewed and validation-error states, horizontal allocation, and keyboard-aware action geometry. |
| Confirmation page | Spending delete; Participant archive; Group archive; history-free Group delete | Confirm the named object, reversibility, and scope on a dedicated server-rendered **Confirmation page**. |

The compact Group shell has five persistent destinations in this order: Groups, Summary, Transactions, Debts, Manage. Every destination is a native link; HTMX may swap Group sections in place while preserving the same URL and response. **Add Spending action** is a native link to the focused full-page form from every active Group section. Sign out appears in every **Page header**, but does not remain fixed while scrolling.

New Groups open in Manage. Established Groups open in Summary. Archived Groups retain persistent **Group navigation** to Groups, Summary, Transactions, Debts, and read-only Manage; every mutation control is suppressed. Active lists exclude archived records; restoration lives in a separate contextual **Archived view**, not mixed into active lists.

## Voice and Tone

Microcopy is concise, factual, calm, and explicit. Brand posture lives in `DESIGN.md`.

| Do | Don't |
|---|---|
| “Updating converted values.” | “Just a moment...” |
| “Converted values are unavailable. Reopen this section to retry.” | “Something went wrong.” |
| “Provisional: a current rate was used for a future Spending.” | “Estimated” without stating why. |
| “This Participant cannot be archived until their Historical Balance is exactly zero.” | “Participant ineligible.” |
| “Rates are unavailable, so archive was not attempted. Reopen Manage to retry.” | “Try again” on a manual Retry button. |
| “Remaining: [amount]” or “Excess: [amount]”. | Color-only allocation feedback. |

Preserve product terms and capitalization: Administrator, Group, Participant, Spending, Total, Payer, Share, Source Currency, Group Currency, Current-Month Summary, Balance, and Settlement Transfer. Do not call Participants members, users, accounts, or debtors. Do not imply that a Settlement Transfer is paid, completed, recorded, or globally minimal.

English is the only v1 language. Display Spending dates as ISO `YYYY-MM-DD` outside forms. Display money with exact currency precision, a currency symbol, and ISO code; never show a symbol alone.

## Component Patterns

Behavioral rules below pair one-for-one with `DESIGN.md` Components.

### Global Shell and Navigation

| Component | Use | Behavioral rules |
|---|---|---|
| **Interactive target** | Every control | Every button, link, disclosure summary, labeled radio/checkbox, field, select, row action, Group row, and navigation destination renders at least 48 by 48 CSS pixels at 320px and 400% zoom. There are no link exceptions. |
| **Page header** | Every page | Announces page or Group context in reading order. Sign out is a protected native form and **Interactive target** in normal flow; it requires the [submission token](#http-and-session-outcomes). First activation marks it pending and suppresses/coalesces repeats until the definitive response. |
| **Group navigation** | Active and archived Group shells | Five native-link destinations remain visible and use `aria-current="page"`. HTMX writes the same URL/history entry and swaps only the section. Forward focus is the stable section heading per the [matrix](#interaction-focus-matrix). Archived Groups retain navigation but suppress mutation controls. |
| **Add Spending action** | Every active Group section | Native link opens the focused full-page **Spending form**; forward focus is its stable `h1` per the [matrix](#interaction-focus-matrix). Disabled when no active Participant exists, with `aria-describedby` guidance and a 48-by-48 recovery link. Hidden for archived Groups. |
| **Ledger section** | Summary, Transactions, Debts, Manage | One primary task per compact section. Section replacement announces the heading and does not create a long anchored page. |

### Financial Presentation

| Component | Use | Behavioral rules |
|---|---|---|
| **Conversion notice** | Summary and Debts | One stable node per derived region uses `role="status"`, `aria-live="polite"`, and `aria-atomic="true"`; the result container toggles `aria-busy`. It announces one transition for Updating, Ready with stale data, Ready with provisional data, or Unavailable. The result heading/container references it with `aria-describedby`; individual amounts are not live regions. |
| **Money row** | Summary, Debts, allocation preview | Keeps label, exact amount, symbol, and ISO code together. Screen-reader text preserves sign/direction and currency; visual alignment never changes reading order. |
| **Participant marker** | Participant and financial rows | Supplements the visible Participant name. Stored color is never the only identity or state signal. |
| **Financial results** | Summary and Debts | Summary orders each Source Currency Group total before its per-Payer rows, then Group Currency major total/per-Payer rows/calculation note; [Group Summary mock](mockups/group-summary.html) illustrates the hierarchy. Debts orders one Balance per Participant, Transfers, then disclosure; [Debts mock](mockups/debts.html) illustrates complete stale and no-partial unavailable states. Exact zero-sum, deterministic transfer, direction, and disclosure invariants remain unchanged. |

### Spending and History

| Component | Use | Behavioral rules |
|---|---|---|
| **Spending form** | Create and edit Spending | Focused full page with the Group/form title on the left and the Edit/Preview state on the right. Cancel is an allow-listed return link. Native Preview rerenders a review state; Approve is available only for reviewed input, while Edit allocation returns to editable state. Approve requires the [submission token](#http-and-session-outcomes). [Add Spending mock](mockups/add-spending.html) illustrates reviewed and retained-error compositions. |
| **Field** | Forms | Visible `label` association and stable `aria-describedby` guidance survive HTMX swaps. Guidance states relevant length, date, earliest-date, and currency-precision constraints before submission. Invalid controls use `aria-invalid="true"` and combine guidance plus stable error IDs in `aria-describedby`. |
| **Share mode control** | Spending form | Exactly Proportional and Exact native controls. Change may request an HTMX preview but never mutates. Edit opens Exact because mode and proportional weights are not persisted. |
| **Allocation table** | Spending form | Semantic horizontal table inside a `tabindex="0"` region labeled “Participant allocation table; scroll horizontally for Payer, Included, and Weight/Share” through stable `aria-labelledby`/`aria-describedby`. Participant color marker and name cells remain sticky at inline start; explicit header IDs and control labels preserve row/column associations. Names up to 100 Unicode characters wrap/break in the sticky cell. Payer, Included, and Weight/Share are 48-by-48 **Interactive targets**. Native Preview submits the full page. HTMX swaps only stable derived amount cells, approval state, and one status node; focused controls remain outside the swap. Requests use latest-input-wins behavior, superseded responses are ignored, and focus, caret, selection, software keyboard, table/page scroll, and active row remain unchanged. |
| **Form action bar** | Spending form | At 320px, first row shows Total plus concise preview/pending/error status; both wrap without truncation. Second row has three equal 48px-minimum actions in this order: Cancel, Preview or Edit allocation, Approve. Gaps are `{spacing.4}`. Approve never repeats the amount and is disabled while preview is pending, stale, invalid, or superseded. Approve requires the [submission token](#http-and-session-outcomes); first activation disables its initiator. Focused controls use `{components.form-action-bar.control-scroll-margin}`. |
| **Transaction row** | Transactions | Native `<details>` summary exposes Description/date left and Total right. Expanded definition rows show Description, Total, Source Currency, date, category, Payer, and Shares before equal Edit/Delete actions. Post-create/edit focuses the row summary without a completion badge. [Transactions mock](mockups/transactions.html) illustrates collapsed, expanded, paginated, and focused-row layouts. |
| **Pagination** | Transactions | Native Previous/Next links encode fixed 25-item page context ordered by `(spent_date DESC, id DESC)`; HTMX may swap the same response. Endpoints expose disabled state. Forward focus is the stable Transactions heading per the [matrix](#interaction-focus-matrix); announce page context. |

### Lifecycle and Access

| Component | Use | Behavioral rules |
|---|---|---|
| **Confirmation page** | Spending deletion; Participant archive; Group archive; history-free Group deletion | Names the object, effect, and reversibility. Server state carries an allow-listed return URL plus stable focus ID. Per the [matrix](#interaction-focus-matrix), Cancel targets the invoker; success targets the next/previous Transaction summary or relevant list heading and announces once. Submit requires the [submission token](#http-and-session-outcomes) and becomes unavailable on first activation. |
| **Archived view** | Archived Groups; archived Participants | Separate native-link destination. Visible “Archived” text is inside the identity heading/row label or referenced by it; no invented ARIA state. Read-only values are definition text or native `readonly` when copying matters. Restore requires the [submission token](#http-and-session-outcomes); per the [matrix](#interaction-focus-matrix), it targets the restored active row or relevant heading and announces once. |
| **Access form** | Sign in | Exactly one password Field and one protected native submit; no username. Submit requires the [submission token](#http-and-session-outcomes) and suppresses/coalesces repeats while pending. Password is never retained. Runtime/transport failures replace navigation with a safe full-page state; HTMX is not required. |
| **Group list** | Groups | Create-by-name precedes active rows and requires the [submission token](#http-and-session-outcomes); Archived Groups is a contextual text link. Established rows enter Summary. Empty active state preserves create and archived navigation. |
| **Management form** | Manage | Ordered sections: Group settings, Participants, Group lifecycle. Unsafe submits require the [submission token](#http-and-session-outcomes). A rate-blocked **Request status** precedes the affected Participant list. Each Participant orders identity, fields, Historical Balance, eligibility, and actions; Add Participant follows the roster. Archive and delete remain distinct. [Manage mock](mockups/manage.html) illustrates eligible/ineligible and unavailable-rate arrangements. |
| **Participant color control** | Participant add/edit | Authoritative labeled text input accepts normalized `#RRGGBB`. New forms receive a varied valid server suggestion; edit starts with stored color. A named swatch previews valid submitted value. Invalid raw text is retained; identity and state never rely on the swatch. The owning submit requires the [submission token](#http-and-session-outcomes). |

### Requests and Calculation Modes

| Component | Use | Behavioral rules |
|---|---|---|
| **Debt mode control** | Debts | Historical is the native checked default. Selecting a mode submits a safe native form; HTMX may swap the result. The mode stays in the URL and Current is not persisted. Enhanced replacement retains focus on the selected radio and announces result status rather than moving focus. |
| **Request status** | Enhanced and native request outcomes | One stable scoped node uses `role="status"`, `aria-live="polite"`, and `aria-atomic="true"`; its owning region toggles `aria-busy`. The pinned official `response-targets` extension declaratively routes expected `4xx`/`5xx` fragments into it. Routine pending and expected failures retain invoker focus and announce once; urgent session loss uses a server-rendered alert/heading instead. Native links/forms present the same outcomes when enhancement is absent. No custom response extension or handler is allowed. |

## Form and Option Contracts

- Field sets, closed currency/category options, defaults, length/date limits, normalized Participant color, amount bounds, and currency precision inherit exactly from [PRD Features](../../prds/prd-debtor-2026-08-10/prd.md#features). Forms expose relevant limits and examples before submission rather than duplicating policy in help prose.
- Every visible **Field** has stable label, guidance, error, and retained-value behavior. Participant color keeps submitted raw text on validation failure; passwords never persist.
- Native validation responses retain safely decoded values. Structurally untrusted requests present a safe fresh-form recovery state without pretending individual fields were accepted.
- Multiple submitted errors render one `role="alert"` summary whose heading has `tabindex="-1"` and receives focus, with links to exact controls. Each control preserves guidance and stable error IDs; allocation-wide Remaining/Excess describes both the allocation region and Approve, while row errors attach only to their row input.

## Financial Allocation

- A Spending has exactly one Payer. Selecting a Payer assigns that active Group-owned Participant the full Total; selecting another row replaces the Payer.
- Proportional and Exact are the only Share modes. There is no multiple-Payer, Equal-share, percentage, or itemized mode.
- On create, Proportional initially includes every active Participant with weight `1`. Participants may be deselected; included weights are positive decimals. Preview shows the exact Source Currency amounts produced by [FR-5](../../prds/prd-debtor-2026-08-10/prd.md#fr-5-exact-allocation).
- On create, Exact initially includes every active Participant with equal minor-unit Shares. The Administrator may deselect Participants and edit amounts. The **Allocation table** shows Remaining or Excess until selected Shares equal the Total exactly.
- No Payer is initially selected. Description and Total start empty; Source Currency defaults to Group Currency; date defaults to current UTC date; Category has no default.
- Edit always opens Exact with the stored single Payer and Share amounts. An archived Participant may remain only in the same existing Payer or Share role and cannot be introduced or moved to another role.
- Share selection is nonempty and Participant-unique. Zero Shares, duplicate Participants, no Payer, more than one Payer, mismatched Payer Total, mismatched Share Total, nonpositive weight, excess precision, and out-of-range amounts are rejected inline by the **Allocation table** or owning **Field**.
- Preview requests are calculations, not mutations. Native Preview rerenders the full page in reviewed, non-editable state with Approve and Edit allocation; therefore native approval cannot outlive the reviewed input. With HTMX, each request carries the current form revision, latest input wins, superseded responses do not swap, and only derived cells/status/approval state change. Approve stays disabled while preview is pending, stale, invalid, or superseded. Approve is the sole ledger mutation.
- Every accepted allocation conserves the Total independently across the single Payer and Shares at Source Currency precision; never round user input to make it pass.
- Source Currency remains editable under the same option, precision, and allocation validation as create. A successful correction treats the corrected stored Source Currency as historical truth and invalidates/recomputes affected Summary and Debts results.

## HTTP and Session Outcomes

Exact statuses, admission bounds, and timeout rules inherit from [Addendum § HTTP Forms, Statuses, And Dispatch](../../prds/prd-debtor-2026-08-10/addendum.md#http-forms-statuses-and-dispatch) and [§ Admission, Timeouts, Probes, And Shutdown](../../prds/prd-debtor-2026-08-10/addendum.md#admission-timeouts-probes-and-shutdown). The UX contract is:

This section owns the submission-token lifecycle and request-outcome mechanics referenced by every unsafe-form component.

- Every unsafe form carries one bounded, expiring, session-bound, single-use submission token distinct from CSRF.
- Valid form errors rerender the canonical native page with retained non-password values and the error pattern above. Successful mutation redirects to the canonical destination so refresh does not replay it.
- Validation before dispatch preserves the unsafe form's single-use submission token. Reservation occurs atomically immediately before dispatch; after reservation the request remains pending until definitive success or rollback.
- A missing, unknown, expired, reserved, or consumed submission token produces an announced `409 Conflict`, invokes no use case, and provides a native canonical-form reload that issues a fresh token. The submission token is distinct from CSRF and never replays an earlier result.
- Rejected, unavailable, oversized, rate-limited, capacity, and timeout outcomes use a safe full-page or scoped **Request status**, state whether no change occurred, and reveal no sensitive detail.
- Unsafe requests never show an ambiguous outcome: before dispatch, retry is explicitly safe; after dispatch, the pending state remains until definitive success or rollback.
- Among admitted valid concurrent mutations, the last committed result is displayed; no stale-edit-conflict state exists.

## Rate and Debt States

| State | Summary | Debts | Interaction |
|---|---|---|---|
| Ready | Show Source Currency Group and per-Payer current-month totals, plus converted Group Currency Group and per-Payer totals. | Show complete Balances, Settlement Transfers, mode, calculation time, Group Currency, unique rates, and warnings. | No completion badge or extra success marker. |
| Updating | Keep Source Currency Group and per-Payer totals available. Keep prior converted values only when Group, current UTC month, Group Currency, conversion mode, relevant ledger revision, and corrected Source Currencies match the pending request; label them “Updating.” Otherwise show an Updating placeholder. | Keep a prior complete result only when Group, Group Currency, mode, relevant ledger revision, and corrected Source Currencies match; label it “Updating.” Otherwise show an Updating placeholder. | Section request proceeds automatically; no manual Retry. |
| Stale | Show converted result with one Group-level stale notice only when every quote remains eligible. | Show complete result and rate disclosure with one Group-level stale notice only when every quote remains eligible. | A context-matching fixed past-date Historical quote has no age limit. A current-date or future-date quote is eligible only through seven UTC calendar days after its effective fetch date. A warning discloses staleness but never extends either limit. |
| Provisional | Mark conversion using a current rate for a future-dated Spending. | Mark every affected disclosed rate/result. | Explain why the result is provisional. |
| Unavailable | Replace prior converted values with unavailable state; keep Source Currency Group and per-Payer totals and ledger operations usable. | Show retryable unavailable state with no partial Balances or Settlement Transfers. | Missing eligible rates, arithmetic failure, quantization failure, or settlement failure returns no partial result. Revisiting the affected section automatically retries. No manual Retry control. |

Historical is the default Debts mode and Current is not persisted. Only context-matching stale quotes may be shown; exact rate identity and calculation rules remain inherited from [PRD FR-8 through FR-12](../../prds/prd-debtor-2026-08-10/prd.md#fr-8-group-currency-summary). The visible result has one Group Currency Balance per Participant, an exact zero sum, completion-order-independent values and warnings, and positive deterministic transfers that fully settle Balances, do not repeat any pair, and number at most `n - 1`. Debtor never claims global minimality. Failure never disables ledger management except rate-dependent Participant archival.

## State Patterns

Rows define state-specific deltas; component behavior, generic focus, request outcomes, and rate states remain canonical in their named sections.

### Authentication

| State | Surface | Treatment |
|---|---|---|
| Anonymous cold load | Sign in | **Access form** with password **Field**, protected submit, and no ledger content. Retain no password after failure. |
| Authentication error | Sign in | Generic inline failure; do not disclose credentials, limiter identity, or session detail. Retryable states say only that sign-in is temporarily unavailable. |
| Session expired/restarted | Any authenticated surface | Return to Sign in without ledger content. State that the session ended; do not claim unsaved-form recovery. Lifecycle policy remains upstream. |

### Group and Participant Lifecycle

| State | Surface | Treatment |
|---|---|---|
| Empty active Groups | Groups | State no active Groups; provide create-by-name. Keep Archived Groups behind its contextual link. |
| Empty archived collection | Archived Groups / Archived Participants | State no archived records; provide safe return to Groups or Manage. Do not mix active records into this state. |
| Group create validation | Groups | Show inline name error and retain the name. Success assigns USD and targets Manage heading. |
| Group edit validation/success | Manage | Attach name/Currency errors to retained Fields. Success targets Group settings; rename preserves identity, and Currency change invalidates converted Summary/Debts context. |
| No Participants exist | Manage / Group shell | **Add Spending action** remains disabled and references guidance that links to Add Participant. The link is a 48-by-48 target. |
| All Participants archived | Manage / Group shell | **Add Spending action** remains disabled and references distinct guidance: no active Participant exists; link to Archived Participants to restore one. The link is a 48-by-48 target. |
| Participant add/edit validation | Manage | Attach name and normalized `#RRGGBB` errors to **Field** and **Participant color control**; retain raw values. Success shows the Participant's current identity throughout history and calculations without changing the Participant's accounting identity. |
| Group archive | Confirmation page / Groups | State that archive is reversible and readable. Success returns to Groups, targets the active-list heading, announces count once, and retains access through Archived Groups. |
| Group delete unavailable | Manage | A Group with any Spending does not expose delete; archive remains the lifecycle action. |
| Group delete eligible | Confirmation page | A history-free Group names the unreferenced Participants that will also be deleted. Success returns to Groups; cancellation returns to Manage. |
| Archived Group | Summary, Transactions, Debts, read-only Manage | Retain navigation and readable history/derived views; suppress mutations. Put visible “Archived” in the Group heading/name. Use definition text or native readonly controls. |
| Archived Participant in history | Transactions, Summary, Debts | Pair current name with visible “Archived” in the row label or referenced description; preserve Payer/Share facts. |
| Participant archive eligible | Manage | Offer archive only after complete all-time Historical Balance is exactly zero. Confirm scope before mutation. |
| Participant archive ineligible | Manage | State the non-zero Historical Balance condition; do not offer a path that bypasses it. |
| Participant archive rate-blocked | Manage | No state change. Show retryable feedback; revisiting Manage retries eligibility. Restore never runs this check. |
| Lifecycle/destructive confirmation | Confirmation page | Name the Participant archive, Group archive, Spending delete, or history-free Group delete; distinguish reversible archive from irreversible delete. Cancel targets the allow-listed invoker; success follows **Confirmation page** and the [matrix](#interaction-focus-matrix). |

### Spending

| State | Surface | Treatment |
|---|---|---|
| Empty Transactions | Transactions | State that no Spendings have been recorded; keep **Add Spending action** available when eligible. |
| Transactions cold load/page change | Transactions | Render complete `<details>` rows and **Pagination**. During an enhanced pending state, retain the rows and report status through **Request status**. Replacement targets Transactions heading and announces page context per the [matrix](#interaction-focus-matrix). |
| Spending preview pending/stale | Spending form | Preserve control, caret, selection, keyboard, allocation/page scroll, and row context; disable Approve. Latest valid response atomically updates derived cells, one polite status, and approval without moving focus. |
| Spending validation error | Spending form | Apply [Form and Option Contracts](#form-and-option-contracts): retain values, target the multiple-error summary or first invalid control, and link errors to controls. Native/enhanced responses share full-page markup. |
| Spending mutation success | Transactions | Create returns to page one and targets the new `<summary>`; edit returns to its canonical page/summary despite reordering. Show no completion badge; follow the [matrix](#interaction-focus-matrix). |
| Spending delete success at page boundary | Transactions | Return to the same page when possible; target next summary, previous summary, or Transactions heading when empty/page changes. Never show an out-of-range page; follow the [matrix](#interaction-focus-matrix). |
| Spending mutation failure | Spending form | Retain safely decoded values, target form heading/error, and state definitive rollback or pre-dispatch non-mutation. Never imply background completion. |

### Calculation

| State | Surface | Treatment |
|---|---|---|
| Empty current month | Summary | State that no Spendings fall in the current UTC month. Keep Source Currency and Group Currency regions distinguishable without fabricating financial activity. |
| Empty all-time ledger | Debts | State that there are no Spendings to calculate; retain **Debt mode control** and calculation context, and show no Settlement Transfers. |
| Debt mode entry/change | Debts | Historical calculates on entry. **Debt mode control** selection starts a complete safe read; selected mode remains visible, focus stays on its radio, and one status announces the replacement. |
| Debt timeout/unavailable | Debts | Apply Unavailable in [Rate and Debt States](#rate-and-debt-states): safe retryable **Request status**, no partial **Financial results**, and retry on revisit. Timeout/status policy remains upstream. |
| Debt calculation invalid | Debts | Apply Unavailable in [Rate and Debt States](#rate-and-debt-states). Arithmetic, quantization, settlement, or completion-order inconsistency yields one sanitized result without Balances/Transfers; never display non-zero-sum, incomplete, repeated-pair, or above-`n - 1` results. |

### Request and Runtime

| State | Surface | Treatment |
|---|---|---|
| Strict request/CSRF rejection | Any form | Before route parsing or dispatch, show one generic form-level request error and a fresh-form route. Do not map unknown/duplicate transport fields to valid controls or assert retention of undecodable input. |
| Submission token conflict | Any unsafe form | Per [HTTP outcomes](#http-and-session-outcomes), invalid token states produce announced `409 Conflict` in **Request status** or a focused conflict heading, no mutation, and canonical-form reload with a fresh token, never resubmission. |
| Oversized input | Sign in / any form | Do not echo the body; state no change occurred and present a fresh native form. Exact limits remain upstream. |
| Read/pre-dispatch timeout | Sign in, safe reads, mutations | Render safe retryable copy. A mutation pre-dispatch timeout states no change occurred; after dispatch keep pending until definitive result. Exact bounds remain upstream. |
| Enhanced response error | Enhanced region | Under [Foundation](#foundation), declarative handling swaps safe errors into scoped **Request status**, clears stale Updating/pending state, and retains native href/action recovery. |
| Offline/network/runtime failure | Global | Under [Foundation](#foundation), native navigation presents failure without HTMX. Enhancement failure retains content and input, clears pending only when the request outcome is known, exposes native recovery, and never claims queued, partial, or successful work. |

## Interaction Primitives

- Tap, click, and keyboard activation are equivalent. No hover-only action.
- Every control follows **Interactive target**. Important navigation and **Add Spending action** stay in the measured lower shell on narrow screens, never a top-left-only menu.
- Under the Foundation enhancement boundary, native links/forms own every route, return destination, validation result, and runtime failure. Enhancement does not change `href`, `action`, method, or full-page response.
- Preview enhancement is latest-input-wins: one form-scoped revision, cancellation of superseded requests or ignoring their responses, the smallest derived-only swap, one atomic polite announcement, and unconditional preservation of focus, caret, selection, keyboard, and scroll. Successful preview never moves focus.
- Every unsafe form explicitly requires the submission token defined by [HTTP and Session Outcomes](#http-and-session-outcomes), including login, create/edit, Approve, confirmation, restore, and Sign out. Enhanced initiators become unavailable on first activation.
- Motion is limited to immediate native focus, pressed, disclosure, and server-response state changes. No authored CSS transitions, custom-JavaScript animation, loading flourish, or celebratory completion effect.
- Spending delete, Participant archive, Group archive, and history-free Group delete pass through a server-rendered **Confirmation page**. Restore is direct but protected.
- **Banned:** modal Spending UI, long anchored Group page, drawer-only important navigation, manual rate Retry, infinite scroll, optimistic mutation claims, stale-edit conflicts, drag interaction, custom application JavaScript, and inline scripts/attributes.

### Interaction Focus Matrix

This matrix owns generic focus mechanics. Every focusable destination has a stable server-owned ID. Each forward full-page or HTMX response renders exactly one allow-listed destination with `autofocus` where the element permits it; focusable headings also receive `tabindex="-1"`. Return URLs encode only allow-listed stable destination IDs. HTMX history snapshots are disabled for private ledger content; a history miss refetches the encoded URL. Back/Forward guarantees only URL, section, page, mode, and disclosure state explicitly encoded by that URL. The browser may restore scroll or focus to a still-present control; otherwise normal document focus applies, with no promise of deterministic prior-focus restoration.

| Interaction | Success focus | Error/pending focus | Back/Forward |
|---|---|---|---|
| Group destination | Forward response autofocuses stable section heading | Pending/error retains invoker and announces **Request status** | Encoded Group/section only; browser restoration when available. |
| Pagination | Forward response autofocuses Transactions heading and announces page context | Pending/error retains activated link | Encoded page and disclosure ID only; browser restoration when available. |
| Debt mode | Enhanced response retains selected radio; native forward response autofocuses result heading | Retain radio; status describes failure | Encoded mode only; browser restoration when available. |
| Spending form open | Forward response autofocuses form `h1` | Native failure is browser/server rendered | Encoded originating section/row only; no prior-focus promise. |
| Preview | Enhanced preview never moves focus; native forward response autofocuses preview status/heading | Retain active control for enhancement; native errors autofocus summary/control | Reviewed/editable state only when URL/server encoded; browser may restore values/scroll. |
| Submitted validation | Forward response autofocuses linked alert summary, or sole invalid control | Summary links focus exact controls | Browser-restored state only; no deterministic focus promise. |
| Create/edit success | Forward response autofocuses committed Transaction `<summary>` | Definitive failure autofocuses form heading/summary | Canonical row page and optional disclosure ID are encoded. |
| Confirmation cancel | Forward response autofocuses allow-listed invoking control | N/A | Encoded return URL only; no arbitrary target. |
| Delete/archive/restore success | Forward response autofocuses next/previous summary or relevant heading; announce once | Pending retains initiator; failure autofocuses invoking context/status | Canonical destination and encoded disclosure only. |
| Group/Participant add or edit | Forward response autofocuses updated settings or Participant row action | Autofocus linked summary/invalid control | Canonical Manage section/row ID only. |
| Participant archive | Forward response autofocuses Manage Participants heading; announce count | Failure autofocuses invoking archive control/status | Encoded Manage section only. |
| Participant restore | Forward response autofocuses restored Participant row/action; announce | Failure autofocuses invoking restore control/status | Canonical Manage row ID only. |
| Group restore | Forward response autofocuses restored Group link; announce | Failure autofocuses invoking restore control/status | Canonical active Group row ID only. |
| Sign out | Forward response autofocuses Sign in heading | Failure autofocuses header control/status | Authenticated history reveals no cached ledger content; focus is not promised. |

## Accessibility Floor

- Support current stable Chrome, Firefox, Safari, and Edge down to 320 CSS pixels.
- Every control is keyboard-operable, programmatically named, and follows **Interactive target**. Native state exposes current, expanded, selected, disabled, invalid, readonly, and busy conditions; archived state uses associated visible text, not invented ARIA.
- Focus indicators use `{colors.focus}`, are at least two CSS pixels thick, sit at a two-pixel offset over the active dark surface, and exceed 3:1 against that adjacent surface. Focus is never removed; HTMX swaps manage focus deliberately.
- Normal text reaches 4.5:1 contrast. Large text, component boundaries, and meaningful graphics reach 3:1. Visual combinations are owned by `DESIGN.md`.
- Labels use `for`/`id` or wrapping association. Guidance and stable error IDs combine in `aria-describedby`; invalid controls use `aria-invalid="true"`. Multiple errors use one focused linked `role="alert"` summary. Submitted non-password values remain present.
- Status is never color-only. `{colors.warning}`, `{colors.success}`, `{colors.accent}`, and Participant colors always pair with visible text or an accessible name.
- Financial tables use semantic headings. The allocation table's labeled focusable horizontal region, sticky Participant header cells, explicit header associations, long-name wrapping, and 48-by-48 controls remain usable at 320px/400% zoom without page-level horizontal scroll. Amounts include currency in accessible text; Balance words/signs and explicit “from/to” copy communicate direction.
- **Participant color control** exposes its text value and label; the swatch has a `{colors.rule}` boundary and is supplemental. Arbitrary valid Participant colors are never assumed to meet text or status contrast.
- **Spending form** is an ordinary full page with one document scroll owner; no modal semantics or focus trap apply.
- Derived regions use one stable polite atomic **Conversion notice**, toggle `aria-busy`, and announce exactly one Updating, stale, provisional, Ready, or Unavailable transition rather than every amount. Respect reduced-motion preferences.
- General enhanced outcomes use the stable polite atomic **Request status** and owning-region `aria-busy`; the official `response-targets` extension routes expected `4xx`/`5xx` fragments there. Validation summaries and urgent session loss use alert/heading treatment instead.
- Forward responses use stable IDs and one server-rendered `autofocus` target. Back/Forward accessibility relies only on encoded state and browser-native restoration, never promised prior focus.

## Responsive & Platform

| Context | Behavior |
|---|---|
| 320px and narrow phone widths | `100dvh` page grid with one scrolling main region and intrinsically measured bottom shell. Five destinations wrap without truncation; Add Spending and conditional guidance stack above navigation; safe-area padding and intrinsic height prevent collision. Allocation alone scrolls horizontally in its named region. |
| Focused Spending form | Full page with one document scroll owner. Sticky **Form action bar** uses safe-area padding and the dynamic viewport. At 320px, Total/status spans row one; Cancel, Preview or Edit allocation, and Approve occupy three equal 48px-minimum columns with `{spacing.4}` gaps on row two. Text wraps without clipping, maximum OMR Total stays outside the Approve label, and focused fields/rows use `{components.form-action-bar.control-scroll-margin}` when the keyboard opens. |
| Wider phone/tablet | Preserve DOM/focus order; allow fields and financial rows to use width without moving primary actions out of reachable flow. |
| Width sufficient for wide composition | Summary uses the two-column editorial proportion. **Group navigation** centers in normal flow and stops consuming the bottom edge. **Spending form** remains a centered page, never a side sheet. |
| Keyboard/browser platform | Native links/forms complete every task. HTMX enhancement follows the focus matrix and stores no private history snapshot. Back/Forward restores only encoded URL/section/page/mode/disclosure state plus any focus/scroll the browser preserves; deterministic prior focus is not promised. |

No independent desktop IA, sidebar, top-left primary navigation, native app, install flow, appearance toggle, or custom gesture layer exists. The exact responsive threshold is implementation-selected by content fit; behavior, order, target size, and 320px support are fixed.

## Inspiration & Anti-patterns

- **Selected: Editorial Contrast, Variant C.** Lift the publication-like ledger, warm paper on charcoal, serif authority for totals, square geometry, rules instead of cards, and persistent yellow controls.
- **Selected: compact Group shell with section swaps.** Lift the stable shell and focused server-rendered/HTMX sections from Variant B of the IA study.
- **Rejected: Precision Ledger, Soft Nocturne, and Instrument Panel.** Do not blend their mint monospaced instrument, lavender rounded-card, or blue operational-dashboard identities into Debtor.
- **Rejected: continuous anchored Group page.** Do not preserve the long-page alternative or its scroll-position complexity.
- **Rejected: multiple Payers and Equal Shares.** Early wireframe labels are superseded by exactly one Payer and Proportional/Exact Shares.
- **Rejected: offline queue/sync and manual Retry.** Debtor is online-only; revisiting an affected derived section automatically retries.
- **Rejected: completion markers after Spending mutation.** Committed history and fresh values are sufficient proof.

## Key Flows

### Flow 1 - UJ-1: Sebr reviews a travel group's month and all-time debts

Covers **FR-4: Record a Spending**, **FR-5: Exact allocation**, **FR-6: Review and maintain history**, **FR-7: Source Currency summary**, **FR-8: Group Currency summary**, **FR-9: Select conversion mode**, **FR-10: Exact Balances**, **FR-11: Deterministic Settlement Transfers**, and **FR-12: Calculation disclosure and failure isolation**.

1. Sebr opens Debtor from a home-screen browser shortcut after paying for shopping or a shared meal and signs in if needed.
2. Groups lists each active Group by name, Group Currency, and active Participant count; Sebr opens the established travel Group in Summary.
3. Sebr activates the persistent native **Add Spending action**; the focused full-page **Spending form** opens and its heading receives focus.
4. In the **Spending form**, Sebr enters the constrained description, Total, Source Currency, date, and category, selects exactly one Payer, and uses Proportional or Exact Shares under [Financial Allocation](#financial-allocation).
5. Sebr uses native Preview; the reviewed, pending, stale, and enhanced behaviors follow **Spending form**, **Allocation table**, and **Form action bar**. Approve remains disabled until the current input is reviewed and exact.
6. Approve submits once and follows the **Spending mutation success** state, returning to Transactions with the committed **Transaction row** summary focused. The ledger remains usable while conversion recalculates.
7. Sebr opens Summary, where Group and per-Payer Source Currency figures remain available and the equivalent Group Currency figures move from Updating to their fresh result. Sebr then opens Debts in Historical mode.
8. **Climax:** Summary and Debts show the updated exact figures in place, including both Source Currency and Group Currency per-Payer current-month totals, all-time Balances, advisory Settlement Transfers, calculation disclosure, and any rate warning.

Visual sequence: [Add Spending](mockups/add-spending.html) illustrates steps 3-6, [Group Summary](mockups/group-summary.html) illustrates step 7, and [Debts](mockups/debts.html) illustrates the climax and unavailable failure boundary.

Failure path: native full-page responses present validation, transport, and runtime failures with retained safe input and unambiguous mutation outcome. If no eligible quote exists, converted values become unavailable while Source Currency summary, history, and ledger mutations remain usable; Debts shows no partial result.

### Flow 2 - UJ-2: Sebr prepares and maintains a Group

Covers **FR-2: Group lifecycle** and **FR-3: Group-owned Participants**.

1. From Groups, Sebr creates a Group with only a valid name.
2. Debtor assigns USD as Group Currency and opens Manage.
3. Setup guidance explains that Sebr can edit Group name and Group Currency and add Participants; **Add Spending action** follows the **No Participants exist** state.
4. Sebr sets the intended Group Currency and adds Group-owned Participants with names and suggested editable `#RRGGBB` colors. The first active Participant unlocks **Add Spending action**.
5. Sebr later renames the Group and edits a Participant's name/color. Manage returns with retained identity, and the Participant's current name appears throughout historical Spendings and calculations.
6. Sebr opens Manage to archive a Participant; the Participant archive states govern the complete all-time Historical Balance eligibility check.
7. Sebr follows **Participant archive eligible**, **Participant archive rate-blocked**, or **Participant archive ineligible**, including confirmation only for exact-zero eligibility.
8. Sebr uses **Archived view** to restore the Participant without a Balance check. Group archive uses its reversible **Confirmation page** and Archived Groups restores it; irreversible Group delete appears only for **Group delete eligible**.
9. **Climax:** Manage shows the intended name, active roster, colors, and Group Currency, while historical Spendings resolve every current Participant name and no referenced accounting identity has been lost.

Visual sequence: [Manage](mockups/manage.html) illustrates setup, identity editing, zero-Balance eligibility, rate-blocked archival, and lifecycle ordering.

Failure path: a stale eligibility view cannot bypass the archive rule; concurrent Spending change or unavailable required rate returns a safe non-mutation result. Reopening Manage reevaluates eligibility.

### Flow 3 - Sebr secures the private ledger

Covers **FR-1: Password-gated access**.

1. Sebr opens Debtor without an authenticated session.
2. Sign in displays the **Access form** with one password **Field** and protected submit; no username, registration, or Participant login appears.
3. Sebr submits the configured password.
4. On success, Debtor opens Groups and focuses its page heading.
5. **Climax:** Sebr sees the private Groups list, while anonymous visitors remain unable to view any ledger data.
6. Sebr can activate Sign out from any **Page header**, returning to Sign in. **Session expired/restarted** does the same without exposing ledger content.

Failure path: incorrect credentials produce a generic inline error and retain no password. Retryable limiter or authenticated-session capacity conditions disclose no secret or client identity and do not expose ledger content.

### Flow 4 - Sebr corrects or removes a Spending without losing context

Extends **FR-6: Review and maintain history**.

1. Sebr opens Transactions; the newest 25 Spendings are shown first.
2. Sebr selects a **Transaction row**, which expands in place with complete Payer, Share, Source Currency, category, and date details.
3. To correct it, Sebr opens the focused **Spending form** under the edit rules in [Financial Allocation](#financial-allocation), previews, and approves; affected historical calculations use the corrected stored Source Currency.
4. To delete it, Sebr chooses Delete and moves to a server-rendered **Confirmation page** naming the Spending and effect.
5. **Climax:** after confirmation, Transactions returns without the Spending and every remaining historical record stays readable; cancellation instead returns with no mutation. **Spending delete success at page boundary** and **Spending mutation success** define the canonical page and focus destination.

Visual sequence: [Transactions](mockups/transactions.html) illustrates expanded Spending facts/actions, pagination, and the canonical focused-row return.

Failure path: validation retains submitted edits on the full page. A mutation failure reports a definitive safe failure and never presents a partial update.
