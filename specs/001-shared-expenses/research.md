# Research: Shared Expenses Manager

## Decision 1: Async execution model and web runtime

**Decision**: Use `acton-htmx` with `tokio` multi-thread runtime; all I/O
boundaries (database, session store, external rates API) stay async.

**Rationale**:
- Satisfies the explicit async requirement.
- Matches the framework's intended non-blocking request model.
- Prevents thread starvation during slow exchange-rate calls.

**Alternatives considered**:
- Blocking SQLite calls on request thread (rejected: violates async-first goal).
- Hybrid async backend with sync outbound HTTP (rejected: introduces latency spikes).

## Decision 2: Exchange-rate provider and cache policy

**Decision**: Use Frankfurter API (`https://api.frankfurter.app/latest`) as the
primary free provider; cache rates per `(from_currency, to_currency, date)` in
SQLite. Refresh lazily on summary request when cache is stale or missing.

**Rationale**:
- Free, simple HTTP API, no API key required.
- Daily data cadence naturally fits "at most once per day" requirement.
- Pair+date cache key enforces SC-004.

**Alternatives considered**:
- `exchangerate.host` (rejected: key/terms stability risk over time).
- Open Exchange Rates free tier (rejected: tighter limits and key management overhead).

## Decision 3: Monetary precision and rounding

**Decision**: Represent money and rates with `rust_decimal::Decimal`.
Store decimals in SQLite via SeaORM; round converted output to 2 decimals using
half-up mode only at presentation/settlement output boundaries.

**Rationale**:
- Avoids binary floating-point drift.
- Keeps per-participant split validation exact.
- Directly supports the spec's explicit rounding rule.

**Alternatives considered**:
- `f64` (rejected: precision errors in debt calculations).
- Integer minor units only (rejected: harder mixed-share math and variable currency scales).

## Decision 4: Provably optimal debt simplification

**Decision**: Compute net balances, then run subset-DP to maximize the number
of zero-sum partitions among non-zero members. For each partition of size `k`,
settle in `k-1` transfers. This guarantees globally minimal transaction count.

**Rationale**:
- Meets SC-005 ("minimum possible number of transactions").
- With max 20 members, `O(n * 2^n)` is practical.
- Deterministic output can be tested with golden cases.

**Alternatives considered**:
- Greedy debtor/creditor matching (rejected: fast but not always optimal).
- ILP solver integration (rejected: unnecessary operational complexity).

## Decision 5: Auth/session/lockout storage

**Decision**: Persist one admin user row, server-side session table (hashed
session tokens), and a dedicated auth state row for failed-attempt counters and
lockout timestamps.

**Rationale**:
- Supports 30-day sliding session expiry and immediate logout invalidation.
- Lockout survives restarts.
- Keeps implementation simple for single-user domain.

**Alternatives considered**:
- Stateless JWT (rejected: weak fit for immediate revocation + sliding expiry).
- In-memory lockout/session state (rejected: restart data loss).

## Decision 6: Frontend interaction contract

**Decision**: Serve semantic HTML pages and HTMX fragments only. Use modern
vanilla CSS with tokens (`:root` custom properties), Grid/Flex layouts, and no
CSS frameworks.

**Rationale**:
- Fully aligned with constitution principles I and III.
- Maintains low frontend complexity and easy auditability.

**Alternatives considered**:
- Custom JavaScript controllers (rejected: violates JS-free principle).
- Tailwind/Bootstrap (rejected: violates vanilla CSS principle).

## Decision 7: TDD test strategy

**Decision**: Enforce Red-Green-Refactor at every layer:
- Unit tests: split validation, balance aggregation, subset-DP simplification.
- Integration tests: authenticated route guards, CRUD flows, lockout behavior.
- Contract tests: exchange-rate adapter (success/stale fallback/failure path).

**Rationale**:
- Required by constitution principle VI.
- Ensures behavioral confidence for money and auth logic.

**Alternatives considered**:
- Post-implementation tests (rejected: violates mandatory TDD).
- Integration-only testing (rejected: poor defect localization).

## Clarification Resolution Summary

All planning-time unknowns are resolved:
- Exchange provider selected (Frankfurter).
- Precision strategy selected (`Decimal` + explicit rounding boundary).
- Debt minimization algorithm selected (subset-DP exact method).
- Auth persistence strategy selected (server-side sessions + persistent lockout).
