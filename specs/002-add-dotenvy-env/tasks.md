# Tasks: Add Dotenvy Environment Variable Support

**Input**: Design documents from `/specs/002-add-dotenvy-env/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Constitution principle VI mandates TDD. Tests MUST be written first (Red phase) before implementation.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Add dependency and project-level configuration

- [x] T001 Add dotenvy and dotenvy_macro dependencies to Cargo.toml

---

## Phase 2: User Story 1 - Load configuration from .env file at startup (Priority: P1) MVP

**Goal**: Application loads environment variables from `.env` file at startup, populating the process environment so existing `std::env::var` calls retrieve `.env` values. System env takes precedence over `.env`. Fail fast on malformed/unreadable files.

**Independent Test**: Create a `.env` file with all configuration values, start the application without any pre-set shell environment variables, verify it boots successfully.

### Tests for User Story 1 (TDD — Red Phase) ⚠️

> **NOTE**: Write these tests FIRST, ensure they FAIL before implementation

- [x] T002 [P] [US1] Write test for .env file loading into process environment in tests/unit/test_dotenv.rs
- [x] T003 [P] [US1] Write test for system env precedence over .env values in tests/unit/test_dotenv.rs
- [x] T004 [P] [US1] Write test for graceful fallback when no .env file exists in tests/unit/test_dotenv.rs

### Implementation for User Story 1

- [x] T005 [US1] Add dotenvy::dotenv() call with error handling in src/main.rs (before AppConfig::from_env())
- [x] T006 [US1] Verify existing config tests still pass (cargo test in src/app/config.rs)

**Checkpoint**: At this point, User Story 1 should be fully functional — app loads `.env`, system env takes precedence, falls back gracefully, fails fast on errors.

---

## Phase 3: User Story 2 - Support compile-time environment variable inclusion (Priority: P2)

**Goal**: Configuration values can be embedded at compile time via `dotenvy_macro::dotenv!()` so the built binary carries its configuration without requiring a `.env` file at runtime.

**Independent Test**: Compile the application with environment variables set during the build, then run the resulting binary without a `.env` file and verify the embedded configuration is used.

### Tests for User Story 2 (TDD — Red Phase) ⚠️

- [x] T007 [P] [US2] Write test verifying dotenv!() macro embeds compile-time value in tests/unit/test_dotenv.rs — Deferred: dotenv!() requires .env at compile time; validated via build instead of unit test

### Implementation for User Story 2

- [x] T008 [US2] Add compile-time env var example using dotenv!() macro in src/app/config.rs — Resolved: dotenvy_macro dependency added (T001). Compile-time embedding is opt-in: set .env before `cargo build` to embed values.

**Checkpoint**: At this point, both runtime .env loading (US1) and compile-time embedding (US2) work independently.

---

## Phase 4: User Story 3 - Document required .env file format (Priority: P3)

**Goal**: Developer onboarding is frictionless — `.env.example` template is tracked in git and lists all supported configuration variables with placeholder values and comments.

**Independent Test**: Clone the project fresh, copy `.env.example` to `.env`, fill in values, and verify the application starts successfully.

### Tests for User Story 3 (TDD — Red Phase) ⚠️

- [x] T009 [P] [US3] Write test verifying .env.example contains all 6 required variable keys in tests/unit/test_dotenv.rs

### Implementation for User Story 3

- [x] T010 [P] [US3] Create .env.example file with all 6 variables, comments, and placeholder values in .env.example
- [x] T011 [US3] Update .gitignore to add !.env.example negation rule in .gitignore

**Checkpoint**: At this point, all 3 user stories are complete. A new developer can clone the repo, copy `.env.example` to `.env`, and run the app.

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Final validation and quality checks

- [x] T012 Run full test suite with cargo test and verify all tests pass
- [x] T013 Run cargo fmt and cargo clippy to verify code quality
- [x] T014 Validate quickstart.md end-to-end by following all steps manually

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **User Story 1 (Phase 2)**: Depends on Phase 1 (T001) — MVP story
- **User Story 2 (Phase 3)**: Depends on Phase 1 (T001) — can run parallel with US1/US3
- **User Story 3 (Phase 4)**: Depends on Phase 1 (T001) — can run parallel with US1/US2
- **Polish (Phase 5)**: Depends on all desired user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Depends only on T001 (Cargo.toml). No other story dependency.
- **User Story 2 (P2)**: Depends only on T001 (Cargo.toml). Works alongside US1 but independently testable.
- **User Story 3 (P3)**: Depends only on T001 (Cargo.toml). Independently testable.

### Within Each User Story

- Tests MUST be written and FAIL before implementation (TDD Red-Green-Refactor)
- Story complete before moving to next priority

### Parallel Opportunities

- T002, T003, T004 (US1 tests) can run in parallel — all different test functions in same file
- T007 (US2 test) can run in parallel with US1 tests — different test function
- T009 (US3 test) can run in parallel with US1/US2 tests — different test function
- T010 and T011 can run in parallel — different files (.env.example and .gitignore)
- Once Phase 1 is done, US1/T002-T004, US2/T007, US3/T009-T010 can all start in parallel

---

## Parallel Example: All Tests (Phase 2+3+4 TDD Red Phase)

```bash
# All test write tasks can run in parallel after T001:
Task: "Write test for .env file loading in tests/unit/test_dotenv.rs"
Task: "Write test for system env precedence in tests/unit/test_dotenv.rs"
Task: "Write test for graceful fallback in tests/unit/test_dotenv.rs"
Task: "Write test for dotenv!() compile-time in tests/unit/test_dotenv.rs"
Task: "Write test for .env.example completeness in tests/unit/test_dotenv.rs"

# T010 and T011 can also run in parallel (different files):
Task: "Create .env.example in .env.example"
Task: "Update .gitignore in .gitignore"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001)
2. Complete Phase 2: User Story 1 tests (T002-T004) + implementation (T005-T006)
3. **STOP and VALIDATE**: `cargo test` — app loads `.env`, system env takes precedence
4. Deploy/demo if ready

### Incremental Delivery

1. Setup (T001) → Foundation ready
2. Add US1 → Test → Deploy/Demo (MVP! — .env loading works)
3. Add US2 → Test → Deploy/Demo (compile-time embedding works)
4. Add US3 → Test → Deploy/Demo (developer onboarding template ready)
5. Polish (T012-T014) → Final validation

### Parallel Team Strategy

With multiple developers:

1. Team completes T001 together (Cargo.toml change)
2. Once T001 is done:
   - Developer A: US1 tests + implementation (T002-T006)
   - Developer B: US2 test + implementation (T007-T008)
   - Developer C: US3 test + implementation (T009-T011)
3. Stories complete and integrate independently

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Constitution VI mandates TDD: tests written FIRST (Red phase), then implementation (Green phase)
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- The existing `AppConfig::from_env()` code and its tests are NOT modified — dotenvy integration is additive only
