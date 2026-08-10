# Spine Pair Review — debtor

## Overall verdict

The current pair is a strong downstream contract: updated source journeys and requirements trace cleanly, every token and component reference resolves, all applicable surface states are committed, and the visual and behavioral spines divide responsibility coherently. The newly introduced submission-token and HTMX response-handling decisions are synchronized across the sources and experience contract; draft status remains only the scheduled close step.

## 1. Flow coverage — strong

Checked both frontmatter sources. The PRD defines **UJ-1. Sebr reviews a travel group's month and all-time debts**, **UJ-2. Sebr prepares and maintains a Group**, and **FR-1: Password-gated access** through **FR-12: Calculation disclosure and failure isolation** (`prd.md` lines 32-35, 116-245). EXPERIENCE.md covers UJ-1 and FR-4 through FR-12 in Flow 1, UJ-2 and FR-2 through FR-3 in Flow 2, FR-1 in Flow 3, and the correction/deletion behavior of FR-6 in Flow 4 (`EXPERIENCE.md` lines 251-307). Each flow has the named protagonist Sebr, numbered steps, an explicit climax, and an applicable failure path. The updated FR-1 duplicate-submission protection is carried through the access, unsafe-action, conflict, and failure behavior even though it does not require a separate journey.

### Findings

No findings.

## 2. Token completeness — strong

Extracted 15 color tokens, 11 typography roles, two rounded tokens, 13 spacing tokens, and 24 component token objects from DESIGN.md frontmatter (`DESIGN.md` lines 11-273). Every color is a six-digit hex value; typography roles use allowed fields; rounded and spacing values are valid CSS dimensions; and component values are literals or valid token references. Every `{path.to.token}` reference in both spines resolves to the DESIGN.md YAML tree, including the revised Form action bar references. Load-bearing contrast targets and measured combinations cover normal text, large text, boundaries, warning boundaries, accent text, and focus (`DESIGN.md` lines 285-305; `EXPERIENCE.md` lines 214-227).

### Findings

No findings.

## 3. Component coverage — strong

Extracted the shared component vocabulary: **Interactive target**, **Page header**, **Group navigation**, **Add Spending action**, **Ledger section**, **Conversion notice**, **Money row**, **Participant marker**, **Spending form**, **Field**, **Share mode control**, **Allocation table**, **Form action bar**, **Transaction row**, **Confirmation page**, **Archived view**, **Access form**, **Group list**, **Management form**, **Participant color control**, **Pagination**, **Debt mode control**, **Financial results**, and **Request status**. All 24 have substantive visual contracts in DESIGN.md.Components and one-for-one behavioral contracts in EXPERIENCE.md.Component Patterns, with identical names (`DESIGN.md` lines 329-356; `EXPERIENCE.md` lines 66-95). Updated submission-token, response-target, autofocus, and action-bar behavior is assigned to the relevant components rather than left implicit.

### Findings

No findings.

## 4. State coverage — strong

Walked every IA surface (`EXPERIENCE.md` lines 28-47). Sign in covers anonymous cold load, authentication errors, session expiry/restart, duplicate-submission conflict, limits, timeout, and network/runtime failure. Groups and both archived collections cover empty, validation, lifecycle, restoration, submission-token conflict, and focus outcomes. Summary covers empty month and Ready, Updating, Stale, Provisional, and Unavailable conversion states. Transactions covers empty, cold/page load, pagination, create/edit/delete success, page-boundary deletion, validation, rollback, archived history, submission conflict, and focus restoration. Debts covers empty ledger, mode entry/change, timeout/unavailable, invalid calculations, and no-partial-result behavior. Manage covers no Participants, all Participants archived, Group/Participant validation, all archival eligibility outcomes, read-only archived Groups, conflict-safe lifecycle actions, and confirmations. Spending form covers native/enhanced preview, pending/stale preview, validation, mutation success/failure, retained input/token, conflict recovery, and focus/scroll preservation. Confirmation page covers cancel, pending, token conflict, failure, successful archive/delete, duplicate suppression, and canonical return focus. Global strict-request, submission-token, oversized-input, timeout, enhanced-response, offline/network/runtime, status-announcement, and accessibility focus states cover the remaining cases (`EXPERIENCE.md` lines 128-227). Permission-denied and offline-write states do not apply to this single-Administrator, online-only product.

### Findings

No findings.

## 5. Visual reference coverage — strong

`imports/` is present and empty. `mockups/` and `wireframes/` are absent, so the rubric directories contain no files to orphan or reference unspecifically. Both working references resolve: `.working/flow-group-shell-2026-08-10.excalidraw` illustrates the selected compact Group shell, and `.working/directions-dark-4.html#direction-c` illustrates Editorial Contrast. Both spines explicitly limit those artifacts to extracted direction and supersede obsolete modal, dimension, contrast, and implementation details (`DESIGN.md` line 283; `EXPERIENCE.md` lines 22, 47). The pair states once that the spines win over conflicting mocks, wireframes, working artifacts, or imports (`EXPERIENCE.md` line 22).

### Findings

No findings.

## 6. Bloat & overspecification — strong

Checked token-covered pixel repetition, source restatement, prose that should be tabular, downstream-irrelevant sections, and decorative narrative. Foundation, Form and Option Contracts, and HTTP and Session Outcomes inherit exact security, transport, status, limit, and timeout policy through specific source links while retaining user-visible consequences (`EXPERIENCE.md` lines 14-26, 97-126). The submission-token paragraphs are load-bearing behavioral deltas backed by matching PRD and addendum requirements, not speculative restatement. Financial Allocation and Rate and Debt States retain interaction-critical input, preview, result, and failure behavior; the focus matrix directly contracts forward-response and browser-history behavior. DESIGN.md keeps visual values in tokens/component contracts and uses prose for rationale and responsive composition rather than duplicating upstream product scope.

### Findings

No findings.

## 7. Inheritance discipline — strong

Both absolute frontmatter source paths resolve, and EXPERIENCE.md's relative section links resolve into those same sources (`DESIGN.md` lines 8-10; `EXPERIENCE.md` lines 7-9). UJ-1, UJ-2, and FR-1 through FR-12 names match verbatim. The updated single-use submission-token requirement is consistent between PRD FR-1 and Security and Privacy, addendum HTTP dispatch rules, and EXPERIENCE.md Foundation, component, outcome, state, and interaction contracts (`prd.md` lines 116-124, 256-262; `addendum.md` lines 69-76; `EXPERIENCE.md` lines 16-20, 73-95, 117-126, 174-188). The pinned official `response-targets` extension is likewise consistent between addendum and EXPERIENCE.md. Product terminology and capitalization are stable, all 24 component names match across spines, and every EXPERIENCE.md token reference resolves to DESIGN.md.

### Findings

No findings.

## 8. Shape fit — strong

DESIGN.md contains every canonical section in order: Brand & Style, Colors, Typography, Layout & Spacing, Elevation & Depth, Shapes, Components, and Do's and Don'ts (`DESIGN.md` lines 277-369). EXPERIENCE.md contains all required defaults: Foundation, Information Architecture, Voice and Tone, Component Patterns, State Patterns, Interaction Primitives, Accessibility Floor, and Key Flows. Responsive & Platform is correctly present for the responsive surface, Inspiration & Anti-patterns is correctly triggered by the direction and IA studies, and the product-specific Form and Option Contracts, Financial Allocation, HTTP and Session Outcomes, Rate and Debt States, and Interaction Focus Matrix sections each carry downstream behavioral decisions. Both `status: draft` values are intentionally ignored as the scheduled final-close step (`DESIGN.md` line 5; `EXPERIENCE.md` line 4).

### Findings

No findings.

## Mechanical notes

- Frontmatter is syntactically complete for its role. DESIGN.md includes required `name` and `description`; both source lists resolve.
- Every local Markdown link and fragment resolves. There are no orphan files in `imports/`, `mockups/`, or `wireframes/`.
- Every `{path.to.token}` cross-reference resolves, including nested Form action bar references.
- All 24 component names match one-for-one across DESIGN.md Components and EXPERIENCE.md Component Patterns.
- Updated submission-token and `response-targets` extension decisions resolve consistently through both sources and EXPERIENCE.md.
- Draft status is ignored as the scheduled final-close step.
- No Mermaid diagrams are present, so Mermaid syntax is not applicable.
- Finding counts: critical 0, high 0, medium 0, low 0.
