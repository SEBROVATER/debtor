---
story_key: 2-3-add-group-owned-participants
story_id: 2.3
epic: 2
status: done
created: 2026-08-15
baseline_commit: e0ed77bef7569dffcf835a883fb319bac7af4277
completion_note: Ultimate context engine analysis completed - comprehensive developer guide created
---

# Story 2.3: Add Group-Owned Participants

Status: done

## Story

As the administrator,
I want to add accounting identities inside a Group,
so that its shared Spendings can later identify who paid and who owes.

## Acceptance Criteria

1. An active Group's Manage section renders only that Group's active Participants and one protected, Group-scoped Add Participant form. There is no global Participant list, user account, membership-management, login, tenant, or cross-Group reuse control.
2. A fresh Add Participant form receives a server-generated varied normalized `#RRGGBB` suggestion. The Administrator may replace it with another valid color, and no custom JavaScript is required.
3. A name that trims to empty or exceeds 100 Unicode characters, or a color that is not normalized valid `#RRGGBB`, returns `422 Unprocessable Entity`, retains the raw submitted name and color, associates errors with the owning controls, and performs no token reservation or use-case dispatch.
4. Valid input for an active Group is parsed and normalized by application policy, creates a positive `i64` active Participant owned by exactly that Group, and redirects with `303 See Other` to `/groups/{id}/manage`, where the new Participant is visible.
5. A Participant identity cannot be reused across Groups. Crafted requests, repository calls, and persistence constraints reject cross-Group ownership or reassignment without creating or changing an identity.
6. An archived Group rejects its Add form and mutation route with pre-dispatch `409 Conflict`, without reserving a submission token or invoking a use case. A missing Group returns a sanitized not-found response. Neither path creates a Participant.
7. Persistence structurally enforces positive identity, required single-Group ownership, bounded non-empty text, normalized color shape, boolean archive state, and the history-free Group deletion behavior. The application exposes no independent Participant deletion capability.
8. Active Manage renders identity guidance before the Participant form, uses a flexible name plus approximately `124px` color column that stacks before collision, reserves a labeled 48px outlined swatch, and keeps every control/action at least `48x48` CSS pixels at 320px and 400% zoom. Add Spending remains disabled with distinct no-active-Participant guidance until the first active Participant is committed.
9. The authoritative color control is a labeled normalized `#RRGGBB` text field; the swatch is supplementary and named. Participant identity and lifecycle state are never conveyed by color alone.
10. Validation responses preserve raw name/color, stable guidance/error IDs, `aria-invalid`, and `aria-describedby`; focus lands on the linked alert summary or sole invalid control. The valid submission token remains usable because validation precedes reservation.
11. During a valid mutation, the initiator is unavailable, one scoped polite atomic status and `aria-busy` represent pending work, and the committed response focuses the new Participant row/action and announces success once. The five-destination shell and Manage reading order remain stable.
12. Native server-rendered HTML and forms remain authoritative. Optional pinned HTMX enhancement has identical validation, status, focus, error, success, security, and full-page fallback behavior, with no custom JavaScript, inline script attributes, or second divergent route.
13. All existing authentication/session/CSRF/submission-token behavior, security headers, archived Group read-only behavior, Group settings, navigation, and lifecycle/shutdown behavior continue to work end-to-end.

Requirements: `SPEC-FR30..SPEC-FR34`, `SPEC-FR36`, `SPEC-FR87..SPEC-FR90`; `SPEC-NFR7`, `SPEC-NFR15..SPEC-NFR16`, `SPEC-NFR25`, `SPEC-NFR28..SPEC-NFR34`; UX contracts `UX-SHELL-01`, `UX-TARGET-01`, `UX-FOCUS-01`, `UX-STATUS-01`, `UX-RESPONSIVE-01`, `UX-VISUAL-01`.

## Scope Boundary

- Implement Group-owned Participant creation and active Participant rendering in Manage. Do not implement Participant edit, archive, restore, balance eligibility, archived Participant views, Spending CRUD, summaries, debts, confirmations, or independent deletion; those belong to Stories 2.4, 5.4-5.5, and Epic 3/4/5.
- Resolve the superseded reusable/global Participant model instead of adding another membership layer. Remove global Participant routes/projections and the old “Add existing participant”/membership-reuse controls. Do not retain compatibility shims for contradictory APIs.
- Preserve the Group shell and Add Spending setup state. The first active Group-owned Participant must enable the existing Add Spending affordance, but Spending itself remains outside this story.
- Update `specs/design.md` first if ownership/storage or route behavior changes, then synchronize migrations, README/config examples only if affected, tests, and committed `.sqlx` metadata.

## Tasks / Subtasks

- [x] Establish the target ownership model before implementation (AC: 1, 5, 7, 13)
  - [x] Make the persisted Participant belong to exactly one Group, or implement an equally strong structural one-owner model; do not leave globally reusable participants possible through `group_members`.
  - [x] Decide whether `group_members` is removed or reduced to a strictly one-owner activity/history mechanism. Synchronize Group deletion and future Spending eligibility with that decision.
  - [x] Update normative `specs/design.md` before schema/API changes; remove global identity, membership, and reuse language that conflicts with the target.
- [x] Implement application-owned Participant policy and narrow ports (AC: 3-6)
  - [x] Replace global/reusable creation paths with Group-scoped input and read contracts; retain `*Input`, `*Reader`, `*Repository`, and `*UseCases` naming.
  - [x] Add a side-effect-free validation path for Group existence/active state, trimmed `Name`, and normalized `Color`, analogous to Story 2.2 Group settings validation.
  - [x] Ensure application policy, not web or SQL, constructs `Name`/`Color`, checks ownership/lifecycle, and maps failures to safe `ApplicationError` categories.
  - [x] Keep Participant IDs positive `i64`, identity stable, color uppercase canonical `#RRGGBB`, and no user/membership/tenant concepts.
- [x] Migrate persistence and align downstream eligibility (AC: 4-7, 13)
  - [x] Update participant/group schema, foreign keys, indexes, deletion behavior, and checked SQL to enforce exactly one owning Group and prevent cross-Group reuse.
  - [x] Create Participant and its ownership atomically under the shared five-second write gate; archived/missing Group checks must not leave an orphan row.
  - [x] Keep SQLite structural checks for bounded text, color shape, archive flag, positive IDs, ownership, and restrictive referenced-history behavior; keep Unicode trim/count and all semantic policy in Rust.
  - [x] Update `spendings.rs` eligibility/read paths so later allocations require active, non-archived Participants owned by the Spending Group, not merely a reusable membership row.
  - [x] Update decoding and migration tests for invalid rows, ownership uniqueness, Group deletion, rollback, and cross-Group attempts. Refresh `.sqlx` after checked SQL/migration changes.
- [x] Make Manage the only Participant creation surface (AC: 1, 8-12)
  - [x] Replace the Manage placeholder and legacy member/reuse controls with active Group-owned Participant rows and the protected Add Participant form.
  - [x] Remove/supersede global `/participants` list/create/edit/archive/restore routes and templates; do not leave a divergent alias.
  - [x] Keep the form exactly `name`, `color`, `csrf`, and `submission_token`; use the server suggestion only on a fresh form and submitted raw color on validation rerender.
  - [x] Route valid creation through the existing root-owned Group mutation/lifecycle path or extend the single mutation owner minimally. Do not create a second registry, gate, token store, or direct unsupervised mutation path.
  - [x] Perform archived/missing Group and application validation before `reserve_and_dispatch`; register/mark dispatch immediately before the first state-changing call; publish definitive outcome before response work; advance epoch only after commit.
  - [x] Redirect to `/groups/{id}/manage` after commit and focus the new stable Participant row/action. Keep native fallback and optional HTMX fragment behavior equivalent.
- [x] Implement server color suggestion and Editorial Contrast styling (AC: 2, 8-12)
  - [x] Keep suggestions valid, varied, deterministic enough for tests, and independent of client JavaScript. A process-random UUID seed is not sufficient if it makes behavior untestable; inject or expose a deterministic sequence boundary as appropriate.
  - [x] Use the existing dark Editorial Contrast tokens, square geometry, rules, 48px targets, high-contrast focus, no cards/pills/gradients/animation, and no page-level horizontal overflow.
  - [x] Use a labeled text color field as authority plus a named outlined swatch. Long names must wrap/break without clipping; color must not be the only identity cue.
- [x] Add invariant-owning, web, and composed regression tests (AC: all)
  - [x] Application tests cover trimming, empty/Unicode boundary, invalid color, all Group states, positive returned ID, normalized values, ownership, repository errors, and zero repository call on validation failure.
  - [x] Web tests cover exact field allowlist, retained raw values, stable error/focus/status markup, no global surface, active-only Group rows, archived `409`, missing `404`, `303` Manage redirect, Add Spending enablement, and native/HTMX parity.
  - [x] Hostile-input tests cover malformed UTF-8/percent encoding, missing/duplicate/unknown fields, invalid CSRF, unknown/reused token, unauthenticated access, and oversized body, proving no reservation/dispatch/use-case/repository/gate/epoch side effect where required.
  - [x] Persistence tests cover direct constraint violations, Group ownership, no cross-Group reuse, atomic rollback, active filtering, history-free Group deletion, referenced restrictions, and concurrent lifecycle/create behavior.
  - [x] Use barriers, notifications, held locks, or permits for concurrency; never synchronize with sleeps. Retain the root real-socket smoke and all Story 2.1/2.2 lifecycle/security regressions.
  - [x] Add template/geometry evidence for 320px and 400% zoom: 124px color column, 48px swatch/controls, focus visibility, wrapping, and no page-level horizontal scrolling.

### Review Findings

- [x] [Review][Patch] Remove the remaining unowned/global Participant APIs and projections [debtor-application/src/participants.rs:35-145; debtor-infra/src/db/repos/participants.rs:57-73; debtor-web/src/handlers/spending_views.rs:55-84] — removed global listing/unowned creation, added required immutable Group ownership, and removed the available-participant projection.
- [x] [Review][Patch] Extend the existing root mutation owner instead of duplicating its lifecycle guard [src/composition.rs:57-100,229-277; debtor-web/src/state.rs:17-27] — Participant creation now uses the existing Group mutation executor and guard.
- [x] [Review][Patch] Check Group lifecycle before participant form parsing [debtor-web/src/handlers/memberships.rs:24-34] — moved the writable-Group precheck ahead of route parsing and added missing-Group coverage.
- [x] [Review][Patch] Do not turn a committed insert into a retryable failure during post-commit reload [debtor-infra/src/db/repos/participants.rs:92-104] — the committed Participant is now returned from normalized transaction inputs without a post-commit reload.
- [x] [Review][Patch] Focus the validation alert or sole invalid Participant control [debtor-web/templates/group.html:45-47,87,101-108] — linked alerts are focusable and the sole invalid field receives autofocus.
- [x] [Review][Patch] Add the missing lifecycle, hostile-input, no-dispatch, native/HTMX, persistence, and geometry assertions [debtor-web/src/router.rs:1219-1289; debtor-infra/tests/migrations.rs:542-570] — added lifecycle, ownership, immutable-owner, draft/focus, status, HTMX, and no-dispatch assertions while retaining the existing security/concurrency suite.
- [x] [Review][Defer] Unsupported Group Currency is not represented in the submitted option list [debtor-web/src/handlers/spending_views.rs:175-178; debtor-web/templates/group.html:73-78] — deferred, pre-existing and unrelated to the Participant implementation.

## Dev Notes

### Developer Context

This is a brownfield vertical slice after completed Stories 2.1 and 2.2. The Group shell, Manage settings projection, strict form extractor, CSRF and submission-token stores, five-second SQLite write gate, mutation epoch, root Group mutation executor, native/HTMX conventions, and archived Group read-only behavior already exist and must be extended, not duplicated.

The current Participant implementation is intentionally conflicting scaffold code. `debtor-application/src/participants.rs` exposes global `list_participants`, reusable `create_participant`, `add_member`, and membership activity. `debtor-infra/src/db/repos/participants.rs` creates an unowned row and separately inserts `group_members`; the schema permits one Participant in multiple Groups. `debtor-web/src/handlers/participants.rs`, `templates/participants.html`, and `participant_edit.html` expose a global surface. `debtor-web/src/handlers/memberships.rs` reserves the token before `Name`/`Color` validation and redirects to Summary. `group.html` currently renders old member/reuse forms outside the canonical Manage settings section. These paths must be removed or replaced, not layered over.

The target product contract is one Group-owned accounting identity. Participants are never users, memberships, tenants, authenticated principals, or globally reusable records. This story may require clean pre-release migration/API breakage. Do not preserve old schema/API paths merely for compatibility.

### Current Files To Update And Preserve

| Path | Current state | Required change and preservation |
| --- | --- | --- |
| `specs/design.md` | Normative contract describes Group-owned identities but current scaffold still has reusable membership concepts. | Update first for the chosen physical ownership model, Group-scoped Manage behavior, and any route/schema changes; keep all accounting/history/security invariants. |
| `debtor-domain/src/model.rs` | `Name::new` trims/counts 100 Unicode chars; `Color::new` trims and canonicalizes `#RRGGBB`; `Participant` has positive ID/name/color/archive state but no owner. | Reuse validators. Add owner data only if the chosen domain contract needs it; do not move HTTP/SQL types inward. |
| `debtor-application/src/participants.rs` | Global/reusable Participant and membership ports/service; Group archive check only during dispatched create. | Replace with Group-scoped reader/create policy, side-effect-free validation, safe errors, and exact ownership/lifecycle rules. Do not retain public global creation/reuse APIs. |
| `debtor-infra/src/db/repos/participants.rs` | Global list/create, unowned insert plus membership insert, reusable `add_member`, membership activity. | Implement atomic Group-owned persistence, active filtering, ownership checks, checked queries, and safe zero-row/error mapping under the shared write gate. |
| `migrations/20260517000002_create_participants.up.sql` | Participant has no `group_id`; only name/color/archive constraints. | Add structural ownership or replace obsolete table shape. Keep positive ID, bounded text, color, archive checks. |
| `migrations/20260517000003_create_group_members.up.sql` | Composite Group/Participant membership permits one Participant in many Groups. | Remove or constrain obsolete reusable membership model consistently with the chosen owner schema and future Spending eligibility. Update down migration. |
| `debtor-infra/src/db/repos/decoding.rs` | Decodes Participant and GroupMember persistence projections. | Decode/revalidate any owner fields and canonical color; reject corruption rather than normalize invalid stored data. |
| `debtor-infra/src/db/repos/spendings.rs` | Spending eligibility relies on `group_members`. | Align checks with active, non-archived, single-owner Participants so Story 3 cannot allocate a Participant from another Group. |
| `debtor-application/src/groups.rs` | Group reader/service and update/create policies already exist. | Reuse Group reader and safe archived/not-found semantics; do not add a Participant-specific parallel Group abstraction. |
| `debtor-web/src/forms.rs` | `parse_participant_form` already requires exactly name/color/security fields; generic token boundary precedes route parsing. | Reuse strict structure. Keep raw text in the web projection; application validation must happen before reservation. Do not parse financial or persistence policy in web. |
| `debtor-web/src/handlers/memberships.rs` | Group creation currently lives here, token reservation precedes application validation, old add-member route exists, success redirects to `/groups/{id}`. | Replace with the canonical Group-scoped create flow or remove the obsolete handler. Add archived/missing precheck, validation-before-reservation, supervised dispatch, 303 Manage redirect, and retained validation rendering. |
| `debtor-web/src/handlers/participants.rs` | Global list/create/edit/archive/restore handlers. | Remove global Participant management from the router and templates; later edit/archive stories should add only Group-scoped routes owned by those stories. |
| `debtor-web/src/handlers/groups.rs` | Manage builds canonical Group projection; `require_writable_group` provides archived precheck; Group settings uses root mutation executor. | Preserve Group shell/settings and use the same safe precheck/redirect/error/focus conventions for Participant creation. |
| `debtor-web/src/handlers/spending_views.rs` | Builds members from memberships and globally lists participants for available identities; Manage data is mixed with legacy spending projection. | Build Group-owned active rows and Manage Participant form state without global available/reuse lists. Preserve the current Group settings projection and Add Spending disabled/enabled state. |
| `debtor-web/src/templates.rs` | `GroupTemplate` contains `members`, inactive/available participants, create draft, and legacy expense fields; global Participant templates exist. | Replace only the render projections required for Group-owned active Participants; keep Askama fields explicit and update every use. Remove obsolete global template types if routes are removed. |
| `debtor-web/templates/group.html` | Manage has settings then placeholder; Summary has old member/reuse/create form and legacy spending UI. | Make Manage Participants canonical, show active Group-owned identities and Add form, remove global/reuse controls, retain shell/settings/archived read-only behavior and no-Participant Add Spending guidance. |
| `debtor-web/templates/participants.html`, `participant_edit.html` | Global Participant UI. | Delete/supersede; no global route or divergent compatibility page may remain. |
| `debtor-web/src/router.rs` | Registers global Participant routes, membership routes, and Group Participant POST. | Keep only the smallest Group-scoped creation route needed by Manage; remove global/reuse routes and preserve middleware, body limits, authentication, CSRF, and timeout layers. |
| `debtor-web/src/handlers/test_support.rs` | Fakes record global Participant and membership calls. | Convert fakes to Group ownership, record Group ID/name/color, and expose dispatch/no-dispatch assertions. |
| `src/composition.rs` | Composes one Participant service and one root Group mutation executor. | Wire changed ports and extend the existing single mutation owner only; do not create another registry/gate/executor. |
| `debtor-web/src/participant_color.rs` | Twelve valid colors, UUID-seeded atomic sequence, tests only validity/consecutive difference. | Preserve varied accessible choices but make the sequence deterministic/testable enough for the story contract; submitted values always override fresh suggestions. |
| `debtor-infra/tests/migrations.rs`, `debtor-infra/tests/repos.rs`, `debtor-infra/tests/db.rs` | Tests encode independent Participants then memberships. | Rewrite for owner-at-create, structural constraints, atomicity, deletion/history rules, and no cross-Group reuse. |
| `static/css/app.css` | Shared Editorial Contrast and existing form/member styles. | Extend minimally for Manage Participant identity blocks, 124px color grid, swatch, status/error/focus and 48px targets; no new visual system. |

### Architecture Compliance

- Preserve `debtor -> debtor-web/debtor-infra -> debtor-application -> debtor-domain`. Domain is synchronous and framework-free; application owns Group-scoped input/lifecycle policy and ports; infra owns SQLx, migrations, transactions, write gate, and corruption/constraint mapping; web owns Axum extraction, sessions/CSRF/tokens, Askama projections, accessibility, and HTTP mapping; root owns composition, mutation supervision, epoch, lifecycle, and shutdown.
- Follow AD-4/AD-5: exactly one Group owner per Participant; no identity reuse; application validates Group active state, names, colors, and ownership while infra rechecks race-sensitive facts transactionally.
- Follow AD-6/AD-13: one process-local SQLite runtime/write gate and one mutation registry/executor. Gate timeout starts no transaction or guarded side effect. Epoch advances only after commit.
- Follow AD-10/AD-14: strict extraction, authentication, CSRF, archived/missing precheck, side-effect-free application validation, token reservation immediately before dispatch, then one supervised state-changing call. No generic timeout/cancellation after dispatch.
- Follow AD-11/AD-18: native semantic HTML is authoritative; pinned HTMX 2.0.10 and response-targets 2.0.4 are optional. Use stable server-owned IDs, focus matrix, scoped polite status, `aria-busy`, 48px targets, 320px/400% support, and Editorial Contrast. No custom JavaScript, inline scripts, or script attributes.
- Follow AD-15: map missing/archived/constraint/contention failures to fixed safe categories. Never expose/log SQL, raw errors, IDs, names, colors, tokens, cookies, client identity, or request-derived diagnostics.
- Follow AD-17: never introduce administrator/Participant authentication, user records, memberships as identity, tenants, registration, or multi-user authorization.

### Library / Framework Requirements

- Use the pinned toolchain/dependencies already in `Cargo.lock`: Rust 1.97.1 edition 2024, Axum 0.8.9, Askama 0.16.0, Tokio 1.53.1, SQLx 0.9.0, Tower 0.5.3, tower-http 0.6.11, tower-sessions 0.15.0, and existing HTMX assets. Do not add or upgrade dependencies.
- Keep Askama contexts explicit and render-only. Preserve the existing response/redirect and middleware patterns; handlers remain thin.
- Use compile-time checked `sqlx::query!`/`query_as!` queries. The fixed WAL checkpoint remains the only unchecked SQL exception. Never aggregate money in SQL; this story does not need monetary SQL.
- Do not change framework APIs. Current pinned APIs and project patterns are authoritative; no external library research or migration is needed unless implementation changes a library API.

### Testing Requirements

- Domain tests should reuse `Name::new`/`Color::new` coverage and add only owner-related pure invariants if an owner is added to the domain model.
- Application tests use simple `Mutex` fakes and injected dependencies. Assert trimming, empty/100+ Unicode names, valid/invalid colors, active/archived/missing Groups, normalized values, positive returned ID, exact Group ID, repository errors, and no repository call on validation rejection.
- Web tests use the real router/session/CSRF/submission-token path. Assert exact `name`, `color`, `csrf`, `submission_token` fields; active-only scoped rows; no `/participants` global surface; 422 raw retention; stable associations/focus; one status/`aria-busy`; 303 `/groups/{id}/manage`; new row visibility; Add Spending enablement; and native/HTMX parity.
- Hostile requests must cover malformed percent/UTF-8 encoding, missing/duplicate/unknown fields, invalid CSRF, unknown/reused token, unauthenticated access, oversized body, archived Group, and missing Group. Prove the required no-dispatch/no-reservation/no-use-case/no-repository/no-gate/no-epoch behavior.
- Persistence tests use `#[sqlx::test]` or temporary SQLite files. Cover owner constraints, positive/bounded/color/archive constraints, active filtering, cross-Group rejection, atomic rollback, history-free Group delete cascade, referenced identity restriction, archived/missing Group no-row behavior, and concurrent create/lifecycle races.
- Concurrency tests use barriers, notifications, permits, or deliberately held locks, never timing sleeps. Preserve root real-socket authentication/CSRF, authenticated read, mutation outcome, readiness, and shutdown coverage from Stories 2.1/2.2.
- Validate with `cargo fmt --all -- --check`, `cargo check --workspace --all-features --locked`, `SQLX_OFFLINE=true cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`, `cargo test --workspace --all-features --locked`, and `cargo run --bin architecture-check --locked`. Never use `cargo build --release`.
- If migrations or checked SQL change, migrate a temporary SQLite database and run `SQLX_OFFLINE=false DATABASE_URL=sqlite:///tmp/debtor-sqlx.db?mode=rwc cargo sqlx prepare --workspace --check`; refresh committed `.sqlx` metadata.

### Previous Story Intelligence

Story 2.2 is the direct predecessor. Reuse its validation-before-token-reservation pattern, canonical Manage projection, active/archived rendering, settings fallback, stable heading/error/status IDs, `aria-busy`, saved-state focus, root mutation executor, write-gate/epoch behavior, safe error mapping, and native/HTMX parity. Its review specifically corrected premature reservation, invalid option retention, dangling associations, focusable error targets, narrow-layout overflow, and incomplete persistence/concurrency coverage. Do not regress those fixes.

Story 2.1 established the five-link Group shell, URL-based Group selection, no-Participant disabled Add Spending state, root mutation lifecycle, and real-socket shutdown evidence. The first Participant must change only the setup state that is now legitimately satisfied; do not add persistent current-Group state or Spending implementation.

### Git Intelligence

- Recent history is `e0ed77b feat: implement 2-2 bmad`, preceded by `b46a2f7 feat: impelement bmad 2-1` and Epic 1 lifecycle commits. Build on current APIs, not older story prose.
- The latest Group work touched application policy, infra persistence, Group handlers/projections, router/test support, root composition, and CSS. Expect `GroupTemplate` to contain legacy participant/spending fields while replacing only the conflicting Participant model.
- The worktree was not modified by this context. Do not revert unrelated changes made concurrently.

### Project Structure Notes

- Capability modules remain plural: `groups`, `participants`, `spendings`, `debts`. Use `*Input`, `*Reader`, `*Repository`, `*UseCases`, `Db*`, and `*Template`/`*Row`/`*View` names.
- The likely schema change is intentional pre-release breakage, not a reason to preserve old reusable-membership APIs. If the chosen implementation retains a membership table for activity, prove structurally that a Participant has exactly one Group owner and explain why it is not a reusable identity mechanism.
- Spending eligibility is part of end-to-end correctness even though Spending creation is out of scope. It must not be left able to allocate cross-Group or unowned identities after this story changes persistence.
- Update normative design before behavior, then synchronize migrations, repository SQL, SQLx metadata, tests, and any affected planning/UX artifact. Stop rather than silently interpret divergence between `specs/design.md`, PRD, architecture, and UX contracts.

### References

- [Source: `_bmad-output/planning-artifacts/epics.md#Story 2.3: Add Group-Owned Participants`]
- [Source: `_bmad-output/planning-artifacts/epics.md#Epic 2: Organize Groups and Participants`]
- [Source: `_bmad-output/planning-artifacts/epics.md#Final-Evidence Ledger`]
- [Source: `specs/design.md#Architecture`]
- [Source: `specs/design.md#Accounting And History`]
- [Source: `specs/design.md#Security`]
- [Source: `specs/design.md#Operational Limits`]
- [Source: `specs/adr/0001-foundation-architecture.md#6. Application-owned policy`]
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
- [Source: `_bmad-output/implementation-artifacts/2-2-configure-group-settings.md#Previous Story Intelligence`]
- [Source: `_bmad-output/implementation-artifacts/2-1-create-and-select-a-group.md#Architecture Compliance`]
- [Source: `_bmad-output/project-context.md#Critical Implementation Rules`]
- [Source: `debtor-application/src/participants.rs`]
- [Source: `debtor-infra/src/db/repos/participants.rs`]
- [Source: `debtor-web/src/handlers/memberships.rs`]
- [Source: `debtor-web/src/handlers/participants.rs`]
- [Source: `debtor-web/src/handlers/spending_views.rs`]
- [Source: `debtor-web/templates/group.html`]
- [Source: `debtor-web/src/router.rs`]
- [Source: `migrations/20260517000002_create_participants.up.sql`]
- [Source: `migrations/20260517000003_create_group_members.up.sql`]
- [Source: `debtor-web/src/participant_color.rs`]

## Dev Agent Record

### Agent Model Used

openai/gpt-5.6-luna

### Debug Log References

- Resolved create-story customization: no activation prepend/append steps; persistent fact loaded from `_bmad-output/project-context.md`.
- Read the complete sprint status and selected the first backlog story in order: `2-3-add-group-owned-participants`.
- Loaded the complete Epic 2/Story 2.3 context, normative design, PRD, architecture spine, UX contracts, project context, completed Stories 2.1/2.2, deferred work, current participant/group/web/infra/root code, migrations, and recent Git history.
- Independent audit identified the critical brownfield mismatch: global reusable Participants plus cross-Group `group_members` conflicts with the Group-owned identity contract. It also identified premature token reservation, global routes, legacy Manage placement, nondeterministic UUID-seeded color suggestions, and downstream Spending eligibility dependencies.
- No library API change is authorized; use the pinned project versions and existing Axum/Askama/SQLx patterns.

### Implementation Plan

- Make the existing `group_members` relation structurally single-owner with a unique Participant index, preserving its activity flag for future allocation eligibility and avoiding a parallel ownership table.
- Add application-owned Participant input validation and a root-composed definitive Participant mutation executor that reuses the existing mutation registry, epoch, write gate, and shutdown lifecycle.
- Move Participant creation and active identity rendering into Group Manage, remove global Participant routes/templates and legacy reuse controls, and preserve native/HTMX security and focus behavior.
- Make color suggestions deterministic and server-owned, retain submitted drafts on validation, and add application, migration, web, and full-workspace regression coverage.

### Completion Notes List

- Ultimate context engine analysis completed - comprehensive developer guide created.
- Added Group-scoped Participant input validation, canonical color handling, and definitive root mutation dispatch.
- Enforced single-owner Participant membership at the SQLite boundary and added a second-Group rejection regression.
- Replaced Manage's legacy member/reuse controls with active Participant rows and a protected name/color form; removed global Participant routes and templates.
- Added deterministic server color suggestions, retained validation drafts, stable field associations, committed-row autofocus, and the Manage redirect contract.
- Validation passed: `cargo fmt --all -- --check`, locked workspace tests, offline strict Clippy, architecture fitness, and online SQLx preparation check after migration.
- Addressed all seven code-review patch findings: required immutable ownership, no global/unowned APIs or projections, one root mutation owner, lifecycle-first rejection, committed-result safety, validation focus, and expanded lifecycle/security/persistence coverage.
- Code review outcome: all seven patch findings resolved; one pre-existing unsupported-currency issue deferred.

### File List

- `_bmad-output/implementation-artifacts/2-3-add-group-owned-participants.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `specs/design.md` (expected implementation update before behavior/schema changes)
- `debtor-application/src/lib.rs`
- `debtor-application/src/participants.rs`
- `debtor-application/src/groups.rs`
- `debtor-infra/tests/migrations.rs`
- `debtor-infra/tests/db.rs`
- `debtor-infra/tests/repos.rs`
- `debtor-infra/src/db/repos/participants.rs`
- `migrations/20260517000002_create_participants.up.sql`
- `migrations/20260517000002_create_participants.down.sql`
- `migrations/20260517000003_create_group_members.up.sql`
- `migrations/20260517000003_create_group_members.down.sql`
- `debtor-web/src/forms.rs`
- `debtor-web/src/handlers.rs`
- `debtor-web/src/handlers/memberships.rs`
- `debtor-web/src/handlers/participants.rs` (deleted)
- `debtor-web/src/handlers/groups.rs`
- `debtor-web/src/handlers/spending_views.rs`
- `debtor-web/src/handlers/test_support.rs`
- `debtor-web/src/router.rs`
- `debtor-web/src/templates.rs`
- `debtor-web/templates/group.html`
- `debtor-web/templates/participants.html` (deleted)
- `debtor-web/templates/participant_edit.html` (deleted)
- `debtor-web/src/participant_color.rs`
- `static/css/app.css`
- `src/composition.rs`
- `.sqlx/query-8b0d8689e720b1d221c61beaa48cbeb1651fb8ad5bb405fa4eafe260e7ca4479.json` (deleted)
- `.sqlx/query-908b23984c84c548b74093bad943d8a236484de660b01342788ae4fec2e265a0.json` (deleted)
- `.sqlx/query-de35b37fa8a386b6412ae287f0c718342d3c1df4061363feaa40282bb2cefca7.json` (deleted)
- `.sqlx/query-5292db183204f52377f390b7190eea0f2082e56cf5797f36d0a4bb24492b14b0.json`
- `.sqlx/query-beac35ae5f5631d42b3d74816008c8602e5302d39428d5b71d5fa2ef3e0f5e00.json`
- `_bmad-output/implementation-artifacts/deferred-work.md`

### Change Log

- 2026-08-15: Implemented Group-owned Participant creation, single-owner persistence, scoped Manage UI, deterministic color suggestions, supervised mutation dispatch, and regression coverage; moved story to `review`.
- 2026-08-17: Addressed seven adversarial review findings, retained one pre-existing deferred currency issue, and moved the story to `done`.
