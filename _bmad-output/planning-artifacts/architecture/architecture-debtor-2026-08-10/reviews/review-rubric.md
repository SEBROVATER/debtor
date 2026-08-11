# Architecture Spine Rubric Review

## Verdict

**FAIL - major revision required before this can serve as the project-wide build substrate.**

The spine captures many of the project's hardest accounting, concurrency, layering, and runtime invariants accurately. It does not, however, meet the supplied good-spine rubric. Several first-release capabilities and cross-epic contracts are absent, several deferred items leave shared interfaces undecided, the brownfield scaffold is not explicitly ratified or superseded, and deployment/environment/provider/operations strategy is incomplete. The frontmatter also calls the document a draft with no bindings while every rule is marked adopted.

## Review Basis

Reviewed against:

- `ARCHITECTURE-SPINE.md`
- `specs/design.md`
- Accepted `specs/adr/0001-foundation-architecture.md`
- Accepted `specs/adr/0002-long-term-foundation-hardening.md`
- `_bmad-output/project-context.md`
- Workspace manifests, lockfile, toolchain, migrations, root composition/configuration, architecture fitness check, web routes/templates, application spending APIs, infrastructure provider implementation, and deployment example

Current-version verification performed on 2026-08-10:

- `cargo search` reports Axum `0.8.9`, Askama `0.16.0`, Tokio `1.53.1`, SQLx `0.9.0`, reqwest `0.13.4`, and rust_decimal `1.42.1` as current.
- `rustc --version` reports `1.97.1`; `cargo sqlx --version` reports `0.9.0`.
- npm reports HTMX `2.0.10`; the official HTMX response-targets page specifies HTMX `2.0.10` and response-targets `2.0.4`.
- These checks establish that the versions named in the spine are current on the review date, but not that all are repository-enforced.

## Findings

### 1. Brownfield reality is neither ratified nor explicitly superseded

The spine's only treatment of the existing implementation is “Migration sequencing from obsolete scaffold” (`ARCHITECTURE-SPINE.md:247`). That does not identify what is authoritative now, what must be removed, or which existing interfaces epics must not build upon.

The mismatch is material and distributed across layers:

- `README.md:7` describes memberships, multiple payers, and Equal/Exact expense input.
- `debtor-application/src/spendings.rs:105-140` exposes multiple-payer and Equal-share command variants.
- `debtor-web/templates/group.html:35-225` implements reusable membership management, an inline expense form, multiple-payer mode, and Equal mode.
- `debtor-web/src/router.rs:77-118` exposes membership and global participant-management routes.
- Existing migrations model `group_members` and `spending_payers`, including plural payer rows.
- `src/composition.rs` ratifies a working Frankfurter adapter, current repositories, session store, and route composition, but the spine does not classify these as retained or superseded.

AD-4 and AD-5 state the target identity and allocation rules, but they do not define the migration boundary through the existing public Rust APIs, routes, schema, and tests. Different epics could remove, adapt, or preserve different portions of the old model and each claim compliance.

Required correction: add an explicit brownfield disposition inventory covering at least memberships/reusable participants, payer collections, Equal mode, global participant routes, inline spending flow, current migrations, current repository APIs, Frankfurter, session/runtime infrastructure, and architecture fitness. Mark each retained, refactored, replaced, or removed, and identify the target owner where replacement crosses epics.

### 2. The document has no effective binding status

Frontmatter says `status: draft` and `binds: []` (`ARCHITECTURE-SPINE.md:8-17`), while the scope says it governs independently implemented epics and every rule is marked `[ADOPTED]`. This is internally inconsistent. A draft that binds nothing cannot be the enforceable substrate claimed by the rubric, and epics can reasonably treat it as advisory.

Required correction: either make the spine accepted and bind the relevant epic/spec set, or remove the adopted labels and keep it explicitly non-binding until acceptance. Define precedence between this spine, `specs/design.md`, and accepted ADRs rather than relying only on a “change authority” convention.

### 3. Deferring the concrete schema leaves cross-epic contracts open

“Concrete schema and table shape” is deferred to a persistence epic (`ARCHITECTURE-SPINE.md:245`). The fixed invariants do not resolve shared schema decisions that group, participant, spending, history, archive/restore, summary, debt, and migration work all consume: identity ownership representation, archive fields, foreign-key paths, aggregate row shape, indexes, deletion restrictions, and how a single payer is represented.

This is especially unsafe in a brownfield system whose current schema encodes the superseded reusable-membership and plural-payer model. A persistence epic cannot choose these details independently after feature epics have chosen ports, entities, queries, and route behavior.

Required correction: fix a minimal target logical data model and aggregate boundaries in the spine or a binding companion. Physical index names and migration statement order may remain deferred; entity ownership, cardinalities, archive state, required constraints, transaction boundaries, and aggregate read/write shapes may not.

### 4. Deferring routes and template layout leaves web epics free to diverge

The route inventory and template layout are deferred wholesale (`ARCHITECTURE-SPINE.md:246`). Native-first HTMX makes stable URLs, full-page versus fragment behavior, status-region targeting, redirect destinations, contextual navigation, and form ownership architectural contracts, not local presentation details.

The current scaffold already has routes that conflict with the target product, including global participant management and an inline spending form. Group, participant, spending, summary, and archive epics can therefore produce incompatible URLs, templates, redirect targets, and HTMX/native behavior while satisfying AD-11 in isolation.

Required correction: bind a route/resource contract and page/fragment ownership map. Exact source file paths and CSS layout can remain deferred, but canonical route families, native fallback routes, mutation/form routes, success redirects, stable HTMX targets, archived contexts, and page-section ownership must be decided before independent epics.

### 5. The required monthly summary capability is not architecturally specified

The capability map names monthly summaries (`ARCHITECTURE-SPINE.md:232`), but AD-9 only says that source-currency totals survive conversion failure. It omits the core behavior from `specs/design.md:63`:

- Current UTC calendar month only.
- Group total and per-payer totals.
- Grouping by original source currency.
- Conversion into group currency using each spending date's historical rate.
- Separation from all-time debts.

“Domain Rust aggregation” in the map is a location, not a rule. Summary and debt epics can choose different date windows, grouping dimensions, rate contexts, and totals.

Required correction: add a binding summary rule that fixes the time boundary, dimensions, conversion basis, exact aggregation owner, degradation behavior, and distinction from all-time debt calculation.

### 6. Spending and group workflow requirements are largely absent

AD-5 fixes financial modes, but not the first-release interaction contract in `specs/design.md:60-66`. Missing decisions include:

- One participant allocation table shared by payer and share editing.
- Initial payer/share selection and amount behavior.
- Proportional and Exact form defaults and visible allocation/difference feedback.
- Edit always reopening in Exact because modes and weights are not persisted.
- Focused full-page Add Spending flow, native Preview, and return to Transactions with the committed row visible.
- Group creation requiring only name, defaulting currency to USD, and opening Manage; established groups opening Summary.
- Active versus contextual archived lists.
- No global participant-management surface.
- Current participant-name resolution in historical detail.
- Server-suggested varied participant colors with submitted color retention.

These are divergence points between group, participant, spending, and web-shell epics. AD-11's generic native HTML rule does not decide them.

Required correction: add a page/workflow contract or binding UX companion covering these states, defaults, transitions, and ownership boundaries.

### 7. Rate semantics are incomplete, and Deferred refers to rules that do not exist

AD-9 covers exact decoding, concurrency, broad context keys, determinism, and failure shape, but omits material rules from `specs/design.md:73-83`:

- Historical is the default mode and uses each spending date.
- Current mode uses the UTC calculation date and is not persisted.
- Future historical dates use the latest current rate and are provisional.
- Past historical contexts are stable for the process lifetime; current/future contexts roll over by UTC date.
- A matching past quote has no stale age limit; current/future stale fallback expires after seven UTC calendar days.
- The debt view discloses mode, calculation time, target currency, unique rates, and stale/provisional warnings.

The Deferred table says cache “expiry” and “stale eligibility” remain binding (`ARCHITECTURE-SPINE.md:251`), but no spine rule actually defines those semantics. This is a false boundary: the implementation is deferred while the alleged fixed behavior is absent.

Required correction: move the complete mode, rollover, stale-eligibility, context-matching, and disclosure semantics into AD-9 or bind an exact companion section.

### 8. Cookie and trusted-proxy security contracts are missing

The spine specifies session durations/capacities and edge sanitation but omits security properties fixed by `specs/design.md:87-93` and already implemented in `src/config.rs:28-34`, `src/composition.rs:110-117`, and `debtor-web/src/state.rs`:

- HTTP-only cookies.
- `SameSite=Strict`.
- Secure cookies required outside debug builds.
- Forwarding headers accepted only from configured trusted proxy CIDRs.
- Exactly one selected forwarding-header format.
- Right-to-left trusted chain interpretation and direct-peer fallback semantics.

AD-12 only assigns sanitation to the edge. That does not prevent an application epic from trusting arbitrary forwarding input or configuring cookies inconsistently, directly undermining login rate limiting and authentication.

Required correction: add the cookie attributes, environment-dependent secure-cookie rule, and application-side trusted-proxy admission contract to the security/topology rules.

### 9. Environment strategy is not decided, deferred, or open

The spine distinguishes local debug HTTP from the production edge topology only indirectly. It does not classify local development, test, CI, staging/pre-production, and production environments or decide:

- Which production invariants staging must reproduce.
- Secret/config injection and validation ownership by environment.
- Secure-cookie and proxy-trust behavior outside debug.
- Provider endpoint override policy and whether non-production may use stubs.
- Database location/lifecycle and migration policy by environment.
- Whether an edge is required for staging acceptance tests.

This violates the rubric's explicit requirement that the environment dimension be decided, deferred, or open. It also leaves security-sensitive behavior to implementation defaults.

Required correction: add an environment matrix with binding invariants and allowed differences, or explicitly mark unresolved choices open with an owner and decision deadline before dependent epics.

### 10. Provider strategy is generic despite a ratified Frankfurter implementation

The spine repeatedly says “provider” and “exchange-rate provider” without selecting Frankfurter, defining the expected Frankfurter v2 contract, or explicitly making provider choice an open/deferred decision. Brownfield code and configuration already select `FrankfurterClient` and `https://api.frankfurter.dev/v2` (`src/composition.rs:13,44-47`; `src/config.rs:50-51`; `.env.example:22-23`), while the normative design and ADRs name Frankfurter in availability/readiness requirements.

This omission allows a rate epic and an operations epic to choose different provider protocols, endpoint configuration, historical-date behavior, or availability assumptions.

Required correction: either ratify Frankfurter v2 as the first-release provider with an application-owned replaceable port and validated base-URL override, or explicitly declare provider selection open with compatibility criteria and resolve it before rate, config, and operations work starts.

### 11. Deployment packaging and operations are deferred below a safe decision boundary

AD-12 decides runtime topology, and AD-14/15 decide several runtime limits, but “Deployment automation and packaging” and “Telemetry backend and export protocol” are deferred to an unspecified operations implementation (`ARCHITECTURE-SPINE.md:249-250`). No rule or deferred/open entry addresses:

- Deployable artifact/container versus host binary.
- Filesystem user/permissions and durable SQLite/WAL volume ownership.
- Backup, restore, and restore verification.
- Upgrade, migration, rollback, and failed-start recovery procedure.
- Disk-capacity monitoring and database growth response.
- Log collection/retention and mandatory operational signals.
- Alerting expectations for readiness failure, cleanup-supervisor failure, provider degradation, and failed checkpoint/shutdown.
- Release promotion and edge-configuration validation ownership.

These are feature-altitude infrastructure and operations decisions. Deferring them without fixed interfaces, owners, or acceptance conditions permits deployment, persistence, security, and runtime epics to make incompatible assumptions.

Required correction: establish a minimal first-release operations contract. Vendor-specific tooling and telemetry protocol may remain deferred only after artifact, volume, backup/restore, upgrade/rollback, required signals, and ownership boundaries are fixed.

### 12. Accessibility and browser support are not covered

No rule or Deferred/Open item captures `specs/design.md:67`: latest stable Chrome/Firefox/Safari/Edge, 320 CSS-pixel support, pointer-independent operation, programmatic labels, two-pixel visible focus at 3:1 contrast, text/component contrast thresholds, and programmatic error association.

“Semantic HTML” in AD-11 is insufficient and does not prevent separate web epics from producing incompatible or inaccessible controls and fragments.

Required correction: add a web accessibility/compatibility invariant with testable acceptance criteria and make it bind all templates, forms, fragments, and CSS.

### 13. Shared unsafe-form parsing is underspecified

AD-10 requires strict CSRF validation and “route validation,” but it does not preserve the normative strict-form contract from `specs/design.md:91` and project context: reject malformed, duplicate, missing, and unknown fields before dispatch through one shared extractor. The conventions table only fixes selected status codes.

The distinction matters because independently implemented forms can otherwise differ in duplicate-field handling, unknown-field acceptance, CSRF extraction, token reservation timing, and whether malformed requests reach route logic.

The submission token is also described only as “bounded” and “expiring.” Neither a bound/expiry policy nor a single owner/configurability rule is fixed or deferred, so separate session and web epics can create incompatible capacity and cleanup behavior.

Required correction: define the exact shared extraction guarantees, rejection ordering, no-dispatch evidence, submission-token owner, capacity/expiry decision, cleanup behavior, and stable response categories.

### 14. Named versions are current but not all are repository-enforced or evidenced by the spine

The listed versions are current on the review date, which satisfies the freshness half of the rubric. Enforcement and provenance remain weak:

- Rust is pinned by `rust-toolchain.toml` and crate resolutions are locked in `Cargo.lock`.
- Workspace dependency declarations use broad compatible ranges, so exact versions depend on preserving the lockfile rather than the manifest table alone.
- SQLx CLI `0.9.0` is installed locally but no repository mechanism shown by the spine pins the CLI version for every developer/CI environment.
- HTMX and response-targets versions are current in official documentation, but no vendored assets exist in the current tree and the spine defers their integrity/provenance records.
- The spine provides no `verified_on` date or source, so a later epic cannot distinguish an intentional tested pin from a stale version claim.

Required correction: add verification provenance/date, state that `Cargo.lock` is the exact crate authority, pin the SQLx CLI installation mechanism used by CI/development, and require vendored HTMX asset hashes/provenance before web implementation consumes them. Deferring the precise hash values is acceptable only if the acquisition/verification gate is binding.

### 15. Several rules use undefined enforcement terms

The rules are generally stronger than a typical architecture document, but some prevention claims depend on undefined terms: “harmless read composition” (AD-2), “canonical” decimals (AD-3), “ledger write” (AD-6), “eligible rate” (AD-8), “bounded expiring” tokens (AD-10), “mandatory in-process supervisors” (AD-15), and “targeted compile/integration tests” (AD-16).

Some meanings can be reconstructed from `specs/design.md` or current code, but the spine does not consistently bind those definitions. This weakens enforceability and can exclude writes from the epoch, omit supervisors from readiness, or vary token/cache behavior while still claiming literal compliance.

Required correction: define these terms in the spine, reference exact normative sections, or attach explicit fitness/acceptance checks. In particular, enumerate epoch-advancing mutations and readiness-gating supervisors.

## Rubric Assessment

| Rubric criterion | Assessment | Basis |
| --- | --- | --- |
| Fixes real divergence points and misses none | Fail | Strong on accounting/concurrency/layers; misses UI workflow, summary, rate fallback/disclosures, accessibility, environments, provider, and operations. |
| Each Rule is enforceable and prevents stated divergence | Partial | Most numerical/runtime rules are testable; governance status and undefined terms weaken enforcement. |
| Deferred cannot let epics diverge | Fail | Schema, routes/templates, packaging, and telemetry are deferred without sufficient shared contracts. |
| Named tech is verified current | Partial pass | All listed versions checked current on 2026-08-10; SQLx CLI and browser assets are not fully repository-enforced and provenance is absent. |
| Brownfield reality ratified except explicit superseded scaffold | Fail | Existing scaffold conflicts are material but only called “obsolete” without a disposition inventory. |
| All `specs/design.md` capabilities covered | Fail | Monthly summary, workflow/defaults, accessibility, rate disclosures/fallback windows, cookie/proxy details, and contextual UI requirements are incomplete or absent. |
| Every feature-altitude dimension decided/deferred/open | Fail | Environment strategy, provider strategy, backup/restore, upgrade/rollback, operational signals, and several deployment details are unclassified. |

## Material Strengths

- AD-1 through AD-3 accurately preserve dependency direction, ownership, and exact monetary truth.
- AD-4 through AD-8 make historical integrity, transactionality, write serialization, snapshot reads, and race-safe participant archival unusually explicit.
- AD-9 correctly fixes deterministic provider concurrency, exact decoding, balance quantization, and settlement bounds where it speaks.
- AD-10 through AD-15 establish strong mutation admission, native-first HTML, topology, singleton ownership, bounded runtime behavior, safe diagnostics, probes, and shutdown.
- AD-16 and AD-17 correctly preserve testability, architecture fitness intent, dependency governance, and the permanent single-administrator boundary.
- The capability map is useful as an ownership index, but it cannot substitute for omitted behavioral rules.

## Minimum Acceptance Bar

Before acceptance, the spine should at minimum:

1. Resolve its draft/adopted/binding contradiction.
2. Add an explicit brownfield retained/superseded disposition inventory.
3. Replace schema and route/template deferrals with minimal binding logical contracts.
4. Add missing monthly-summary, group/spending workflow, rate fallback/disclosure, accessibility, cookie, and trusted-proxy rules.
5. Decide or explicitly open environment and provider strategy.
6. Fix the deployment/operations boundary for artifact, durable volume, backup/restore, upgrade/rollback, required signals, and ownership.
7. Add version verification provenance and repository enforcement for tool/browser assets.
8. Define ambiguous enforcement terms and enumerate the associated acceptance checks.
