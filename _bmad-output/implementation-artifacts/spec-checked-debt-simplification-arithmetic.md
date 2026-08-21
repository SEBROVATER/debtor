---
title: 'Use checked arithmetic for debt simplification transfer limits'
type: 'bugfix'
created: '2026-08-21T16:44:31+06:00'
status: 'done'
baseline_commit: '0a1a4cc6ca0538efd5e68ea4d8c917f665b0ce1c'
review_loop_iteration: 0
context:
  - '{project-root}/specs/design.md'
  - '{project-root}/_bmad-output/project-context.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** `debtor-domain` debt settlement simplification uses `usize::saturating_sub(1)` when enforcing its maximum transfer count. Saturation silently turns an impossible zero-participant subtraction into zero instead of surfacing the domain's required checked calculation failure.

**Approach:** Replace the saturating operation with a checked operation that produces `CalculationError::ArithmeticOverflow`, and add a direct domain regression test for the otherwise unreachable zero-count boundary. Keep normal settlement behavior, including all-zero balances, unchanged.

## Boundaries & Constraints

**Always:** Keep the change local to the pure `debtor-domain` settlement rule; return typed checked domain errors, never panic, saturate, default, or emit partial transfers; retain deterministic transfer ordering and the existing at-most-`n - 1` bound; place the regression test in the owning inline domain test module; preserve the all-zero balance case as a successful empty transfer set because the transfer loop is not entered.

**Ask First:** Expanding the change beyond the simplifier, changing externally observable calculation-error mapping, altering settlement matching/order rules, or changing public APIs requires human approval.

**Never:** Do not use floating point, unchecked conversion, `unwrap`/`expect` in production code, SQL, persistence, application/web changes, compatibility shims, or unrelated formatting/refactoring.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Normal settlement | Simplifier enters its debtor/creditor matching loop with a participant count of two or more | The transfer limit remains `participant_count - 1`; valid transfers retain existing output and ordering | N/A |
| Checked boundary | The isolated transfer-limit calculation receives participant count `0` | No saturated `0` limit is produced | Return `CalculationError::ArithmeticOverflow` |
| All-zero balances | `simplify` receives only zero participant balances | The matching loop is skipped and simplification returns an empty transfer collection | Must not evaluate the transfer-limit subtraction or return an error |

</frozen-after-approval>

## Code Map

- `debtor-domain/src/debts/simplify.rs` -- owns deterministic balance simplification, the maximum transfer-count invariant, and its inline unit tests.
- `debtor-domain/src/debts.rs` -- defines `CalculationError`, including the checked-arithmetic failure variant used by the simplifier.
- `debtor-application/src/debts.rs` -- maps the domain arithmetic error to a sanitized application reason; no change is expected, but it establishes the error's existing outward handling.

## Tasks & Acceptance

**Execution:**
- [x] `debtor-domain/src/debts/simplify.rs` -- replace the `saturating_sub(1)` transfer-limit calculation with a private checked calculation evaluated only in the active matching loop, propagating `CalculationError::ArithmeticOverflow` on underflow -- ensures arithmetic failures cannot be silently converted into a valid-looking settlement limit while retaining the reachable valid path.
- [x] `debtor-domain/src/debts/simplify.rs` -- add an inline unit regression test that invokes the transfer-limit calculation with zero participants and asserts the typed arithmetic-overflow error -- proves the exact underflow boundary does not regress to saturation and is testable without constructing an impossible simplifier state.

**Acceptance Criteria:**
- Given a transfer limit is requested for zero participants, when the simplifier's internal limit calculation runs, then it returns `CalculationError::ArithmeticOverflow` rather than a saturated zero limit.
- Given a valid nonzero settlement enters the debtor-creditor matching loop, when its transfer bound is checked, then the simplifier enforces the existing at-most-`participant_count - 1` limit and preserves its current deterministic output.
- Given every participant balance is zero, when `simplify` runs, then it returns an empty successful transfer collection without evaluating the underflow-prone calculation.

## Design Notes

The zero-count condition cannot be reached through the public `simplify` loop: entering it requires at least one debtor and one creditor, so the collected participant count is at least two. A small private helper makes that invariant's arithmetic independently testable without weakening production behavior or manufacturing an infeasible input. The helper must remain called inside the loop; computing it before the loop would incorrectly reject all-zero balances.

## Verification

**Commands:**
- `cargo fmt --all -- --check` -- expected: Rust formatting succeeds without modifications.
- `cargo test -p debtor-domain` -- expected: existing domain tests and the zero-count checked-arithmetic regression pass.
- `cargo check --workspace --all-features --locked` -- expected: all workspace packages compile in debug mode.
- `SQLX_OFFLINE=true cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` -- expected: no warnings, including lint regressions in the test module.
- `cargo test --workspace --all-features --locked` -- expected: all workspace tests pass.

## Suggested Review Order

**Checked transfer bound**

- Calculates the invariant with a typed failure instead of silently saturating.
  [`simplify.rs:58`](../../debtor-domain/src/debts/simplify.rs#L58)

- Keeps the underflow policy small, local, and independently testable.
  [`simplify.rs:98`](../../debtor-domain/src/debts/simplify.rs#L98)

**Regression coverage**

- Locks the zero-count boundary to the domain arithmetic-overflow error.
  [`simplify.rs:114`](../../debtor-domain/src/debts/simplify.rs#L114)
