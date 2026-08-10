---
project_name: 'debtor'
user_name: 'sebr'
date: '2026-08-08'
sections_completed: ['technology_stack', 'language_rules', 'framework_rules', 'testing_rules', 'quality_rules', 'workflow_rules', 'anti_patterns']
existing_patterns_found: 18
status: 'complete'
rule_count: 67
optimized_for_llm: true
---

# Project Context for AI Agents

_This file contains critical rules and patterns that AI agents must follow when implementing code in this project. Focus on unobvious details that agents might otherwise miss._

---

## Technology Stack & Versions

- Rust 1.97.1 toolchain; edition 2024, MSRV 1.97, Cargo resolver 3.
- Axum 0.8.9; Askama 0.16.0; Tokio 1.53.1; Tower 0.5.3; tower-http 0.6.11; tower-sessions 0.15.0.
- SQLx/sqlx-cli 0.9.0 with bundled SQLite and committed offline query metadata.
- reqwest 0.13.4 with rustls; rust_decimal 1.42.1; chrono 0.4.45.
- Argon2 0.5.3; thiserror 2.0.20; anyhow 1.0.104; serde 1.0.229; serde_json 1.0.151; UUID 1.24.0.
- The production workspace and `tools/password-hash` are independent Cargo workspaces; validate each separately.
- Preserve `Cargo.lock`, use `--locked`, and consult current crate documentation before changing framework/library APIs.

## Critical Implementation Rules

### Language-Specific Rules

- Use `rust_decimal::Decimal` for all money and rates; never introduce `f32`/`f64`, float arithmetic, or lossy numeric conversion.
- Application commands parse raw amount/code/date text and construct financial values; transport adapters only decode field structure and preserve submitted text.
- Treat currency precision as validation, not rounding: JPY/KRW use 0 minor units, OMR 3, all others 2.
- Persist money as canonical decimal `TEXT`; repository decoding must revalidate canonical form and reject malformed stored values rather than normalize them.
- Use checked domain errors for arithmetic, quantization, and settlement; never panic, default a failed conversion to zero, or return partial results.
- Keep domain code synchronous and deterministic. Use explicit sorting or `BTreeMap`/`BTreeSet`; participant ID is the tie-breaker.
- Ledger entity IDs are positive `i64`; UUIDs are only for session/CSRF randomness.
- Parse dates strictly as `%Y-%m-%d` `NaiveDate`, reject dates before `2025-01-01`, and use UTC for current calculations/defaults.
- Use `thiserror` for typed domain/application/adapter errors. Keep `anyhow` in root process/config/runtime orchestration.
- Never let SQLx, reqwest, Axum, Argon2, session, or other outer-layer types cross application-owned ports.
- Avoid `unwrap`/`expect` in production paths; workspace lints warn on both, and unsafe Rust is forbidden.

### Framework-Specific Rules

- Preserve `root -> debtor-web/debtor-infra -> debtor-application -> debtor-domain`: domain owns pure rules; application owns use cases, input policy, and ports; infra owns concrete adapters; web owns HTTP; root owns configuration, composition, migrations, lifecycle, and startup.
- Keep handlers thin and inject external effects through narrow application traits. Concrete adapters appear only in root; use cases must run with in-memory fakes and injected clocks.
- Render semantic HTML with Askama and vanilla CSS. Do not add custom JavaScript; core behavior must remain server-rendered whether or not HTMX is used.
- Support current stable Chrome, Firefox, Safari, and Edge down to 320 CSS pixels. Controls are pointer-independent with labeled two-CSS-pixel focus indicators at 3:1 contrast; normal text reaches 4.5:1, large text/components/meaningful graphics reach 3:1, and inline errors are programmatically associated.
- Use the shared strict form/CSRF extractor. Reject malformed, missing, duplicate, or unknown fields before dispatch; rerender validation with `422` and submitted values retained; redirect successful mutations with `303`.
- Return `409` for archived mutation/form routes before invoking a use case.
- Give mutations a bounded pre-dispatch deadline, mark dispatch immediately before the first state-changing use-case call, and never apply a generic timeout after dispatch.
- Probe and static routes must not create or load sessions.
- Use checked SQLx macros and refresh committed `.sqlx` metadata after SQL/migration changes. The fixed WAL-checkpoint PRAGMA is the sole verified unchecked-query exception.
- Serialize ledger mutations through the shared five-second write gate; keep eligibility checks and aggregate writes in one transaction.
- Load complete spending/debt aggregates from one SQLite snapshot, commit it, then perform network rate requests.
- Convert adapter diagnostics to safe application reasons and sanitized HTTP responses; never expose raw SQLx/provider/cryptography errors.

### Testing Rules

- Test pure financial rules in `debtor-domain` with examples, boundaries, and property tests; assert deterministic ordering and exact conservation.
- Test application use cases without Axum, SQLite, network, or wall clock using injected clocks and simple fakes backed by `Mutex<Vec<_>>`, maps, or atomics; add no mocking framework without concrete need.
- Test concrete infra/web adapters separately, including malformed or bounded external input, safe failure mapping, cache/single-flight and password/session behavior, and persistence decoding. Use `#[sqlx::test]` plus temporary file databases for WAL, locking, multi-connection, and paired migration/constraint checks.
- Make concurrency tests deterministic with barriers, notifications, or deliberately held locks, not timing sleeps; assert rejected/timed-out work starts no guarded side effect and permitted completion-order variation does not change deterministic outputs.
- Web tests use shared fake use cases and verify status, headers, redirects, retained submitted values, CSRF rejection, and no dispatch for pre-use-case rejection.
- Retain a root real-socket smoke test covering authentication/CSRF, an authenticated read, startup ordering, and bounded shutdown.
- Place each regression test in the layer that owns the invariant; add cross-layer tests only for composition or adapter contracts.
- Keep test-only `allow` attributes narrow; do not weaken workspace lint policy for test helpers.

### Code Quality & Style Rules

- Keep `cargo fmt` clean and satisfy workspace Clippy `pedantic` with warnings denied in validation; do not suppress lints broadly.
- Feature modules use plural nouns (`groups`, `participants`, `spendings`, `debts`) and mirror capabilities across layers where useful.
- Name interfaces `*Reader`, `*Repository`, `*Provider`, or `*UseCases`; implementations `*Service`, `*Store`, `*Client`, or `*Gate`.
- Use `*Input` for transport-neutral raw commands, private `Db*` types for persistence rows, and `*Template`/`*Row`/`*View` for rendering projections.
- Document public APIs with rustdoc and include `# Errors` for fallible methods; comments should explain non-obvious constraints, not restate code.
- Prefer the smallest correct change: keep logic local until reuse/composition is real, and avoid speculative helpers or abstractions.
- Before first deployment, prefer clean breaking changes over compatibility shims; remove superseded paths rather than maintaining both.

### Development Workflow Rules

- `specs/design.md` is normative. Update it first for behavior changes, then synchronize affected ADRs, README/config examples, migrations, tests, and SQLx metadata in the same change.
- Validate the production workspace with `cargo fmt --all -- --check`, `cargo check --workspace --all-features --locked`, `SQLX_OFFLINE=true cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`, and `cargo test --workspace --all-features --locked`.
- Run architecture fitness with `cargo run --bin architecture-check --locked`; dependency-policy changes must also pass `cargo deny check`.
- When checked SQL or migrations change, migrate a temporary SQLite database and run `SQLX_OFFLINE=false DATABASE_URL=sqlite:///tmp/debtor-sqlx.db?mode=rwc cargo sqlx prepare --workspace --check`; commit refreshed `.sqlx` metadata.
- Validate `tools/password-hash` separately with its manifest-path formatting, Clippy, and test commands; it is intentionally outside the production workspace.
- Never use `cargo build --release` for routine checks, tests, or validation; use debug `cargo check`, `cargo test`, and `cargo run`.
- Local startup requires a valid `APP_ADMIN_PASSWORD_HASH`; generate it with the independent helper and keep secrets out of commands, logs, fixtures, and committed files.

### Critical Don't-Miss Rules

- Debtor is permanently single-administrator. Participants are accounting identities, never users; do not add registration, usernames, tenants, participant authentication, or multi-user authorization.
- All totals and persisted payer/share amounts are positive, precision-valid, and at most `999_999_999_999`; payer totals and share totals must each equal the spending total exactly in source-currency minor units.
- Allocations are nonempty and participant-unique. Equal-split residual minor units go to ascending participant IDs; zero shares are invalid.
- Never aggregate money in SQL. Parse, validate, sum, quantize, and format monetary values in Rust with checked arithmetic.
- Preserve history: every participant belongs to exactly one group; a group with no spendings may be deleted with its unreferenced participants; referenced groups/participants use restrictive deletion; participants are otherwise archived, not independently deleted.
- New allocations require active participants owned by the spending's group. An update may retain an archived participant only in the same existing payer/share role; it may not introduce or change that role.
- Archived groups are readable but entirely mutation-disabled. Historical details resolve current participant names and remain available for inactive/archived identities.
- Spending history uses fixed 25-item keyset pagination ordered by `(spent_date DESC, id DESC)`; detail/edit/delete load one complete aggregate directly rather than all history.
- Decode exchange-rate JSON numbers lexically into arbitrary-precision `Decimal`. Preserve context keys `(source, target, requested date, effective date)`, bounded deterministic LRU caches, per-key single-flight, and global/request-level concurrency limits.
- Historical rates default per spending date; current mode uses the UTC calculation date; future historical dates use current rates and are provisional. Stale fallback must match context: fixed past-date quotes have no age limit, while current/future quotes expire after seven UTC calendar days. Without an eligible quote, debts returns retryable `503`; monthly source-currency summaries and CRUD remain available while only converted summaries become retryable unavailable.
- Quantize final balances with largest signed remainder and participant-ID tie-breaking to preserve exact zero sum. Settlement is deterministic greedy, positive, pair-unique, complete, and at most `n - 1`, not globally minimal.
- Validate the bounded Argon2id v19 admin hash before database connection/migration. Non-debug builds require secure cookies.
- Every unsafe request, including login, requires exactly one valid session-backed CSRF token. Rotate session ID and CSRF on login; save before redirect; flush on logout; never evict authenticated sessions to satisfy capacity.
- Trust forwarding headers only from configured proxy CIDRs in the selected format. Never log credentials, hashes, cookies, session/CSRF IDs, limiter keys, SQL/database messages, values, identifiers, query strings, or provider URLs.
- Preserve the supported topology: one process and one local WAL SQLite volume, `synchronous=FULL`, five-second busy/write-gate bounds, and no external writers or multiple app instances.
- Keep probe admission separate from user traffic; readiness checks SQLite and mandatory supervisors, never Frankfurter or ledger contents. After mutation dispatch, return a definitive commit/rollback result rather than canceling on a generic timeout.

---

## Usage Guidelines

**For AI Agents:**

- Read this file and `specs/design.md` before implementing code.
- Follow every applicable rule; when uncertain, preserve the stricter accounting, history, security, and layer boundary.
- Update this file when a new unobvious implementation pattern becomes normative.

**For Humans:**

- Keep this file lean and agent-focused; update it when the stack or normative patterns change.
- Periodically remove obsolete or now-obvious guidance.

Last Updated: 2026-08-08
