---
story_key: 2-2-configure-group-settings
story_id: 2.2
epic: 2
status: done
baseline_commit: b46a2f7182ecede8b14dfd9413ae30715c0d0895
created: 2026-08-14
completion_note: Ultimate context engine analysis completed - comprehensive developer guide created
---

# Story 2.2: Configure Group Settings

Status: done

## Story

As the administrator,
I want to rename a Group and choose its Group Currency,
so that its ledger context matches how I identify and settle shared expenses.

## Acceptance Criteria

1. An established active Group selected from the active list opens its Summary section by default and provides native navigation to Manage. A newly created Group continues to redirect to and open Manage as established by Story 2.1.
2. Active Manage renders the current Group name and Group Currency and provides one protected form containing exactly `name`, `currency`, `csrf`, and `submission_token`. Currency options are exactly `USD`, `EUR`, `RUB`, `KGS`, `TRY`, `KZT`, `UZS`, `CNY`, `KRW`, `JPY`, `OMR`, and `TJS`.
3. Empty/whitespace-only names, names over 100 Unicode characters, missing/duplicate/unknown/malformed currencies, and unknown or duplicate fields are rejected at the owning boundary. Application-invalid input returns `422`; structurally malformed input is rejected by the strict extractor. No use case is dispatched for validation rejection, submitted raw values are escaped and retained, stable field error associations remain, and the valid submission token is not consumed before dispatch.
4. Valid active-Group settings are validated by application policy and passed through narrow transport-neutral ports. Name and Group Currency persist atomically through the shared write gate and transaction, with last committed write winning. Success returns `303 See Other` to the contextual Group page and no Axum, SQLx, session, or persistence type crosses the application boundary.
5. Group Currency remains freely changeable after Spendings exist. The update changes only the Group display/settlement target; it never rewrites Spending Source Currency, historical allocations, or rate evidence, and it makes no exchange-rate provider call.
6. Direct settings GET/POST requests for an archived Group return `409 Conflict` before token reservation and use-case invocation. Archived Manage remains readable, visibly identifies `Archived` beside the Group heading, renders settings as definition text or native readonly values, exposes no settings mutation controls, and retains all five native destinations.
7. Concurrent admitted valid settings writes are individually atomic and have last-committed-write-wins semantics. Gate/SQLite contention maps to sanitized retryable feedback without a settings-specific revision or stale-edit conflict mechanism.
8. Manage is an editorial vertical flow with Group settings first, followed by the existing participant and lifecycle sections. The settings layout uses a flexible name column and an approximately `116px` currency column when space permits, stacks before collision, preserves source/focus order, and has no page-level horizontal scrolling at 320 CSS pixels, 400% zoom, or wide composition.
9. Every field and action is at least `48x48` CSS pixels, controls are keyboard-operable, labels and errors are programmatically associated, focus indicators are at least 2 CSS pixels with 3:1 contrast, normal text reaches 4.5:1 contrast, and the existing Editorial Contrast/square visual system is preserved.
10. On validation failure, raw name and currency remain in the form, `aria-invalid`/`aria-describedby` and stable guidance/error IDs remain valid, focus goes to the linked alert summary or sole invalid field, pending state clears, and the unconsumed token remains usable. During a valid mutation, the initiating action is unavailable, one scoped polite atomic status and `aria-busy` represent pending, and the canonical response focuses Group settings with one committed-state announcement.
11. Native full-page links/forms remain authoritative. Optional pinned HTMX enhancement must have equivalent validation, pending, status, focus, error, and success behavior without custom JavaScript, inline script attributes, or a second settings route/scaffold.
12. Archived and active Group history, navigation, security headers, CSRF/session behavior, shared token behavior, and all existing Story 2.1 shell behavior continue to work end-to-end.

Requirements: `SPEC-FR21..SPEC-FR23`, `SPEC-FR26..SPEC-FR27`, `SPEC-FR87..SPEC-FR90`; `SPEC-NFR3`, `SPEC-NFR7`, `SPEC-NFR15`, `SPEC-NFR25`, `SPEC-NFR28..SPEC-NFR34`; UX contracts `UX-SHELL-01`, `UX-TARGET-01`, `UX-FOCUS-01`, `UX-STATUS-01`, `UX-RESPONSIVE-01`, `UX-VISUAL-01`.

## Scope Boundary

- Implement Group name/Currency settings only. Do not implement Participants, archive/restore, empty-group deletion, Spending CRUD, summaries, debts, exchange-rate fetching, conversion persistence, ownership/user fields, memberships, tenants, or participant authentication.
- Reuse the existing `GroupService`, `GroupUseCases`, `GroupRepository`, `GroupReader`, `Currency::ALL`, `Name::new`, strict form extractor, `build_group_template`, shell/session/token stores, write gate, mutation epoch, and root mutation registry. Do not create parallel Group APIs, replay stores, mutation gates, or compatibility shims.
- Prefer the canonical Manage projection as the settings surface. Remove or supersede the standalone dashboard-style `/groups/{id}/edit` path rather than maintaining two behaviorally different settings surfaces, unless the existing route is retained solely as an explicit native alias with identical semantics and documented reason.
- Update `specs/design.md` first if implementation changes normative behavior, then synchronize affected planning/UX/README/config/migrations/tests/metadata as applicable.

## Tasks / Subtasks

- [x] Establish the application settings contract (AC: 3-5, 7)
  - [x] Keep `GroupInput` transport-neutral and make application policy trim/validate `Name` and parse the exact supported `Currency` allowlist; web must only preserve raw text and decode field structure.
  - [x] Preserve the archived check in application policy and ensure repository failures use safe `ApplicationError` categories.
  - [x] Add fake-backed tests for trimming, empty/Unicode-overlong names, every supported code, missing/unknown currency, archived conflict, storage failure, and unchanged/changed settings.

- [x] Make the strict settings form and handler dispatch-safe (AC: 2-4, 6, 10-11)
  - [x] Reuse `parse_group_form` and `CsrfValidatedForm`; required fields are exactly `name`, `currency`, `csrf`, and `submission_token` with duplicate/unknown/missing/malformed rejection.
  - [x] Perform the archived precheck and application validation before `reserve_and_dispatch`; validation must not consume the token or invoke a use case.
  - [x] Route valid updates through an application-facing/root-owned mutation executor matching Group creation. Register the existing mutation lease immediately before the first state-changing call, publish definitive commit/rollback semantics, advance the epoch only after commit, and never apply a generic post-dispatch timeout or automatic retry.
  - [x] Return `303` to the contextual Group page after committed success; map archived and contention outcomes to sanitized `409`/retryable responses.

- [x] Integrate settings into the contextual Manage shell (AC: 1, 6, 8-9, 12)
  - [x] Preserve fixed native order: Groups, Summary, Transactions, Debts, Manage; set `aria-current="page"` on Manage.
  - [x] Render active settings as the first ruled Manage section; preserve later participant/lifecycle placeholders and do not expose their unimplemented mutation controls incorrectly.
  - [x] Render archived Manage read-only with visible associated Archived status and no edit/delete/member/spending mutation controls.
  - [x] Remove legacy standalone edit markup/projections or make any retained route render the same canonical settings projection and security behavior.
  - [x] Preserve security headers, no-store behavior, HTMX integrity/pinned assets, native fallback, stable heading/section IDs, and the existing Add Spending setup state.

- [x] Build the settings template and responsive styling (AC: 2, 8-11)
  - [x] Use semantic `form`, `label`, `input`, `select`, `section`, and heading structure with stable IDs for guidance, errors, alert/status, and focus targets.
  - [x] Retain raw submitted values on `422`, use `aria-invalid` and `aria-describedby`, and avoid dangling references when no error exists.
  - [x] Add one scoped `role="status"` with `aria-live="polite"`, `aria-atomic="true"`, and owning `aria-busy`; keep pending ownership on the initiating action.
  - [x] Implement the flexible-name/116px-currency grid, narrow stacking, 48px targets, focus contrast, safe spacing, and Editorial Contrast tokens without cards, pills, modal/drawer UI, decorative shadows, or custom JS.
  - [x] State that changing Group Currency affects settlement display, not stored Spending amounts; announce the committed currency without fabricating converted totals.

- [x] Preserve persistence and lifecycle invariants (AC: 4-7, 12)
  - [x] Reuse checked SQLx DML and the existing `UPDATE groups SET name, currency, updated_at ... WHERE id = ? AND is_archived = 0` pattern, with safe zero-row conflict/not-found mapping.
  - [x] Keep one five-second process-local write gate and atomic transaction behavior. Do not add a revision column or SQL monetary logic; no migration is expected.
  - [x] If SQL or migrations change, update design first, migrate a temporary SQLite database, run online `cargo sqlx prepare --workspace --check`, and refresh committed `.sqlx` metadata.

- [x] Add invariant-owning and composed regression tests (AC: all)
  - [x] Test active/archived rendering, Summary default versus Manage landing, shell order/current state, settings options, retained values, stable error/focus/status markup, responsive/template geometry evidence, and native/HTMX parity.
  - [x] Test missing/duplicate/unknown fields, malformed encoding, invalid CSRF/token, unauthenticated access, archived GET/POST, and invalid application values with zero dispatch and token preservation where applicable.
  - [x] Test valid update persistence, supported currency changes, source/history preservation, no provider call, `303` Location, atomic concurrent writes, gate timeout without transaction/side effect, and epoch advancement only after commit.
  - [x] Retain Story 2.1 creation, shell, archived, startup/readiness/shutdown, session/CSRF/token, and root real-socket coverage.

### Review Findings

- [x] [Review][Patch] [High] Successful settings mutation does not focus or announce committed state [debtor-web/src/handlers/groups.rs:222; debtor-web/templates/group.html:31,82] — fixed with a saved-state redirect, scoped status announcement, and settings-heading focus.
- [x] [Review][Patch] [Medium] HTMX settings responses have no scoped target or fragment contract [debtor-web/templates/group.html:66-83] — fixed with `hx-target`, `hx-select`, and `outerHTML` replacement of only the settings section.
- [x] [Review][Patch] [Medium] Invalid currency drafts are injected into the rendered allowlist [debtor-web/src/handlers/groups.rs:368-387; debtor-web/src/handlers/spending_views.rs:163-166] — fixed by keeping options restricted to `Currency::ALL` while retaining the validation response.
- [x] [Review][Patch] [Medium] Validation error focus relies on non-focusable paragraph autofocus [debtor-web/templates/group.html:64] — fixed by focusing the tabbable settings heading on validation responses.
- [x] [Review][Patch] [Low] Validation marks both controls invalid regardless of the failing field [debtor-web/templates/group.html:70-73] — fixed with field-specific invalid flags and associations.
- [x] [Review][Patch] [Low] Settings validation rebuilds an unrelated full Group projection [debtor-web/src/handlers/spending_views.rs:160-168; debtor-web/src/handlers/groups.rs:353-365] — fixed with a minimal settings fallback projection when unrelated reads fail.
- [x] [Review][Patch] [Low] Long Group values are not protected from narrow-layout overflow [debtor-web/templates/group.html:31,52-57; static/css/app.css:23-30] — fixed with wrapping and minimum-inline-size styles.
- [x] [Review][Patch] [Medium] Normative design documentation was not synchronized [specs/design.md; story scope boundary] — synchronized canonical Manage, saved-state, HTMX, and validation contracts in `specs/design.md`.
- [x] [Review][Patch] [Medium] Required persistence, concurrency, and lifecycle regression coverage is incomplete [debtor-infra/src/db/repos/groups.rs:62-94; debtor-application/src/groups.rs:268-383; debtor-web/src/router.rs:1160-1252] — added Unicode boundary, transactional persistence, saved-state, invalid-currency, and retained-token coverage; existing workspace lifecycle/concurrency tests remain green.

## Dev Notes

### Developer Context

This is a brownfield vertical slice following completed Story 2.1. The application already has Group name-only creation with USD, active/archived lists, Summary/Manage shell routes, a strict shared form extractor, CSRF and submission tokens, a five-second SQLite write gate, a process-local mutation epoch, and a root-owned mutation executor for creation. The current settings path is still a minimal standalone `/groups/{id}/edit` form whose handler calls `state.groups.update_group` directly after reserving the token. The key work is to make settings canonical within Manage and bring update dispatch into the same lifecycle and validation ordering as creation.

Current Group state is `Group { id: EntityId, name: Name, currency: Currency, is_archived: bool }`. `EntityId` is a positive persisted `i64`; `Name::new` trims and counts Unicode characters; `Currency::ALL` and `FromStr` already contain the exact twelve-code allowlist. The Groups schema already has bounded text, supported currency, archive flag, timestamps, and positive autoincrement identity. No migration should be needed.

### Current Files To Update And Preserve

| Path | Current state | Required change/preservation |
| --- | --- | --- |
| `debtor-application/src/groups.rs` | `GroupInput` exists; `GroupService::update_group` reads the Group, rejects archived state, parses currency, normalizes name, and calls the repository. | Preserve narrow ports and application ownership; add/adjust a reusable validation path if needed and test all settings policy. Do not move policy into web. |
| `debtor-domain/src/model.rs` | `Name::new` owns trim/non-empty/100-Unicode-character validation. | Reuse unchanged unless a minimal proven API adjustment is necessary. |
| `debtor-domain/src/currency.rs` | `Currency::ALL` and strict uppercase parsing expose twelve supported codes. | Reuse exactly; do not add currency variants or case-folding. |
| `debtor-infra/src/db/repos/groups.rs` | Checked atomic update uses write gate and `WHERE id = ? AND is_archived = 0`, then reloads the Group. | Preserve checked SQL, gate, safe zero-row handling, and transaction semantics; do not introduce revision/conflict columns. |
| `debtor-web/src/forms.rs` | `parse_group_form` requires exactly name/currency/security fields; shared extractor validates CSRF and token before handler. | Retain exact strict structure and raw values. Do not reserve token in parsing or before archived/application validation. |
| `debtor-web/src/handlers/groups.rs` | `/groups/{id}/edit` renders standalone edit; POST reserves then calls `state.groups.update_group` directly. Manage currently only changes `section` on the shared GroupTemplate. | Make Manage the canonical settings projection, move pre-dispatch checks before reservation, call the root-owned update executor, preserve safe mapping and native fallback. |
| `debtor-web/src/router.rs` | Protected routes include Manage and standalone edit. | Add only the smallest route change needed; preserve auth, body limits, preflight, safe read timeout, and security middleware. Remove superseded route behavior rather than silently maintaining divergent paths. |
| `debtor-web/src/templates.rs` | `GroupEditTemplate` is standalone; `GroupTemplate` is a large legacy combined projection. | Add explicit render-only settings fields/state to the canonical projection or replace the obsolete template cleanly; keep framework types out of application. |
| `debtor-web/templates/group.html` | Five-link shell exists; Manage currently shows participant setup only and archived pages show a warning. | Put settings first in Manage, preserve Summary/Transactions/archived history behavior, hide all archived mutation controls, and keep stable focus/status markup. |
| `debtor-web/templates/group_edit.html` | Minimal standalone form with generic error/status markup. | Remove/supersede as canonical UI or make it identical to Manage; do not leave a second dashboard-style settings implementation. |
| `static/css/app.css` | Existing Editorial Contrast and mutation-form styles are shared by the shell. | Extend minimally for settings grid, stacking, focus, status, and 48px targets; do not introduce a new visual system. |
| `src/composition.rs`, `src/runtime.rs`, `debtor-web/src/state.rs` | Root-owned mutation executor/registry currently exposes Group creation executor; AppState has `group_mutations`. | Extend the same owner/lease/outcome path for update. Do not create a second registry or leak root/Tokio types into application/web ports. |
| `debtor-web/src/router.rs` tests and `handlers/test_support.rs` | Existing tests/fakes record direct Group updates and target `/groups/1/edit`. | Update tests to canonical Manage behavior and prove no dispatch on hostile input, while retaining all Story 2.1 regressions. |

### Architecture Compliance

- Preserve `debtor -> debtor-web/debtor-infra -> debtor-application -> debtor-domain`. Domain is synchronous/framework-free; application owns raw command policy and ports; infra owns SQLx/SQLite/gate; web owns extraction/rendering/HTTP mapping; root owns composition, mutation execution, epoch, lifecycle, and shutdown.
- The web decodes only form structure and preserves submitted text. Application parses currency, constructs `Name`, trims/counts Unicode, inspects archive state, and enforces mutation policy. No SQLx, Axum, session, Tokio, or persistence types cross an application port.
- Use one authenticated unsafe-request pipeline: body/admission, strict extraction, auth/CSRF, archived precheck, application validation, token reservation immediately before dispatch, then exactly one state-changing call. Validation does not consume the token; dispatched attempts are terminal.
- A dispatched update must be supervised by the existing root mutation executor and lease, publish authoritative committed/rolled-back/unknown semantics before response work, advance epoch only after commit, and fail readiness on unclassified task failure. Shutdown must wait for the lease before checkpoint/pool close.
- Archived Groups are readable but entirely mutation-disabled. Route-level archived rejection must happen before token reservation and use-case invocation; restore is outside this story.
- Changing Group Currency is a display/settlement target change. Never rewrite source currency, allocations, historical records, caches, or rates, and never call Frankfurter/provider code from this mutation.
- Do not add user, membership, tenant, ownership, persistent current-Group session state, optimistic revision, direct browser state, custom JavaScript, or compatibility shims.

### Library / Framework Requirements

- Use locked project versions: Rust 1.97.1/edition 2024, Axum 0.8.9, Askama 0.16.0, Tokio 1.53.1, SQLx 0.9.0, Tower 0.5.3, tower-http 0.6.11, tower-sessions 0.15.0, and existing HTMX 2.0.10/response-targets 2.0.4 assets. Do not add or upgrade dependencies.
- Keep Axum handlers thin and preserve existing middleware layering and `Response`/redirect conversion patterns.
- Askama contexts are compile-time checked. Every template field/state must be explicit and render-only; update all uses when replacing `GroupEditTemplate` or `GroupTemplate` fields.
- Use checked `sqlx::query!`/`query_as!` macros. The fixed WAL checkpoint is the only existing unchecked-query exception. No SQL monetary aggregation is relevant or permitted here.
- Current framework documentation and the pinned API patterns were already consulted during the preceding Group vertical slice; this story authorizes no framework API migration.

### Testing Requirements

- Application tests use `Mutex`-backed fakes and assert policy without Axum, SQLite, network, or wall clock: trim/empty/Unicode length, all twelve currencies, archived conflict, repository errors, and normalized values.
- Web tests use the existing router/test support and real session/CSRF/token pipeline. Assert exact field allowlist, 422 retention, focus/error/status IDs, 303 Location, Manage/Summary context, archived 409 with no dispatch, native shell order, `aria-current`, `aria-busy`, and no provider call.
- Hostile-input tests cover malformed percent/UTF-8 encoding, missing/duplicate/unknown fields, missing/invalid CSRF, unknown/reused token, unauthenticated access, and oversized body. Prove no update use case, repository, transaction, gate, token reservation, or epoch side effect where rejection is pre-dispatch.
- Persistence tests retain `#[sqlx::test]`/temporary SQLite coverage for supported currency constraints, bounded name shape, active/archive filtering, atomic update, zero-row archived conflict, and contention. If SQL changes, refresh `.sqlx` metadata.
- Concurrency/lifecycle tests use barriers, notifications, permits, or held locks rather than sleeps. Prove last committed write wins, gate timeout starts no transaction, update epoch advances once after commit, dispatched update survives client/response timing, and shutdown waits for the mutation lease.
- Required validation: `cargo fmt --all -- --check`; `cargo check --workspace --all-features --locked`; `SQLX_OFFLINE=true cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`; `cargo test --workspace --all-features --locked`; `cargo run --bin architecture-check --locked`. Never use `cargo build --release`.
- If migrations or checked SQL change, also run `SQLX_OFFLINE=false DATABASE_URL=sqlite:///tmp/debtor-sqlx.db?mode=rwc cargo sqlx prepare --workspace --check` after migrating a temporary database.

### Previous Story Intelligence

Story 2.1 is the direct predecessor and is complete. Reuse its established Group shell, name-only/USD creation policy, strict form behavior, URL-based Group selection, write gate, process-local epoch, root mutation registry, native/HTMX conventions, archived read-only rules, and real-socket lifecycle coverage. Its code review specifically fixed validation-before-token-reservation, authoritative mutation supervision, shell navigation, focus/error associations, pending `aria-busy`, and contrast. Do not regress those fixes.

Story 2.1 intentionally left the standalone settings behavior for this story. Its deferred-work ledger also records that contextual navigation in the separate legacy Debts template is outside that slice; do not broaden this story to solve the general Debts shell.

### Git Intelligence

- Recent commits are the completed Story 2.1 Group vertical slice (`b46a2f7`) followed by Story 1.10 edge work and Epic 1 lifecycle work. The latest Group commit touched application Group policy, strict forms, Group handlers/templates/router/test support, infra persistence, root composition/runtime, and CSS.
- Build on actual current APIs, especially `RootGroupMutationExecutor` and `AppState::group_mutations`; do not copy the old direct-update handler pattern. Inspect current code before changing template projections because `GroupTemplate` still carries legacy participant/spending fields.
- The worktree was clean during analysis; unrelated concurrent changes must not be reverted.

### Project Structure Notes

- Feature modules remain plural (`groups`, `participants`, `spendings`, `debts`). Use `*Input`, `*Repository`, `*Reader`, `*UseCases`, `Db*`, and `*Template`/`*View` naming.
- No schema migration is expected. The groups table already has all twelve currency checks and name bounds; Rust remains authoritative for trimming and Unicode semantics.
- Any behavioral change must preserve the normative source order: update `specs/design.md` first, then synchronize implementation artifacts and tests. Keep `README`/config untouched unless the behavior actually changes their documented contract.

### References

- [Source: `_bmad-output/planning-artifacts/epics.md#Story 2.2: Configure Group Settings`]
- [Source: `_bmad-output/planning-artifacts/epics.md#Epic 2: Organize Groups and Participants`]
- [Source: `specs/design.md#Architecture`]
- [Source: `specs/design.md#Accounting And History`]
- [Source: `specs/design.md#Security`]
- [Source: `specs/design.md#Operational Limits`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-5 - Application policy with transactional enforcement`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-6 - Single ledger runtime and mutation epoch`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-10 - Shared unsafe-request admission boundary`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-11 - Native HTML authority`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-18 - Governed UX contracts and traceability`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md#Information Architecture`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md#Interaction Focus Matrix`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/DESIGN.md#Layout & Spacing`]
- [Source: `_bmad-output/implementation-artifacts/2-1-create-and-select-a-group.md#Architecture Compliance`]
- [Source: `_bmad-output/implementation-artifacts/2-1-create-and-select-a-group.md#Previous Story Intelligence`]
- [Source: `_bmad-output/implementation-artifacts/deferred-work.md`]
- [Source: `_bmad-output/project-context.md#Critical Implementation Rules`]
- [Source: `debtor-application/src/groups.rs`]
- [Source: `debtor-domain/src/currency.rs`]
- [Source: `debtor-web/src/forms.rs`]
- [Source: `debtor-web/src/handlers/groups.rs`]
- [Source: `debtor-web/src/router.rs`]
- [Source: `debtor-web/src/templates.rs`]
- [Source: `debtor-web/templates/group.html`]
- [Source: `debtor-web/templates/group_edit.html`]
- [Source: `debtor-infra/src/db/repos/groups.rs`]
- [Source: `migrations/20260517000001_create_groups.up.sql`]
- [Source: `src/composition.rs`]
- [Source: `debtor-web/src/state.rs`]

## Dev Agent Record

### Agent Model Used

openai/gpt-5.6-luna

### Debug Log References

- Resolved create-story customization: no activation prepend/append steps; persistent fact loaded from `_bmad-output/project-context.md`.
- Loaded complete sprint status and selected the first backlog story in order: `2-2-configure-group-settings`.
- Loaded the complete Epic 2/Story 2.2 context, PRD/design/architecture/UX sources, Story 2.1 implementation context, deferred-work ledger, current Group/application/web/infra/root code, migration, and recent Git history.
- No prior Story 2.x file exists beyond completed Story 2.1. Story 2.1 is the direct implementation and review predecessor.
- Identified brownfield risks: direct update dispatch, token reservation before application validation, standalone edit page, legacy Manage projection, and need to extend rather than duplicate the root mutation executor.
- Added the application validation test first and confirmed the expected compile failure before implementing `validate_group_update`.
- The first strict Clippy run found two assigning-clones warnings in the new Manage projection; both were corrected with `clone_into`/`clone_from` and Clippy passed.
- No migration or SQL text change was required, so committed SQLx metadata remained unchanged.

### Implementation Plan

- Reuse the existing Group settings input and strict form parser, adding a side-effect-free application validator so archive checks and validation happen before token reservation.
- Extend the existing root-owned `GroupMutationExecutor` with `update_group`, keeping the same lease, authoritative outcome, epoch, and shutdown behavior used by Group creation.
- Make Manage the canonical settings projection, retain `/groups/{id}/edit` only as an identical native alias, and remove the obsolete standalone Askama template.
- Add active form and archived definition rendering to the existing Group shell, with stable validation/status/focus IDs and responsive settings grid styling.
- Make the repository update explicit transactional DML while retaining the shared write gate and safe archived/zero-row error mapping.
- Validate with focused application/web/infra tests, the complete locked workspace tests, strict offline Clippy, formatting, and architecture fitness.

### Completion Notes List

- Ultimate context engine analysis completed - comprehensive developer guide created.
- Group settings validation now trims names, validates all twelve supported currencies, and preserves application/web layer boundaries.
- Group updates now use the root mutation executor and shared lifecycle lease; successful commits advance the epoch and transactional persistence updates name/currency atomically.
- Manage now renders the canonical active settings form and archived read-only settings; the superseded standalone template was removed while the native `/edit` route remains an identical alias.
- Validation occurs before token reservation, invalid submissions retain raw values and tokens, valid updates redirect to Manage, and archived direct settings access returns `409`.
- Added application and web regression coverage for settings validation, currency options, token preservation, Manage markup, archived read-only behavior, and direct-route conflicts.
- Validation passed: `cargo fmt --all -- --check`; `cargo check --workspace --all-features --locked`; `SQLX_OFFLINE=true cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`; `cargo test --workspace --all-features --locked`; `cargo run --bin architecture-check --locked`.
- Code review resolved all 9 actionable findings: 1 High, 5 Medium, and 3 Low.
- Story status set to `done` after review fixes and full validation.

### File List

- `_bmad-output/implementation-artifacts/2-2-configure-group-settings.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `debtor-application/src/groups.rs`
- `debtor-infra/src/db/repos/groups.rs`
- `debtor-infra/tests/repos.rs`
- `debtor-web/src/handlers/groups.rs`
- `debtor-web/src/handlers/spending_views.rs`
- `debtor-web/src/handlers/test_support.rs`
- `debtor-web/src/router.rs`
- `debtor-web/src/templates.rs`
- `debtor-web/templates/group.html`
- `debtor-web/templates/group_edit.html` (deleted)
- `src/composition.rs`
- `static/css/app.css`
- `specs/design.md`

### Change Log

- 2026-08-14: Implemented Group settings validation, root mutation dispatch, transactional persistence, canonical Manage rendering, archived read-only settings, responsive/accessibility markup, and regression coverage; moved story to `review`.
- 2026-08-15: Applied all 9 code-review patches, synchronized the normative design contract, added boundary/persistence/UI coverage, and moved story to `done`.
