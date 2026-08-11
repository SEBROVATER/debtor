---
stepsCompleted:
  - step-01-validate-prerequisites
  - step-02-design-epics
  - step-03-create-stories
  - step-04-final-validation
inputDocuments:
  - _bmad-output/specs/spec-debtor/SPEC.md
  - _bmad-output/specs/spec-debtor/glossary.md
  - _bmad-output/planning-artifacts/prds/prd-debtor-2026-08-10/addendum.md
  - specs/design.md
  - _bmad-output/project-context.md
---

# debtor - Epic Breakdown

## Overview

This document provides the complete epic and story breakdown for debtor, decomposing the canonical SPEC and all declared companions into implementable stories.

## Requirements Inventory

### Functional Requirements

FR1: Debtor provides a private ledger operated by exactly one administrator authenticated with one configured password and without usernames, registration, participant login, tenants, or multi-user authorization.

FR2: Debtor refuses startup before database connection or migration when `APP_ADMIN_PASSWORD_HASH` is missing, exceeds 256 encoded bytes, or is not a canonical bounded Argon2id v19 PHC hash with valid `m`, `t`, and `p` parameters.

FR3: Debtor requires a session-backed CSRF token to render and submit the login workflow.

FR4: Debtor authenticates the administrator only after the submitted password is successfully verified against the configured password hash.

FR5: Successful login rotates the session identifier and CSRF token, durably establishes the authenticated session before setting a cookie or redirecting, and emits no authenticated cookie if persistence fails.

FR6: Anonymous login sessions expire after ten minutes of inactivity and are limited to 4,096 live sessions without evicting authenticated sessions.

FR7: Authenticated sessions expire after 30 days of inactivity, refresh on every request, and are limited to 32 live sessions without eviction.

FR8: A correct login attempted while authenticated-session capacity is full flushes its anonymous session and returns retryable `503 Service Unavailable`.

FR9: Restarting Debtor invalidates every anonymous and authenticated session.

FR10: Logging out flushes the current session and revokes its authenticated access.

FR11: Session cookies are HTTP-only and `SameSite=Strict`, with secure cookies required outside debug/local operation.

FR12: Login permits at most five post-CSRF password-verification attempts per trusted client IP in any rolling five-minute window.

FR13: An unseen login client receives retryable `429 Too Many Requests` when the 4,096-client login-limiter capacity is full.

FR14: Anonymous users are denied access to all ledger pages and ledger mutations.

FR15: Every unsafe request, including login, requires exactly one valid session-backed synchronizer CSRF token; missing, duplicate, malformed, or incorrect tokens are rejected before route parsing, password verification, or use-case dispatch.

FR16: Every rendered unsafe form carries a bounded, expiring, session-bound, single-use submission token distinct from its CSRF token.

FR17: Anonymous submission tokens are limited to one per session, 4,096 total, and ten minutes of inactivity; authenticated tokens are limited to 32 per session, 1,024 total, and a 30-minute absolute lifetime.

FR18: Missing, unknown, expired, reserved, or consumed submission tokens return `409 Conflict` without invoking the requested use case.

FR19: Validation rejected before dispatch preserves the submission token, while a token reserved immediately before dispatch remains consumed after exactly one attempt regardless of commit, rollback, task failure, or response delivery.

FR20: The administrator can create a Group by supplying a trimmed, non-empty name of at most 100 Unicode characters.

FR21: A newly created Group has `USD` as its initial Group Currency and opens in its Manage section.

FR22: An established Group opens in its Summary section.

FR23: The administrator can edit a Group name and freely change its Group Currency among `USD`, `EUR`, `RUB`, `KGS`, `TRY`, `KZT`, `UZS`, `CNY`, `KRW`, `JPY`, `OMR`, and `TJS`.

FR24: The administrator can archive and restore Groups.

FR25: Active Group lists exclude archived Groups, which remain accessible through a separate contextual archived view.

FR26: Archived Groups remain readable but hide all mutation and settings controls except the permitted Group restore action.

FR27: Direct form or mutation requests against an archived Group, other than the permitted Group restore route, return `409 Conflict` without invoking a use case.

FR28: A Group with no Spendings can be deleted together with its unreferenced Group-owned Participants.

FR29: A Group containing any Spending cannot be deleted and can only be archived.

FR30: The administrator can add, edit, archive, and restore Participants within their owning Group.

FR31: Every Participant belongs to exactly one Group, is never reusable across Groups, and is not exposed through a global Participant-management surface.

FR32: Participant names are trimmed, non-empty, and at most 100 Unicode characters.

FR33: Participant colors use normalized `#RRGGBB` values.

FR34: A new Participant form suggests a varied valid color while allowing the administrator to choose another color.

FR35: Active Participant lists exclude archived Participants, which remain available through contextual archived views.

FR36: Participants cannot be independently deleted through the application.

FR37: A Participant can be archived only when a complete all-time Historical-mode calculation gives that Participant an exact zero Balance in Group Currency.

FR38: Participant archival commits only if the ledger, UTC calculation date, and rate eligibility remain unchanged throughout the archival attempt.

FR39: Missing or ineligible exchange-rate evidence blocks Participant archival with retryable feedback and no state change.

FR40: Restoring a Participant does not require a Balance or exchange-rate eligibility check.

FR41: Archived Participants remain included wherever their historical Spendings affect history, summaries, Balances, or Settlement Transfers.

FR42: Historical views display a Participant's current name after that Participant is renamed.

FR43: The administrator can create, inspect, edit, and delete a dated Spending within a Group.

FR44: Each Spending contains a trimmed, non-empty description of at most 200 Unicode characters, one supported category, one Source Currency, one positive Total, exactly one Payer, and one or more Participant Shares.

FR45: Supported Spending categories are `food`, `transport`, `housing`, `fun`, `shopping`, `bills`, `health`, and `other`.

FR46: Spending dates parse strictly as `YYYY-MM-DD` and are on or after `2025-01-01`.

FR47: Spending Totals, Payer amounts, and Share amounts are positive and no greater than `999_999_999_999`.

FR48: Monetary input permits zero minor units for JPY and KRW, three for OMR, and two for every other supported currency, rejecting excess precision rather than rounding it.

FR49: Exactly one active Participant owned by the Spending's Group is selected as Payer and pays exactly the Spending Total.

FR50: Spending Shares are non-empty, Participant-unique, positive, owned by the Spending's Group, and sum exactly to the Total in Source Currency minor units.

FR51: Spending allocation offers only Proportional and Exact Share modes.

FR52: A new Spending form starts with an empty description and Total, the current UTC date, Source Currency equal to Group Currency, no category, no Payer, and every active Participant selected for sharing.

FR53: Selecting a Payer on a new Spending initially assigns that Payer the full Total paid.

FR54: Proportional mode initially assigns weight `1` to every active Participant and permits Participants to be deselected.

FR55: Every selected Proportional weight is a positive decimal no greater than `1,000,000` with at most six fractional digits.

FR56: Proportional allocation deterministically distributes exact minor units by submitted weight, assigns residual units by descending remainder with ascending Participant ID as the tie-breaker, and uses identical results for Preview and commit.

FR57: Exact mode initially divides Total minor units equally among all active Participants and assigns residual units in ascending Participant ID order.

FR58: Exact mode permits Participant deselection and Share editing and displays the remaining or excess difference until selected Shares equal the Total exactly.

FR59: Allocation Preview displays each resulting exact Source Currency amount and produces no aggregate when normalization or checked allocation is invalid.

FR60: Editing an existing Spending opens in Exact mode with its stored Payer and Share amounts because allocation mode and Proportional weights are not persisted.

FR61: A Spending update may retain an archived Participant only in the same existing Payer or Share role and rejects introducing that Participant into a new role or changing the existing role.

FR62: Every Spending mutation either commits the complete Spending and all allocations or leaves the ledger unchanged.

FR63: Among valid concurrently admitted mutations, the last committed write determines the resulting ledger state without a stale-edit conflict.

FR64: The persistent Add Spending action opens a focused full-page form and, after successful commit, returns to Transactions with the committed row visible.

FR65: Ordinary Spending history uses fixed 25-item pages ordered newest first by Spending date and then descending Spending ID.

FR66: Spending detail remains readable for archived Groups and archived Participants.

FR67: The selected Group Summary shows exact Spending totals for the current UTC calendar month only.

FR68: The current-month source summary shows the Group Total and each Payer's paid Total grouped by original Source Currency without requiring exchange-rate conversion.

FR69: The current-month converted summary converts every included Spending from Source Currency to Group Currency using the rate context for that Spending's date.

FR70: Converted current-month values accumulate exactly per Payer before final target-currency quantization, and the displayed Group Total equals the exact sum of all displayed Payer totals.

FR71: Final converted Payer totals are quantized together by truncation toward zero and descending fractional remainder, with ascending Participant ID as the tie-breaker.

FR72: If any required quote is unavailable or checked conversion, aggregation, or quantization fails, the entire converted summary is withheld behind one sanitized unavailable warning while source totals and ordinary ledger CRUD remain available.

FR73: Debtor uses an exact synthetic rate of `1` without provider access when Source Currency equals Group Currency and discloses that rate.

FR74: Historical rate requests use the Spending date by default, while future Spending dates use the latest current rate and are marked provisional.

FR75: Current conversion mode uses the UTC calculation date for every Spending and is not persisted.

FR76: On rate-provider failure, Debtor may use the latest context-matching eligible prior quote and identifies it as stale.

FR77: A fixed past-date quote remains stale-eligible without an age limit, while current and future quotes remain eligible only through seven UTC calendar days after their effective fetch date.

FR78: The administrator can calculate every Participant's all-time Balance in Group Currency using either Historical mode or Current mode.

FR79: Historical Balance mode is the default and converts each Spending using its Spending-date context.

FR80: Balance results include archived historical identities, are deterministic, are quantized to Group Currency precision, and sum to exactly zero.

FR81: The debts view discloses the selected conversion mode, UTC calculation time, target Group Currency, every unique rate used, and all stale or provisional warnings.

FR82: If any required rate is unavailable, the debts view returns retryable `503 Service Unavailable` and exposes no partial Balances or Settlement Transfers.

FR83: If debt conversion, aggregation, quantization, or settlement fails, Debtor returns one sanitized calculation failure without substituting zero, panicking, or exposing partial Balances or Transfers.

FR84: Debtor derives advisory Settlement Transfers on demand from all-time Balances without recording repayments, paid state, settlement checkpoints, or transfer completion.

FR85: Settlement Transfers are deterministic, positive, pair-unique, and sufficient to settle every included Participant Balance.

FR86: Settlement orders matching by descending absolute Balance and then Participant ID and produces at most `n - 1` Transfers for `n` included Participants without claiming global transfer-count minimality.

FR87: Group, Participant, and Spending validation failures return `422 Unprocessable Entity`, display inline field errors, and retain every raw submitted value, including Participant color.

FR88: Successful mutations return `303 See Other` redirects.

FR89: Forms reject malformed, missing, duplicate, and unknown fields before route-specific parsing or use-case dispatch.

FR90: Every core interaction works through native links and forms without HTMX, while HTMX may progressively enhance those same full-page paths.

FR91: Enhanced expected `4xx` and `5xx` responses target a stable, programmatically announced status region.

FR92: The interface uses server-rendered semantic HTML and remains operable on current stable Chrome, Firefox, Safari, and Edge down to 320 CSS pixels.

FR93: Every control is keyboard-operable, programmatically labelled, and visibly focused with an indicator at least two CSS pixels thick and at least 3:1 contrast.

FR94: Normal text has at least 4.5:1 contrast, large text and meaningful controls or graphics at least 3:1 contrast, and inline errors are programmatically associated with their fields.

FR95: Login and authenticated HTML responses prevent caching and send `nosniff`, `no-referrer`, and the prescribed restrictive content-security policy.

FR96: Probe and static-asset routes neither create nor load sessions.

FR97: `/healthz` reports process liveness independently of ledger contents, exchange-rate availability, and ordinary user-traffic saturation.

FR98: `/readyz` checks SQLite and mandatory in-process cleanup supervisors but does not depend on exchange-rate-provider availability or ledger contents.

FR99: Failure of mandatory session-expiry or submission-token cleanup fails readiness, stops new admission, and initiates shutdown.

FR100: Login request bodies are limited to 8 KiB and other form request bodies to 256 KiB.

FR101: Debtor admits at most 64 concurrent user requests and four concurrent login requests while reserving a separate four-request budget for health and readiness probes.

FR102: Safe dynamic reads other than debts and login time out after 30 seconds, debts after 90 seconds, and probes after two seconds with a one-second SQLite readiness limit.

FR103: Ledger mutations have a 30-second pre-dispatch deadline, but once dispatched continue until a definitive commit or rollback result is known rather than being cancelled by a generic request timeout.

FR104: Graceful shutdown stops admission, drains HTTP for at most ten seconds, then waits for every dispatched mutation to finish before closing ledger storage.

FR105: After valid local configuration is supplied, `cargo run` creates or connects to the database, runs migrations, binds the configured address, reports a non-secret local URL, and supports graceful shutdown without Docker, a frontend build, manual migration, SQLx preparation, or exchange-rate-provider availability.

### NonFunctional Requirements

NFR1: Login bodies are limited to 8 KiB and other form bodies to 256 KiB; at most 64 user requests and four login requests run concurrently, with a separate four-request probe budget.

NFR2: Safe dynamic reads other than Debts and login have a 30-second timeout, Debts has a 90-second timeout, probes have a two-second outer timeout, and SQLite readiness has a one-second inner timeout.

NFR3: A 30-second absolute pre-dispatch deadline covers body extraction, authentication, CSRF, and asynchronous web prechecks; after dispatch, no generic application or edge timeout may cancel the mutation.

NFR4: Exchange-rate requests have a five-second connect timeout, 20-second total timeout, and 64 KiB response limit; at most four provider calls run globally or per debt calculation, identical uncached keys use single-flight, and each cache class holds at most 4,096 deterministic-LRU entries.

NFR5: All money and rates use exact `Decimal` and canonical SQLite `TEXT`; checked Rust owns parsing, conversion, validation, aggregation, quantization, and formatting, while floating point, lossy conversion, SQL monetary work, silent rounding, zero substitution, and partial results are forbidden.

NFR6: Every Total and persisted Payer or Share amount is positive, precision-valid, and at most `999_999_999_999`; JPY/KRW allow zero minor units, OMR three, and all other supported currencies two.

NFR7: Names and descriptions obey their Unicode limits, dates are strict and use UTC policy, colors are normalized, ledger IDs are positive `i64`, and UUIDs are restricted to session and CSRF randomness.

NFR8: Exactly one active Group-owned Participant pays the Total, and nonempty, unique, positive Shares conserve that Total exactly.

NFR9: Proportional and Exact allocation use checked integer/minor-unit arithmetic with the specified bounds, residual ordering, Participant-ID ties, and exact closure; Preview and commit use the same operation.

NFR10: Domain behavior and all output-affecting ordering are synchronous and deterministic, with Participant ID as the final tie-breaker and provider completion order unable to alter results or disclosures.

NFR11: Converted monthly values accumulate exactly without per-Spending rounding, Payer totals quantize together, and the displayed Group total is their exact sum; failure withholds the whole converted section.

NFR12: Balances quantize with largest signed remainder and preserve exact zero sum; Settlement is deterministic, positive, complete, pair-unique, and bounded by `n - 1` Transfers.

NFR13: Rate lookup, deduplication, caching, fallback, freshness, same-currency synthesis, and disclosure preserve the full `(source, target, R, F)` context and exact UTC eligibility rules.

NFR14: Repository decoding revalidates canonical monetary form and rejects malformed or noncanonical stored values as corruption rather than normalizing them.

NFR15: Complete Spending writes and all eligibility checks are transactional; complete aggregates come from one SQLite snapshot, and provider requests never hold a database transaction.

NFR16: Referenced identities survive archival, and Participant archival is admitted only from an unchanged immutable all-time Historical context with an exact zero Group Currency Balance.

NFR17: `APP_ADMIN_PASSWORD_HASH` is required, at most 256 encoded bytes, canonical Argon2id v19 with exactly bounded `m`, `t`, and `p`, a 16-64 byte decoded salt, and a 32-64 byte output; cheap validation precedes KDF and database work.

NFR18: Password verification concurrency is two; login allows five attempts per trusted client IP in five rolling minutes, tracks at most 4,096 active keys without eviction, and fails closed at capacity.

NFR19: Sessions are process-local, in-memory, server-side, HTTP-only, `SameSite=Strict`, securely cookie-bound outside debug, rotated durably on login, flushed on logout, and invalidated by restart.

NFR20: Anonymous sessions expire after ten inactive minutes and cap at 4,096 without authenticated eviction; authenticated sessions expire after 30 inactive days, refresh per request, and cap at 32 without eviction.

NFR21: Every unsafe request requires exactly one correct session-backed CSRF synchronizer token before password verification, route parsing, or dispatch.

NFR22: Separate bounded anonymous/authenticated submission-token stores enforce expiry, per-session limits, fail-closed capacity, atomic pre-dispatch reservation, and terminal single use.

NFR23: Login and authenticated HTML send no-store, nosniff, no-referrer, and the prescribed restrictive CSP; approved scripts use fixed routes, media types, immutable digest mappings, and nosniff.

NFR24: Forwarding headers are trusted only from configured immediate proxy CIDRs in one selected format; production validates a nonempty policy before admission and resolves client identity identically across edge protocols.

NFR25: Logs and user-facing errors exclude all credentials, session/security identifiers, client identity, SQL/database/provider diagnostics, monetary values, entity identifiers, URLs, query strings, and request-derived data; SQLite logs use only the fixed allowlists.

NFR26: Exchange-rate-provider availability never gates startup, readiness, or ledger CRUD, and financial failures produce no partial converted summaries, Balances, or Transfers.

NFR27: Health reports liveness; readiness checks only SQLite and mandatory supervisors; cleanup-supervisor failure fails readiness, stops admission, and initiates shutdown.

NFR28: Every control supports pointer-independent operation, programmatic labels, a two-CSS-pixel 3:1 focus indicator, required text/component contrast, and programmatically associated inline errors.

NFR29: The current stable Chrome, Firefox, Safari, and Edge are supported down to 320 CSS pixels, and every core interaction retains a native full-page path.

NFR30: The UI uses semantic server-rendered Askama HTML and vanilla CSS; only pinned self-hosted HTMX and its official `response-targets` extension are permitted, with no custom JavaScript, custom extensions, inline scripts, or script attributes.

NFR31: Shutdown stops admission, drains HTTP for at most ten seconds, then waits for all dispatched mutations before bounded checkpoint and pool close while preserving authoritative mutation outcomes and treating unknown outcomes as fatal.

NFR32: Structured safe error categories prevent raw adapter diagnostics from crossing inward or reaching HTTP; checked financial failures never panic, substitute zero, or return partial output.

NFR33: Unsafe Rust is forbidden, production avoids `unwrap` and `expect`, formatting remains clean, Clippy pedantic warnings are denied, and lint suppression remains narrow.

NFR34: Public APIs have rustdoc and fallible methods document `# Errors`; comments explain non-obvious constraints, changes remain minimal, and speculative abstractions or mocking frameworks are avoided.

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

No standalone UX design contract is included in this run. Actionable interaction, accessibility, responsive, and progressive-enhancement requirements from the canonical companions are captured in FR87-FR96, NFR23, and NFR28-NFR30.

### FR Coverage Map

FR1: Epic 1 - Operate one private single-Administrator ledger.
FR2: Epic 1 - Validate the configured password hash before persistence startup.
FR3: Epic 1 - Establish the CSRF-protected login flow.
FR4: Epic 1 - Verify the configured administrator password.
FR5: Epic 1 - Durably rotate and promote authenticated sessions.
FR6: Epic 1 - Bound and expire anonymous sessions.
FR7: Epic 1 - Bound, refresh, and expire authenticated sessions.
FR8: Epic 1 - Fail authenticated promotion safely at capacity.
FR9: Epic 1 - Invalidate sessions on restart.
FR10: Epic 1 - Flush authentication on logout.
FR11: Epic 1 - Apply the required cookie policy.
FR12: Epic 1 - Rate-limit password verification per trusted client.
FR13: Epic 1 - Fail closed when limiter-key capacity is full.
FR14: Epic 1 - Require authentication for all ledger access.
FR15: Epic 1 - Enforce strict CSRF before unsafe processing.
FR16: Epic 1 - Issue distinct single-use submission tokens.
FR17: Epic 1 - Bound and expire both submission-token pools.
FR18: Epic 1 - Reject invalid or reused submission tokens without dispatch.
FR19: Epic 1 - Reserve tokens exactly at dispatch and consume attempts terminally.
FR20: Epic 2 - Create valid Groups.
FR21: Epic 2 - Default new Groups to USD and Manage.
FR22: Epic 2 - Open established Groups in Summary.
FR23: Epic 2 - Edit Group names and supported Group Currency.
FR24: Epic 2 - Archive and restore Groups.
FR25: Epic 2 - Separate active and archived Group views.
FR26: Epic 2 - Keep archived Groups readable and immutable.
FR27: Epic 2 - Reject direct archived-Group mutations before dispatch.
FR28: Epic 2 - Delete history-free Groups with unreferenced Participants.
FR29: Epics 2 and 3 - Define history-aware Group deletion, then structurally verify restriction when Spending persistence exists.
FR30: Epics 2 and 5 - Add/edit/restore Participants with Group management; complete zero-Balance archival with all-time debts.
FR31: Epic 2 - Enforce Group-owned, non-user Participant identity.
FR32: Epic 2 - Validate Participant names.
FR33: Epic 2 - Normalize Participant colors.
FR34: Epic 2 - Suggest varied Participant colors while permitting selection.
FR35: Epic 5 - Separate active and archived Participant views with the archive/restore capability.
FR36: Epic 2 - Prohibit independent Participant deletion.
FR37: Epic 5 - Require exact zero Historical Balance for Participant archival.
FR38: Epic 5 - Revalidate ledger, UTC date, and quote eligibility before archive commit.
FR39: Epic 5 - Block archival safely when rate evidence is unavailable.
FR40: Epic 5 - Restore Participants without Balance or rate checks after archival is available.
FR41: Epics 4 and 5 - Preserve archived identities in current-month summaries and all-time financial outputs.
FR42: Epic 3 - Resolve current Participant names in historical Spending views.
FR43: Epic 3 - Create, inspect, edit, and delete Spendings.
FR44: Epic 3 - Capture every required Spending field and allocation.
FR45: Epic 3 - Restrict Spending categories to the supported set.
FR46: Epic 3 - Validate strict bounded Spending dates.
FR47: Epic 3 - Enforce positive bounded monetary amounts.
FR48: Epic 3 - Validate currency-specific minor-unit precision without rounding.
FR49: Epic 3 - Require one active Group-owned Payer for the Total.
FR50: Epic 3 - Require unique positive exact-conserving Shares.
FR51: Epic 3 - Offer only Proportional and Exact Share modes.
FR52: Epic 3 - Apply new-Spending form defaults.
FR53: Epic 3 - Assign the full paid Total when selecting a Payer.
FR54: Epic 3 - Initialize and edit Proportional selections.
FR55: Epic 3 - Validate Proportional weights.
FR56: Epic 3 - Allocate Proportional Shares deterministically and identically for Preview/commit.
FR57: Epic 3 - Initialize Exact Shares with deterministic residual assignment.
FR58: Epic 3 - Edit Exact Shares against a displayed closing difference.
FR59: Epic 3 - Preview exact allocations and reject invalid normalization.
FR60: Epic 3 - Open existing Spendings in Exact mode.
FR61: Epic 3 - Preserve but never introduce archived allocation roles on update.
FR62: Epic 3 - Commit complete Spending aggregates atomically.
FR63: Epic 3 - Apply last-committed-write semantics.
FR64: Epic 3 - Provide focused Add Spending and return to the committed row.
FR65: Epic 3 - Browse fixed keyset-paginated Spending history.
FR66: Epic 3 - Read Spending detail for archived identities and Groups.
FR67: Epic 4 - Show current UTC-month Spending totals.
FR68: Epic 4 - Show exact source-currency Group and Payer totals independently of rates.
FR69: Epic 4 - Convert current-month totals with Spending-date contexts.
FR70: Epic 4 - Accumulate converted values exactly and conserve the displayed Group total.
FR71: Epic 4 - Quantize final Payer totals together deterministically.
FR72: Epic 4 - Withhold the whole converted section safely on quote or calculation failure.
FR73: Epic 4 - Synthesize and disclose same-currency rates without provider calls.
FR74: Epic 4 - Apply Historical and provisional future-date rate contexts.
FR75: Epic 5 - Support non-persisted current conversion mode for all-time debts.
FR76: Epics 4 and 5 - Use and disclose context-matching stale quotes for Historical/future and Current modes.
FR77: Epics 4 and 5 - Enforce fixed-past and refreshable stale-eligibility windows in every conversion mode.
FR78: Epic 5 - Calculate all-time Participant Balances in Historical or Current mode.
FR79: Epic 5 - Default Balances to Spending-date Historical conversion.
FR80: Epic 5 - Produce deterministic target-precision exact-zero-sum Balances.
FR81: Epic 5 - Disclose debt calculation context, rates, and warnings.
FR82: Epic 5 - Return retryable unavailability without partial debts.
FR83: Epic 5 - Sanitize checked calculation failures without partial output.
FR84: Epic 5 - Derive advisory Transfers without repayment state.
FR85: Epic 5 - Produce positive, pair-unique, complete deterministic Transfers.
FR86: Epic 5 - Use bounded deterministic greedy Settlement ordering.
FR87: Epic 2 - Establish shared retained-value inline validation for Group and Participant forms; Epic 3 extends it to Spendings.
FR88: Epic 2 - Establish successful mutation redirects for ledger forms.
FR89: Epic 2 - Establish strict field extraction before ledger dispatch.
FR90: Epic 1 - Establish native full-page interaction with optional HTMX enhancement.
FR91: Epic 1 - Route enhanced failures to an announced status region.
FR92: Epic 1 - Establish the semantic responsive browser-compatible shell.
FR93: Epic 1 - Establish keyboard operation, labels, and visible focus.
FR94: Epic 1 - Establish contrast and programmatic error association.
FR95: Epic 1 - Apply no-store and mandatory browser security headers.
FR96: Epic 1 - Keep probes and static assets session-free.
FR97: Epic 1 - Expose independent process liveness.
FR98: Epic 1 - Expose SQLite/supervisor readiness without provider coupling.
FR99: Epic 1 - Treat cleanup-supervisor failure as fatal to readiness and admission.
FR100: Epic 1 - Enforce route-specific request-body limits.
FR101: Epic 1 - Separate and bound user, login, and probe admission.
FR102: Epics 1 and 5 - Establish route timeout classes, then verify the 90-second class on the Debts route.
FR103: Epics 1 and 2 - Establish bounded dispatch/outcome primitives, then integrate them with the first ledger mutation.
FR104: Epics 1 and 2 - Establish graceful lifecycle coordination, then prove shutdown waits for real dispatched ledger mutations.
FR105: Epic 1 - Run the complete local application with one command and no external build/provider prerequisite.

## Epic List

### Epic 1: Securely Operate and Access Debtor

The administrator can start a healthy local Debtor process, sign in and out securely, and use a resilient, accessible server-rendered shell whose unsafe actions are protected before any ledger capability is added.

**FRs covered:** FR1-FR19, FR90-FR105

**Implementation notes:** Establishes workspace composition, the password helper contract, authentication/session/CSRF/submission-token foundations, the shared HTML shell, admission budgets, probes, timeout classes, and narrow mutation-outcome/lifecycle primitives exercised by the authenticated local application. Real ledger-mutation integration begins with Group mutation in Epic 2 rather than creating an unexercised infrastructure path.

### Epic 2: Organize Groups and Participants

The administrator can create and configure Groups, set up and maintain active Group-owned Participant identities, navigate archived Group contexts, and safely manage Group lifecycle changes that do not require debt calculation.

**FRs covered:** FR20-FR34, FR36, FR87-FR89; shares structural FR29 verification with Epic 3; completes real-ledger FR103-FR104 integration; FR30, FR35, and FR40 are completed with Participant lifecycle in Epic 5

**Implementation notes:** Delivers Group-centered Manage and Summary navigation, strict retained-value forms, Group archive/restore/delete rules, active Participant add/edit, persistence integrity, and the first real use of definitive ledger-mutation dispatch/shutdown coordination. Participant archive, archived views, and restore are delivered together in Epic 5 because archival requires the all-time Historical Balance engine and immutable quote context.

### Epic 3: Record and Maintain Exact Spendings

The administrator can create, preview, inspect, correct, browse, and delete exact multi-currency Spendings with one Payer and deterministic Proportional or Exact Shares.

**FRs covered:** FR42-FR66; completes structural FR29 verification and extends FR87-FR89 to Spending forms

**Implementation notes:** Delivers exact domain allocation rules, application-owned raw input policy, transactional aggregate persistence, complete snapshot materialization, direct aggregate loads, fixed keyset history, archived-role update rules, and native/HTMX Preview parity. Participant-archival revalidation machinery is deferred until its first consumer in Epic 5.

### Epic 4: Understand Current-Month Spending

The administrator can answer the selected Group's exact current UTC-month totals by Source Currency and, when rate evidence permits, see conserved per-Payer and Group totals in Group Currency with complete stale/provisional disclosure.

**FRs covered:** FR67-FR74 and Historical/future portions of FR76-FR77; shares FR41 with Epic 5

**Implementation notes:** Source totals remain independently available. This epic establishes bounded lexical-Decimal provider access, complete requested/fetch/effective-date evidence for Historical and provisional-future contexts, context-keyed single-flight/LRU caching, deterministic final quantization, and whole-section degraded behavior. Current-mode orchestration and immutable archival-attempt bundles are deferred to their first consumers in Epic 5.

### Epic 5: Calculate Debts, Settle, and Safely Retire Identities

The administrator can calculate all-time Historical or Current Balances, receive complete advisory Settlement Transfers, and archive, browse, or restore Participants while retaining every referenced identity in history and admitting archive only from an unchanged exact-zero Historical context.

**FRs covered:** FR35, FR37-FR41, FR75-FR86; completes FR30 and verifies the Debts-specific portion of FR102

**Implementation notes:** Builds on complete Spending snapshots and the rate adapter from prior epics, adds exact-zero-sum quantization and deterministic Settlement, returns no partial financial output, and binds Participant archival to an immutable ledger/date/quote attempt with final transactional revalidation.

### Cross-Cutting Story Rule

Primary FR ownership in the coverage map does not make cross-cutting behavior a one-time implementation. Every affected web story must repeat applicable strict-form, authentication, CSRF, submission-token, security-header, native-fallback, accessibility, responsive, safe-error, admission, timeout, and no-pre-dispatch-side-effect acceptance criteria. Every affected financial or persistence story must likewise repeat applicable exactness, determinism, checked-arithmetic, corruption-rejection, transaction, snapshot, diagnostic-safety, and concurrency criteria.

Each story must leave a runnable vertical increment and remove any superseded path rather than retaining parallel scaffolds. Introduce each infrastructure primitive in the first story that exercises it, while preserving complete required evidence at existing boundaries. Within Epic 4, source totals remain independently complete before converted totals are added. Within Epic 5, complete all-time calculation precedes Settlement and Participant archival consumers. A story cannot claim a cross-cutting FR complete until every route or financial output introduced by that story verifies the applicable behavior. Every SQL statement uses checked SQLx macros except the fixed WAL-checkpoint PRAGMA; whenever checked SQL or migrations change, migrate a temporary database, run online `cargo sqlx prepare --workspace --check`, and commit refreshed `.sqlx` metadata.

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

**Requirements:** FR2; NFR17, NFR25, NFR33-NFR34; exact Argon2 bounds, helper independence, cheap validation, secret-safe diagnostics, and validation requirements.

### Story 1.2: Start the Complete Local Application

As the administrator,
I want Debtor to start locally from one validated command,
So that I can run a dependable private ledger without external build or provider prerequisites.

**Acceptance Criteria:**

**Given** the repository is checked out with the pinned Rust 1.97.1 toolchain and lockfiles
**When** the production workspace is inspected and validated
**Then** it uses edition 2024, MSRV 1.97, Cargo resolver 3, and the minimal toolchain profile with rustfmt and Clippy, and contains the root composition crate plus `debtor-domain`, `debtor-application`, `debtor-infra`, and `debtor-web` with inward dependency direction enforced by `architecture-check`
**And** `tools/password-hash` remains an independent workspace and routine validation never uses `cargo build --release`.

**Given** a new local operator follows the repository bootstrap instructions
**When** `.env.example` is copied to `.env` and a Story 1.1 password hash is supplied
**Then** the example documents every mandatory variable without containing secrets and bare `cargo run` unambiguously selects the application binary
**And** no Docker, frontend build, manual migration, metadata generation, or provider availability is required.

**Given** `.env` contains valid local configuration and a password hash accepted by Story 1.1
**When** `cargo run` starts Debtor
**Then** it loads configuration, creates or connects to the configured SQLite database, runs migrations, enables foreign keys, WAL, `synchronous=FULL`, and a five-second busy timeout, composes concrete adapters behind application-owned ports, and binds the configured address
**And** it logs only a non-secret local URL including its `http://` scheme.

**Given** Frankfurter is unavailable and no Docker service, frontend build, manual migration, or SQLx metadata generation has been run
**When** valid local startup occurs
**Then** Debtor still reaches socket admission successfully
**And** provider availability is not consulted during startup.

**Given** the application is running with no dispatched ledger mutation
**When** shutdown is requested
**Then** the process stops accepting work, closes its server and SQLite resources without panic, and preserves WAL sidecars if checkpointing fails
**And** it exits without representing an unknown storage outcome as rollback.

**Given** production Cargo manifests are resolved
**When** dependency policy is inspected
**Then** they pin Axum 0.8.9, Askama 0.16.0, Tokio 1.53.1, Tower 0.5.3, tower-http 0.6.11, tower-sessions 0.15.0, SQLx/sqlx-cli 0.9.0 with bundled SQLite, reqwest 0.13.4 with rustls, rust_decimal 1.42.1, chrono 0.4.45, Argon2 0.5.3, thiserror 2.0.20, anyhow 1.0.104, serde 1.0.229, serde_json 1.0.151, and UUID 1.24.0
**And** lockfiles are preserved, Cargo validation uses `--locked`, current crate documentation is consulted before framework/library API changes, and `cargo deny check` passes whenever dependency policy changes.

**Given** application SQL and migrations are inspected
**When** SQLx policy is validated
**Then** every SQL statement uses compile-time checked SQLx macros except the fixed WAL-checkpoint PRAGMA
**And** checked SQL or migration changes are applied to a temporary database, verified with online `cargo sqlx prepare --workspace --check`, and accompanied by committed refreshed `.sqlx` metadata.

**Given** the production workspace implementation is complete
**When** formatting, locked check, offline Clippy with warnings denied, workspace tests, and `architecture-check` run
**Then** every check passes
**And** startup/composition code preserves the required inward dependency direction.

**Requirements:** FR9, FR105; NFR25, NFR31-NFR34; startup, architecture, SQLite, local-run, toolchain, testing, and validation requirements.

### Story 1.3: Open a Protected and Accessible Login Page

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

**Requirements:** FR1, FR3, FR6, FR11, FR14-FR17, FR90-FR96; NFR19-NFR23, NFR28-NFR30; semantic HTML, static asset, session-free route, accessibility, responsive, and strict security-header requirements.

### Story 1.4: Sign In with Bounded Password Verification

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

**Requirements:** FR4-FR5, FR8, FR12-FR13, FR15, FR18-FR19, FR88-FR89, FR100; NFR17-NFR18, NFR21-NFR25; trusted-proxy, strict-form, password-concurrency, limiter, durable-promotion, and safe-diagnostic requirements.

### Story 1.5: Maintain an Authenticated Session and Sign Out

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

**Requirements:** FR1, FR7, FR9-FR11, FR14-FR16, FR18-FR19, FR88, FR90-FR95; NFR19-NFR23, NFR25, NFR28-NFR30; authenticated access, expiry refresh, logout, restart invalidation, and shared web-policy requirements.

### Story 1.6: Protect Every Unsafe Form from Replay

As the administrator,
I want each unsafe form submission accepted for at most one dispatch,
So that retries, double-clicks, and replayed requests cannot repeat a mutation.

**Acceptance Criteria:**

**Given** authenticated unsafe forms are rendered
**When** submission tokens are issued
**Then** the authenticated pool holds at most 1,024 live tokens globally and 32 per session, each with a 30-minute absolute expiry
**And** it remains isolated from the 4,096-token anonymous pool.

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
**Then** tests prove zero dispatch for malformed fields, oversized bodies, failed authentication, invalid CSRF, invalid tokens, archived-route prechecks, and validation errors
**And** concurrency tests use barriers or notifications rather than timing sleeps.

**Requirements:** FR15-FR19, FR87-FR91, FR100, FR103; NFR1, NFR21-NFR22, NFR25, NFR30, NFR32-NFR34; strict extraction, dispatch-boundary, deterministic concurrency, safe failure, and web-testing requirements.

### Story 1.7: Expose Health, Readiness, and Bounded Admission

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

**Requirements:** FR96-FR102; NFR1-NFR4, NFR25-NFR27, NFR31-NFR34; probe separation, timeout classification, mandatory supervision, bounded cleanup, and deterministic concurrency-testing requirements.

### Story 1.8: Preserve Definitive Mutations Through Shutdown

As the administrator,
I want admitted mutations to reach an authoritative outcome even during shutdown,
So that Debtor never reports an unknown ledger state as a rollback or silently cancels committed work.

**Acceptance Criteria:**

**Given** an unsafe ledger request has not yet dispatched
**When** body extraction, authentication, CSRF, token checks, or asynchronous web prechecks exceed the 30-second absolute pre-dispatch deadline
**Then** the request is rejected without reserving a still-valid token, invoking a state-changing use case, opening a transaction, or starting a guarded side effect
**And** deterministic tests prove the absence of dispatch.

**Given** every pre-dispatch check succeeds
**When** the first state-changing use-case call begins
**Then** the root mutation executor records dispatch at that exact boundary and tracks the mutation as running
**And** no generic request timeout cancels it after that point.

**Given** persistence returns an authoritative commit or rollback result
**When** the mutation executor receives it
**Then** it synchronously and infallibly publishes `Committed` or `RolledBack` before response rendering or delivery
**And** response failure cannot change the published outcome or trigger automatic retry.

**Given** a mutation task fails unexpectedly
**When** rollback is authoritatively established
**Then** the executor may publish `RolledBack`
**And** otherwise it publishes `Unknown`, initiates fatal shutdown, suppresses automatic retry, and never represents the outcome as rollback.

**Given** shutdown begins
**When** the root lifecycle coordinator acts
**Then** it stops new admission and drains HTTP connections for at most ten seconds
**And** after that drain it waits without a fixed total wall-clock deadline until every already-dispatched mutation is no longer running.

**Given** no dispatched mutation remains running
**When** storage shutdown proceeds
**Then** Debtor attempts the bounded fixed WAL-checkpoint PRAGMA, preserves WAL sidecars if checkpointing fails, and closes the pool in order
**And** SQLite diagnostics contain only allowed fixed operation names and result categories.

**Given** the composed application is tested over a real socket
**When** the startup smoke scenario logs in with CSRF and a single-use token, performs an authenticated read, initiates coordinated shutdown, and observes completion
**Then** startup ordering, authentication protection, no-pre-dispatch side effects, bounded HTTP drain, and resource closure are verified
**And** secrets, identifiers, SQL, values, query strings, and provider URLs do not appear in captured logs.

**Requirements:** FR103-FR105; NFR3, NFR15, NFR25-NFR27, NFR31-NFR34; authoritative outcome, SQLite diagnostic, root smoke-test, and shutdown requirements.

### Story 1.9: Operate Debtor Behind a Sanitizing HTTPS Edge

As the administrator,
I want production Debtor exposed through a verified sanitizing HTTPS edge,
So that modern client protocols cannot bypass identity, replay, body-limit, or mutation-completion protections.

**Acceptance Criteria:**

**Given** production traffic reaches Debtor
**When** edge and backend responsibilities are inspected
**Then** the edge owns TLS, automatic certificates, HTTP/3/QUIC, `Alt-Svc`, and HTTP/2 or HTTP/1.1 client fallback while Debtor remains a private HTTP/1.1 TCP backend with no certificate, QUIC, UDP, or HTTP/3 listener dependency
**And** direct insecure HTTP is limited to debug/local operation.

**Given** a client request carries forwarding headers
**When** the edge proxies it to Debtor
**Then** the edge strips untrusted forwarding input or appends its immediate peer while preserving chain order, and its source CIDR/header mode matches `APP_TRUSTED_PROXY_CIDRS` and `APP_TRUSTED_PROXY_HEADER`
**And** Debtor resolves identical client identity and login-limiter behavior over HTTP/3 and TCP fallback.

**Given** TLS/QUIC early data is available
**When** an unsafe request is attempted through early data
**Then** the edge disables early data or returns `425 Too Early`; only explicitly marked `GET` and `HEAD` paths may pass early data
**And** CSRF is never treated as replay protection for early-data mutation.

**Given** login or another form request reaches the edge
**When** body limits are enforced
**Then** the edge permits at most 8 KiB for `/login` and 256 KiB for other form endpoints, matching or tightening application limits
**And** oversized input is rejected before backend mutation dispatch.

**Given** backend transport is configured
**When** the edge manages connections and timeouts
**Then** it reuses private HTTP/1.1 backend connections and may bound connect/response-header waits
**And** no edge request, read, write, or stream timeout can expire before an admitted post-dispatch mutation reaches definitive completion.

**Given** HTTP/3 is introduced
**When** rollout begins
**Then** `Alt-Svc` uses a short lifetime until UDP/443 reachability and edge telemetry are verified
**And** the lifetime is not increased until blocked UDP falls back to HTTP/2 or HTTP/1.1, unsafe early data receives `425`, and forwarded client identity matches across every protocol.

**Given** edge policy is tested or documented as deployable configuration
**When** the production contract is validated
**Then** forwarding sanitation, protocol fallback, body limits, backend reuse/timeouts, early-data rejection, and staged `Alt-Svc` rollout have reproducible verification steps
**And** no secret, client identity, query string, provider URL, or request-derived value is introduced into application logs.

**Requirements:** FR12, FR100, FR103; NFR1, NFR3, NFR23-NFR25, NFR31-NFR34; edge TLS/HTTP3 ownership, forwarding sanitation, early-data rejection, body limits, backend transport, timeout, fallback, and rollout requirements.

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

**Given** gate acquisition cannot complete within five seconds
**When** Group creation is attempted
**Then** it returns sanitized retryable feedback without opening a transaction or starting a guarded persistence side effect
**And** deterministic tests prove no repository write occurred.

**Given** multiple valid Group creations are admitted
**When** their commits complete in any permitted order
**Then** every committed Group appears in the active list and the last committed state governs without optimistic revision columns
**And** ordering exposed to the UI is deterministic.

**Given** Group persistence is migrated
**When** invalid direct rows are attempted
**Then** SQLite structurally enforces supported currency, active/archive flag shape, bounded non-empty text shape, positive identity, and required relationships without duplicating Unicode trimming or monetary rules
**And** checked SQLx queries have matching committed offline metadata.

**Given** shutdown begins while a Group mutation is dispatched
**When** HTTP drain reaches its bound
**Then** the process waits for the mutation's authoritative completion before checkpoint and pool close
**And** the composed test verifies FR103-FR104 against a real ledger mutation.

**Requirements:** FR20-FR22, FR25, FR87-FR89, FR103-FR104; NFR3, NFR7, NFR15, NFR25, NFR31-NFR34; Group default, write-gate, SQLx, migration, strict-form, vertical-slice, and first real mutation-integration requirements.

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

**Requirements:** FR21-FR23, FR26-FR27, FR87-FR90; NFR3, NFR7, NFR15, NFR25, NFR28-NFR34; supported-code, last-commit, archived-route, strict-form, accessibility, and transaction requirements.

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

**Requirements:** FR30-FR34, FR36, FR87-FR90; NFR7, NFR15-NFR16, NFR25, NFR28-NFR34; Participant ownership, color suggestion, no-global-surface, migration, strict-form, accessibility, and history-preservation requirements.

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

**Given** a Participant is referenced by future Spending history
**When** its name is edited
**Then** the identity ID and relationships remain unchanged so historical projections can resolve the current name
**And** the mutation never copies or rewrites monetary allocation data.

**Given** the administrator looks for a Participant delete action or submits a crafted delete request
**When** routes and use cases are evaluated
**Then** no independent Participant deletion capability exists
**And** persistence restricts destructive behavior outside history-free owning-Group deletion.

**Given** concurrent valid edits are admitted
**When** commits complete
**Then** each update is atomic and the last committed name/color wins without optimistic revision handling
**And** safe failure mapping exposes no SQL, identifiers, values, or request-derived diagnostics.

**Requirements:** FR30-FR33, FR36, FR42, FR87-FR90; NFR7, NFR15-NFR16, NFR25, NFR28-NFR34; immutable ownership, current-name history projection, no-delete, strict-form, transaction, and safe-diagnostic requirements.

### Story 2.5: Archive, Restore, or Delete a History-Free Group

As the administrator,
I want to retire unused Groups without losing referenced history,
So that active navigation stays clean while destructive actions remain safe.

**Acceptance Criteria:**

**Given** an active Group exists
**When** the administrator submits its protected archive form
**Then** the shared write gate atomically marks the Group archived and redirects with `303` to an appropriate contextual page
**And** its owned Participants remain stored and unchanged.

**Given** a Group is archived
**When** active Group navigation is rendered
**Then** the Group is absent from active lists and present in a separate contextual archived-Groups view
**And** the archived Group remains readable with no settings, Participant, Spending, archive, or delete controls other than the permitted restore action.

**Given** any archived Group mutation or mutation-form route other than restore is addressed directly
**When** the request is processed
**Then** Debtor returns `409 Conflict` before token reservation and use-case invocation
**And** web tests prove zero dispatch.

**Given** a valid protected restore form is submitted for an archived Group
**When** restore dispatches
**Then** the Group becomes active atomically, its existing Participants remain owned by it, and success redirects with `303`
**And** no Balance or exchange-rate calculation is required for Group restoration.

**Given** an active Group has no Spendings
**When** the administrator confirms and dispatches deletion
**Then** one transaction deletes the Group and its unreferenced Group-owned Participants, publishes a definitive outcome, and redirects with `303`
**And** no independent Participant-delete path is introduced.

**Given** the application repository reports that a Group has any Spending history
**When** deletion is requested
**Then** the use case refuses destructive deletion and offers archival instead, with no state change
**And** no owned Participant is deleted.

**Given** archive, restore, or delete validation fails before dispatch or a duplicate token is replayed
**When** the request is rejected
**Then** applicable `422` or `409` behavior is returned with no state-changing use case invocation
**And** valid pre-dispatch validation does not consume the token.

**Given** Group lifecycle operations race with another admitted Group mutation
**When** write-gate and transaction boundaries execute
**Then** operations serialize, eligibility is checked transactionally, and only complete committed states become visible
**And** timeout or constraint failures remain sanitized and log only allowed fixed SQLite categories.

**Requirements:** FR24-FR29, FR36, FR87-FR90; NFR3, NFR15-NFR16, NFR22, NFR25, NFR31-NFR34; archived-Group context, immutable page, history-free cascade, future Spending restriction, strict-form, and transactional lifecycle requirements.

## Epic 3: Record and Maintain Exact Spendings

The administrator can create, preview, inspect, correct, browse, and delete exact multi-currency Spendings with one Payer and deterministic Proportional or Exact Shares.

### Story 3.1: Preview a Proportional Spending

As the administrator,
I want to preview a Spending divided by proportional weights,
So that I can see exact Source Currency Shares before committing the transaction.

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

**Requirements:** FR43-FR56, FR59, FR64, FR87-FR94; NFR5-NFR10, NFR25, NFR28-NFR34; application input ownership, exact Decimal, deterministic Proportional Preview, native fallback, and retained validation requirements.

### Story 3.2: Preview Exact Shares

As the administrator,
I want to preview and edit exact Participant Shares,
So that I can assign precise Source Currency responsibility before committing.

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

**Requirements:** FR49-FR59; NFR5-NFR10, NFR25, NFR28-NFR34; deterministic Exact initialization, difference display, uniqueness, positivity, exact closure, and Preview parity requirements.

### Story 3.3: Commit a Complete Spending

As the administrator,
I want a valid previewed Spending saved as one complete aggregate,
So that the ledger records exactly one Payer and conserved Shares or changes nothing.

**Acceptance Criteria:**

**Given** valid Proportional or Exact raw form input is submitted for commit
**When** application validation runs
**Then** it reparses every field and invokes the same allocation operation used by Preview rather than trusting displayed amounts or prior client state
**And** Proportional mode/weights are not persisted.

**Given** Group, Payer, Share, amount, precision, date, category, uniqueness, or conservation validation fails
**When** commit is requested
**Then** Debtor returns `422` with all raw values retained, no aggregate is persisted, and the submission token remains unconsumed until dispatch
**And** no partial Payer or Share result is exposed as valid.

**Given** a valid aggregate is ready to dispatch
**When** the mutation enters the write gate and transaction
**Then** persistence rechecks that the Group is active and every new Payer/Share Participant is active and owned by that Group, then inserts Spending, Payer, and all Shares atomically
**And** any eligibility race, constraint failure, or checked error rolls back the complete aggregate.

**Given** monetary values are persisted
**When** rows are inspected
**Then** Total, Payer amount, and Share amounts are canonical decimal `TEXT` with valid Source Currency precision and positivity
**And** SQLite performs no monetary parsing, conversion, equality check, or aggregation.

**Given** stored monetary text is malformed, noncanonical, nonpositive, over precision, over bounds, or nonconserving when decoded as an aggregate
**When** the repository reads it
**Then** it returns a safe corruption failure rather than normalizing, rounding, substituting zero, or returning a partial Spending
**And** raw row values never reach logs or HTTP.

**Given** the schema migration is applied
**When** structural integrity is tested
**Then** SQLite restricts supported category/currency codes, boolean flags, bounded text shape, ISO dates on or after `2025-01-01`, required relationships, unique allocation roles, and referenced Group/Participant deletion
**And** a Group with any Spending can no longer be structurally deleted.

**Given** commit succeeds
**When** the authoritative result is published
**Then** Debtor redirects with `303` to Transactions and the committed row is visible in deterministic newest-first order
**And** replaying the consumed submission token returns `409` without a second write.

**Given** checked SQL or migrations changed
**When** persistence validation runs
**Then** temporary-file SQLite tests cover WAL, constraints, rollback, multi-connection locking, and Group deletion restriction, and committed `.sqlx` metadata is refreshed with online prepare
**And** application use-case tests remain runnable with fakes and no SQLite.

**Requirements:** FR29, FR43-FR64; NFR5-NFR10, NFR14-NFR16, NFR22, NFR25, NFR31-NFR34; canonical persistence, corruption rejection, atomic aggregate, eligibility, write-gate, SQLx, and migration requirements.

### Story 3.4: Browse and Inspect Spending History

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

**Requirements:** FR42-FR43, FR65-FR66; NFR2, NFR5, NFR10, NFR14-NFR16, NFR25, NFR28-NFR34; keyset pagination, direct snapshot loading, current-name resolution, archived readability, and corruption-safe rendering requirements.

### Story 3.5: Correct an Existing Spending

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

**Requirements:** FR43-FR44, FR46-FR51, FR56-FR63, FR66, FR87-FR90; NFR3, NFR5-NFR10, NFR14-NFR16, NFR22, NFR25, NFR28-NFR34; Exact-on-edit, archived-role retention, atomic replacement, last-commit, and retained-validation requirements.

### Story 3.6: Delete a Spending Atomically

As the administrator,
I want to delete an incorrect Spending as one complete aggregate,
So that no orphaned Payer or Share data remains in the ledger.

**Acceptance Criteria:**

**Given** an active Group owns a Spending
**When** the delete confirmation page opens
**Then** Debtor direct-loads and displays the complete aggregate with Source Currency Total, Payer, Shares, date, category, and description
**And** it does not load all Group history.

**Given** the protected delete form has valid authentication, CSRF, submission token, and route state
**When** deletion dispatches
**Then** the write gate and one transaction delete the Spending, Payer, and all Shares atomically
**And** the root executor publishes one authoritative result before response work.

**Given** any aggregate row cannot be deleted or the transaction fails
**When** persistence returns
**Then** the complete Spending remains unchanged or the authoritative failure is reported; no partial allocation deletion is visible
**And** raw SQLite diagnostics, values, and identifiers are sanitized.

**Given** deletion succeeds
**When** the response is produced
**Then** Debtor redirects with `303` to Transactions, the row is absent, and remaining keyset ordering stays valid
**And** if it was the Group's last Spending, later history-free Group deletion can become eligible.

**Given** two concurrent delete attempts or an edit/delete race targets the same Spending
**When** operations serialize through the write gate
**Then** at most one delete commits, later work observes the committed state, and duplicate token replay returns `409` without dispatch
**And** no automatic retry is attempted.

**Given** the owning Group is archived
**When** delete confirmation or mutation is addressed directly
**Then** Debtor returns `409` before use-case invocation while read-only detail remains accessible
**And** the page exposes no delete control.

**Given** deletion tests run
**When** transaction failure, held gate timeout, constraint failure, duplicate dispatch, and archived precheck are exercised
**Then** tests prove atomicity and zero guarded side effects for every pre-dispatch rejection using deterministic coordination
**And** checked SQLx metadata remains current.

**Requirements:** FR43, FR62-FR66, FR87-FR90; NFR3, NFR14-NFR16, NFR22, NFR25, NFR31-NFR34; direct aggregate loading, atomic deletion, history-free eligibility, archived rejection, and deterministic concurrency requirements.

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

**Requirements:** FR41, FR67-FR68, FR72; NFR2, NFR5-NFR7, NFR10, NFR14-NFR16, NFR25-NFR30, NFR32-NFR34; current UTC-month, Rust aggregation, archived identity, provider-independent fallback, and accessible summary requirements.

### Story 4.2: Resolve Exact Historical Rate Evidence

As the administrator,
I want monthly conversion to use exact date-appropriate rate evidence,
So that later Group Currency totals are reproducible and correctly contextualized.

**Acceptance Criteria:**

**Given** a Spending requests conversion from Source Currency to Group Currency for original date `R` and calculation date `C`
**When** the Historical context is built
**Then** effective fetch date is `F = min(R, C)` and the context key is `(source, target, R, F)`
**And** the provider's effective date is retained separately as quote evidence.

**Given** Source Currency equals Group Currency
**When** the quote is resolved
**Then** Debtor returns a synthetic exact `Decimal` rate of `1`, performs no provider call, and marks the evidence for disclosure
**And** the result participates in deterministic ordering like any other unique context.

**Given** the provider returns a JSON numeric rate
**When** the response is decoded
**Then** the complete arbitrary-precision numeric lexeme is captured without an intermediate floating-point representation and parsed through checked exact `rust_decimal::Decimal` representability
**And** malformed, nonpositive, oversized, out-of-range, excess-scale, or not-exactly-representable input is rejected with a safe adapter reason rather than rounded.

**Given** a provider call is required
**When** HTTP executes
**Then** connect time is limited to five seconds, total time to 20 seconds, response body to 64 KiB, and rustls is used
**And** logs expose no provider URL, query, response value, or request-derived context.

**Given** multiple Spendings require identical uncached `(source, target, R, F)` contexts
**When** one calculation resolves them
**Then** contexts are deduplicated and identical uncached requests use per-key single-flight
**And** at most four provider calls run globally and at most four unique requests run concurrently for that calculation.

**Given** requests complete in different orders
**When** quote evidence is assembled
**Then** deterministic context ordering produces identical evidence and later calculations regardless of completion order
**And** tests coordinate completion explicitly rather than relying on sleeps.

**Given** a fixed-past Historical quote succeeds
**When** it is cached
**Then** it enters the stable Historical cache; a future-date Historical quote uses the refreshable class and is marked provisional
**And** each class is capped at 4,096 entries with deterministic LRU eviction.

**Given** UTC day rolls over
**When** refreshable contexts are requested again
**Then** current/future fetch context uses the new UTC calculation date while stable past contexts may remain for process lifetime
**And** eviction may refetch but never crosses context keys or changes one calculation's assembled evidence.

**Requirements:** FR69, FR73-FR74; NFR4-NFR5, NFR10, NFR13, NFR25-NFR27, NFR32-NFR34; lexical Decimal decoding, context identity, provider bounds, single-flight, cache classes, deterministic concurrency, and provisional evidence requirements.

### Story 4.3: Review Conserved Group Currency Monthly Totals

As the administrator,
I want current-month Payer totals converted exactly into Group Currency,
So that the displayed Group total reconciles to the sum of the displayed Payer totals.

**Acceptance Criteria:**

**Given** current-month conversion is requested
**When** persistence supplies input
**Then** one SQLite snapshot materializes Group Currency and all complete included Spendings before the read transaction is released
**And** no provider request holds a database transaction.

**Given** exact quote evidence exists for every unique context
**When** conversion runs
**Then** each Payer's paid values are multiplied and accumulated exactly in Rust without per-Spending rounding
**And** checked arithmetic failure produces no partial converted Payer total.

**Given** exact per-Payer accumulations require target-currency quantization
**When** final totals are calculated
**Then** all Payer totals are truncated toward zero together and residual target minor units are assigned by descending fractional remainder with ascending Participant ID ties
**And** the displayed Group total is derived as the exact sum of displayed Payer totals.

**Given** input Spending or provider completion order varies
**When** the same immutable monthly input and quote evidence are calculated
**Then** Payer order, quantized amounts, Group total, and evidence disclosure remain identical
**And** exact conservation is covered by examples, boundaries, and property tests.

**Given** same-currency, fixed-past, and provisional future contexts are used
**When** converted Summary is rendered
**Then** it identifies Group Currency and discloses unique requested/fetch/provider-effective evidence in deterministic order, including synthetic and provisional markers
**And** no manual refresh control is added.

**Given** any quote is missing or checked conversion, accumulation, or quantization fails
**When** the converted projection is built
**Then** the entire converted section is withheld with one sanitized unavailable state and no partial converted rows
**And** the already calculated source-currency section and ledger CRUD remain available.

**Given** converted totals are displayed at 320 CSS pixels or without HTMX
**When** the administrator reviews multiple Payers
**Then** semantic labels clearly associate each Payer, target currency, amount, and evidence/warning region with accessible contrast and native rendering
**And** no custom JavaScript is required.

**Requirements:** FR67, FR69-FR74; NFR5, NFR10-NFR13, NFR15, NFR25-NFR30, NFR32-NFR34; snapshot release, exact accumulation, joint quantization, deterministic conservation, disclosure, and whole-section failure requirements.

### Story 4.4: Preserve Source Totals When Conversion Is Unavailable

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

**Requirements:** FR72, FR74, FR76-FR77; NFR4-NFR5, NFR13, NFR25-NFR27, NFR32-NFR34; context-matching Historical/future stale fallback, seven-day eligibility, whole-section degradation, cause separation, and provider-independent operation requirements.

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
**And** this verifies the Debts-specific portion of FR102 without affecting mutation timeout policy.

**Requirements:** FR41, FR74, FR78-FR83, FR102; NFR2, NFR4-NFR5, NFR10, NFR12-NFR16, NFR25-NFR30, NFR32-NFR34; immutable all-time snapshot, Historical evidence, exact-zero-sum quantization, disclosure, no-partial, and Debts timeout requirements.

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

**Requirements:** FR75-FR83; NFR4-NFR5, NFR10, NFR12-NFR13, NFR25-NFR30, NFR32-NFR34; non-persisted Current mode, current-class stale fallback, seven-day eligibility, exact-zero-sum reuse, disclosure, and no-partial requirements.

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

**Requirements:** FR41, FR83-FR86; NFR5, NFR10, NFR12, NFR25, NFR28-NFR34; deterministic greedy Settlement, positivity, pair uniqueness, completeness, `n - 1` bound, no partial output, and advisory-only requirements.

### Story 5.4: Archive a Zero-Balance Participant Safely

As the administrator,
I want to archive a Participant only when their Historical Balance is exactly zero in an unchanged context,
So that active choices stay clean without hiding unsettled obligations or racing ledger changes.

**Acceptance Criteria:**

**Given** an active Participant in an active Group is selected for archival
**When** one archive attempt begins
**Then** it captures an immutable complete all-time ledger snapshot/generation, UTC calculation context, and Historical quote bundle using the same calculation rules as Story 5.1
**And** provider requests occur only after the database snapshot transaction is released.

**Given** required Historical evidence is unavailable or only ineligible stale evidence exists
**When** archive eligibility is calculated
**Then** Debtor returns sanitized retryable feedback with no state change, no partial Balance, and no archive dispatch
**And** ordinary Participant/Spending operations remain available.

**Given** the target Participant's final quantized Group Currency Balance is nonzero
**When** eligibility is evaluated
**Then** archival is refused with clear sanitized feedback and no mutation
**And** Settlement advice is not treated as recorded repayment.

**Given** the target Participant's complete Historical Balance is exactly zero
**When** the archive mutation reaches final admission through the write gate
**Then** one transaction revalidates Participant/Group lifecycle, ledger generation, current UTC date, and eligibility of every quote in the immutable bundle before changing active status
**And** any mismatch returns retryable feedback with no archive commit.

**Given** a Spending or other ledger mutation commits between calculation snapshot and archive admission
**When** final generation revalidation runs
**Then** archival is rejected even if a newly calculated Balance might also be zero
**And** a later explicit attempt must take a new snapshot and quote bundle.

**Given** UTC date rolls over or refreshable quote evidence expires before commit
**When** final revalidation runs
**Then** archival is rejected with no state change
**And** the attempt never silently substitutes a newly fetched or revised provider quote.

**Given** every eligibility condition remains unchanged
**When** archive commits
**Then** only the Participant's lifecycle state changes atomically, the identity and every historical reference remain stored, and success redirects with `303`
**And** no rate evidence, Balance, Transfer, or repayment state is persisted.

**Given** ledger-generation support is introduced for this first consumer
**When** migrations and repositories change
**Then** every relevant ledger mutation advances/revalidates the generation transactionally, checked SQLx metadata is refreshed, and deterministic race tests coordinate held snapshots/gates without sleeps
**And** no optimistic user-facing stale-edit revision path is added.

**Given** the owning Group is archived, CSRF/token validation fails, or the archive token is replayed
**When** the request is processed
**Then** archived Group access returns pre-use-case `409`, other pre-dispatch failures invoke no use case, and replay dispatches at most once
**And** post-dispatch work is not cancelled by a generic timeout.

**Requirements:** FR30, FR37-FR39; NFR3, NFR10, NFR13, NFR15-NFR16, NFR22, NFR25, NFR31-NFR34; immutable attempt, exact-zero eligibility, generation/date/quote revalidation, no persisted evidence, write-gate, and deterministic race requirements.

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
**Then** the identity remains available only in the same stored Payer or Share role under Story 3.5 rules
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

**Requirements:** FR30, FR35, FR40-FR42; NFR3, NFR10, NFR15-NFR16, NFR22, NFR25, NFR28-NFR34; contextual archived views, historical inclusion, unconditional restore, active allocation eligibility, no-delete, and lifecycle concurrency requirements.
