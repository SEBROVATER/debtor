# Debtor Design Contract

## Goal

Debtor is a private, password-gated ledger for one administrator to record shared, multi-currency spendings and view advisory settlement transfers. Transfers are derived on demand; repayments, paid status, and settlement checkpoints are not recorded.

## Decision Records

This document is the normative product and architecture contract. The rationale and consequences for the foundation decisions are recorded in [ADR 0001: Foundation Architecture](adr/0001-foundation-architecture.md). Later ADRs MUST explicitly identify any decision they supersede, and this document MUST be synchronized in the same change.

## Release Scope

The first release MUST provide groups, group-owned participants, spending CRUD with exactly one payer and proportional or exact shares, current-month group spending summaries, and advisory group settlements in twelve supported currencies and eight spending categories. Groups and participants support archive/restore; historical allocations remain visible.

The supported currency codes are `USD`, `EUR`, `RUB`, `KGS`, `TRY`, `KZT`, `UZS`, `CNY`, `KRW`, `JPY`, `OMR`, and `TJS`. The supported spending category codes and current display labels are `food`, `transport`, `housing`, `fun`, `shopping`, `bills`, `health`, and `other`.

It MUST NOT provide statistics beyond the current-month group spending summary, multiple payers, direct percentage or itemized splits, repayment tracking, settlement date ranges, exact global transfer minimization, persistent sessions, manual rate refresh, registration, usernames, or custom application JavaScript. Pre-release migrations MAY be rewritten; database compatibility is not promised.

## User Model

There is exactly one administrator as a permanent product boundary. Authentication is a password gate with no username, user table, registration, participant login, tenant, or multi-user authorization model. Participants are accounting identities, not application users.

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
- Lifecycle and spending input policy MUST be application-owned: transport adapters may decode field structure and preserve raw submitted text, but application commands MUST parse amounts/codes, validate payer/share selections, construct allocations, inspect archived lifecycle state, and apply financial invariants for every inbound adapter. Transactional persistence guards remain mandatory for race-safe facts.
- Debt arithmetic, quantization, and settlement failures MUST return checked domain errors and map to a fixed, sanitized application calculation reason; they MUST NOT panic, default failed conversions to zero, or return partial transfers.
- The supported first-release production topology is one application process with one local SQLite volume behind a sanitizing HTTPS reverse proxy. Direct insecure HTTP is debug/local only. Multiple application instances and external SQLite writers are unsupported.
- SQLite persistence uses explicit WAL with `synchronous=FULL`, a five-second busy timeout, one process-local ledger write gate with a five-second acquisition timeout, and snapshot-consistent reads. Among admitted valid operations, the last committed write wins; optimistic revision columns are not used.

External effects MUST be constructor-injected through application-owned ports. Use cases MUST be testable with fakes, without a database, network, Axum, or wall clock.

## Accounting And History

- Money MUST use exact `Decimal`, be persisted as canonical SQLite `TEXT`, and be aggregated in Rust. Floating point and SQL monetary aggregates are forbidden.
- Currency precision is zero minor units for JPY/KRW, three for OMR, and two for other supported currencies. Input with excess precision is invalid.
- Group and participant names are trimmed, non-empty, and at most 100 Unicode characters. Spending descriptions are trimmed, non-empty, and at most 200 Unicode characters. Participant colors are normalized `#RRGGBB` values. Dates are strict `YYYY-MM-DD` dates on or after `2025-01-01`; totals and all persisted payer/share amounts are positive and at most `999_999_999_999`.
- Exactly one active group-owned participant is the payer and pays the spending total. Share totals MUST equal the spending total exactly in source-currency minor units.
- Proportional and Exact are the only share modes. Proportional mode initially selects every active participant with weight `1`, permits deselection, requires every selected weight to be a positive decimal, displays each resulting exact-currency amount, and assigns residual minor units by largest fractional remainder with participant ID as the tie-breaker. Exact mode initially selects every active participant with an equal minor-unit allocation, permits deselection and amount editing, and displays the remaining or excess difference until selected shares equal the total.
- Transport-neutral application commands MUST own the single-payer selection, proportional/exact share construction, participant-ID validation, allocation precision, largest-remainder allocation, and total validation. Web form parsing MUST NOT construct financial allocations beyond decoding submitted values.
- Every participant belongs to exactly one group and is never reused across groups. New allocations require an active participant owned by the spending's group. An update may retain an archived participant only in the same existing payer or share role; it may not introduce or change that archived participant's role. Historical archived identities remain in history, balances, and transfers.
- Archived groups are read-only. A group with no spendings may be deleted together with its unreferenced group-owned participants; a group with any spending may only be archived. Participants are otherwise archived/restored, never deleted independently through the application.
- Participant archival MUST use the complete all-time Historical-mode calculation and is allowed only when that Participant's Group Currency balance is exactly zero. If required rates have no fresh or eligible stale quote, archival is blocked with retryable feedback and no state change. Eligibility and archival MUST be race-safe against concurrent spending mutations; restoration does not require a balance check.
- Spending aggregate writes, including participant ownership and active-eligibility checks and allocations, MUST be transactional. Latest valid write wins.
- Complete spending aggregates MUST be loaded from one database snapshot. Debt calculation MUST materialize group currency and complete spendings before releasing the database read transaction; network rate requests MUST NOT hold a database transaction.
- The infrastructure adapter MUST serialize all ledger mutations through one process-local write gate. Gate acquisition and SQLite lock waits are bounded at five seconds; a timed-out mutation MUST not begin a transaction.
- The group page MUST offer participant add/edit/archive/restore and one expense form with exactly one Payer plus Proportional/Exact Share choices. Description and amount start empty; Source Currency defaults to Group Currency; date defaults to the current UTC date; Category has no default; Payer selection and Share editing use one Participant allocation table. No Payer is initially selected, and selecting one assigns the full Total. Input modes and proportional weights are not persisted, so every edit opens Exact with stored Payer and Share amounts. There is no separate global participant-management surface. Ordinary spending history MUST use fixed 25-item keyset pagination ordered by `(spent_date DESC, id DESC)`; detail/edit/delete MUST load one complete aggregate directly rather than materializing all group history.
- The persistent Add Spending action opens a focused full-page form and returns to Transactions with the committed row visible. HTMX MAY update allocation previews on field change, but the same form MUST provide an explicit native Preview submission and every enhanced link/form MUST remain a valid full-page path.
- Group creation requires only a valid group name and assigns `USD` as the initial group currency. A newly created group opens in its contextual Manage section for currency and participant setup; an established group opens in Summary. Active lists exclude archived groups and participants, which are available through separate contextual archived views for restoration.
- The group page MUST summarize spendings dated in the current UTC calendar month. It MUST show the group total and per-payer totals grouped by original source currency, plus the same totals converted to the group currency using each spending date's historical rate. All-time debts remain a separate calculation. If conversion cannot obtain a current or context-matching stale quote, source-currency totals and ordinary group functionality remain available while only the converted summary shows a retryable warning.
- New participant forms MUST suggest a varied valid color from the server while allowing the administrator to select a different color before submission. Validation re-renders retain the submitted color.
- Historical spending detail MUST remain readable for archived groups and resolve current participant names for archived identities. Spending CRUD, group settings, empty-group deletion, and participant add/edit/archive/restore are server-rendered and CSRF protected.
- Validation failures on group, participant, and spending forms MUST return `422` with an inline error and all submitted values retained. Archived group pages MUST hide mutation/settings controls, and direct archived mutation or form routes MUST return `409` without invoking a use case.
- The single web interface MUST support the latest stable Chrome, Firefox, Safari, and Edge at viewport widths down to 320 CSS pixels. Every control MUST be reachable and operable without a pointer, use a visible focus indicator at least two CSS pixels thick with at least 3:1 contrast against adjacent colors, and have a programmatic label. Text contrast MUST be at least 4.5:1 for normal text and 3:1 for large text; user-interface components and meaningful graphics MUST reach 3:1. Inline errors MUST be programmatically associated with their fields. Formal accessibility certification is not required.
- Domain and repository Rust code owns exact `Decimal` parsing, canonical formatting, precision, positivity, allocation equality, and monetary aggregation. SQLite stores monetary values as `TEXT` and MUST NOT parse, convert with floating point, or aggregate monetary values.
- SQLite MUST structurally restrict referenced group deletion, supported currency/category codes, boolean flags, bounded non-empty text, valid participant color shape, and ISO spending dates on or after `2025-01-01`. These checks MUST NOT duplicate Rust monetary arithmetic or Unicode trimming rules.

## Rates And Settlements

A spending retains its source currency; a group currency is a freely changeable settlement display target.

- Historical mode is the default and requests a rate for each spending date.
- Current mode uses the UTC calculation date for every spending and is not persisted.
- Future dates in historical mode use the latest current rate and are marked provisional.
- Cache keys include source, target, original requested date, and effective fetch date. Both stable and refreshable cache classes are capped at 4,096 entries with deterministic LRU eviction. Past historical entries may live for the process lifetime; current and future historical entries refresh on UTC day rollover.
- Exchange-rate JSON numbers MUST be decoded lexically with arbitrary precision into `Decimal`. Provider calls use a five-second connect timeout, 20-second total timeout, and 64 KiB response limit. At most four provider calls may be in flight globally, identical uncached keys use per-key single-flight, and each debt calculation deduplicates unique rate contexts with at most four concurrent requests. Completion order MUST NOT change balances, disclosures, or warnings.
- On provider failure, the latest context-matching prior quote MAY be used with a stale warning. A fixed past-date historical quote remains stale-eligible without an age limit; a current or future-date quote is stale-eligible only through seven UTC calendar days after its effective fetch date. Without an eligible quote, the debts view returns retryable `503`; a current-month group summary retains its source-currency totals and shows the converted summary as retryable unavailable. CRUD remains available.
- Final balances are quantized to target minor units with largest-remainder allocation and participant ID tie-breaking, preserving an exact zero sum.
- Settlement uses a deterministic greedy matcher ordered by descending absolute balance then participant ID. Transfers are positive, settle every balance, do not repeat a participant pair, and number at most `n - 1`. They are not guaranteed globally minimal.
- The debts view MUST disclose the selected mode, calculation time, target currency, unique rates used, and stale/provisional warnings.

## Security

`APP_ADMIN_PASSWORD_HASH` MUST contain a valid bounded Argon2id v19 hash; startup fails before database connection or migration if it is absent or invalid. Sessions use a process-local in-memory server-side store, HTTP-only `SameSite=Strict` cookies, 30-day authenticated inactivity expiry refreshed on every request, rotation after login, and flush on logout. Anonymous login/CSRF sessions use ten-minute inactivity expiry, are explicitly saved before login rendering, and are capped at 4,096 live records without evicting authenticated sessions. Authenticated sessions are capped at 32 live records without eviction; a correct-password promotion at capacity flushes the anonymous login session and returns retryable `503`. Expired records are physically deleted every five minutes using an indexed expiry structure. Restarting the process logs the administrator out.

Every unsafe request, including login, MUST carry a session-backed synchronizer CSRF token. Login attempts MUST allow five attempts per trusted client IP in a rolling five-minute window, track at most 4,096 active client keys, and fail closed for an unseen key when full. Forwarding headers are trusted only for configured proxies and one explicitly selected header mode. Secrets, password hashes, session IDs, CSRF tokens, and client-IP limiter keys MUST NOT be logged. Non-debug builds require secure cookies; debug builds may use insecure local cookies.

Unsafe requests MUST use one shared CSRF-validating form extractor before route-specific parsing or use-case invocation. Missing, duplicate, malformed, or incorrect tokens are rejected. Every rendered unsafe form also carries a bounded, expiring, session-bound single-use submission token distinct from CSRF; the server atomically reserves it immediately before dispatch, permits one mutation attempt, and rejects a missing, unknown, expired, reserved, or consumed token with `409` and no use-case invocation. Validation before dispatch does not consume the token. Pinned self-hosted HTMX and its pinned official `response-targets` extension MAY progressively enhance native links and forms; custom application JavaScript, custom HTMX extensions, and inline script attributes are forbidden. Expected enhanced `4xx`/`5xx` fragments target a stable announced status region. Every core interaction MUST remain functional when HTMX is unavailable. Login and authenticated HTML MUST send `Cache-Control: no-store`, `X-Content-Type-Options: nosniff`, `Referrer-Policy: no-referrer`, and the progressive-enhancement content security policy defined by the foundation ADR. Probe and static routes, including HTMX assets, MUST not create or load sessions.

Trusted proxies MUST strip untrusted forwarding input or append their immediate peer while preserving chain order. Only proxies within `APP_TRUSTED_PROXY_CIDRS` may supply the selected `APP_TRUSTED_PROXY_HEADER` format.

## Edge Proxy Contract

- TLS, automatic certificates, HTTP/3/QUIC, `Alt-Svc`, and client-facing HTTP/2 or HTTP/1.1 fallback are edge responsibilities. Debtor remains a private HTTP/1.1 TCP backend and has no QUIC, TLS-certificate, or UDP listener dependency.
- The edge MUST sanitize forwarding headers before each backend request. Its source address/CIDR and selected forwarding-header mode MUST match `APP_TRUSTED_PROXY_CIDRS` and `APP_TRUSTED_PROXY_HEADER`. Client-IP resolution and login-rate-limit behavior MUST be identical over HTTP/3 and TCP fallback protocols.
- The edge MUST reject unsafe early-data requests with `425 Too Early`, or disable TLS/QUIC early data entirely. Only `GET` and `HEAD` may be allowed through an explicitly marked early-data path; CSRF does not make a replay-safe mutation.
- The edge MUST reuse backend connections and use a private HTTP/1.1 transport. Backend connect and response-header timeouts may be bounded, but no proxy read, write, stream, or request timeout may be shorter than an admitted mutation's definitive completion. Edge body limits MUST be at most 8 KiB for `/login` and 256 KiB for other form endpoints.
- Roll out HTTP/3 with a short `Alt-Svc` lifetime first, verify UDP/443 reachability and edge telemetry, then increase the advertised lifetime. Before increasing it, verify that blocked UDP still falls back to HTTP/2 or HTTP/1.1, unsafe early data receives `425`, and the same forwarded client identity is resolved through each protocol.

## Local Run Contract

After copying `.env.example` to `.env` and supplying a valid `APP_ADMIN_PASSWORD_HASH`, `cargo run` MUST be sufficient to run the complete local application. It MUST load configuration, create/connect and migrate SQLite, enable foreign keys, compose adapters and services, bind the configured address, log the local URL including its `http://` scheme without secrets, and shut down gracefully. The independent password helper is run with `cargo run --manifest-path tools/password-hash/Cargo.toml`.

Local startup MUST NOT require Docker, a frontend build, manual migrations, SQLx metadata generation, or Frankfurter availability. Local monetary databases are pre-release and may need to be deleted and recreated after canonical persistence changes.

## Operational Limits

- Login form bodies MUST be limited to 8 KiB and other form bodies to 256 KiB.
- User traffic MUST be limited to 64 in-flight requests and login to four in-flight requests. Health and readiness MUST use a separate four-request probe budget so user saturation cannot starve orchestration.
- Safe dynamic reads and login MUST have a 30-second timeout. Debts MUST have a 90-second timeout. Probes MUST have a two-second outer timeout and a one-second inner SQLite readiness timeout.
- Ledger mutations MUST use a 30-second absolute pre-dispatch deadline for body extraction, authentication, CSRF, and asynchronous web prechecks, then bound write-gate and SQLite waits. Once the use case is dispatched it MUST NOT be cut off by a generic timeout and MUST return a definitive commit or rollback result. A reverse proxy MUST NOT impose a shorter mutation timeout after dispatch.
- `/healthz` is process liveness. `/readyz` checks SQLite and mandatory in-process supervisor health only; Frankfurter availability and ledger contents MUST NOT gate startup or readiness.
- Shutdown MUST stop admission, drain for at most ten seconds, attempt a bounded WAL checkpoint, close the pool, and preserve WAL sidecars if checkpointing fails. Structured logs MUST be secret-safe. SQLite adapter diagnostics MAY emit only fixed operation names and bounded result-code categories; they MUST NOT emit SQL, database messages, values, identifiers, or request-derived data.
- Stable historical exchange-rate cache contexts MUST be capped at 4,096 with deterministic LRU eviction; current/future contexts MUST roll over by UTC date. Eviction may refetch but MUST NOT alter quote correctness or cross-context fallback.
- HTTP/3/QUIC MUST terminate at the sanitizing reverse proxy, not the Debtor process. The proxy MUST follow the edge proxy contract for fallback, forwarding-header sanitation, early-data rejection, body limits, backend transport reuse/timeouts, `Alt-Svc` rollout, and cross-protocol client-IP validation.
- The production workspace MUST use the pinned Rust toolchain and locked dependency checks. Architecture fitness MUST verify required package presence and dependency direction; targeted tests enforce responsibility ownership. Dependency advisories, sources, and permissive-license policy MUST be checked in CI.

## Maintenance

This document is the product and architecture source of truth. Update it before changing behavior, then synchronize the relevant ADR, README status, configuration examples, migrations, tests, and SQLx metadata in the same change. Before first deployment, breaking Rust APIs, configuration, routes, and database schemas are allowed when they produce a cleaner architecture; remove superseded paths rather than preserving shims. Security, accounting, and historical-integrity invariants remain mandatory. Pre-release migrations MAY be rewritten and database compatibility is not promised.
