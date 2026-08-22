---
stepsCompleted:
  - step-01-validate-prerequisites
  - step-02-design-epics
  - step-03-create-stories
  - step-04-final-validation
validationStatus: pending-revalidation-2026-08-22
inputDocuments:
  - _bmad-output/specs/spec-debtor/SPEC.md
  - _bmad-output/specs/spec-debtor/glossary.md
  - _bmad-output/planning-artifacts/prds/prd-debtor-2026-08-10/prd.md
  - _bmad-output/planning-artifacts/prds/prd-debtor-2026-08-10/addendum.md
  - _bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md
  - _bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/DESIGN.md
  - _bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md
  - specs/design.md
  - _bmad-output/project-context.md
  - _bmad-output/planning-artifacts/sprint-change-proposal-2026-08-12.md
---

# debtor - Epic Breakdown

## Overview

This document provides the complete epic and story breakdown for debtor, decomposing the canonical SPEC and all declared companions into implementable stories.

## Requirements Inventory

### Decomposed Functional Requirements (`SPEC-FR*`)

SPEC-FR1: Debtor provides a private ledger operated by exactly one administrator authenticated with one configured password and without usernames, registration, participant login, tenants, or multi-user authorization.

SPEC-FR2: Debtor refuses startup before database connection or migration when `APP_ADMIN_PASSWORD_HASH` is missing, exceeds 256 encoded bytes, or is not a canonical bounded Argon2id v19 PHC hash with valid `m`, `t`, and `p` parameters.

SPEC-FR3: Debtor requires a session-backed CSRF token to render and submit the login workflow.

SPEC-FR4: Debtor authenticates the administrator only after the submitted password is successfully verified against the configured password hash.

SPEC-FR5: Successful login rotates the session identifier and CSRF token, durably establishes the authenticated session before setting a cookie or redirecting, and emits no authenticated cookie if persistence fails.

SPEC-FR6: Anonymous login sessions expire after ten minutes of inactivity and are limited to 4,096 live sessions without evicting authenticated sessions.

SPEC-FR7: Authenticated sessions expire after 30 days of inactivity, refresh on every request, and are limited to 32 live sessions without eviction.

SPEC-FR8: A correct login attempted while authenticated-session capacity is full flushes its anonymous session and returns retryable `503 Service Unavailable`.

SPEC-FR9: Restarting Debtor invalidates every anonymous and authenticated session.

SPEC-FR10: Logging out flushes the current session and revokes its authenticated access.

SPEC-FR11: Session cookies are HTTP-only and `SameSite=Strict`, with secure cookies required outside debug/local operation.

SPEC-FR12: Login permits at most five post-CSRF password-verification attempts per trusted client IP in any rolling five-minute window.

SPEC-FR13: An unseen login client receives retryable `429 Too Many Requests` when the 4,096-client login-limiter capacity is full.

SPEC-FR14: Anonymous users are denied access to all ledger pages and ledger mutations.

SPEC-FR15: Every unsafe request, including login, requires exactly one valid session-backed synchronizer CSRF token; missing, duplicate, malformed, or incorrect tokens are rejected before route parsing, password verification, or use-case dispatch.

SPEC-FR16: Every rendered unsafe form carries a bounded, expiring, session-bound, single-use submission token distinct from its CSRF token.

SPEC-FR17: Anonymous submission tokens are limited to one per session, 4,096 total, and ten minutes of inactivity; authenticated tokens are limited to 32 per session, 1,024 total, and a 30-minute absolute lifetime.

SPEC-FR18: Missing, unknown, expired, reserved, or consumed submission tokens return `409 Conflict` without invoking the requested use case.

SPEC-FR19: Validation rejected before dispatch preserves the submission token, while a token reserved immediately before dispatch remains consumed after exactly one attempt regardless of commit, rollback, task failure, or response delivery.

SPEC-FR20: The administrator can create a Group by supplying a trimmed, non-empty name of at most 100 Unicode characters.

SPEC-FR21: A newly created Group has `USD` as its initial Group Currency and opens in its Manage section.

SPEC-FR22: An established Group opens in its Summary section.

SPEC-FR23: The administrator can edit a Group name and freely change its Group Currency among `USD`, `EUR`, `RUB`, `KGS`, `TRY`, `KZT`, `UZS`, `CNY`, `KRW`, `JPY`, `OMR`, and `TJS`.

SPEC-FR24: The administrator can archive and restore Groups.

SPEC-FR25: Active Group lists exclude archived Groups, which remain accessible through a separate contextual archived view.

SPEC-FR26: Archived Groups remain readable but hide all mutation and settings controls except the permitted Group restore action.

SPEC-FR27: Direct form or mutation requests against an archived Group, other than the permitted Group restore route, return `409 Conflict` without invoking a use case.

SPEC-FR28: A Group with no Spendings can be deleted together with its unreferenced Group-owned Participants.

SPEC-FR29: A Group containing any Spending cannot be deleted and can only be archived.

SPEC-FR30: The administrator can add, edit, archive, and restore Participants within their owning Group.

SPEC-FR31: Every Participant belongs to exactly one Group, is never reusable across Groups, and is not exposed through a global Participant-management surface.

SPEC-FR32: Participant names are trimmed, non-empty, and at most 100 Unicode characters.

SPEC-FR33: Participant colors use normalized `#RRGGBB` values.

SPEC-FR34: A new Participant form suggests a varied valid color while allowing the administrator to choose another color.

SPEC-FR35: Active Participant lists exclude archived Participants, which remain available through contextual archived views.

SPEC-FR36: Participants cannot be independently deleted through the application.

SPEC-FR37: A Participant can be archived only when a complete all-time Historical-mode calculation gives that Participant an exact zero Balance in Group Currency.

SPEC-FR38: Participant archival commits only if the ledger, UTC calculation date, and rate eligibility remain unchanged throughout the archival attempt.

SPEC-FR39: Missing or ineligible exchange-rate evidence blocks Participant archival with retryable feedback and no state change.

SPEC-FR40: Restoring a Participant does not require a Balance or exchange-rate eligibility check.

SPEC-FR41: Archived Participants remain included wherever their historical Spendings affect history, summaries, Balances, or Settlement Transfers.

SPEC-FR42: Historical views display a Participant's current name after that Participant is renamed.

SPEC-FR43: The administrator can create, inspect, edit, and delete a dated Spending within a Group.

SPEC-FR44: Each Spending contains a trimmed, non-empty description of at most 200 Unicode characters, one supported category, one Source Currency, one positive Total, exactly one Payer, and one or more Participant Shares.

SPEC-FR45: Supported Spending categories are `food`, `transport`, `housing`, `fun`, `shopping`, `bills`, `health`, and `other`.

SPEC-FR46: Spending dates parse strictly as `YYYY-MM-DD` and are on or after `2025-01-01`.

SPEC-FR47: Spending Totals, Payer amounts, and Share amounts are positive and no greater than `999_999_999_999`.

SPEC-FR48: Monetary input permits zero minor units for JPY and KRW, three for OMR, and two for every other supported currency, rejecting excess precision rather than rounding it.

SPEC-FR49: Exactly one active Participant owned by the Spending's Group is selected as Payer and pays exactly the Spending Total.

SPEC-FR50: Spending Shares are non-empty, Participant-unique, positive, owned by the Spending's Group, and sum exactly to the Total in Source Currency minor units.

SPEC-FR51: Spending allocation offers only Proportional and Exact Share modes.

SPEC-FR52: A new Spending form starts with an empty description and Total, the current UTC date, Source Currency equal to Group Currency, no category, no Payer, and every active Participant selected for sharing.

SPEC-FR53: Selecting a Payer on a new Spending initially assigns that Payer the full Total paid.

SPEC-FR54: Proportional mode initially assigns weight `1` to every active Participant and permits Participants to be deselected.

SPEC-FR55: Every selected Proportional weight is a positive decimal no greater than `1,000,000` with at most six fractional digits.

SPEC-FR56: Proportional allocation deterministically distributes exact minor units by submitted weight, assigns residual units by descending remainder with ascending Participant ID as the tie-breaker, and uses identical results for Preview and commit.

SPEC-FR57: Exact mode initially divides Total minor units equally among all active Participants and assigns residual units in ascending Participant ID order.

SPEC-FR58: Exact mode permits Participant deselection and Share editing and displays the remaining or excess difference until selected Shares equal the Total exactly.

SPEC-FR59: Allocation Preview displays each resulting exact Source Currency amount and produces no aggregate when normalization or checked allocation is invalid.

SPEC-FR60: Editing an existing Spending opens in Exact mode with its stored Payer and Share amounts because allocation mode and Proportional weights are not persisted.

SPEC-FR61: A Spending update may retain an archived Participant only in the same existing Payer or Share role and rejects introducing that Participant into a new role or changing the existing role.

SPEC-FR62: Every Spending mutation either commits the complete Spending and all allocations or leaves the ledger unchanged.

SPEC-FR63: Among valid concurrently admitted mutations, the last committed write determines the resulting ledger state without a stale-edit conflict.

SPEC-FR64: The persistent Add Spending action opens a focused full-page form and, after successful commit, returns to Transactions with the committed row visible.

SPEC-FR65: Ordinary Spending history uses fixed 25-item pages ordered newest first by Spending date and then descending Spending ID.

SPEC-FR66: Spending detail remains readable for archived Groups and archived Participants.

SPEC-FR67: The selected Group Summary shows exact Spending totals for the current UTC calendar month only.

SPEC-FR68: The current-month source summary shows the Group Total and each Payer's paid Total grouped by original Source Currency without requiring exchange-rate conversion.

SPEC-FR69: The current-month converted summary converts every included Spending from Source Currency to Group Currency using the rate context for that Spending's date.

SPEC-FR70: Converted current-month values accumulate exactly per Payer before final target-currency quantization, and the displayed Group Total equals the exact sum of all displayed Payer totals.

SPEC-FR71: Final converted Payer totals are quantized together by truncation toward zero and descending fractional remainder, with ascending Participant ID as the tie-breaker.

SPEC-FR72: If any required quote is unavailable or checked conversion, aggregation, or quantization fails, the entire converted summary is withheld behind one sanitized unavailable warning while source totals and ordinary ledger CRUD remain available.

SPEC-FR73: Debtor uses an exact synthetic rate of `1` without provider access when Source Currency equals Group Currency and discloses that rate.

SPEC-FR74: Historical rate requests use the Spending date by default, while future Spending dates use the latest current rate and are marked provisional.

SPEC-FR75: Current conversion mode uses the UTC calculation date for every Spending and is not persisted.

SPEC-FR76: On rate-provider failure, Debtor may use the latest context-matching eligible prior quote and identifies it as stale.

SPEC-FR77: A fixed past-date quote remains stale-eligible without an age limit, while current and future quotes remain eligible only through seven UTC calendar days after their effective fetch date.

SPEC-FR78: The administrator can calculate every Participant's all-time Balance in Group Currency using either Historical mode or Current mode.

SPEC-FR79: Historical Balance mode is the default and converts each Spending using its Spending-date context.

SPEC-FR80: Balance results include archived historical identities, are deterministic, are quantized to Group Currency precision, and sum to exactly zero.

SPEC-FR81: The debts view discloses the selected conversion mode, UTC calculation time, target Group Currency, every unique rate used, and all stale or provisional warnings.

SPEC-FR82: If any required rate is unavailable, the debts view returns retryable `503 Service Unavailable` and exposes no partial Balances or Settlement Transfers.

SPEC-FR83: If debt conversion, aggregation, quantization, or settlement fails, Debtor returns one sanitized calculation failure without substituting zero, panicking, or exposing partial Balances or Transfers.

SPEC-FR84: Debtor derives advisory Settlement Transfers on demand from all-time Balances without recording repayments, paid state, settlement checkpoints, or transfer completion.

SPEC-FR85: Settlement Transfers are deterministic, positive, pair-unique, and sufficient to settle every included Participant Balance.

SPEC-FR86: Settlement orders matching by descending absolute Balance and then Participant ID and produces at most `n - 1` Transfers for `n` included Participants without claiming global transfer-count minimality.

SPEC-FR87: Group, Participant, and Spending validation failures return `422 Unprocessable Entity`, display inline field errors, and retain every raw submitted value, including Participant color.

SPEC-FR88: Successful mutations return `303 See Other` redirects.

SPEC-FR89: Forms reject malformed, missing, duplicate, and unknown fields before route-specific parsing or use-case dispatch.

SPEC-FR90: Every core interaction works through native links and forms without HTMX, while HTMX may progressively enhance those same full-page paths.

SPEC-FR91: Enhanced expected `4xx` and `5xx` responses target a stable, programmatically announced status region.

SPEC-FR92: The interface uses server-rendered semantic HTML and remains operable on current stable Chrome, Firefox, Safari, and Edge down to 320 CSS pixels.

SPEC-FR93: Every control is keyboard-operable, programmatically labelled, and visibly focused with an indicator at least two CSS pixels thick and at least 3:1 contrast.

SPEC-FR94: Normal text has at least 4.5:1 contrast, large text and meaningful controls or graphics at least 3:1 contrast, and inline errors are programmatically associated with their fields.

SPEC-FR95: Login and authenticated HTML responses prevent caching and send `nosniff`, `no-referrer`, and the prescribed restrictive content-security policy.

SPEC-FR96: Probe and static-asset routes neither create nor load sessions.

SPEC-FR97: `/healthz` reports process liveness independently of ledger contents, exchange-rate availability, and ordinary user-traffic saturation.

SPEC-FR98: `/readyz` checks SQLite and mandatory in-process cleanup supervisors but does not depend on exchange-rate-provider availability or ledger contents.

SPEC-FR99: Failure of mandatory session-expiry or submission-token cleanup fails readiness, stops new admission, and initiates shutdown.

SPEC-FR100: Login request bodies are limited to 8 KiB and other form request bodies to 256 KiB.

SPEC-FR101: Debtor admits at most 64 concurrent user requests and four concurrent login requests while reserving a separate four-request budget for health and readiness probes.

SPEC-FR102: Safe dynamic reads other than debts and login time out after 30 seconds, debts after 90 seconds, and probes after two seconds with a one-second SQLite readiness limit.

SPEC-FR103: Ledger mutations have a 30-second pre-dispatch deadline, but once dispatched continue until a definitive commit or rollback result is known rather than being cancelled by a generic request timeout.

SPEC-FR104: Graceful shutdown stops admission, drains HTTP for at most ten seconds, then waits for every dispatched mutation to finish before closing ledger storage.

SPEC-FR105: After valid local configuration is supplied, `cargo run` creates or connects to the database, runs migrations, binds the configured address, reports a non-secret local URL, and supports graceful shutdown without Docker, a frontend build, manual migration, SQLx preparation, or exchange-rate-provider availability.

### Decomposed Non-Functional Requirements (`SPEC-NFR*`)

SPEC-NFR1: Login bodies are limited to 8 KiB and other form bodies to 256 KiB; at most 64 user requests and four login requests run concurrently, with a separate four-request probe budget.

SPEC-NFR2: Safe dynamic reads other than Debts and login have a 30-second timeout, Debts has a 90-second timeout, probes have a two-second outer timeout, and SQLite readiness has a one-second inner timeout.

SPEC-NFR3: A 30-second absolute pre-dispatch deadline covers body extraction, authentication, CSRF, and asynchronous web prechecks; after dispatch, no generic application or edge timeout may cancel the mutation.

SPEC-NFR4: Exchange-rate requests have a five-second connect timeout, 20-second total timeout, and 64 KiB response limit; at most four provider calls run globally or per debt calculation, identical uncached keys use single-flight, and each cache class holds at most 4,096 deterministic-LRU entries.

SPEC-NFR5: All money and rates use exact `Decimal` and canonical SQLite `TEXT`; checked Rust owns parsing, conversion, validation, aggregation, quantization, and formatting, while floating point, lossy conversion, SQL monetary work, silent rounding, zero substitution, and partial results are forbidden.

SPEC-NFR6: Every Total and persisted Payer or Share amount is positive, precision-valid, and at most `999_999_999_999`; JPY/KRW allow zero minor units, OMR three, and all other supported currencies two.

SPEC-NFR7: Names and descriptions obey their Unicode limits, dates are strict and use UTC policy, colors are normalized, ledger IDs are positive `i64`, and UUIDs are restricted to session and CSRF randomness.

SPEC-NFR8: Exactly one active Group-owned Participant pays the Total, and nonempty, unique, positive Shares conserve that Total exactly.

SPEC-NFR9: Proportional and Exact allocation use checked integer/minor-unit arithmetic with the specified bounds, residual ordering, Participant-ID ties, and exact closure; Preview and commit use the same operation.

SPEC-NFR10: Domain behavior and all output-affecting ordering are synchronous and deterministic, with Participant ID as the final tie-breaker and provider completion order unable to alter results or disclosures.

SPEC-NFR11: Converted monthly values accumulate exactly without per-Spending rounding, Payer totals quantize together, and the displayed Group total is their exact sum; failure withholds the whole converted section.

SPEC-NFR12: Balances quantize with largest signed remainder and preserve exact zero sum; Settlement is deterministic, positive, complete, pair-unique, and bounded by `n - 1` Transfers.

SPEC-NFR13: Rate lookup, deduplication, caching, fallback, freshness, same-currency synthesis, and disclosure preserve the full `(source, target, R, F)` context and exact UTC eligibility rules.

SPEC-NFR14: Repository decoding revalidates canonical monetary form and rejects malformed or noncanonical stored values as corruption rather than normalizing them.

SPEC-NFR15: Complete Spending writes and all eligibility checks are transactional; complete aggregates come from one SQLite snapshot, and provider requests never hold a database transaction.

SPEC-NFR16: Referenced identities survive archival, and Participant archival is admitted only from an unchanged immutable all-time Historical context with an exact zero Group Currency Balance.

SPEC-NFR17: `APP_ADMIN_PASSWORD_HASH` is required, at most 256 encoded bytes, canonical Argon2id v19 with exactly bounded `m`, `t`, and `p`, a 16-64 byte decoded salt, and a 32-64 byte output; cheap validation precedes KDF and database work.

SPEC-NFR18: Password verification concurrency is two; login allows five attempts per trusted client IP in five rolling minutes, tracks at most 4,096 active keys without eviction, and fails closed at capacity.

SPEC-NFR19: Sessions are process-local, in-memory, server-side, HTTP-only, `SameSite=Strict`, securely cookie-bound outside debug, rotated durably on login, flushed on logout, and invalidated by restart.

SPEC-NFR20: Anonymous sessions expire after ten inactive minutes and cap at 4,096 without authenticated eviction; authenticated sessions expire after 30 inactive days, refresh per request, and cap at 32 without eviction.

SPEC-NFR21: Every unsafe request requires exactly one correct session-backed CSRF synchronizer token before password verification, route parsing, or dispatch.

SPEC-NFR22: Separate bounded anonymous/authenticated submission-token stores enforce expiry, per-session limits, fail-closed capacity, atomic pre-dispatch reservation, and terminal single use.

SPEC-NFR23: Login and authenticated HTML send no-store, nosniff, no-referrer, and the prescribed restrictive CSP; approved scripts use fixed routes, media types, immutable digest mappings, and nosniff.

SPEC-NFR24: Forwarding headers are trusted only from configured immediate proxy CIDRs in one selected format; production validates a nonempty policy before admission and resolves client identity identically across edge protocols.

SPEC-NFR25: Logs and user-facing errors exclude all credentials, session/security identifiers, client identity, SQL/database/provider diagnostics, monetary values, entity identifiers, URLs, query strings, and request-derived data; SQLite logs use only the fixed allowlists.

SPEC-NFR26: Exchange-rate-provider availability never gates startup, readiness, or ledger CRUD, and financial failures produce no partial converted summaries, Balances, or Transfers.

SPEC-NFR27: Health reports liveness; readiness checks only SQLite and mandatory supervisors; cleanup-supervisor failure fails readiness, stops admission, and initiates shutdown.

SPEC-NFR28: Every control supports pointer-independent operation, programmatic labels, a two-CSS-pixel 3:1 focus indicator, required text/component contrast, and programmatically associated inline errors.

SPEC-NFR29: The current stable Chrome, Firefox, Safari, and Edge are supported down to 320 CSS pixels, and every core interaction retains a native full-page path.

SPEC-NFR30: The UI uses semantic server-rendered Askama HTML and vanilla CSS. Pinned self-hosted HTMX core and its pinned official `response-targets` extension are the only currently approved browser-side JavaScript infrastructure. Manually authored application JavaScript, inline scripts and event handlers, custom HTMX extensions, application-owned HTMX event handlers, client-side financial state, and features requiring imperative post-swap behavior are forbidden. Other official extensions require explicit design and security approval before addition.

SPEC-NFR31: Shutdown stops admission, drains HTTP for at most ten seconds, then waits for all dispatched mutations before bounded checkpoint and pool close while preserving authoritative mutation outcomes and treating unknown outcomes as fatal.

SPEC-NFR32: Structured safe error categories prevent raw adapter diagnostics from crossing inward or reaching HTTP; checked financial failures never panic, substitute zero, or return partial output.

SPEC-NFR33: Unsafe Rust is forbidden, production avoids `unwrap` and `expect`, formatting remains clean, Clippy pedantic warnings are denied, and lint suppression remains narrow.

SPEC-NFR34: Public APIs have rustdoc and fallible methods document `# Errors`; comments explain non-obvious constraints, changes remain minimal, and speculative abstractions or mocking frameworks are avoided.

### Additional Requirements

- Preserve the permanent single-Administrator model: Participants are Group-owned accounting identities, never users, memberships, tenants, authenticated principals, or reusable global identities.
- Keep v1 to Groups, Participants, Spending CRUD, current UTC-month summaries, all-time Balances, and advisory Settlement Transfers; retain every explicit SPEC non-goal.
- Support exactly the twelve currencies and eight categories enumerated by the contract.
- Preserve inward dependency direction `debtor (root) -> debtor-web / debtor-infra -> debtor-application -> debtor-domain`.
- `debtor-domain` owns synchronous deterministic business and financial rules without I/O or framework dependencies.
- `debtor-application` owns use cases, lifecycle and Spending input policy, authentication orchestration, and narrow mockable ports.
- `debtor-infra` owns SQLx, HTTP clients, cryptography, rate caching, persistence, and concrete external adapters.
- `debtor-web` owns Axum, trusted-proxy resolution, strict forms, CSRF/session mechanics, cookies, Askama rendering, view models, and HTTP mapping; handlers contain no financial, SQL, network, or cryptographic logic.
- The root crate owns configuration, concrete composition, migrations, startup ordering, process lifecycle, mutation execution, server startup, and shutdown.
- Axum, SQLx, reqwest, Argon2, tower-sessions, and other outer types must not cross application-owned ports.
- External effects and clocks are constructor-injected; use cases run with fakes without Axum, SQLite, network, or wall clock.
- Transport adapters decode structure and preserve raw text; application `*Input` values parse and validate all domain fields and construct financial allocations.
- Pin Rust 1.97.1, edition 2024, MSRV 1.97, Cargo resolver 3, and the minimal rustfmt/clippy toolchain profile.
- Preserve all crate versions pinned in the adopted project context and lockfiles, use `--locked`, and consult current crate documentation before API changes.
- Keep the production workspace and `tools/password-hash` as independent Cargo workspaces.
- Support one Debtor process and one private local SQLite volume behind a sanitizing HTTPS reverse proxy; multiple instances and external writers are unsupported.
- Enable SQLite WAL, `synchronous=FULL`, foreign keys, a five-second busy timeout, and one process-local five-second ledger write gate whose timeout starts no transaction or guarded side effect.
- Use SQLite constraints for supported codes, flags, bounded non-empty text, color/date shape, relationships, and referenced-Group deletion, but not Unicode trimming or monetary arithmetic.
- Use compile-time checked SQLx macros; the fixed WAL-checkpoint PRAGMA is the sole unchecked exception, and committed `.sqlx` metadata must remain current.
- Among admitted valid mutations, last committed write wins; do not add optimistic revisions or stale-edit conflicts.
- Restrict referenced Group deletion, delete only history-free Groups with unreferenced owned Participants, and never independently cascade-delete Participants through the application.
- Require active owned Participants for new allocations; updates may retain archived Participants only in unchanged existing Payer or Share roles.
- Group creation accepts only name, persists USD, and opens Manage; established Groups open Summary; active lists exclude archived identities and provide contextual archived views.
- Preserve the specified Spending-form defaults, one allocation table, native Preview, Exact-on-edit behavior, persistent Add action, and return to the committed Transactions row.
- Use fixed 25-item `(spent_date DESC, id DESC)` keyset history pages and direct complete-aggregate loads for detail, edit, and delete.
- Use one shared strict form/CSRF/submission-token extractor; return `422` with retained raw input for validation, `303` after successful mutation, and pre-dispatch `409` for archived or invalid-token mutation routes.
- Probe and static routes, including pinned HTMX assets, neither create nor load sessions.
- Decode provider JSON numbers lexically and directly into arbitrary-precision `Decimal`; preserve requested/effective dates, immutable quote bundles, disclosure, and non-persisted rate evidence.
- The Debts view discloses mode, calculation time, target currency, unique rates, and stale/provisional warnings.
- Domain financial tests cover examples, boundaries, and properties for exactness, checked arithmetic, allocation, quantization, deterministic ordering, conservation, and Settlement completeness.
- Application tests use injected clocks and simple fakes without Axum, SQLite, network, or wall clock.
- Infrastructure and web adapter tests cover malformed/oversized input, persistence corruption, safe errors, cache/LRU/single-flight/concurrency, capacity limits, sessions, strict forms, CSRF, and dispatch exclusion.
- Use `#[sqlx::test]` and temporary file databases for WAL, locking, multi-connection behavior, migrations, and constraints.
- Coordinate concurrency tests with barriers, notifications, or deliberately held locks, never timing sleeps.
- Web tests verify statuses, headers, redirects, retained values, malformed/duplicate/unknown-field and CSRF rejection, and no pre-use-case dispatch; retain a real-socket root smoke test.
- Validate with `cargo fmt --all -- --check`, `cargo check --workspace --all-features --locked`, offline Clippy with denied warnings, and `cargo test --workspace --all-features --locked`.
- Run `cargo run --bin architecture-check --locked`; run `cargo deny check` when dependency policy changes.
- After checked SQL or migration changes, migrate a temporary database, run online `cargo sqlx prepare --workspace --check`, and commit refreshed `.sqlx` metadata.
- Validate the independent password helper with its manifest-path fmt, Clippy, and test commands.
- Never use `cargo build --release` for routine validation; use debug `cargo check`, `cargo test`, and `cargo run`.
- After `.env` and a valid password hash are supplied, `cargo run` performs full local startup and graceful shutdown without Docker, frontend build, manual migration, SQLx generation, or Frankfurter availability.
- The edge owns TLS, certificates, HTTP/3/QUIC, `Alt-Svc`, and HTTP/2 or HTTP/1.1 fallback; Debtor remains a private HTTP/1.1 backend.
- The edge sanitizes forwarding headers, disables early data or returns `425` for unsafe early-data requests, reuses backend connections, preserves admitted mutation completion, and enforces matching body limits.
- Roll HTTP/3 out with a short `Alt-Svc` lifetime and verify UDP reachability, TCP fallback, unsafe early-data rejection, and identical client identity before extending it.
- Follow the prescribed plural module and `*Reader`/`*Repository`/`*Provider`/`*UseCases`, `*Service`/`*Store`/`*Client`/`*Gate`, `*Input`, `Db*`, and rendering-projection naming conventions.
- Update normative `specs/design.md` before behavior, synchronize every affected companion artifact in the same change, and stop rather than interpret divergence.
- Later ADRs explicitly identify superseded decisions and synchronize `specs/design.md`.
- Before first deployment, prefer clean breaking APIs, routes, configuration, migrations, and schemas over compatibility shims while preserving security, accounting, and historical integrity.

### UX Design Requirements

Final `DESIGN.md` and `EXPERIENCE.md` contracts bind every affected web story through the stable UX registry. Story acceptance criteria repeat applicable testable consequences; broad cross-cutting policy does not substitute for route-specific proof. `specs/design.md` and accepted ADRs remain upstream authority; the UX contracts govern visual and interaction details within that envelope under AD-18.

| UX contract | Story owners |
|---|---|
| `UX-SHELL-01` | 1.4-1.6, 2.1-2.5, 3.1, 3.4, 4.1-4.3, 5.1-5.5 |
| `UX-TARGET-01` | 1.4-1.7, 2.1-2.5, 3.1-3.5, 4.1-4.3, 5.1-5.5 |
| `UX-ALLOC-01` | 3.1-3.2 and 3.4 |
| `UX-FOCUS-01` | 1.4-1.7, 2.1-2.5, 3.1-3.5, 4.1-4.3, 5.1-5.5 |
| `UX-STATUS-01` | 1.4-1.7, 2.1-2.5, 3.1-3.5, 4.1-4.3, 5.1-5.5 |
| `UX-PREVIEW-NATIVE-01` | 3.1-3.2 and 3.4 |
| `UX-PREVIEW-LATEST-01` | 3.1-3.2 and 3.4 |
| `UX-CONFIRM-01` | 2.5, 3.5, and 5.4 |
| `UX-RESPONSIVE-01` | 1.4-1.7, 2.1-2.5, 3.1-3.5, 4.1-4.3, 5.1-5.5 |
| `UX-VISUAL-01` | 1.4-1.7, 2.1-2.5, 3.1-3.5, 4.1-4.3, 5.1-5.5 |

### Source-Qualified Requirement Crosswalk

Identifier namespaces are artifact-qualified and must never be shortened in planning, implementation, review, or test evidence:

- `PRD-FR-1` through `PRD-FR-12` refer only to product requirements in `prd.md`.
- `SPEC-FR1` through `SPEC-FR105` and `SPEC-NFR1` through `SPEC-NFR34` refer only to the decomposed implementation requirements in this document.
- PRD non-functional requirements are cited by their named source section because the PRD does not assign them identifiers.
- `UX-*` refers only to the stable registry in final `EXPERIENCE.md`; architecture decisions retain their `AD-*` namespace.

| Product source | Decomposed requirements | Architecture / UX source | Epic and story acceptance owners |
|---|---|---|---|
| `PRD-FR-1` Password-gated access | `SPEC-FR1..SPEC-FR19`, `SPEC-FR90..SPEC-FR105` | AD-10 through AD-18; `UX-SHELL-01`, `UX-TARGET-01`, `UX-FOCUS-01`, `UX-STATUS-01`, `UX-RESPONSIVE-01`, `UX-VISUAL-01` | Epic 1, Stories 1.1-1.9; first real mutation evidence completes in Story 2.1 |
| `PRD-FR-2` Group lifecycle | `SPEC-FR20..SPEC-FR29` | AD-4 through AD-7, AD-10, AD-11, AD-18; `UX-SHELL-01`, `UX-FOCUS-01`, `UX-CONFIRM-01` | Epic 2, Stories 2.1-2.2 and 2.5; Spending-backed deletion proof completes in Story 3.1 |
| `PRD-FR-3` Group-owned Participants | `SPEC-FR30..SPEC-FR42` | AD-4 through AD-8, AD-18; `UX-SHELL-01`, `UX-FOCUS-01`, `UX-CONFIRM-01` | Epics 2 and 5, Stories 2.3-2.4 and 5.4-5.5; historical-name projection is proved in Story 3.3 |
| `PRD-FR-4` Record a Spending | `SPEC-FR43..SPEC-FR59`, `SPEC-FR64` | AD-3 through AD-7, AD-10, AD-11, AD-18; `UX-ALLOC-01`, `UX-PREVIEW-NATIVE-01`, `UX-PREVIEW-LATEST-01`, `UX-RESPONSIVE-01` | Epic 3, Stories 3.1-3.2 |
| `PRD-FR-5` Exact allocation | `SPEC-FR47..SPEC-FR61` | AD-3, AD-5, AD-10, AD-18; `UX-ALLOC-01`, `UX-PREVIEW-NATIVE-01`, `UX-PREVIEW-LATEST-01` | Epic 3, Stories 3.1-3.2 and 3.4 |
| `PRD-FR-6` Review and maintain history | `SPEC-FR42..SPEC-FR43`, `SPEC-FR60..SPEC-FR66` | AD-4 through AD-7, AD-18; `UX-FOCUS-01`, `UX-CONFIRM-01` | Epic 3, Stories 3.3-3.5 |
| `PRD-FR-7` Source Currency summary | `SPEC-FR67..SPEC-FR68` | AD-3, AD-7, AD-18; `UX-SHELL-01`, `UX-STATUS-01` | Epic 4, Story 4.1 |
| `PRD-FR-8` Group Currency summary | `SPEC-FR69..SPEC-FR77` | AD-3, AD-7, AD-9, AD-18; `UX-STATUS-01`, `UX-RESPONSIVE-01` | Epic 4, Stories 4.2-4.3 |
| `PRD-FR-9` Select conversion mode | `SPEC-FR75`, `SPEC-FR78..SPEC-FR79` | AD-7, AD-9, AD-18; `UX-FOCUS-01`, `UX-STATUS-01` | Epic 5, Stories 5.1-5.2 |
| `PRD-FR-10` Exact Balances | `SPEC-FR78..SPEC-FR83` | AD-3, AD-7 through AD-9, AD-18; `UX-STATUS-01` | Epic 5, Stories 5.1-5.2 |
| `PRD-FR-11` Deterministic Settlement Transfers | `SPEC-FR84..SPEC-FR86` | AD-3, AD-9, AD-18; `UX-STATUS-01` | Epic 5, Story 5.3 |
| `PRD-FR-12` Calculation disclosure and failure isolation | `SPEC-FR72..SPEC-FR83` | AD-9, AD-15, AD-18; `UX-STATUS-01` | Epics 4 and 5, Stories 4.2-4.3 and 5.1-5.2 |

PRD non-functional source clauses map as follows: `UX Acceptance Requirements` to `SPEC-FR90..SPEC-FR94`, `SPEC-NFR28..SPEC-NFR30`, AD-11, AD-18, and the UX registry; `Correctness And Historical Integrity` to `SPEC-NFR5..SPEC-NFR16` and AD-3 through AD-9; `Security And Privacy` to `SPEC-NFR17..SPEC-NFR25` and AD-10 through AD-15; `Availability And Bounded Operation` to `SPEC-NFR1..SPEC-NFR4`, `SPEC-NFR26..SPEC-NFR27`, `SPEC-NFR31..SPEC-NFR32`, and AD-12 through AD-16. Each story's `Requirements` line names its decomposed `SPEC-*` obligations; this crosswalk supplies the corresponding product, architecture, and UX provenance.

### SPEC-FR Coverage Map

SPEC-FR1: Epic 1 - Operate one private single-Administrator ledger.
SPEC-FR2: Epic 1 - Validate the configured password hash before persistence startup.
SPEC-FR3: Epic 1 - Establish the CSRF-protected login flow.
SPEC-FR4: Epic 1 - Verify the configured administrator password.
SPEC-FR5: Epic 1 - Durably rotate and promote authenticated sessions.
SPEC-FR6: Epic 1 - Bound and expire anonymous sessions.
SPEC-FR7: Epic 1 - Bound, refresh, and expire authenticated sessions.
SPEC-FR8: Epic 1 - Fail authenticated promotion safely at capacity.
SPEC-FR9: Epic 1 - Invalidate sessions on restart.
SPEC-FR10: Epic 1 - Flush authentication on logout.
SPEC-FR11: Epic 1 - Apply the required cookie policy.
SPEC-FR12: Epic 1 - Rate-limit password verification per trusted client.
SPEC-FR13: Epic 1 - Fail closed when limiter-key capacity is full.
SPEC-FR14: Epic 1 - Require authentication for all ledger access.
SPEC-FR15: Epic 1 - Enforce strict CSRF before unsafe processing.
SPEC-FR16: Epic 1 - Issue distinct single-use submission tokens.
SPEC-FR17: Epic 1 - Bound and expire both submission-token pools.
SPEC-FR18: Epic 1 - Reject invalid or reused submission tokens without dispatch.
SPEC-FR19: Epic 1 - Reserve tokens exactly at dispatch and consume attempts terminally.
SPEC-FR20: Epic 2 - Create valid Groups.
SPEC-FR21: Epic 2 - Default new Groups to USD and Manage.
SPEC-FR22: Epic 2 - Open established Groups in Summary.
SPEC-FR23: Epic 2 - Edit Group names and supported Group Currency.
SPEC-FR24: Epic 2 - Archive and restore Groups.
SPEC-FR25: Epic 2 - Separate active and archived Group views.
SPEC-FR26: Epic 2 - Keep archived Groups readable and immutable.
SPEC-FR27: Epic 2 - Reject direct archived-Group mutations before dispatch.
SPEC-FR28: Epic 2 - Delete history-free Groups with unreferenced Participants.
SPEC-FR29: Epics 2 and 3 - Define history-aware Group deletion, then structurally verify restriction with the first Spending persistence in Story 3.1.
SPEC-FR30: Epics 2 and 5 - Add/edit/restore Participants with Group management; complete zero-Balance archival with all-time debts.
SPEC-FR31: Epic 2 - Enforce Group-owned, non-user Participant identity.
SPEC-FR32: Epic 2 - Validate Participant names.
SPEC-FR33: Epic 2 - Normalize Participant colors.
SPEC-FR34: Epic 2 - Suggest varied Participant colors while permitting selection.
SPEC-FR35: Epic 5 - Separate active and archived Participant views with the archive/restore capability.
SPEC-FR36: Epic 2 - Prohibit independent Participant deletion.
SPEC-FR37: Epic 5 - Require exact zero Historical Balance for Participant archival.
SPEC-FR38: Epic 5 - Revalidate ledger, UTC date, and quote eligibility before archive commit.
SPEC-FR39: Epic 5 - Block archival safely when rate evidence is unavailable.
SPEC-FR40: Epic 5 - Restore Participants without Balance or rate checks after archival is available.
SPEC-FR41: Epics 4 and 5 - Preserve archived identities in current-month summaries and all-time financial outputs.
SPEC-FR42: Story 3.3 - Resolve current Participant names in historical Spending views.
SPEC-FR43: Stories 3.1-3.5 - Create, inspect, edit, and delete Spendings.
SPEC-FR44: Epic 3 - Capture every required Spending field and allocation.
SPEC-FR45: Epic 3 - Restrict Spending categories to the supported set.
SPEC-FR46: Epic 3 - Validate strict bounded Spending dates.
SPEC-FR47: Epic 3 - Enforce positive bounded monetary amounts.
SPEC-FR48: Epic 3 - Validate currency-specific minor-unit precision without rounding.
SPEC-FR49: Epic 3 - Require one active Group-owned Payer for the Total.
SPEC-FR50: Epic 3 - Require unique positive exact-conserving Shares.
SPEC-FR51: Epic 3 - Offer only Proportional and Exact Share modes.
SPEC-FR52: Epic 3 - Apply new-Spending form defaults.
SPEC-FR53: Epic 3 - Assign the full paid Total when selecting a Payer.
SPEC-FR54: Epic 3 - Initialize and edit Proportional selections.
SPEC-FR55: Epic 3 - Validate Proportional weights.
SPEC-FR56: Epic 3 - Allocate Proportional Shares deterministically and identically for Preview/commit.
SPEC-FR57: Epic 3 - Initialize Exact Shares with deterministic residual assignment.
SPEC-FR58: Epic 3 - Edit Exact Shares against a displayed closing difference.
SPEC-FR59: Epic 3 - Preview exact allocations and reject invalid normalization.
SPEC-FR60: Story 3.4 - Open existing Spendings in Exact mode.
SPEC-FR61: Story 3.4 - Preserve but never introduce archived allocation roles on update.
SPEC-FR62: Stories 3.1, 3.4, and 3.5 - Commit, replace, and delete complete Spending aggregates atomically.
SPEC-FR63: Story 3.4 - Apply last-committed-write semantics.
SPEC-FR64: Stories 3.1-3.2 - Provide focused Add Spending and return to the committed row.
SPEC-FR65: Story 3.3 - Browse fixed keyset-paginated Spending history.
SPEC-FR66: Stories 3.3-3.4 - Read Spending detail for archived identities and Groups.
SPEC-FR67: Epic 4 - Show current UTC-month Spending totals.
SPEC-FR68: Epic 4 - Show exact source-currency Group and Payer totals independently of rates.
SPEC-FR69: Epic 4 - Convert current-month totals with Spending-date contexts.
SPEC-FR70: Epic 4 - Accumulate converted values exactly and conserve the displayed Group total.
SPEC-FR71: Epic 4 - Quantize final Payer totals together deterministically.
SPEC-FR72: Epic 4 - Withhold the whole converted section safely on quote or calculation failure.
SPEC-FR73: Epic 4 - Synthesize and disclose same-currency rates without provider calls.
SPEC-FR74: Epic 4 - Apply Historical and provisional future-date rate contexts.
SPEC-FR75: Epic 5 - Support non-persisted current conversion mode for all-time debts.
SPEC-FR76: Epics 4 and 5 - Use and disclose context-matching stale quotes for Historical/future and Current modes.
SPEC-FR77: Epics 4 and 5 - Enforce fixed-past and refreshable stale-eligibility windows in every conversion mode.
SPEC-FR78: Epic 5 - Calculate all-time Participant Balances in Historical or Current mode.
SPEC-FR79: Epic 5 - Default Balances to Spending-date Historical conversion.
SPEC-FR80: Epic 5 - Produce deterministic target-precision exact-zero-sum Balances.
SPEC-FR81: Epic 5 - Disclose debt calculation context, rates, and warnings.
SPEC-FR82: Epic 5 - Return retryable unavailability without partial debts.
SPEC-FR83: Epic 5 - Sanitize checked calculation failures without partial output.
SPEC-FR84: Epic 5 - Derive advisory Transfers without repayment state.
SPEC-FR85: Epic 5 - Produce positive, pair-unique, complete deterministic Transfers.
SPEC-FR86: Epic 5 - Use bounded deterministic greedy Settlement ordering.
SPEC-FR87: Epic 2 - Establish shared retained-value inline validation for Group and Participant forms; Epic 3 extends it to Spendings.
SPEC-FR88: Epic 2 - Establish successful mutation redirects for ledger forms.
SPEC-FR89: Epic 2 - Establish strict field extraction before ledger dispatch.
SPEC-FR90: Epic 1 - Establish native full-page interaction with optional HTMX enhancement.
SPEC-FR91: Epic 1 - Route enhanced failures to an announced status region.
SPEC-FR92: Epic 1 - Establish the semantic responsive browser-compatible shell.
SPEC-FR93: Epic 1 - Establish keyboard operation, labels, and visible focus.
SPEC-FR94: Epic 1 - Establish contrast and programmatic error association.
SPEC-FR95: Epic 1 - Apply no-store and mandatory browser security headers.
SPEC-FR96: Epic 1 - Keep probes and static assets session-free.
SPEC-FR97: Epic 1 - Expose independent process liveness.
SPEC-FR98: Epic 1 - Expose SQLite/supervisor readiness without provider coupling.
SPEC-FR99: Epic 1 - Treat cleanup-supervisor failure as fatal to readiness and admission.
SPEC-FR100: Epic 1 - Enforce route-specific request-body limits.
SPEC-FR101: Epic 1 - Separate and bound user, login, and probe admission.
SPEC-FR102: Epics 1 and 5 - Establish route timeout classes, then verify the 90-second class on the Debts route.
SPEC-FR103: Epics 1 and 2 - Establish bounded dispatch/outcome primitives, then integrate them with the first ledger mutation.
SPEC-FR104: Epics 1 and 2 - Establish graceful lifecycle coordination, then prove shutdown waits for real dispatched ledger mutations.
SPEC-FR105: Epic 1 - Run the complete local application with one command and no external build/provider prerequisite.

## Epic List

### Epic 1: Securely Operate and Access Debtor

The administrator can start a healthy local Debtor process, sign in and out securely, and use a resilient, accessible server-rendered shell whose unsafe actions are protected before any ledger capability is added.

**SPEC-FRs covered:** SPEC-FR1..SPEC-FR19, SPEC-FR90..SPEC-FR103, and SPEC-FR105; establishes the lifecycle primitive for SPEC-FR104, whose real-mutation acceptance evidence completes in Story 2.1

**Implementation notes:** Establishes workspace composition, the password helper contract, authentication/session/CSRF/submission-token foundations, the shared HTML shell, admission budgets, probes, timeout classes, and narrow mutation-outcome/lifecycle primitives exercised by the authenticated local application. Real ledger-mutation integration begins with Group mutation in Epic 2 rather than creating an unexercised infrastructure path.

### Epic 2: Organize Groups and Participants

The administrator can create and configure Groups, set up and maintain active Group-owned Participant identities, navigate archived Group contexts, and safely manage Group lifecycle changes that do not require debt calculation.

**SPEC-FRs covered:** SPEC-FR20..SPEC-FR28, SPEC-FR31..SPEC-FR34, SPEC-FR36, and SPEC-FR87..SPEC-FR89; completes real-ledger SPEC-FR103..SPEC-FR104 integration; SPEC-FR29 completes in Epic 3, while SPEC-FR30, SPEC-FR35, and SPEC-FR40 complete with Participant lifecycle in Epic 5

**Implementation notes:** Delivers Group-centered Manage and Summary navigation, strict retained-value forms, Group archive/restore/delete rules, active Participant add/edit, persistence integrity, and the first real use of definitive ledger-mutation dispatch/shutdown coordination. Participant archive, archived views, and restore are delivered together in Epic 5 because archival requires the all-time Historical Balance engine and immutable quote context.

### Epic 3: Record and Maintain Exact Spendings

The administrator can create, preview, inspect, correct, browse, and delete exact multi-currency Spendings with one Payer and deterministic Proportional or Exact Shares.

**SPEC-FRs covered:** SPEC-FR42..SPEC-FR66; completes structural SPEC-FR29 verification and extends SPEC-FR87..SPEC-FR89 to Spending forms

**Implementation notes:** Delivers exact domain allocation rules, application-owned raw input policy, transactional aggregate persistence, complete snapshot materialization, direct aggregate loads, fixed keyset history, archived-role update rules, and native/HTMX Preview parity. Participant-archival revalidation machinery is deferred until its first consumer in Epic 5.

### Epic 4: Understand Current-Month Spending

The administrator can answer the selected Group's exact current UTC-month totals by Source Currency and, when rate evidence permits, see conserved per-Payer and Group totals in Group Currency with complete stale/provisional disclosure.

**SPEC-FRs covered:** SPEC-FR67..SPEC-FR74 and Historical/future portions of SPEC-FR76..SPEC-FR77; shares SPEC-FR41 with Epic 5

**Implementation notes:** Source totals remain independently available. This epic establishes bounded lexical-Decimal provider access, complete requested/fetch/effective-date evidence for Historical and provisional-future contexts, context-keyed single-flight/LRU caching, deterministic final quantization, and whole-section degraded behavior. Current-mode orchestration and immutable archival-attempt bundles are deferred to their first consumers in Epic 5.

### Epic 5: Calculate Debts, Settle, and Safely Retire Identities

The administrator can calculate all-time Historical or Current Balances, receive complete advisory Settlement Transfers, and archive, browse, or restore Participants while retaining every referenced identity in history and admitting archive only from an unchanged exact-zero Historical context.

**SPEC-FRs covered:** SPEC-FR35, SPEC-FR37..SPEC-FR41, SPEC-FR75..SPEC-FR86; completes SPEC-FR30 and verifies the Debts-specific portion of SPEC-FR102

**Implementation notes:** Builds on complete Spending snapshots and the rate adapter from prior epics, adds exact-zero-sum quantization and deterministic Settlement, returns no partial financial output, and binds Participant archival to an immutable ledger/date/quote attempt with final transactional revalidation.

### Epic Validation Hotspots

Epic 1 is complete only through runnable startup and restart outcomes; Story 2.1 supplies the real-mutation lifecycle evidence. Epic 2 delivers active Participant management but does not complete the rate-dependent Participant archive and restore obligations owned by Epic 5. Every Epic 4 rate story must end in visible Administrator Summary value. Epic 5 must reuse the accepted Historical engine for Settlement and Participant archival rather than introduce parallel calculation paths. Global UX ownership is an index only; affected stories require route-specific `UX-*` acceptance evidence.

Coverage ownership does not imply completion: every shared requirement names the story that supplies its final evidence. Cross-cutting criteria are repeated only when applicable and specialized to the route, state, financial output, or persistence boundary introduced by that story. Infrastructure is never accepted as a standalone milestone; it completes only inside a runnable vertical outcome.

### Final-Evidence Ledger

The following shared requirements have partial enabling work before their final evidence. A story may record only the evidence it introduces; only the named final-evidence story may mark the requirement complete in sprint tracking.

| Requirement | Earlier enabling evidence | Final-evidence story | Completion status before implementation |
| --- | --- | --- | --- |
| `SPEC-FR29` | Story 2.5 defines the history-free delete/archive behavior | Story 3.1 proves the referenced-Group restriction with first Spending persistence | Introduced |
| `SPEC-FR30` | Stories 2.3-2.4 add and edit active Participants | Stories 5.4-5.5 prove archival and unconditional restore | Partially evidenced |
| `SPEC-FR40` | No earlier story claims restore eligibility | Story 5.5 proves direct restore without Balance or rate checks | Not started |
| `SPEC-FR103` | Story 1.5 establishes Login-token reservation; Story 1.7 extends the shared route-neutral boundary | Story 2.1 proves those primitives around a real ledger mutation | Partially evidenced |
| `SPEC-FR104` | Story 1.9 proves shutdown with no active mutation | Story 2.1 proves shutdown waits for a dispatched real ledger mutation | Partially evidenced |

### Assignment Packets

Each packet has one primary route/use case and must be split before assignment if its scoped work exceeds seven developer days or introduces unrelated work. Its checklist must name application/domain work, adapter work, route-specific UX evidence, tests, and validation commands; it cannot close a requirement ahead of the Final-Evidence Ledger.

| Story | One-developer packet boundary | Estimate |
| --- | --- | --- |
| 1.2 | Validated configuration, SQLite startup/migration, root composition, and provider-independent local socket admission. Restart, shutdown, architecture/dependency governance, and broad SQL verification remain with their existing owners. | 3-5 days |
| 1.4 | Anonymous Login session, CSRF, and one-per-session submission-token issuance with its bounded pool, expiry, and cleanup. | 3-5 days |
| 1.5 | Trusted-client resolution, strict Login admission, anonymous-token reservation, bounded Argon2 verification, limiter, and durable promotion. Extract proxy-policy/config validation or session internals only if this route packet exceeds estimate. | 4-6 days |
| 1.7 | Extend the established Login/Sign-out token path to authenticated page-scoped tokens and the route-neutral non-Login extractor; prove cross-route races and cleanup. Later routes reuse it and prove only route-specific prechecks. | 4-6 days |
| 2.1 | Create Group through a real dispatched ledger mutation and Manage redirect. It retains final evidence for `SPEC-FR103` and `SPEC-FR104`. | 4-6 days |
| 3.1 | Proportional Preview, reviewed approval, and the sole shared create-Spending aggregate path. If persistence/migration proof cannot fit, an explicit prerequisite packet lands with this story before Exact creation. | 5-7 days |
| 3.2 | Exact Preview, reviewed approval, and reuse of Story 3.1's create aggregate path. It must not reimplement shared persistence. | 3-5 days |
| 4.2 | Fresh/synthetic Historical conversion plus exact monthly Group-Currency projection and disclosure. Cache fallback/rollover remains in Story 4.3. | 5-7 days |
| 4.3 | Stable/refreshable cache admission, stale fallback, rollover, and whole-section unavailable behavior, reusing Story 4.2's projection. | 4-6 days |
| 5.1 | Historical snapshot, quote orchestration, exact zero-sum Balance calculation, and Debts result/disclosure. Current mode, Settlement, and archival admission remain later consumers. | 5-7 days |
| 5.4 | Confirmed zero-Balance Participant archive attempt with snapshot/epoch/date/quote revalidation and lifecycle commit, reusing Story 5.1's Historical engine. | 5-7 days |

Every assignment packet follows this ordered checklist:

1. Confirm its single route/use-case boundary, applicable `SPEC-*` requirements, Final-Evidence Ledger status, and explicitly excluded later-consumer work.
2. Implement the smallest required domain/application policy and injected port changes; retain or add fake-backed application tests.
3. Implement only the concrete adapter, persistence, migration, or runtime work consumed by that packet; add adapter tests and refresh SQLx metadata when applicable.
4. Implement the native web route and its required route-specific `UX-*` behavior; verify strict admission and no-dispatch rejection where the route is unsafe.
5. Add invariant-owning tests, then run the packet's workspace, architecture, and helper validation commands required by its changed areas.
6. Record requirement evidence without marking a shared requirement complete before its Final-Evidence Ledger owner; split and revalidate if a remaining item crosses the packet boundary.

Preserve the five approved epic boundaries unless revised-story validation demonstrates a forward dependency, a story that cannot fit one Developer context, detached route-specific UX evidence, premature shared-requirement completion, or parallel financial/lifecycle machinery. Only such concrete evidence reopens epic grouping.

The epic map does not override repository reality. Each story identifies retained, replaced, and removed brownfield paths, removes superseded behavior instead of preserving parallel scaffolds, and exercises shared primitives through the first runnable consumer. Mechanical coverage and owner indexes are necessary but do not replace criterion-level predecessor, route, state, and test-evidence validation.

### Cross-Cutting Story Rule

Primary FR ownership in the coverage map does not make cross-cutting behavior a one-time implementation. Every affected web story must repeat applicable strict-form, authentication, CSRF, submission-token, security-header, native-fallback, accessibility, responsive, safe-error, admission, timeout, and no-pre-dispatch-side-effect acceptance criteria. Every affected financial or persistence story must likewise repeat applicable exactness, determinism, checked-arithmetic, corruption-rejection, transaction, snapshot, diagnostic-safety, and concurrency criteria.

Each story must leave a runnable vertical increment and remove any superseded path rather than retaining parallel scaffolds. Introduce each infrastructure primitive in the first story that exercises it: Story 1.4 owns anonymous Login-token issuance, expiry, capacity, and cleanup; Story 1.5 owns Login-token reservation and terminal dispatch; Story 1.6 reuses that path for Sign out; Story 1.7 extends the established path to authenticated page-scoped tokens and the route-neutral extractor. Within Epic 4, source totals remain independently complete before converted totals are added. Within Epic 5, complete all-time calculation precedes Settlement and Participant archival consumers. A story cannot claim a cross-cutting FR complete until every route or financial output introduced by that story verifies the applicable behavior. Every SQL statement uses checked SQLx macros except the fixed WAL-checkpoint PRAGMA; whenever checked SQL or migrations change, migrate a temporary database, run online `cargo sqlx prepare --workspace --check`, and commit refreshed `.sqlx` metadata.

## Epic 1: Securely Operate and Access Debtor

The administrator can start a healthy local Debtor process, sign in and out securely, and use a resilient, accessible server-rendered shell whose unsafe actions are protected before ledger capabilities are added.

### Story 1.1: Prepare and Validate the Administrator Password

As the administrator,
I want to generate and validate the configured password hash before Debtor touches external state,
So that invalid credentials cannot start a partially initialized service.

**Acceptance Criteria:**

**Given** the independent `tools/password-hash` workspace is run through its protected input flow
**When** the administrator supplies a password
**Then** it emits a canonical Argon2id v19 PHC hash using exactly `m=19,456`, `t=2`, `p=1`, a 16-byte OS-generated salt, and a 32-byte output
**And** neither password nor generated hash is written to logs, fixtures, or committed files.

**Given** `APP_ADMIN_PASSWORD_HASH` is absent, exceeds 256 encoded bytes, is structurally noncanonical, is not Argon2id v19, or contains parameters other than exactly `m`, `t`, and `p`
**When** startup configuration is validated
**Then** validation fails before database connection, migration, socket admission, or password KDF work
**And** the sanitized error reveals no credential or hash content.

**Given** an Argon2id v19 PHC value is structurally canonical
**When** its bounded parameters are validated
**Then** memory must be `19,456..=65,536` KiB, iterations `2..=5`, parallelism `1..=4`, decoded salt length `16..=64` bytes, and output length `32..=64` bytes
**And** any out-of-range value is rejected before external side effects.

**Given** the helper implementation is complete
**When** its independent formatting, locked Clippy with warnings denied, and locked tests run through manifest-path commands
**Then** every check passes without folding the helper into the production workspace
**And** production code contains no unsafe Rust or credential-revealing diagnostics.

**Requirements:** SPEC-FR2; SPEC-NFR17, SPEC-NFR25, SPEC-NFR33..SPEC-NFR34; exact Argon2 bounds, helper independence, cheap validation, secret-safe diagnostics, and validation requirements. Brownfield disposition: retain the independent helper boundary, replace permissive startup validation, and remove credential-revealing paths without compatibility shims.

### Story 1.2: Start a Persistent Local Application

As the administrator,
I want one validated command to start a persistent local Debtor process,
So that I can reach my private ledger without external build or provider prerequisites.

**Acceptance Criteria:**

**Given** the repository is checked out with the pinned Rust 1.97.1 toolchain and lockfiles
**When** the production workspace is inspected
**Then** it uses edition 2024, MSRV 1.97, Cargo resolver 3, the minimal rustfmt/Clippy profile, and the root plus four required crates with inward-only manifest dependencies
**And** `tools/password-hash` remains independent and routine work never uses `cargo build --release`.

**Given** a local operator copies `.env.example` and supplies a Story 1.1 password hash
**When** configuration is loaded
**Then** every mandatory variable is documented without a secret and bare `cargo run` selects the application binary
**And** invalid required configuration fails before database connection, migration, or socket admission.

**Given** local configuration is valid
**When** `cargo run` starts Debtor
**Then** it creates or connects to the configured persistent SQLite database, runs only required migrations, enables foreign keys, WAL, `synchronous=FULL`, and a five-second busy timeout, composes concrete adapters behind application-owned ports, and binds the configured address
**And** it reports only a non-secret local URL including `http://`.

**Given** Frankfurter is unavailable and no Docker service, frontend build, manual migration, or SQLx metadata generation has run
**When** startup occurs
**Then** Debtor reaches socket admission
**And** provider availability is not consulted.

**Given** production manifests are resolved
**When** dependency versions are inspected
**Then** they retain the adopted pinned versions and features recorded by the architecture and project context
**And** lockfiles are preserved, validation uses `--locked`, and current crate documentation is consulted before framework API changes.

**Given** SQL or migrations required by this runnable slice change
**When** persistence work is validated
**Then** every SQL statement uses checked SQLx macros except the fixed WAL-checkpoint PRAGMA, temporary-database migration and online prepare checks pass, and refreshed `.sqlx` metadata is committed
**And** SQLite constraints stay structural and do not duplicate Unicode trimming or monetary arithmetic.

**Given** the current brownfield application contains retained and superseded startup paths
**When** this story completes
**Then** the existing crate direction, root composition, and checked-query workflow are retained; obsolete configuration, migration, and startup paths are replaced or removed
**And** no parallel compatibility startup remains.

**Requirements:** SPEC-FR105; SPEC-NFR25..SPEC-NFR26, SPEC-NFR33..SPEC-NFR34; startup, architecture, SQLite, local-run, toolchain, checked-SQLx, provider-independence, and brownfield-replacement requirements. No UX IDs apply because this story introduces no user-facing HTML contract.

### Story 1.3: Restart and Validate the Composed Local Application

As the administrator,
I want Debtor to stop cleanly and restart against the same initialized local database,
So that I can trust the composed application and its local ledger lifecycle.

**Acceptance Criteria:**

**Given** Story 1.2 started Debtor against a persistent local SQLite database and no ledger mutation is active
**When** shutdown is requested
**Then** the process stops admission, closes the HTTP server and SQLite resources in lifecycle order, attempts the bounded fixed WAL checkpoint, and exits without panic
**And** checkpoint failure preserves WAL sidecars and never represents an unknown storage outcome as rollback.

**Given** the process completed normal shutdown or preserved recoverable WAL sidecars
**When** `cargo run` starts again with the same valid configuration and database path
**Then** Debtor reconnects, applies no already-applied migration twice, retains the initialized database state, and reaches socket admission
**And** no manual migration, database recreation, SQLx generation, Docker service, frontend build, or provider availability is required.

**Given** a restart follows a failed checkpoint with intact SQLite sidecars
**When** SQLite opens the database under the configured WAL/`synchronous=FULL` policy
**Then** SQLite recovery produces a usable consistent database or startup fails safely before socket admission
**And** startup never deletes sidecars or silently recreates the ledger to hide recovery failure.

**Given** the composed workspace is validated
**When** `cargo fmt --all -- --check`, locked workspace check, offline Clippy with warnings denied, locked workspace tests, and `cargo run --bin architecture-check --locked` execute
**Then** every command passes and architecture fitness verifies every production package plus normal/build dependency direction
**And** production code contains no unsafe Rust or broad lint suppression.

**Given** dependency policy changed
**When** validation executes
**Then** `cargo deny check` passes for advisories, sources, and reviewed permissive licenses
**And** feature trimming and isolated dependency upgrades follow the adopted architecture policy.

**Given** checked SQL or migrations changed
**When** SQLx validation executes against a migrated temporary database
**Then** online `cargo sqlx prepare --workspace --check` passes and committed offline metadata matches
**And** the fixed WAL-checkpoint PRAGMA remains the sole verified unchecked query.

**Given** the brownfield runtime includes obsolete composition, shutdown, or validation paths
**When** this story completes
**Then** retained root ownership and lifecycle behavior are exercised by the restarted application, while superseded paths are removed
**And** no alternate startup/shutdown compatibility flow remains.

**Requirements:** SPEC-FR105; SPEC-NFR25..SPEC-NFR27, SPEC-NFR31..SPEC-NFR34; basic no-active-mutation shutdown, restart, WAL recovery, architecture fitness, SQLx metadata, dependency policy, complete validation, and brownfield-replacement requirements. Story 1.9 later proves authenticated-runtime shutdown, and Story 2.1 completes real-mutation `SPEC-FR103..SPEC-FR104` evidence. No UX IDs apply.

### Story 1.4: Open a Protected and Accessible Login Page

As the administrator,
I want to open a secure and accessible login page,
So that I can begin authentication without exposing credentials or creating unsafe browser state.

**Acceptance Criteria:**

**Given** an anonymous browser has no live session
**When** it requests `GET /login`
**Then** Debtor creates an anonymous server-side session with a ten-minute inactivity expiry, generates a session-backed CSRF token and one distinct single-use login submission token, explicitly saves the session before rendering, and emits the required cookie
**And** no password verification occurs.

**Given** an anonymous session already has a valid unexpired login token
**When** the login page is rendered again
**Then** valid anonymous activity refreshes the session and token ten-minute inactivity expiry, and the token store still holds at most one anonymous token for that session and at most 4,096 anonymous tokens globally
**And** anonymous token/session capacity cannot consume or evict authenticated capacity.

**Given** anonymous session or token capacity is full
**When** a new anonymous browser requests the login page
**Then** admission fails closed with sanitized retryable feedback and no partial session/token state
**And** no authenticated session is evicted.

**Given** the login page is rendered
**When** its HTML is inspected or operated without HTMX
**Then** it contains semantic server-rendered HTML, a programmatically labelled password field, exactly one CSRF token, exactly one submission token, and a valid native form action
**And** no username, registration, Participant-login, inline script, inline script attribute, or custom application JavaScript is present.

**Given** HTMX enhancement is available
**When** the login form or an expected error response is used
**Then** only the pinned self-hosted HTMX asset and pinned official `response-targets` extension are used, expected errors target a stable programmatically announced status region, and the same interaction remains functional as a full-page form without HTMX
**And** approved script assets use fixed routes, JavaScript media types, immutable digest mappings, and `nosniff`.

**Given** login HTML is returned
**When** response headers are inspected
**Then** it sends `Cache-Control: no-store`, `X-Content-Type-Options: nosniff`, `Referrer-Policy: no-referrer`, and the prescribed restrictive Content Security Policy
**And** cookies are `HttpOnly` and `SameSite=Strict`; non-debug cookies are Secure while debug/local cookies may support local HTTP.

**Given** the login page is used in current stable Chrome, Firefox, Safari, or Edge at widths down to 320 CSS pixels
**When** the administrator navigates with a keyboard or another pointer-independent method
**Then** every control remains reachable, operable, and programmatically labelled; focus is at least two CSS pixels thick with at least 3:1 adjacent contrast; required text/control contrast holds; and any inline error target is programmatically associated
**And** no horizontal layout assumption prevents login.

**Given** a probe or pinned static-asset route is requested
**When** middleware processes the request
**Then** it neither creates nor loads a session
**And** it cannot mint CSRF or submission tokens.

**Given** anonymous sessions or login tokens expire
**When** indexed expiry processing or bounded request-time cleanup runs
**Then** expired state is physically removed without scanning an unbounded store
**And** cleanup failure is sanitized without exposing session or token state.

**Given** `GET /login` renders in a supported browser at 320 CSS pixels and 400% zoom
**When** the Access form is inspected and operated without a pointer
**Then** the password field, submit, and every link/control render at least 48 by 48 CSS pixels without page-level horizontal scrolling or clipped text
**And** the dark Editorial Contrast tokens, square geometry, field/rule states, required contrast, and absence of decorative transition/depth match `UX-TARGET-01`, `UX-RESPONSIVE-01`, and `UX-VISUAL-01`.

**Given** the login page is reached through a forward native or enhanced navigation
**When** the response is rendered
**Then** the stable Sign in heading is the single server-owned focus destination where forward focus is required, while ordinary refresh uses normal document focus
**And** stable IDs and focus treatment satisfy `UX-SHELL-01` and `UX-FOCUS-01` without custom JavaScript.

**Given** login rendering or an expected enhanced request enters pending, capacity, timeout, or unavailable state
**When** status changes
**Then** one stable scoped node uses polite atomic announcement, its owning region exposes `aria-busy`, and expected `4xx`/`5xx` fragments route declaratively through the official extension
**And** native full-page recovery presents the same safe outcome under `UX-STATUS-01`.

**Requirements:** SPEC-FR1, SPEC-FR3, SPEC-FR6, SPEC-FR11, SPEC-FR14..SPEC-FR17, SPEC-FR90..SPEC-FR96; SPEC-NFR19..SPEC-NFR23, SPEC-NFR28..SPEC-NFR30; anonymous Login-token issuance, capacity, expiry, cleanup, semantic HTML, static asset, session-free route, accessibility, responsive, and strict security-header requirements; UX contracts: `UX-SHELL-01`, `UX-TARGET-01`, `UX-FOCUS-01`, `UX-STATUS-01`, `UX-RESPONSIVE-01`, `UX-VISUAL-01`.

**UX acceptance evidence:** Web tests cite the listed UX contracts and verify the Login route's exact 48-by-48 geometry at 320px/400% zoom, stable Sign in focus target, scoped status/`aria-busy` behavior, native/enhanced parity, Editorial Contrast state tokens, and absence of custom JavaScript or decorative motion. Brownfield disposition: retain shared server-rendered/session boundaries, replace any unprotected or client-dependent login surface, and remove superseded assets/routes rather than preserve parallel paths.

### Story 1.5: Sign In with Bounded Password Verification

As the administrator,
I want login attempts verified safely and successful authentication persisted atomically,
So that I can access Debtor without brute-force, proxy-spoofing, or partial-session risk.

**Acceptance Criteria:**

**Given** production proxy configuration has an empty trusted CIDR set, an unrecognized or nonsingular forwarding-header mode, or otherwise cannot establish one trusted client-resolution policy
**When** Debtor starts
**Then** startup fails before socket admission with a sanitized configuration error
**And** debug/local mode may use the direct peer only as explicitly allowed.

**Given** a login request arrives through an immediate peer outside `APP_TRUSTED_PROXY_CIDRS`
**When** forwarding headers are present
**Then** Debtor ignores them and resolves the direct peer according to environment policy
**And** no raw forwarding value or resolved client IP is logged.

**Given** a trusted proxy supplies the configured forwarding format
**When** client identity is resolved
**Then** only that selected format and trusted chain order are accepted
**And** the resulting limiter behavior is identical for edge HTTP/3 and TCP fallback requests.

**Given** a login body exceeds 8 KiB or cannot be structurally decoded within bounds
**When** `POST /login` is processed
**Then** it is rejected by the shared strict form extractor before CSRF validation, password verification, limiter reservation, or authentication dispatch
**And** no credential or submitted value is logged.

**Given** the form structure is valid but CSRF is missing, duplicate, malformed, or incorrect
**When** login is submitted
**Then** the request is rejected before limiter reservation or password verification
**And** the submission token is not consumed because dispatch did not occur.

**Given** bounded structural decoding and CSRF validation succeed but a required non-security field is missing, duplicate, malformed, or unknown
**When** strict route-field validation runs
**Then** the request is rejected before password parsing, limiter reservation, submission-token reservation, or authentication dispatch
**And** the valid submission token remains usable.

**Given** CSRF and route validation succeed but the submission token is missing, unknown, expired, reserved, or consumed
**When** login is submitted
**Then** Debtor returns `409 Conflict` with sanitized feedback
**And** neither the limiter nor password verifier is invoked.

**Given** a valid login attempt is ready for password verification
**When** its submission token is atomically reserved and the trusted-client limiter is consulted
**Then** the limiter records one attempt immediately before every password verification, including a correct password, permits at most five attempts in a rolling five-minute window, tracks at most 4,096 active client keys without eviction, and fails closed with retryable `429` for an unseen key at capacity
**And** any rejection before password verification records no attempt.

**Given** a trusted-client limiter history ages beyond its rolling five-minute window
**When** indexed bounded expiry cleanup runs or that key is next evaluated
**Then** the expired history is physically removed and capacity becomes reusable
**And** active histories are never evicted to admit an unseen key.

**Given** password verification is admitted
**When** concurrent attempts are processed
**Then** at most two Argon2 verifications run concurrently using the already validated configured hash
**And** incorrect credentials receive a fixed sanitized response without revealing whether failure came from comparison details.

**Given** the submitted password is correct and authenticated capacity is available
**When** login promotion occurs
**Then** Debtor atomically rotates and durably stores the session ID, authenticated state, and a new CSRF token before emitting an authenticated cookie or `303` redirect
**And** only after durable promotion does it reset the trusted-client limiter history.

**Given** correct-password promotion finds 32 live authenticated sessions or durable session persistence fails
**When** promotion is attempted
**Then** Debtor flushes the anonymous login session and returns retryable `503 Service Unavailable` without emitting an authenticated cookie
**And** the reserved submission token remains terminal because one dispatch occurred.

**Given** the protected Login form is submitted once
**When** password verification or durable promotion is pending
**Then** the submit initiator becomes unavailable, repeated activation is suppressed or coalesced, the form region exposes `aria-busy`, and one stable polite atomic status node announces the pending state without moving focus
**And** native submission remains authoritative under `UX-STATUS-01` and `UX-FOCUS-01`.

**Given** credentials are incorrect or sign-in is rate-limited, capacity-blocked, timed out, or temporarily unavailable
**When** the safe Login response renders
**Then** the password is not retained, the response discloses no credential/client/session detail, and the stable Login error/status destination receives the exact alert or focus treatment defined by the outcome
**And** recovery remains a protected native form with a fresh applicable token rather than a replay.

**Given** authentication and durable promotion succeed
**When** the `303` destination renders
**Then** the stable authenticated page heading is the single forward-focus destination and no private page is restored from an HTMX history snapshot
**And** browser history reveals no cached Login password or ledger content.

**Given** every Login outcome is rendered at 320 CSS pixels and 400% zoom
**When** controls, messages, and recovery actions wrap
**Then** all controls remain at least 48 by 48 CSS pixels, text and focus contrast hold, no clipping or page-level horizontal scroll occurs, and Editorial Contrast states remain square and motion-free
**And** native and enhanced responses are visually and behaviorally equivalent.

**Requirements:** SPEC-FR4..SPEC-FR5, SPEC-FR8, SPEC-FR12..SPEC-FR13, SPEC-FR15, SPEC-FR18..SPEC-FR19, SPEC-FR88..SPEC-FR89, SPEC-FR100, SPEC-FR103 (Login admission only); SPEC-NFR17..SPEC-NFR18, SPEC-NFR21..SPEC-NFR25; trusted-proxy, strict-form, anonymous-token reservation, password-concurrency, limiter, durable-promotion, and safe-diagnostic requirements; UX contracts: `UX-SHELL-01`, `UX-TARGET-01`, `UX-FOCUS-01`, `UX-STATUS-01`, `UX-RESPONSIVE-01`, `UX-VISUAL-01`.

**UX acceptance evidence:** Web tests cite the listed UX contracts and verify pending suppression, stable status/`aria-busy`, exact success/error focus, non-retained password, native recovery, 320px/400% geometry, and visual-state parity. Brownfield disposition: retain validated password/session/limiter adapters and the shared strict pipeline; remove bypass routes, duplicate Login handlers, and client-dependent pending/error behavior.

### Story 1.6: Maintain an Authenticated Session and Sign Out

As the administrator,
I want authenticated access to persist safely until I sign out or the session expires,
So that I can use Debtor without repeated login while retaining reliable revocation.

**Acceptance Criteria:**

**Given** login promotion completed durably
**When** the administrator follows the `303` redirect
**Then** Debtor renders an authenticated no-store home shell using the rotated server-side session and CSRF token
**And** the response exposes no session identifier or security token in HTML beyond the tokens required by rendered unsafe forms.

**Given** an anonymous, expired, flushed, or otherwise invalid session requests any ledger route
**When** authentication middleware evaluates it
**Then** ledger access and mutation are denied before any ledger use case is invoked
**And** the response follows the native full-page authentication path with sanitized feedback.

**Given** a valid authenticated session is used
**When** any authenticated request completes
**Then** its 30-day inactivity expiry is refreshed and persisted using indexed expiry state
**And** the cookie remains `HttpOnly` and `SameSite=Strict`, and no refresh can create a 33rd authenticated session or evict another authenticated session.

**Given** the administrator opens an authenticated page
**When** the page is rendered at supported browser widths and without HTMX
**Then** it uses the shared semantic, keyboard-operable, responsive shell and required security headers
**And** every unsafe form on the page receives the current session-backed CSRF token and a distinct submission token.

**Given** a valid authenticated session submits logout with exactly one valid CSRF token and one valid single-use submission token
**When** logout is dispatched
**Then** Debtor atomically reserves the submission token, flushes the server-side session, expires the browser cookie, and redirects with `303`
**And** replaying the logout token returns `409` without a second dispatch.

**Given** logout form structure, CSRF, or submission-token validation fails before dispatch
**When** the request is rejected
**Then** the session remains authenticated, no logout use case is invoked, and applicable invalid-token responses follow the shared status contract
**And** validation before dispatch does not consume a valid submission token.

**Given** an authenticated session reaches 30 days of inactivity
**When** expiry processing runs or the session is next presented
**Then** the record is physically deleted and ledger access is denied
**And** no stale authenticated record is retained to consume capacity.

**Given** Debtor restarts
**When** a previously issued anonymous or authenticated cookie is presented
**Then** no corresponding process-local session exists, the administrator is logged out, and ledger access is denied
**And** restart does not attempt to restore session state from SQLite.

**Given** a valid authenticated request renders the shared page shell
**When** it is viewed at 320 CSS pixels, 400% zoom, or a wide composition
**Then** header content remains in reading order, every control including Sign out is at least 48 by 48 CSS pixels, no private content is clipped or horizontally scrolled, and wide adaptation preserves DOM/focus order
**And** Editorial Contrast tokens, square controls, double-rule hierarchy, focus geometry, and motion prohibitions satisfy `UX-SHELL-01`, `UX-TARGET-01`, `UX-RESPONSIVE-01`, and `UX-VISUAL-01`.

**Given** Sign out is activated
**When** its protected request is pending
**Then** the initiator becomes unavailable, repeated activation is suppressed or coalesced, the owning region exposes `aria-busy`, and one stable polite atomic status node reports pending/failure without moving focus
**And** native form submission remains authoritative.

**Given** logout commits successfully
**When** the `303` Sign in page renders
**Then** the stable Sign in heading receives forward focus, authenticated history exposes no cached ledger page, and no prior private HTMX snapshot is restored
**And** failure instead focuses the header control or scoped status according to `UX-FOCUS-01` and `UX-STATUS-01`.

**Requirements:** SPEC-FR1, SPEC-FR7, SPEC-FR9..SPEC-FR11, SPEC-FR14..SPEC-FR16, SPEC-FR18..SPEC-FR19, SPEC-FR88, SPEC-FR90..SPEC-FR95; SPEC-NFR19..SPEC-NFR23, SPEC-NFR25, SPEC-NFR28..SPEC-NFR30; authenticated access, expiry refresh, logout, restart invalidation, and shared web-policy requirements; UX contracts: `UX-SHELL-01`, `UX-TARGET-01`, `UX-FOCUS-01`, `UX-STATUS-01`, `UX-RESPONSIVE-01`, `UX-VISUAL-01`.

**UX acceptance evidence:** Web tests cite the listed UX contracts and verify shell geometry, Sign-out pending suppression, exact success/failure focus, scoped status/`aria-busy`, no-store history behavior, 320px/400% zoom, wide normal-flow adaptation, and native/enhanced parity. Brownfield disposition: retain the process-local session boundary and shared headers; remove alternate auth shells, unprotected logout, persistent-session paths, and private HTMX history snapshots.

### Story 1.7: Extend Replay Protection Beyond Login and Sign Out

As the administrator,
I want the established Login and Sign-out replay protection extended consistently to authenticated forms,
So that later ledger mutations reuse one safe boundary rather than creating route-local guards.

**Acceptance Criteria:**

**Given** Stories 1.4 through 1.6 have already issued anonymous Login tokens, reserved them immediately before Login dispatch, and reused that reservation path for Sign out
**When** the shared replay boundary is extended
**Then** those completed Login and Sign-out behaviors remain unchanged and no token store, reservation path, or extractor is duplicated
**And** Story 1.7 does not become a prerequisite for the completed Login or Sign-out outcomes.

**Given** authenticated unsafe forms are rendered
**When** submission tokens are issued
**Then** the authenticated pool holds at most 1,024 live tokens globally and 32 per session, each with a 30-minute absolute expiry
**And** it remains isolated from the 4,096-token anonymous pool.

**Given** one authenticated response renders multiple mutually exclusive unsafe forms
**When** submission protection is issued
**Then** the forms may carry one shared page-scoped, session-bound single-use submission token distinct from CSRF, so rendered form count does not consume one store record per form
**And** the first dispatch reserves that token terminally, every other form from that stale response then returns `409` without dispatch, and the canonical redirect/reload issues fresh protection.

**Given** multiple tabs or rendered pages are open
**When** page-scoped tokens are issued
**Then** each rendered response uses its own token, the 32-per-session and 1,024-global bounds remain enforced, and capacity failure is page-level retryable without partially protected forms
**And** later Manage and archived-list stories reuse this contract rather than adding Participant-count caps, unprotected forms, or token-per-row stores.

**Given** either token pool or per-session limit is full
**When** another unsafe form requires a token
**Then** issuance fails closed with sanitized retryable feedback and no token from the other pool is displaced
**And** no unsafe form is rendered with missing or fabricated protection.

**Given** a non-login form body exceeds 256 KiB or cannot be structurally decoded within bounds
**When** the shared extractor processes it
**Then** rejection occurs before CSRF validation, route-specific parsing, submission-token reservation, or use-case dispatch
**And** no guarded side effect starts.

**Given** a non-login form is structurally decoded within bounds
**When** the shared extractor proceeds
**Then** it establishes authentication and exactly one correct session-backed CSRF token before route-specific known-field and value parsing
**And** missing, duplicate, malformed, or incorrect CSRF rejects before submission-token reservation or dispatch.

**Given** CSRF succeeds but a required non-security field is missing, duplicate, malformed, or unknown
**When** strict route-field validation runs
**Then** the request is rejected before route-specific value construction, submission-token reservation, or dispatch
**And** the valid submission token remains usable.

**Given** structure, authentication, CSRF, and route-specific validation succeed
**When** an unsafe operation is ready for its first state-changing use-case call
**Then** the server atomically reserves the session-bound token immediately before dispatch and marks the request as dispatched at that boundary
**And** no generic pre-dispatch work runs after reservation.

**Given** two concurrent requests present the same valid token
**When** both attempt reservation
**Then** exactly one can reserve and dispatch while the other receives `409 Conflict`
**And** deterministic coordination proves that the rejected request invokes no use case or guarded side effect.

**Given** a token has been reserved for one dispatch
**When** the use case commits, rolls back, returns an application error, its task fails, or response delivery fails
**Then** the reservation remains terminal and every replay returns `409` without dispatch
**And** no automatic retry is triggered by the token store.

**Given** a token is missing, unknown, expired, reserved, consumed, or bound to another session
**When** an unsafe route receives it
**Then** Debtor returns `409 Conflict` before use-case invocation
**And** full-page and enhanced responses use the shared sanitized status presentation.

**Given** authenticated tokens expire or their session is flushed
**When** indexed cleanup runs
**Then** expired/session-owned records are physically removed in bounded work and capacity becomes reusable
**And** cleanup never logs token or session identifiers.

**Given** web adapter tests exercise every pre-dispatch rejection path
**When** shared fake use cases record invocations
**Then** tests prove zero dispatch for malformed fields, oversized bodies, failed authentication, invalid CSRF, invalid tokens, and validation errors available through login, logout, and the authenticated shell at this point in the sequence
**And** each later story that introduces an archived route proves its own precheck and zero-dispatch behavior, while concurrency tests use barriers or notifications rather than timing sleeps.

**Given** an unsafe form token is missing, unknown, expired, reserved, consumed, or session-mismatched
**When** the request is rejected before dispatch
**Then** the native response renders a focused conflict heading or scoped stable status node announcing `409 Conflict`, states that no change occurred, and provides a canonical-form reload that issues a fresh token
**And** the recovery action is at least 48 by 48 CSS pixels and never resubmits the prior request.

**Given** valid form input fails before token reservation
**When** the canonical validation response renders
**Then** the token remains usable, multiple errors focus one linked `role="alert"` summary or the sole invalid control, and stable guidance/error IDs remain associated
**And** no generic replay-conflict presentation replaces field validation.

**Given** one request reserves a token and dispatches
**When** mutation execution remains pending
**Then** the initiating control stays unavailable, the owning region exposes `aria-busy`, one polite atomic status reports pending, and no generic timeout or client retry claims a result
**And** pending remains until definitive success or rollback.

**Given** native and enhanced invalid-token, validation, pending, and definitive responses are compared at 320 CSS pixels and 400% zoom
**When** recovery and messages wrap
**Then** focus, status, target geometry, contrast, and Editorial Contrast states remain equivalent without custom JavaScript or overlays.

**Requirements:** SPEC-FR15..SPEC-FR19, SPEC-FR87..SPEC-FR91, SPEC-FR100, SPEC-FR103 (route-neutral extension only); SPEC-NFR1, SPEC-NFR21..SPEC-NFR22, SPEC-NFR25, SPEC-NFR30, SPEC-NFR32..SPEC-NFR34; authenticated-token extension, non-Login strict extraction, dispatch-boundary, deterministic concurrency, safe failure, and web-testing requirements; UX contracts: `UX-TARGET-01`, `UX-FOCUS-01`, `UX-STATUS-01`, `UX-RESPONSIVE-01`, `UX-VISUAL-01`.

**UX acceptance evidence:** Shared web contract tests plus Login, Logout, and authenticated-form consumers cite the listed UX contracts and verify exact conflict focus, fresh-form recovery, retained pre-dispatch token behavior, `aria-busy`, pending suppression, 48-by-48 recovery controls, 320px/400% rendering, and native/enhanced parity. Brownfield disposition: retain one shared extractor/token store and remove route-local replay guards, duplicate token stores, ambiguous retry paths, and any post-dispatch generic timeout wrapper.

### Story 1.8: Expose Health, Readiness, and Bounded Admission

As the administrator,
I want Debtor to report trustworthy health while bounding incoming work,
So that overload or failed mandatory maintenance cannot leave the service falsely ready.

**Acceptance Criteria:**

**Given** the process event loop is alive
**When** `/healthz` is requested
**Then** it reports process liveness within a two-second outer timeout without loading ledger contents, SQLite readiness, sessions, or exchange-rate state
**And** it uses the separate probe admission budget.

**Given** SQLite responds within its one-second inner readiness timeout and all mandatory supervisors are healthy
**When** `/readyz` is requested
**Then** it reports ready within the two-second outer timeout
**And** Frankfurter availability and ledger contents are not consulted.

**Given** SQLite readiness fails or a mandatory session-expiry or submission-token cleanup supervisor fails
**When** readiness is evaluated
**Then** `/readyz` reports not ready, new user admission stops, and coordinated shutdown begins
**And** `/healthz` continues to represent process liveness until the process exits.

**Given** user traffic is saturated at 64 in-flight requests or login is saturated at four
**When** a probe request arrives
**Then** up to four probe requests can still be admitted independently
**And** deterministic tests prove user saturation cannot starve health/readiness.

**Given** login, ordinary safe dynamic reads, and probes are classified
**When** timeout middleware is composed
**Then** login and ordinary safe reads receive 30-second limits and probes receive the two-second outer/one-second SQLite limits
**And** mutation requests are excluded from generic post-dispatch timeout cancellation.

**Given** anonymous sessions, authenticated sessions, and both submission-token pools contain expiring records
**When** mandatory indexed cleanup supervisors run at the specified cadence, including five-minute session cleanup
**Then** expired records are physically removed in bounded work without evicting live authenticated sessions or scanning unbounded storage
**And** supervisor status is observable by readiness without exposing record contents.

**Given** a cleanup iteration returns an error, panics, exits unexpectedly, or otherwise cannot guarantee continued expiry processing
**When** root supervision observes the failure
**Then** readiness fails, admission closes, and shutdown is initiated exactly once
**And** logs contain only fixed safe supervisor/operation categories.

**Given** probe and admission behavior is tested
**When** held permits, failed supervisors, slow SQLite readiness, and provider unavailability are simulated
**Then** tests use barriers, notifications, or held resources rather than sleeps and assert exact admission/readiness outcomes
**And** no probe creates or loads a session.

**Requirements:** SPEC-FR96..SPEC-FR102; SPEC-NFR1..SPEC-NFR4, SPEC-NFR25..SPEC-NFR27, SPEC-NFR31..SPEC-NFR34; probe separation, timeout classification, mandatory supervision, bounded cleanup, and deterministic concurrency-testing requirements. No UX IDs apply because probes and supervisors are machine/operator interfaces rather than rendered Administrator controls. Brownfield disposition: retain separate probe admission and supervised cleanup owners; remove provider-coupled readiness, session-loading probe middleware, shared user/probe saturation, and unsupervised cleanup paths.

### Story 1.9: Shut Down the Authenticated Runtime Safely

As the administrator,
I want the authenticated runtime to stop admission and close its resources safely,
So that the service can shut down cleanly before ledger mutations are introduced.

**Acceptance Criteria:**

**Given** shutdown begins before any ledger-mutation route exists
**When** the root lifecycle coordinator acts
**Then** it stops new admission and drains HTTP connections for at most ten seconds
**And** it observes an empty dispatched-mutation registry rather than simulating a future ledger mutation.

**Given** HTTP drain completes and no dispatched mutation exists
**When** storage shutdown proceeds
**Then** Debtor attempts the bounded fixed WAL-checkpoint PRAGMA, preserves WAL sidecars if checkpointing fails, and closes the pool in order
**And** SQLite diagnostics contain only allowed fixed operation names and result categories.

**Given** the composed application is tested over a real socket
**When** the startup smoke scenario logs in with CSRF and a single-use token, performs an authenticated read, initiates coordinated shutdown, and observes completion
**Then** startup ordering, authentication protection, bounded HTTP drain with no active ledger mutation, and resource closure are verified
**And** secrets, identifiers, SQL, values, query strings, and provider URLs do not appear in captured logs.

**Requirements:** SPEC-FR105; SPEC-NFR25..SPEC-NFR27, SPEC-NFR31..SPEC-NFR34; SQLite diagnostic, root smoke-test, no-active-mutation drain, and resource-shutdown requirements. Story 2.1 completes real-mutation `SPEC-FR103` and `SPEC-FR104` evidence. No UX IDs apply because this story verifies runtime composition rather than changing rendered controls. Brownfield disposition: retain the root lifecycle coordinator and real-socket smoke boundary; remove duplicate shutdown paths, simulated mutation completion, ambiguous `Unknown`-as-rollback handling, and unbounded post-drain resource closure.

## Epic 2: Organize Groups and Participants

The administrator can create and configure Groups, set up and maintain active Group-owned Participant identities, navigate archived Group contexts, and safely manage Group lifecycle changes that do not require debt calculation.

### Story 2.1: Create and Select a Group

As the administrator,
I want to create and select a Group,
So that I have a private ledger context in which to organize shared Spendings.

**Acceptance Criteria:**

**Given** the administrator has no active Groups
**When** the authenticated home page is opened
**Then** it renders an accessible active-Group empty state and a protected native Create Group form
**And** archived Groups are not included in the active list.

**Given** a Group creation form is rendered
**When** its fields are inspected
**Then** it accepts only a Group name plus the shared CSRF and submission-token fields
**And** it does not expose Group Currency, user, membership, tenant, or Participant fields.

**Given** the submitted name trims to empty or exceeds 100 Unicode characters
**When** creation is validated
**Then** Debtor returns `422 Unprocessable Entity` with a programmatically associated inline error and the raw submitted name retained
**And** validation occurs before token reservation and dispatch, so the valid submission token remains usable.

**Given** the submitted name is valid
**When** creation dispatches
**Then** the application-owned input policy creates a Group with a positive `i64` ID, `USD` Group Currency, and active status through a narrow repository port
**And** web code does not construct persistence or framework-owned domain state.

**Given** the process-local ledger write gate is available
**When** Group creation begins its state-changing use case
**Then** the submission token is reserved and dispatch marked immediately before the call, the gate serializes the mutation, SQLite work commits transactionally, and the root executor publishes the authoritative result before response work
**And** success redirects with `303` to the new Group's Manage section.

**Given** a Group creation request has not yet dispatched
**When** body extraction, authentication, CSRF, token checks, or asynchronous web prechecks exceed the 30-second absolute pre-dispatch deadline
**Then** the request is rejected without reserving a still-valid token, invoking Group creation, opening a transaction, or starting a guarded side effect
**And** deterministic tests prove the absence of dispatch.

**Given** Group creation has dispatched
**When** persistence returns an authoritative commit or rollback result
**Then** the root mutation executor synchronously and infallibly publishes `Committed` or `RolledBack` before response rendering or delivery
**And** no generic request timeout, response failure, or automatic retry can change that outcome.

**Given** a Group creation request is oversized, unauthenticated, malformed, missing/duplicate/unknown-field, CSRF-invalid, submission-token-invalid, or application-invalid
**When** the shared unsafe-form pipeline processes it
**Then** rejection occurs at the prescribed boundary before Group creation dispatch, transaction opening, write-gate side effects, or mutation-epoch advancement
**And** web tests with the shared fake prove zero dispatch for every hostile-input path and preserve a still-valid token when validation fails before reservation.

**Given** a dispatched Group mutation task fails unexpectedly
**When** rollback is authoritatively established
**Then** the executor may publish `RolledBack`
**And** otherwise it publishes `Unknown`, initiates fatal shutdown, suppresses automatic retry, and never represents the outcome as rollback.

**Given** gate acquisition cannot complete within five seconds
**When** Group creation is attempted
**Then** it returns sanitized retryable feedback without opening a transaction or starting a guarded persistence side effect
**And** deterministic tests prove no repository write occurred.

**Given** multiple valid Group creations are admitted
**When** their commits complete in any permitted order
**Then** every committed Group appears in the active list, the last committed state governs without optimistic revision columns, and the one process-local mutation epoch advances exactly once after each successful ledger commit and never after rollback/rejection
**And** every later Group, Participant, and Spending mutation reuses this epoch owner rather than introducing another revision mechanism, while ordering exposed to the UI remains deterministic.

**Given** Group persistence is migrated
**When** invalid direct rows are attempted
**Then** SQLite structurally enforces supported currency, active/archive flag shape, bounded non-empty text shape, positive identity, and required relationships without duplicating Unicode trimming or monetary rules
**And** checked SQLx queries have matching committed offline metadata.

**Given** shutdown begins while a Group mutation is dispatched
**When** HTTP drain reaches its bound
**Then** the process waits for the mutation's authoritative completion before checkpoint and pool close
**And** the composed test verifies SPEC-FR103..SPEC-FR104 against a real ledger mutation.

**Given** Groups has no active Groups
**When** it renders at 320 CSS pixels or 400% zoom
**Then** create-by-name precedes the empty active list, Archived Groups is a contextual text link, and every field/link/button/row target is at least 48 by 48 CSS pixels without clipping or horizontal page scroll
**And** the page uses the exact Group-list Editorial Contrast hierarchy and states.

**Given** Group creation fails validation
**When** one or multiple errors render
**Then** the raw name remains, the sole invalid control or linked `role="alert"` summary receives focus, stable guidance/error IDs remain associated, and no pending status remains
**And** native and enhanced responses use the same canonical markup.

**Given** valid Group creation dispatches
**When** it is pending and then commits
**Then** the initiator is unavailable while one scoped polite atomic status and `aria-busy` represent pending, and the `303` Manage destination autofocuses the stable Manage heading
**And** the new Group shell exposes five native destinations in fixed order with Manage current.

**Given** the new Group has no Participant
**When** its Group shell renders
**Then** Add Spending is visibly disabled with adjacent setup guidance and a 48-by-48 Manage/Add Participant recovery link
**And** no future Spending capability is required for this setup state to function.

**Requirements:** SPEC-FR14..SPEC-FR19, SPEC-FR20..SPEC-FR22, SPEC-FR25, SPEC-FR87..SPEC-FR89, SPEC-FR103..SPEC-FR104; SPEC-NFR3, SPEC-NFR7, SPEC-NFR15, SPEC-NFR21..SPEC-NFR22, SPEC-NFR25, SPEC-NFR31..SPEC-NFR34; Group default, shared hostile-input rejection, write-gate, process-local mutation epoch, SQLx, migration, strict-form, vertical-slice, and first real mutation-integration requirements; UX contracts: `UX-SHELL-01`, `UX-TARGET-01`, `UX-FOCUS-01`, `UX-STATUS-01`, `UX-RESPONSIVE-01`, `UX-VISUAL-01`.

**UX acceptance evidence:** Tests cite the listed UX contracts and verify Groups empty/create states, 48-by-48 geometry at 320px/400% zoom, stable validation and Manage focus, status/`aria-busy`, five-link native shell order, disabled Add Spending guidance, wide normal-flow adaptation, and visual tokens. Brownfield disposition: retain the write gate/root mutation executor; replace reusable memberships/global Participant assumptions and remove alternate Group-create routes or non-USD creation paths.

### Story 2.2: Configure Group Settings

As the administrator,
I want to rename a Group and choose its Group Currency,
So that its ledger context matches how I identify and settle shared expenses.

**Acceptance Criteria:**

**Given** an established active Group is selected from the active list
**When** its contextual page opens
**Then** Debtor opens the Summary section by default and provides native navigation to Manage
**And** a newly created Group continues to open Manage as established in Story 2.1.

**Given** the active Group's Manage section is rendered
**When** settings are inspected
**Then** the current raw display values are shown with a protected form for Group name and exactly one Group Currency from `USD`, `EUR`, `RUB`, `KGS`, `TRY`, `KZT`, `UZS`, `CNY`, `KRW`, `JPY`, `OMR`, or `TJS`
**And** the page retains the shared accessible, responsive, security-header, and native-fallback behavior.

**Given** a name trims to empty, exceeds 100 Unicode characters, or the currency code is missing, duplicate, unknown, or malformed
**When** settings are submitted
**Then** Debtor returns `422 Unprocessable Entity` for application validation or rejects malformed structure through the strict extractor, safely escapes and retains every submitted value, and dispatches no use case
**And** the submission token is not consumed before dispatch.

**Given** valid changed settings are submitted for an active Group
**When** the mutation dispatches
**Then** application-owned policy validates and passes transport-neutral values through a narrow repository port, the shared write gate and transaction persist name and Group Currency atomically, and success redirects with `303` to the contextual Group page
**And** no SQLx or Axum type crosses the application boundary.

**Given** the Group already has or later acquires Spendings
**When** Group Currency is changed
**Then** the setting is accepted as a freely changeable display/settlement target without rewriting Source Currency or historical allocation data
**And** no exchange-rate provider call is made by the settings mutation.

**Given** an archived Group ID is addressed directly through a settings form or mutation route
**When** the request is processed
**Then** Debtor returns `409 Conflict` before token reservation and use-case invocation
**And** the archived page exposes no settings control.

**Given** concurrent valid settings writes are admitted
**When** they commit
**Then** each complete write is atomic and the last committed settings state wins without a stale-edit conflict
**And** gate or SQLite timeout failures expose only sanitized retryable categories.

**Given** active Group Manage renders
**When** settings are viewed at 320 CSS pixels, 400% zoom, or wide composition
**Then** Group settings is the first ruled Manage section, its flexible name plus 116px Currency grid stacks before collision, and every field/action remains at least 48 by 48 CSS pixels
**And** the five-link native shell has Manage current and preserves order/focus across width changes.

**Given** settings validation fails
**When** one or multiple errors render
**Then** raw name and Currency remain, stable guidance/error IDs associate with fields, and focus targets the linked alert summary or sole invalid field
**And** the valid submission token remains usable and pending status clears.

**Given** valid settings dispatch
**When** it is pending and then commits
**Then** the initiating action is unavailable, one scoped status and `aria-busy` represent pending, and the canonical response focuses Group settings with one committed-state announcement
**And** a Currency change visibly invalidates later converted contexts without fabricating a converted result in this story.

**Given** an archived Group opens read-only Manage
**When** its shell renders
**Then** visible “Archived” text is associated with the Group heading, settings are definition text or native readonly values, mutation controls are absent, and five native destinations remain available
**And** direct mutation still returns pre-dispatch `409`.

**Requirements:** SPEC-FR21..SPEC-FR23, SPEC-FR26..SPEC-FR27, SPEC-FR87..SPEC-FR90; SPEC-NFR3, SPEC-NFR7, SPEC-NFR15, SPEC-NFR25, SPEC-NFR28..SPEC-NFR34; supported-code, last-commit, archived-route, strict-form, accessibility, and transaction requirements; UX contracts: `UX-SHELL-01`, `UX-TARGET-01`, `UX-FOCUS-01`, `UX-STATUS-01`, `UX-RESPONSIVE-01`, `UX-VISUAL-01`.

**UX acceptance evidence:** Tests cite the listed UX contracts and verify Manage section order, responsive settings grid, 48-by-48 controls at 320px/400% zoom, validation/success focus, scoped pending status, archived read-only state, native shell parity, and Editorial Contrast rendering. Brownfield disposition: retain application-owned settings policy and transactional repository; remove dashboard-style settings, editable ownership/user fields, archived mutation forms, and compatibility routes for superseded Group models.

### Story 2.3: Add Group-Owned Participants

As the administrator,
I want to add accounting identities inside a Group,
So that its shared Spendings can later identify who paid and who owes.

**Acceptance Criteria:**

**Given** an active Group's Manage section is opened
**When** Participant management is rendered
**Then** it shows only that Group's active Participants and a protected Add Participant form within the Group context
**And** no global Participant list, user account, membership, login, or cross-Group reuse control exists.

**Given** the Add Participant form is first rendered
**When** the color field is populated
**Then** the server suggests a varied normalized `#RRGGBB` value while allowing the administrator to choose another valid color before submission
**And** the suggestion does not require custom JavaScript.

**Given** a Participant name trims to empty or exceeds 100 Unicode characters, or the color is not normalized valid `#RRGGBB`
**When** the form is submitted
**Then** Debtor returns `422 Unprocessable Entity` with programmatically associated inline errors and retains the raw submitted name and color
**And** validation occurs before token reservation or use-case dispatch.

**Given** valid Participant input is submitted for an active Group
**When** creation dispatches
**Then** application-owned policy creates a positive-`i64` Participant identity owned by exactly that Group, with the normalized chosen color and active state
**And** success redirects with `303` to the Group's Manage section where the new Participant is visible.

**Given** a valid Participant ID from another Group is supplied through a crafted request or persistence call
**When** ownership is checked
**Then** the operation is rejected without creating or reassigning an identity
**And** no application path can reuse one Participant across Groups.

**Given** the target Group is archived or does not exist
**When** the add form or mutation route is requested
**Then** an archived target returns `409 Conflict` before use-case invocation and a missing target returns a sanitized not-found response
**And** no submission token is reserved for the archived precheck.

**Given** Participant persistence is migrated
**When** direct invalid rows or Group deletion are exercised
**Then** SQLite enforces positive IDs, required Group ownership, bounded non-empty text shape, normalized color shape, boolean status shape, and Group-owned cascade behavior for a history-free Group
**And** application routes still expose no independent Participant deletion.

**Given** multiple Participants exist in a Group
**When** another color suggestion is requested
**Then** the server varies valid suggestions predictably enough to avoid always presenting the same default while remaining deterministic under test
**And** submitted color always wins over the suggestion on validation rerender.

**Given** Manage has no active Participants
**When** the Participants section renders at 320 CSS pixels or 400% zoom
**Then** identity guidance precedes the add form, the flexible name plus 124px color grid stacks before collision, the color control reserves a 48px outlined swatch, and every action remains at least 48 by 48 CSS pixels
**And** Add Spending remains disabled with distinct no-Participant guidance.

**Given** the server suggests a valid color
**When** the Participant color control renders
**Then** the normalized `#RRGGBB` text field is authoritative, a named outlined swatch is supplementary, and identity/state never relies on color alone
**And** no custom JavaScript is used.

**Given** Participant validation fails
**When** the canonical Manage response renders
**Then** raw name and color remain, field guidance/error IDs remain stable, invalid state is programmatically exposed, and focus targets the linked alert summary or sole invalid field
**And** the valid submission token is preserved.

**Given** Participant creation is pending and then commits
**When** the response transitions
**Then** the initiator is unavailable under one scoped `aria-busy`/polite status, success focuses the new Participant row/action and announces once, and the first active Participant enables Add Spending
**And** the five-destination shell and Manage reading order remain stable.

**Requirements:** SPEC-FR30..SPEC-FR34, SPEC-FR36, SPEC-FR87..SPEC-FR90; SPEC-NFR7, SPEC-NFR15..SPEC-NFR16, SPEC-NFR25, SPEC-NFR28..SPEC-NFR34; Participant ownership, color suggestion, no-global-surface, migration, strict-form, accessibility, and history-preservation requirements; UX contracts: `UX-SHELL-01`, `UX-TARGET-01`, `UX-FOCUS-01`, `UX-STATUS-01`, `UX-RESPONSIVE-01`, `UX-VISUAL-01`.

**UX acceptance evidence:** Tests cite the listed UX contracts and verify empty/setup guidance, responsive name/color geometry, 48px swatch and controls at 320px/400% zoom, raw-color validation retention, exact success/error focus, status/`aria-busy`, Add Spending enablement, and Editorial Contrast states. Brownfield disposition: retain Group-scoped repository ownership; remove global Participant routes, memberships, user/account fields, independent delete, and client-generated color defaults.

### Story 2.4: Edit Active Participants Without Deleting Identity

As the administrator,
I want to correct an active Participant's name and color inside its Group,
So that the accounting identity remains recognizable without breaking ownership or future history.

**Acceptance Criteria:**

**Given** an active Participant belongs to an active Group
**When** its edit form is opened from Group Manage
**Then** the current name and color are shown in a protected, accessible form scoped to that Group
**And** Group ownership is not editable.

**Given** the submitted name trims to empty or exceeds 100 Unicode characters, or the color is invalid
**When** edit validation runs
**Then** Debtor returns `422 Unprocessable Entity`, renders programmatically associated inline errors, and retains every raw submitted value
**And** no token is reserved and no use case is invoked.

**Given** valid changed values are submitted
**When** the edit dispatches
**Then** application-owned policy preserves the Participant's positive ID and owning Group, normalizes the valid color, and persists name/color atomically through the shared write gate
**And** success redirects with `303` to Group Manage.

**Given** a crafted request addresses a Participant through a different Group ID
**When** the application and transactional repository guards evaluate ownership
**Then** the mutation is rejected without changing the Participant
**And** the response reveals no cross-Group identity details.

**Given** the owning Group is archived
**When** the Participant edit form or mutation route is requested directly
**Then** Debtor returns `409 Conflict` before use-case dispatch and displays no edit control on the archived Group page
**And** the existing identity remains readable.

**Given** the administrator looks for a Participant delete action or submits a crafted delete request
**When** routes and use cases are evaluated
**Then** no independent Participant deletion capability exists
**And** persistence restricts destructive behavior outside history-free owning-Group deletion.

**Given** concurrent valid edits are admitted
**When** commits complete
**Then** each update is atomic and the last committed name/color wins without optimistic revision handling
**And** safe failure mapping exposes no SQL, identifiers, values, or request-derived diagnostics.

**Given** an active Participant edit block renders in Manage
**When** it is viewed at 320 CSS pixels or 400% zoom
**Then** visible identity precedes the flexible name plus 124px color fields and Save action, the grid stacks before collision, and every field/action remains at least 48 by 48 CSS pixels
**And** no archive, delete, Historical Balance, or eligibility control is fabricated before its owning capability exists.

**Given** the stored or submitted color is valid
**When** the color control renders
**Then** normalized text remains authoritative, the named outlined swatch is supplementary, and the current Participant name remains the accessible identity
**And** arbitrary stored colors are never assumed to satisfy text/status contrast.

**Given** edit validation fails
**When** Manage rerenders
**Then** raw name/color remain, stable field guidance and error associations persist, and focus targets the linked alert summary or sole invalid field
**And** no status color or swatch substitutes for the error text.

**Given** a valid edit is pending and then commits
**When** the response transitions
**Then** Save becomes unavailable under one scoped `aria-busy`/polite status, success focuses the updated Participant row/action and announces once, and the shell/Manage reading order remain stable
**And** no completion badge or identity replacement is introduced.

**Requirements:** SPEC-FR30..SPEC-FR33, SPEC-FR36, SPEC-FR87..SPEC-FR90; SPEC-NFR7, SPEC-NFR15..SPEC-NFR16, SPEC-NFR25, SPEC-NFR28..SPEC-NFR34; immutable identity and ownership, no-delete, strict-form, transaction, and safe-diagnostic requirements; UX contracts: `UX-SHELL-01`, `UX-TARGET-01`, `UX-FOCUS-01`, `UX-STATUS-01`, `UX-RESPONSIVE-01`, `UX-VISUAL-01`. Story 3.3 proves current-name resolution after persisted Spending history exists.

**UX acceptance evidence:** Tests cite the listed UX contracts and verify Participant block order, responsive name/color geometry, 48-by-48 controls, color text/swatch semantics, retained validation and focus, pending/committed status, stable identity, and native/enhanced parity at 320px/400% zoom. Brownfield disposition: retain stable Participant IDs and Group ownership; remove global edit routes, ownership reassignment, independent delete, client-only swatches, and user/account abstractions.

### Story 2.5: Archive, Restore, or Delete a History-Free Group

As the administrator,
I want to retire unused Groups without losing referenced history,
So that active navigation stays clean while destructive actions remain safe.

**Acceptance Criteria:**

**Given** an active Group can be archived
**When** the administrator activates Archive from Manage
**Then** Debtor opens a full server-rendered confirmation page naming the Group, stating that archive is reversible and history remains readable, and carrying an allow-listed return URL plus stable invoker focus ID
**And** Cancel returns to the exact Manage archive control with no mutation.

**Given** the protected archive confirmation is submitted once
**When** dispatch is pending and then commits
**Then** the initiator becomes unavailable under one scoped status/`aria-busy`, the write gate atomically archives the Group, and success redirects to Groups with the active-list heading focused and one count announcement
**And** replay returns `409` without a second dispatch.

**Given** a Group is archived
**When** active and archived views render
**Then** it is absent from active rows, present in a separate contextual Archived Groups view, visibly labelled "Archived," and readable through the five-link shell without mutation controls except protected Restore
**And** every other direct archived mutation/form route returns pre-use-case `409`.

**Given** any archived Group mutation or mutation-form route other than restore is addressed directly
**When** the request is processed
**Then** Debtor returns `409 Conflict` before token reservation and use-case invocation
**And** web tests prove zero dispatch.

**Given** protected Restore commits
**When** the canonical Groups response renders
**Then** the restored Group link receives focus and one announcement, while ownership and Participants remain unchanged
**And** Restore requires no Balance/rate calculation and no confirmation page.

**Given** an active Group has no Spendings
**When** Delete is activated
**Then** a full server-rendered confirmation names the Group, lists the unreferenced owned Participants that will also be deleted, states irreversibility, and provides an allow-listed Cancel target back to Manage
**And** the one-shot protected Confirm action is visually last and destructive, while server-owned confirmation state binds the exact disclosed Participant-ID set.

**Given** confirmed deletion commits
**When** the canonical Groups response renders
**Then** the Group and unreferenced Participants are deleted atomically, the active-list heading receives focus, and one announcement reports completion
**And** the transaction rejects with no deletion if Spendings exist or the current owned Participant-ID set differs from the disclosed set; Story 3.1 completes structural Spending restriction proof.

**Given** archive, restore, or delete validation fails before dispatch or a duplicate token is replayed
**When** the request is rejected
**Then** applicable `422` or `409` behavior is returned with no state-changing use case invocation
**And** valid pre-dispatch validation does not consume the token.

**Given** Group lifecycle operations race with another admitted Group mutation
**When** write-gate and transaction boundaries execute
**Then** operations serialize, eligibility is checked transactionally, and only complete committed states become visible
**And** timeout or constraint failures remain sanitized and log only allowed fixed SQLite categories.

**Given** lifecycle confirmation, archived view, and canonical return states render at 320 CSS pixels and 400% zoom
**When** names, scope text, actions, and messages wrap
**Then** every control remains 48 by 48 CSS pixels, focus stays visible, no clipping/page horizontal scroll occurs, and reversible archive versus irreversible delete is distinguished with text plus Editorial Contrast states
**And** native and enhanced paths remain equivalent.

**Requirements:** SPEC-FR24..SPEC-FR28, SPEC-FR36, SPEC-FR87..SPEC-FR90; SPEC-NFR3, SPEC-NFR15..SPEC-NFR16, SPEC-NFR22, SPEC-NFR25, SPEC-NFR31..SPEC-NFR34; archived-Group context, immutable page, history-free cascade, strict-form, and transactional lifecycle requirements; UX contracts: `UX-SHELL-01`, `UX-TARGET-01`, `UX-FOCUS-01`, `UX-STATUS-01`, `UX-CONFIRM-01`, `UX-RESPONSIVE-01`, `UX-VISUAL-01`. Story 3.1 completes `SPEC-FR29` with first Spending persistence.

**UX acceptance evidence:** Tests cite the listed UX contracts and verify named scope/reversibility, one-shot pending state, allow-listed Cancel focus, archive/delete/restore success focus, Groups returns, archived read-only shell, 48-by-48 geometry, and native/enhanced parity. Brownfield disposition: retain transactional archive/restore/delete rules; remove direct archive/delete mutations without confirmation, arbitrary return URLs/focus IDs, mixed active/archived lists, and independent Participant deletion.

## Epic 3: Record and Maintain Exact Spendings

The administrator can create, preview, inspect, correct, browse, and delete exact multi-currency Spendings with one Payer and deterministic Proportional or Exact Shares.

### Story 3.1: Record a Proportional Spending

As the administrator,
I want to preview and record a Spending divided by proportional weights,
So that I can complete an exact shared transaction in one usable flow.

**Acceptance Criteria:**

**Given** an active Group has active Participants
**When** the persistent Add Spending action is opened
**Then** a focused full-page form starts with empty description and Total, current UTC date, Source Currency equal to Group Currency, no Category, no Payer, and every active Participant selected with Proportional weight `1`
**And** the administrator can deselect active Share Participants while archived Participants remain unavailable for a new allocation.

**Given** the form is rendered
**When** fields and choices are inspected
**Then** Source Currency is limited to the twelve supported codes, Category to the eight supported codes with no default, Share mode to Proportional or Exact, and exactly one Payer can be selected
**And** description is limited to a trimmed non-empty 200 Unicode characters and date to strict `YYYY-MM-DD` on or after `2025-01-01`.

**Given** a Payer is selected
**When** the allocation table updates through native Preview or optional HTMX enhancement
**Then** that one active Group-owned Participant is shown as paying the full Total
**And** Payer selection remains distinct from Share responsibility.

**Given** Total or weight text is submitted
**When** transport-neutral application input parses it
**Then** Total uses exact `Decimal`, is positive, at most `999_999_999_999`, and has valid Source Currency precision; each selected weight is positive, at most `1,000,000`, and has at most six fractional digits
**And** web parsing only preserves raw structure/text and never constructs allocations.

**Given** valid Proportional input with at least one unique selected Share Participant
**When** Preview is calculated
**Then** one checked `i128` integer-ratio operation normalizes weights at the maximum submitted scale, divides exact Total minor units, and assigns residual units by descending remainder with ascending Participant ID ties
**And** the candidate conserves the Total exactly; if any resulting Share is zero, Preview marks the selection invalid and requires a larger Total or deselection rather than accepting an aggregate.

**Given** normalization, multiplication, division, precision, ownership, uniqueness, or checked arithmetic fails
**When** Preview is requested
**Then** Debtor returns `422 Unprocessable Entity` with retained raw values and programmatically associated errors, produces no allocation aggregate, and consumes no submission token
**And** it never rounds invalid input, substitutes zero, panics, or uses floating point.

**Given** identical valid fields are submitted through explicit native Preview and HTMX-enhanced Preview
**When** results are compared
**Then** both paths call the same application/domain allocation operation and return identical ordered exact amounts
**And** HTMX failure leaves the native path fully usable.

**Given** the Group is archived or a selected Participant belongs to another Group
**When** the form or Preview route is requested
**Then** an archived Group returns pre-use-case `409`, while ownership validation returns sanitized `422` without an aggregate
**And** no state-changing dispatch or provider call occurs.

**Given** Add Spending opens
**When** the focused form renders
**Then** its stable `h1` receives forward focus, the page has one document scroll owner and `min-height: 100dvh`, fields use two columns then stack at 350px or narrower, and the sticky in-flow action bar clears keyboard/safe-area growth
**And** all fields/actions remain 48 by 48 CSS pixels at 320px/400% zoom.

**Given** any active Group section renders the governed shell
**When** Add Spending eligibility is evaluated
**Then** Add Spending is a native link from every eligible active section, is hidden for archived Groups, and when no active Participant exists is disabled with the exact governed setup guidance and 48-by-48 recovery link
**And** activation opens the focused form at its stable heading without requiring HTMX.

**Given** the allocation table renders
**When** inspected at 320px or 400% zoom
**Then** a labelled focusable internal scroll region contains a 520px semantic table with columns `116/76/76/92/160`, sticky Participant identity, explicit header/control associations, long-name wrapping, and no page-level horizontal scroll
**And** Payer, Included, Weight, and Share controls remain 48 by 48.

**Given** native Preview succeeds
**When** the full page rerenders
**Then** reviewed allocation input is non-editable, Approve applies only to that reviewed input, Edit allocation returns to editable state, and the preview heading/status receives the prescribed native forward focus
**And** Cancel remains an allow-listed return link.

**Given** enhanced Preview requests overlap
**When** input revisions change
**Then** latest input wins, superseded responses never swap, only derived cells/status/approval state change, and focus, caret, selection, keyboard, active row, table/page scroll remain unchanged
**And** one polite atomic status plus `aria-busy` owns pending/ready/error transitions.

**Given** Preview is pending, stale, invalid, or superseded
**When** the action bar renders
**Then** Approve is disabled, Total/status occupy row one, Cancel/Preview-or-Edit/Approve occupy three equal 48px-minimum columns in row two, and amount never appears inside Approve
**And** messages wrap without clipping.

**Given** the current Proportional input has a valid reviewed allocation
**When** the administrator approves it
**Then** the server validates the reviewed-input binding, reparses the same raw input, reruns the allocation operation, and atomically persists one complete Spending with its Payer and Shares through the sole shared create aggregate path
**And** a stale, mismatched, invalid, or replayed review cannot dispatch or create a partial aggregate.

**Given** a valid create aggregate is ready to dispatch
**When** the shared create path enters the write gate and transaction
**Then** persistence rechecks that the Group is active and every new Payer/Share Participant is active and owned by that Group, then persists canonical decimal `TEXT` for the Spending, Payer, and Shares atomically
**And** any eligibility race, constraint failure, or checked error rolls back the complete aggregate; malformed or noncanonical stored monetary values are corruption on hydration and never reach logs or HTTP.

**Given** the first Spending has committed for a Group
**When** Group deletion is attempted through the application or directly against SQLite
**Then** deletion is refused in favor of archive and SQLite restricts direct deletion
**And** paired tests prove neither the referenced Group nor any owned Participant is deleted, completing final evidence for `SPEC-FR29`.

**Requirements:** SPEC-FR29, SPEC-FR43..SPEC-FR56, SPEC-FR59, SPEC-FR62..SPEC-FR64, SPEC-FR87..SPEC-FR94; SPEC-NFR5..SPEC-NFR10, SPEC-NFR14..SPEC-NFR16, SPEC-NFR22, SPEC-NFR25, SPEC-NFR28..SPEC-NFR34; application input ownership, exact Decimal, deterministic Proportional Preview/commit, reviewed-input binding, sole atomic create aggregate persistence, canonical hydration, Spending-backed Group deletion restriction, native fallback, and retained validation requirements; UX contracts: `UX-SHELL-01`, `UX-TARGET-01`, `UX-ALLOC-01`, `UX-FOCUS-01`, `UX-STATUS-01`, `UX-PREVIEW-NATIVE-01`, `UX-PREVIEW-LATEST-01`, `UX-RESPONSIVE-01`, `UX-VISUAL-01`.

**UX acceptance evidence:** Tests cite the listed UX contracts and verify Add Spending placement/eligibility/archived behavior and native focused-form routing, exact allocation geometry, 100-character names, maximum OMR Total, 320px/400% zoom, keyboard/safe area, native reviewed state, enhanced revision rejection/state preservation, action-bar layout, focus/status behavior, and Editorial Contrast rendering. Brownfield disposition: retain exact allocation domain functions where conforming; remove plural Payers, Equal mode, inline/modal Spending flows, client-constructed allocations, and full-form HTMX swaps.

### Story 3.2: Record a Spending with Exact Shares

As the administrator,
I want to preview, edit, and record exact Participant Shares,
So that I can complete a precisely allocated shared transaction.

**Acceptance Criteria:**

**Given** valid Total text and active Group Participants
**When** Exact mode is selected for a new Spending
**Then** every active Participant is initially selected, Total minor units are divided equally, and residual units are assigned one at a time in ascending Participant ID order
**And** the candidate conserves the Total exactly; if any initialized Share is zero, it remains visibly invalid until the Total increases or Participants are deselected, and no aggregate is accepted.

**Given** the administrator deselects a Participant or edits an exact Share amount
**When** Preview recalculates
**Then** the allocation table shows each selected Participant's exact amount and a remaining or excess difference against the Total
**And** Payer selection remains exactly one independent control in the same table.

**Given** selected exact Shares contain a duplicate Participant, zero/negative amount, excess currency precision, amount above `999_999_999_999`, or cross-Group/inactive identity
**When** Preview is requested
**Then** Debtor returns `422` with retained raw selections/amounts and no aggregate
**And** checked Rust parsing performs no rounding, SQL arithmetic, floating point, or zero substitution.

**Given** selected exact Shares do not sum to the Total in Source Currency minor units
**When** Preview or commit validation runs
**Then** the displayed difference identifies the remaining or excess amount and the aggregate is not accepted
**And** the response is deterministic regardless of submitted field order.

**Given** unique positive precision-valid Shares sum exactly to the Total and one active Group-owned Payer pays the full Total
**When** Exact Preview succeeds
**Then** the ordered aggregate conserves both Payer Total and Share Total exactly
**And** native and enhanced Preview paths return identical results from the same application/domain operation.

**Given** the administrator switches between Proportional and Exact before commit
**When** either mode is previewed
**Then** only the submitted active mode determines the displayed allocation and invalid hidden/stale mode fields are rejected under the strict known-field contract rather than silently interpreted
**And** no mode or weight state is persisted.

**Given** Exact mode renders in the allocation table
**When** viewed at 320px/400% zoom with long names and maximum OMR Total
**Then** the same labelled 520px internal-scroll table and `116/76/76/92/160` geometry remain stable, Participant identity stays sticky, Share controls remain 48 by 48, and the page does not scroll horizontally
**And** selection and Payer state remain programmatically exposed.

**Given** selected Shares do not close the Total
**When** Exact Preview responds
**Then** one stable allocation status says "Remaining: [amount]" or "Excess: [amount]," describes both the allocation region and Approve, and never relies on color alone
**And** row-specific errors attach only to their row inputs.

**Given** native Exact Preview succeeds
**When** reviewed state renders
**Then** input is non-editable, Approve is limited to the reviewed Exact input, Edit allocation restores editable controls, and prescribed native focus targets the review status/heading
**And** no stale editable value can be approved.

**Given** enhanced Exact requests overlap
**When** Shares, inclusion, Total, Payer, Source Currency, or mode changes
**Then** latest revision wins, superseded responses are ignored, only derived cells/status/approval swap, and focus/caret/selection/keyboard/scroll/active row remain unchanged
**And** Approve stays disabled while pending, stale, invalid, or superseded.

**Given** the current Exact input has a valid reviewed allocation
**When** the administrator approves it
**Then** the server revalidates the same raw input and exact closure, then reuses Story 3.1's reviewed-input and complete-aggregate create path to persist one Spending with its Payer and Shares
**And** a stale, mismatched, invalid, or replayed review cannot dispatch or create a partial aggregate.

**Requirements:** SPEC-FR49..SPEC-FR59, SPEC-FR62..SPEC-FR64; SPEC-NFR5..SPEC-NFR10, SPEC-NFR14..SPEC-NFR16, SPEC-NFR22, SPEC-NFR25, SPEC-NFR28..SPEC-NFR34; deterministic Exact initialization, difference display, uniqueness, positivity, exact closure, Preview/commit parity, and reuse of Story 3.1's sole atomic aggregate path; UX contracts: `UX-TARGET-01`, `UX-ALLOC-01`, `UX-FOCUS-01`, `UX-STATUS-01`, `UX-PREVIEW-NATIVE-01`, `UX-PREVIEW-LATEST-01`, `UX-RESPONSIVE-01`, `UX-VISUAL-01`.

**UX acceptance evidence:** Tests cite the listed UX contracts and verify Exact-mode geometry, Remaining/Excess associations, row errors, reviewed non-editable state, stale approval prevention, enhanced latest-input behavior, interaction-state preservation, and responsive/visual parity. Brownfield disposition: retain shared exact allocation/conservation logic; remove Equal mode as a persisted/input mode, client-side difference authority, editable native review state, and mode-field compatibility paths.

### Story 3.3: Browse and Inspect Spending History

As the administrator,
I want to browse Transactions and open one complete Spending,
So that I can review exact history without loading the entire ledger.

**Acceptance Criteria:**

**Given** a Group has more than 25 Spendings
**When** Transactions is opened or paged
**Then** each page contains at most 25 rows ordered by `(spent_date DESC, id DESC)` using keyset cursors
**And** no offset pagination or SQL monetary aggregation is used.

**Given** multiple Spendings share the same date
**When** consecutive pages are traversed
**Then** descending positive Spending ID provides a stable tie-breaker with no duplicate or skipped row
**And** cursor input is bounded, strictly parsed, and safely rejected when malformed.

**Given** a Transactions row is displayed
**When** its amount and identity fields are rendered
**Then** canonical monetary text is decoded and revalidated in Rust, Source Currency is shown, and the current Payer name is resolved
**And** corruption withholds the affected aggregate behind a sanitized failure rather than displaying normalized or partial data.

**Given** the administrator opens a Spending detail route
**When** persistence loads it
**Then** one database snapshot returns the complete Spending, Payer, all ordered Shares, Group context, and current Participant names directly by ID
**And** it does not materialize all Group history.

**Given** a referenced Participant was renamed or archived after the Spending committed
**When** history or detail is rendered
**Then** the current Participant name is displayed and the historical Payer/Share role and exact amount remain unchanged
**And** archived identities are not filtered from the aggregate.

**Given** the owning Group is archived
**When** Transactions or Spending detail is requested
**Then** historical content remains readable through the same accessible, responsive native path
**And** mutation, settings, and edit/delete controls are absent.

**Given** a safe history/detail read exceeds its 30-second timeout or persistence fails
**When** the web layer maps the failure
**Then** it returns sanitized bounded feedback with no raw SQL, values, identifiers, or partial aggregate
**And** no session or submission-token state is mutated beyond normal authenticated-session refresh.

**Given** Transactions renders Spending history
**When** rows are inspected
**Then** each Spending is a native `<details>` row whose 48px-minimum `<summary>` shows disclosure plus Description/date left and unbroken Source Currency Total right, while expanded content uses the specified definition layout before equal Edit/Delete actions
**And** Participant marker never replaces the visible current name.

**Given** more than 25 Spendings exist
**When** pagination renders at 320px/400% zoom
**Then** Previous and Next are equal 48px outlined native links with readable disabled endpoints and page context between/above them
**And** no infinite scroll, offset paging, clipping, or page-level horizontal scroll appears.

**Given** a native or enhanced page change succeeds
**When** the canonical Transactions response renders
**Then** the stable Transactions heading receives forward focus and one scoped polite atomic status announces page context
**And** pending/error retains the activated link, toggles `aria-busy`, and preserves the current rows until outcome.

**Given** the Group or referenced Participant is archived
**When** Transactions/detail renders
**Then** visible "Archived" text is associated with the Group/Participant identity, history remains readable through the five-link shell, and mutation actions are suppressed for archived Groups
**And** financial facts retain normal readable contrast.

**Requirements:** SPEC-FR42..SPEC-FR43, SPEC-FR65..SPEC-FR66; SPEC-NFR2, SPEC-NFR5, SPEC-NFR10, SPEC-NFR14..SPEC-NFR16, SPEC-NFR25, SPEC-NFR28..SPEC-NFR34; keyset pagination, direct snapshot loading, current-name resolution, archived readability, and corruption-safe rendering requirements; UX contracts: `UX-SHELL-01`, `UX-TARGET-01`, `UX-FOCUS-01`, `UX-STATUS-01`, `UX-RESPONSIVE-01`, `UX-VISUAL-01`.

**UX acceptance evidence:** Tests cite the listed UX contracts and verify row semantics/layout, long descriptions/names, exact amount wrapping, 48-by-48 summaries/actions/pagination at 320px/400% zoom, page-change focus/status, archived text/readability, shell parity, and Editorial Contrast states. Brownfield disposition: retain direct aggregate reads and keyset ordering; remove all-history materialization, offset/infinite paging, table rows that hide detail/actions, stale stored-name projections, and archived-history filtering.

### Story 3.4: Correct an Existing Spending

As the administrator,
I want to edit a complete Spending and its allocations,
So that corrections preserve exact accounting and archived historical identities.

**Acceptance Criteria:**

**Given** an active Group owns a Spending
**When** its edit form opens
**Then** Debtor direct-loads one complete aggregate and renders Exact mode with stored Payer and Share amounts
**And** no prior Proportional mode or weights are reconstructed or persisted.

**Given** the existing Payer or a Share Participant is archived
**When** the edit form is rendered
**Then** that Participant remains visible only in the same stored Payer or Share role with the stored amount
**And** active Group-owned Participants remain available for otherwise valid changes.

**Given** submitted update input introduces an archived Participant, changes an archived Participant from Payer to Share or Share to Payer, or newly adds that identity to another role
**When** application validation runs
**Then** Debtor returns `422` with retained raw input and no aggregate replacement
**And** no submission token is consumed before dispatch.

**Given** valid raw edit input is submitted
**When** Preview and commit validation run
**Then** all description/date/code/precision/Payer/Share rules are reparsed through the same Exact allocation and conservation logic used for creation
**And** Payer Total and Share Total each equal the Spending Total exactly.

**Given** a valid replacement is dispatched
**When** the write gate and transaction execute
**Then** persistence rechecks Group ownership, active eligibility for every newly introduced role, and archived-role retention before atomically replacing scalar fields, Payer, and all Shares
**And** any failure rolls back the complete replacement.

**Given** two valid edits are admitted
**When** commits complete
**Then** the last committed complete aggregate wins without an optimistic stale-edit conflict
**And** every read observes either the old or new complete snapshot, never a mixed allocation.

**Given** update succeeds
**When** the authoritative commit is published
**Then** Debtor redirects with `303` to the Spending or Transactions context showing the corrected aggregate
**And** a replayed token returns `409` without another update.

**Given** the owning Group is archived
**When** edit form, Preview, or update routes are requested directly
**Then** Debtor returns `409` before invoking a use case
**And** read-only detail remains available.

**Given** Edit opens from a Transaction row
**When** the focused form renders
**Then** its stable `h1` receives forward focus, Exact mode shows stored Payer/Shares, archived identities carry visible "Archived" text only in retained roles, and Cancel carries an allow-listed row return target
**And** the full-page form/allocation/action-bar geometry from Stories 3.1-3.2 remains exact.

**Given** edit Preview is native
**When** valid reviewed state renders
**Then** all input is non-editable, Approve is bound to the currently reviewed edit, Edit allocation restores controls, and changing any field/revision invalidates approval
**And** Source Currency correction is included in the reviewed binding.

**Given** enhanced edit Preview requests overlap
**When** any editable field/revision changes
**Then** latest input wins, superseded responses cannot swap or re-enable Approve, only derived/status/approval state changes, and interaction state is preserved
**And** archived retained rows cannot be changed through stale enhanced state.

**Given** edit commits successfully
**When** the corrected Spending reorders history or remains on its page
**Then** the canonical page/disclosure ID is encoded and the committed Transaction `<summary>` receives forward focus without a completion badge
**And** failure instead focuses the form heading/linked error while retaining safely decoded input.

**Requirements:** SPEC-FR43..SPEC-FR44, SPEC-FR46..SPEC-FR51, SPEC-FR56..SPEC-FR63, SPEC-FR66, SPEC-FR87..SPEC-FR90; SPEC-NFR3, SPEC-NFR5..SPEC-NFR10, SPEC-NFR14..SPEC-NFR16, SPEC-NFR22, SPEC-NFR25, SPEC-NFR28..SPEC-NFR34; Exact-on-edit, archived-role retention, atomic replacement, last-commit, and retained-validation requirements; UX contracts: `UX-TARGET-01`, `UX-ALLOC-01`, `UX-FOCUS-01`, `UX-STATUS-01`, `UX-PREVIEW-NATIVE-01`, `UX-PREVIEW-LATEST-01`, `UX-RESPONSIVE-01`, `UX-VISUAL-01`.

**UX acceptance evidence:** Tests cite the listed UX contracts and verify Exact-on-edit geometry, archived-role text and immutability, reviewed-input binding, Source Currency revision invalidation, superseded-response rejection, preserved interaction state, canonical reordered-row focus, and native/enhanced parity. Brownfield disposition: retain complete aggregate replacement and exact rules; remove reconstructed Proportional/weight state, partial scalar/allocation updates, archived-role migration shims, direct unreviewed commits, and stale-edit conflict APIs.

### Story 3.5: Delete a Spending Atomically

As the administrator,
I want to delete an incorrect Spending as one complete aggregate,
So that no orphaned Payer or Share data remains in the ledger.

**Acceptance Criteria:**

**Given** an active Group owns a Spending
**When** Delete is activated from its expanded Transaction row
**Then** Debtor direct-loads one complete aggregate and opens a full server-rendered confirmation naming the Spending, exact Source Currency Total, Payer, Shares, date, category, description, irreversible effect, and one-shot protection
**And** server state carries only an allow-listed canonical return URL plus stable invoking Delete-control/disclosure ID.

**Given** Cancel is activated
**When** Transactions renders
**Then** no mutation occurs, the canonical page and disclosure state are encoded, and focus returns to the invoking Delete control in the expanded row
**And** no arbitrary URL or DOM selector is accepted.

**Given** protected Confirm is activated once
**When** deletion is pending
**Then** the initiator becomes unavailable, repeated activation is suppressed, one stable scoped status/`aria-busy` reports pending, and the write-gated transaction deletes the complete aggregate or rolls back
**And** the submission token remains terminal after dispatch.

**Given** any aggregate row cannot be deleted or the transaction fails
**When** persistence returns
**Then** the complete Spending remains unchanged or the authoritative failure is reported; no partial allocation deletion is visible
**And** raw SQLite diagnostics, values, and identifiers are sanitized.

**Given** deletion commits
**When** canonical Transactions is selected
**Then** Debtor returns to the same page when still valid and focuses the next row summary, otherwise the previous row summary, otherwise the Transactions heading when empty or page count changed
**And** no out-of-range page, orphaned disclosure target, completion badge, or duplicate announcement appears; deleting the last Spending can make later history-free Group deletion eligible.

**Given** two concurrent delete attempts or an edit/delete race targets the same Spending
**When** operations serialize through the write gate
**Then** at most one delete commits, later work observes the committed state, failure focuses the invoking context/scoped status, and duplicate token replay returns `409` without dispatch
**And** no automatic retry or optimistic stale-edit conflict is exposed.

**Given** the owning Group is archived
**When** delete confirmation or mutation is addressed directly
**Then** Debtor returns `409` before use-case invocation while read-only detail remains accessible
**And** the page exposes no delete control.

**Given** deletion tests run
**When** transaction failure, held gate timeout, constraint failure, duplicate dispatch, and archived precheck are exercised
**Then** tests prove atomicity and zero guarded side effects for every pre-dispatch rejection using deterministic coordination
**And** checked SQLx metadata remains current.

**Given** confirmation/canonical return renders at 320px/400% zoom
**When** long details/actions wrap
**Then** destructive and Cancel actions remain at least 48 by 48 CSS pixels, focus remains visible, no clipping/page horizontal scroll occurs, and coral destructive treatment is paired with explicit text
**And** native and enhanced paths remain equivalent.

**Requirements:** SPEC-FR43, SPEC-FR62..SPEC-FR66, SPEC-FR87..SPEC-FR90; SPEC-NFR3, SPEC-NFR14..SPEC-NFR16, SPEC-NFR22, SPEC-NFR25, SPEC-NFR31..SPEC-NFR34; direct aggregate loading, atomic deletion, history-free eligibility, archived rejection, and deterministic concurrency requirements; UX contracts: `UX-TARGET-01`, `UX-FOCUS-01`, `UX-STATUS-01`, `UX-CONFIRM-01`, `UX-RESPONSIVE-01`, `UX-VISUAL-01`.

**UX acceptance evidence:** Tests cite the listed UX contracts and verify complete named scope, allow-listed Cancel return, one-shot pending state, atomic failure, page-boundary next/previous/heading success focus, no out-of-range page, archived precheck, 48-by-48 geometry, and native/enhanced parity. Brownfield disposition: retain direct aggregate load and atomic deletion; remove inline/browser confirmations, arbitrary return targets, unconditional page-one redirects, partial allocation deletion paths, and duplicate destructive routes.

## Epic 4: Understand Current-Month Spending

The administrator can see exact current UTC-month totals by Source Currency and conserved Group Currency totals with complete rate evidence and safe degradation.

### Story 4.1: Review Exact Source Currency Monthly Totals

As the administrator,
I want to see this month's exact totals in each original Source Currency,
So that I can understand current spending even when exchange rates are unavailable.

**Acceptance Criteria:**

**Given** a selected Group has Spendings across multiple dates
**When** Summary is calculated
**Then** only Spendings whose dates fall in the current UTC calendar month are included
**And** no arbitrary date-range control or non-monthly statistics are introduced.

**Given** current-month Spendings use one or more Source Currencies
**When** totals are produced
**Then** the Group total and each Payer's paid total are grouped by original Source Currency and summed with checked `Decimal` arithmetic in Rust
**And** SQLite performs no monetary parsing, conversion, or aggregation.

**Given** a Payer was archived or renamed after paying a current-month Spending
**When** Summary is rendered
**Then** the Spending still contributes, the archived identity remains included, and its current Participant name is displayed
**And** active-Participant filtering does not alter historical totals.

**Given** canonical stored amounts decode successfully
**When** all included amounts are aggregated
**Then** every displayed amount has valid Source Currency precision and deterministic Payer-ID ordering
**And** exact checked sums are independent of database row order.

**Given** the Group has no current-month Spendings
**When** Summary opens
**Then** it renders an accessible source-summary empty state rather than zero-valued fabricated currencies
**And** ordinary Group navigation and Add Spending remain available.

**Given** Frankfurter is offline, rate caches are empty, or converted-summary calculation is unavailable
**When** source totals are requested
**Then** source totals render unchanged without any provider call
**And** ledger CRUD remains usable.

**Given** stored corruption or checked source aggregation failure prevents a trustworthy total
**When** Summary maps the failure
**Then** it returns sanitized feedback without displaying a partial affected source summary or substituting zero
**And** logs contain no amounts, identities, SQL, or row values.

**Given** Summary is used without HTMX at 320 CSS pixels or by keyboard
**When** source totals contain multiple currencies/Payers
**Then** semantic headings/tables or lists preserve labels, focus, contrast, and readable relationships without horizontal interaction dependency
**And** the same full-page result is available in supported browsers.

**Given** Summary opens through native or enhanced Group navigation
**When** the source result renders
**Then** the stable Summary heading receives forward focus for native navigation, enhanced navigation uses the same URL/response and focus contract, and the five-link shell marks Summary current
**And** Add Spending remains persistently available for active eligible Groups.

**Given** one or more Source Currencies have current-month Spendings
**When** financial results render
**Then** `YYYY-MM` month title and `YYYY-MM · UTC` context are explicit, each currency block shows its Group total before indented per-Payer rows, and every amount includes symbol plus ISO code with tabular numerals
**And** Participant color is supplementary to visible current name and "Archived" text.

**Given** Summary is viewed at 320px/400% zoom or wide composition
**When** source blocks, long names, currencies, and shell controls wrap
**Then** the narrow layout is one column with no horizontal dependency, all shell/actions remain 48 by 48 CSS pixels, and wide adaptation uses the governed reading measure without reordering DOM/focus
**And** Editorial Contrast rules, typography, square geometry, and motion prohibition hold.

**Given** source calculation is pending, empty, or unavailable from corruption/checked failure
**When** state changes
**Then** one stable scoped status uses polite atomic announcement and `aria-busy`, no individual amount is live, empty state fabricates no currency, and failure exposes no partial affected totals
**And** native full-page and enhanced section responses are equivalent.

**Requirements:** SPEC-FR41, SPEC-FR67..SPEC-FR68, SPEC-FR72; SPEC-NFR2, SPEC-NFR5..SPEC-NFR7, SPEC-NFR10, SPEC-NFR14..SPEC-NFR16, SPEC-NFR25..SPEC-NFR30, SPEC-NFR32..SPEC-NFR34; current UTC-month, Rust aggregation, archived identity, provider-independent fallback, and accessible summary requirements; UX contracts: `UX-SHELL-01`, `UX-TARGET-01`, `UX-FOCUS-01`, `UX-STATUS-01`, `UX-RESPONSIVE-01`, `UX-VISUAL-01`.

**UX acceptance evidence:** Tests cite the listed UX contracts and verify Summary heading/focus, shell order/current state, source hierarchy, archived names, symbol+ISO amounts, 320px/400% and wide composition, empty/error status/`aria-busy`, no live amount rows, and visual parity. Brownfield disposition: retain exact Rust aggregation and complete reads; remove SQL monetary totals, arbitrary timeframe/statistics UI, active-only historical filtering, and provider-coupled source summaries.

### Story 4.2: Review Conserved Group Currency Monthly Totals

As the administrator,
I want current-month Payer totals converted with exact date-appropriate evidence into Group Currency,
So that I can see reproducible totals whose displayed Group amount reconciles exactly to the displayed Payer amounts.

**Acceptance Criteria:**

**Given** current-month conversion is requested
**When** persistence supplies input
**Then** one SQLite snapshot materializes Group Currency and every complete included Spending before releasing the read transaction
**And** no provider request holds a database transaction.

**Given** a Spending has Source Currency, target Group Currency, original date `R`, and UTC calculation date `C`
**When** Historical context is built
**Then** `F = min(R, C)`, identity is `(source, target, R, F)`, and provider effective date remains separate evidence
**And** same-currency conversion synthesizes exact Decimal `1`, performs no provider call, and remains disclosed.

**Given** Frankfurter returns a JSON numeric rate
**When** the adapter decodes it
**Then** the complete lexical number is parsed directly into exactly representable positive `Decimal` without floating point or rounding
**And** malformed, nonpositive, oversized, excess-scale, or unrepresentable responses map to a safe adapter reason.

**Given** fresh provider calls are required
**When** contexts resolve
**Then** calls use rustls, five-second connect/20-second total/64 KiB bounds, identical uncached keys use single-flight, contexts deduplicate, and at most four calls run globally and per calculation
**And** completion order cannot change evidence or output.

**Given** exact fresh/synthetic evidence exists for every context
**When** conversion runs
**Then** paid values multiply and accumulate exactly per Payer without per-Spending rounding, final Payer totals truncate together and distribute residual units by descending remainder with ascending Participant-ID ties, and the displayed Group total is their exact sum
**And** checked failure produces no partial converted row.

**Given** same-currency, fixed-past, and future contexts contribute
**When** the converted Summary renders
**Then** Group Currency major total precedes per-Payer rows and deterministic unique rate evidence, future contexts are visibly provisional with explicit reason, and no manual Retry exists
**And** every amount includes symbol plus ISO code.

**Given** fresh resolution or checked conversion/aggregation/quantization fails
**When** the projection is built
**Then** the entire converted region is withheld under one sanitized unavailable state while Story 4.1 source totals and ledger CRUD remain usable
**And** no partial prior converted rows or zero substitutions remain.

**Given** conversion moves through Updating, Ready, Provisional, or Unavailable
**When** native/enhanced Summary renders at 320px/400% zoom or wide composition
**Then** one stable conversion notice uses polite atomic announcement and `aria-busy`, individual amounts are not live, focus remains governed by Summary navigation, narrow/wide hierarchy remains readable, and all controls are 48 by 48 CSS pixels
**And** Editorial Contrast warning/text/rule states and native parity hold.

**Requirements:** SPEC-FR67, SPEC-FR69..SPEC-FR74; SPEC-NFR4..SPEC-NFR5, SPEC-NFR10..SPEC-NFR15, SPEC-NFR25..SPEC-NFR30, SPEC-NFR32..SPEC-NFR34; snapshot release, exact context/rate decoding, provider bounds, single-flight, exact accumulation, joint quantization, deterministic disclosure, and whole-section failure requirements; UX contracts: `UX-SHELL-01`, `UX-TARGET-01`, `UX-FOCUS-01`, `UX-STATUS-01`, `UX-RESPONSIVE-01`, `UX-VISUAL-01`.

**UX acceptance evidence:** Tests cite the listed UX contracts and verify source/converted hierarchy, stable conversion status/`aria-busy`, no live amount rows, provisional disclosure, exact conserved display, 320px/400% and wide rendering, focus retention, and native/enhanced parity. Brownfield disposition: retain conforming provider/domain boundaries; remove standalone rate milestones, floating-point JSON decoding, provider I/O under database transactions, per-Spending displayed rounding, partial converted results, and manual Retry UI.

### Story 4.3: Preserve Source Totals When Conversion Is Unavailable

As the administrator,
I want monthly Summary to degrade safely when fresh rates cannot be obtained,
So that I can still use exact source totals and the ledger without seeing misleading partial conversion.

**Acceptance Criteria:**

**Given** a fresh fixed-past Historical request fails
**When** the stable cache contains a prior quote for the exact `(source, target, R, F)` key
**Then** that quote remains eligible without an age limit and is returned with a stale warning
**And** no quote from another requested/fetch context is substituted.

**Given** a provisional future Historical request fails
**When** the refreshable cache contains a prior quote for the same Source/Target and original future `R`
**Then** it is eligible inclusively through seven UTC calendar days after its prior effective fetch date `F` and is disclosed as stale plus provisional
**And** an older or differently keyed quote is rejected.

**Given** refreshable cache entries cross UTC rollover or exceed stale eligibility
**When** conversion is requested
**Then** Debtor attempts the current fetch context and uses no expired fallback
**And** deterministic LRU eviction cannot make an ineligible quote eligible.

**Given** no fresh or eligible stale quote exists for any required monthly context
**When** Summary renders
**Then** application cause remains retryable exchange-rate `Unavailable`, the whole converted section shows one sanitized retryable warning, and no converted Payer or Group total is displayed
**And** source totals, Group navigation, and Spending CRUD remain usable.

**Given** every quote exists but checked conversion, accumulation, or quantization fails
**When** Summary renders
**Then** the inward cause remains `Calculation` while the rendered converted section uses the same whole-section unavailable presentation
**And** no partial result, zero substitution, panic, or raw cause reaches the administrator.

**Given** Frankfurter remains unavailable during startup, readiness checks, or ordinary ledger requests
**When** those paths execute
**Then** provider state does not affect socket admission, `/readyz`, or CRUD dispatch
**And** only conversion consumers observe rate unavailability.

**Given** fresh fixed-past Historical evidence succeeds
**When** cached
**Then** it enters a stable class keyed by exact `(source, target, R, F)`; future/current-class evidence enters a separate refreshable class
**And** each class is capped at 4,096 contexts with deterministic LRU eviction and per-key single-flight.

**Given** UTC date rolls over
**When** refreshable contexts are requested
**Then** they use the new calculation/fetch date while stable past contexts may remain until deterministic eviction
**And** eviction may refetch but cannot cross keys, change one immutable quote bundle, or make ineligible evidence eligible.

**Given** a fresh fixed-past request fails
**When** exact-key stable evidence exists
**Then** it remains stale-eligible without age limit and Summary displays complete converted totals with one explicit stale notice
**And** no other requested/fetch context substitutes.

**Given** a fresh future Historical request fails
**When** refreshable evidence matches Source/Target and original future `R`
**Then** it is eligible inclusively through seven UTC days after prior `F` and complete totals display one stale-plus-provisional notice
**And** older/different evidence is rejected.

**Given** every quote remains fresh or eligible stale
**When** completion/cache order varies
**Then** totals, evidence ordering, stale/provisional classification, warnings, and LRU behavior remain deterministic
**And** tests coordinate calls explicitly without sleeps.

**Given** any required context lacks fresh/eligible stale evidence
**When** Summary renders
**Then** inward cause remains retryable `Unavailable`, prior converted values are replaced by one retryable unavailable state, and no converted Payer/Group total remains
**And** Story 4.1 source totals, shell, and Spending CRUD remain available.

**Given** all evidence exists but checked conversion/aggregation/quantization fails
**When** Summary renders
**Then** inward cause remains `Calculation` while the same whole converted region is unavailable with no partial/zero/panic/raw cause
**And** source totals and CRUD remain usable.

**Given** converted state changes among Updating, Stale, Provisional, and Unavailable
**When** native/enhanced Summary renders at 320px/400% zoom
**Then** one Group-level ruled notice uses polite atomic announcement and `aria-busy`, stale/provisional reason is explicit text, no manual Retry exists, and revisiting Summary retries automatically
**And** controls, focus, source totals, and Editorial Contrast warning states remain stable.

**Requirements:** SPEC-FR72, SPEC-FR74, SPEC-FR76..SPEC-FR77; SPEC-NFR4..SPEC-NFR5, SPEC-NFR10, SPEC-NFR13, SPEC-NFR25..SPEC-NFR34; stable/refreshable caches, deterministic LRU/rollover, Historical/future stale fallback, whole-region degradation, cause separation, and provider-independent operation requirements; UX contracts: `UX-SHELL-01`, `UX-TARGET-01`, `UX-FOCUS-01`, `UX-STATUS-01`, `UX-RESPONSIVE-01`, `UX-VISUAL-01`.

**UX acceptance evidence:** Tests cite the listed UX contracts and verify Updating/Stale/Provisional/Unavailable transitions, one status/`aria-busy`, exact warning text, no partial retained converted rows, source/CRUD continuity, automatic revisit retry, 320px/400% rendering, and native/enhanced parity. Brownfield disposition: retain bounded rate adapter owners where conforming; remove unbounded/mixed caches, cross-context fallback, manual Retry controls, provider-gated readiness/CRUD, partial stale displays, and raw cause leakage.

## Epic 5: Calculate Debts, Settle, and Safely Retire Identities

The administrator can calculate all-time Historical or Current Balances, receive complete advisory Settlement Transfers, and archive, browse, or restore Participants without losing referenced history.

### Story 5.1: Calculate Exact Historical Balances

As the administrator,
I want all-time Participant Balances calculated at each Spending's historical context,
So that I can see an exact zero-sum picture of who is owed and who owes.

**Acceptance Criteria:**

**Given** a Group has all-time Spending history
**When** Historical Debts calculation begins
**Then** one SQLite snapshot materializes Group Currency, every Group-owned Participant, every complete Spending, and all Payer/Shares before committing the read transaction
**And** no network request holds the database transaction.

**Given** the immutable ledger snapshot is materialized at UTC calculation date/time `C`
**When** quote contexts are assembled
**Then** Historical mode is selected by default, each Spending uses original date `R` with `F = min(R, C)`, future dates use provisional current evidence, and same-currency contexts synthesize exact `1`
**And** unique contexts are deduplicated into one immutable quote bundle.

**Given** complete exact quote evidence is available
**When** Participant positions are calculated
**Then** each Payer receives its converted paid Total, each Participant is charged its converted Share, every Group-owned Participant with no activity starts and remains at exact zero, all operations use checked `Decimal`, and archived identities remain included
**And** input/provider completion order cannot change exact pre-quantized positions.

**Given** exact positions require Group Currency quantization
**When** final Balances are produced
**Then** largest signed-remainder allocation with Participant-ID tie-breaking quantizes them together at target precision and preserves an exact zero sum
**And** no individual rounding step can create or destroy value.

**Given** the same immutable snapshot/date/quote bundle is calculated repeatedly
**When** row order or provider completion order differs
**Then** ordered Balances, evidence, and warnings are identical
**And** domain property tests verify exact zero sum, checked boundaries, and deterministic ties.

**Given** any stored aggregate is corrupt, any required quote lacks eligible evidence, or conversion/aggregation/quantization fails
**When** Debts is requested
**Then** missing quote returns retryable `503`, checked calculation returns one sanitized calculation failure, and neither path exposes partial Balances or Transfers
**And** source monthly Summary and ledger CRUD remain available.

**Given** Historical Debts succeeds
**When** the view renders
**Then** it discloses Historical mode, UTC calculation time, target Group Currency, deterministically ordered unique rates, and stale/provisional/synthetic warnings
**And** it uses the shared accessible full-page path with no custom JavaScript.

**Given** Historical Debts exceeds its assigned route limit
**When** 90 seconds elapse before completion
**Then** the safe read is cancelled with sanitized retryable feedback and no persisted financial state
**And** this verifies the Debts-specific portion of SPEC-FR102 without affecting mutation timeout policy.

**Given** Debts opens
**When** Historical calculation begins through native navigation
**Then** the five-link shell marks Debts current, Historical is the checked default, and the stable result heading receives forward focus while one scoped status announces Updating
**And** enhanced replacement retains focus on the selected mode control instead of moving it.

**Given** Historical calculation succeeds
**When** Financial results render
**Then** one Balance per Participant appears before Settlement and disclosure sections, each amount includes symbol plus ISO code and explicit direction/sign text, and unique rates show context left/equation right in deterministic order
**And** stale/provisional/synthetic context is explicit text rather than color alone.

**Given** the Group has no Spendings
**When** Debts renders
**Then** it states that no Spendings exist, retains the mode control and calculation context, shows one exact zero Group Currency Balance for every Group-owned Participant, and shows no Settlement Transfer
**And** the zero Balances are complete derived results rather than fabricated activity or an unavailable state.

**Given** calculation is Updating, Ready, stale/provisional, timed out, or unavailable
**When** state changes
**Then** one stable polite atomic status owns the transition, individual amounts are not live, and unavailable replaces prior results with one no-partial block plus attempted context
**And** enhanced Debts uses HTMX's request class for its scoped Updating placeholder without dynamic `aria-busy`, retains the activated rate-mode radio for enhanced success and expected enhanced errors, and introduces no application-owned HTMX event handler, client-side financial state, or imperative post-swap behavior.

**Given** Debts renders at 320px/400% zoom or wide composition
**When** mode controls, long names, rates, warnings, and amounts wrap
**Then** all controls remain 48 by 48 CSS pixels, mode/Balance/Transfer/disclosure order remains intact, no page horizontal scroll occurs, and Editorial Contrast states remain readable and motion-free
**And** native/enhanced paths are equivalent.

**Requirements:** SPEC-FR41, SPEC-FR74, SPEC-FR78..SPEC-FR83, SPEC-FR102; SPEC-NFR2, SPEC-NFR4..SPEC-NFR5, SPEC-NFR10, SPEC-NFR12..SPEC-NFR16, SPEC-NFR25..SPEC-NFR30, SPEC-NFR32..SPEC-NFR34; immutable all-time snapshot, Historical evidence, exact-zero-sum quantization, disclosure, no-partial, and Debts timeout requirements; UX contracts: `UX-SHELL-01`, `UX-TARGET-01`, `UX-FOCUS-01`, `UX-STATUS-01`, `UX-RESPONSIVE-01`, `UX-VISUAL-01`.

**UX acceptance evidence:** Tests cite the listed UX contracts and verify Historical default, native heading/enhanced radio focus, result order, exact direction/currency text, empty/no-partial states, status/`aria-busy`, automatic revisit retry, and 320px/400% responsive/visual parity. Brownfield disposition: retain conforming complete snapshot/rate capability from Epic 4; remove partial debt projections, SQL monetary calculations, active-only identities, persisted Historical results, manual Retry, and alternate debt routes.

### Story 5.2: Recalculate Balances at Current Rates

As the administrator,
I want to recalculate all-time Balances using the current UTC rate context,
So that I can compare historical obligations with what settlement means today.

**Acceptance Criteria:**

**Given** the Debts view defaults to Historical mode
**When** the administrator selects Current mode through a native request
**Then** every Spending uses the calculation UTC date as its requested/fetch context regardless of Spending date
**And** the selected mode is not persisted, so a new default calculation remains Historical.

**Given** multiple Spendings share a Source/Target pair in Current mode
**When** quote contexts are assembled
**Then** they deduplicate to the current calculation-date context, same-currency pairs synthesize exact `1`, and provider concurrency/single-flight bounds remain enforced
**And** one immutable quote bundle is used for that calculation.

**Given** a fresh Current quote request fails after UTC rollover
**When** the refreshable cache contains prior current-class evidence for the same Source/Target pair
**Then** the latest prior current-class quote is eligible inclusively through seven UTC calendar days after its prior effective fetch date
**And** it is disclosed as stale with its original evidence.

**Given** prior Current evidence is older than seven UTC days or belongs to another pair/class
**When** fresh resolution fails
**Then** no fallback is used and Debts returns retryable `503` with no partial Balances or Transfers
**And** Historical fixed-past evidence is never borrowed for Current mode.

**Given** complete Current evidence is available
**When** all-time positions are converted and quantized
**Then** the same checked exact arithmetic, deterministic ordering, largest signed-remainder rule, Participant-ID ties, and exact-zero-sum invariant used by Historical mode apply
**And** archived referenced identities remain included.

**Given** Current Debts succeeds
**When** the result is rendered
**Then** it discloses Current mode, calculation UTC time, target Group Currency, unique current/synthetic rates, and stale warnings
**And** no Spending Source Currency, allocation, or persisted setting is changed.

**Given** the provider returns a revised rate on a later calculation
**When** no persisted rate evidence is required by the ordinary Debts view
**Then** the later calculation may reflect the revision while each individual result remains internally immutable and reproducible from its displayed context
**And** no manual refresh or saved Current mode is introduced.

**Given** Debts defaults to Historical
**When** Current is selected through the safe native form
**Then** Current remains encoded in the URL for that result but is not persisted, and the native forward response focuses the result heading
**And** an enhanced replacement retains focus on the selected Current radio and announces status instead of moving focus.

**Given** an enhanced Current calculation is pending or replaces a prior complete result
**When** the mode form requests the replacement
**Then** HTMX's request class hides prior financial content and shows one scoped Updating placeholder
**And** the completed server-rendered replacement restores complete results or the scoped no-partial failure without dynamic `aria-busy`, retained client-side financial state, custom application JavaScript, inline script attributes, or a custom HTMX extension.

**Given** Current succeeds with fresh or eligible stale evidence
**When** results render
**Then** Current is visibly/programmatically selected, Balances preserve exact direction/currency presentation, disclosure identifies current/synthetic evidence and stale dates, and one Group-level warning announces once
**And** no individual amount is live.

**Given** Current is unavailable
**When** native/enhanced result renders
**Then** no partial Balance/Transfer remains, enhanced expected errors retain the activated rate-mode control while the scoped server-rendered status announces the failure, native full-page errors may autofocus their heading, and revisiting/reselecting safely recalculates
**And** no manual Retry, persisted preference, application-owned HTMX event handler, client-side financial state, or imperative post-swap behavior appears.

**Requirements:** SPEC-FR75..SPEC-FR83; SPEC-NFR4..SPEC-NFR5, SPEC-NFR10, SPEC-NFR12..SPEC-NFR13, SPEC-NFR25..SPEC-NFR30, SPEC-NFR32..SPEC-NFR34; non-persisted Current mode, current-class stale fallback, seven-day eligibility, exact-zero-sum reuse, disclosure, and no-partial requirements; UX contracts: `UX-SHELL-01`, `UX-TARGET-01`, `UX-FOCUS-01`, `UX-STATUS-01`, `UX-RESPONSIVE-01`, `UX-VISUAL-01`.

**UX acceptance evidence:** Tests cite the listed UX contracts and verify URL/non-persistence, selected-radio/native result focus, CSS-only Updating placeholder, final stable status semantics, absence of inline handlers/custom scripts, stale/unavailable disclosure, no partial output, 48-by-48 mode controls at 320px/400% zoom, and native/enhanced parity. Brownfield disposition: retain Epic 4 refreshable cache/rate owners; remove persisted Current preferences, mixed Historical/Current fallback, manual refresh, mode-specific duplicate arithmetic, client-side financial state, and focus-moving enhanced swaps.

### Story 5.3: Derive Complete Advisory Settlement Transfers

As the administrator,
I want advisory Transfers derived from exact all-time Balances,
So that I know who could pay whom to settle the Group without recording repayment state.

**Acceptance Criteria:**

**Given** a successful Historical or Current calculation produced quantized exact-zero-sum Balances
**When** Settlement runs
**Then** it consumes that complete immutable Balance set only after financial calculation succeeds
**And** no Transfer is produced from partial, unquantized, corrupt, or unavailable Balances.

**Given** positive creditors and negative debtors exist
**When** deterministic greedy matching begins
**Then** each side is ordered by descending absolute Balance with ascending Participant ID ties and each Transfer amount is the checked positive minimum needed between the current pair
**And** completion order or unordered input cannot alter output.

**Given** `n` Participants are included in the Balance calculation
**When** Settlement completes
**Then** every Balance is fully settled, no Participant pair repeats, every Transfer is positive and target-precision-valid, and at most `n - 1` Transfers are returned
**And** the UI and documentation do not claim globally minimal Transfer count.

**Given** all Participant Balances are exactly zero
**When** Settlement runs
**Then** Debtor renders an accessible settled empty state with no fabricated Transfer
**And** no persistence or provider call occurs beyond the completed Balance calculation.

**Given** checked arithmetic, conservation, positivity, pair uniqueness, or completion fails
**When** Settlement evaluates the result
**Then** it returns the fixed sanitized calculation failure with no partial Transfers
**And** it never panics, defaults an amount to zero, or silently drops a Participant.

**Given** advisory Transfers are rendered
**When** the administrator reviews them
**Then** each row clearly identifies paying Participant, receiving Participant, positive Group Currency amount, selected mode, and the shared calculation evidence/warnings
**And** archived referenced identities use current names and remain included.

**Given** the administrator revisits Debts or changes modes
**When** Transfers are recalculated
**Then** no repayment, paid/settled status, checkpoint, date range, or transfer-completion record is persisted
**And** Transfers remain purely derived advice.

**Given** Historical or Current Balances complete
**When** Debts renders Settlement Transfers
**Then** Transfers appear after Balances and before disclosure, each row explicitly states "from [Participant] to [Participant]" plus positive Group Currency symbol/ISO amount, and archived identities carry visible text
**And** color or sign alone never communicates direction.

**Given** every Balance is zero
**When** Settlement renders
**Then** a factual settled empty state appears in the Transfer section with no fabricated row, completion badge, celebratory motion, or persisted state
**And** mode and disclosure remain visible.

**Given** Settlement fails checked validation
**When** Debts maps the result
**Then** the entire financial result follows the Story 5.1 unavailable/no-partial contract rather than showing Balances without trustworthy Transfers
**And** one stable status announces the sanitized failure.

**Given** Transfers render at 320px/400% zoom
**When** long names/directions/amounts wrap
**Then** reading order remains payer, receiver, amount; exact currency stays intact; no page horizontal scroll occurs; and Editorial Contrast ruled rows preserve required contrast
**And** native/enhanced output remains identical.

**Requirements:** SPEC-FR41, SPEC-FR83..SPEC-FR86; SPEC-NFR5, SPEC-NFR10, SPEC-NFR12, SPEC-NFR25, SPEC-NFR28..SPEC-NFR34; deterministic greedy Settlement, positivity, pair uniqueness, completeness, `n - 1` bound, no partial output, and advisory-only requirements; UX contracts: `UX-SHELL-01`, `UX-TARGET-01`, `UX-FOCUS-01`, `UX-STATUS-01`, `UX-RESPONSIVE-01`, `UX-VISUAL-01`.

**UX acceptance evidence:** Tests cite the listed UX contracts and verify section order, explicit from/to direction, settled empty state, no partial Settlement failure, archived names, 320px/400% wrapping, status behavior, and no completion animation/state. Brownfield disposition: retain conforming deterministic Settlement rules; remove repayment/paid/checkpoint persistence, transfer-minimization claims, partial Transfer rendering, unordered map output, and color-only debt direction.

### Story 5.4: Archive a Zero-Balance Participant Safely

As the administrator,
I want to archive a Participant only when their Historical Balance is exactly zero in an unchanged context,
So that active choices stay clean without hiding unsettled obligations or racing ledger changes.

**Acceptance Criteria:**

**Given** Manage evaluates an active Participant
**When** the complete Historical engine shows exact zero Balance and required evidence is currently available
**Then** Manage may expose Archive with factual eligibility text, but the page does not persist/cache eligibility or authorize mutation
**And** nonzero or rate-blocked states expose no bypass.

**Given** Archive is activated
**When** the confirmation page opens
**Then** it names the Participant and Group, states archive is reversible and removes the identity from new allocations while preserving history, and carries only an allow-listed Manage return plus stable invoker focus ID
**And** Cancel returns to the exact Participant Archive control with no calculation or mutation.

**Given** protected Confirm is submitted
**When** exactly one archive attempt dispatches
**Then** the application captures a new immutable all-time snapshot/generation, UTC context, and Historical quote bundle using Story 5.1 rules after dispatch; provider I/O occurs after snapshot release
**And** confirmation state never substitutes for server-owned eligibility.

**Given** required Historical evidence is unavailable or only ineligible stale evidence exists
**When** archive eligibility is calculated
**Then** Debtor returns sanitized retryable feedback with no archive commit or partial Balance, focuses the invoking Archive control/scoped status, and keeps the already dispatched attempt and reserved submission token terminal
**And** ordinary Participant/Spending operations remain available and a retry requires a newly rendered confirmation/token.

**Given** the target Participant's final quantized Group Currency Balance is nonzero
**When** eligibility is evaluated
**Then** archival is refused with clear sanitized no-change feedback, no commit, and focus on the invoking Archive control/scoped status
**And** Settlement advice is not treated as recorded repayment.

**Given** the target Participant's complete Historical Balance is exactly zero
**When** the archive mutation reaches final admission through the write gate
**Then** one transaction revalidates Participant/Group lifecycle, ledger generation, current UTC date, and eligibility of every quote in the immutable bundle before changing active status
**And** any mismatch returns retryable feedback with no archive commit.

**Given** a Spending or other ledger mutation commits between calculation snapshot and archive admission
**When** final generation revalidation runs
**Then** archival is rejected with no commit and focus on the invoking Archive control/scoped status even if a newly calculated Balance might also be zero
**And** a later explicit attempt must take a new snapshot and quote bundle.

**Given** UTC date rolls over or refreshable quote evidence expires before commit
**When** final revalidation runs
**Then** archival is rejected with no state change and focus on the invoking Archive control/scoped status
**And** the attempt never silently substitutes a newly fetched or revised provider quote.

**Given** every eligibility condition remains unchanged
**When** archive commits
**Then** only the Participant's lifecycle state changes atomically, the identity and every historical reference remain stored, and Manage returns with Participants heading focused plus one announcement
**And** no rate evidence, Balance, Transfer, or repayment state is persisted and the consumed submission token prevents a second dispatch.

**Given** the process-local mutation epoch established by Story 2.1 is consumed by Participant archival
**When** snapshot capture and final admission execute
**Then** the attempt captures the current epoch and final admission requires it unchanged, while deterministic race tests coordinate held snapshots/gates without sleeps
**And** no database revision column, parallel generation owner, or optimistic user-facing stale-edit path is added.

**Given** the owning Group is archived, CSRF/token validation fails, or the archive token is replayed
**When** the request is processed
**Then** archived Group access returns pre-use-case `409`, other pre-dispatch failures invoke no use case, and replay dispatches at most once
**And** post-dispatch work is not cancelled by a generic timeout.

**Given** confirmation/Manage states render at 320px/400% zoom
**When** long names, Balance, eligibility, and actions wrap
**Then** every control remains 48 by 48 CSS pixels, Historical Balance precedes eligibility and equal actions, coral archive state pairs with explicit text, and no clipping/page horizontal scroll occurs
**And** native/enhanced paths remain equivalent.

**Requirements:** SPEC-FR30, SPEC-FR37..SPEC-FR39; SPEC-NFR3, SPEC-NFR10, SPEC-NFR13, SPEC-NFR15..SPEC-NFR16, SPEC-NFR22, SPEC-NFR25, SPEC-NFR31..SPEC-NFR34; immutable attempt, exact-zero eligibility, generation/date/quote revalidation, no persisted evidence, write-gate, and deterministic race requirements; UX contracts: `UX-SHELL-01`, `UX-TARGET-01`, `UX-FOCUS-01`, `UX-STATUS-01`, `UX-CONFIRM-01`, `UX-RESPONSIVE-01`, `UX-VISUAL-01`.

**UX acceptance evidence:** Tests cite the listed UX contracts and verify eligible/ineligible/rate-blocked Manage states, uncached confirmation, allow-listed Cancel focus, new server-owned attempt on Confirm, one-shot pending state, revalidation failure focus, success heading focus, 48-by-48 geometry, and native/enhanced parity. Brownfield disposition: retain the shared Historical engine/write gate; remove client-authorized or cached eligibility, archive without confirmation, rate-evidence persistence, independent calculation paths, and unguarded Participant lifecycle routes.

### Story 5.5: Browse and Restore Archived Participants

As the administrator,
I want archived Participants separated from active choices and restorable in context,
So that historical identities remain available without cluttering new allocations.

**Acceptance Criteria:**

**Given** a Participant is archived
**When** active Group Manage or a new Spending form is rendered
**Then** the identity is absent from active Participant lists, Payer choices, and new Share choices
**And** it is available through a separate contextual archived-Participants view inside its owning Group.

**Given** the archived-Participants view is opened
**When** identities are displayed
**Then** each remains readable with current name/color and a protected restore action, and no independent delete action exists
**And** the page uses the shared accessible, responsive, native full-page behavior.

**Given** an archived Participant is referenced by Spending history
**When** Transactions, Spending detail, monthly Summary, Historical/Current Balances, or Settlement Transfers are rendered
**Then** the identity remains included wherever its historical data contributes and resolves its current name
**And** archival never removes or rewrites Payer/Share records.

**Given** an existing Spending containing an archived identity is edited
**When** its Exact allocation form opens
**Then** the identity remains available only in the same stored Payer or Share role under Story 3.4 rules
**And** it cannot be introduced into a new or changed role.

**Given** a valid protected restore request targets an archived Participant in an active Group
**When** restore dispatches
**Then** the write gate atomically marks the identity active and redirects with `303` to Group Manage
**And** no Balance calculation, provider request, quote check, or ledger-generation eligibility check is performed.

**Given** a restored Participant is shown after commit
**When** active Group and new Spending views reload
**Then** the identity returns to active lists and becomes eligible for new Payer/Share selection
**And** all historical relationships remain unchanged.

**Given** the owning Group is archived or a restore request uses another Group's Participant ID
**When** the route is processed
**Then** archived Group mutation returns `409` before dispatch and ownership mismatch is rejected without revealing cross-Group details
**And** no state change occurs.

**Given** restore is replayed, races another lifecycle mutation, or persistence fails
**When** shared token/gate/transaction handling executes
**Then** at most one valid lifecycle change commits, failures are sanitized, and no identity is deleted or duplicated
**And** deterministic tests verify active/archived list membership and historical inclusion.

**Given** all Participants are archived
**When** active Manage and the Group shell render
**Then** active roster is empty, Add Spending remains disabled with distinct no-active-Participant guidance and a 48-by-48 Archived Participants recovery link
**And** archived identities are not mixed into active controls.

**Given** Archived Participants opens
**When** the contextual view renders at 320px/400% zoom
**Then** each row visibly says "Archived," shows current name and supplementary outlined color marker, exposes one protected 48-by-48 Restore action, and provides no delete action
**And** empty archived state provides a safe 48-by-48 return to Manage.

**Given** Restore is activated
**When** dispatch is pending and then commits
**Then** the initiator is unavailable under one scoped pending status, no Balance/provider/generation check occurs, and the canonical Manage response focuses the restored Participant row/action with one announcement
**And** replay cannot dispatch twice.

**Given** Restore fails or the owning Group is archived
**When** the response renders
**Then** failure focuses the invoking Restore control/status, archived-Group mutation returns pre-dispatch `409`, and no cross-Group identity detail is revealed
**And** the identity remains archived and readable.

**Given** historical surfaces render an archived Participant
**When** Transactions, Summary, Debts, or edit forms display it
**Then** current name plus visible associated "Archived" text remains, financial roles/facts stay unchanged, and color alone never communicates state
**And** restoring changes only active eligibility, not historical presentation.

**Requirements:** SPEC-FR30, SPEC-FR35, SPEC-FR40..SPEC-FR42; SPEC-NFR3, SPEC-NFR10, SPEC-NFR15..SPEC-NFR16, SPEC-NFR22, SPEC-NFR25, SPEC-NFR28..SPEC-NFR34; contextual archived views, historical inclusion, unconditional restore, active allocation eligibility, no-delete, and lifecycle concurrency requirements; UX contracts: `UX-SHELL-01`, `UX-TARGET-01`, `UX-FOCUS-01`, `UX-STATUS-01`, `UX-RESPONSIVE-01`, `UX-VISUAL-01`.

**UX acceptance evidence:** Tests cite the listed UX contracts and verify all-archived setup guidance, separate archived view/empty state, 48-by-48 controls at 320px/400% zoom, exact pending/failure/success focus, direct restore without calculation, historical visible state, and native/enhanced parity. Brownfield disposition: retain stable Group-owned identities/history; remove mixed active/archived lists, independent delete, restore eligibility checks, global Participant views, and color-only archive state.
