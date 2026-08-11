# Accounting Architecture Review - Round 3

## Verdict

**CHANGES REQUIRED - no critical hole remains; one high-severity synchronization contradiction remains in rate stale-fallback identity.**

## Findings

### AR3-01 - High - The synchronized rate contract still gives incompatible stale-fallback identities

**Architecture reference:** `ARCHITECTURE-SPINE.md:132`  
**Normative reference:** `specs/design.md:78-80`  
**Conflicting synchronized specification:** `_bmad-output/planning-artifacts/prds/prd-debtor-2026-08-10/addendum.md:64-65`

The spine now coherently separates the request/cache key `(source, target, R, F)` from returned provider-effective-date evidence and defines stale selection by temporal class: fixed-past fallback is an exact-key match; current fallback selects the latest prior current-class quote for the currency pair; future fallback additionally requires the same original `R`. The normative design contract states the same model. This necessarily permits current and future stale selection across UTC rollover after refresh failure even though the prior quote has an earlier `F`; for Current mode, the prior quote also has an earlier `R`.

The PRD addendum instead states that rate identity is the complete `(source, target, original requested date, effective fetch date)` tuple, that stale fallback must match that complete context, and that fallback across contexts is forbidden. Read literally, this prohibits both cross-rollover paths required by the adopted spine. An independently implemented rate/cache epic following the addendum must reject a quote that an archival, debts, or summary epic following the spine must accept. The disagreement changes calculation availability, stale disclosures, converted summaries, settlement results, and participant-archival admission for the same ledger and calculation date.

**Required closure:** Synchronize the addendum to distinguish request/cache identity, returned quote evidence, and stale-candidate selection identity. Preserve exact-key matching for fixed past quotes, pair/current-class matching for Current fallback, and pair plus original-`R` matching for future fallback, with the existing inclusive seven-UTC-day eligibility rule.
