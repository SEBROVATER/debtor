# Architecture Spine Rubric Review - Round 2

## Verdict

**FAIL - targeted closure is still required before the spine can govern independent epic implementation.**

The gate fixes materially improved the artifact. The spine now covers the major brownfield replacement boundary, logical data ownership, accounting aggregate shape, rate direction and settlement polarity, monthly-summary dimensions, security state transitions, topology, environments, and safe implementation deferrals. Current named technology is credible and the physical-schema, route-inventory, template-layout, proxy-vendor, and operations-playbook deferrals are not failures by themselves.

The remaining failures are narrower but architecture-bearing. The document still declares itself a non-binding draft, and several accounting and operational rules permit incompatible compliant implementations or make a claimed guarantee unenforceable. These are not requests for full UX requirements, physical schema, route inventory, or an operations runbook.

## Review Basis

- Reviewed artifact: `ARCHITECTURE-SPINE.md`, updated 2026-08-10.
- Normative source: `specs/design.md`, including synchronized gate changes.
- Accepted decisions: `specs/adr/0001-foundation-architecture.md` and `specs/adr/0002-long-term-foundation-hardening.md`.
- Brownfield evidence: current workspace structure, application/web surfaces, migrations, retained runtime/provider/session foundations, and the prior round reviews.
- Rubric: real epic divergence points, enforceable ADs, safe Deferred boundaries, current named technology, explicit brownfield disposition, architecture-bearing source coverage, and complete feature-altitude structural/operational classification.

## Findings

### 1. The spine still has no effective binding status

Frontmatter remains `status: draft` and `binds: []` (`ARCHITECTURE-SPINE.md:8-17`), while its scope says it governs independently implemented epics and AD-1 through AD-17 are marked `[ADOPTED]`. The source-precedence convention explains which document wins on substance, but it does not make this artifact binding.

An epic can reasonably treat a draft that binds nothing as advisory, including its brownfield removals and implementation deferrals. This is a governance blocker independent of the quality of the rules.

Required closure: accept the spine and identify the implementation/epic scope it binds, or explicitly define the external acceptance mechanism that turns all adopted ADs and Deferred boundaries into mandatory build constraints.

### 2. Canonical persistence grammar is not separated from user-input grammar

AD-3 assigns domain `format_decimal`/`parse_decimal` one canonical grammar that forbids redundant trailing zeroes, while AD-5 says application commands parse raw submitted amounts and weights. It does not state whether ordinary form input such as `1.00` in USD is accepted and normalized for canonical persistence or rejected as noncanonical.

The normative source requires excess precision rejection and canonical persistence, but does not require user-entered values to arrive in canonical storage form. A spending-command epic can therefore accept `1.00`, while a shared parser/domain epic rejects it, with both claiming compliance.

Required closure: scope strict canonical lexical validation to persisted/hydrated values and define the accepted raw input grammar and normalization boundary, or explicitly state that raw form values must already be canonical.

### 3. Proportional allocation is still not executable from the rule

AD-5 fixes positive decimal weights, largest fractional remainder, and ascending participant-ID ties, but leaves the finite-`Decimal` procedure undecided: accepted weight scale/range, checked normalization, evaluation order, quotient/remainder representation, and the failure result when intermediate arithmetic exceeds `rust_decimal` capacity.

`total * weight / sum` and `total / sum * weight` can round or overflow differently in finite decimal arithmetic. Independently implemented command and domain epics can assign different residual units or accept different inputs while satisfying the literal rule.

Required closure: bind one checked ideal-share/remainder procedure and safe representability limits, with deterministic failure rather than implicit intermediate rounding.

### 4. Exact-mode initial allocation has an unresolved residual rule

The normative source requires Exact mode to initially select every active participant with an equal minor-unit allocation. AD-5 bans Equal mode and says stored edits reopen as Exact, but does not decide how the initial Exact allocation distributes indivisible residual minor units.

For a total not divisible by participant count, separate native-preview and spending-command implementations can choose different participants for the extra units. This is a financial command/default contract, not a request for page layout.

Required closure: define the initial Exact allocation algorithm and tie direction, or assign one application operation as the sole authority consumed by every rendering path.

### 5. Converted monthly-summary output precision is undecided

AD-9 correctly fixes the month, grouping dimensions, per-spending Historical conversion, exact target aggregation, and all-or-unavailable degradation. It does not say whether the resulting group and per-payer converted totals are quantized to target minor units, how reconciliation between independently displayed totals is handled, or whether unquantized decimal values are rendered.

One summary epic can quantize each spending before summation, another can quantize only final buckets, and another can display full exact products. The new wording rules out some ordering mistakes but does not produce one user-visible monetary result.

Required closure: bind final display quantization and reconciliation after exact aggregation, including checked failure behavior. Per-spending intermediate rounding should remain forbidden.

### 6. Rate request, cache, and temporal metadata identities remain inconsistent

AD-9 defines request identity as `(source, target, requested_date, calculation_date)`, calls effective date returned metadata, uses the phrase “effective fetch date” for stale eligibility, and then says fixed past contexts remain stable for process lifetime while eviction may refetch and later observe provider revisions.

These statements do not establish one cache/single-flight key or distinguish quote effective date, fetch date, and calculation date. Including calculation date in a fixed-past request identity can create a new key every UTC day; keying by returned effective date requires a pre-response lookup indirection; treating “stable for process lifetime” literally conflicts with eviction/refetch revision semantics.

Required closure: define the lookup/single-flight key, stored quote identity, and separate `effective_date`/`fetched_on` semantics. Narrow “stable” to cache-class/eligibility behavior rather than immutable observation if eviction and revisions remain allowed.

### 7. The claimed post-dispatch bound is not established

AD-14 says the root mutation registry reaches an “intrinsically bounded terminal result” before checkpoint and pool close, and AD-12 forbids an edge timeout from expiring an admitted mutation. Composite archival runs rate work after dispatch. Although each provider call is bounded and concurrency is capped, the number of unique historical rate contexts grows with an unbounded ledger, so total post-dispatch duration has no fixed upper bound.

Implementations can consequently choose an infinite edge timeout, a finite but potentially violating timeout, or unbounded shutdown after the ten-second HTTP drain. All preserve some literal clauses but not the complete operational guarantee.

Required closure: either establish a real request-level work/time bound for dispatched composite mutations, or explicitly define shutdown and edge behavior as potentially open-ended after dispatch and remove the unsupported “intrinsically bounded” claim. A vendor-specific proxy playbook is not needed.

### 8. Archived-group rejection is overbroad enough to include restoration

AD-4 says archived groups are read-only and “mutation/form routes reject them before use-case invocation until restoration.” The release contract also requires group restoration. Read literally, the shared archived-group guard rejects the restoration form and mutation because restoration necessarily targets an archived group.

Different web/lifecycle epics can either exempt restoration or conclude that no archived-group mutation may invoke a use case.

Required closure: state that restoration and read-only views are the explicit archived-group exceptions; all other forms and mutations remain rejected before use-case invocation.

### 9. Login-limiter capacity response diverges from the accepted ADR

ADR 0001 section 7 fixes retryable `429` behavior for an unseen client when the 4,096-key limiter is full. AD-14 says only “fails closed” and the shared application taxonomy includes `Unavailable`, which can naturally map to `503`. The conventions table does not resolve this case.

Required closure: preserve the accepted `429` mapping in the spine or explicitly supersede and synchronize the ADR. This is a small but real cross-layer web/application contract.

### 10. Mandatory supervisor membership and cleanup failure policy are incomplete

AD-13 names several shared-state owners, AD-14 makes expired-record cleanup periodic and says cleanup failure fails readiness and initiates shutdown, and AD-15 gates readiness on “mandatory in-process supervisors.” It is not clear whether that failure rule covers session cleanup only or also submission-token and limiter indexed cleanup, nor which owner reports each health signal.

One implementation can treat token-cleanup death as readiness-fatal while another continues until capacity exhaustion. Both can claim that their chosen mandatory supervisors remain healthy.

Required closure: enumerate the mandatory supervisors at owner granularity and define which cleanup failures fail readiness and initiate shutdown. Backend, retention, alert routing, and an operations playbook may remain deferred.

## Rubric Assessment

| Criterion | Assessment | Basis |
| --- | --- | --- |
| Fixes real epic divergence points | Partial pass | Most first-round gaps are closed; raw/canonical decimal parsing, proportional allocation, Exact initialization, summary quantization, rate identity, and restoration still diverge. |
| Enforceable ADs | Fail | Frontmatter remains non-binding; AD-14 claims an unproven bound; several algorithms/state boundaries remain non-executable. |
| Safe Deferred boundaries | Pass | Physical names/order, route and source layout, proxy vendor, cache representation, asset digest timing, and pre-deployment operations design are now bounded by adequate logical interfaces. |
| Current named technology | Pass | The 2026-08-10 stack is current and compatible; `Cargo.lock` is named as exact Rust authority and browser assets have a pre-use verification gate. |
| Brownfield retained/superseded reality | Pass | Retained foundation and superseded membership, payer, Equal-mode, route, flow, API, schema, and test concepts are explicitly classified. |
| Architecture-bearing source/ADR coverage | Partial pass | Major accounting, security, topology, accessibility, summary, workflow, and history requirements are covered directly or through the normative source; the findings identify the remaining material gaps and one ADR mismatch. |
| Feature-altitude structural/operational dimensions | Partial pass | Structure, topology, provider, environment, deployment boundary, diagnostics, test strategy, and future scaling are classified; post-dispatch duration and mandatory-supervisor policy remain unresolved. |

## Material Strengths

- AD-3 through AD-9 now establish a substantially stronger shared accounting contract, including aggregate cardinality, canonical persistence intent, quote direction, immutable calculation contexts, signed quantization, settlement direction, summary dimensions, degradation, and archival revalidation.
- AD-10 through AD-15 now close the major security-review gaps: strict admission order, terminal submission tokens, concrete Argon2 bounds, trusted-client resolution, cookies, early-data rejection, singleton ownership, diagnostics, startup barrier, and mutation shielding.
- The Structural Seed and Brownfield Disposition are sufficient logical/migration guidance without prematurely choosing physical tables or retaining obsolete scaffold concepts.
- The Deferred table generally defers implementation choices only after shared interfaces and acceptance boundaries are fixed. It does not need a full route inventory, physical schema, proxy configuration, backup procedure, or telemetry playbook at this altitude.
- AD-11's direct binding to the browser/accessibility acceptance criteria in `specs/design.md` is adequate; duplicating the complete UX contract in the spine is unnecessary.

## Acceptance Gate

Acceptance requires resolving Findings 1 through 7. Findings 8 through 10 are small, local clarifications but should be closed in the same gate pass because they affect lifecycle, HTTP, and readiness interoperability. No broader UX, schema, route, or operations expansion is required.
