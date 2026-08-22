---
title: Debtor Visual Design Contract
name: Debtor
description: A dark editorial ledger for exact, dependable shared-expense accounting.
status: final
created: 2026-08-10
updated: 2026-08-10
sources:
  - /home/sebr/projects/pet/debtor/_bmad-output/planning-artifacts/prds/prd-debtor-2026-08-10/prd.md
  - /home/sebr/projects/pet/debtor/_bmad-output/planning-artifacts/prds/prd-debtor-2026-08-10/addendum.md
colors:
  background: '#101113'
  surface: '#181A1D'
  surface-strong: '#202226'
  navigation: '#151619'
  text: '#F5F0E7'
  text-muted: '#AAA59C'
  rule: '#6D6C69'
  accent: '#F0D36C'
  on-accent: '#211C08'
  warning: '#E88467'
  warning-text: '#F4BAA7'
  success: '#A9D6A0'
  input: '#121315'
  shadow: '#000000'
  focus: '#FFFFFF'
typography:
  body:
    fontFamily: 'system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif'
    fontSize: 16px
    fontWeight: '400'
  group-title:
    fontFamily: 'Georgia, "Times New Roman", serif'
    fontSize: 1.1rem
    fontWeight: '500'
  section-title:
    fontFamily: 'Georgia, "Times New Roman", serif'
    fontSize: 1.55rem
    fontWeight: '500'
    letterSpacing: -0.035em
  amount:
    fontFamily: 'Georgia, "Times New Roman", serif'
    fontSize: 2.15rem
    fontWeight: '500'
    letterSpacing: -0.045em
  form-title:
    fontFamily: 'Georgia, "Times New Roman", serif'
    fontSize: 1.18rem
    fontWeight: '500'
  eyebrow:
    fontFamily: 'system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif'
    fontSize: 0.68rem
    fontWeight: '750'
    letterSpacing: 0.12em
  label:
    fontFamily: 'system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif'
    fontSize: 0.66rem
    fontWeight: '700'
  row:
    fontFamily: 'system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif'
    fontSize: 0.78rem
    fontWeight: '400'
  table:
    fontFamily: 'system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif'
    fontSize: 0.7rem
    fontWeight: '400'
  meta:
    fontFamily: 'system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif'
    fontSize: 0.75rem
    fontWeight: '400'
  navigation:
    fontFamily: 'system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif'
    fontSize: 0.6rem
    fontWeight: '700'
rounded:
  none: 0px
  full: 9999px
spacing:
  '1': 4px
  '2': 5px
  '3': 7px
  '4': 8px
  '5': 9px
  '6': 10px
  '7': 12px
  '8': 14px
  '9': 16px
  '10': 18px
  '11': 20px
  '12': 30px
  '13': 34px
components:
  interactive-target:
    min-block-size: 48px
    min-inline-size: 48px
    focus: '{colors.focus}'
  page-header:
    background: '{colors.background}'
    foreground: '{colors.text}'
    rule: '4px double {colors.rule}'
    padding-mobile: '20px 20px 14px'
    padding-wide: '20px 30px 14px'
    focus: '{colors.focus}'
  group-navigation:
    background: 'rgba(21, 22, 25, 0.96)'
    foreground: '{colors.text-muted}'
    active: '{colors.accent}'
    rule: '1px solid {colors.rule}'
    columns: 'repeat(5, 1fr)'
    backdrop-filter: 'blur(14px)'
    focus: '{colors.focus}'
    min-block-size: 48px
    min-inline-size: 48px
  add-spending-action:
    background: '{colors.accent}'
    foreground: '{colors.on-accent}'
    radius: '{rounded.none}'
    min-height: 48px
    min-width: 48px
    shadow: '5px 5px 0 {colors.shadow}'
    focus: '{colors.focus}'
    disabled-background: '{colors.input}'
    disabled-foreground: '{colors.text-muted}'
    disabled-rule: '1px solid {colors.rule}'
  ledger-section:
    background: transparent
    foreground: '{colors.text}'
    rule: '1px solid {colors.rule}'
    radius: '{rounded.none}'
    padding: '18px 0'
  conversion-notice:
    background: transparent
    foreground: '{colors.warning-text}'
    emphasis: '{colors.warning}'
    rule: '1px solid {colors.warning}'
    radius: '{rounded.none}'
  money-row:
    foreground: '{colors.text}'
    secondary: '{colors.text-muted}'
    rule: '1px solid {colors.rule}'
  participant-marker:
    radius: '{rounded.full}'
    size: 9px
    outline: '1px solid {colors.rule}'
  spending-form:
    background: '{colors.background}'
    foreground: '{colors.text}'
    rule: '1px solid {colors.rule}'
    accent-rule: '5px solid {colors.accent}'
    radius: '{rounded.none}'
    focus: '{colors.focus}'
  field:
    background: '{colors.input}'
    foreground: '{colors.text}'
    rule: '1px solid {colors.rule}'
    radius: '{rounded.none}'
    min-height: 48px
    min-width: 48px
    focus: '{colors.focus}'
    error: '{colors.warning}'
  share-mode-control:
    background: transparent
    foreground: '{colors.text-muted}'
    selected: '{colors.accent}'
    rule: '1px solid {colors.rule}'
    radius: '{rounded.none}'
    min-height: 48px
    min-width: 48px
    focus: '{colors.focus}'
  allocation-table:
    foreground: '{colors.text}'
    secondary: '{colors.text-muted}'
    rule: '1px solid {colors.rule}'
    success: '{colors.success}'
    sticky-background: '{colors.background}'
    focus: '{colors.focus}'
    min-control-block-size: 48px
    min-control-inline-size: 48px
  form-action-bar:
    background: '{colors.surface-strong}'
    foreground: '{colors.text}'
    rule: '1px solid {colors.rule}'
    min-block-size: 48px
    focus: '{colors.focus}'
    rows: 'auto auto'
    action-columns: 'repeat(3, minmax(0, 1fr))'
    gap: 8px
    min-action-block-size: 48px
    min-action-inline-size: 48px
    control-scroll-margin: 'calc(12rem + env(safe-area-inset-bottom))'
  transaction-row:
    background: transparent
    foreground: '{colors.text}'
    secondary: '{colors.text-muted}'
    rule: '1px solid {colors.rule}'
    min-summary-block-size: 58px
    min-summary-inline-size: 48px
  confirmation-page:
    background: '{colors.background}'
    foreground: '{colors.text}'
    emphasis: '{colors.warning}'
    radius: '{rounded.none}'
    rule: '1px solid {colors.rule}'
    focus: '{colors.focus}'
    min-action-block-size: 48px
    min-action-inline-size: 48px
  archived-view:
    background: '{colors.background}'
    foreground: '{colors.text-muted}'
    rule: '1px solid {colors.rule}'
    focus: '{colors.focus}'
    min-action-block-size: 48px
    min-action-inline-size: 48px
  access-form:
    background: '{colors.background}'
    foreground: '{colors.text}'
    error: '{colors.warning-text}'
    rule: '1px solid {colors.rule}'
    focus: '{colors.focus}'
    min-action-block-size: 48px
    min-action-inline-size: 48px
  group-list:
    background: transparent
    foreground: '{colors.text}'
    secondary: '{colors.text-muted}'
    rule: '1px solid {colors.rule}'
    focus: '{colors.focus}'
    min-row-block-size: 48px
    min-row-inline-size: 48px
  management-form:
    background: transparent
    foreground: '{colors.text}'
    secondary: '{colors.text-muted}'
    rule: '1px solid {colors.rule}'
    min-action-block-size: 48px
    min-action-inline-size: 48px
  participant-color-control:
    background: '{colors.input}'
    foreground: '{colors.text}'
    rule: '1px solid {colors.rule}'
    focus: '{colors.focus}'
    radius: '{rounded.none}'
    min-block-size: 48px
    min-inline-size: 48px
  pagination:
    background: transparent
    foreground: '{colors.text}'
    disabled: '{colors.text-muted}'
    rule: '1px solid {colors.rule}'
    focus: '{colors.focus}'
    min-height: 48px
    min-width: 48px
  debt-mode-control:
    background: transparent
    foreground: '{colors.text-muted}'
    selected: '{colors.accent}'
    rule: '1px solid {colors.rule}'
    focus: '{colors.focus}'
    min-height: 48px
    min-width: 48px
  financial-results:
    background: transparent
    foreground: '{colors.text}'
    secondary: '{colors.text-muted}'
    rule: '1px solid {colors.rule}'
    warning: '{colors.warning-text}'
  request-status:
    background: transparent
    foreground: '{colors.text}'
    emphasis: '{colors.warning-text}'
    rule: '1px solid {colors.warning}'
    radius: '{rounded.none}'
---

# Debtor Visual Design Contract

## Brand & Style

Debtor is **Editorial Contrast**: a composed, publication-like ledger that gives exact figures the authority of a financial record without becoming a dashboard. Warm paper-toned type sits on charcoal. Typographic scale, double rules, hairlines, and deliberate whitespace establish hierarchy; card-heavy chrome does not. The persistent yellow action color is unmistakable but never decorative.

Visual decisions reinforce a deliberately small, self-operated product. A new destination, persistent control, preference, or derived view must serve a confirmed Administrator job rather than imitate a competitor or make the product look more complete. The same sober ledger language must fit an ongoing personal Group as naturally as a travel Group.

The product is dark-only. There is no light mode or appearance toggle. Variant C, Editorial Contrast, is authoritative only for visual identity. Editorial Contrast's earlier modal, low-contrast rule, undersized controls, and copied CSS are superseded by [Layout & Spacing](#layout--spacing) and [Components](#components).

## Colors

- **Charcoal (`{colors.background}`)** is the page canvas. **Editorial surface (`{colors.surface}`)** and **strong surface (`{colors.surface-strong}`)** are reserved for literal regions such as the form action bar, rather than card decoration.
- **Paper (`{colors.text}`)** is primary text. **Muted paper (`{colors.text-muted}`)** is metadata, labels, inactive navigation, and secondary amounts; never use it where loss of contrast would hide a financial fact.
- **Rule (`{colors.rule}`)** supplies hairlines, component boundaries, and the page header's double rule. It is the smallest same-hue lift from the selected warm gray that reaches 3:1 against every active dark surface; rules, not filled cards, carry structure.
- **Action yellow (`{colors.accent}`)** with `{colors.on-accent}` marks primary actions, current Group destination, selected Share mode, and the Spending form's top edge. It is not a category color or a generic status badge.
- **Coral (`{colors.warning}`)** and `{colors.warning-text}` identify stale, provisional, unavailable, destructive, and other attention states only when the accompanying words state the condition. **Sage (`{colors.success}`)** confirms an exact allocation; it never substitutes for the amount or message.
- Participant markers use each Participant's stored color. They are supplemental identity cues, never the only way to identify a Participant or encode Balance direction.

Load-bearing combinations meet the product floor: normal text at 4.5:1, large text and component boundaries at 3:1. `{colors.rule}` reaches 3.60:1 on `{colors.background}`, 3.32:1 on `{colors.surface}`, 3.03:1 on `{colors.surface-strong}`, 3.45:1 on `{colors.navigation}`, and 3.54:1 on `{colors.input}`. `{colors.text}` and `{colors.text-muted}` exceed 4.5:1 on those surfaces; `{colors.on-accent}` on `{colors.accent}` exceeds 3:1. `{colors.warning}` boundaries exceed 3:1 on every active surface. Focus uses `{colors.focus}` with a two-pixel offset that leaves the active dark surface between the control and outline; it exceeds 3:1 against every such surface.

## Typography

Typography supplies the editorial contrast. Group titles, section titles, form titles, and major amounts use the Georgia stack. Everything operational uses the system sans stack.

- `{typography.amount}` is reserved for primary totals. Monetary rows use tabular numerals even when their labels remain proportional.
- `{typography.section-title}` leads each compact Group section; `{typography.group-title}` anchors the shell header.
- `{typography.eyebrow}` and `{typography.label}` are uppercase where used as category labels, with the exact tracked spacing in the tokens. Do not set messages or financial values in all caps.
- `{typography.row}`, `{typography.table}`, and `{typography.meta}` keep dense ledger material readable without competing with totals.

Money always displays a currency symbol plus ISO code; do not rely on symbol alone. Spending dates display as ISO `YYYY-MM-DD` outside forms. Preserve exact decimal digits required by the currency and use tabular numerals for comparable columns.

## Layout & Spacing

One responsive composition serves phone and desktop. The Group shell is compact and section-based, never a long anchored page. Content uses 18px horizontal padding on phone and 30px when wide; the page header uses 20px on phone and 30px when wide.

On narrow screens the page is a `100dvh` grid with header, one scrolling main region, and an intrinsically sized bottom shell. The shell paints after but never overlays main content; it owns Add Spending, conditional setup guidance, and five-destination navigation, while `env(safe-area-inset-bottom)` pads its lower edge. Navigation labels may wrap to two lines without truncation. Every shell control is at least 48 by 48 CSS pixels, focus outlines stay inside the measured stack, and content never sits behind the shell. At wide composition widths, the same DOM preserves the destination order as the shell moves into centered normal flow rather than consuming the bottom edge.

Summary heading pairs the month title left with `YYYY-MM · UTC` right. Summary material is one column on narrow screens. When the viewport can hold the rendered wide composition without compression, it becomes a `1.25fr / 0.75fr` editorial grid with `{spacing.13}` between columns. The wide reading measure and bottom-shell measure are both `min(100%, 1040px)`; Group navigation occupies at most 680px on the left, and Add Spending aligns right in the same shell row. [Group Summary mock](mockups/group-summary.html) illustrates the one-column phone Summary, wide two-column adaptation, per-currency source blocks, and shared shell placement. The maximum-width breakpoint is chosen by content fit rather than by creating a separate desktop experience.

The focused **Spending form** is a full page with a single document scroll owner and `min-height: 100dvh`. Fields use a two-column grid with `{spacing.6}` gaps; Description spans both columns, and at 350px or narrower all fields stack in source order. The header keeps Group eyebrow and form title left with the current Edit/Preview state right. The sticky **Form action bar** is in document flow, padded by `env(safe-area-inset-bottom)`, and remains at the visible dynamic-viewport edge when the software keyboard opens. At 320px its Total/status row sits above three equal action columns. Focused fields and allocation controls use `{components.form-action-bar.control-scroll-margin}` so browser scrolling clears the bar at its maximum wrapped height. [Add Spending mock](mockups/add-spending.html) illustrates reviewed and validation-error states at 320px, including keyboard-shortened viewport behavior.

At 320px the **Allocation table** remains a horizontal semantic table inside its own clearly bordered, labeled, keyboard-focusable scroll region. Its intrinsic table width is 520px: Participant is 116px and sticky at inline start, Payer and Included are 76px each, Weight is 92px, and Share is 160px. Participant uses `{colors.background}` with a right `{colors.rule}` boundary; controls remain 48 by 48. Names wrap/break without clipping, Share values stay right-aligned and unbroken, and the page never scrolls horizontally. Verify with a 100-character Participant name, maximum OMR Total, enlarged text, and 400% zoom.

Manage is a vertical editorial form, not a settings dashboard: Group settings first, Participant roster second, Group lifecycle last, each separated by `{colors.rule}` and a serif section heading. Preferred phone grids use flexible name plus 116px Group Currency, and flexible Participant name plus 124px color; the color control itself reserves 48px for its outlined swatch. These grids stack in source order before any 48px target or text collides. Participant blocks order identity, fields, Historical Balance, eligibility copy, then two equal actions; Add Participant is separated by a top rule. [Manage mock](mockups/manage.html) illustrates eligible, ineligible, and rate-unavailable Participant archive presentation plus Group lifecycle ordering. Sign in centers one narrow **Access form** in the reading column. Groups places create-by-name above the **Group list**, with the contextual archived link adjacent to the list heading. Financial results stack Group totals before per-Payer rows; Debts stacks mode, Balances, Settlement Transfers, then calculation disclosure. [Debts mock](mockups/debts.html) illustrates complete and unavailable all-time results. [Transactions mock](mockups/transactions.html) illustrates collapsed/expanded history rows, paired row actions, and pagination context.

## Elevation & Depth

Hierarchy comes from tone, rules, type, and placement. Cards do not float. The full-page **Spending form** uses the page canvas and a five-pixel `{colors.accent}` top rule; it has no scrim, modal layer, or sheet shadow. The persistent **Add Spending action** alone carries the hard `5px 5px` shadow established by Editorial Contrast. Do not add gradients, ambient card shadows, hover lift, or decorative depth.

## Shapes

The application is square-edged. Fields, sections, buttons, navigation, notices, full-page forms, and confirmation surfaces use `{rounded.none}`. The only round shape is the small **Participant marker**, using `{rounded.full}` because it is a color swatch rather than a container. Do not introduce pills or soft cards.

## Components

| Component | Visual contract |
|---|---|
| **Interactive target** | Every button, link, summary, radio/checkbox label, input, select, row action, and disclosure has a rendered pointer target at least 48 by 48 CSS pixels at 320px and 400% zoom. There are no inline-link exceptions. |
| **Page header** | `{components.page-header.background}` with Group eyebrow and serif title; separated by the four-pixel double `{colors.rule}` rule. Sign out is muted, textual, in normal flow, and a full **Interactive target**. |
| **Group navigation** | Five equal persistent **Interactive target** destinations over `{components.group-navigation.background}` with a top rule and 14px backdrop blur. Labels may wrap to two lines without truncation. Inactive labels use `{colors.text-muted}`; `aria-current` uses `{colors.accent}`. The 16px square mark rotates 45 degrees; text remains the identifying label. |
| **Add Spending action** | Thumb-reachable, at least 48px high, square, bold, `{colors.accent}` on `{colors.on-accent}`, with the sole hard shadow. Disabled uses `{colors.input}`, `{colors.text-muted}`, and a `{colors.rule}` boundary, never opacity alone; its setup guidance is adjacent muted text above the persistent shell with a textual Manage link. |
| **Ledger section** | Transparent, edge-to-edge within content, 18px vertical padding, bottom hairline; the first section also has a top hairline. Major total uses `{typography.amount}`. |
| **Conversion notice** | One concise Group-level ruled notice, with coral emphasis and explicit text. Use one-pixel top and bottom rules, no filled alert card, no icon-only severity. |
| **Money row** | Two-column label/value rhythm with a top hairline. Values align right and use tabular numerals; secondary context uses `{colors.text-muted}`. |
| **Participant marker** | 9px stored-color circle beside the Participant name with a `{colors.rule}` outline so any valid stored color retains a visible boundary. Never replace the visible name or encode a debt state by color alone. |
| **Spending form** | Focused full-page composition with `{colors.background}`, five-pixel `{colors.accent}` top rule, serif title, one reading-order column on narrow screens, and no modal/sheet treatment. On wide screens it remains a centered full page rather than becoming a side sheet. |
| **Field** | `{colors.input}` fill, one-pixel `{colors.rule}` border, square edge, visible label, and at least 48px interactive height. Focus adds a minimum two-pixel high-contrast outline without moving layout. |
| **Share mode control** | Two adjacent text controls for Proportional and Exact. Selected mode uses `{colors.accent}` for text and border; selection remains programmatically exposed. Each target is at least 48px high. |
| **Allocation table** | Horizontal semantic table in a labeled, focusable internal scroll region with a visible `{colors.rule}` boundary, `{colors.focus}` focus outline, and no page-level horizontal scroll. Participant color marker and name stay sticky at inline start on `{colors.background}`; long names wrap/break without clipping. Payer and Included controls, and either Weight or Share controls, are each **Interactive targets**. Derived cells and one status node are the only HTMX swap visuals. |
| **Form action bar** | Sticky in the full-page form's document flow with safe-area padding and an intrinsic two-row grid. The first row spans all columns and shows Total plus concise preview/pending/error status; amount and status wrap without truncation or overlap. The second row has three equal `minmax(0, 1fr)` columns with `{spacing.4}` gaps: Cancel; Preview or Edit allocation; and Approve. Every action is at least 48 by 48; labels wrap without clipping. The amount never appears inside Approve. Approve uses `{colors.accent}` / `{colors.on-accent}`; pending/stale uses the disabled palette. Fields and table controls use `{components.form-action-bar.control-scroll-margin}` so maximum OMR Total, 320px, 400% zoom, keyboard, and safe-area growth do not obscure focus. |
| **Transaction row** | Transparent `<details>` ledger row with hairline separation. Summary is a two-column grid: disclosure sign plus Description and date at left, unbroken Total at right. Expanded detail uses a two-column definition list with 108px labels and flexible values; Edit/Delete are equal columns with `{spacing.4}` gap. Its `<summary>` and actions are full **Interactive targets**. |
| **Confirmation page** | Full server-rendered page, not a modal. Uses normal editorial hierarchy, explicit object details, a coral destructive action, and a non-destructive return action. |
| **Archived view** | Contextual list uses muted labels and rules, but names and historical financial facts retain normal readable contrast. Read-only Group state is stated in text; restoration actions use the standard action hierarchy. |
| **Access form** | One narrow, rule-led sign-in composition with serif heading, one password Field, form-level status below the heading, and one full-width primary submit. Limiter, capacity, timeout, and oversized-input messages use text plus `{colors.warning-text}`, never a filled card. |
| **Group list** | Edge-to-edge editorial rows separated by `{colors.rule}`. Name is primary; Group Currency and active Participant count share one muted metadata line. Empty copy and create-by-name form remain in the same reading column; the archived link is textual, not a badge. |
| **Management form** | Three ordered ruled sections: Group settings, Participants, Group lifecycle. Section headings pair the title left with contextual archived links right. Participant blocks use 16px separation; name/color fields precede Balance and eligibility, then equal Save/Archive actions. Add Participant and each lifecycle item begin after a rule. Standard actions remain yellow/textual; archive uses coral text and rule; irreversible delete is visually last. |
| **Participant color control** | Labeled `#RRGGBB` text Field with a named swatch preview outlined by `{colors.rule}`. The visible normalized value remains text; the swatch is supplementary. Invalid submitted text remains visible with its inline error. |
| **Pagination** | Previous and Next are equal 48px outlined controls below history, with current-page context as text between or above them. Disabled endpoints retain readable text and boundary; focus uses `{colors.focus}`. No infinite-scroll affordance. |
| **Debt mode control** | Historical and Current are two adjacent 48px controls using the **Share mode control** visual language. `{colors.accent}` identifies the selected mode in addition to native checked state; focus uses `{colors.focus}`. |
| **Financial results** | Summary groups each Source Currency as a bold Group-total row followed by indented per-Payer **Money row** entries; for Group Currency, show the major serif total, followed by per-Payer rows and then the calculation note. Debts uses ruled Balances, Settlement Transfers, then a two-column definition disclosure. Unique rates span both columns; each rate places context left and equation right, with stale context on its own line. Unavailable Debts uses one ruled no-partial block before attempted-calculation context. |
| **Request status** | Stable scoped region with explicit pending, failure, and committed-state text. Routine updates use `{colors.text-muted}`; failures use `{colors.warning-text}` and `{colors.warning}` rules; committed lifecycle status uses `{colors.success}`. It never overlays controls, relies on animation, or requires imperative post-swap behavior. |

## Do's and Don'ts

| Do | Don't |
|---|---|
| Use Editorial Contrast exactly: charcoal, warm paper, serif financial hierarchy, rules, square edges, yellow actions. | Blend in mint ledger, lavender soft-card, or blue instrument-panel treatments from rejected variants. |
| Use rules and whitespace to group ledger material. | Wrap every summary in a filled or elevated card. |
| Keep Add Spending and Group destinations visually persistent and thumb-reachable. | Put an important destination only in a top-left control or transient menu. |
| Pair every status color with concise status text. | Encode stale, provisional, unavailable, archived, Balance, or Participant identity through color alone. |
| Keep every interactive target at least 48 by 48 CSS pixels and focus outlines at least two CSS pixels thick. | Copy the smaller control dimensions or modal overlay from the selected working artifact. |
| Use `{colors.rule}` for every required component boundary and `{colors.focus}` for focus. | Restore the original low-contrast `#3B3A37` on controls, rows, or meaningful separators. |
| Keep HTMX-enhanced fragments visually identical to native full-page responses. | Make a core action depend on HTMX or hide its native link or form destination. |
| Use immediate native state changes only. | Add CSS transitions, custom-JavaScript animation, celebratory effects, gradients, or hover lift. |
