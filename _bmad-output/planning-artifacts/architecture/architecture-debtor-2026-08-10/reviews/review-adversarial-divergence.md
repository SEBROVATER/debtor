# Adversarial Divergence Review

## Verdict

**FAIL - the spine does not yet prevent incompatible independently implemented epics.** The adopted decisions establish strong layer, safety, and accounting boundaries, but several shared contracts are named without defining the representation or semantics that make them interoperable. The failures below are not deferred schema, route, or implementation choices: each permits two implementations that satisfy every AD literally while disagreeing on persisted truth, application-facing types, financial output, security admission, or operational behavior.

## Method

For each finding, two hypothetical epics were constructed independently. Both implementations were required to honor all literal AD requirements, including inward dependencies, exact `Decimal`, Rust-owned calculation, transactional guards, deterministic processing, bounded state, and the single-process topology. A finding is included only where the pair can remain compliant yet cannot be composed without changing one epic's behavior or contract.

## Findings

### 1. CRITICAL - Canonical monetary `TEXT` has no canonical grammar

**Evidence:** AD-3 requires canonical SQLite `TEXT`; the persistence convention requires Rust-owned canonical decimal validation. Neither defines the accepted lexical grammar or whether currency scale is represented in storage.

**Epic A:** The spending epic canonicalizes a USD amount to fixed currency scale, storing `1.00`, rejecting `1`, `1.0`, `01.00`, and exponent notation when decoding rows.

**Epic B:** The summary/persistence epic canonicalizes every `Decimal` to normalized coefficient form, storing `1`, rejecting `1.00` as noncanonical while separately validating USD minor-unit precision.

Both use exact `Decimal`, canonical `TEXT`, checked Rust parsing, and no SQL arithmetic. Each rejects the other's valid writes as corrupt. This is a shared persisted-data contract, not a concrete table-shape decision. The spine must define one canonical decimal grammar, including zero, sign, leading/trailing zeros, decimal point, exponent notation, and scale policy.

### 2. CRITICAL - A “complete spending aggregate” has no owned application-facing shape

**Evidence:** AD-5, AD-7, AD-8, and the capability map repeatedly require complete spending aggregates, but no AD assigns ownership or defines the minimum semantic shape shared by spending writes, history, summaries, debts, and archival.

**Epic A:** Spending CRUD exposes an application-owned aggregate containing `total`, `payer_id`, and participant-unique positive `shares`; payer payment is implicit from `total`.

**Epic B:** Debt calculation exposes a domain aggregate containing explicit typed payer and share allocation records, including a positive payer allocation equal to total, and expects the snapshot reader to return that representation.

Both preserve exactly one payer, exact payer/share equality, positive values, group ownership, and snapshot completeness. Their repositories and use cases cannot compose because “complete” does not determine whether payer payment is implicit, whether payer and shares share one collection, whether a payer may also have a share, or which crate owns the canonical aggregate. Deferring SQL table shape does not justify deferring this cross-epic semantic contract.

### 3. CRITICAL - Rate quote direction and context identity are underdefined

**Evidence:** AD-9 says contexts include source, target, requested date, and effective date, while also requiring deduplication, caching, and single-flight. It never defines the quote equation or distinguishes request identity from returned quote metadata. Effective date is generally unknown until the provider responds.

**Epic A:** The provider port returns “target units per one source unit”; callers multiply. Cache and single-flight identity is `(source, target, requested_date)`, with `effective_date` stored as quote metadata.

**Epic B:** The provider port returns “source units per one target unit”; callers divide. It treats all four named fields as the context/cache key and maintains an index from requested context to effective contexts.

Both decode lexically into `Decimal`, preserve all four fields, bound and deterministically evict caches, deduplicate, and respect concurrency. Combining Epic A's calculator with Epic B's provider silently produces reciprocal financial results; combining their cache ports is type- and lookup-incompatible. The spine needs a quote equation and separate request-key versus resolved-quote identity.

### 4. HIGH - Historical/current/future and stale-cache semantics are asserted but not specified by any AD

**Evidence:** AD-9 names rate contexts, stable/refreshable caches, stale/provisional warnings, and settlement mode, but does not define either mode, future-date resolution, cache-class membership, UTC rollover, or stale eligibility. The Deferred table claims these semantics “remain binding,” although the AD body contains no such binding rule.

**Epic A:** Past requested dates are stable forever; current and future requests use the calculation date, refresh at UTC rollover, and permit stale fallback for seven UTC days.

**Epic B:** Every requested date, including a future date, is fetched and cached under its effective provider date; all successful historical responses are stable, while refreshable entries expire after a rolling duration.

Both can truthfully implement stable and refreshable bounded caches, requested/effective dates, stale/provisional warnings, and deterministic output under the literal ADs. They disagree on provider calls, cache hits, warning status, and whether a calculation is available. A companion source contains much of the intended policy, but the spine does not incorporate it into an AD despite claiming the decision is fixed.

### 5. HIGH - Monthly conversion order and rate-date selection are not fixed

**Evidence:** AD-7 requires snapshot-complete reads for converted summaries, and AD-9 requires deterministic rate processing, but neither states what is converted, at which date, or in which order before aggregation.

**Epic A:** Converts each spending and each payer contribution using that spending's requested historical date, then aggregates converted values for the current month.

**Epic B:** Aggregates exact source-currency monthly and per-payer buckets first, then converts each bucket using the UTC calculation date.

Both materialize complete aggregates from one snapshot, perform all arithmetic in Rust, preserve source-currency totals, use exact rates, and produce deterministic converted summaries. Their group totals differ whenever rates vary during the month. This is calculation behavior shared by spending, rate, and summary epics, not a rendering detail.

### 6. HIGH - Largest-remainder tie direction and signed quantization are not executable rules

**Evidence:** AD-5 and AD-9 say participant ID is the tie-breaker, but never say lower or higher ID wins. AD-9 applies largest fractional remainder to signed final balances without defining floor/truncation, remainder sign/order, or correction direction.

**Epic A:** Orders equal remainders by ascending participant ID and quantizes signed values from mathematical floor.

**Epic B:** Orders equal remainders by descending participant ID and quantizes from truncation toward zero with a signed correction pass.

Both use largest fractional remainder, use participant ID as the tie-breaker, preserve exact zero sum, and are deterministic. They can assign a residual minor unit to different participants, producing different balances, archival eligibility near zero, and transfers. “Participant-ID ties” must define ordering direction, and signed quantization must define its baseline and correction ordering.

### 7. HIGH - Settlement transfer polarity and matching transition are undefined

**Evidence:** AD-9 fixes ordering, positivity, completeness, pair uniqueness, and the `n - 1` bound, but does not define balance polarity, transfer endpoint meaning, or the exact debtor/creditor selection transition.

**Epic A:** Positive balance means the participant is owed money; a transfer is `{ from: debtor, to: creditor, amount }`.

**Epic B:** Positive balance means the participant owes money; a transfer is `{ from: creditor, to: debtor, amount }`, with the web layer reversing labels for display.

Each can produce checked, positive, complete, pair-unique deterministic greedy transfers bounded by `n - 1`. A debt service from one epic and a view/application port from the other reverses who pays whom. The spine must define the balance equation/sign and transfer direction as application-facing semantics, not merely algorithmic properties.

### 8. HIGH - “Fixed structured reason categories” has no fixed taxonomy

**Evidence:** AD-15 requires fixed structured application-facing reason categories, while conventions provide selected HTTP statuses. No AD enumerates the reason set, assigns ownership, or maps failures such as unavailable rate, epoch mismatch, gate timeout, SQLite busy, archived lifecycle, integrity rejection, and capacity exhaustion.

**Epic A:** Exposes feature-local enums such as `ArchiveFailure::ConcurrentMutation` and `SpendingFailure::IneligibleParticipant`; web exhaustively maps those variants.

**Epic B:** Exposes a shared application enum such as `Retryable`, `Conflict`, `Invalid`, and `Unavailable`, with safe operation-specific codes as fields.

Both are fixed, structured, safe, and contain no adapter diagnostics. Their application ports and web mappings cannot compose, and they can disagree on retryability/status while satisfying the listed `409`/`422` conventions. The taxonomy and its owning layer are architecture contracts; diagnostic strings and adapter details can remain deferred.

### 9. HIGH - The mutation-dispatch boundary is ambiguous for archival and other asynchronous prechecks

**Evidence:** AD-10 reserves a token immediately before first mutation dispatch; AD-14 puts asynchronous prechecks inside the pre-dispatch deadline; AD-8 defines archival as snapshot, provider I/O, calculation, then guarded commit. No AD identifies which archival step crosses dispatch.

**Epic A:** Reserves the token before invoking the archival use case. Snapshot/rate/calculation therefore happen after dispatch, outside the 30-second deadline, and any provider failure spends the one mutation attempt.

**Epic B:** Treats snapshot/rate/calculation as asynchronous prechecks. It reserves the token only before the final guarded archive command, so precheck timeout or unavailable rates leave the token reusable.

Both reserve immediately before what they independently define as first mutation dispatch, never hold a transaction over provider I/O, and perform the AD-8 epoch protocol. They disagree on timeout coverage, duplicate admission, token consumption, and how long the edge must keep a request alive. The spine needs one explicit dispatch point for composite mutation use cases and a definitive token transition for every post-reservation outcome.

### 10. HIGH - Login limiter admission and trusted-client identity are not a shared web/edge contract

**Evidence:** AD-12 assigns forwarding sanitation to the edge; AD-14 limits attempts per trusted client IP; AD-2 assigns authentication/session mechanics to web. The ADs do not define the accepted forwarding format, chain-selection algorithm, direct-peer fallback, or which rejected requests count as login attempts.

**Epic A:** The edge replaces `X-Forwarded-For`; web trusts the leftmost value from configured proxy peers and increments the limiter only when password verification runs after valid CSRF.

**Epic B:** The edge appends its peer to RFC `Forwarded`; web walks the chain right-to-left and increments after bounded login-body admission, including malformed credentials but excluding CSRF failures.

Both use a sanitizing trusted proxy, one selected interpretation, a rolling five-attempt limit, bounded active keys, and fail closed at capacity. Deployed together, one pair may collapse all users to the proxy address or accept an attacker-controlled address, and the epics disagree on limiter state transitions. Concrete proxy vendor configuration may remain deferred, but canonical trusted-client resolution and attempt-counting semantics cannot.

## Required Closure

The spine need not choose tables, routes, template layout, proxy vendor, or concrete cache data structures. It does need to bind canonical lexical formats, shared application/domain contracts, exact calculation semantics, security state transitions, and cross-component edge/web interpretation. Until those contracts are adopted, independent epic compliance is insufficient to guarantee composition.
