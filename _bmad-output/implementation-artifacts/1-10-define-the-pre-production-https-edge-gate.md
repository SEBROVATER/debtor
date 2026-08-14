---
story_key: 1-10-define-the-pre-production-https-edge-gate
story_id: 1.10
epic: 1
status: done
created: 2026-08-14
baseline_commit: 62a5f978ba661051a37852f43e8bfa199aa01970
---

# Story 1.10: Define the Pre-Production HTTPS Edge Gate

Status: done

## Story

As the administrator,
I want an approved edge product and verification environment before production rollout,
so that deployable HTTPS configuration can satisfy the fixed transport contract without guessing a vendor.

## Acceptance Criteria

1. **Application scope boundary:** Given Phase 4 application implementation is planned, when its scope is approved, then no vendor-specific edge configuration, backend TLS, backend QUIC, backend UDP listener, certificate automation, or production deployment verification is assigned to an application feature story, and Debtor retains the private HTTP/1.1 backend contract while direct insecure HTTP remains debug/local only.
2. **Edge decision record:** Given pre-production operations planning begins, when an edge product and exact version are selected, then a decision record captures the selected product/version, deployment verification environment, source of truth for edge configuration, rollback/rollout assumptions, and every fixed edge obligation mapped to concrete configuration plus executable evidence, and production rollout cannot proceed without that completed operations gate.
3. **Forwarding sanitation parity:** Given a client request carries forwarding headers, when the edge proxies it to Debtor, then the edge strips untrusted forwarding input or appends its immediate peer while preserving chain order, its backend source CIDR and selected header mode match `APP_TRUSTED_PROXY_CIDRS` and `APP_TRUSTED_PROXY_HEADER`, and Debtor resolves identical client identity and login-limiter behavior over HTTP/3 and TCP fallback.
4. **Early-data replay safety:** Given TLS/QUIC early data is available, when an unsafe request is attempted through early data, then the edge disables early data or returns `425 Too Early`; only explicitly allow-listed, replay-safe, session-free `GET` and `HEAD` routes such as probes or immutable static assets may pass early data, and login, authenticated HTML, or any route that creates, loads, refreshes, or mutates session/token state is excluded. CSRF must never be treated as replay protection for early data.
5. **Edge body limits:** Given login or another form request reaches the edge, when body limits are enforced, then the edge permits at most 8 KiB for `/login` and 256 KiB for other form endpoints, matching or tightening application limits, and oversized input is rejected before backend mutation dispatch.
6. **Backend transport and timeouts:** Given backend transport is configured, when the edge manages connections and timeouts, then it reuses private HTTP/1.1 backend connections, may bound connect and response-header waits only where proven safe, and no edge request, read, write, stream, or response timeout can expire before an admitted post-dispatch mutation reaches definitive completion.
7. **HTTP/3 staged rollout:** Given HTTP/3 is introduced, when rollout begins, then `Alt-Svc` uses a short lifetime until UDP/443 reachability and edge telemetry are verified, and the lifetime is not increased until blocked UDP falls back to HTTP/2 or HTTP/1.1, unsafe early data receives `425` or is disabled, and forwarded client identity matches across every protocol.
8. **Reproducible verification evidence:** Given edge policy is tested or documented as deployable configuration, when the production contract is validated, then forwarding sanitation, protocol fallback, body limits, backend reuse/timeouts, early-data rejection, staged `Alt-Svc` rollout, and safe diagnostics have reproducible verification steps or captured evidence, and no secret, client identity, query string, provider URL, or request-derived value is introduced into application logs.

**Requirements:** `SPEC-FR12`, `SPEC-FR100`, `SPEC-FR103` (pre-dispatch and no-shorter-edge-timeout contract only); `SPEC-NFR1`, `SPEC-NFR3`, `SPEC-NFR23..SPEC-NFR25`, `SPEC-NFR31..SPEC-NFR34`; pre-production edge decision, vendor-specific verification environment, fixed TLS/HTTP3, forwarding, early-data, body-limit, timeout, fallback, and rollout obligations. This is a pre-production operations gate, not a Phase 4 application implementation story. No UX IDs apply because the story defines operator/deployment readiness rather than a rendered Administrator route.

## Scope Boundary

- This story produces a pre-production operations gate and decision record for the production HTTPS edge. It does not implement a new Administrator-facing route.
- Do not add TLS, QUIC, UDP, certificate management, HTTP/3, or `Alt-Svc` behavior to the Rust application. Those remain edge responsibilities.
- Do not add users, tenants, participant authentication, registration, persistent sessions, shared session storage, multiple app instances, external SQLite writers, or deployment-topology abstractions.
- Do not move Axum, Tokio, SQLx, Caddy, proxy, certificate, or runtime types into `debtor-application` or `debtor-domain`.
- Do not mark `SPEC-FR103` or `SPEC-FR104` final evidence complete. Story 2.1 still owns the first real ledger mutation, definitive mutation outcome publication, and shutdown waiting for an active dispatched ledger mutation.
- Do not rely on the existing `deploy/Caddyfile.example` as production-ready evidence unless the story explicitly selects Caddy, pins a version, updates the example to satisfy this story, and records executable verification.
- Do not claim early-data safety by allowing every `GET`/`HEAD`; session-touching safe methods such as `GET /login` and authenticated HTML are excluded unless early data is disabled entirely.

## Tasks / Subtasks

- [x] Record the edge operations decision (AC: 1, 2)
  - [x] Choose and record the edge product, exact version, and verification environment, or halt with a concrete operations blocker rather than silently using the non-normative Caddy example.
  - [x] Add the decision in the repository's decision-record style, preferably `specs/adr/0003-pre-production-edge-gate.md` unless a more specific existing convention is found.
  - [x] If the selected product/version changes the normative edge contract, update `specs/design.md` first and synchronize ADRs, README, config examples, deploy examples, and tests in the same change.
  - [x] Update the architecture deferred-decision entry so reverse-proxy vendor/configuration is no longer ambiguous once selected.

- [x] Produce deployable edge configuration or a checked configuration template (AC: 2-7)
  - [x] Put vendor-specific deployable config or template under `deploy/`, keeping product/version assumptions explicit.
  - [x] Preserve the Rust backend as private HTTP/1.1 TCP on `APP_BIND`; no backend TLS or QUIC listener.
  - [x] Configure forwarding sanitation so the selected mode and backend source CIDR exactly match `.env.example` guidance and application configuration.
  - [x] Configure early-data handling as disabled, or as a narrow explicit allow-list limited to session-free replay-safe routes, with unsafe early data returning `425 Too Early` before Debtor is reached.
  - [x] Configure body limits at or below 8 KiB for `/login` and 256 KiB for other form endpoints.
  - [x] Configure private backend HTTP/1.1 reuse and remove or justify any edge response/read/write/stream timeout that could cut off future admitted post-dispatch mutations.
  - [x] Configure short initial `Alt-Svc`/HTTP3 rollout behavior if the selected edge exposes it directly; otherwise document the product-managed behavior and the verification commands that prove it.

- [x] Update operator documentation and examples (AC: 1-8)
  - [x] Update `README.md#Edge Deployment` with the selected edge product/version, the verification environment, and the required go/no-go checks.
  - [x] Update `.env.example` only if wording is needed to align selected proxy CIDR/header mode or verification steps; do not introduce multiple header modes or production direct-peer fallback.
  - [x] Update `deploy/Caddyfile.example` if Caddy remains as an example or becomes selected; keep its non-normative label unless the decision record promotes it.
  - [x] If Caddy is selected or retained as a primary example, explicitly address Caddy's 0-RTT behavior, all forwarding headers, body limits, backend transport, and unsafe timeout risks.

- [x] Add reproducible verification evidence (AC: 3-8)
  - [x] Document or script how to verify short `Alt-Svc`, UDP/443 reachability, and HTTP/2 or HTTP/1.1 fallback when UDP is blocked.
  - [x] Document or script how to send spoofed forwarding headers and prove Debtor sees the same trusted client identity and limiter budget over HTTP/3 and fallback.
  - [x] Document or script how to send marked early-data unsafe requests and prove `425` or disabled early data without backend dispatch.
  - [x] Document or script how to prove `/login` and other form body limits reject oversized input before backend mutation dispatch.
  - [x] Document or script how to inspect edge telemetry for backend HTTP/1.1 connection reuse and prove no configured edge timeout cuts off a deliberately slow admitted mutation before final response. If no real mutation exists yet, record the check as a Story 2.1 follow-up gate rather than pretending it passed.
  - [x] Capture or specify application-log checks that exclude credentials, hashes, cookies, session/CSRF/submission tokens, client identity, forwarding chains, query strings, provider URLs, SQL/database messages, raw adapter diagnostics, and request-derived values.

- [x] Validate changed artifacts (AC: 1-8)
  - [x] If only Markdown, `.env.example`, or deploy templates change, validate by reviewing links, commands, product/version references, config syntax, and story acceptance mapping.
  - [x] If Rust configuration, proxy parsing, router body limits, middleware, runtime, or tests change, also run `cargo fmt --all -- --check`, `cargo check --workspace --all-features --locked`, `SQLX_OFFLINE=true cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`, `cargo test --workspace --all-features --locked`, and `cargo run --bin architecture-check --locked`.
  - [x] Run `cargo deny check` only if dependency manifests, lockfiles, or dependency policy change.
  - [x] Run online SQLx prepare only if checked SQL or migrations change; no SQL or migration change is expected.
  - [x] Never use `cargo build --release` for validation.

### Review Findings

- [x] [Review][Patch] Remove or prove the post-dispatch response-header timeout [deploy/Caddyfile.example:74-80] — removed the response-header timeout; only connection setup is bounded, and real slow-mutation evidence remains a Story 2.1 gate.
- [x] [Review][Patch] Sanitize all forwarding and proxy-identity headers [deploy/Caddyfile.example:62-73] — expanded deletion coverage to forwarding and common proxy-identity headers and added those headers to the spoofed verification set.
- [x] [Review][Patch] Make forwarding-parity verification submit real failed logins [deploy/edge-verification.md:63-128] — added a fresh session/CSRF/submission-token flow and controlled wrong-password submissions over HTTP/3 and fallback.
- [x] [Review][Patch] Pin every Caddy verification command [deploy/edge-verification.md:17-56,177-193] — Caddy validation, adaptation, and run commands now use the pinned `caddy:2.11.2` image.
- [x] [Review][Patch] Make blocked-UDP fallback verification reproducible [deploy/edge-verification.md:148-177] — added a Linux `iptables` block/restore procedure and verbose HTTP/2 protocol assertion.
- [x] [Review][Patch] Add observable backend connection-reuse evidence [deploy/edge-verification.md:50-61,177-193] — added repeated requests and `ss` established-connection capture for the private backend.
- [x] [Review][Patch] Turn safe-diagnostics verification into an executable assertion [deploy/edge-verification.md:195-215] — added stderr capture and sentinel absence assertions.
- [x] [Review][Patch] Document the required HTTPS/HTTP/3 verification environment [deploy/edge-verification.md:5-18] — documented DNS/hosts, trusted test certificate, firewall, and private backend prerequisites.
- [x] [Review][Patch] Prevent the backend variable from enabling upstream TLS [deploy/Caddyfile.example:8-16,65] — removed backend interpolation and require operators to replace only a literal host:port value.

## Dev Notes

### Developer Context

Story 1.10 is the last Epic 1 backlog item and exists to close the deployment-substrate ambiguity before production rollout. It should not become a backend networking feature. The current application already implements the private HTTP backend, trusted proxy configuration, session-safe request pipeline, body limits, safe diagnostics, and bounded runtime behavior that the edge must preserve.

The current repository already contains an edge section in `README.md` and a non-normative `deploy/Caddyfile.example`. Those are useful starting points, but not sufficient completion evidence. The story requires an approved product/version, verification environment, concrete configuration, and executable/captured verification. Treat the current Caddyfile as an example to fix or replace, not proof that the gate is closed.

### Architecture Compliance

- Preserve dependency direction `root -> debtor-web/debtor-infra -> debtor-application -> debtor-domain`.
- Follow AD-12: production is one Debtor process with one local WAL SQLite volume behind one sanitizing HTTPS reverse proxy. Debtor serves private HTTP/1.1 TCP; the edge owns TLS, certificates, HTTP/2, HTTP/3/QUIC, `Alt-Svc`, fallback, forwarding sanitation, backend reuse, body limits, early data, and mutation-compatible timeouts.
- Follow AD-14: Login bodies are 8 KiB, other form bodies are 256 KiB, user/login/probe admission remains bounded, and no timeout after dispatch may cancel a mutation. The edge must not impose a shorter post-dispatch timeout.
- Follow AD-15: logs and diagnostics use fixed categories and must not contain credentials, hashes, cookies, session/CSRF/submission tokens, limiter keys, client IPs, query strings, provider URLs, SQL/database messages, monetary values, entity identifiers, raw errors, or request-derived data.
- Follow AD-16: keep verification in the owning layer. Edge config/evidence belongs to operations/deploy docs; proxy parser tests stay in `debtor-web`; root real-socket/runtime tests stay in root if Rust behavior changes.
- Follow AD-17: no multi-user, tenant, participant auth, registration, or new authorization abstractions.
- Follow AD-18 only if rendered web artifacts are touched. No UX IDs apply to this story as written.

### Current Files To Update And Preserve

| Path | Current state | Required change or preservation |
| --- | --- | --- |
| `specs/design.md` | Normative source already contains the Edge Proxy Contract: edge owns TLS/HTTP3/Alt-Svc/fallback, forwarding sanitation, early-data rejection, body limits, backend reuse/timeouts, and staged rollout. | Update first only if selected product/version changes behavior or narrows the contract. Otherwise cite and preserve it. |
| `specs/adr/0001-foundation-architecture.md` | Records supported topology, local readiness/shutdown, native HTMX policy, and reverse proxy responsibilities. | Preserve unless a new ADR must cross-reference it. Later ADRs must identify any supersession. |
| `specs/adr/0002-long-term-foundation-hardening.md` | Keeps HTTP/3/QUIC at the reverse proxy and application as private HTTP/1.1 backend. | Preserve unless adding a new ADR that records the pre-production edge gate. |
| `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md` | AD-12 and deferred table currently defer reverse-proxy vendor and vendor-specific configuration. | If planning artifacts are maintained as handoff docs, update deferred entry after selecting product/version. Do not silently diverge from `specs/design.md`. |
| `README.md` | Has Edge Deployment guidance and a link to the non-normative Caddy example. | Update with selected edge product/version, verification environment, gate status, and reproducible checks. Preserve private HTTP/1.1 backend and no-shorter-timeout language. |
| `.env.example` | Documents trusted proxy CIDRs/header, direct-peer debug behavior, and cross-protocol parity. | Preserve exact single selected header mode and production nonempty CIDR requirement. Update only for clearer selected-product guidance; do not add multiple modes or direct production fallback. |
| `deploy/Caddyfile.example` | Non-normative Caddy sample strips `X-Forwarded-For`, sets `X-Forwarded-For {remote_host}`, enforces form sizes, responds `425` for non-GET/HEAD early data, and sets HTTP/1.1 transport with `response_header_timeout 95s`. | If Caddy is selected or retained as example, review against current Caddy docs. Disable 0-RTT or explicitly allow-list only session-free routes; strip untrusted `Forwarded` too; remove or justify timeouts that could cut off future mutations. Keep non-normative label unless promoted by ADR. |
| `src/config.rs` | Reads `APP_TRUSTED_PROXY_CIDRS`/`APP_TRUSTED_PROXY_HEADER`; non-debug requires nonempty CIDRs; invalid config is safe. | No change expected. If changed, maintain pre-DB/pre-socket failure and safe diagnostics. |
| `src/composition.rs` | Parses `TrustedProxyConfig` before database connection/migration and injects it into web state. | No change expected. Preserve root-only concrete composition and early invalid-proxy failure. |
| `debtor-web/src/state.rs` | `TrustedProxyConfig` supports exactly `forwarded` or `x-forwarded-for`; untrusted peers ignore forwarding; trusted selected headers are parsed right-to-left; tests cover malformed headers and canonicalization. | No change expected. Preserve exact one selected mode and no raw forwarding/client diagnostics. |
| `debtor-web/src/handlers/auth.rs` | Login resolves trusted client from `ConnectInfo` and headers before limiter/password verification. | No change expected. Preserve safe malformed-forwarding response and no client identity logging. |
| `debtor-web/src/router.rs` | Enforces 8 KiB login and 256 KiB protected body limits; tests prove pre-handler rejection. | No change expected. Edge must mirror or tighten these limits. |
| `debtor-web/src/middleware.rs` | Logs safe route patterns/status/latency, enforces timeout classes, and avoids post-dispatch mutation cancellation. | No change expected. Preserve no raw URI/query/body and no generic post-dispatch timeout. |
| `src/runtime.rs` | Runs Axum over a TCP listener; closes admission, drains HTTP, waits for mutation registry empty, checkpoints WAL, and closes pool. | No edge transport change expected. Do not add TLS/QUIC/HTTP3. Story 2.1 owns real mutation outcomes. |

Expected unchanged areas: `debtor-domain`, `debtor-application` financial use cases, `debtor-infra` rate/persistence logic, migrations, `.sqlx`, templates, CSS, and Cargo manifests unless the story intentionally changes validation tooling or dependency policy.

### Edge Verification Matrix

| Obligation | Required evidence |
| --- | --- |
| Product/version selected | ADR or operations decision states product, version, config artifact path, verification environment, and rollout/rollback assumptions. |
| Private backend | Config points to Debtor over private HTTP/1.1 only; no backend TLS/QUIC/UDP requirement is introduced. |
| Forwarding sanitation | Spoofed client-supplied forwarding values are stripped or safely appended; selected header mode matches `APP_TRUSTED_PROXY_HEADER`; backend source CIDR matches `APP_TRUSTED_PROXY_CIDRS`. |
| Cross-protocol identity | Controlled failed login attempts over HTTP/3 and HTTP/2 or HTTP/1.1 fallback consume one limiter budget for the same resolved client identity without logging it. |
| Early data | 0-RTT is disabled, or unsafe marked early-data requests receive `425` before backend forwarding; `GET /login` and authenticated pages are not allowed merely because they are `GET`. |
| Body limits | Payloads over 8 KiB for `/login` and over 256 KiB for other forms are rejected at or before the edge and do not dispatch mutations. |
| Backend reuse | Edge telemetry or config proof shows private backend HTTP/1.1 reuse and no unintended backend TLS/HTTP2/HTTP3. |
| Timeout safety | Any connect/response-header timeout is documented as pre-dispatch safe; no read/write/request/stream timeout can terminate a future admitted post-dispatch mutation before final response. |
| Alt-Svc rollout | Initial short lifetime is verified; longer lifetime is blocked until UDP reachability, fallback, early-data, telemetry, and identity parity checks pass. |
| Safe diagnostics | Captured application logs exclude secret/client/forwarding/query/provider/request-derived values. |

### Library / Framework Requirements

- Current project stack remains pinned: Rust 1.97.1/edition 2024, Axum 0.8.9, Tokio 1.53.1, Tower 0.5.3, tower-http 0.6.11, SQLx 0.9.0, tower-sessions 0.15.0. Do not upgrade or add framework dependencies for this story.
- Current Caddy documentation was consulted for the existing sample only. Caddy Caddyfile docs expose `request_body max_size`, `reverse_proxy` `header_up`, and `transport http` settings including `dial_timeout`, `response_header_timeout`, `read_timeout`, `write_timeout`, keepalive, and `versions`. Caddy docs also document global server `0rtt off`, trusted proxies, and strict proxy parsing.
- If Caddy is selected, prefer disabling 0-RTT with the documented global server option unless there is executable evidence for a narrower session-free allow-list. Be careful that Caddy defaults and upstream behavior may pass headers or manage `Alt-Svc`; document the actual selected version behavior rather than assuming the example is complete.
- If another edge product is selected, consult that product's current official documentation before writing config or verification instructions. Record the product/version and doc source in the decision record.

### Previous Story Intelligence

Story 1.9 completed the authenticated runtime shutdown foundation. Preserve these learnings:

- The Rust process binds a private local TCP listener and logs an `http://` URL; that is correct for local/debug and for a private backend behind a production edge.
- There is now one root-owned mutation registry/barrier, but it is not the real mutation executor and does not publish `Committed`, `RolledBack`, or `Unknown` outcomes. Do not fabricate final mutation evidence in this story.
- Keep fixed 64 user, four login, and four probe budgets; body limits and timeout classes are already application-owned.
- Safe reads may be forced down after drain; unsafe post-dispatch mutations must not be generically cancelled.
- Tests use barriers, `Notify`, held resources, and explicit observations rather than sleeps. If Rust verification changes, preserve that style.

### Git Intelligence

- Current `HEAD`: `62a5f97` (`feat: impelement bmad 1-9`). It changed runtime/composition/web lifecycle paths and updated `.sqlx` metadata for test SQL. Build on this, do not reconstruct runtime behavior from older prose.
- `ff80a9c` completed Story 1.8 admission/readiness/probe foundations. Preserve separate probe capacity and session-free probe/static routes.
- `b3b3628` completed Story 1.7 shared authenticated submission-token boundary. Do not create an edge or route-level replay system that claims to replace tokens; early-data safety is separate.
- `fdaca09` and prior commits established authenticated sessions, logout, trusted proxy validation, local startup, SQLite composition, and restart coverage.

### Testing Requirements

- For docs/config-only implementation, the decisive tests are executable edge verification commands, config validation, and captured evidence. Do not claim a check passed if the selected edge product or verification environment was unavailable.
- If Caddy is selected, run the selected version's config validation command against the committed Caddyfile/template and record the command/output in completion notes. Use the selected version, not an unspecified system install.
- Verify forwarding sanitation with spoofed `X-Forwarded-For` and `Forwarded` inputs. If the selected application mode is `x-forwarded-for`, the edge should still strip or neutralize untrusted `Forwarded` so downstream tools do not ingest attacker input.
- Verify early data by disabling 0-RTT or by a marked early-data test that proves unsafe routes get `425` before backend dispatch. Do not count ordinary non-early-data `POST` rejection as evidence.
- Verify body limits at both edge and app boundary if possible; the app already has tests for `413` before handlers.
- Verify login limiter parity across HTTP/3 and fallback without logging the client identity or limiter key. Use controlled failed attempts and safe status checks.
- Verify no application logs contain forbidden values. Prefer sentinel strings that are not real secrets and assert absence in captured logs.
- If Rust behavior changes, place tests in the owner layer named above and run the full workspace validation commands. No SQLx metadata update is expected unless checked SQL or migrations change.

### References

- [Source: `_bmad-output/planning-artifacts/epics.md#Story 1.10: Define the Pre-Production HTTPS Edge Gate`]
- [Source: `_bmad-output/planning-artifacts/epics.md#Final-Evidence Ledger`]
- [Source: `specs/design.md#Edge Proxy Contract`]
- [Source: `specs/design.md#Operational Limits`]
- [Source: `specs/adr/0001-foundation-architecture.md#2. Supported deployment topology`]
- [Source: `specs/adr/0001-foundation-architecture.md#10. Fixed HTTP resource budgets`]
- [Source: `specs/adr/0001-foundation-architecture.md#11. Local readiness and shutdown`]
- [Source: `specs/adr/0002-long-term-foundation-hardening.md#Decisions`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-12 - Single-process edge topology`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-14 - Bounded operational envelope`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-15 - Safe failures, logging, and readiness`]
- [Source: `_bmad-output/planning-artifacts/prds/prd-debtor-2026-08-10/addendum.md#Edge Transport And Rollout`]
- [Source: `README.md#Edge Deployment`]
- [Source: `.env.example`]
- [Source: `deploy/Caddyfile.example`]
- [Source: `src/config.rs`]
- [Source: `src/main.rs`]
- [Source: `src/runtime.rs`]
- [Source: `src/composition.rs`]
- [Source: `debtor-web/src/state.rs`]
- [Source: `debtor-web/src/router.rs`]
- [Source: `debtor-web/src/middleware.rs`]
- [Source: Context7 `/websites/caddyserver` Caddy Caddyfile server options for `0rtt`, trusted proxies, and strict proxy parsing]
- [Source: Context7 `/websites/caddyserver_caddyfile` Caddy `reverse_proxy`, `request_body`, matchers, `respond`, and `header_up` documentation]

## Dev Agent Record

### Agent Model Used

openai/gpt-5.5

### Debug Log References

- Ultimate context engine analysis completed - comprehensive developer guide created.
- Selected the first backlog story in complete sprint order: `1-10-define-the-pre-production-https-edge-gate`.
- Loaded project context, normative design contract, Epic 1, PRD/addendum, architecture spine, UX contracts, ADRs, Story 1.9, current edge docs/config/code files, recent commits, and current Caddy documentation through Context7.
- Open clarifications: none. Edge product/version selection is the story's implementation deliverable; if approval cannot be completed from available operations context, record a blocker rather than inventing completion.
- Captured baseline commit `62a5f978ba661051a37852f43e8bfa199aa01970` and moved sprint tracking from `ready-for-dev` to `in-progress` before implementation.
- Consulted current Caddy documentation via Context7 for Caddyfile `request_body`, `reverse_proxy header_up`, HTTP transport settings, global `0rtt off`, `strict_sni_host`, and `caddy validate` behavior.
- Selected Caddy `2.11.2` after confirming the local exact binary with `caddy version`; Docker was not available in this environment.
- Validated the Caddy template with `caddy validate --config deploy/Caddyfile.example --adapter caddyfile`; validation passed with non-fatal warnings for intentional explicit forwarding header setting after stripping attacker input.
- Ran `git diff --check`, required artifact existence checks, `cargo fmt --all -- --check`, `cargo check --workspace --all-features --locked`, `SQLX_OFFLINE=true cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`, `cargo test --workspace --all-features --locked`, and `cargo run --bin architecture-check --locked`.

### Completion Notes List

- Ultimate context engine analysis completed - comprehensive developer guide created.
- Story scope is limited to pre-production edge gate definition and verification. Backend TLS/QUIC/HTTP3 and real ledger mutation final evidence are explicitly out of scope.
- Added ADR 0003 selecting Caddy `2.11.2`, defining the pre-production verification environment, source-of-truth edge template, rollout/rollback assumptions, fixed edge obligations, and evidence requirements.
- Promoted `deploy/Caddyfile.example` into the selected Caddy 2.11.2 pre-production template with global 0-RTT disabled, strict SNI host validation, explicit short `Alt-Svc` lifetime, forwarding-header sanitation, body limits, private HTTP/1.1 upstream reuse, and mutation-compatible timeout constraints.
- Added `deploy/edge-verification.md` with reproducible checks for version/config validation, private backend reuse, forwarding parity, early-data rejection, body limits, HTTP/3 fallback, timeout safety, and safe diagnostics.
- Updated README and `.env.example` operator guidance to align the selected Caddy template with `APP_TRUSTED_PROXY_CIDRS` and `APP_TRUSTED_PROXY_HEADER=x-forwarded-for` without adding production direct-peer fallback or multiple selected modes.
- Updated the architecture spine deferred table so reverse-proxy vendor/configuration is no longer ambiguous for pre-production.
- No Rust behavior, dependencies, SQL, migrations, or SQLx metadata changed. Full workspace validation still passed.
- Applied all nine code-review patches: removed the unsafe response-header timeout, expanded forwarding sanitation, made login/fallback/log checks executable, pinned all Caddy commands, documented environment prerequisites, added live connection evidence, and removed backend URL interpolation.
- Revalidated the revised Caddy template and adapted transport with Caddy `2.11.2`; full workspace regression tests passed again.

### File List

- `.env.example`
- `README.md`
- `_bmad-output/implementation-artifacts/1-10-define-the-pre-production-https-edge-gate.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md`
- `deploy/Caddyfile.example`
- `deploy/edge-verification.md`
- `specs/adr/0003-pre-production-edge-gate.md`

### Change Log

- 2026-08-14: Implemented Story 1.10 pre-production HTTPS edge gate with Caddy 2.11.2 ADR, checked edge template, verification runbook, operator documentation, config-example alignment, and architecture deferred-decision update.
- 2026-08-14: Addressed all nine code-review findings; story moved to done.
