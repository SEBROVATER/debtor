# PRD to Polished UX Final Closure Report

## Scope

This final closure pass re-read the latest confirmed `prd.md`, UX `.memlog.md`, polished `DESIGN.md`, and polished `EXPERIENCE.md`. It reconciles the current source rather than carrying forward superseded conclusions from earlier passes. Particular attention was given to the later decision sequence: single Payer with Proportional/Exact Shares, Participant archive eligibility, Group defaults, authoritative UJ-2, native-first HTMX enhancement, the full-page Spending form, single-use submission tokens, and the polished accessibility/focus/table/action-bar contracts.

The PRD is the confirmed product source. The memlog is provenance, not an independent requirements source. An override is accepted only where a later memlog change records synchronization into the normative source set and the current PRD reflects that change. `DESIGN.md` and `EXPERIENCE.md` are assessed together; mockups are illustrative and subordinate to those spines.

## Verdict

**Closed.** The polished UX spines preserve the latest confirmed PRD and correctly apply all later source-synchronized decisions. Earlier required-HTMX/native-overlay conclusions are explicitly superseded by the current native-first, full-page source contract. All prior reconciliation findings remain resolved. No critical, high, medium, or low findings remain.

## Counts

| Severity | Count |
|---|---:|
| Critical | 0 |
| High | 0 |
| Medium | 0 |
| Low | 0 |

## Later Decision Reconciliation

### Single Payer and Proportional/Exact Shares

**Closed.** The memlog override replaces multiple-Payer plus Equal/Exact behavior with exactly one Payer and Proportional/Exact Shares, then records synchronization into the PRD and supporting source set (`.memlog.md` 19-20).

The current artifacts agree:

- The PRD requires exactly one Payer and only Proportional or Exact Shares (`prd.md` 160, 168-180); multiple Payers and percentage/itemized modes remain out of scope (`prd.md` 74, 86).
- Selecting a Payer assigns the full Total and selecting another replaces it (`prd.md` 168; `EXPERIENCE.md` 126).
- Proportional create defaults every active Participant to weight `1`, allows deselection, requires positive decimal weights, and displays exact Source Currency allocations under FR-5 (`prd.md` 178; `EXPERIENCE.md` 128).
- Exact create defaults equal minor-unit Shares and displays Remaining/Excess (`prd.md` 179; `EXPERIENCE.md` 129).
- The largest-fractional-remainder algorithm and Participant-ID tie break remain authoritative through the explicit FR-5 inheritance (`prd.md` 178; `EXPERIENCE.md` 128).
- Edit opens Exact with stored Payer/Shares because mode and weights are not persisted; archived roles cannot be introduced or moved (`prd.md` 180, 190; `EXPERIENCE.md` 93, 131).
- Every accepted allocation independently conserves the Total across the one Payer and Shares at Source Currency precision (`prd.md` 174-180; `EXPERIENCE.md` 132-135).

No rejected Equal-share or multiple-Payer behavior survives in the polished contracts (`EXPERIENCE.md` 127, 294).

### Participant Archive

**Closed.** The memlog's zero-Historical-Balance archive override was synchronized upstream (`.memlog.md` 14-15) and is now part of both UJ-2 and FR-3 (`prd.md` 35, 149-150).

The UX contract preserves all branches:

- Archive is offered only after a complete all-time Historical calculation gives the Participant an exact zero Group Currency Balance (`EXPERIENCE.md` 191, 326-328).
- Non-zero Balance makes archive unavailable and exposes no bypass (`EXPERIENCE.md` 192).
- Missing required rates leave the Participant active, present retryable feedback, and trigger reevaluation when Manage is revisited (`EXPERIENCE.md` 193, 333).
- A stale eligibility view cannot bypass the rule and concurrent Spending changes produce safe non-mutation (`EXPERIENCE.md` 333).
- Eligible archive uses a named reversible confirmation and a protected unsafe submit (`EXPERIENCE.md` 103, 194, 237).
- Restore remains available without a Balance check and returns focus to the restored row/action (`prd.md` 150; `EXPERIENCE.md` 104, 193, 257, 328).
- Archived Participants remain visible by current name plus explicit Archived text in history and calculations (`prd.md` 148, 188; `EXPERIENCE.md` 190).

### Group Defaults, Lifecycle, and UJ-2

**Closed.** The name-only/USD/Manage override was synchronized upstream (`.memlog.md` 16-17), and the later change explicitly added UJ-2 to the authoritative PRD (`.memlog.md` 39; `prd.md` 35).

The current UJ-2 chain is complete and consistent:

- Create asks only for a valid name, assigns USD, and opens Manage (`prd.md` 35, 136; `EXPERIENCE.md` 181, 321-323).
- Established Groups open Summary (`prd.md` 136; `EXPERIENCE.md` 45).
- Manage supports Group name/Currency edits and Participant add/edit with suggested editable normalized colors (`prd.md` 132-152; `EXPERIENCE.md` 38, 91-92, 182, 185, 324-325).
- Add Spending stays disabled when no active Participant exists, with distinct recovery for no Participants versus all archived (`EXPERIENCE.md` 75, 183-184).
- Contextual Archived Groups and Archived Participants remain separate from active lists and provide restoration (`prd.md` 137-139; `EXPERIENCE.md` 34, 39, 45, 104, 180, 328).
- Group archive is reversible/readable; Group delete appears only when history-free and names deleted unreferenced Participants (`prd.md` 132, 138-139; `EXPERIENCE.md` 186-189).
- UJ-2 explicitly covers setup completion, archive eligibility, rate blocking, restore, Group archive/restore, and eligible deletion (`prd.md` 35; `EXPERIENCE.md` 317-333).

### Native-First HTMX Fallback

**Closed.** The memlog first records a required-HTMX/native-overlay direction (`.memlog.md` 33-34), then explicitly supersedes it with native HTML plus optional progressive enhancement and records synchronization into the PRD and architecture sources (`.memlog.md` 36-37). The latest decision wins and is reflected in the current PRD (`prd.md` 54, 56, 60, 156).

The polished spines implement the superseding contract:

- Semantic server-rendered HTML and valid native links/forms complete every task (`EXPERIENCE.md` 18, 43, 233, 284).
- Pinned self-hosted HTMX and the pinned official `response-targets` extension are optional enhancements only (`EXPERIENCE.md` 18, 115).
- No custom application JavaScript, inline script, inline script attribute, or unapproved extension is allowed (`EXPERIENCE.md` 18, 115, 238).
- Enhancement never changes native `href`, `action`, method, return destination, or full-page response (`EXPERIENCE.md` 43, 233).
- Native links/forms expose validation, transport, runtime, pagination, debt mode, navigation, and request-error outcomes when enhancement is absent or fails (`EXPERIENCE.md` 20, 97, 105, 114-115, 226-227, 284).
- HTMX history snapshots are disabled for private ledger content; misses refetch encoded URLs, and Back/Forward promises only encoded state plus browser-native restoration (`EXPERIENCE.md` 242, 284).

There is no remaining required-HTMX, unsupported-`noscript`, modal-overlay, or CSP contradiction in the current UX contract.

### Full-Page Spending Form and Native Preview

**Closed.** The source-synchronized superseding decision makes Add Spending a focused full-page form and requires an explicit native Preview submit (`.memlog.md` 36-37; `prd.md` 54, 156).

The polished contract is unambiguous:

- Add Spending is a native link from every active Group section to a focused full-page form (`EXPERIENCE.md` 40, 43, 75, 306).
- The form remains a centered full page at wide widths and has no modal, sheet, scrim, or focus trap (`DESIGN.md` 315, 323, 327, 341; `EXPERIENCE.md` 238, 271, 283).
- Native Preview submits the full page and renders reviewed, non-editable input with Approve and Edit allocation; approval therefore cannot outlive reviewed input (`EXPERIENCE.md` 91, 94-95, 133, 308).
- HTMX may preview field changes but swaps only stable derived cells, approval state, and one status node; focused controls remain outside the swap (`EXPERIENCE.md` 94, 133).
- Latest input wins; superseded responses do not swap; focus, caret, selection, software keyboard, active row, and table/page scroll remain unchanged (`EXPERIENCE.md` 94, 133, 202, 234).
- Approve is the sole ledger mutation and is disabled whenever preview is pending, stale, invalid, or superseded (`EXPERIENCE.md` 95, 133).
- Successful create/edit returns to the canonical Transactions page and focuses the committed row summary (`prd.md` 54, 156; `EXPERIENCE.md` 96, 204, 252, 309).

### Single-Use Submission Token

**Closed.** The later memlog override adds a bounded, expiring, session-bound, single-use token distinct from CSRF to every unsafe form and records synchronization into the source set (`.memlog.md` 41-42). The PRD now contains the same feature and cross-cutting requirement (`prd.md` 124, 258-262).

The UX lifecycle is complete:

- Every unsafe form explicitly carries one token distinct from CSRF, including login, create/edit, Approve, confirmations, restore, and Sign out (`EXPERIENCE.md` 22, 73, 91, 95, 103-108, 143, 235).
- Valid pre-dispatch form errors preserve the token (`EXPERIENCE.md` 144-145).
- Reservation is atomic immediately before use-case dispatch; after reservation the request remains pending until definitive commit or rollback (`EXPERIENCE.md` 145, 148).
- Missing, unknown, expired, reserved, or consumed tokens return an announced `409 Conflict`, invoke no use case, and offer canonical-form reload with a fresh token rather than replay (`EXPERIENCE.md` 146, 223).
- First activation suppresses/coalesces duplicate initiation and makes the initiating unsafe control unavailable while pending (`EXPERIENCE.md` 73, 95, 103, 105, 235).
- The token never substitutes for CSRF and never replays a prior result (`prd.md` 124, 259; `EXPERIENCE.md` 143, 146).

No unsafe-form component is omitted from the token contract.

## Polished Accessibility and Interaction Verification

### Interactive targets and responsive shell

**Closed.** Every button, link, summary, labeled radio/checkbox, input, select, row action, Group row, and destination must render at least 48 by 48 CSS pixels at 320px and 400% zoom; there are no inline-link exceptions (`DESIGN.md` 93-96, 333; `EXPERIENCE.md` 72, 232, 264). The narrow shell is an intrinsic `100dvh` grid region rather than an overlay, reserves safe-area space, permits two-line labels, and never covers main content (`DESIGN.md` 311; `EXPERIENCE.md` 280).

Disabled Add Spending guidance is associated through `aria-describedby` and provides a 48-by-48 recovery link, with distinct no-Participant and all-archived copy (`EXPERIENCE.md` 75, 183-184).

### Focus and history

**Closed.** The Interaction Focus Matrix centralizes stable server-owned IDs, one allow-listed forward destination, native/HTMX success and error targets, cancellation returns, mutation returns, and bounded Back/Forward guarantees (`EXPERIENCE.md` 240-259).

- Full-page Spending open targets the form `h1`; create/edit success targets the committed Transaction summary (`EXPERIENCE.md` 249, 252).
- Enhanced Preview never moves focus; native Preview targets its status/heading; validation targets a linked alert summary or sole invalid control (`EXPERIENCE.md` 250-251).
- Debt mode enhancement retains the selected radio; native navigation targets the result heading (`EXPERIENCE.md` 114, 214, 248).
- Pending and expected failures retain the invoker and announce one scoped status; session loss uses an urgent server-rendered heading/alert (`EXPERIENCE.md` 115).
- Back/Forward promises only URL-encoded section/page/mode/disclosure state and browser-native restoration, avoiding a false deterministic prior-focus guarantee (`EXPERIENCE.md` 242, 284).

### Allocation table

**Closed.** The polished 320px allocation design remains a semantic table in its own labeled, keyboard-focusable horizontal scroll region, with no page-level horizontal scroll (`DESIGN.md` 317, 344; `EXPERIENCE.md` 94, 269-270).

- The intrinsic 520px table and column widths are explicit; Participant identity is sticky at the inline edge with an opaque background and visible boundary (`DESIGN.md` 317).
- Long names up to 100 Unicode characters wrap/break without clipping, Share amounts remain unbroken/right-aligned, and controls remain 48 by 48 (`DESIGN.md` 317; `EXPERIENCE.md` 94).
- Stable labels/descriptions and explicit header IDs preserve row/column/control associations (`EXPERIENCE.md` 94).
- The contract explicitly requires verification at 320px, 400% zoom, enlarged text, maximum OMR Total, and a 100-character Participant name (`DESIGN.md` 317).

### Form action bar

**Closed.** At 320px the action bar has Total/status on row one and three equal minimum-48px actions on row two in the required order: Cancel, Preview or Edit allocation, Approve (`.memlog.md` 40; `DESIGN.md` 179-190, 315, 345; `EXPERIENCE.md` 95, 281).

The amount never appears inside Approve; labels/status wrap without clipping; safe-area, dynamic viewport, keyboard, maximum OMR Total, and maximum wrapped height are accounted for through document flow and control scroll margin (`DESIGN.md` 315, 345; `EXPERIENCE.md` 95, 281).

### Validation, statuses, and derived regions

**Closed.** Labels, guidance, stable error IDs, `aria-invalid`, retained values, and one linked focused `role="alert"` summary for multiple errors are explicit (`EXPERIENCE.md` 92, 119-122, 203, 267). Allocation-wide errors describe the region and Approve; row errors attach only to the owning input (`EXPERIENCE.md` 122).

Conversion and request statuses use stable scoped polite atomic live regions with `aria-busy`; they announce one transition rather than every amount and distinguish expected failures from urgent session loss (`EXPERIENCE.md` 82, 115, 272-273). Archived state uses visible associated text rather than invented ARIA (`EXPERIENCE.md` 104, 189-190, 264).

### Contrast

**Closed.** The polished tokens retain the independently passing set. Recalculated WCAG ratios are:

| Foreground / adjacent background | Ratio | Required | Result |
|---|---:|---:|---|
| `rule #6D6C69` / `background #101113` | 3.60:1 | 3:1 | Pass |
| `rule #6D6C69` / `surface #181A1D` | 3.32:1 | 3:1 | Pass |
| `rule #6D6C69` / `surface-strong #202226` | 3.03:1 | 3:1 | Pass |
| `rule #6D6C69` / `navigation #151619` | 3.45:1 | 3:1 | Pass |
| `rule #6D6C69` / `input #121315` | 3.54:1 | 3:1 | Pass |
| `text #F5F0E7` / active dark surfaces | 14.03-16.64:1 | 4.5:1 | Pass |
| `text-muted #AAA59C` / active dark surfaces | 6.50-7.71:1 | 4.5:1 | Pass |
| `warning #E88467` / active dark surfaces | 6.01-7.13:1 | 3:1 | Pass |
| `warning-text #F4BAA7` / active dark surfaces | 9.43-11.18:1 | 4.5:1 | Pass |
| `focus #FFFFFF` / active dark surfaces | 15.93-18.89:1 | 3:1 | Pass |
| `on-accent #211C08` / `accent #F0D36C` | 11.54:1 | 4.5:1 | Pass |
| `success #A9D6A0` / `surface-strong #202226` | 9.72:1 | 4.5:1 | Pass |

The narrowest passing boundary is `rule` on `surface-strong` at 3.03:1. The exact opaque token must be retained without opacity or transform antialiasing that reduces effective contrast.

## Earlier Reconciliation Closure

All earlier findings remain resolved under the polished contracts:

| Earlier area | Final verification |
|---|---|
| Group rename and Participant edit/color | Present in Manage with retained validation, identity preservation, normalized color, suggestion, and historical current-name display. |
| Group/Participant/Spending field contracts | Exact options, defaults, length/date/precision/amount/allocation rules inherit directly from PRD Features and are exposed through guidance/errors. |
| Per-Payer Source and Group Currency summaries | Explicit in IA, Financial results, Rate states, and UJ-1. |
| Source Currency correction | Editable under create validation; corrected stored currency invalidates/recomputes derived results. |
| Archived Group navigation | Five persistent native destinations remain; all mutation controls are suppressed. |
| Manage, archive, pagination, Debts mode, financial, sign-in, and Groups surfaces | Complete component/state contracts remain; five promoted mocks are illustrative only and subordinate to the spines. |
| Updating-value reuse | Restricted to matching Group, period where applicable, Group Currency, mode, ledger revision, and corrected Source Currencies. |
| Edit/delete page boundaries | Canonical page and next/previous/heading focus destinations are deterministic. |
| Structural request rejection | Untrusted structure maps to safe fresh-form recovery before dispatch; no false retained-field claim. |
| Stale quote eligibility | Fixed historical quotes have no age limit; current/future stale quotes have the seven-UTC-day bound; warnings never extend eligibility. |
| Balance/Settlement invariants | Exact zero sum, completion-order independence, full deterministic settlement, pair uniqueness, `n - 1` bound, and no partial invalid result remain explicit. |
| Product smallness, personal/shared use, trust, and stewardship | Preserved in Foundation, Brand, UJ-2, and maintenance/correction flows. |

## Provenance Closure

The complete later decision chain is correctly represented:

- `.memlog.md` 14-20: Participant archive, Group defaults, and single-Payer/Proportional allocation overrides were synchronized upstream and remain current.
- `.memlog.md` 28 and 39: UJ-2 was confirmed and then added to the authoritative PRD; the PRD and Experience flow now agree.
- `.memlog.md` 33-34: required HTMX/native overlay was a source-synchronized intermediate decision.
- `.memlog.md` 36-37: native-first optional HTMX, mandatory native fallbacks, full-page Spending, and explicit Preview later superseded that intermediate decision and were synchronized upstream; the current PRD and spines consistently implement the later rule.
- `.memlog.md` 38 and 40: the horizontal semantic allocation table and two-row/three-action bar are fully extracted into both polished spines.
- `.memlog.md` 41-42: single-use submission tokens and declarative expected-error handling are synchronized and fully represented.
- `.memlog.md` 44-46: promoted mockups remain illustrative references; all load-bearing decisions are extracted into the spines, so no mock is required to resolve a contract ambiguity.

## Final Assessment

No latest confirmed PRD requirement or later source-synchronized decision is dropped, contradicted, or left as an unresolved UX surface. The polished DESIGN and EXPERIENCE contracts are closed for downstream implementation planning.
