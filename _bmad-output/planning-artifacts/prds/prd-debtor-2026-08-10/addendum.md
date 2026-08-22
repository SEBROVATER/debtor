# Debtor PRD Addendum

## Contract Authority

This addendum is the complete technical companion to `prd.md` as of 2026-08-11 and preserves downstream depth supplied by `specs/design.md` and `_bmad-output/project-context.md`. `specs/design.md` remains normative and governs any conflict. If these artifacts diverge, downstream work stops until all three are synchronized; this file never silently supersedes another source.

## Foundational Product Boundaries

- Debtor is permanently a private single-Administrator workflow.
- Participants are independent Group-owned accounting identities; global reusable Participants and Memberships are outside the product model, and there is no separate global Participant management surface.
- Arbitrary timeframe sums are deferred beyond v1; v1 includes only the fixed current UTC calendar month.

## Architecture, Ownership, And Inputs

- Preserve the inward dependency direction `debtor (root) -> debtor-web / debtor-infra -> debtor-application -> debtor-domain`. This expresses permitted direction rather than an exhaustive direct-edge whitelist; web and infrastructure may use domain entities and value objects when application-facing interfaces require them.
- `debtor-domain` owns synchronous deterministic rules without I/O or framework dependencies. `debtor-application` owns use cases, lifecycle and spending input policy, authentication admission and verification orchestration, and narrow mockable ports. `debtor-infra` owns SQLx, HTTP clients, cryptography, caching, and concrete adapters. `debtor-web` owns Axum, trusted-proxy resolution, strict forms, CSRF and session mechanics, cookies, Askama rendering, view models, and HTTP error mapping. The root owns configuration, concrete-adapter composition, migrations, startup, lifecycle, and shutdown.
- Framework, persistence, HTTP, cryptography, session, and adapter types, including Axum, SQLx, reqwest, Argon2, and tower-sessions types, must not cross application-owned ports. Adapters for external effects and clocks are constructor-injected; use cases remain runnable with fakes and without Axum, SQLite, network access, or wall clock.
- Transport adapters decode field structure and retain raw submitted text only. Transport-neutral application inputs parse amount, currency/category code, `%Y-%m-%d` date text, and proportional weight text; validate the single Payer, Share selections, ownership, lifecycle, and Share mode; construct allocations; and apply all financial invariants. Web parsing must not construct financial allocations.
- Group and Participant names are trimmed, non-empty, and at most 100 Unicode characters. Spending descriptions are trimmed, non-empty, and at most 200 Unicode characters. Dates parse strictly as `%Y-%m-%d`, must be on or after `2025-01-01`, and use UTC for current calculations and defaults. Participant colors must satisfy normalized `#RRGGBB` shape. Application validation and transactional persistence guards remain distinct: application owns policy, while infrastructure rechecks race-sensitive persisted facts.

## Monetary Representation And Corruption Handling

- Use `rust_decimal::Decimal` for all money and rates. Floating-point types, lossy numeric conversion, SQL monetary parsing, SQL monetary conversion, and SQL monetary aggregation are forbidden.
- Every Total and persisted Payer/Share amount is positive, precision-valid, and at most `999_999_999_999`. JPY and KRW permit zero minor units, OMR permits three, and every other supported currency permits two; excess precision is invalid rather than rounded.
- Exactly one active Group-owned Participant pays the Spending Total. Share allocations are nonempty and Participant-unique, zero Shares are invalid, and Shares equal the Total exactly in Source Currency minor units. Proportional weights are positive, at most `1,000,000`, and at most six fractional digits; Preview and commit share one checked integer-ratio operation with descending remainders and ascending Participant ID ties. Exact initialization divides total minor units by active-Participant count and assigns residual units in ascending Participant ID order; acceptance requires a zero remaining/excess difference.
- Persist money as canonical decimal SQLite `TEXT`. Domain and repository Rust code owns exact parsing, canonical formatting, precision, positivity, equality, and checked aggregation. Repository decoding must revalidate canonical form and reject malformed or noncanonical stored values as corruption rather than normalize them.
- Arithmetic, quantization, and settlement use checked domain errors. A failed conversion must never panic, default to zero, or produce partial Balances or Settlement Transfers.
- The Current-Month Summary always preserves Source Currency totals as a conversion-independent fallback and separately provides Group Currency totals.

## IDs And Determinism

- Ledger entity IDs are positive `i64` values. UUIDs are limited to session and CSRF randomness.
- Domain behavior is synchronous and deterministic. Any unordered input that can affect output must be explicitly sorted or held in ordered collections such as `BTreeMap` or `BTreeSet`; Participant ID is the final tie-breaker.
- Converted Balances use largest signed-remainder quantization at target-currency precision with Participant-ID tie-breaking and exact zero-sum conservation.
- Settlement uses deterministic greedy matching ordered by descending absolute Balance and then Participant ID. For `n` Participants included in the settlement calculation, Settlement Transfers are positive, complete, pair-unique, and at most `n - 1`; global transfer-count minimality is not promised.

## SQLite Integrity And Write Semantics

- The supported production topology is one application process and one local SQLite volume. Multiple application instances and external SQLite writers are unsupported.
- SQLite explicitly uses WAL, `synchronous=FULL`, foreign keys, and a five-second busy timeout. One process-local write gate serializes every ledger mutation and has a five-second acquisition timeout. A gate timeout occurs before transaction creation, so timed-out work starts no transaction or guarded side effect.
- Group ownership, active Participant eligibility, aggregate replacement, allocations, and commit remain in one transaction. An update may retain an archived Participant only in an existing Payer or Share role. It may not introduce the archived Participant into a new role or change an existing role. Referenced Group deletion is restricted; a history-free Group may be deleted with its unreferenced Group-owned Participants. Participants otherwise archive/restore and are never independently cascade-deleted through the application.
- Group creation accepts only a valid name and persists `USD` as the initial Group Currency. New Groups enter Manage; established Groups enter Summary. Archived Groups and Participants are omitted from active lists and loaded through separate contextual archived views.
- Participant archival requires one immutable all-time Historical-mode ledger snapshot, UTC calculation context, and quote bundle with an exact zero Group Currency Balance. Final admission revalidates ledger epoch, UTC date, and quote eligibility; any mismatch or missing eligible quote blocks the mutation with retryable feedback. Provider calls hold no database transaction. Rate evidence is not persisted, so a later attempt may observe a provider revision. Restore does not require Balance eligibility.
- Among admitted valid mutations, the last committed write wins. Optimistic revision columns and stale-edit conflicts are not used.
- SQLite structurally restricts supported currency and category codes, boolean flags, bounded non-empty text, Participant color shape, ISO Spending dates on or after `2025-01-01`, relationships, and referenced-Group deletion. It must not duplicate Unicode trimming or Rust monetary arithmetic.
- Compile-time checked SQLx macros are required. The fixed WAL-checkpoint PRAGMA is the sole verified unchecked-query exception; committed `.sqlx` offline metadata must match checked SQL and migrations.

## Snapshot, Pagination, And Direct Loading

- A complete Spending aggregate is loaded from one SQLite snapshot. Debt calculation materializes Group Currency and every complete Spending before committing/releasing the read transaction; provider requests must never hold a database transaction.
- Ordinary Spending history uses fixed 25-item keyset pages ordered by `(spent_date DESC, id DESC)`.
- Detail, edit, and delete paths load one complete aggregate directly rather than materializing Group history. Group-owned Participant relationships and archived historical identities remain resolved by those direct loads; historical views display each Participant's current name after a rename.

## Safe Failures And Diagnostic Allowlists

- Application-facing failures use structured safe reason categories. Raw SQLx, reqwest/provider, cryptography, session, and other adapter errors must not cross inward-facing ports or appear in HTTP responses. Domain, application, and adapter error types use `thiserror`; `anyhow` is confined to root process, configuration, and runtime orchestration.
- Debt arithmetic, quantization, and settlement errors map to one fixed sanitized application calculation reason. They must not panic, substitute zero, or return partial calculations. For monthly summaries, missing quotes remain retryable exchange-rate `Unavailable` and checked arithmetic remains `Calculation`; the summary projection consumes either cause and collapses them only into a whole-section unavailable rendering while retaining Source Currency totals.
- SQLite logs may contain only a fixed operation name and bounded result category. The operation allowlist is `statement`, `pool_acquire`, `connection`, and `adapter`; the category allowlist is `contention`, `statement`, `readonly`, `io`, `integrity`, `open`, `constraint`, `timeout`, `closed`, `protocol`, and `other`.
- SQLite diagnostics must not contain SQL, SQLite/database messages, monetary or other values, entity identifiers, query strings, provider URLs, or request-derived data. Logs must also exclude credentials, password hashes, cookies, session IDs, CSRF tokens, client-IP limiter keys, and client IPs.

## Rates, Caches, And Provider Bounds

- Decode provider JSON numbers lexically and with arbitrary precision directly into `Decimal`.
- For original requested date `R` and UTC calculation date `C`, fetch date is `F = min(R, C)`. Deduplication, single-flight, and cache lookup key on `(source, target, R, F)`; provider effective date is returned quote evidence. Fixed-past stale fallback requires the exact key. After rollover refresh failure, Current fallback selects the latest prior current-class quote for the pair; future fallback also requires the same original `R`. Same-currency conversion is a disclosed synthetic exact rate of `1` without provider I/O.
- Stable historical and refreshable current/future cache classes are each capped at 4,096 entries and use deterministic LRU eviction. Past historical entries may live for the process lifetime and remain stale-eligible without an age limit. Current/future contexts refresh on UTC rollover and prior refreshable quotes remain eligible inclusively through seven UTC calendar days after prior `F`. Eviction may cause refetching and later calculations may observe provider revisions, but one calculation's immutable quote bundle and context boundaries do not change.
- Provider requests have a five-second connect timeout, a 20-second total timeout, and a 64 KiB response limit. At most four provider calls are in flight globally. Identical uncached keys use per-key single-flight.
- Each debt calculation deduplicates unique rate contexts and issues at most four provider requests concurrently. Completion order must not alter Balances, rate disclosure order, or warnings.

## HTTP Forms, Statuses, And Dispatch

- Render semantic HTML with Askama and vanilla CSS. Pinned self-hosted HTMX core plus its pinned official `response-targets` extension are the only currently approved browser-side JavaScript infrastructure and may progressively enhance valid native links/forms. The extension declaratively routes expected `4xx`/`5xx` fragments to stable announced status targets. Every core interaction retains a full-page path when HTMX is unavailable. Manually authored application JavaScript, inline scripts and event handlers, custom HTMX extensions, application-owned HTMX event handlers, client-side financial state, and features requiring imperative post-swap behavior are forbidden. Other official HTMX extensions require explicit design and security approval before addition.
- One shared strict form/CSRF/submission-token extractor rejects malformed, missing, duplicate, and unknown fields before route-specific parsing or use-case dispatch. CSRF rejection likewise occurs before password verification and dispatch. Every rendered unsafe form receives a session-bound single-use token distinct from CSRF. One web store separates 4,096 anonymous tokens (one per session, ten-minute inactivity expiry) from 1,024 authenticated tokens (32 per session, 30-minute absolute expiry); indexed cleanup is a mandatory supervisor. Validation before dispatch preserves the token; immediately before dispatch the server atomically reserves it. Reservation is terminal regardless of outcome. Missing, unknown, expired, reserved, or consumed tokens return `409 Conflict` without invoking a use case.
- Group, Participant, and Spending validation failures return `422 Unprocessable Entity`, render inline errors, and retain every raw submitted value, including the submitted Participant color. Successful mutations redirect with `303 See Other`.
- Archived Group pages suppress mutation/settings controls. Direct archived mutation and form routes return `409 Conflict` before invoking any use case.
- Ledger-mutation dispatch is marked immediately before the first state-changing use-case call. A 30-second absolute pre-dispatch deadline covers body extraction, authentication, CSRF, and asynchronous web prechecks. After dispatch, no generic request timeout may cancel the use case; the response reports a definitive commit or rollback result.
- Rate unavailability without a context-matching quote maps debt calculation to retryable `503 Service Unavailable`; ordinary ledger operations remain dispatchable. Correct-password session promotion at authenticated capacity also returns retryable `503`. Login-limiter key exhaustion for a new client fails closed with retryable `429 Too Many Requests`.

## Authentication, Sessions, And CSRF

- `APP_ADMIN_PASSWORD_HASH` is required, capped at 256 encoded bytes, cheaply rejected for noncanonical structure, and validated before database connection or migration. It must be an Argon2id v19 PHC hash with exactly `m`, `t`, and `p`: memory cost `19,456..=65,536` KiB, time cost `2..=5`, parallelism `1..=4`, decoded salt length `16..=64` bytes, and output length `32..=64` bytes. The independent helper emits the fixed profile `m=19,456`, `t=2`, `p=1`, a 16-byte OS-generated salt, and a 32-byte output. Password verification concurrency is capped at two.
- Sessions use a process-local, in-memory, server-side store and HTTP-only `SameSite=Strict` cookies. Non-debug builds require secure cookies; debug builds may use insecure cookies for local HTTP. Process restart invalidates every session.
- Anonymous login/CSRF sessions expire after ten minutes of inactivity, are explicitly saved before rendering login, and are capped at 4,096 live records. Full anonymous capacity rejects new anonymous admission; anonymous churn never evicts an authenticated session.
- Authenticated sessions expire after 30 days of inactivity, refreshed on every request, and are capped at 32 live records without eviction. If correct-password promotion finds that cap full, it flushes the anonymous login session and returns retryable `503`.
- Successful login atomically rotates and durably stores session ID, authenticated state, and CSRF before cookie/redirect; persistence failure emits no authenticated cookie. The limiter reserves every post-CSRF password verification, including a correct one, and resets only after durable promotion. Logout flushes the session. Session/token expiry cleanup uses mandatory supervised indexed workers; failure fails readiness, stops admission, and initiates shutdown.
- Every unsafe request, including login, requires exactly one correct session-backed synchronizer token. Missing, duplicate, malformed, and incorrect tokens are rejected by the shared extractor before route parsing or dispatch.
- Login permits five attempts per trusted client IP in a rolling five-minute window. The limiter holds at most 4,096 active client keys, uses an indexed next-expiry structure, does not evict active keys, and fails closed with retryable `429` for an unseen key at capacity.

## Headers, Proxy Trust, And Session-Free Routes

- Login and authenticated HTML responses send `Cache-Control: no-store`, `X-Content-Type-Options: nosniff`, `Referrer-Policy: no-referrer`, and `Content-Security-Policy: default-src 'none'; script-src 'self'; script-src-attr 'none'; connect-src 'self'; style-src 'self' 'unsafe-inline'; form-action 'self'; base-uri 'none'; frame-ancestors 'none'`.
- Probe and static routes, including pinned HTMX and extension assets, must neither create nor load sessions.
- Forwarding headers are accepted only from immediate peers within `APP_TRUSTED_PROXY_CIDRS` and only in the single format selected by `APP_TRUSTED_PROXY_HEADER`. Production requires a nonempty CIDR set and recognized singular mode before socket admission; direct-peer mode is debug/local only. The edge strips untrusted forwarding input or appends its immediate peer while preserving chain order. Client identity and login limiting must resolve identically over HTTP/3 and TCP fallback.

## Admission, Timeouts, Probes, And Shutdown

- Application login bodies are limited to 8 KiB and other form bodies to 256 KiB.
- User traffic has 64 in-flight permits; login has four in-flight permits. `/healthz` and `/readyz` share a separate four-request probe budget so user saturation cannot starve orchestration.
- Safe dynamic reads other than Debts, and login, have a 30-second timeout. Debts have a 90-second timeout. Probes have a two-second outer timeout, and SQLite readiness has a one-second inner timeout.
- Write-gate acquisition and SQLite lock waiting are each bounded at five seconds. The 30-second mutation pre-dispatch deadline and the no-timeout-after-dispatch rule remain separate from those storage waits.
- `/healthz` reports process liveness. `/readyz` checks SQLite and mandatory in-process supervisor health only; exchange-rate-provider availability and ledger contents never gate startup or readiness.
- Graceful shutdown stops admission and drains HTTP for at most ten seconds, then waits without a fixed total deadline until no dispatched mutation remains running before checkpoint and pool close. The executor synchronously publishes authoritative `Committed`/`RolledBack` before response work. Task failure may report `RolledBack` only when authoritatively established; otherwise it publishes `Unknown`, makes shutdown fatal, suppresses automatic retry, and is never represented as rollback. Checkpoint failure leaves WAL sidecars intact for recovery.

## Edge Transport And Rollout

- A sanitizing HTTPS reverse proxy owns TLS, automatic certificates, HTTP/3/QUIC, `Alt-Svc`, and client-facing HTTP/2 or HTTP/1.1 fallback. Debtor remains a private HTTP/1.1 TCP backend with no certificate, QUIC, or UDP listener dependency; direct insecure HTTP is debug/local only.
- The edge sanitizes forwarding headers before every backend request, and its source CIDR and forwarding mode must match `APP_TRUSTED_PROXY_CIDRS` and `APP_TRUSTED_PROXY_HEADER`.
- TLS/QUIC early data is disabled or unsafe early-data requests receive `425 Too Early`. Only `GET` and `HEAD` may traverse an explicitly marked early-data path; CSRF does not make a mutation replay-safe.
- The edge reuses private HTTP/1.1 backend connections. Backend connect and response-header timeouts may be bounded, but no edge request, read, write, or stream timeout may expire before an admitted mutation reaches definitive completion after dispatch.
- Edge body limits are at most 8 KiB for `/login` and 256 KiB for other form endpoints.
- HTTP/3 rollout begins with a short `Alt-Svc` lifetime. Before increasing it, verify UDP/443 reachability and edge telemetry, TCP fallback to HTTP/2 or HTTP/1.1 when UDP is blocked, `425` handling for unsafe early data, and identical forwarded client identity through every protocol.

## Local Run And Tool Independence

- After copying `.env.example` to `.env` and setting a valid `APP_ADMIN_PASSWORD_HASH`, `cargo run` is sufficient to load configuration, create or connect to the SQLite database, run migrations, enable foreign keys, compose adapters and services, bind the configured address, log a non-secret local URL including its `http://` scheme, and shut down gracefully.
- Local startup must not require Docker, a frontend build, manual migrations, SQLx metadata generation, or Frankfurter availability.
- Generate the password hash with `cargo run --manifest-path tools/password-hash/Cargo.toml`. Secrets must not appear in commands, logs, fixtures, or committed files.
- Local monetary databases are pre-release artifacts and may need deletion and recreation after canonical-persistence or migration rewrites.

## Maintenance And Pre-Release Policy

- `specs/design.md` is the normative product and architecture contract. Update it before behavioral implementation, then synchronize affected ADRs, README status, configuration examples, migrations, tests, and SQLx metadata in the same change.
- A later ADR must identify every decision it supersedes, and `specs/design.md` must be synchronized with that supersession.
- Before first deployment, breaking Rust APIs, configuration, routes, migrations, and database schemas are allowed when they produce a cleaner architecture. Remove superseded paths instead of keeping compatibility shims. Pre-release migrations may be rewritten, and database compatibility is not promised.
- Security, accounting, and historical-integrity invariants remain mandatory through pre-release breaking changes.

## Toolchain, Dependencies, And Workspaces

- Pin Rust `1.97.1`, edition `2024`, MSRV `1.97`, Cargo resolver `3`, and the minimal toolchain profile with `rustfmt` and `clippy`.
- Pin Axum `0.8.9`, Askama `0.16.0`, Tokio `1.53.1`, Tower `0.5.3`, tower-http `0.6.11`, tower-sessions `0.15.0`, SQLx/sqlx-cli `0.9.0` with bundled SQLite, reqwest `0.13.4` with rustls, rust_decimal `1.42.1`, chrono `0.4.45`, Argon2 `0.5.3`, thiserror `2.0.20`, anyhow `1.0.104`, serde `1.0.229`, serde_json `1.0.151`, and UUID `1.24.0`.
- Preserve lockfiles, use `--locked`, and consult current crate documentation before changing framework or library APIs.
- The production workspace and `tools/password-hash` are independent Cargo workspaces. They must be validated separately; the helper is not folded into production workspace checks.

## Testing Contract

- Domain financial tests use examples, boundary cases, and property tests for precision, checked arithmetic, allocation, quantization, deterministic ordering, exact conservation, and settlement completeness.
- Application tests run without Axum, SQLite, network, or wall clock, using injected clocks and lightweight fakes backed by simple maps, `Mutex<Vec<_>>`, or atomics. Do not add a mocking framework without concrete need.
- Infrastructure and web adapters are tested separately for malformed and oversized external input, canonical persistence corruption rejection, safe error mapping, cache bounds/LRU/single-flight, provider concurrency, password/session/limiter capacity behavior, and strict form/CSRF behavior.
- Use `#[sqlx::test]` and temporary file databases for WAL, locking, multi-connection behavior, and paired migration/constraint coverage.
- Concurrency tests coordinate with barriers, notifications, or deliberately held locks rather than timing sleeps. They assert that rejected or timed-out work starts no guarded side effect and that permitted completion-order variation does not alter deterministic outputs.
- Web tests use shared fake use cases and verify statuses, security headers, redirects, retained submitted values, malformed/duplicate/unknown-field and CSRF rejection, and no dispatch for every pre-use-case rejection.
- Retain a root real-socket smoke test covering authentication and CSRF, an authenticated read, startup ordering, and bounded shutdown.
- Put each regression test in the layer owning the invariant. Cross-layer tests are reserved for composition and adapter contracts. Keep test-only `allow` attributes narrow and never weaken workspace lint policy for helpers.

## Code Quality And Validation

- Keep `cargo fmt` clean; workspace Clippy uses `pedantic` and validation denies warnings. Unsafe Rust is forbidden, production paths avoid `unwrap`/`expect`, and lint suppression must not be broad.
- Feature modules use plural nouns such as `groups`, `participants`, `spendings`, and `debts`, and mirror capabilities across layers where useful. Interfaces use `*Reader`, `*Repository`, `*Provider`, or `*UseCases`; implementations use `*Service`, `*Store`, `*Client`, or `*Gate`; transport-neutral commands use `*Input`; persistence rows use private `Db*`; rendering projections use `*Template`, `*Row`, or `*View`.
- Public APIs have rustdoc and fallible methods document `# Errors`. Comments explain non-obvious constraints rather than restating code. Prefer the smallest correct local change and avoid speculative abstractions.
- Validate the production workspace with `cargo fmt --all -- --check`, `cargo check --workspace --all-features --locked`, `SQLX_OFFLINE=true cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`, and `cargo test --workspace --all-features --locked`.
- Run architecture fitness with `cargo run --bin architecture-check --locked`; it verifies required package presence and dependency direction, while targeted tests verify responsibility ownership. Dependency advisories, sources, and permissive-license policy are checked with `cargo deny check` when dependency policy changes.
- When checked SQL or migrations change, migrate a temporary SQLite database and run `SQLX_OFFLINE=false DATABASE_URL=sqlite:///tmp/debtor-sqlx.db?mode=rwc cargo sqlx prepare --workspace --check`, then commit refreshed `.sqlx` metadata.
- Validate the independent helper with `cargo fmt --manifest-path tools/password-hash/Cargo.toml -- --check`, `cargo clippy --manifest-path tools/password-hash/Cargo.toml --all-targets --all-features --locked -- -D warnings`, and `cargo test --manifest-path tools/password-hash/Cargo.toml --locked`.
- Never use `cargo build --release` for routine checking, testing, or validation; use debug `cargo check`, `cargo test`, and `cargo run`.

## Decision Rationale

- The researched comparable landscape included Splitwise, tricount, Settle Up, Spliit, I Hate Money, Cospend, and SplitPro. These alternatives were rejected because their extra collaboration features or deployment complexity did not fit Debtor's product boundaries.
