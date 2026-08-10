# Accessibility Review - Final Verification

**Artifacts reviewed:** Current `DESIGN.md`, current `EXPERIENCE.md`, the confirmed source floor in `specs/design.md`, `.working/directions-dark-4.html` Variant C, `.working/flow-group-shell-2026-08-10.excalidraw` Variant B, and every finding from both prior accessibility-review passes.

**Review basis:** The normative source requires native server-rendered completion when HTMX is unavailable; current stable Chrome, Firefox, Safari, and Edge through 320 CSS pixels; keyboard operation; programmatic labels; visible focus at least 2 CSS pixels thick and 3:1 against adjacent colors; 4.5:1 normal text; 3:1 large text, components, and meaningful graphics; and programmatic inline-error association (`specs/design.md:60-67,91`). The UX spines strengthen this with 48-by-48 targets at 320px/400% zoom, explicit error summaries, an internally scrolling allocation table, scoped HTMX swaps, announced status regions, and a bounded focus/history contract (`EXPERIENCE.md:214-227`).

## Verdict

**Accessibility-ready for implementation.** No Critical, High, Medium, or Low specification findings remain. The final fixes resolve the last focus/history mechanism, enhanced-error announcement, and target-size consistency issues. All required behaviors are now expressed as implementable native or explicitly permitted declarative HTMX patterns. The selected working references remain visually useful but are explicitly and adequately superseded as implementation sources.

## Findings

No remaining findings.

## Final Fix Verification

| Final prior finding | Status | Verification |
|---|---|---|
| A11Y-R1 Medium - Focus/history restoration mechanism | **Resolved** | Every focus destination has a stable server-owned ID; forward responses have exactly one allow-listed `autofocus` target, focusable headings use `tabindex="-1"`, return URLs encode only allow-listed IDs, private HTMX history snapshots are disabled, and Back/Forward promises only encoded state plus browser-native restoration (`EXPERIENCE.md:74-75,86-87,92,193-212,227,237`). |
| A11Y-R2 Medium - Enhanced errors not guaranteed to announce | **Resolved** | **Request status** is a stable polite atomic `role="status"`; its owner toggles `aria-busy`; the pinned official `response-targets` extension routes expected `4xx`/`5xx` fragments; routine errors retain invoker focus and announce once; urgent session loss uses focused alert/heading treatment (`specs/design.md:91`; `EXPERIENCE.md:16,20,95,123-125,176,179,186,188,226`). |
| A11Y-R3 Low - 48px inline-link inconsistency | **Resolved** | Both spines now state that every link/control is at least 48 by 48 and that there are no link exceptions (`DESIGN.md:333,366`; `EXPERIENCE.md:72,185,217`). |

## Complete Prior-Finding Verification

| Original finding | Final status | Verification |
|---|---|---|
| A11Y-01 Critical - Modal architecture conflict | **Resolved** | Spending is a native focused full-page form; modal, overlay, trap, Escape-close, scrim, and side-sheet behavior are removed and banned (`specs/design.md:61,91`; `DESIGN.md:315,323,341`; `EXPERIENCE.md:40,80,191,224,234`). |
| A11Y-02 High - Incomplete 48px coverage | **Resolved** | Universal 48-by-48 rules enumerate every button, link, summary, radio/checkbox label, field, select, row action, Group row, and navigation target, with no link exception (`DESIGN.md:93-96,333-356`; `EXPERIENCE.md:72-95,217`). |
| A11Y-03 High - Preview swap loses focus | **Resolved** | HTMX swaps only derived cells, approval state, and one status node; latest input wins; superseded responses are ignored; focus, caret, selection, keyboard, active row, and page/table scroll are preserved (`EXPERIENCE.md:83,113,161,187,203`). |
| A11Y-04 High - No viable 320px allocation layout | **Resolved** | The semantic table uses a labeled keyboard-focusable horizontal region, sticky wrapping Participant cells, explicit header associations, 48-by-48 controls, and no page-level horizontal scroll, with long-name/OMR/400%-zoom verification (`DESIGN.md:317,344`; `EXPERIENCE.md:83,222,233`). |
| A11Y-05 High - Contradictory swap/history focus | **Resolved** | The bounded focus matrix distinguishes every interaction, uses one forward-response autofocus target, disables private history snapshots, and limits Back/Forward claims to encoded state and native restoration (`EXPERIENCE.md:193-212,227,237`). |
| A11Y-06 Medium - Incomplete derived notice semantics | **Resolved** | Each derived region has one stable polite atomic status, owning `aria-busy`, one transition announcement, and `aria-describedby`; individual amounts are not live regions (`EXPERIENCE.md:77,225`). |
| A11Y-07 Medium - Incomplete error association | **Resolved** | Labels, stable guidance/error IDs, `aria-invalid`, linked focused multi-error summaries, row-specific errors, allocation-wide association, retained non-password values, and enhanced/general status semantics are explicit (`EXPERIENCE.md:81,99-102,162,220,226`). |
| A11Y-08 Medium - All Participants archived gap | **Resolved** | “No Participants exist” and “All Participants archived” have distinct associated guidance and 48-by-48 recovery links (`EXPERIENCE.md:75,151-152`). |
| A11Y-09 Medium - Transaction disclosure ambiguity | **Resolved** | Transaction rows are native `<details>` with named 48-by-48 `<summary>`, separate edit/delete controls, and explicit post-mutation summary focus (`DESIGN.md:346`; `EXPERIENCE.md:85,160,163-164`). |
| A11Y-10 Medium - Confirmation return focus absent | **Resolved** | Server state carries an allow-listed return URL/focus ID; Cancel and success responses autofocus exact invoking/successor targets; lifecycle changes announce once (`EXPERIENCE.md:86,154,164,174,195-212`). |
| A11Y-11 Medium - Noscript replacement undefined | **Resolved by supersession** | HTMX is optional; valid native links/forms complete every task and present server/browser failures, eliminating the unsupported-noscript state (`specs/design.md:61,91`; `EXPERIENCE.md:16,43,80,186,237`). |
| A11Y-12 Medium - Archived semantics vague | **Resolved** | Visible “Archived” is included in or referenced by the identity label; invented ARIA is forbidden; read-only values use definition text or native readonly controls (`EXPERIENCE.md:87,166-167,217`). |
| A11Y-13 Low - Guidance association vague | **Resolved** | Labels use wrapping or `for`/`id`; guidance and errors use stable `aria-describedby`; relevant constraints precede submission and survive swaps (`EXPERIENCE.md:81,99-102,220`). |
| A11Y-14 Low - Working reference regression trap | **Resolved** | Both spines explicitly supersede the references' modal, dimensions, low-contrast rule, and copied implementation CSS (`DESIGN.md:283,366-368`; `EXPERIENCE.md:22,47`). |

## Objective Contrast Verification

WCAG relative-luminance calculations confirm the token claims in `DESIGN.md:294`.

| Foreground | Surface | Ratio | Floor | Result |
|---|---|---:|---:|---|
| Paper `#F5F0E7` | Background `#101113` | 16.64:1 | 4.5:1 | Pass |
| Paper `#F5F0E7` | Strong surface `#202226` | 14.03:1 | 4.5:1 | Pass |
| Muted paper `#AAA59C` | Background `#101113` | 7.71:1 | 4.5:1 | Pass |
| Muted paper `#AAA59C` | Strong surface `#202226` | 6.50:1 | 4.5:1 | Pass |
| Rule `#6D6C69` | Background `#101113` | 3.60:1 | 3:1 | Pass |
| Rule `#6D6C69` | Surface `#181A1D` | 3.32:1 | 3:1 | Pass |
| Rule `#6D6C69` | Strong surface `#202226` | 3.03:1 | 3:1 | Pass, narrowest margin |
| Rule `#6D6C69` | Navigation `#151619` | 3.45:1 | 3:1 | Pass |
| Rule `#6D6C69` | Input `#121315` | 3.54:1 | 3:1 | Pass |
| On-accent `#211C08` | Accent `#F0D36C` | 11.54:1 | 4.5:1 | Pass |
| Accent `#F0D36C` | Strong surface `#202226` | 10.80:1 | 4.5:1 text / 3:1 UI | Pass |
| Warning `#E88467` | Strong surface `#202226` | 6.01:1 | 4.5:1 text / 3:1 UI | Pass |
| Warning text `#F4BAA7` | Strong surface `#202226` | 9.43:1 | 4.5:1 | Pass |
| Success `#A9D6A0` | Strong surface `#202226` | 9.72:1 | 4.5:1 | Pass |
| Focus `#FFFFFF` | Strong surface `#202226` | 15.93:1 | 3:1 | Pass |

The 2px focus offset remains load-bearing: white directly against Accent is only 1.47:1, while the offset leaves the dark surface adjacent and yields at least 15.93:1 (`DESIGN.md:294`).

## Coverage Verification

- **Native fallback:** Every destination, Preview, mutation, validation path, and recovery is a valid native link/form; HTMX and `response-targets` are optional enhancements (`specs/design.md:61,91`; `EXPERIENCE.md:16,43,80-95,186`). Pass.
- **48px targets:** Every control and link, including navigation, Sign out, Group rows, recovery links, allocation labels, disclosures, form actions, and lifecycle actions, is covered without exceptions (`DESIGN.md:333-356`; `EXPERIENCE.md:72-95,217`). Pass.
- **Keyboard operation:** Native controls, `<details>/<summary>`, no hover-only/drag behavior, and no modal containment cover all tasks (`EXPERIENCE.md:72-95,184-191`). Pass.
- **Focus order and visibility:** DOM order is stable across layouts; white 2px offset focus passes contrast; forward focus and history behavior are bounded and implementable (`DESIGN.md:294,311,315-317`; `EXPERIENCE.md:193-218,227,235-237`). Pass.
- **Labels/descriptions:** Fields, disabled actions, table regions/headers, transaction summaries, archived identities, currencies, Balance direction, and Participant colors have visible and programmatic identity (`EXPERIENCE.md:75,78-95,99-102,151-153,166-167,220-223`). Pass.
- **Error states:** Inline, multi-field, allocation-wide, strict request, submission-token conflict, oversized, timeout, rate-limited, unavailable, mutation, transport, and runtime outcomes have retained-value, focus, announcement, and native recovery contracts (`EXPERIENCE.md:20,81,95,99-102,119-126,162,165,172-180,204,220,226`). Pass.
- **Scoped HTMX behavior:** Preview swaps are derived-only/latest-wins and preserve active interaction state; expected error fragments use the pinned official extension and one scoped announced status (`specs/design.md:91`; `EXPERIENCE.md:16,83,95,113,161,179,186-188,225-226`). Pass.
- **Full-page Spending form:** One document scroll owner, no modal semantics, sticky intrinsic two-row action bar, safe-area/dynamic-viewport behavior, sufficient scroll margin, native Preview/review/edit/approve, and duplicate-dispatch protection are explicit (`DESIGN.md:315,323,341,345`; `EXPERIENCE.md:20,80,84,113,122-125,188,224,234`). Pass.
- **Horizontal allocation table:** The named focusable internal region, sticky Participant column, long-name wrapping, explicit associations, full-size controls, and no page horizontal scroll satisfy 320px reflow (`DESIGN.md:317,344`; `EXPERIENCE.md:83,222,233`). Pass.
- **Group notices and rate states:** Updating, stale, provisional, Ready, and Unavailable are explicit, announced once, and preserve Source Currency/no-partial-result behavior (`EXPERIENCE.md:55-60,77,128-138,171-173,225`). Pass.
- **No-participant and empty states:** No Participants, all archived, empty active/archived Groups, empty month, empty ledger, and empty Transactions retain correct recovery/action context (`EXPERIENCE.md:147-159`). Pass.
- **Destructive confirmations:** Named scope, reversibility, protected submit, single-use token, cancellation focus, successor focus, and one committed announcement are explicit (`EXPERIENCE.md:20,86,154-156,164,168-176,188,190,206-211`). Pass.
- **Noscript state:** No special unsupported state is required because native operation is complete without HTMX (`EXPERIENCE.md:16,80,88,95,186,237`). Pass.
- **Color-only distinctions:** Selection uses native checked/current state; warnings use words; Balance uses sign/position words; transfers use from/to; archive uses text; Participant swatches are supplemental (`DESIGN.md:290-294,335,343,348,352,354-356,365`; `EXPERIENCE.md:77-95,132-138,166-167,217,221-225`). Pass.
- **Selected references:** Variant C still contains obsolete `#3B3A37`, undersized controls, and modal markup (`.working/directions-dark-4.html:64-75,109-120,166,183`), while Variant B contains obsolete compact form choices (`.working/flow-group-shell-2026-08-10.excalidraw:44-77`). Their authority is explicitly limited to identity and compact-shell direction (`DESIGN.md:283`; `EXPERIENCE.md:22,47`). Pass as historical references only.

## Implementation Acceptance Tests

1. At an effective 320 CSS pixels and at 400% zoom, verify every control/link is at least 48 by 48, navigation and action labels wrap, the shell never overlays content, and only the named allocation region scrolls horizontally.
2. Complete every core task with HTMX and its extension disabled, including Preview, correction, approval, navigation, Debt mode, confirmations, restore, and Sign out.
3. Race enhanced allocation previews under keyboard and screen-reader operation; verify latest-result wins and focus, caret, software keyboard, active row, values, and page/table scroll remain unchanged.
4. Test every focus-matrix row in current Chrome, Firefox, Safari, and Edge, including one autofocus target per forward response and no deterministic prior-focus claim on Back/Forward.
5. Trigger `422`, `409`, `429`, `503`, strict-request, submission-token, oversized, timeout, transport, and runtime failures in native and enhanced paths; verify exact announcement, focus, recovery, retained safe values, and cleared pending state.
6. Verify single, multiple, row-specific, and allocation-wide errors link to exact controls while retaining guidance and non-password values.
7. Verify Updating, stale, provisional, Ready, and Unavailable each announce once, set correct busy state, remain text-distinct, and never expose partial Debts results.
8. Verify no Participants, all Participants archived, all empty collections, archived/read-only identities, and every destructive confirmation/recovery path.
9. Automate contrast checks, retaining explicit coverage for Rule on Strong surface and offset Focus around Accent controls.

## Counts

| Severity | Count |
|---|---:|
| Critical | 0 |
| High | 0 |
| Medium | 0 |
| Low | 0 |
| **Total** | **0** |
