# ADR 0001: Foundation Architecture

- Status: Accepted
- Date: 2026-08-04
- Scope: First-release architecture and operational foundation

## Context

Debtor is a private, permanently single-administrator ledger with exact multi-currency accounting, preserved historical identities, server-rendered HTML, and process-local authentication. The project needs explicit boundaries and operational assumptions before more feature work is added. Without them, direct framework and persistence concerns can spread inward, SQLite behavior can be interpreted differently by callers, and resource limits can remain implicit.

`specs/design.md` remains the normative product and architecture contract. This ADR records the rationale and consequences of the accepted foundation decisions. If a later ADR changes one of these decisions, it MUST identify the superseded section and `specs/design.md` MUST be synchronized in the same change.

## Decisions

### 1. Inward-only dependency direction

The permitted dependency direction is:

```text
debtor (root) -> debtor-web / debtor-infra -> debtor-application -> debtor-domain
```

The diagram expresses direction, not an exhaustive list of direct Cargo edges. Web and infrastructure MAY use domain entities and value objects directly when needed by application-facing interfaces. Domain and application MUST NOT depend on outer framework, SQLx, HTTP, cryptography, session, or adapter types. The root is the composition boundary.

### 2. Supported deployment topology

The supported first-release production topology is one application process with one local SQLite volume behind a sanitizing HTTPS reverse proxy. The proxy MUST strip untrusted forwarding input or append its immediate peer according to the configured forwarding mode. Direct insecure HTTP is for debug/local use only. Multiple application instances, shared sessions, and external SQLite writers are unsupported.

### 3. Rust-owned monetary invariants

Domain and repository Rust code owns exact `Decimal` parsing, canonical formatting, currency precision, positivity, allocation equality, and monetary aggregation. SQLite stores monetary values as `TEXT` and enforces structural relationships, but MUST NOT perform monetary parsing, floating-point conversion, or monetary aggregation. Converted monthly summaries accumulate exact per-payer values and use one conserved target-minor-unit quantization whose group total is the exact sum of displayed payer totals. This avoids divergent financial rules between Rust and SQL.

### 4. Durable, serialized SQLite writes

SQLite uses explicit WAL mode, `synchronous=FULL`, and a five-second busy timeout. The infrastructure adapter serializes all ledger mutations with one process-local write gate and gives gate acquisition five seconds. Spending eligibility checks, aggregate replacement, allocations, and commit remain one transaction. Among admitted valid operations, the last committed write wins. Optimistic revision columns are not introduced.

### 5. Snapshot-consistent reads

Complete spending aggregates are read from one database snapshot. Debt calculation first materializes group settlement currency and all complete spendings, then releases the database read transaction before making exchange-rate requests. A rate provider request MUST never hold a database transaction.

### 6. Application-owned policy

The application layer owns lifecycle decisions, authentication admission and verification orchestration, and spending payer/share mode policy. Web owns HTTP extraction, trusted-proxy resolution, CSRF and session mechanics, cookies, Askama view models, and harmless read composition. Infrastructure remains the final transactional eligibility guard for persisted allocations.

### 7. Bounded login limiting

Login attempts remain limited to five attempts per trusted client IP in a rolling five-minute window. The in-memory limiter holds at most 4,096 active client keys, uses an indexed next-expiry structure instead of scanning the entire map for every request, and fails closed with retryable `429` behavior for an unseen client when the key capacity is full. Active keys are not evicted and client IPs are not logged.

### 8. Exact and bounded rate processing

JSON exchange-rate numbers are decoded lexically with arbitrary precision into `Decimal`. Provider requests use a five-second connect timeout, 20-second total timeout, and 64 KiB response limit. At most four provider calls are in flight globally, and identical uncached keys use per-key single-flight. Each debt calculation deduplicates unique rate contexts and fetches at most four concurrently. Both cache classes are capped at 4,096 entries with deterministic LRU eviction. Completion order MUST NOT change balances, rate disclosure order, or warnings. Existing historical/current, stale, and provisional semantics remain authoritative. Participant archival uses one immutable time-of-decision snapshot, UTC calculation context, and quote bundle, revalidates ledger/time/quote eligibility before commit, and persists no rate evidence; a later attempt may observe a provider revision.

### 9. Bounded process-local sessions

Anonymous login/CSRF sessions use ten-minute inactivity expiry and are explicitly saved before a login page is rendered. At most 4,096 live anonymous records may exist; a full store rejects new anonymous admission and never evicts authenticated sessions. Authenticated sessions do not consume anonymous capacity and use 30-day inactivity expiry refreshed on requests. Login rotates the session ID and CSRF token; logout flushes state; restart invalidates all sessions. Expired records are physically deleted every five minutes. Cleanup failure fails readiness and triggers bounded shutdown.

### 10. Fixed HTTP resource budgets

Login form bodies are limited to 8 KiB and other form bodies to 256 KiB. User traffic has 64 in-flight request permits and login has four. Health and readiness use a separate four-request probe budget so user saturation cannot starve orchestration. Safe dynamic reads and login have a 30-second timeout; debts have a 90-second timeout; probes have a two-second outer timeout and a one-second inner SQLite readiness timeout.

Ledger mutations use one 30-second absolute deadline for all pre-dispatch work, including body extraction, authentication, CSRF, submission-token reservation, and asynchronous web prechecks, followed by bounded admission, write-gate, and SQLite waits. Every rendered unsafe form carries a bounded, expiring, session-bound single-use submission token. One process-local web-owned store separates 4,096 anonymous tokens (one per session, ten-minute inactivity expiry) from 1,024 authenticated tokens (32 per session, 30-minute absolute expiry); capacity exhaustion fails closed with retryable feedback, and indexed cleanup removes expired records. Session-expiry and submission-token cleanup are mandatory supervised workers whose failure fails readiness, stops admission, and initiates shutdown. The server atomically reserves a token immediately before dispatch; only one request may cross that boundary, and missing, unknown, expired, reserved, or consumed tokens return `409` without invoking a use case. Reservation is terminal after one dispatch regardless of its outcome or response delivery; validation before dispatch does not consume the token. The token prevents duplicate dispatch but is not an idempotency key that replays a prior response. There is no generic timeout after the use case begins. Mutations MUST return a definitive commit or rollback result. A reverse proxy MUST NOT impose a shorter mutation timeout after dispatch.

### 11. Local readiness and shutdown

`/healthz` is process liveness. `/readyz` checks SQLite and mandatory in-process supervisor health only, never Frankfurter availability or ledger contents. Startup validates configuration and the complete bounded Argon2id v19 policy before connecting or migrating SQLite. Shutdown stops admission and drains HTTP for at most ten seconds, then waits without a fixed total deadline until no dispatched mutation remains running before bounded WAL checkpoint and pool close. The mutation executor publishes authoritative `Committed`/`RolledBack` synchronously before response work; an unestablished outcome is `Unknown`, never false rollback, makes shutdown fatal, and suppresses automatic retry. Checkpoint failure preserves WAL sidecars for recovery. Logs are structured and secret-safe.

### 12. Pre-release migration policy

Pre-release migrations MAY be rewritten and local databases MAY need to be recreated. Breaking Rust APIs, configuration, and routes are also allowed when they remove superseded paths rather than preserve shims. Security, accounting, and historical-integrity invariants remain mandatory. Database compatibility is not promised. The repository MUST keep committed SQLx offline metadata synchronized with checked queries and migrations.

### 13. Native-first self-hosted HTMX enhancement

Pinned self-hosted HTMX plus its pinned official `response-targets` extension are the only permitted client-side libraries and MAY progressively enhance native links and forms. The extension routes expected `4xx`/`5xx` fragments to declared status targets; custom HTMX extensions are forbidden. Every core interaction remains a valid full-page path when HTMX is unavailable. Custom application JavaScript, inline scripts, and inline script attributes are forbidden. HTMX static assets are session-free. Login and authenticated HTML use `Content-Security-Policy: default-src 'none'; script-src 'self'; script-src-attr 'none'; connect-src 'self'; style-src 'self' 'unsafe-inline'; form-action 'self'; base-uri 'none'; frame-ancestors 'none'`.

## Consequences

### Benefits

- Financial rules have one authoritative implementation in Rust.
- The supported single-process topology gives deterministic writer ordering without adding revision conflicts or persistent coordination infrastructure.
- Snapshot reads prevent mixed-version aggregates and avoid holding database locks during network calls.
- Explicit resource budgets bound anonymous sessions, limiter keys, request work, provider work, and shutdown time.
- ADR rationale is durable while `specs/design.md` remains the single normative contract.

### Costs

- The application does not support horizontal scaling or external database writers in this release.
- WAL sidecars and `synchronous=FULL` trade filesystem space and write latency for durability.
- Per-key rate single-flight, bounded concurrency, session admission accounting, and supervised cleanup add implementation and test complexity.
- Unsafe mutation requests cannot be cut off by a generic application timeout after dispatch, so reverse-proxy configuration must respect the mutation contract.
- Native full-page fallbacks add route and focus-state obligations but keep core behavior recoverable without custom application JavaScript or HTMX runtime hooks.

## Rejected Alternatives

- A literal Cargo edge whitelist requiring web and infrastructure to re-export every domain type through application. This adds a facade without improving inward ownership.
- Multiple app instances with a shared session/rate-limit store. This conflicts with the first-release process-local session contract.
- SQLite optimistic revision columns. This changes latest-valid-write-wins into explicit stale-edit conflicts.
- SQL monetary checks and aggregates. They duplicate or weaken exact Rust `Decimal` rules.
- Global session eviction. It would allow anonymous churn to log out the administrator.
- Generic mutation request timeouts with no idempotency keys. A timeout could occur after commit and make a retry unsafe.
- Frankfurter availability as a startup or readiness dependency. CRUD and local startup must remain available during provider outages.
