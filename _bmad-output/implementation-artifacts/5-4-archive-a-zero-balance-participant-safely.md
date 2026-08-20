---
story_key: 5-4-archive-a-zero-balance-participant-safely
story_id: 5.4
epic: 5
status: done
created: 2026-08-20
baseline_commit: 0306289
completion_note: Ultimate context engine analysis completed - comprehensive developer guide created
---

# Story 5.4: Archive a Zero-Balance Participant Safely

Status: done

## Story

As the administrator,
I want to archive a Participant only when their Historical Balance is exactly zero in an unchanged context,
so that active choices stay clean without hiding unsettled obligations or racing ledger changes.

## Acceptance Criteria

1. Given Manage evaluates an active Participant, when the complete Historical engine shows an exact-zero Balance and required rate evidence is currently available, then Manage may show Archive with factual eligibility text, but does not persist/cache eligibility or authorize mutation; nonzero or rate-blocked states expose no bypass.
2. Given Archive is activated, when its confirmation page opens, then it names the Participant and Group, states that archive is reversible, removes the identity from new allocations, and preserves history; it carries only an allow-listed Manage return and stable invoker focus ID. Cancel returns to the exact Archive control without calculation or mutation.
3. Given protected Confirm is submitted, when exactly one archive attempt dispatches, then a new immutable all-time snapshot/generation, UTC context, and Historical quote bundle are captured after dispatch; provider I/O occurs after the snapshot transaction and write gate are released. Confirmation display state never authorizes the mutation.
4. Given required Historical evidence is unavailable/ineligible or the target's final quantized Group Currency Balance is nonzero, when eligibility is evaluated, then no archive commits, no partial Balance is exposed, and a sanitized no-change/retryable response returns focus to the invoking Archive control or scoped status. The dispatched submission token remains terminal and retry requires a newly rendered confirmation/token.
5. Given the final target Balance is exactly zero, when final admission executes under the shared write gate, then one transaction revalidates active Group, target ownership and active/unarchived lifecycle, the captured ledger generation, current UTC date, and every quote's eligibility before changing only `participants.is_archived`. Any mismatch returns retryable feedback and commits nothing.
6. Given another ledger mutation commits after snapshot capture but before archive admission, or UTC rolls over, or refreshable evidence expires, when final revalidation occurs, then archive is rejected without refetching/substituting quote evidence. A subsequent explicit attempt must capture a new context.
7. Given archive commits, then all Participant identity, memberships, Spendings, Payers, Shares, historical summaries, Balances, and Settlement Transfers remain stored and resolvable; no rate evidence, Balance, Transfer, repayment, or eligibility result is persisted. Manage redirects with `303`, focuses the Participants heading, and announces once.
8. Given the Group is archived, CSRF/form/token validation fails, or the archive token is replayed, then archived Group requests return pre-use-case `409`, all other pre-dispatch rejections invoke no use case, and a replay dispatches at most once. Post-dispatch work has no generic cancellation timeout.
9. Given Manage and confirmation render with long names, Balance, eligibility text, and actions at 320px/400% zoom, then Historical Balance precedes eligibility and equal actions, every control remains at least 48 by 48 CSS pixels, coral archive styling is paired with explicit text, no clipping/page-level horizontal scroll occurs, and native/enhanced paths remain equivalent.

Requirements: `SPEC-FR30`, `SPEC-FR37..SPEC-FR39`; `SPEC-NFR3`, `SPEC-NFR10`, `SPEC-NFR13`, `SPEC-NFR15..SPEC-NFR16`, `SPEC-NFR22`, `SPEC-NFR25`, `SPEC-NFR31..SPEC-NFR34`; UX contracts `UX-SHELL-01`, `UX-TARGET-01`, `UX-FOCUS-01`, `UX-STATUS-01`, `UX-CONFIRM-01`, `UX-RESPONSIVE-01`, `UX-VISUAL-01`.

## Scope Boundary

- Implement only zero-Balance Participant archival. Story 5.5 owns the separate archived-Participants list and unconditional restore flow.
- Reuse Story 5.1's Historical snapshot/rate/balance engine. Do not add a parallel calculator, provider, cache, snapshot query, or settlement-based eligibility path.
- Correct the currently split epoch/write-gate ownership only as necessary to make archive admission race-safe. The generation that archive captures and validates must be owned by the same `SqliteLedgerRuntime` gate that serializes all ledger commits and must advance only after an authoritative commit.
- Do not use or expose `ParticipantUseCases::set_archived` / `ParticipantRepository::set_participant_archived` as the archive path. Replace/remove superseded unscoped archive semantics rather than retaining an unsafe compatibility path.
- Do not add migrations, SQLx metadata, dependencies, global Participant routes, optimistic revisions, stale-edit UX, rate-evidence persistence, repayment/payment state, manual retry, custom JavaScript, custom HTMX extensions, inline script attributes, or a generic post-dispatch timeout. Refresh `.sqlx` only if checked SQL changes.
- `specs/design.md` already specifies the behavior. Do not alter it unless implementation proves an actual contract divergence; then update it first and synchronize all affected artifacts.

## Tasks / Subtasks

- [x] Establish one safe archive-attempt contract and shared Historical calculation seam (AC: 1, 3-6)
  - [x] Extract reusable, application-owned Historical snapshot/quote/balance work from `DebtService` without changing its complete-or-no-result Debts behavior. The archival result must contain only the immutable capture necessary for final admission: complete `LedgerSnapshot`, gate-owned generation, UTC instant/date, ordered Historical `RateQuote` bundle, target currency, and final quantized balances.
  - [x] Keep `DebtService::calculate` as the sole Debts orchestration path, including `simplify` after balance quantization. Archive eligibility examines the target's final quantized `BTreeMap<EntityId, Decimal>` entry only; it never uses raw positions, Transfers, or page-rendered values.
  - [x] Add narrow application-owned ports/types for gated snapshot-plus-generation capture and final archive admission. Outer SQLx/runtime types must not enter `debtor-application` or `debtor-domain`.
  - [x] Validate the complete snapshot before provider calls: Group ID/ownership, unique Participant identities, complete Spending aggregates, and allocation references. Retain deterministic context sorting, four-request bound, same-currency synthetic `1` quote, exact Decimal arithmetic, no partial results, and no provider I/O under a database transaction/gate.
  - [x] Revalidate captured quotes without refetching or replacing them. Fixed past Historical evidence remains eligible under its exact context; refreshable current/future evidence uses the existing inclusive seven-UTC-day rule. A changed UTC date rejects the attempt.

- [x] Move mutation generation into the shared SQLite runtime and implement atomic archive admission (AC: 3-7)
  - [x] Extend `SqliteLedgerRuntime` / `SqliteLedgerStore` so the same process-local write gate governs both the mutation generation and every ledger commit. Remove reliance on root `DispatchedMutationRegistry::committed_epoch` for archival eligibility; it is currently advanced after the repository returns and can race a database commit.
  - [x] Make all existing successful ledger writes advance the shared generation immediately after their authoritative SQLite commit while still under the gate. Failed, rejected, or timed-out-before-transaction writes must not advance it. Preserve the root registry only for dispatched-mutation lifecycle/shutdown tracking unless it can be cleanly simplified without changing those guarantees.
  - [x] Add a group-scoped archive operation which, during final gated transaction, checks Group active state, target Participant's owning Group, active membership, unarchived identity state, captured generation, captured UTC date, and every captured quote's eligibility. Commit only `participants.is_archived = 1` and its normal update timestamp; never change membership activity or historical allocation rows.
  - [x] Map nonzero final balance to a safe conflict/no-change result; map generation/date/quote invalidation and unavailable evidence to a safe retryable result; preserve `NotFound`/Group archived distinctions and sanitized storage failures. Do not disclose IDs, amounts, SQL, provider diagnostics, or URLs.
  - [x] Extend `RootGroupMutationExecutor` with the archive operation so dispatch registration, definitive committed/rolled-back publication, shutdown waiting, and no post-dispatch cancellation remain exactly consistent with existing Group/Spending mutations.

- [x] Add Group-scoped Manage eligibility and confirmation flow (AC: 1-4, 7-9)
  - [x] Extend `MemberRow`/Manage projection with display-ready Historical Balance, archive eligibility state/copy, and a stable archive-control ID. Evaluate eligibility only for active Manage Participants; historical calculations must still include inactive/archived identities. Do not make Summary, Transactions, or Spending-form rendering perform this provider calculation.
  - [x] Show each active Participant in this order: identity, existing editable fields, Historical Balance, factual eligibility/rate-unavailable copy, then equal Edit/Archive actions. Expose the Archive link only for an exact-zero, currently available Manage calculation. Preserve existing add/edit retained-value behavior and the settings -> Participants -> Group lifecycle order.
  - [x] Add `GET`/`POST /groups/{group_id}/participants/{participant_id}/archive` in the existing `memberships` handler module and router. Use `require_writable_group` before form parsing/reservation on POST. Verify active, owned, unarchived target for GET; return no global Participant surface.
  - [x] Reuse `ConfirmTemplate`, strict `parse_lifecycle_form`, `CsrfValidatedForm::reserve_and_dispatch`, and existing server-owned confirmation/session patterns. Bind only group ID, participant ID, fixed Manage return/focus ID, and submission token to the session; never bind or trust displayed Balance, eligibility, snapshot, epoch, or quote evidence.
  - [x] Confirmation must say the archive is reversible, removes the identity from new allocations, and preserves history. Cancel returns to `#participant-{id}-archive` and focuses that valid control. On success redirect to canonical Manage with Participants heading focus and one status announcement. On post-dispatch failure, return safe focused status/control feedback without implying a background result.
  - [x] Keep valid native forms authoritative. HTMX may enhance the same routes through the pinned response-targets pattern only; preserve identical content/status/focus semantics, one-shot pending initiator behavior, strict no-cache/security headers, and no custom JavaScript.

- [x] Test at the owning layer and retain existing regressions (AC: all)
  - [x] Application tests with fixed injected clock and fakes: exact zero accepts final admission; positive/negative final quantized balance refuses; same-currency zero uses no provider call; missing/ineligible/expired quote and checked arithmetic fail with no partial result; archived/inactive identities remain in the calculation but target admission requires active/unarchived ownership; snapshot completes before provider work.
  - [x] Application/infra coordination tests: capture generation under the gate; a Spending/group/participant mutation between capture and final admission rejects archive; UTC rollover and quote expiry reject without refetch; all final conditions are rechecked; no evidence is persisted. Use barriers, notifications, or held gates only, never timing sleeps.
  - [x] Infrastructure tests with `#[sqlx::test]`/temporary file databases as appropriate: successful archive changes only `participants.is_archived`; referenced history and snapshot decoding remain intact; new allocation eligibility excludes archived identities; cross-Group/missing/inactive/already-archived target and archived/missing Group reject atomically; concurrent archive attempts yield one commit and one safe failure; gate contention starts no transaction; generation advances only after commit.
  - [x] Web/router/template tests: eligible/nonzero/rate-blocked Manage projection; exact confirmation wording and allow-listed Cancel target; Confirm starts a fresh server attempt; successful `303`, Participants-heading focus, and one announcement; terminal replay token; archived Group/CSRF/missing/duplicate/unknown fields/token paths have no dispatch; failure focus/status and no partial balance output; native/enhanced parity; 48px targets, focus outline, long-name/money wrapping, and no horizontal page scroll.
  - [x] Preserve all Story 5.1-5.3 Historical/Current Debts, rate/cache, snapshot, settlement, strict CSRF/token, root smoke, Group/Spending mutation lifecycle, and archived-history tests. Do not bundle the deferred pre-existing `simplify.rs` saturating-arithmetic item.

- [x] Run focused and full validation (AC: all)
  - [x] `cargo fmt --all -- --check`
  - [x] `cargo check --workspace --all-features --locked`
  - [x] `SQLX_OFFLINE=true cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`
  - [x] `cargo test --workspace --all-features --locked`
  - [x] `cargo run --bin architecture-check --locked`
  - [x] If checked SQL/migrations change: migrate a temporary SQLite database, run `SQLX_OFFLINE=false DATABASE_URL=sqlite:///tmp/debtor-sqlx.db?mode=rwc cargo sqlx prepare --workspace --check`, and commit refreshed `.sqlx` metadata.
  - [x] Never use `cargo build --release`.

### Review Findings

- [x] [Review][Patch] Provisional future rates can authorize an exact Historical archive [`debtor-application/src/debts.rs:372`] — resolved: archive admission now rejects provisional quote evidence.
- [x] [Review][Patch] Final UTC check occurs before the archive transaction [`debtor-infra/src/db/repos/participants.rs:126`] — resolved: final admission checks occur within the guarded transaction.
- [x] [Review][Patch] Archive failures lose the required retry focus [`debtor-web/src/handlers/memberships.rs:179`] — resolved: safe failure redirects return to the invoking Archive control without reusing the terminal token.
- [x] [Review][Patch] Nonzero balance is mislabeled as unavailable rates [`debtor-web/src/handlers/spending_views.rs:345`] — resolved: Manage projects separate eligible, nonzero, and rate-unavailable states.
- [x] [Review][Patch] Manage action order and equal-action layout violate the contract [`debtor-web/templates/group.html:118`] — resolved: balance and eligibility precede a responsive equal-action container.
- [x] [Review][Patch] Unreachable duplicate debt calculation remains suppressed [`debtor-application/src/debts.rs:421`] — resolved: removed the duplicate body and unreachable-code suppression.

## Dev Notes

### Developer Context

The completed Debt stack already has the correct financial calculation seam. `DebtService::calculate` loads one complete `LedgerSnapshot`, releases its SQLite transaction, gets deterministic Historical/Current quotes, accumulates exact `Decimal` balances, calls `quantize_balances`, then calls `simplify`. Archive must reuse the Historical calculation mechanics up through final quantized balances, not make a second calculator and not use Settlement as payment evidence.

The critical defect to address is architectural, not cosmetic. `SqliteLedgerRuntime` presently owns the five-second write gate, but `DispatchedMutationRegistry` in root owns `committed_epoch` and advances it only after a use case returns. That gap permits a database mutation to commit after archive capture but before root publishes the epoch. Archive cannot read this root epoch as its generation source. Place the archive generation with the same runtime/gate that serializes commits and validate it under final archive transaction admission.

The existing `set_participant_archived(id, archived)` SQL update is unscoped and checks neither Group, lifecycle, balance, quote evidence, nor generation. It is a superseded unsafe seam. Do not call it from web, application, or composition for archive.

### Required Architecture Compliance

- Preserve `root -> debtor-web/debtor-infra -> debtor-application -> debtor-domain`. Domain remains synchronous/deterministic; application owns use-case policy and narrow ports; infra owns SQLx/gate/transactions; web owns authenticated strict HTTP/form/rendering; root composes and supervises definitive mutation lifetime.
- Use `rust_decimal::Decimal` and checked Rust arithmetic only. Never add floats, SQL monetary aggregation, SQL numeric conversion, rounding-to-pass, zero substitution, `unwrap`/`expect` in production paths, or partial financial output.
- Treat quote bundle evidence as immutable application data. Provider calls occur after snapshot transaction/gate release, and final quote revalidation must not issue a new provider request or mutate cache/evidence.
- Keep output-affecting inputs deterministic with `BTreeMap`/explicit sorting and Participant-ID tie breaks. Preserve existing Historical context keys, synthetic quote behavior, cache/single-flight bounds, and provider completion-order independence.
- Preserve history. Archive does not delete Participant, Group membership, Spendings, Payers, or Shares; it simply prevents this active identity from becoming a new allocation candidate. Historical projections continue showing the current name plus visible `Archived` text.
- Keep all failure messages/statuses safe and factual. Do not log or render raw SQLx/provider details, values, identifiers, session/CSRF/token data, URLs, query strings, or credentials.
- Preserve root mutation registration and shutdown guarantees: dispatch only after token reservation, publish committed/rolled-back outcome synchronously before response work, and never use a generic timeout/cancellation after dispatch.

### Existing Files To Update And Preserve

| Path | Required change and preservation rule |
| --- | --- |
| `debtor-application/src/debts.rs` | Factor the existing Historical snapshot/quote/balance mechanics into a reusable archive-capable seam. Preserve `DebtService` behavior, complete-or-no-result boundary, one snapshot before provider I/O, Context ordering, four-call bound, final quantization, and settlement order. |
| `debtor-application/src/participants.rs` | Replace the generic/unscoped archive operation with explicit Group-scoped archival policy/input and narrow ports/use case. Preserve active participant create/edit behavior; restore is Story 5.5. |
| `debtor-application/src/errors.rs` and exports | Add typed, sanitized archive admission reason(s) only when existing safe taxonomy cannot accurately distinguish nonzero/conflict from retryable invalidation/unavailable evidence. |
| `debtor-application/src/groups.rs` | Extend `GroupMutationExecutor` for the archive use case, not a second executor or web-side mutation. |
| `debtor-infra/src/db/repos.rs` | Make `SqliteLedgerRuntime` co-own the gated generation and expose only needed application-facing adapter capabilities. Preserve five-second gate semantics and safe SQLite diagnostics. |
| `debtor-infra/src/db/repos/snapshots.rs` | Reuse complete snapshot materialization; add gated snapshot/generation capture only as needed. Never hold transaction/gate during provider I/O. |
| `debtor-infra/src/db/repos/participants.rs` | Replace unguarded archive update with final group-scoped transactional admission, preserving canonical decoding and all existing Participant writes. |
| `src/composition.rs` and `src/runtime.rs` | Wire the archive service/ports through the one `RootGroupMutationExecutor`; retain process lifecycle registration/shutdown. Remove root epoch dependency for archive rather than introducing parallel generation owners. |
| `debtor-web/src/handlers/memberships.rs` | Add archive GET/POST alongside Group-scoped Participant editing. Preserve auth, writable-group guard, strict form parsing, token reservation order, and safe response mapping. |
| `debtor-web/src/handlers/spending_views.rs` | Build Manage-only display eligibility from the shared Historical seam. Preserve other section projections and active allocation filtering. |
| `debtor-web/src/handlers/groups.rs`, `debtor-web/src/router.rs`, `debtor-web/src/session.rs` | Reuse Group confirmation/focus conventions; add only allow-listed archive binding/state and Group-scoped routes. |
| `debtor-web/src/state.rs`, `debtor-web/src/templates.rs`, `debtor-web/templates/group.html`, `debtor-web/templates/confirm.html`, `static/css/app.css` | Add typed row state, confirmation/Manage markup, minimal responsive styling, stable focus/status nodes, native/HTMX parity, and no custom script. Preserve settings/Participants/lifecycle order and existing editorial tokens. |
| `debtor-web/src/handlers/test_support.rs`, adapter tests, router tests | Extend fakes and regression coverage using fixed clocks and synchronization primitives; do not add a mocking framework. |

### UX Requirements

- Use the existing dark Editorial Contrast visual language: charcoal, warm paper text, rules, serif section headings, square controls, yellow standard actions, and coral archive emphasis paired with words. Do not add cards, pills, animations, gradients, or hover lift.
- In active Manage, Participant block order is identity -> editable fields -> Historical Balance -> eligibility copy -> equal actions. A rate-blocked message is factual: `Rates are unavailable, so archive was not attempted. Reopen Manage to retry.` A nonzero message states that archive requires an exactly zero Historical Balance.
- Archive is a full server-rendered confirmation page, not a modal. It identifies the object and reversible effect. Cancel is a valid native link to the stable initiating control; Confirm is a protected native form whose first activation is unavailable/pending.
- Controls, links, inputs, and actions remain at least 48px in both dimensions with 2px high-contrast focus outlines. Long Group/Participant names and money wrap safely at 320px/400% zoom without page-level horizontal scrolling.
- Status nodes are stable, polite, atomic, and scoped. Maintain the exception for the existing CSS-only HTMX Debts Updating behavior; this archive flow must not introduce dynamic client state or ARIA mutation through custom JavaScript.

### Library And Framework Requirements

- Preserve pinned Rust 1.97.1, Axum 0.8.9, Askama 0.16.0, Tokio 1.53.1, SQLx 0.9.0, reqwest 0.13.4, `rust_decimal` 1.42.1, HTMX 2.0.10, and response-targets 2.0.4. Add no dependency.
- SQLx checked queries run inside `Transaction<'_, Sqlite>` through `&mut *tx`; inspect conditional-update `rows_affected()` before commit and roll back/reject on a failed final condition. SQLx transactions commit/roll back explicitly. [Source: Context7 `/websites/rs_sqlx`, consulted 2026-08-20]
- Keep Askama typed and escaped. Templates present server-projected data only; no eligibility, financial, SQL, rate, or lifecycle policy belongs in templates or handlers.
- Before changing any library API, reconsult its current Context7 documentation as required by the project contract.

### Previous Story Intelligence

- Story 5.1 established snapshot-owned Participant projection, zero-identity seeding, exact payer-minus-share aggregation, signed largest-remainder quantization, rate disclosure, and a complete-or-no-result Debts boundary. Extend these mechanics; do not add an archive-specific identity query/calculator.
- Story 5.2 established Current-mode orchestration and strict stale-evidence boundaries. Archive uses Historical only, but must preserve the same immutable quote discipline and must not change rate cache/provider semantics.
- Story 5.3 proved Settlement follows quantization and remains advisory-only. Archive eligibility is a zero final Balance check, not evidence that any Transfer was paid. Its deferred `saturating_sub` review item is unrelated and must stay out of this scope.
- Recent commits `15f4b48`, `de757ad`, and `0306289` deliberately kept migrations, SQLx metadata, Cargo manifests, snapshot persistence, rate adapter/cache, and composition unchanged. Story 5.4 changes the runtime/archive seams only because immutable archival admission requires it.

### Anti-Patterns To Avoid

- Do not archive from a Manage-page balance, confirmation GET result, session-stored eligibility, a regular `DebtService::calculate` result without gated generation capture, raw/partial/pre-quantized balance, Settlement Transfer, or an unscoped `UPDATE`.
- Do not hold the SQLite transaction or write gate over provider I/O, persist quote/balance/transfer/eligibility evidence, refetch/substitute quotes at final admission, or claim an archive attempt is valid after context drift.
- Do not introduce a database revision column, a second gate/epoch owner, optimistic stale-edit conflict UI, global Participant management, Participant deletion, membership removal, automatic retry, manual Retry button, or Story 5.5 restore/list behavior.
- Do not use floats, SQL monetary operations, unordered outputs, panic/default-zero paths, raw error output/logging, custom JavaScript, inline handlers, custom HTMX extensions, CSP relaxation, or post-dispatch generic cancellation.

### References

- [Source: `specs/design.md#Accounting And History`]
- [Source: `specs/design.md#Rates And Settlements`]
- [Source: `_bmad-output/project-context.md#Critical Implementation Rules`]
- [Source: `_bmad-output/planning-artifacts/epics.md#Story 5.4: Archive a Zero-Balance Participant Safely`]
- [Source: `_bmad-output/planning-artifacts/epics.md#Epic 5: Calculate Debts, Settle, and Safely Retire Identities`]
- [Source: `_bmad-output/planning-artifacts/prds/prd-debtor-2026-08-10/prd.md#Group-owned Participants`]
- [Source: `_bmad-output/planning-artifacts/prds/prd-debtor-2026-08-10/addendum.md#SQLite Integrity And Write Semantics`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-6 - Single ledger runtime and mutation epoch`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-8 - Epoch-guarded participant archival`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-10 - Shared unsafe-request admission boundary`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/DESIGN.md#Layout & Spacing`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md#Group and Participant Lifecycle`]
- [Source: `debtor-application/src/debts.rs`]
- [Source: `debtor-application/src/participants.rs`]
- [Source: `debtor-infra/src/db/repos.rs`]
- [Source: `debtor-infra/src/db/repos/participants.rs`]
- [Source: `debtor-infra/src/db/repos/snapshots.rs`]
- [Source: `src/composition.rs`]
- [Source: `src/runtime.rs`]
- [Source: `debtor-web/src/handlers/memberships.rs`]
- [Source: `debtor-web/src/handlers/spending_views.rs`]
- [Source: `debtor-web/templates/group.html`]
- [Source: Context7 `/websites/rs_sqlx`, consulted 2026-08-20]

## Dev Agent Record

### Agent Model Used

openai/gpt-5.6-terra

### Debug Log References

- Resolved `bmad-create-story` customization: no activation prepend/append or completion terminal steps configured.
- Loaded full sprint state, persistent project context, normative design contract, Epic 5 context, PRD/addendum, architecture spine, UX contracts, Story 5.3 intelligence, deferred-work ledger, current participant/debt/runtime/web code, and recent relevant commits.
- Used parallel planning and codebase research. Consulted SQLx transaction semantics through Context7.
- Identified the current root-owned mutation epoch as unsafe for archive admission because it is not guarded by the SQLite write gate and advances after repository return.
- Implemented the scoped Participant archive route, protected confirmation, exact-zero Historical precheck, conditional Group/member lifecycle update, and Manage eligibility presentation.
- Fresh full workspace validation passed: formatting, workspace check, strict offline Clippy, all workspace tests, and architecture fitness.
- Work remains in progress: the archive attempt does not yet capture/revalidate a generation owned by the SQLite write gate or an immutable quote bundle at final admission. Tasks remain unchecked to preserve the required accounting/history invariant.
- Resumed implementation added gated snapshot-generation capture, immutable quote admission checks, UTC validation, and post-commit generation advancement across ledger mutations.
- Final validation passed: format, check, strict Clippy, 42 application tests, 36 domain tests, 36 infra unit tests, 43 infra integration tests, 116 web tests, root tests/smoke, architecture fitness, and SQLx metadata verification.

### Completion Notes List

- Ultimate context engine analysis completed - comprehensive developer guide created.
- Implemented the native Group-scoped confirmation and archive baseline, but did not mark the story complete because final immutable-context revalidation is still absent.
- Completed immutable archive admission and full regression validation; story is ready for code review.
- Resolved all six code-review findings and revalidated the full workspace.

### File List

- `_bmad-output/implementation-artifacts/5-4-archive-a-zero-balance-participant-safely.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `.sqlx/query-7755206ea234ca62fd64cd958507ae336e1105299c673544dbb8062d5a25e58e.json`
- `.sqlx/query-cf3f5c0211b80b66304b0ddea5ea64e25193b1f0b1351b7ca57c7127017ef394.json` (removed)
- `debtor-application/src/groups.rs`
- `debtor-application/src/participants.rs`
- `debtor-application/src/debts.rs`
- `debtor-infra/src/db/repos.rs`
- `debtor-infra/src/db/repos/groups.rs`
- `debtor-infra/src/db/repos/participants.rs`
- `debtor-infra/src/db/repos/snapshots.rs`
- `debtor-infra/src/db/repos/spendings.rs`
- `debtor-infra/tests/repos.rs`
- `debtor-web/src/handlers.rs`
- `debtor-web/src/handlers/groups.rs`
- `debtor-web/src/handlers/memberships.rs`
- `debtor-web/src/handlers/spending_views.rs`
- `debtor-web/src/handlers/test_support.rs`
- `debtor-web/src/router.rs`
- `debtor-web/src/templates.rs`
- `debtor-web/templates/group.html`
- `src/composition.rs`
- `static/css/app.css`

### Change Log

- 2026-08-20: Created comprehensive implementation context; status set to ready-for-dev.
- 2026-08-20: Began scoped Participant archive implementation; retained in-progress status pending immutable generation/quote admission.
- 2026-08-20: Completed gate-owned immutable archive admission, regression coverage, and full validation; status set to review.
- 2026-08-20: Resolved six code-review findings; status set to done.
