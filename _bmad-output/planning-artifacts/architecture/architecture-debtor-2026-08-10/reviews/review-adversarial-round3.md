# Narrow Adversarial Review - Round 3

## Verdict

**CHANGES REQUIRED - two high-severity cross-epic incompatibilities remain.**

## Findings

### 1. HIGH - Monthly rate absence has two incompatible exhaustive error categories

**Evidence:** AD-15 assigns provider failure to `Unavailable` with a `503` mapping, while AD-9 requires missing rates and checked conversion, aggregation, or quantization failure to make the converted monthly section unavailable under one sanitized `Calculation` reason (`ARCHITECTURE-SPINE.md:132,182`). AD-15 otherwise reserves `Calculation` for checked financial failure and gives monthly calculation failure only a presentation exception; it does not supersede the provider-failure category. The normative source likewise requires one retryable converted-summary warning but does not resolve the inward-facing category collision (`specs/design.md:63,80`).

An independently implemented rate/application epic can return `Unavailable` when no quote exists, as AD-15 requires. A summary epic can require the same condition to arrive as `Calculation`, as AD-9 requires, so that rate absence and arithmetic failure share one reason. The two ports cannot compose under an exhaustive taxonomy without one epic violating an adopted rule; they also disagree on retry semantics because deterministic checked arithmetic is ordinarily a sanitized `500`, whereas provider absence is retryable `503`.

**Required closure:** Choose the application category for monthly quote absence and distinguish it from checked arithmetic failure, or explicitly define a monthly-summary projection that consumes both exhaustive categories and collapses them only at the rendered-section boundary. Bind retryability for each source condition without relabeling deterministic arithmetic failure as provider unavailability.

### 2. HIGH - Task failure can falsely terminalize an already committed mutation as rollback

**Evidence:** AD-10 requires a definitive commit/rollback result after dispatch and makes token reservation terminal even on task failure (`ARCHITECTURE-SPINE.md:138`). AD-14 requires shutdown to wait for registry terminal states, but declares that task panic/failure terminalizes as rollback/failure (`ARCHITECTURE-SPINE.md:176`). A database commit and the process-local registry transition cannot be atomic. A dispatched task can commit successfully and then panic, be cancelled by supervisor failure, or fail while publishing its result to the registry. The literal AD-14 fallback then records rollback/failure despite durable committed state.

The persistence epic can correctly commit and advance the mutation epoch, while the root supervision epic correctly observes task failure and terminalizes the registry as rollback/failure. Both obey their local rules, but the web receives a false failure for a committed mutation. Because the submission token is terminal, a user retry with a new token can duplicate a non-idempotent mutation. Shutdown may also checkpoint and close based on a terminal state that is not the definitive database outcome promised by AD-10.

**Required closure:** Make definitive outcome publication part of the mutation execution contract: prohibit any panic/cancellation window after commit and before registry publication, or add a durable operation/result mechanism that can recover the committed outcome. Task failure may be classified as rollback only when rollback is authoritatively established; an unknown outcome must not be represented as rollback/failure or permit ordinary retry.
