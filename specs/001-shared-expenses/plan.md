# Implementation Plan: Shared Expenses Manager

**Branch**: `001-shared-expenses` | **Date**: 2026-02-23 | **Spec**: `/mnt/d/projects/pet/debtor/specs/001-shared-expenses/spec.md`  
**Input**: Feature specification from `/mnt/d/projects/pet/debtor/specs/001-shared-expenses/spec.md` and user constraints:
- `acton-htmx` Rust crate as the web framework
- `htmx` as the only JavaScript library
- frontend implemented with modern vanilla HTML + CSS
- SQLite as the primary database with `sea-orm` as ORM
- async-first implementation and mandatory TDD workflow

## Summary

Build a single-user, server-rendered shared-expense manager in Rust with
`acton-htmx`, HTMX-driven hypermedia interactions, and SQLite/SeaORM
persistence. The system will support secure authentication, group/member and
expense lifecycle management, multi-currency debt simplification with cached
exchange rates, and a provably optimal debt-settlement projection. Delivery is
strictly test-first (Red-Green-Refactor).

## Technical Context

**Language/Version**: Rust stable 1.78+ (Edition 2024)  
**Primary Dependencies**: `acton-htmx`, `tokio`, `sea-orm`, `sea-orm-migration`, `argon2`, `rust_decimal`, `time`, `uuid`, browser `htmx`  
**Storage**: SQLite (WAL mode) via SeaORM entities and migrations  
**Testing**: `cargo test`, `tokio::test`, integration/contract tests for routes and exchange-rate adapter  
**Target Platform**: Linux container or host on local network; modern desktop/mobile browsers  
**Project Type**: Monolithic web application (server-rendered HTML + HTMX partials)  
**Performance Goals**: Login page to dashboard < 5 seconds; debt summary for 20 members/200 expenses < 3 seconds  
**Constraints**: JS-free frontend except HTMX; semantic HTML + vanilla CSS; single admin account; 30-day sliding sessions; lockout after 5 failed logins for 15 minutes; exchange-rate fetch max once/day per pair; async I/O only; strict TDD  
**Scale/Scope**: Single owner, dozens of groups, up to ~20 members and ~200 expenses per group

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### Pre-Phase 0 Gate Review

| Principle Gate | Status | Plan Alignment |
|----------------|--------|----------------|
| I. JavaScript-Free Frontend | PASS | UI contracts use server-rendered HTML + HTMX only; no custom JS planned. |
| II. Rust Backend (acton-htmx) | PASS | Backend stack is Rust + `acton-htmx`; async runtime via `tokio`. |
| III. Vanilla CSS & Semantic HTML | PASS | UI guidance explicitly limits styling to vanilla CSS and semantic tags. |
| IV. Single-User Secured Access | PASS | Exactly one admin account, Argon2 password hash, server-side sessions, auth-guarded routes. |
| V. Simplicity & Personal-First | PASS | Monolith architecture, minimal dependencies, no unnecessary abstractions. |
| VI. Test-Driven Development | PASS | Test-first workflow is mandatory in quickstart and test strategy. |

**Gate Result (Pre-Phase 0)**: PASS

### Post-Phase 1 Gate Review

| Principle Gate | Status | Design Evidence |
|----------------|--------|-----------------|
| I. JavaScript-Free Frontend | PASS | `contracts/http-interface.md` defines HTMX form/fragment contracts only. |
| II. Rust Backend (acton-htmx) | PASS | `data-model.md` and quickstart use Rust + `acton-htmx` conventions. |
| III. Vanilla CSS & Semantic HTML | PASS | quickstart mandates semantic templates and CSS tokens without frameworks. |
| IV. Single-User Secured Access | PASS | Data model includes single admin, session lifecycle, and lockout state. |
| V. Simplicity & Personal-First | PASS | Chosen schema and routing remain monolithic and feature-focused. |
| VI. Test-Driven Development | PASS | quickstart defines Red-Green-Refactor as release gate. |

**Gate Result (Post-Phase 1)**: PASS

## Project Structure

### Documentation (this feature)

```text
/mnt/d/projects/pet/debtor/specs/001-shared-expenses/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── http-interface.md
│   └── exchange-rate-provider.md
└── tasks.md
```

### Source Code (repository root)

```text
/mnt/d/projects/pet/debtor/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── app/
│   ├── auth/
│   ├── groups/
│   ├── expenses/
│   ├── debts/
│   ├── exchange_rates/
│   ├── db/
│   └── web/
│       ├── handlers/
│       ├── templates/
│       └── components/
├── migrations/
├── static/
│   └── css/
└── tests/
    ├── unit/
    ├── integration/
    └── contract/
```

**Structure Decision**: Single Rust web application. The frontend is not a
separate SPA and is delivered via server-rendered templates and HTMX fragments.
This keeps the architecture aligned with constitution simplicity constraints.

## Complexity Tracking

No constitution violations identified; no complexity exemptions required.
