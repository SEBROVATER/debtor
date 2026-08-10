# One-Handed Mobile Stress Review

**Scope:** Complete UJ-1 and UJ-2 replay at 320 CSS pixels through Groups, the compact switched Group shell, persistent Add Spending, the focused full-page Spending form, sticky actions, allocation previews and horizontal table, Transactions, Summary, Debts, Manage, archive/restore, confirmations, page-header Sign out, interruptions/errors, and the single-layout desktop adaptation.

**Verdict:** Implementation-ready for the confirmed one-handed interaction intent. No real unhandled risks remain in the reviewed spines.

## Findings

None.

## Final Verification

| Boundary | Result | Evidence |
|---|---|---|
| Native HTMX fallback and expected error presentation | Closed | Native links/forms remain complete paths; the pinned official `response-targets` extension routes expected enhanced errors to a stable announced status region without custom handlers (`EXPERIENCE.md:16-18`, `EXPERIENCE.md:95`, `EXPERIENCE.md:179-180`, `EXPERIENCE.md:186`). |
| Full-page form, keyboard, and safe-area geometry | Closed | One document scroll owner, `100dvh`, dynamic-viewport sticky actions, safe-area padding, and maximum-bar-height scroll margin keep focused controls reachable (`DESIGN.md:315`, `DESIGN.md:345`; `EXPERIENCE.md:234`). |
| 320px sticky action-bar crowding | Closed | Total/status occupies a wrapping first row; Cancel, Preview or Edit allocation, and Approve occupy three equal 48px-minimum columns; the amount stays outside Approve (`DESIGN.md:179-190`, `DESIGN.md:315`, `DESIGN.md:345`; `EXPERIENCE.md:84`, `EXPERIENCE.md:234`). |
| Latest-input allocation previews | Closed | Form revision, superseded-response rejection, derived-only swaps, pending/stale approval suppression, and focus/caret/keyboard/scroll retention are explicit (`EXPERIENCE.md:83`, `EXPERIENCE.md:113`, `EXPERIENCE.md:161`, `EXPERIENCE.md:187`). |
| Native and enhanced duplicate-mutation suppression | Closed | Every unsafe form uses a bounded, expiring, session-bound single-use token atomically reserved immediately before dispatch; invalid/reused tokens return announced `409` without use-case invocation (`specs/design.md:91`; `EXPERIENCE.md:20`, `EXPERIENCE.md:122-123`, `EXPERIENCE.md:176`, `EXPERIENCE.md:188`). |
| Horizontal allocation table | Closed | The table has a labeled focusable internal scroll region, sticky bounded Participant column, semantic associations, long-name breaking, 48px controls, derived-only swaps, and no page-level horizontal scroll (`DESIGN.md:317`, `DESIGN.md:344`; `EXPERIENCE.md:83`, `EXPERIENCE.md:222`). |
| Confirmation, mutation, restore, and Sign out focus | Closed | Forward responses carry one allow-listed autofocus target; canonical success, cancel, conflict, and failure destinations are explicit (`EXPERIENCE.md:73-74`, `EXPERIENCE.md:86-88`, `EXPERIENCE.md:174-176`, `EXPERIENCE.md:193-212`). |
| Transactions expansion, pagination, Summary, and Debts | Closed | Native disclosures and links preserve full-page paths; pagination, mode, updating, unavailable, and complete-result states have explicit focus/status behavior (`EXPERIENCE.md:85`, `EXPERIENCE.md:92-95`, `EXPERIENCE.md:128-138`, `EXPERIENCE.md:160`, `EXPERIENCE.md:171-173`). |
| Manage, archive, restore, and stale eligibility | Closed | Active/archived separation, rate-blocked eligibility, confirmations, direct protected restore, canonical returns, and announcements are explicit (`EXPERIENCE.md:86-90`, `EXPERIENCE.md:150-156`, `EXPERIENCE.md:166-170`, `EXPERIENCE.md:207-211`). |
| Persistent 320px Group shell | Closed | The intrinsic bottom shell owns Add Spending, setup guidance, wrapping five-destination navigation, focus clearance, and safe-area padding without overlaying content (`DESIGN.md:311`, `DESIGN.md:333-336`; `EXPERIENCE.md:72-75`, `EXPERIENCE.md:233`). |
| Desktop single-layout adaptation | Closed | The same DOM, destination order, and focus order move into normal flow; no separate desktop IA or side-sheet form exists (`DESIGN.md:311-315`, `DESIGN.md:341`; `EXPERIENCE.md:235-239`). |

## Severity Count

| Severity | Count |
|---|---:|
| Critical | 0 |
| High | 0 |
| Medium | 0 |
| Low | 0 |
| **Total** | **0** |
