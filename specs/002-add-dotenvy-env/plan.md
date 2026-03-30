# Implementation Plan: Add Dotenvy Environment Variable Support

**Branch**: `002-add-dotenvy-env` | **Date**: 2026-03-29 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/002-add-dotenvy-env/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

Add `dotenvy` crate integration to load environment variables from a `.env` file at application startup, with optional compile-time embedding via `dotenvy_macro`. The existing `AppConfig::from_env()` pattern and all `std::env::var` calls remain unchanged — `dotenvy::dotenv()` populates the process environment before config reads. Includes `.env.example` template, `.gitignore` update, and fail-fast error handling for malformed/unreadable `.env` files.

## Technical Context

**Language/Version**: Rust 1.94.1 (Edition 2024)
**Primary Dependencies**: dotenvy (runtime .env loading), dotenvy_macro (compile-time embedding), acton-htmx, sea-orm, tokio
**Storage**: SQLite (existing, unchanged)
**Testing**: cargo test (built-in), TDD (mandatory per constitution VI)
**Target Platform**: Linux (personal/local-use web service)
**Project Type**: Web service (server-rendered HTMX application)
**Performance Goals**: N/A — config loading is a one-time startup operation
**Constraints**: Simplicity-first (YAGNI principle V), no JavaScript (principle I), existing `AppConfig::from_env()` pattern must be preserved
**Scale/Scope**: Single-user personal project, 6 environment variables, 1 new dependency pair (dotenvy + dotenvy_macro)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. JavaScript-Free Frontend | PASS | No frontend changes; config loading is backend-only |
| II. Rust Backend (acton-htmx) | PASS | Adding standard Rust crate (dotenvy) to existing backend |
| III. Vanilla CSS & Semantic HTML | PASS | No CSS/HTML changes |
| IV. Single-User Secured Access | PASS | Config mechanism unchanged; secrets remain hashed |
| V. Simplicity & Personal-First | PASS | Minimal dependency addition; no abstractions added |
| VI. Test-Driven Development | PASS | Tests will be written before implementation (Red-Green-Refactor) |

**Gate Result**: PASS — No violations. All principles satisfied.

## Project Structure

### Documentation (this feature)

```text
specs/002-add-dotenvy-env/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
src/
├── app/
│   ├── config.rs        # MODIFY — add dotenvy integration before env::var calls
│   ├── mod.rs
│   └── state.rs
├── auth/
├── db/
├── debts/
├── exchange_rates/
├── expenses/
├── groups/
├── web/
├── lib.rs
└── main.rs              # MODIFY — call dotenvy::dotenv() at startup

tests/
├── contract.rs
├── contract/
├── integration.rs
├── integration/
├── unit.rs
├── unit/
└── support/

Cargo.toml               # MODIFY — add dotenvy + dotenvy_macro dependencies
.gitignore               # MODIFY — add !.env.example negation rule
.env.example             # CREATE — template with all config variables
```

**Structure Decision**: Single project (Option 1). Changes are localized to existing files (`config.rs`, `main.rs`, `Cargo.toml`, `.gitignore`) plus one new file (`.env.example`). No new modules or directories needed.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

*No violations — section not applicable.*
