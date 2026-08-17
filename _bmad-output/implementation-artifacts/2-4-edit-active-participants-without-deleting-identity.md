---
story_key: 2-4-edit-active-participants-without-deleting-identity
story_id: 2.4
epic: 2
status: done
created: 2026-08-17
baseline_commit: 6d66e7c
completion_note: Ultimate context engine analysis completed - comprehensive developer guide created
---

# Story 2.4: Edit Active Participants Without Deleting Identity

Status: done

## Story

As the administrator,
I want to correct an active Participant's name and color inside its Group,
so that the accounting identity remains recognizable without breaking ownership or future history.

## Acceptance Criteria

1. An active Participant in an active Group is editable from the Group's Manage section. The form is Group-scoped, shows the current name and color, accepts exactly `name`, `color`, `csrf`, and `submission_token`, and never exposes Group ownership as an editable field.
2. Empty/whitespace-only names, names over 100 Unicode characters, invalid colors, missing/duplicate/unknown fields, malformed encoding, invalid CSRF, and invalid/reused submission tokens are rejected at the existing strict boundary. Application validation returns `422`, retains every raw submitted name/color value, associates the error with the owning control, and performs no token reservation, use-case dispatch, repository call, gate acquisition, or epoch change.
3. A valid edit trims and validates the name, normalizes a valid color to uppercase `#RRGGBB`, preserves the positive Participant ID and immutable owning Group, updates name/color atomically through the shared five-second mutation/write-gate path, and redirects with `303 See Other` to `/groups/{id}/manage`.
4. A crafted request using a different Group ID cannot update or disclose the Participant. Application checks and the transactional repository predicate both enforce `(group_id, participant_id)` ownership; failure leaves the Participant unchanged and maps to a safe conflict/not-found response without raw diagnostics or identity details.
5. An archived or missing Group rejects direct edit GET/POST before token reservation and use-case dispatch. Archived Group pages remain readable, retain the Participant identity, and render no edit control. Archived Participants are not editable; no archive, restore, delete, Historical Balance, or eligibility behavior is added by this story.
6. No independent Participant deletion or reassignment capability exists. The existing immutable ownership and restrictive history-preservation rules remain intact, and editing never replaces the identity or deletes/recreates its row.
7. Concurrent admitted valid edits are individually atomic and use last-committed-write-wins semantics. No revision column, stale-edit conflict, second mutation registry, second write gate, or post-dispatch generic timeout is introduced.
8. Manage renders each active Participant in the order identity, editable name/color controls, then Save action. At 320 CSS pixels and 400% zoom the flexible name column and approximately 124px color column stack before collision; labels, swatch, fields, and Save are at least 48px targets; long names wrap without clipping; the page has no horizontal overflow.
9. The normalized text color field is authoritative; the named outlined swatch is supplementary. Participant name remains the accessible identity, and color never communicates identity state or validation state by itself.
10. During a valid edit the Save initiator becomes unavailable under one scoped polite atomic status and `aria-busy`. On commit, the updated Participant row/action receives focus and one success announcement is rendered. Native server-rendered HTML remains authoritative; optional pinned HTMX enhancement has equivalent native fallback, validation, status, focus, security, and committed-state behavior without custom JavaScript or inline script attributes.
11. Existing authentication/session/CSRF/submission-token behavior, Group shell and Manage reading order, security headers, active/archived filtering, historical-name projection, Group settings, Add Spending setup state, readiness, lifecycle, and shutdown behavior continue to work end-to-end.

Requirements: `SPEC-FR30..SPEC-FR33`, `SPEC-FR36`, `SPEC-FR87..SPEC-FR90`; `SPEC-NFR7`, `SPEC-NFR15..SPEC-NFR16`, `SPEC-NFR25`, `SPEC-NFR28..SPEC-NFR34`; UX contracts `UX-SHELL-01`, `UX-TARGET-01`, `UX-FOCUS-01`, `UX-STATUS-01`, `UX-RESPONSIVE-01`, `UX-VISUAL-01`.

## Scope Boundary

- Implement only active Participant name/color editing within the owning active Group's Manage section.
- Do not implement Participant archive, restore, deletion, balance eligibility, archived Participant views, Spending CRUD, summaries, debts, confirmations, reassignment, memberships, or participant authentication.
- Extend the existing Group-scoped Participant model and root mutation executor. Do not revive the deleted global `/participants` surface, global edit templates, reusable identity APIs, or compatibility shims.
- Update `specs/design.md` first only if the implementation reveals a normative behavior mismatch; synchronize migrations, tests, README/config, and `.sqlx` metadata whenever affected.

## Tasks / Subtasks

- [x] Extend application-owned Group-scoped edit policy (AC: 1-7)
  - [x] Add a transport-neutral `ParticipantUpdateInput` or equivalent carrying `group_id`, `participant_id`, raw `name`, and raw `color`; preserve `*Input` naming and no outer-layer types.
  - [x] Add a side-effect-free validator for positive IDs, `Name::new` trimming/Unicode length, and `Color::new` normalization; invalid input must be rejected before token reservation and repository access.
  - [x] Require the owning Group to exist and be active and the Participant to exist, belong to that Group, and be active/non-archived before dispatch. Do not disclose cross-Group identity details.
  - [x] Change application/repository update signatures from ID-only to Group-scoped. Preserve stable ID and immutable Group ownership.
  - [x] Keep safe `ApplicationError` categories and map archived/mismatch/missing cases without raw SQL or identifiers.

- [x] Reuse the single supervised mutation owner (AC: 3, 7, 11)
  - [x] Add the smallest `update_group_participant` operation to `GroupMutationExecutor` and `RootGroupMutationExecutor`, reusing the existing mutation lease, definitive outcome publication, readiness failure behavior, epoch advancement only after commit, shutdown tracking, and contention mapping.
  - [x] Do not call `state.participants.update_participant` directly from a web handler and do not create a second registry, gate, executor, or retry path.
  - [x] Ensure no generic request timeout cancels a dispatched update; last committed valid edit wins.

- [x] Enforce ownership and atomic persistence (AC: 3-7)
  - [x] Update `ParticipantRepository` and `SqliteLedgerStore` so DML is scoped by both Group ID and Participant ID and only updates active/non-archived Participants.
  - [x] Use checked SQLx macros and the existing write gate; map zero affected rows through existing safe lifecycle/participant helpers without leaking whether another Group owns the row.
  - [x] Preserve the existing `participants.group_id` immutability trigger, restrictive deletion/history rules, canonical stored color, and Group-owned allocation eligibility.
  - [x] No migration is expected from the current schema. If SQL changes, refresh committed `.sqlx` metadata and run the required temporary SQLite/online `cargo sqlx prepare --workspace --check` workflow.

- [x] Add Group-scoped web routes and Manage projections (AC: 1, 5, 8-11)
  - [x] Add only Group-scoped edit GET/POST routes under `/groups/{group_id}/participants/{participant_id}/edit`; keep route naming consistent and do not add a global alias.
  - [x] GET verifies active Group and Participant ownership/activity before rendering current values. Archived Group direct GET/POST returns pre-dispatch `409`; missing records use sanitized `404` behavior.
  - [x] POST order remains authentication, writable-Group precheck, strict structure parsing, side-effect-free application validation, token reservation immediately before supervised dispatch, then one state-changing call.
  - [x] Extend `GroupTemplate`/`MemberRow` with narrowly scoped render state for edit draft/error/focus/status. Askama fields remain explicit and render-only.
  - [x] Render active Participant identity before edit fields, preserve submitted raw drafts on `422`, field-specific `aria-invalid`/`aria-describedby`, stable guidance/error IDs, scoped polite status, and owning `aria-busy`.
  - [x] On success redirect to Manage with an allow-listed Participant focus target. HTMX swaps only the owning Participant block with equivalent native behavior; no custom JavaScript or second route projection.
  - [x] Archived Group rendering remains read-only and the five-destination shell and Manage reading order are preserved.

- [x] Preserve Editorial Contrast responsive behavior (AC: 8-10)
  - [x] Reuse existing `.participant-form-grid`, `.participant-row`, swatch, focus, and 48px control patterns; extend CSS minimally.
  - [x] Keep the flexible name plus 124px color geometry, stack before collision, make long Unicode names wrap/break, retain the outlined 48px named swatch, and prevent page-level horizontal scrolling.
  - [x] Do not add cards, pills, gradients, animation, hover lift, modal/drawer UI, light mode, or custom scripts.

- [x] Add invariant-owning and composed regression coverage (AC: all)
  - [x] Application tests cover normalization, empty/101-Unicode-boundary names, invalid colors, positive ID, exact Group scope, archived/missing/mismatched Group/Participant, repository errors, and zero repository calls for validation rejection.
  - [x] Repository tests cover successful update, persisted normalized values, stable ID/owner, wrong Group no-change/no-disclosure, archived rejection, atomicity, gate/SQLite contention, last-commit-wins, and unchanged historical ownership.
  - [x] Web tests use the real router/session/CSRF/submission-token pipeline to verify exact field allowlist, current-value rendering, raw retention, 422/focus/error associations, 303 Location, updated-row focus/status, archived 409 with zero dispatch, missing 404, and no global/delete route.
  - [x] Hostile-input tests cover malformed percent/UTF-8, missing/duplicate/unknown fields, invalid CSRF, unknown/reused token, unauthenticated access, oversized body, cross-Group path, and proof of no reservation/dispatch/use-case/repository/gate/epoch side effect where required.
  - [x] Verify native and optional HTMX parity plus 320px/400% geometry, 48px controls, long-name wrapping, focus visibility, and no page overflow.
  - [x] Retain Story 2.1-2.3 authentication, shell, settings, ownership, Add Spending, readiness, shutdown, SQLx, and real-socket regressions.

## Dev Notes

### Developer Context

This is a brownfield vertical slice immediately after completed Story 2.3. Group-owned Participants already exist and are rendered in Manage. The current application and repository still expose an ID-only `update_participant` API, but there is no Group-scoped edit route or supervised root update operation. The implementation must extend what exists rather than build a parallel Participant workflow.

The current `ParticipantService::update_participant` reads only `participant_id`, rejects archived identities, and calls an ID-only repository update. That is insufficient for a URL containing a Group ID: a pre-handler lookup alone is not a race-safe ownership guard. The repository SQL must also require both IDs in the update predicate. The current root `GroupMutationExecutor` supervises Group creation/settings and Participant creation; add the edit operation there so the web layer never bypasses mutation lifecycle/shutdown coordination.

Manage currently renders active Participant rows as read-only identity/color text plus the Add Participant form. `MemberRow` has no edit draft/error/action state. The old global participant templates are commented out and the old global routes were intentionally removed in Story 2.3; do not restore them. Add editing to the canonical Group Manage projection only.

### Current Files To Update And Preserve

| Path | Current state | Required change / preservation |
|---|---|---|
| `specs/design.md` | Normative contract requires Group-owned immutable identity, trimmed names, normalized colors, history preservation, strict unsafe forms, and no independent Participant deletion. | Read before implementation. Update first only if behavior/schema interpretation changes; preserve all accounting, security, and ownership invariants. |
| `debtor-domain/src/model.rs` | `Name::new` trims/counts 100 Unicode characters; `Color::new` trims and canonicalizes `#RRGGBB`; `Participant` has positive ID, name, color, archive state. | Reuse validators and domain values. Do not move HTTP/SQL/session types inward or replace the identity. |
| `debtor-application/src/participants.rs` | Group-owned creation plus legacy ID-only `update_participant`; reader/repository/use-case traits still include obsolete membership operations. | Add Group-scoped update input/validation and scoped port signatures. Preserve existing creation and active-member behavior unless a minimal compile-driven adjustment is required. Do not expose reassignment/delete. |
| `debtor-application/src/groups.rs` | `GroupMutationExecutor` exposes Group create/update and Participant create. | Add only the Participant update operation needed by this story, retaining the existing root-owned abstraction. |
| `debtor-infra/src/db/repos/participants.rs` | Participant update uses write gate and `UPDATE ... WHERE id = ? AND is_archived = 0`, then reloads by ID. | Scope DML by Group and Participant IDs, preserve checked SQL, safe zero-row mapping, gate, atomicity, and canonical decoding. Avoid post-commit behavior that can turn a committed write into a false retryable failure. |
| `debtor-web/src/forms.rs` | `parse_participant_form` strictly requires `name`, `color`, `csrf`, `submission_token`; shared extractor validates CSRF/token structure. | Reuse the exact parser for edit. Do not add ownership fields, permissive parsing, or token reservation in parsing. Add duplicate/missing/unknown tests if needed. |
| `debtor-web/src/handlers/memberships.rs` | Group-scoped create handler performs writable-Group precheck, application validation before reservation, supervised dispatch, 422 draft rendering, and 303 Manage redirect. | Extend this established pattern for edit GET/POST. Keep lifecycle check before route-specific work and use the same safe mapping/focus conventions. |
| `debtor-web/src/handlers/groups.rs` | `require_writable_group` implements the archived pre-dispatch boundary; canonical Manage is built through `build_group_manage_template`. | Reuse, do not add a Participant-specific Group abstraction or duplicate archived checks with divergent semantics. |
| `debtor-web/src/handlers/spending_views.rs` | `build_group_template` loads Group members, filters active/non-archived rows, builds `MemberRow`, and `build_group_manage_template` sets Manage state. | Extend projection minimally for active edit forms/drafts/focus/status. Preserve active filtering, Group settings, spending projections, and fallback behavior. |
| `debtor-web/src/templates.rs` | `GroupTemplate` owns settings, member rows, focus participant, creation draft, and status/error fields; `MemberRow` has identity/display fields. | Add explicit render-only edit fields/state or a focused row projection. Do not resurrect commented global `ParticipantsTemplate`/`ParticipantEditTemplate`. |
| `debtor-web/templates/group.html` | Manage renders active Participant rows and Add Participant form; archived Group renders read-only settings/history. | Add per-row Group-scoped edit controls/forms in identity-first order, stable IDs/ARIA/status/focus, and no edit controls for archived Groups. Keep native form and optional HTMX response equivalent. |
| `debtor-web/src/router.rs` and `debtor-web/src/handlers.rs` | Protected router has Group Manage and Group Participant-create route; no Participant edit route. | Add only Group-scoped GET/POST edit routes while preserving middleware, body limits, authentication, CSRF, safe-read timeout, and security headers. |
| `src/composition.rs` | `RootGroupMutationExecutor` owns the shared mutation registry, lifecycle guard, epoch, and Participant creation dispatch. | Add Participant update using the same lease/task/guard path. Never create another executor/gate/registry. |
| `debtor-web/src/handlers/test_support.rs` | Fakes record Group-owned creation and mutation dispatch behavior. | Extend fakes to record Group ID, Participant ID, normalized values, and no-dispatch cases. |
| `static/css/app.css` | Existing Participant form grid uses flexible name + 124px color column, stacked responsive layout, 48px controls, swatches, and focus outline. | Reuse/extend minimally for edit blocks, status, and action placement; preserve Editorial Contrast and no overflow. |
| `debtor-infra/tests/repos.rs`, `debtor-infra/tests/migrations.rs`, `debtor-web/src/router.rs`, `debtor-application/src/participants.rs` | Existing Story 2.3 tests cover ownership, active filtering, strict forms, lifecycle, and persistence. | Add update-specific ownership, normalization, archival, hostile-input, focus/status, concurrency, and unchanged-history assertions in the layer owning each invariant. |

### Architecture Compliance

- Preserve `debtor -> debtor-web/debtor-infra -> debtor-application -> debtor-domain`. Domain is synchronous/deterministic; application owns raw update policy and ports; infra owns SQLx, transaction, write gate, and race-sensitive ownership enforcement; web owns HTTP extraction, sessions/CSRF/submission tokens, Askama projections, accessibility, and safe status mapping; root owns composition, mutation supervision, epoch, startup, and shutdown.
- Follow AD-4/AD-5: Participant ownership is immutable and exactly one Group; application validates lifecycle/ownership and infra rechecks `(group_id, participant_id)` transactionally. Never implement reassignment, global identity, or user/account concepts.
- Follow AD-6/AD-13: use the one process-local SQLite runtime/write gate/mutation registry. Gate timeout begins no transaction or guarded side effect; epoch advances only after commit.
- Follow AD-10/AD-14: strict extraction, authentication, CSRF, Group lifecycle precheck, side-effect-free validation, token reservation immediately before dispatch, one supervised state-changing call, and no generic cancellation after dispatch.
- Follow AD-11/AD-18: native semantic HTML is authoritative; pinned HTMX 2.0.10 and response-targets 2.0.4 are optional. Use stable server-owned focus IDs, scoped polite status, `aria-busy`, 48px targets, 320px/400% support, and Editorial Contrast. No custom JavaScript, inline scripts, or script attributes.
- Follow AD-15: map missing/archived/mismatch/contention/storage errors to fixed safe categories. Never expose or log SQL, raw adapter errors, security identifiers, participant IDs, names/colors, URLs, or request-derived diagnostics.

### Library / Framework Requirements

- Keep the pinned stack: Rust 1.97.1/edition 2024, Axum 0.8.9, Askama 0.16.0, Tokio 1.53.1, SQLx/sqlx-cli 0.9.0, Tower 0.5.3, tower-http 0.6.11, tower-sessions 0.15.0, reqwest 0.13.4, and rust_decimal 1.42.1. Do not add or upgrade dependencies for this story.
- Preserve current Axum `Response`/redirect and middleware patterns, explicit Askama contexts, strict `CsrfValidatedForm`, and existing HTMX asset/version/digest configuration. No framework API migration is authorized.
- Use compile-time checked `sqlx::query!`/`query_as!` queries. The fixed WAL checkpoint is the only existing unchecked SQL exception. No monetary SQL or provider call is relevant to this story.
- Current framework/library versions and APIs were already established and verified by the architecture/project context; implementation should follow repository patterns rather than introduce new APIs.

### Testing Requirements

- Application: fake-backed async tests without Axum, SQLite, network, or wall clock. Assert trim/normalization, empty and 101-Unicode names, invalid colors, positive ID, exact Group scope, active/archived/missing/mismatch cases, storage errors, and no repository access on validation failure.
- Infrastructure: temporary SQLite/`#[sqlx::test]` tests for successful scoped update, canonical persisted values, stable ID/owner, zero-row wrong-Group/archived behavior, unchanged row after rejected update, atomic write, gate contention, and last-committed-write-wins. Coordinate concurrency with barriers/notifications/held locks, never sleeps.
- Web: real router/session/CSRF/submission-token path. Assert current values render, exact four-field allowlist, raw draft retention on 422, stable guidance/error IDs, field-specific `aria-invalid`/`aria-describedby`, focusable alert/sole invalid control, scoped status and `aria-busy`, `303` Manage redirect, updated row/action focus, archived 409 with zero dispatch, missing 404, cross-Group no disclosure, and absence of global/delete routes.
- Hostile inputs: malformed percent/UTF-8, missing/duplicate/unknown fields, invalid CSRF, unknown/reused token, unauthenticated request, oversized body, archived Group, missing Group, wrong Group ID, and archived Participant. Prove no reservation/dispatch/use-case/repository/gate/epoch side effect for pre-dispatch rejection.
- UX: native and HTMX parity, five-link shell/Manage order, 48px controls, visible 2px focus at 3:1, 320px/400% geometry, 124px color column, long-name wrapping, no page-level horizontal scroll, color text/swatch semantics, and one announcement on success.
- Required validation commands: `cargo fmt --all -- --check`; `cargo check --workspace --all-features --locked`; `SQLX_OFFLINE=true cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`; `cargo test --workspace --all-features --locked`; `cargo run --bin architecture-check --locked`.
- If checked SQL changes: migrate a temporary database and run `SQLX_OFFLINE=false DATABASE_URL=sqlite:///tmp/debtor-sqlx.db?mode=rwc cargo sqlx prepare --workspace --check`; refresh committed `.sqlx`. Never use `cargo build --release`.

### Previous Story Intelligence

Story 2.3 is the direct predecessor. Reuse its Group-owned schema, immutable ownership trigger, active Participant filtering, Group-scoped Manage projection, validation-before-token-reservation order, root mutation executor, deterministic server color/form patterns, saved focus/status conventions, and native/HTMX parity. Its review fixed seven important issues: removal of global/unowned APIs, use of the existing root mutation owner, lifecycle-first rejection, committed-result safety, validation focus, and expanded hostile-input/persistence/geometry coverage. Do not regress those fixes.

Story 2.2 established the canonical Manage settings projection, archived read-only behavior, settings fallback, field-specific error associations, saved-state focus/status, root update supervision, and last-commit semantics. The Participant edit must fit into that same Manage structure and must not create a divergent edit page or dispatch path.

Story 2.1 established the five-destination Group shell, URL-based selection, no-Participant Add Spending setup state, real mutation lifecycle/shutdown evidence, and the authenticated native/HTMX boundary. The edit story must preserve all of it.

### Git Intelligence

- Recent commits are `6d66e7c feat: implement 2-3 bmad`, `e0ed77b feat: implement 2-2 bmad`, `b46a2f7 feat: impelement bmad 2-1`, followed by completed Epic 1 lifecycle work.
- Story 2.3 touched application Participant policy, infra ownership/persistence, web forms/handlers/projections/router/templates, root composition, CSS, migrations, tests, and SQLx metadata. Build on those current APIs and inspect them before editing; older global Participant code is superseded.
- The current worktree was clean during analysis. Do not revert unrelated changes made concurrently.

### Project Structure Notes

- Capability modules remain plural: `groups`, `participants`, `spendings`, `debts`. Use `*Input`, `*Reader`, `*Repository`, `*UseCases`, `Db*`, and `*Template`/`*Row`/`*View` names.
- The current schema already has `participants.group_id` as required and immutable; no migration is expected unless implementation exposes a genuine structural gap.
- Keep Participant editing in Group Manage. Do not add a global participant page, ownership field, cross-Group picker, independent delete, archive controls, or compatibility alias.
- Updating the Participant row is the correct history-preserving behavior: later Spending history must resolve the current Participant name, while the positive ID and Group ownership remain stable.

### References

- [Source: `_bmad-output/planning-artifacts/epics.md#Story 2.4: Edit Active Participants Without Deleting Identity`]
- [Source: `_bmad-output/planning-artifacts/epics.md#Epic 2: Organize Groups and Participants`]
- [Source: `_bmad-output/planning-artifacts/epics.md#Final-Evidence Ledger`]
- [Source: `specs/design.md#Architecture`]
- [Source: `specs/design.md#Accounting And History`]
- [Source: `specs/design.md#Security`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-4 - Group-owned identity and history`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-5 - Application policy with transactional enforcement`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-6 - Single ledger runtime and mutation epoch`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-10 - Shared unsafe-request admission boundary`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-11 - Native HTML authority`]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-debtor-2026-08-10/ARCHITECTURE-SPINE.md#AD-18 - Governed UX contracts and traceability`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md#Information Architecture`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md#Lifecycle and Access`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/EXPERIENCE.md#Interaction Focus Matrix`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/DESIGN.md#Layout & Spacing`]
- [Source: `_bmad-output/planning-artifacts/ux-designs/ux-debtor-2026-08-10/DESIGN.md#Components`]
- [Source: `_bmad-output/implementation-artifacts/2-3-add-group-owned-participants.md#Previous Story Intelligence`]
- [Source: `_bmad-output/implementation-artifacts/2-2-configure-group-settings.md#Previous Story Intelligence`]
- [Source: `_bmad-output/project-context.md#Critical Implementation Rules`]
- [Source: `debtor-application/src/participants.rs`]
- [Source: `debtor-infra/src/db/repos/participants.rs`]
- [Source: `debtor-web/src/handlers/memberships.rs`]
- [Source: `debtor-web/src/handlers/spending_views.rs`]
- [Source: `debtor-web/src/templates.rs`]
- [Source: `debtor-web/templates/group.html`]
- [Source: `debtor-web/src/forms.rs`]
- [Source: `debtor-web/src/router.rs`]
- [Source: `src/composition.rs`]
- [Source: `migrations/20260517000002_create_participants.up.sql`]
- [Source: `static/css/app.css`]

## Dev Agent Record

### Agent Model Used

openai/gpt-5.6-luna

### Debug Log References

- Resolved create-story customization: no activation prepend/append steps; loaded persistent fact `_bmad-output/project-context.md`.
- Read the complete `sprint-status.yaml` and selected the first backlog story in order: `2-4-edit-active-participants-without-deleting-identity`.
- Loaded the complete Epic 2/Story 2.4 context, normative design, architecture spine, UX contracts, project context, completed Stories 2.2/2.3, current Participant application/infra/web/root code, schema, CSS, and recent Git history.
- Current implementation audit found an ID-only Participant update API and repository predicate, no Group-scoped edit route/projection, and no supervised root Participant-update operation. Existing Group-owned schema and mutation infrastructure must be extended, not duplicated.
- No dependency or framework API change is authorized; the pinned project versions and established patterns are sufficient.
- Final route audit added archived/protected access assertions for the new Group-scoped edit path; targeted tests passed.

### Completion Notes List

- Ultimate context engine analysis completed - comprehensive developer guide created.
- Implemented `ParticipantUpdateInput` and application validation with active Group, active Participant, and immutable ownership checks.
- Added root-supervised Participant updates and transactional `(group_id, participant_id)` SQL with canonical value persistence.
- Added Group-scoped Manage edit forms, strict native/HTMX routes, retained validation drafts, accessible status/focus behavior, and responsive Editorial Contrast styling.
- Added application, repository, and web regression coverage, including protected and archived route coverage; refreshed SQLx metadata.
- Validation passed: `cargo fmt --all -- --check`, `cargo check --workspace --all-features --locked`, `cargo test --workspace --all-features --locked`, strict offline Clippy, architecture fitness, and online SQLx preparation check.
- Code review completed with seven findings resolved; final workspace tests, strict Clippy, formatting, architecture fitness, and SQLx metadata checks passed.

### File List

- `_bmad-output/implementation-artifacts/2-4-edit-active-participants-without-deleting-identity.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `.sqlx/query-943eec0f6d3dfbf69aed5d1147b135cc80b12a5b40491e2dee652f3e7b2432f1.json`
- `.sqlx/query-5a661e087377a5aedd386b3dad7e76fd2ec7827aa5b27ec966dad66905d47073.json` (deleted obsolete query metadata)
- `debtor-application/src/groups.rs`
- `debtor-application/src/participants.rs`
- `debtor-infra/src/db/repos/participants.rs`
- `debtor-infra/tests/repos.rs`
- `debtor-web/src/handlers.rs`
- `debtor-web/src/handlers/groups.rs`
- `debtor-web/src/handlers/memberships.rs`
- `debtor-web/src/handlers/spending_views.rs`
- `debtor-web/src/handlers/test_support.rs`
- `debtor-web/src/router.rs`
- `debtor-web/src/templates.rs`
- `debtor-web/templates/group.html`
- `src/composition.rs`
- `static/css/app.css`
- `debtor-web/templates/participant_edit_row.html`

### Change Log

- 2026-08-17: Implemented Group-scoped active Participant editing, supervised mutation dispatch, transactional ownership enforcement, Manage UI, accessibility behavior, regression tests, and SQLx metadata; moved story to `review`.
- 2026-08-17: Resolved all seven code review findings with transactional lifecycle guards, HTMX row rendering, safe read-error mapping, deterministic focus/guidance associations, and regression coverage; moved story to `done`.

### Review Findings

- [x] [Review][Patch] HTMX validation swaps a full document into the Participant row — `group.html:98`, `memberships.rs:235-249`. Added an Askama row fragment for enhanced validation responses and `HX-Redirect` for successful enhanced mutations.
- [x] [Review][Patch] Participant update does not transactionally require an active Group — `debtor-infra/src/db/repos/participants.rs:94-101`. Added the active-Group predicate to the checked transactional update and lifecycle regression coverage.
- [x] [Review][Patch] Participant update does not transactionally require active membership — `debtor-infra/src/db/repos/participants.rs:94-101`. Added the active-membership predicate to the update and persistence regression coverage.
- [x] [Review][Patch] Edit GET converts Participant read/storage failures into `404` — `debtor-web/src/handlers/memberships.rs:187-196`. The lookup now returns and maps application errors instead of collapsing them to false.
- [x] [Review][Patch] Validation can emit multiple autofocus targets — `debtor-web/src/handlers/memberships.rs:237-249`, `debtor-web/templates/group.html:94,102,105`. Validation clears row focus so only the invalid control receives autofocus.
- [x] [Review][Patch] Edit fields lack field-specific guidance associations — `debtor-web/templates/group.html:98-110`. Added distinct name/color guidance IDs and control associations.
- [x] [Review][Patch] Required edit regression coverage is incomplete — `debtor-application/src/participants.rs:501-529`, `debtor-infra/tests/repos.rs:326-372`, `debtor-web/src/router.rs:1374-1435`. Added HTMX fragment, lifecycle predicate, protected-route, and persistence regression coverage; shared strict-form/token tests remain authoritative for common boundary behavior.
