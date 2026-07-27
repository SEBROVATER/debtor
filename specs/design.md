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

## Rates And Settlements

A spending retains its source currency; a group currency is a freely changeable settlement display target.

- Historical mode is the default and requests a rate for each spending date.
- Current mode uses the UTC calculation date for every spending and is not persisted.
- Future dates in historical mode use the latest current rate and are marked provisional.
- Cache keys include source, target, and requested/effective date. Historical entries may live for the process lifetime; current entries refresh on UTC day rollover.
- On provider failure, a matching stale cached rate MAY be used with a warning. Without one, only the debts view returns retryable `503`; CRUD remains available.
- Final balances are quantized to target minor units with largest-remainder allocation and participant ID tie-breaking, preserving an exact zero sum.
- Settlement uses a deterministic greedy matcher ordered by descending absolute balance then participant ID. Transfers are positive, settle every balance, do not repeat a participant pair, and number at most `n - 1`. They are not guaranteed globally minimal.
- The debts view MUST disclose the selected mode, calculation time, target currency, unique rates used, and stale/provisional warnings.

## Security

`APP_ADMIN_PASSWORD_HASH` MUST contain a valid Argon2id hash; startup fails if it is absent or invalid. Sessions use an in-memory server-side store, HTTP-only `SameSite=Strict` cookies, 30-day inactivity expiry, rotation after login, and flush on logout. Restarting the process logs the administrator out.

Every unsafe request, including login, MUST carry a session-backed synchronizer CSRF token. Login attempts MUST be throttled per trusted client IP. Forwarding headers are trusted only for configured reverse proxies. Secrets, password hashes, session IDs, and CSRF tokens MUST NOT be logged. Production runs behind HTTPS with secure cookies enabled.

## Local Run Contract

After copying `.env.example` to `.env` and supplying a valid `APP_ADMIN_PASSWORD_HASH`, `cargo run` MUST be sufficient to run the complete local application. It MUST load configuration, create/connect and migrate SQLite, enable foreign keys, compose adapters and services, bind the configured address, log the local URL without secrets, and shut down gracefully.

Local startup MUST NOT require Docker, a frontend build, manual migrations, SQLx metadata generation, or Frankfurter availability. HTTP cookies may be insecure locally only when `APP_SESSION_COOKIE_SECURE=false`; production configuration MUST require secure cookies.

## Maintenance

This document is the product and architecture source of truth. Update it before changing behavior, then synchronize README status, configuration examples, migrations, tests, and SQLx metadata in the same change.
