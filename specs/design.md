# Debtor Design Contract

## Goal

Debtor is a private, password-gated ledger for one administrator to record shared, multi-currency spendings and view advisory settlement transfers. Transfers are derived on demand; repayments, paid status, and settlement checkpoints are not recorded.

## Decision Records

This document is the normative product and architecture contract. The rationale and consequences for the foundation decisions are recorded in [ADR 0001: Foundation Architecture](adr/0001-foundation-architecture.md). Later ADRs MUST explicitly identify any decision they supersede, and this document MUST be synchronized in the same change.

## Release Scope

The first release MUST provide groups, reusable participants, memberships, spending CRUD, multiple payers, equal or exact shares, and advisory group settlements in twelve supported currencies and eight spending categories. Groups and participants support archive/restore; historical allocations remain visible.

It MUST NOT provide statistics, ratio/percentage/weighted splits, repayment tracking, settlement date ranges, exact global transfer minimization, persistent sessions, manual rate refresh, registration, usernames, or custom JavaScript. Pre-release migrations MAY be rewritten; database compatibility is not promised.

## User Model

There is exactly one administrator. Authentication is a password gate with no username, user table, registration, or participant login. Participants are accounting identities, not application users.

## Architecture

The production workspace MUST preserve this inward-only dependency direction:

```text
debtor (root) -> debtor-web / debtor-infra -> debtor-application -> debtor-domain
```

The diagram expresses permitted direction, not an exhaustive list of direct Cargo edges. `debtor-web` and `debtor-infra` MAY use domain entities and value objects directly when required by application-facing interfaces. The root composes concrete adapters with application-owned ports and services. Outer crates MUST NOT leak framework, persistence, HTTP, cryptography, session, or adapter types inward.

- `debtor-domain` owns synchronous, deterministic business rules and has no I/O or framework dependencies.
- `debtor-application` owns use cases and narrow, mockable inbound/outbound ports.
- `debtor-infra` owns SQLx, HTTP, cryptography, caching, and other concrete adapters.
- `debtor-web` owns Axum, forms, middleware, Askama rendering, and HTTP error mapping; handlers contain no financial, SQL, network, or cryptographic logic.
- The root crate owns configuration, composition, migrations, process lifecycle, and server startup.
- Application-facing failures MUST use structured, safe reason categories. Raw SQLx, HTTP, provider, cryptography, and other adapter diagnostics MUST NOT cross inward-facing ports or reach user-facing responses.
- The supported first-release production topology is one application process with one local SQLite volume behind a sanitizing HTTPS reverse proxy. Direct insecure HTTP is debug/local only. Multiple application instances and external SQLite writers are unsupported.
- SQLite persistence uses explicit WAL with `synchronous=FULL`, a five-second busy timeout, one process-local ledger write gate with a five-second acquisition timeout, and snapshot-consistent reads. Among admitted valid operations, the last committed write wins; optimistic revision columns are not used.

External effects MUST be constructor-injected through application-owned ports. Use cases MUST be testable with fakes, without a database, network, Axum, or wall clock.

## Accounting And History

- Money MUST use exact `Decimal`, be persisted as canonical SQLite `TEXT`, and be aggregated in Rust. Floating point and SQL monetary aggregates are forbidden.
- Currency precision is zero minor units for JPY/KRW, three for OMR, and two for other supported currencies. Input with excess precision is invalid.
- Names and descriptions are trimmed and bounded; dates are valid ISO dates on or after `2025-01-01`; totals and all persisted payer/share amounts are positive.
- Payer totals and share totals MUST each equal the spending total exactly in source-currency minor units.
- Equal splits assign residual minor units in ascending participant-ID order. Exact and equal are the only split modes.
- New allocations require a globally active participant with active group membership. Historical inactive or archived identities remain in history, balances, and transfers.
- Archived groups are read-only. Empty groups may be deleted; groups with spendings may only be archived. Participants are archived/restored, never deleted through the application. Referenced memberships may only be deactivated/reactivated.
- Spending aggregate writes, including membership eligibility checks and allocations, MUST be transactional. Latest valid write wins.
- Complete spending aggregates MUST be loaded from one database snapshot. Debt calculation MUST materialize group currency and complete spendings before releasing the database read transaction; network rate requests MUST NOT hold a database transaction.
- The infrastructure adapter MUST serialize all ledger mutations through one process-local write gate. Gate acquisition and SQLite lock waits are bounded at five seconds; a timed-out mutation MUST not begin a transaction.
- The group page MUST offer atomic create-and-join for new participants and one expense form with independent single/multiple-payer and equal/exact-share choices. These input modes are not persisted; edit screens infer the closest mode from stored allocations.
- New participant forms MUST suggest a varied valid color from the server while allowing the administrator to select a different color before submission. Validation re-renders retain the submitted color.
- Historical spending detail MUST remain readable for archived groups and resolve current participant names for inactive identities. Spending CRUD, group settings, empty-group deletion, participant editing, and reversible membership deactivation are server-rendered and CSRF protected.
- Validation failures on group, participant, create-and-join, and spending forms MUST return `422` with an inline error and all submitted values retained. Archived group pages MUST hide mutation/settings controls, and direct archived mutation or form routes MUST return `409` without invoking a use case.
- Domain and repository Rust code owns exact `Decimal` parsing, canonical formatting, precision, positivity, allocation equality, and monetary aggregation. SQLite stores monetary values as `TEXT` and MUST NOT parse, convert with floating point, or aggregate monetary values.
- SQLite MUST structurally restrict referenced group deletion, supported currency/category codes, boolean flags, bounded non-empty text, valid participant color shape, and ISO spending dates on or after `2025-01-01`. These checks MUST NOT duplicate Rust monetary arithmetic or Unicode trimming rules.

## Rates And Settlements

A spending retains its source currency; a group currency is a freely changeable settlement display target.

- Historical mode is the default and requests a rate for each spending date.
- Current mode uses the UTC calculation date for every spending and is not persisted.
- Future dates in historical mode use the latest current rate and are marked provisional.
- Cache keys include source, target, original requested date, and effective fetch date. Past historical entries may live for the process lifetime; current and future historical entries refresh on UTC day rollover.
- Exchange-rate JSON numbers MUST be decoded lexically with arbitrary precision into `Decimal`. Provider calls use a five-second connect timeout, 20-second total timeout, and 64 KiB response limit. At most four provider calls may be in flight globally, identical uncached keys use per-key single-flight, and each debt calculation deduplicates unique rate contexts with at most four concurrent requests. Completion order MUST NOT change balances, disclosures, or warnings.
- On provider failure, the latest context-matching prior quote MAY be used with a stale warning. Without one, only the debts view returns retryable `503`; CRUD remains available.
- Final balances are quantized to target minor units with largest-remainder allocation and participant ID tie-breaking, preserving an exact zero sum.
- Settlement uses a deterministic greedy matcher ordered by descending absolute balance then participant ID. Transfers are positive, settle every balance, do not repeat a participant pair, and number at most `n - 1`. They are not guaranteed globally minimal.
- The debts view MUST disclose the selected mode, calculation time, target currency, unique rates used, and stale/provisional warnings.

## Security

`APP_ADMIN_PASSWORD_HASH` MUST contain a valid bounded Argon2id v19 hash; startup fails before database connection or migration if it is absent or invalid. Sessions use a process-local in-memory server-side store, HTTP-only `SameSite=Strict` cookies, 30-day authenticated inactivity expiry refreshed on every request, rotation after login, and flush on logout. Anonymous login/CSRF sessions use ten-minute inactivity expiry, are explicitly saved before login rendering, and are capped at 4,096 live records without evicting authenticated sessions. Expired records are physically deleted every five minutes. Restarting the process logs the administrator out.

Every unsafe request, including login, MUST carry a session-backed synchronizer CSRF token. Login attempts MUST allow five attempts per trusted client IP in a rolling five-minute window, track at most 4,096 active client keys, and fail closed for an unseen key when full. Forwarding headers are trusted only for configured proxies and one explicitly selected header mode. Secrets, password hashes, session IDs, CSRF tokens, and client-IP limiter keys MUST NOT be logged. Non-debug builds require secure cookies; debug builds may use insecure local cookies.

Trusted proxies MUST strip untrusted forwarding input or append their immediate peer while preserving chain order. Only proxies within `APP_TRUSTED_PROXY_CIDRS` may supply the selected `APP_TRUSTED_PROXY_HEADER` format.

## Local Run Contract

After copying `.env.example` to `.env` and supplying a valid `APP_ADMIN_PASSWORD_HASH`, `cargo run` MUST be sufficient to run the complete local application. It MUST load configuration, create/connect and migrate SQLite, enable foreign keys, compose adapters and services, bind the configured address, log the local URL including its `http://` scheme without secrets, and shut down gracefully. The independent password helper is run with `cargo run --manifest-path tools/password-hash/Cargo.toml`.

Local startup MUST NOT require Docker, a frontend build, manual migrations, SQLx metadata generation, or Frankfurter availability. Local monetary databases are pre-release and may need to be deleted and recreated after canonical persistence changes.

## Operational Limits

- Login form bodies MUST be limited to 8 KiB and other form bodies to 256 KiB.
- User traffic MUST be limited to 64 in-flight requests and login to four in-flight requests. Health and readiness MUST use a separate four-request probe budget so user saturation cannot starve orchestration.
- Safe dynamic reads and login MUST have a 30-second timeout. Debts MUST have a 90-second timeout. Probes MUST have a two-second outer timeout and a one-second inner SQLite readiness timeout.
- Ledger mutations MUST bound body, authentication, admission, write-gate, and SQLite waits, but MUST NOT be cut off by a generic timeout after the use case begins. They MUST return a definitive commit or rollback result. A reverse proxy MUST NOT impose a shorter mutation timeout after dispatch.
- `/healthz` is process liveness. `/readyz` checks SQLite and mandatory in-process supervisor health only; Frankfurter availability and ledger contents MUST NOT gate startup or readiness.
- Shutdown MUST stop admission, drain for at most ten seconds, attempt a bounded WAL checkpoint, close the pool, and preserve WAL sidecars if checkpointing fails. Structured logs MUST be secret-safe.

## Maintenance

This document is the product and architecture source of truth. Update it before changing behavior, then synchronize the relevant ADR, README status, configuration examples, migrations, tests, and SQLx metadata in the same change. Pre-release migrations MAY be rewritten and database compatibility is not promised.
