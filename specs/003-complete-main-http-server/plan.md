# Implementation Plan: Complete Main Function with HTTP Server

**Branch**: `003-complete-main-http-server` | **Date**: 2026-04-12 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/003-complete-main-http-server/spec.md`

## Summary

The application initializes all domain logic (handlers, routes, auth, CSRF, templates, database) but exits immediately because `main()` does not start an HTTP server. This plan wires the existing components to an Axum-based HTTP transport layer so the application binds to a configurable address, routes requests to existing handlers, applies auth/CSRF middleware, serves static assets and rendered HTML templates, and shuts down gracefully on termination signals.

## Technical Context

**Language/Version**: Rust 1.94.1 (Edition 2024)  
**Primary Dependencies**: Axum 0.8 (HTTP framework, via acton-dx's underlying stack), SeaORM 1.x (ORM), tokio 1.x (async runtime), minijinja (template engine — see research.md)  
**Storage**: SQLite via SeaORM (existing)  
**Testing**: `cargo test` — Rust built-in test framework with TDD (mandatory per constitution)  
**Target Platform**: Linux server (single-user, local network)  
**Project Type**: Web service (server-rendered HTMX application)  
**Performance Goals**: Single-user app — sub-second response times for all requests  
**Constraints**: No custom JavaScript (HTMX only), vanilla CSS, single admin user  
**Scale/Scope**: 1 user, 16 routes, ~50 template-rendered pages

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. JavaScript-Free Frontend | PASS | No JS added — HTMX loaded via CDN in existing `layout.html`. No bundlers, build steps, or custom JS. |
| II. Rust Backend (acton-htmx) | PASS WITH JUSTIFICATION | See Complexity Tracking. acton-dx is used with `default-features = false` and its HTTP layer (Axum) is added directly because acton-dx's full stack (SQLx, actor sessions) conflicts with the existing SeaORM + custom session architecture. Axum is the same framework acton-dx wraps. The constitution's guides reference is respected — the pattern follows acton-dx's documented Axum usage. |
| III. Vanilla CSS & Semantic HTML | PASS | Static CSS served from `static/css/app.css`. No CSS frameworks. Existing HTML templates use semantic elements. |
| IV. Single-User Secured Access | PASS | Existing auth middleware (`enforce_auth`, `extract_session_cookie`) wired into Axum middleware layer. All expense routes behind auth. Argon2 password hashing and server-side sessions preserved. |
| V. Simplicity & Personal-First | PASS | Minimal adapter layer — thin glue handlers bridge Axum extractors to existing domain handlers. No over-engineered abstractions. |
| VI. Test-Driven Development | PASS | TDD cycle followed: tests written before implementation for each new component (server startup, route wiring, middleware, template rendering). |

## Project Structure

### Documentation (this feature)

```text
specs/003-complete-main-http-server/
├── plan.md              # This file
├── research.md          # Phase 0: Framework decision, template engine, middleware approach
├── data-model.md        # Phase 1: No new entities — adapter layer only
├── quickstart.md        # Phase 1: How to run the server
├── contracts/           # Phase 1: HTTP endpoint contracts
│   └── endpoints.md     # Request/response contracts for all 16 routes
└── tasks.md             # Phase 2 output (created by /speckit.tasks)
```

### Source Code (repository root)

```text
src/
├── main.rs              # MODIFIED: Wire AppState → Axum router → TcpListener → serve
├── app/
│   ├── config.rs        # MODIFIED: Add APP_HOST and APP_PORT fields with defaults
│   └── state.rs         # Unchanged
├── auth/                # Unchanged — existing middleware reused
├── db/                  # Unchanged — existing connection/bootstrap reused
├── debts/               # Unchanged
├── exchange_rates/      # Unchanged
├── expenses/            # Unchanged
├── groups/              # Unchanged
└── web/
    ├── mod.rs           # MODIFIED: Re-export new modules
    ├── router.rs        # Unchanged — declarative route table preserved
    ├── csrf.rs          # Unchanged
    ├── error.rs         # MODIFIED: Implement axum IntoResponse for AppError
    ├── handlers/        # Unchanged — domain handlers preserved as-is
    ├── templates/       # Unchanged — HTML template files
    ├── axum_router.rs   # NEW: Build Axum Router from route_specs()
    ├── axum_handlers.rs # NEW: Thin adapter handlers (Axum extractors → domain handlers → HTTP responses)
    ├── middleware.rs     # NEW: Axum middleware layers (auth, CSRF, session extraction)
    └── templates.rs     # NEW: Template engine initialization and render helpers

static/
└── css/app.css          # Unchanged — served via Axum static file serving

tests/
├── unit/                # Existing tests preserved
├── integration/         # MODIFIED: Add HTTP-level integration tests
│   ├── test_server_startup.rs        # NEW: Server binds, health check responds
│   ├── test_route_wiring.rs          # NEW: All 16 routes dispatch correctly
│   ├── test_auth_middleware.rs        # NEW: Protected routes redirect, public routes pass
│   ├── test_csrf_middleware.rs        # NEW: CSRF validation on state-changing routes
│   ├── test_static_serving.rs        # NEW: CSS files served correctly
│   └── test_template_rendering.rs    # NEW: HTML responses contain expected elements
└── contract/            # Existing contract tests preserved
```

**Structure Decision**: Single project layout (existing). New code is added under `src/web/` as adapter modules that bridge Axum to existing domain logic. No new top-level directories or crates needed.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| Using Axum directly instead of acton-dx full stack | acton-dx's `htmx` feature enables SQLx, actor-based sessions, and Askama templates — all conflicting with the existing SeaORM, custom session repos, and Mustache-style templates. Axum is the HTTP layer acton-dx wraps internally. | Enabling `default-features = true` on acton-dx would require replacing SeaORM with SQLx, rewriting session management to use actor-based sessions, and converting templates to Askama — a massive rewrite far beyond the scope of "complete main." |
