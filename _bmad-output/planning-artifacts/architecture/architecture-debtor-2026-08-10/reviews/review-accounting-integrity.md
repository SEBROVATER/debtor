# Accounting And Data-Integrity Review

## Review Basis

- Reviewed artifact: `ARCHITECTURE-SPINE.md` dated 2026-08-10.
- Normative contract: `specs/design.md`.
- Accepted decisions: `specs/adr/0001-foundation-architecture.md` and `specs/adr/0002-long-term-foundation-hardening.md`.
- Focus: exact money, ownership and history, spending modes and cardinality, transaction boundaries, snapshot consistency, archival race safety, rate context/staleness/cache behavior, deterministic quantization and settlement, and summary degradation.

## Verdict

**CHANGES REQUIRED - accounting integrity is not yet implementation-deterministic.**

The spine preserves the intended boundaries and states most headline invariants, but it is not yet a sufficient contract for independently implemented epics. In particular, AD-9 compresses materially different rate, conversion, quantization, and settlement rules into assertions without defining their executable semantics. AD-8 protects archival from database writes but not from the passage of UTC time or changing external quote observations. Several transaction and cardinality rules are also left inferential. Two implementations can conform to the current wording and still accept different ledgers, archive a participant under different conditions, or return different balances, transfers, and converted summaries.

## Findings

### AI-01 - Critical - Rate mode, effective-date, refresh, and stale eligibility rules are absent

**Spine references:** AD-9, lines 128-132; Deferred, line 251.  
**Normative references:** `specs/design.md` lines 75-80 and 117; ADR 0001 section 8.

AD-9 says that contexts contain source, target, requested date, and effective date, but it does not bind the rules that produce those values or select a usable quote. It omits all of the following normative behavior:

- Historical is the default mode and requests each spending date.
- Current mode uses one UTC calculation date for every spending.
- A future historical request uses the latest current rate and is provisional.
- Past historical entries are stable, while current and future contexts refresh on UTC day rollover.
- A stale quote must match the same context.
- Fixed past historical quotes have no stale age limit.
- Current and future stale quotes are eligible only through seven UTC calendar days after effective fetch date.

The Deferred table claims that keys, expiry, and stale eligibility remain binding, but no spine rule actually states those semantics. A deferred implementation detail cannot import an unstated contract. Compliant rate epics could classify the same request differently, use a quote from the wrong requested date, apply rolling-hour rather than UTC-calendar expiry, or accept stale current rates for different durations. Those choices directly alter debts, settlements, summaries, and participant archival eligibility.

**Required correction:** Put the complete mode-to-requested/effective-date mapping, cache class, UTC rollover, context-matching, stale window, and provisional rules in an adopted invariant. Define the calculation context from which all dates are derived.

### AI-02 - Critical - FX conversion and signed quantization are asserted but not algorithmically defined

**Spine references:** AD-3, lines 63-67; AD-9, lines 128-132.  
**Normative references:** `specs/design.md` lines 40, 48-53, 63, and 79-82; ADR 0001 sections 3 and 8.

The spine names `Decimal`, largest fractional remainder, participant-ID ties, and exact zero-sum preservation, but does not define the calculation that precedes quantization or the signed residual algorithm. Missing decisions include:

- Quote orientation and the exact conversion expression.
- Whether source amounts are converted per allocation, per participant net, per spending, or after grouping.
- Whether and where intermediate products or quotients may be rounded.
- The baseline operation for signed values: mathematical floor, truncation toward zero, or separate creditor/debtor treatment.
- How positive and negative residual units are assigned while preserving zero sum.
- How checked `Decimal` overflow or non-representable division is surfaced.

Largest-remainder language alone is insufficient for signed balances. For example, floor and truncation produce different remainder rankings for debtors. Converting and rounding each allocation also differs from accumulating exact converted balances and quantizing once. Both approaches can be described as deterministic and largest-remainder based, yet produce different balances and transfers.

**Required correction:** Specify an executable conversion and quantization procedure, including rate direction, accumulation unit, prohibition on intermediate rounding, signed baseline/remainder handling, deterministic iteration order, checked-failure behavior, and invariants checked after quantization.

### AI-03 - Critical - Participant archival is not safe across UTC rollover or changed rate observations

**Spine references:** AD-8, lines 93-125; AD-9, lines 128-132.  
**Normative references:** `specs/design.md` lines 56, 58, and 75-80.

The mutation epoch detects committed ledger changes only. It does not detect that the calculation date crossed midnight while the gate was released for provider I/O, that a current/future quote expired at rollover, or that a cache miss/refetch observed a revised provider value. This matters whenever all-time Historical mode includes future-dated spendings, because those use the latest current rate. The sequence permits a participant to be archived after midnight using the prior day's effective context as long as no ledger write advanced the epoch.

The final transactional "recheck eligibility" cannot repair this unless the architecture defines what rate-time facts it rechecks and how it does so without network I/O in the transaction. No such rule exists. Consequently, otherwise compliant archival implementations can either commit yesterday's zero balance, reject it, or recalculate, yielding different lifecycle and future spending outcomes.

**Required correction:** Capture one immutable archival calculation context containing UTC calculation date/time and all requested/effective rate contexts. At final admission, require both the ledger epoch and every time-sensitive context-validity predicate to remain valid. If UTC rollover or stale eligibility changes, return retryable with no state change and restart from a new snapshot.

### AI-04 - High - Provider revisions make cache eviction capable of changing archival and ledger-visible outcomes

**Spine references:** AD-9, lines 128-132; Deferred, line 251.  
**Normative references:** `specs/design.md` lines 78-80 and 117; ADR 0001 section 8; ADR 0002 decision 6.

The spine says deterministic eviction may refetch but may not alter correctness. It does not define correctness when a provider revises a historical quote. A process-lifetime cache hit can return the earlier value while an evicted key can return a revised value. That difference can change balances and, uniquely importantly, whether archival is permitted. Per-key single-flight removes duplicate concurrent calls; it does not make observations stable across eviction or restart.

This is not merely a display repeatability concern. Archival is a persisted ledger lifecycle mutation derived from external rates. Two compliant cache implementations with different but deterministic access/eviction histories can reach different archive outcomes for the same ledger.

**Required correction:** State the accepted consistency model for provider revisions. If archival must be reproducible from ledger state, pin or persist the exact quote bundle used for the archival decision, or require a provider/version contract that makes fixed-date observations immutable. If time-of-decision external truth is intentionally accepted, explicitly narrow the determinism claim and define archival audit/disclosure requirements.

### AI-05 - High - Settlement ordering does not define a unique greedy algorithm

**Spine references:** AD-9, lines 128-132.  
**Normative reference:** `specs/design.md` line 82.

"Descending absolute balance then participant ID greedy matching" does not uniquely define settlement. It leaves open whether one globally sorted list is walked once, creditors and debtors are sorted separately, parties are reinserted/re-sorted after partial settlement, and whether participant ID is ascending or descending on ties. Different greedy variants satisfy positivity, completeness, pair uniqueness, and the `n - 1` bound but can choose different transfer pairs and amounts.

Because settlements are a user-visible advisory ledger outcome, an epic must not infer this algorithm.

**Required correction:** Define the exact debtor and creditor queues, sort directions including ID direction, pair-selection step, transfer amount, cursor/re-sort behavior after each transfer, output ordering, and postconditions.

### AI-06 - High - Converted monthly summary arithmetic and degradation are underdefined

**Spine references:** AD-7, lines 87-91; AD-9, lines 128-132; capability map line 232.  
**Normative references:** `specs/design.md` lines 63 and 80.

AD-9 states that source totals survive and converted totals become unavailable, but it omits the defining summary rules: current UTC calendar month, group total plus per-payer totals, grouping by original source currency, and use of each spending date's Historical rate. It also does not define whether converted totals are formed by converting each spending exactly and then aggregating or by quantizing individual converted values before aggregation. Those approaches differ.

The phrase "mark only converted totals retryably unavailable" also does not explicitly prohibit a partially converted summary when one of several contexts is unavailable. One implementation may suppress the complete converted section atomically; another may show available currencies and warn on the rest. That produces materially different totals and can make a partial number appear complete.

**Required correction:** Bind the exact inclusion window, grouping dimensions, per-spending historical conversion, aggregation-before-display-quantization rule, target precision, and all-or-unavailable behavior for the converted section. Source-currency totals must be computed entirely from the database snapshot and remain available independently of rate success.

### AI-07 - High - One database snapshot is not one complete calculation context

**Spine references:** AD-7, lines 87-91; AD-9, lines 128-132.  
**Normative references:** `specs/design.md` lines 58, 63, 76-80, and 83.

AD-7 correctly snapshots group currency and spendings, but no rule requires the clock, mode, UTC calculation date, requested/effective rate contexts, and disclosure timestamp to be captured once before context derivation. A calculation crossing UTC midnight can select the monthly inclusion window using one date, create current/future rate contexts using another, and disclose a third timestamp. Concurrent tasks can independently read the clock and split one calculation across cache generations.

The database result is snapshot-consistent while the complete financial result is not. This can alter included spendings, selected quotes, provisional/stale status, balances, and warnings without any database mutation.

**Required correction:** Introduce an immutable application-owned `CalculationContext` captured once per request before snapshot selection. It must bind mode, UTC calculation instant/date, target currency resolution, monthly window where applicable, and deterministic context/disclosure ordering. All repository and rate work for that calculation must consume it rather than read the clock independently.

### AI-08 - High - Monetary limits and currency precision mapping are not bound by the spine

**Spine references:** AD-3, lines 63-67; consistency conventions lines 203-206.  
**Normative references:** `specs/design.md` lines 15, 49-50, and 68.

AD-3 refers generically to currency minor-unit precision and positivity but does not state the fixed mapping: zero for JPY/KRW, three for OMR, and two for every other supported currency. It also omits the upper bound of `999_999_999_999` for totals and every persisted payer/share amount. Deferring these values to feature code permits independently implemented parsers, domain constructors, and repository hydration paths to accept different values or fail at different arithmetic stages.

The maximum is part of overflow safety, not merely form validation. Without binding it at domain construction and persistence hydration, one implementation can persist values another cannot aggregate or convert.

**Required correction:** Add the complete supported-currency precision table and maximum to AD-3, and require identical validation on input construction and database hydration. State that excess precision is rejected rather than rounded and canonical TEXT has one normative formatter/parser.

### AI-09 - High - Lifecycle preconditions are not uniformly inside their committing transactions

**Spine references:** AD-4, lines 69-73; AD-5, lines 75-79; AD-6, lines 81-85.  
**Normative references:** `specs/design.md` lines 55, 57, and 59; ADR 0001 section 4.

AD-5 explicitly mandates transactional rechecks for spending ownership/activity and aggregate integrity. AD-4 states empty-group deletion, archived-group read-only behavior, and restricted cascades, but does not require the decisive "group has no spendings" check, group lifecycle check, and deletion/cascade to occur in one committing transaction. AD-6 only says writes acquire the gate before beginning a transaction; it does not say all persisted mutation preconditions must be read after gate acquisition in that same transaction.

The single process-local gate reduces races only if every lifecycle epic uses it around both decision and write. Current wording allows an application precheck outside the gate followed by a write transaction. Database foreign keys may prevent one destructive race, but they do not establish consistent application outcomes or cover every archive/restore and settings race.

**Required correction:** Establish a general mutation rule: every persisted lifecycle precondition that authorizes a write is authoritatively reloaded after gate acquisition and checked in the same transaction as the write. Explicitly apply it to empty-group deletion, group archive/restore/settings, participant add/edit/restore, and spending CRUD. Keep database restrictions as defense in depth.

### AI-10 - High - Share-set cardinality and role semantics permit divergent aggregate models

**Spine references:** AD-4, lines 69-73; AD-5, lines 75-79.  
**Normative references:** `specs/design.md` lines 51-54 and 60.

The spine requires one payer and names two modes, but does not completely define the persisted aggregate cardinality:

- At least one distinct share participant is required.
- A participant ID may occur at most once in the share set.
- The payer is allowed to also be a shareholder; payer and share are independent roles.
- Every persisted payer/share amount is positive and source-currency precise.
- Payer amount equals total and distinct share amounts sum exactly to total in source minor units.
- Modes and proportional weights are transient and are not persisted; stored edits reopen as Exact.

Without these rules, implementations can reject payer self-shares, merge duplicate submitted IDs, count duplicates independently, persist mode/weights and later recompute allocations, or model payer and shares as mutually exclusive. All can change balances for the same submitted form.

The archived-participant update wording is also ambiguous about whether "same role" permits changing the retained archived participant's share amount. The normative contract forbids introducing or changing the archived identity's role, not editing the amount of a retained role. That distinction should be explicit.

**Required correction:** Define aggregate cardinality, uniqueness, payer/share overlap, transient versus persisted fields, and archived-role update semantics in AD-5. Require duplicate participant IDs to be rejected rather than silently merged or overwritten.

### AI-11 - Medium - Proportional allocation lacks bounds and an exact normalization procedure

**Spine references:** AD-3, lines 63-67; AD-5, lines 75-79.  
**Normative references:** `specs/design.md` lines 52-53.

Positive `Decimal` weights and largest remainder do not by themselves define a safe, unique allocation. The spine does not bind a weight precision/range, checked summation behavior, the ideal-share expression, how division precision is handled by `rust_decimal`, or the exact remainder representation used for ranking. Mathematically equivalent evaluation orders can overflow or round differently within finite `Decimal` representation.

One epic may reject a high-scale weight, another may accept it and round an intermediate quotient, and a third may normalize weights before multiplication. They can return validation, calculation failure, or different residual recipients for the same input.

**Required correction:** Define accepted weight scale/range or a canonical integer-ratio normalization, checked sum and multiplication order, the representation used to compare remainders, and a fixed safe failure result when exact processing exceeds representable bounds.

### AI-12 - Medium - Financial calculation failure semantics do not cover all arithmetic paths

**Spine references:** AD-3, lines 63-67; AD-9, lines 128-132; AD-15, lines 178-182.  
**Normative reference:** `specs/design.md` line 40.

The spine guarantees no partial debts or archival result when rates are unavailable, but it does not carry forward the broader rule that debt arithmetic, conversion, quantization, and settlement failures must return checked domain errors, map to one fixed sanitized application calculation reason, and never panic or substitute zero. Generic safe-error wording in AD-15 does not prohibit a financial implementation from dropping a failed conversion, defaulting a failed multiplication to zero, or exposing partially accumulated summaries.

**Required correction:** Add an adopted financial-failure invariant covering every checked arithmetic and quantization operation. Define atomic failure for debts/settlements and atomic converted-section degradation for summaries; prohibit panic, saturation, zero-defaulting, skipped entries, and partial transfers.

## Cross-Cutting Acceptance Conditions

The spine should not be considered accounting-complete until it yields one answer for each of these fixtures independent of epic boundaries:

1. The same spending input produces one canonical persisted aggregate, including duplicate IDs, payer self-share, archived retained roles, and currency precision boundaries.
2. A fixed ledger snapshot plus a fixed rate bundle produces byte-for-byte identical balances, warnings, disclosures, and ordered transfers.
3. Signed quantization fixtures cover positive and negative ties, zero balances, multiple residual units, and participant-ID ties while preserving exact zero sum.
4. Settlement fixtures define the exact ordered transfer list, not only total correctness and count bounds.
5. Rate fixtures cover historical, current, future/provisional, UTC rollover, context-matching stale fallback, seven-day eligibility boundaries, eviction/refetch, and provider revision.
6. Archival fixtures cross a ledger mutation, UTC midnight, stale-expiry boundary, and provider/cache refresh; every invalidated decision commits no state.
7. Monthly summary fixtures prove source totals survive complete rate failure and that the converted section never presents a partial amount as complete.
8. Mutation integration tests prove all authoritative ownership, lifecycle, emptiness, activity, and aggregate checks occur in the committing transaction under the shared gate.

## Positive Observations

- AD-3 correctly excludes floating point and SQL monetary arithmetic/aggregation.
- AD-4 preserves group ownership and referenced identities.
- AD-6 establishes the correct single-process write serialization and SQLite durability baseline.
- AD-7 correctly prevents mixed database snapshots and database transactions spanning provider I/O.
- AD-8 recognizes that archival requires a two-phase epoch guard rather than a naive check-then-write.
- AD-9 correctly requires lexical arbitrary-precision rate decoding, bounded/single-flight fetching, deterministic ties, zero-sum balances, and graceful summary degradation at a high level.

These strengths are necessary but not sufficient: the missing executable semantics above remain capable of changing ledger-visible outcomes.
