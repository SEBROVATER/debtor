# Debtor Design Contract

## Goal

Debtor is a private, password-gated ledger for one administrator to record shared, multi-currency spendings and view advisory settlement transfers. Transfers are derived on demand; repayments, paid status, and settlement checkpoints are not recorded.

## Release Scope

The first release MUST provide groups, reusable participants, memberships, spending CRUD, multiple payers, equal or exact shares, and advisory group settlements in twelve supported currencies and eight spending categories. Groups and participants support archive/restore; historical allocations remain visible.

It MUST NOT provide statistics, ratio/percentage/weighted splits, repayment tracking, settlement date ranges, exact global transfer minimization, persistent sessions, manual rate refresh, registration, usernames, or custom JavaScript. Pre-release migrations MAY be rewritten; database compatibility is not promised.

## User Model

There is exactly one administrator. Authentication is a password gate with no username, user table, registration, or participant login. Participants are accounting identities, not application users.

## Architecture

The production workspace MUST preserve this dependency graph:

```text
debtor (root) -> debtor-web -> debtor-application -> debtor-domain
              -> debtor-infra -> debtor-application -> debtor-domain
```

The root composes concrete adapters with application-owned ports and services; outer crates never leak framework or persistence types inward.

- `debtor-domain` owns synchronous, deterministic business rules and has no I/O or framework dependencies.
- `debtor-application` owns use cases and narrow, mockable inbound/outbound ports.
- `debtor-infra` owns SQLx, HTTP, cryptography, caching, and other concrete adapters.
- `debtor-web` owns Axum, forms, middleware, Askama rendering, and HTTP error mapping; handlers contain no financial, SQL, network, or cryptographic logic.
- The root crate owns configuration, composition, migrations, process lifecycle, and server startup.

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
- The group page MUST offer atomic create-and-join for new participants and one expense form with independent single/multiple-payer and equal/exact-share choices. These input modes are not persisted; edit screens infer the closest mode from stored allocations.
- Historical spending detail MUST remain readable for archived groups and resolve current participant names for inactive identities. Spending CRUD, group settings, empty-group deletion, participant editing, and reversible membership deactivation are server-rendered and CSRF protected.

## Rates And Settlements

A spending retains its source currency; a group currency is a freely changeable settlement display target.

- Historical mode is the default and requests a rate for each spending date.
- Current mode uses the UTC calculation date for every spending and is not persisted.
- Future dates in historical mode use the latest current rate and are marked provisional.
- Cache keys include source, target, original requested date, and effective fetch date. Past historical entries may live for the process lifetime; current and future historical entries refresh on UTC day rollover.
- On provider failure, the latest context-matching prior quote MAY be used with a stale warning. Without one, only the debts view returns retryable `503`; CRUD remains available.
- Final balances are quantized to target minor units with largest-remainder allocation and participant ID tie-breaking, preserving an exact zero sum.
- Settlement uses a deterministic greedy matcher ordered by descending absolute balance then participant ID. Transfers are positive, settle every balance, do not repeat a participant pair, and number at most `n - 1`. They are not guaranteed globally minimal.
- The debts view MUST disclose the selected mode, calculation time, target currency, unique rates used, and stale/provisional warnings.

## Security

`APP_ADMIN_PASSWORD_HASH` MUST contain a valid bounded Argon2id v19 hash; startup fails if it is absent or invalid. Sessions use an in-memory server-side store, HTTP-only `SameSite=Strict` cookies, 30-day inactivity expiry refreshed on every request, rotation after login, and flush on logout. Restarting the process logs the administrator out.

Every unsafe request, including login, MUST carry a session-backed synchronizer CSRF token. Login attempts MUST allow five attempts per trusted client IP in a rolling five-minute window. Forwarding headers are trusted only for configured proxies and one explicitly selected header mode. Secrets, password hashes, session IDs, and CSRF tokens MUST NOT be logged. Non-debug builds require secure cookies; debug builds may use insecure local cookies.

Trusted proxies MUST strip untrusted forwarding input or append their immediate peer while preserving chain order. Only proxies within `APP_TRUSTED_PROXY_CIDRS` may supply the selected `APP_TRUSTED_PROXY_HEADER` format.

## Local Run Contract

After copying `.env.example` to `.env` and supplying a valid `APP_ADMIN_PASSWORD_HASH`, `cargo run` MUST be sufficient to run the complete local application. It MUST load configuration, create/connect and migrate SQLite, enable foreign keys, compose adapters and services, bind the configured address, log the local URL without secrets, and shut down gracefully. The independent password helper is run with `cargo run --manifest-path tools/password-hash/Cargo.toml`.

Local startup MUST NOT require Docker, a frontend build, manual migrations, SQLx metadata generation, or Frankfurter availability. Local monetary databases are pre-release and may need to be deleted and recreated after canonical persistence changes.

## Maintenance

This document is the product and architecture source of truth. Update it before changing behavior, then synchronize README status, configuration examples, migrations, tests, and SQLx metadata in the same change.
