# Accounting And Data-Integrity Review - Round 2

## Review Basis

- Reviewed artifact: `ARCHITECTURE-SPINE.md`, dated 2026-08-10.
- Normative contract: `specs/design.md`.
- Synchronized companions: `specs/adr/0001-foundation-architecture.md`, `specs/adr/0002-long-term-foundation-hardening.md`, PRD, and PRD addendum.
- Prior accounting review: `reviews/review-accounting-integrity.md`.
- Retested: rate modes and staleness, conversion and signed quantization, archival UTC/provider-revision consistency, settlement, monthly summaries, immutable calculation context, precision and maxima, transactional lifecycle, spending cardinality, proportional inputs, and checked failures.
- Review boundary: architecture contracts only. Domain implementation details already conclusively owned by a normative rule are not repeated as findings.

## Verdict

**CHANGES REQUIRED - no critical hole remains, but four architecture holes can still produce divergent accepted allocations or financial outputs.**

The second-pass spine closes the prior critical findings and most high findings. Historical/Current/future rate modes, UTC rollover, stale windows, immutable quote bundles, archival epoch/time admission, provider revision policy, quote orientation, debt conversion order, signed quantization, settlement queues, monetary precision/maxima, transactional persisted preconditions, and spending cardinality are now architecture-level invariants. The remaining gaps are narrower but still cross epic boundaries and therefore cannot safely be left to local implementation choice.

## Remaining Findings

### AR2-01 - High - Request, cache, quote, and stale-fallback identity are not one coherent model

**Spine references:** AD-8, line 97; AD-9, line 132; Deferred, line 283.  
**Normative references:** `specs/design.md` lines 78-80; PRD addendum lines 64-65.

AD-9 defines request identity as `(source, target, requested_date, calculation_date)` and treats effective date as returned metadata. The synchronized specifications instead bind cache/rate identity to source, target, original requested date, and effective fetch date. AD-9 then says stale fallback must "match context" without defining whether that means the request tuple, the returned effective-date tuple, or both.

The distinction affects observable accounting. Including `calculation_date` in a fixed-past request identity can create a new identity each UTC day despite the same rule declaring fixed-past contexts stable for process lifetime. Excluding returned effective date from fallback identity permits two provider observations for the same request but different effective dates to compete without an architectural selection rule. Independently implemented cache, single-flight, disclosure, archival, and summary paths can therefore choose different quote evidence while each claims context matching.

**Required closure:** Define separate immutable request-context identity and returned quote-evidence identity, then bind deduplication, single-flight, stable/refreshable cache lookup, stale-candidate matching, disclosure uniqueness, and archival revalidation to the appropriate identity. Reconcile whether `calculation_date` or returned effective date participates in each identity so fixed-past stability and UTC refresh semantics do not contradict the key model.

### AR2-02 - High - Proportional allocation still lacks a representable normalization contract

**Spine references:** AD-3, line 67; AD-5, line 79.  
**Normative references:** `specs/design.md` lines 52-53; PRD addendum lines 24-27.

The spine requires positive `Decimal` weights and largest-fractional-remainder allocation, but does not constrain weight scale or magnitude or define a canonical ratio calculation. With finite `rust_decimal::Decimal`, mathematically equivalent forms such as `total * weight / sum`, `weight / sum * total`, and normalized integer ratios have different overflow, rounding, and representability boundaries. Largest-remainder ranking is not unique if the ideal shares have already been rounded differently.

This is not an internal optimization choice: it determines whether the same submitted weights are accepted, rejected, or assigned to different residual recipients. Application parsing, domain allocation, native Preview, enhanced Preview, and final submission need one contract.

**Required closure:** Bind either a safe weight grammar with maximum scale/magnitude plus one checked evaluation and remainder representation, or canonicalize weights to bounded integer ratios before allocation. Define the safe failure category when normalization or checked arithmetic is not representable. Preview and commit must consume the same canonical operation.

### AR2-03 - Medium - Monthly converted totals have no target-precision conservation rule

**Spine reference:** AD-9, line 132.  
**Normative references:** `specs/design.md` lines 49 and 63; PRD FR-7/FR-8; PRD addendum lines 28 and 34.

AD-9 now correctly fixes the UTC-month window, source-currency grouping, per-spending Historical quote, exact conversion before aggregation, and all-or-unavailable converted section. It stops at "exact target aggregation" and does not say how group and per-payer totals become target-currency minor-unit values.

Leaving the final projection implicit permits at least three different outputs: unquantized decimal totals, independently quantized per-payer totals whose sum differs from the displayed group total, or a conserved group/per-payer quantization with deterministic residual assignment. Currency precision in AD-3 rejects excess-precision persisted money but does not itself define calculated-summary projection. The same snapshot and quote bundle can therefore produce different displayed Group Currency totals.

**Required closure:** State whether converted summaries are exact informational decimals or target-minor-unit amounts. If they are currency amounts, require quantization once after exact aggregation and define a deterministic conservation relationship between the group total and per-payer totals, including the tie-breaker. Do not permit per-spending display rounding to feed aggregation.

### AR2-04 - Medium - Checked failure is atomic for debts, but not for allocation and converted-summary arithmetic

**Spine references:** AD-3, line 67; AD-9, line 132; AD-15, line 182.  
**Normative references:** `specs/design.md` lines 40, 63, and 80; PRD addendum lines 27-28 and 56-57.

AD-9 forbids panic, saturation, zero substitution, skipped entries, and partial debts/transfers. Its monthly-summary degradation rule makes the converted section unavailable when a quote context is unavailable, but does not assign the same atomic result when checked multiplication, summation, or final quantization fails. AD-5 likewise has no explicit result for non-representable proportional normalization. AD-15 provides a `Calculation` category but does not determine which surface fails or whether source totals survive.

An implementation can therefore return a storage-style failure for the whole group page, omit one converted total, retain a partially accumulated converted section, or convert the section to retryable unavailable. Allocation paths can disagree between validation and calculation failures. Those are user-visible architectural outcomes, not domain error-type details.

**Required closure:** Extend the checked-failure invariant to every proportional-allocation, conversion, aggregation, and quantization path. Proportional construction must fail without an aggregate; debts and settlements must remain wholly absent; monthly source totals must remain available while the entire converted section fails atomically under one bounded application reason. Panic, saturation, zero substitution, entry omission, and partial converted totals must be forbidden explicitly.

## Retest Disposition

The following prior concerns are closed at architecture altitude and are not findings in this round:

- Rate modes, requested dates, UTC refresh, fixed-past and seven-day stale eligibility, provisional future quotes, global/provider concurrency, immutable per-calculation quote bundles, and accepted later provider revisions.
- Quote orientation, source-net conversion without intermediate rounding, participant accumulation, signed truncation/remainder ordering, target zero-sum preservation, and balance signs.
- Archival capture of ledger snapshot, mutation epoch, UTC instant/date, requested/effective contexts, and quote bundle; final epoch/date/eligibility admission; retry without state change; non-persistence of quote evidence.
- Separate deterministic debtor/creditor queues, transfer direction and amount, cursor advancement without re-sorting, generated output order, pair uniqueness, completeness, and the `n - 1` bound.
- Immutable application `CalculationContext`, one-snapshot calculation reads, and release of database transactions before provider I/O.
- Canonical exact decimal grammar, checked `Decimal`, canonical TEXT persistence, hydration validation, precision mapping, excess-precision rejection, positivity, and `999_999_999_999` maximum.
- Shared gate ownership, mutation epoch advancement only after commit, and authoritative reloading of every persisted write precondition in the committing transaction.
- Complete `Spending` aggregate cardinality: one payer, nonempty participant-unique shares, payer/share overlap, positive exact amounts, equality to total, rejected duplicates, transient modes/weights, Exact edit reopening, and archived-role retention.

## Conclusion

The accounting spine is materially stronger than round 1 and no longer has a critical archival, rate-mode, conversion, settlement, precision, transaction, or cardinality defect. It is not yet implementation-deterministic for rate evidence identity, proportional normalization, or converted-summary projection/failure. Closing AR2-01 through AR2-04 is sufficient for this review scope; no spine edit was made by this review.
