# Tasks: Shared Expenses Manager

**Input**: Design documents from `/mnt/d/projects/pet/debtor/specs/001-shared-expenses/`  
**Prerequisites**: `/mnt/d/projects/pet/debtor/specs/001-shared-expenses/plan.md`, `/mnt/d/projects/pet/debtor/specs/001-shared-expenses/spec.md`, `/mnt/d/projects/pet/debtor/specs/001-shared-expenses/research.md`, `/mnt/d/projects/pet/debtor/specs/001-shared-expenses/data-model.md`, `/mnt/d/projects/pet/debtor/specs/001-shared-expenses/contracts/`

**Tests**: Mandatory. TDD is required by the constitution and by feature constraints (Red-Green-Refactor for every phase, including foundational work).

**Organization**: Tasks are grouped by phase and by user story so each story can be implemented and verified independently.

## Format: `[ID] [P?] [Story] Description`

- `[P]` means the task can be executed in parallel with other `[P]` tasks (no file conflicts, no unmet dependencies).
- `[Story]` labels are used only in user-story phases (`[US1]`..`[US5]`).
- Every task includes explicit file path(s).

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Initialize Rust web app structure and baseline frontend assets.

- [X] T001 Create crate/module skeleton in `/mnt/d/projects/pet/debtor/src/main.rs` and `/mnt/d/projects/pet/debtor/src/app/mod.rs`
- [X] T002 Add core dependencies and crate features in `/mnt/d/projects/pet/debtor/Cargo.toml`
- [X] T003 [P] Implement environment configuration loader in `/mnt/d/projects/pet/debtor/src/app/config.rs`
- [X] T004 [P] Implement app bootstrap wiring in `/mnt/d/projects/pet/debtor/src/app/state.rs`
- [X] T005 [P] Create semantic base layout template with HTMX import in `/mnt/d/projects/pet/debtor/src/web/templates/layout.html`
- [X] T006 [P] Create CSS design tokens and responsive baseline styles in `/mnt/d/projects/pet/debtor/static/css/app.css`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Build shared persistence, routing, middleware, and security primitives required by all stories.

**⚠️ CRITICAL**: User-story implementation starts only after this phase is complete. Tests in this phase MUST be written first and fail first.

### Tests for Foundational Phase (write first, must fail first)

- [X] T007 [P] Add DB bootstrap and migration smoke test in `/mnt/d/projects/pet/debtor/tests/integration/test_db_bootstrap.rs`
- [X] T008 [P] Add foundational unauthenticated-redirect matrix test for protected routes in `/mnt/d/projects/pet/debtor/tests/integration/test_foundation_auth_redirects.rs`
- [X] T009 [P] Add session-cookie policy unit test (HttpOnly/SameSite/Secure behavior) in `/mnt/d/projects/pet/debtor/tests/unit/test_session_cookie_policy.rs`
- [X] T010 [P] Add CSRF middleware contract test for state-changing form routes in `/mnt/d/projects/pet/debtor/tests/integration/test_csrf_contract.rs`

### Implementation for Foundational Phase

- [X] T011 Create initial SeaORM migration schema in `/mnt/d/projects/pet/debtor/migrations/src/m20260223_000001_init_schema.rs`
- [X] T012 Register migrator modules in `/mnt/d/projects/pet/debtor/migrations/src/lib.rs` and `/mnt/d/projects/pet/debtor/migrations/src/main.rs`
- [X] T013 [P] Create auth/session entity models in `/mnt/d/projects/pet/debtor/src/db/entities/admin_users.rs`, `/mnt/d/projects/pet/debtor/src/db/entities/auth_state.rs`, and `/mnt/d/projects/pet/debtor/src/db/entities/sessions.rs`
- [X] T014 [P] Create domain entity models in `/mnt/d/projects/pet/debtor/src/db/entities/groups.rs`, `/mnt/d/projects/pet/debtor/src/db/entities/members.rs`, `/mnt/d/projects/pet/debtor/src/db/entities/expenses.rs`, `/mnt/d/projects/pet/debtor/src/db/entities/expense_shares.rs`, and `/mnt/d/projects/pet/debtor/src/db/entities/exchange_rates.rs`
- [X] T015 Wire entity exports and relations in `/mnt/d/projects/pet/debtor/src/db/entities/mod.rs`
- [X] T016 Implement async SQLite connection and WAL settings in `/mnt/d/projects/pet/debtor/src/db/connection.rs`
- [X] T017 Implement DB bootstrap + startup migrations in `/mnt/d/projects/pet/debtor/src/db/bootstrap.rs`
- [X] T018 [P] Implement shared application error mapping in `/mnt/d/projects/pet/debtor/src/web/error.rs`
- [X] T019 [P] Build reusable test fixtures/seeding helpers in `/mnt/d/projects/pet/debtor/tests/support/mod.rs`
- [X] T020 Implement CSRF token generation/verification middleware in `/mnt/d/projects/pet/debtor/src/web/csrf.rs` and `/mnt/d/projects/pet/debtor/src/web/router.rs`
- [X] T021 Implement auth guard middleware and session extraction in `/mnt/d/projects/pet/debtor/src/auth/middleware.rs`
- [X] T022 Implement base router with health/public/protected groups in `/mnt/d/projects/pet/debtor/src/web/router.rs`
- [X] T023 Implement secure session-cookie defaults and policy wiring in `/mnt/d/projects/pet/debtor/src/auth/session_repo.rs` and `/mnt/d/projects/pet/debtor/src/app/config.rs`
- [X] T024 Add foundational routing smoke tests in `/mnt/d/projects/pet/debtor/tests/integration/test_foundation_routing.rs`

**Checkpoint**: Foundation complete; story phases can proceed.

---

## Phase 3: User Story 1 - Secure Access (Priority: P1) 🎯 MVP

**Goal**: Enforce single-user authentication with lockout, session sliding expiry, secure session cookies, and logout invalidation.

**Independent Test**: Unauthenticated request redirects to `/login`; valid login reaches dashboard; invalid login stays on login; lockout triggers after five failures; logout revokes session.

### Tests for User Story 1 (write first, must fail first)

- [X] T025 [P] [US1] Add auth route contract tests (including no registration endpoint) in `/mnt/d/projects/pet/debtor/tests/contract/test_auth_contract.rs`
- [X] T026 [P] [US1] Add successful login integration test in `/mnt/d/projects/pet/debtor/tests/integration/test_login_success.rs`
- [X] T027 [P] [US1] Add invalid credential integration test in `/mnt/d/projects/pet/debtor/tests/integration/test_login_failure.rs`
- [X] T028 [P] [US1] Add lockout-after-five-failures integration test in `/mnt/d/projects/pet/debtor/tests/integration/test_login_lockout.rs`
- [X] T029 [P] [US1] Add logout revocation integration test in `/mnt/d/projects/pet/debtor/tests/integration/test_logout.rs`
- [X] T030 [P] [US1] Add sliding-session-expiry extension integration test in `/mnt/d/projects/pet/debtor/tests/integration/test_session_sliding_expiry.rs`
- [X] T031 [P] [US1] Add expired-session rejection integration test in `/mnt/d/projects/pet/debtor/tests/integration/test_session_expiry_boundary.rs`
- [X] T032 [P] [US1] Add login CSRF validation integration test in `/mnt/d/projects/pet/debtor/tests/integration/test_login_csrf.rs`

### Implementation for User Story 1

- [X] T033 [P] [US1] Implement Argon2 password verification helpers in `/mnt/d/projects/pet/debtor/src/auth/password.rs`
- [X] T034 [P] [US1] Implement failed-attempt and lockout persistence in `/mnt/d/projects/pet/debtor/src/auth/auth_state_repo.rs`
- [X] T035 [P] [US1] Implement session persistence with sliding expiry and immediate revocation in `/mnt/d/projects/pet/debtor/src/auth/session_repo.rs`
- [X] T036 [US1] Implement login/logout orchestration service in `/mnt/d/projects/pet/debtor/src/auth/login_service.rs`
- [X] T037 [US1] Implement login/logout HTTP handlers in `/mnt/d/projects/pet/debtor/src/web/handlers/auth_handlers.rs`
- [X] T038 [US1] Create login template with invalid/lockout/CSRF states in `/mnt/d/projects/pet/debtor/src/web/templates/auth/login.html`
- [X] T039 [US1] Wire auth handlers and dashboard landing handler in `/mnt/d/projects/pet/debtor/src/web/router.rs` and `/mnt/d/projects/pet/debtor/src/web/handlers/dashboard_handler.rs`
- [X] T040 [US1] Implement single-admin bootstrap flow with registration disabled in `/mnt/d/projects/pet/debtor/src/db/bootstrap.rs` and `/mnt/d/projects/pet/debtor/src/web/router.rs`

**Checkpoint**: US1 is independently functional and testable (MVP candidate).

---

## Phase 4: User Story 2 - Group Management (Priority: P1)

**Goal**: Manage groups and members (create/rename/delete groups, add/rename/remove members, update target currency).

**Independent Test**: Create group, manage member lifecycle, update currency, and delete group without creating expenses.

### Tests for User Story 2 (write first, must fail first)

- [X] T041 [P] [US2] Add group endpoint contract tests in `/mnt/d/projects/pet/debtor/tests/contract/test_groups_contract.rs`
- [X] T042 [P] [US2] Add member endpoint contract tests in `/mnt/d/projects/pet/debtor/tests/contract/test_members_contract.rs`
- [X] T043 [P] [US2] Add create/list group integration test in `/mnt/d/projects/pet/debtor/tests/integration/test_groups_create.rs`
- [X] T044 [P] [US2] Add member CRUD + inactive-history integration test in `/mnt/d/projects/pet/debtor/tests/integration/test_members_crud.rs`
- [X] T045 [P] [US2] Add currency-change same-response debt-refresh integration test in `/mnt/d/projects/pet/debtor/tests/integration/test_groups_currency_recalc.rs`
- [X] T046 [P] [US2] Add group-delete flow integration test in `/mnt/d/projects/pet/debtor/tests/integration/test_groups_update_delete.rs`

### Implementation for User Story 2

- [X] T047 [P] [US2] Implement group repository operations in `/mnt/d/projects/pet/debtor/src/groups/group_repo.rs`
- [X] T048 [P] [US2] Implement member repository with soft-delete in `/mnt/d/projects/pet/debtor/src/groups/member_repo.rs`
- [X] T049 [US2] Implement group validation/service logic in `/mnt/d/projects/pet/debtor/src/groups/group_service.rs`
- [X] T050 [US2] Implement member validation/service logic in `/mnt/d/projects/pet/debtor/src/groups/member_service.rs`
- [X] T051 [US2] Implement group/member handlers in `/mnt/d/projects/pet/debtor/src/web/handlers/group_handlers.rs`
- [X] T052 [US2] Create group/member templates and fragments in `/mnt/d/projects/pet/debtor/src/web/templates/groups/detail.html` and `/mnt/d/projects/pet/debtor/src/web/templates/groups/partials/member_list.html`
- [X] T053 [US2] Wire group/member routes and currency-change debt refresh trigger in `/mnt/d/projects/pet/debtor/src/web/router.rs` and `/mnt/d/projects/pet/debtor/src/web/templates/groups/detail.html`

**Checkpoint**: US2 works independently with authentication and no expense dependencies.

---

## Phase 5: User Story 3 - Expense Recording (Priority: P1)

**Goal**: Record/edit/delete expenses with payer, participants, optional note, and mixed share modes.

**Independent Test**: Add an expense to a populated group and verify it appears correctly; edit and delete operations recalculate dependent views.

### Tests for User Story 3 (write first, must fail first)

- [X] T054 [P] [US3] Add unit tests for share normalization modes in `/mnt/d/projects/pet/debtor/tests/unit/test_share_splitter.rs`
- [X] T055 [P] [US3] Add expense endpoint contract tests in `/mnt/d/projects/pet/debtor/tests/contract/test_expenses_contract.rs`
- [X] T056 [P] [US3] Add expense creation integration test in `/mnt/d/projects/pet/debtor/tests/integration/test_expenses_create.rs`
- [X] T057 [P] [US3] Add expense edit integration test in `/mnt/d/projects/pet/debtor/tests/integration/test_expenses_update.rs`
- [X] T058 [P] [US3] Add expense delete integration test in `/mnt/d/projects/pet/debtor/tests/integration/test_expenses_delete.rs`
- [X] T059 [P] [US3] Add no-settlement behavior regression test (no pay/settle route or action) in `/mnt/d/projects/pet/debtor/tests/contract/test_no_settlement_contract.rs`
- [X] T060 [P] [US3] Add expense-entry duration budget integration test for SC-002 in `/mnt/d/projects/pet/debtor/tests/integration/test_expense_entry_performance.rs`

### Implementation for User Story 3

- [X] T061 [P] [US3] Implement expense repository with payer/group checks in `/mnt/d/projects/pet/debtor/src/expenses/expense_repo.rs`
- [X] T062 [P] [US3] Implement expense share repository in `/mnt/d/projects/pet/debtor/src/expenses/share_repo.rs`
- [X] T063 [US3] Implement share splitting/validation engine in `/mnt/d/projects/pet/debtor/src/expenses/share_splitter.rs`
- [X] T064 [US3] Implement expense service orchestration in `/mnt/d/projects/pet/debtor/src/expenses/expense_service.rs`
- [X] T065 [US3] Implement expense handlers in `/mnt/d/projects/pet/debtor/src/web/handlers/expense_handlers.rs`
- [X] T066 [US3] Create expense forms/list templates and fragments in `/mnt/d/projects/pet/debtor/src/web/templates/expenses/list.html` and `/mnt/d/projects/pet/debtor/src/web/templates/expenses/partials/expense_form.html`
- [X] T067 [US3] Enforce no-settlement route/action surface in `/mnt/d/projects/pet/debtor/src/web/router.rs` and `/mnt/d/projects/pet/debtor/src/web/templates/expenses/list.html`

**Checkpoint**: US3 is independently functional for complete expense lifecycle flows.

---

## Phase 6: User Story 4 - Debt Summary & Smart Recalculation (Priority: P2)

**Goal**: Compute and display the minimal set of debt transfers per group in target currency.

**Independent Test**: For cross-connected expenses, summary renders minimal transfer set and shows "no outstanding debts" when balances are zero.

### Tests for User Story 4 (write first, must fail first)

- [X] T068 [P] [US4] Add unit tests for balance aggregation in `/mnt/d/projects/pet/debtor/tests/unit/test_balance_calculator.rs`
- [X] T069 [P] [US4] Add unit tests proving minimal subset-DP simplification in `/mnt/d/projects/pet/debtor/tests/unit/test_debt_simplify.rs`
- [X] T070 [P] [US4] Add debt summary contract test in `/mnt/d/projects/pet/debtor/tests/contract/test_debts_contract.rs`
- [X] T071 [P] [US4] Add debt summary integration test in `/mnt/d/projects/pet/debtor/tests/integration/test_debt_summary.rs`
- [X] T072 [P] [US4] Add zero-balance "no outstanding debts" integration test in `/mnt/d/projects/pet/debtor/tests/integration/test_debt_summary_zero_balances.rs`
- [X] T073 [P] [US4] Add deterministic ordering and rounding integration test in `/mnt/d/projects/pet/debtor/tests/integration/test_debt_summary_rounding_order.rs`

### Implementation for User Story 4

- [X] T074 [P] [US4] Implement balance aggregation logic in `/mnt/d/projects/pet/debtor/src/debts/balance_calculator.rs`
- [X] T075 [P] [US4] Implement exact subset-DP simplification algorithm in `/mnt/d/projects/pet/debtor/src/debts/simplify.rs`
- [X] T076 [US4] Implement debt summary service (rounding + deterministic ordering) in `/mnt/d/projects/pet/debtor/src/debts/debt_summary_service.rs`
- [X] T077 [US4] Implement debt summary handler and response models in `/mnt/d/projects/pet/debtor/src/web/handlers/debt_handlers.rs`
- [X] T078 [US4] Create debt summary templates/fragments in `/mnt/d/projects/pet/debtor/src/web/templates/debts/summary.html`
- [X] T079 [US4] Wire debt summary route and refresh integration points in `/mnt/d/projects/pet/debtor/src/web/router.rs` and `/mnt/d/projects/pet/debtor/src/web/handlers/expense_handlers.rs`

**Checkpoint**: US4 produces minimal debt transactions from existing stored data.

---

## Phase 7: User Story 5 - Multi-Currency & Exchange Rates (Priority: P2)

**Goal**: Fetch/cache exchange rates lazily and apply stale-cache fallback policy for debt conversion.

**Independent Test**: Multi-currency debt summary fetches rates on demand, reuses same-day cache, and falls back to stale cache with warning when provider fails.

### Tests for User Story 5 (write first, must fail first)

- [X] T080 [P] [US5] Add Frankfurter provider contract tests in `/mnt/d/projects/pet/debtor/tests/contract/test_exchange_provider_contract.rs`
- [X] T081 [P] [US5] Add stale-or-missing cache fetch integration test in `/mnt/d/projects/pet/debtor/tests/integration/test_exchange_fetch_on_demand.rs`
- [X] T082 [P] [US5] Add same-day cache reuse integration test in `/mnt/d/projects/pet/debtor/tests/integration/test_exchange_cache_reuse.rs`
- [X] T083 [P] [US5] Add fallback warning with existing cache integration test in `/mnt/d/projects/pet/debtor/tests/integration/test_exchange_fallback.rs`
- [X] T084 [P] [US5] Add no-cache provider-failure hard-error integration test in `/mnt/d/projects/pet/debtor/tests/integration/test_exchange_no_cache_failure.rs`
- [X] T085 [P] [US5] Add exchange-cache day-rollover boundary test in `/mnt/d/projects/pet/debtor/tests/integration/test_exchange_cache_day_rollover.rs`

### Implementation for User Story 5

- [X] T086 [P] [US5] Implement async Frankfurter client adapter in `/mnt/d/projects/pet/debtor/src/exchange_rates/frankfurter_client.rs`
- [X] T087 [P] [US5] Implement exchange-rate repository and cache lookups in `/mnt/d/projects/pet/debtor/src/exchange_rates/rate_repo.rs`
- [X] T088 [US5] Implement exchange-rate service (lazy refresh, reuse, fallback) in `/mnt/d/projects/pet/debtor/src/exchange_rates/rate_service.rs`
- [X] T089 [US5] Integrate conversion workflow into debt summary service in `/mnt/d/projects/pet/debtor/src/debts/debt_summary_service.rs`
- [X] T090 [US5] Surface stale warnings and conversion errors in `/mnt/d/projects/pet/debtor/src/web/handlers/debt_handlers.rs` and `/mnt/d/projects/pet/debtor/src/web/templates/debts/summary.html`
- [X] T091 [US5] Wire exchange-rate configuration/provider registration in `/mnt/d/projects/pet/debtor/src/app/config.rs` and `/mnt/d/projects/pet/debtor/src/app/state.rs`

**Checkpoint**: US5 delivers complete multi-currency behavior with robust fallback semantics.

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Harden quality, performance, and documentation across all stories.

- [X] T092 [P] Add login-to-dashboard performance integration test for SC-001 in `/mnt/d/projects/pet/debtor/tests/integration/test_login_performance.rs`
- [X] T093 [P] Add performance integration test for 20 members/200 expenses summary budget (SC-003) in `/mnt/d/projects/pet/debtor/tests/integration/test_summary_performance.rs`
- [X] T094 [P] Add end-to-end regression test (login -> group -> expense -> debt summary) in `/mnt/d/projects/pet/debtor/tests/integration/test_end_to_end_journey.rs`
- [X] T095 [P] Add HTMX-only frontend compliance test in `/mnt/d/projects/pet/debtor/tests/integration/test_no_custom_js.rs`
- [X] T096 [P] Add semantic HTML smoke checks for key templates in `/mnt/d/projects/pet/debtor/tests/integration/test_semantic_html.rs`
- [X] T097 [P] Add authenticated-route redirect completeness test for SC-006 in `/mnt/d/projects/pet/debtor/tests/integration/test_auth_redirect_matrix.rs`
- [X] T098 [P] Add exchange fetch-frequency integration test for SC-004 in `/mnt/d/projects/pet/debtor/tests/integration/test_exchange_fetch_frequency.rs`
- [X] T099 [P] Add cookie-security and CSRF-rotation regression tests in `/mnt/d/projects/pet/debtor/tests/integration/test_security_headers_and_csrf.rs`
- [X] T100 [P] Optimize debt summary hot path for SC-003 target in `/mnt/d/projects/pet/debtor/src/debts/debt_summary_service.rs` and `/mnt/d/projects/pet/debtor/src/expenses/expense_repo.rs`
- [X] T101 Update executable runbook for local setup/test in `/mnt/d/projects/pet/debtor/specs/001-shared-expenses/quickstart.md`
- [X] T102 Record final cross-story validation checklist updates in `/mnt/d/projects/pet/debtor/specs/001-shared-expenses/checklists/requirements.md`

---

## Dependencies & Execution Order

### Phase Dependencies

- Setup (Phase 1): no dependencies.
- Foundational (Phase 2): depends on Phase 1 and blocks all story phases.
- User Story phases (Phase 3-7): start after Phase 2.
- Polish (Phase 8): depends on completed stories.

### User Story Dependencies

- US1 (Secure Access): depends on Foundational only.
- US2 (Group Management): depends on US1 for protected access flow.
- US3 (Expense Recording): depends on US2 entities/services.
- US4 (Debt Summary): depends on US3 expense/share data model.
- US5 (Multi-Currency): depends on US4 summary pipeline integration points.

### Suggested Completion Order

1. Phase 1 -> Phase 2
2. US1 (MVP)
3. US2
4. US3
5. US4
6. US5
7. Phase 8 polish

### Within Each Story

- Write tests first and confirm they fail (Red).
- Implement minimum code to pass (Green).
- Refactor safely with tests still passing (Refactor).

---

## Parallel Opportunities

### Setup

- Run `T003`, `T004`, `T005`, and `T006` in parallel after `T001` and `T002`.

### Foundational

- Run tests `T007`-`T010` in parallel.
- Run implementation tasks `T013`, `T014`, `T018`, and `T019` in parallel after `T011` and `T012`.

### User Story 1

- Run tests `T025`-`T032` in parallel.
- Run repository/helper tasks `T033`-`T035` in parallel.

### User Story 2

- Run tests `T041`-`T046` in parallel.
- Run repository tasks `T047` and `T048` in parallel.

### User Story 3

- Run tests `T054`-`T060` in parallel.
- Run repository tasks `T061` and `T062` in parallel.

### User Story 4

- Run tests `T068`-`T073` in parallel.
- Run algorithm tasks `T074` and `T075` in parallel.

### User Story 5

- Run tests `T080`-`T085` in parallel.
- Run adapter/repository tasks `T086` and `T087` in parallel.

---

## Parallel Example Commands by Story

### US1

```bash
Task: "T025 [US1] tests/contract/test_auth_contract.rs"
Task: "T030 [US1] tests/integration/test_session_sliding_expiry.rs"
Task: "T032 [US1] tests/integration/test_login_csrf.rs"
```

### US2

```bash
Task: "T041 [US2] tests/contract/test_groups_contract.rs"
Task: "T042 [US2] tests/contract/test_members_contract.rs"
Task: "T045 [US2] tests/integration/test_groups_currency_recalc.rs"
```

### US3

```bash
Task: "T054 [US3] tests/unit/test_share_splitter.rs"
Task: "T055 [US3] tests/contract/test_expenses_contract.rs"
Task: "T059 [US3] tests/contract/test_no_settlement_contract.rs"
```

### US4

```bash
Task: "T068 [US4] tests/unit/test_balance_calculator.rs"
Task: "T069 [US4] tests/unit/test_debt_simplify.rs"
Task: "T070 [US4] tests/contract/test_debts_contract.rs"
```

### US5

```bash
Task: "T080 [US5] tests/contract/test_exchange_provider_contract.rs"
Task: "T082 [US5] tests/integration/test_exchange_cache_reuse.rs"
Task: "T085 [US5] tests/integration/test_exchange_cache_day_rollover.rs"
```

---

## Implementation Strategy

### MVP First (US1)

1. Complete Phase 1 and Phase 2.
2. Complete US1 tasks (`T025`-`T040`).
3. Validate US1 independently against its acceptance scenarios.
4. Freeze and demo MVP before expanding scope.

### Incremental Delivery

1. Add US2 after US1 passes.
2. Add US3 after US2 passes.
3. Add US4 after US3 passes.
4. Add US5 after US4 passes.
5. Execute Phase 8 cross-cutting hardening.

### Parallel Team Strategy

1. Team aligns on Phases 1-2 together.
2. After foundation, parallelize test-writing tasks within each story (`[P]` tasks).
3. Keep implementation sequence aligned with story dependencies to avoid merge conflicts.
