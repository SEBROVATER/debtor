# Adversarial Divergence Review - Round 2

## Verdict

**CHANGES REQUIRED - the first-pass failures are substantially closed, but five cross-epic composition gaps remain.** Canonical money, the logical `Spending` aggregate, quote polarity, primary rate-mode selection, monthly conversion order, signed quantization polarity, settlement direction/queue transitions, top-level error categories, composite mutation dispatch, and trusted-chain resolution now have binding answers. The remaining findings below are limited to cases where two independently implemented epics can obey every adopted decision and still disagree on application-visible financial availability, summary values, failure behavior, or security state.

## Method

Each prior divergence was re-tested against the current text of AD-3, AD-5, AD-8 through AD-10, AD-12 through AD-15, the consistency conventions, and the normative `specs/design.md`. A finding is retained only when both hypothetical epics can satisfy every literal AD without choosing a forbidden physical table, route, template, cache representation, or source-code shape. Product and UX omissions that do not break epic composition are excluded.

## Remaining Findings

### 1. HIGH - Refreshable stale fallback has no interoperable context-matching rule

**Evidence:** AD-9 defines request identity as `(source, target, requested_date, calculation_date)`, requires current/future contexts to refresh on UTC rollover, requires stale fallback to match context, and permits current/future quotes through seven UTC calendar days after the "effective fetch date." It does not state whether stale matching includes the prior `calculation_date`, nor does it distinguish the provider-returned effective quote date from the local date on which a quote was fetched.

**Epic A:** The rate-cache epic treats the four-field request identity as the exact stale-fallback key. At UTC rollover, yesterday's current/future quote cannot match today's request because `calculation_date` changed. The seven-day rule only applies to repeated failures within one calculation date, with age measured from provider `effective_date`.

**Epic B:** The debt epic treats rollover as requiring a refresh attempt but permits fallback to the latest prior refreshable quote for the same source, target, and requested-date class across calculation dates. It measures seven days from locally captured fetch date because a provider effective date is quote metadata, not a fetch timestamp.

Both preserve request identity, rollover refresh, context-matching fallback, the seven-calendar-day bound, immutable per-calculation bundles, and deterministic caches. They disagree on whether a debt, monthly conversion, or archival attempt remains available after a rollover/provider failure, especially around weekends when provider effective date and fetch date differ.

**Required closure:** Define a separate stale-lookup identity for refreshable contexts, state whether a prior calculation date may match after the mandatory rollover refresh fails, identify the date from which the seven-day window is counted, and define the inclusive boundary. This is semantic key behavior, not cache representation.

### 2. HIGH - Same-currency conversion is not assigned a provider-independent identity rule

**Evidence:** AD-9 defines quote direction and request contexts but never states whether `source == target` produces a domain identity quote of exactly `1` or must pass through the exchange-rate provider and quote bundle.

**Epic A:** The spending-summary epic converts group-currency spendings with a synthetic exact identity quote, creates no provider request, and cannot mark that conversion stale or unavailable.

**Epic B:** The rate epic creates a normal `(currency, currency, requested_date, calculation_date)` request and requires a returned or eligible stale quote. A provider rejection can therefore make debts or the entire converted monthly section unavailable.

Both multiply source values by a target-per-source quote, use exact `Decimal`, honor immutable ordered contexts, and apply the all-or-unavailable summary rule. They disagree on provider calls, quote disclosures, warning sets, calculation availability, and archival eligibility for a ledger that needs no real FX conversion.

**Required closure:** Bind same-currency conversion to an exact provider-independent identity quote and state whether it is omitted from requested/returned quote disclosures, or explicitly require provider treatment. No concrete provider-port type is needed.

### 3. HIGH - Converted monthly totals have no final target-minor-unit rule

**Evidence:** AD-9 now fixes the current UTC month, source grouping, per-spending Historical conversion, exact target aggregation, and atomic unavailability. It does not say whether derived group/per-payer target totals retain all exact product scale or are quantized for monetary presentation, and if quantized, which signed/unsigned rounding rule applies. AD-3 rejects excess precision on input and hydration but does not impose a derived-summary quantizer.

**Epic A:** The summary use case returns exact aggregate products, so `10 EUR * 1.08333 USD/EUR` contributes canonical `10.8333` to a USD converted total.

**Epic B:** The summary use case treats the result as USD money and quantizes the final aggregate to two minor units, using truncation or a conventional rounding rule selected by that epic.

Both convert each spending without intermediate rounding, aggregate exactly in Rust, avoid SQL arithmetic, and preserve source totals. Their application-facing converted amounts differ, so summary and rendering epics cannot agree on the value or accepted scale.

**Required closure:** State whether converted summary totals are exact analytical decimals or target-currency money. If they are money, define one final aggregate quantization rule and when it is applied; do not prescribe a rendering or Rust struct shape.

### 4. HIGH - The exhaustive error categories still overlap semantically

**Evidence:** AD-15 enumerates `Validation`, `NotFound`, `Conflict`, `Unavailable`, `Storage`, `Configuration`, and `Calculation`, but gives no disjoint category definitions or required HTTP/retry semantics beyond selected conventions for validation and lifecycle/submission conflicts. AD-8 calls epoch or quote-eligibility mismatch "retryable" without assigning a category. AD-6 gate timeout, SQLite busy/lock exhaustion, malformed hydrated money, and provider exhaustion are likewise not uniquely classified.

**Epic A:** Persistence reports gate timeout and SQLite busy as `Storage`; archival epoch mismatch as `Conflict`; corrupt canonical money as `Storage`; and absent eligible rates as `Unavailable`.

**Epic B:** Application reports both bounded storage contention and epoch mismatch as retryable `Unavailable`; maps invalid hydrated financial state to `Calculation`; and reserves `Storage` for non-retryable adapter failures.

Both use only the exhaustive safe taxonomy, bounded operation-specific reasons, sanitized exhaustive web mapping, and no raw adapter diagnostics. Yet a repository epic and web epic can disagree on status, retryability, token-result messaging, and whether an operation is safely repeatable.

**Required closure:** Define category boundaries and the category plus retryability/HTTP class for shared cross-epic failures. Operation-specific reason enum layout and adapter diagnostics may remain implementation-owned.

### 5. HIGH - Login-attempt accounting has no shared transition point

**Evidence:** AD-10 fixes CSRF before password verification and places authentication/password admission before route parsing and token reservation. AD-14 permits five attempts per trusted client IP in a rolling five-minute window but never defines which admitted requests increment the limiter or whether a successful verification clears prior attempts.

**Epic A:** The login web epic increments once immediately before every password verification, counting correct and incorrect passwords after valid CSRF, and retains the rolling history after success.

**Epic B:** The authentication epic increments only failed password verifications, does not count a correct password, and clears the key after successful promotion.

Both reject CSRF failures before password work, use the one trusted-client resolver, enforce at most five attempts in a rolling five minutes, keep the 4,096-key bound, and fail closed at capacity. Combined with independently written limiter and login epics, they disagree on the limiter API/state transition and can allow different numbers of guesses after successes or repeated authenticated-session creation.

**Required closure:** Define exactly when one attempt is recorded, which pre-verification failures are excluded, whether a correct verification consumes an attempt, and whether success clears or retains the rolling history. This does not require form or limiter data-structure detail.

## Closed Prior Findings

| Prior area | Round-2 result |
| --- | --- |
| Canonical money | Closed by domain-owned parser/formatter grammar, normalized scale, canonical `TEXT`, bounds, precision map, and input/hydration enforcement in AD-3. |
| Aggregate shape | Closed at the required logical altitude by domain `Spending` ownership and complete semantic cardinality/role fields in AD-5; persistence decomposition remains correctly deferred. |
| Quote direction and request context | Quote polarity and request-versus-returned metadata are closed; refreshable stale matching remains open as Finding 1. |
| Rate modes and staleness | Historical, Current, future/provisional, rollover, cache classes, and broad stale windows are closed; cross-rollover stale identity/date semantics remain open as Finding 1. |
| Monthly conversion | Inclusion, grouping, per-spending Historical conversion, exact aggregation order, and atomic degradation are closed; final derived scale remains open as Finding 3. |
| Signed quantization | Closed by truncation toward zero, signed remainder ordering, ascending participant-ID ties, and exact zero-sum preservation in AD-9. |
| Settlement polarity and algorithm | Closed by explicit balance signs, separate queues, amount/ID ordering, debtor-to-creditor direction, head advancement without re-sort, preserved generation order, and postconditions. |
| Error taxonomy | Top-level ownership and exhaustive variants are closed; disjoint semantics remain open as Finding 4. |
| Dispatch point | Closed for composite business prechecks by reservation before exactly one application mutation dispatch and archival snapshot/rate work inside that dispatched use case. |
| Trusted client | Resolver ownership, peer trust, selected format, right-to-left chain walk, malformed-input rejection, fallback, and universal consumption are closed; limiter attempt transitions remain open as Finding 5. |

## Required Closure Standard

The spine is composition-safe for these reviewed areas when one fixed ledger snapshot, calculation context, quote observation history, and unsafe-login request sequence yield the same calculation availability, converted summary amount, safe failure class, and limiter state regardless of which epic supplied the implementation. The required additions are semantic contracts only; they do not require product-flow expansion, route/template inventory, table/index design, concrete cache structures, or source-owned type layouts.
