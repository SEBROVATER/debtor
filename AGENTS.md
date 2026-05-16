# Repository Guidelines

## Project Overview

**debtor** is a personal, single-owner expense-sharing web application built as a Rust Cargo workspace with an Axum backend, HTMX-driven frontend, SQLite database, and zero custom JavaScript.

---

## Project Structure

```
debtor/
├── Cargo.toml                  # Workspace root + binary crate
├── src/
│   └── main.rs                 # Application entry point (composition root)
├── migrations/                 # SQLx migration files (workspace-level)
├── debtor-domain/              # Pure domain logic — zero I/O dependencies
│   └── src/
│       ├── lib.rs
│       ├── traits.rs           # Repository/service trait definitions
│       ├── debts/              # Balance calculation, debt simplification
│       ├── expenses/           # Share splitting logic
│       └── groups/             # Group membership rules
├── debtor-infra/               # Infrastructure adapters
│   └── src/
│       ├── lib.rs
│       ├── db/repos/           # SQLx repository implementations
│       ├── exchange_rates/     # Frankfurter HTTP client, caching
│       └── auth/               # Argon2id password hashing
├── debtor-web/                 # HTTP layer
│   └── src/
│       ├── lib.rs
│       ├── router.rs           # Axum route definitions
│       ├── state.rs            # AppState (shared handler context)
│       ├── handlers/           # Request handlers (auth, expenses, groups, debts)
│       ├── middleware/         # Auth, CSRF middleware
│       └── templates/          # Askama template types
│           └── partials/
├── static/css/                 # Vanilla CSS — no frameworks
├── specs/                      # Feature specs and plans (speckit workflow)
└── .env.example                # Environment variable reference
```

### Dependency Flow

```
debtor (root) → debtor-web → debtor-domain ← debtor-infra
```

- `debtor-domain` defines traits; `debtor-infra` implements them.
- `debtor-web` depends on domain traits, not infrastructure.
- Root crate wires concrete implementations at startup.

---

## Build, Test, and Development Commands

```bash
cargo check              # Fast compilation check
cargo test               # Run all tests (workspace-wide)
cargo test -p <crate>    # Run tests for a specific crate
cargo clippy             # Lint (pedantic + nursery, workspace-wide)
cargo fmt                # Format code
cargo build --release    # Production build
cargo run                # Run the application (requires .env)
```

Copy `.env.example` to `.env` and populate `APP_ADMIN_PASSWORD_HASH` before running. Generate a hash with:

```bash
echo -n "yourpassword" | argon2 somesalt -e
```

---

## Core Principles

### I. JavaScript-Free Frontend

The frontend MUST be implemented without JavaScript except for the single permitted library: [htmx](https://htmx.org). No JavaScript frameworks, bundlers, build steps, or custom JS scripts are allowed. All interactivity MUST be expressed through hypermedia (HTMX attributes, HTML forms, server responses). Browser-native behaviour is preferred over any programmatic workaround.

### II. Rust Backend (Axum)

The backend MUST be implemented in Rust using [Axum](https://github.com/tokio-rs/axum) as the HTTP framework, composed with mature, independently-maintained crates:

- **axum-htmx**: HTMX request extractors and response helpers.
- **Askama**: compile-time type-safe HTML templates.
- **tower-sessions**: cookie-based server-side session management.
- **SQLx**: compile-time verified SQL queries against SQLite.
- **argon2**: password hashing (Argon2id).

External crates MAY be used freely provided they are reliable, actively maintained, and have a meaningful user base. New technology additions SHOULD be documented in the relevant feature plan.

### III. Vanilla CSS & Modern HTML

Styling MUST use modern, vanilla CSS only. CSS frameworks (Bootstrap, Tailwind, Bulma, etc.) are NOT permitted. HTML MUST be semantic — use the correct element for the correct purpose (`<nav>`, `<main>`, `<section>`, `<article>`, `<time>`, etc.). CSS custom properties MUST be used for design tokens (colours, spacing, typography). Layouts MUST use CSS Grid or Flexbox.

Modern web platform features MUST be preferred over deprecated patterns:
- Use CSS logical properties (`margin-block`, `padding-inline`) over physical ones.
- Use `<dialog>`, `popover`, and `inert` instead of custom modal/tab implementations.
- Use `:has()`, container queries, and cascade layers where appropriate.
- Avoid legacy workarounds (e.g., clearfix hacks, float-based layouts) when native solutions exist.

### IV. Single-User Secured Access

Authentication MUST be implemented. The system MUST support exactly one user account (the owner). There MUST be no self-registration flow. All expense-related routes MUST be behind authentication. Credentials MUST be stored securely (hashed with a modern algorithm, e.g., Argon2). Sessions MUST be managed server-side with secure, HTTP-only cookies.

### V. Workspace Architecture

The project MUST be organised as a [Cargo workspace] with the binary crate at the root and three library crates:

| Crate | Responsibility |
|---|---|
| Root (`debtor`) | Binary crate; composition root that wires everything together |
| `debtor-domain` | Pure domain logic (zero I/O deps); defines repository and service traits |
| `debtor-infra` | Infrastructure adapters; implements domain traits with SQLx, reqwest, argon2 |
| `debtor-web` | HTTP layer; Axum handlers, Askama templates, middleware, routing |

Dependency flow MUST be unidirectional:

```
debtor (root) → debtor-web → debtor-domain ← debtor-infra
```

`debtor-domain` defines traits for external dependencies (repositories, providers). Infrastructure implements them. Dependency inversion uses `Arc<dyn Trait>` to keep handlers testable.

---

## Coding Style & Naming Conventions

- **Language**: Rust, edition 2024. Follow idiomatic Rust patterns throughout.
- **Formatting**: `cargo fmt` — all code must be formatted before committing.
- **Linting**: `cargo clippy --workspace` — workspace uses pedantic + nursery lints. Resolve all warnings; treat new warnings as errors.
- **Naming**: `snake_case` for functions, variables, and modules; `PascalCase` for types and traits.
- **Modules**: one domain concept per directory. Expose the public API through `mod.rs`.
- **Error handling**: use `thiserror`-derived error types; avoid `.unwrap()` and `.expect()` in production code paths.
- **`unsafe`**: must include a comment justifying its use.
- **Documentation**: all public items must have doc comments (`#![warn(missing_docs)]` enforced per crate).

---

## Planning Protocol

Before creating any plan, the agent MUST ask comprehensive clarifying questions to resolve undefined spots. At minimum, cover:

- **Scope boundaries**: What exactly is in-scope vs out-of-scope?
- **Error handling**: How should errors be surfaced to users? Redirect? Flash message? Error page?
- **Edge cases**: Empty states, zero amounts, single-member groups, duplicate entries, concurrent modifications.
- **Data model specifics**: Field names, types, constraints, defaults, nullable columns.
- **UI/UX expectations**: Which pages need what content? Navigation flow? Form validation UX?
- **Naming conventions**: Preferred names for domain concepts, database columns, routes.
- **Testing strategy**: Which test suite(s) are relevant? What behaviours need test coverage?
- **Integration points**: Does this feature touch external APIs, existing services, or other features?

No plan should be created until all undefined spots are clarified. This prevents rework and ensures the plan is actionable.

---

## Documentation-First Development

Before using any Rust crate or framework, the agent MUST consult its documentation via the Context7 MCP tool. This ensures:

- Code follows the current API patterns (not outdated examples).
- Feature flags and configuration options are used correctly.
- Deprecations and breaking changes are avoided.
- Recommended idioms and best practices from the crate authors are followed.

**Workflow**:
1. Use `context7_resolve-library-id` to find the correct library ID.
2. Use `context7_query-docs` to fetch relevant documentation for the specific use case.
3. Write code based on the fetched documentation, not from memory or assumptions.

---

## Testing Guidelines

Tests are written using Rust's built-in test framework (`#[test]`, `#[tokio::test]`).

**TDD is mandatory.** Follow Red → Green → Refactor strictly:

1. **Red**: Write a failing test that defines the desired behaviour.
2. **Green**: Write the minimal implementation to make the test pass.
3. **Refactor**: Clean up code while keeping all tests passing.

Tests must be written before implementation; no feature is complete without prior-written tests. Skipping the Red phase (writing tests after implementation) is NOT permitted.

### Per-Crate Test Suites

| Crate | Test Location | Purpose |
|---|---|---|
| `debtor-domain` | `debtor-domain/tests/` | Pure logic, no I/O — `#[test]` only (no async) |
| `debtor-infra` | `debtor-infra/tests/` | Repository and adapter tests with real or mock I/O |
| `debtor-web` | `debtor-web/tests/` | Handler and integration tests against test doubles |
| Workspace-level | `tests/` (if needed) | Cross-crate integration tests |

**Conventions:**
- Test files are named `test_<subject>.rs` (e.g., `test_balance_calculator.rs`).
- Test functions use descriptive `snake_case` names that read as sentences (e.g., `aggregates_balances_across_expenses`).
- Each test must be independent — no shared mutable state between tests.

---

## Commit & Pull Request Guidelines

Commits follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add expense deletion endpoint
fix: correct rounding in share splitter
refactor: extract balance calculator into domain crate
test: add contract tests for group member service
chore: update sqlx to 0.8
```

- Use the **imperative mood** in the subject line.
- Keep the subject line under 72 characters.
- Reference a spec when relevant (e.g., `feat(specs/001): implement expense creation`).

Pull requests must:
- Include a clear description of what changed and why.
- Pass `cargo fmt --check`, `cargo clippy`, and `cargo test` without errors.
- Contain tests written before the implementation (TDD — no after-the-fact test additions).

---

## Configuration & Security Notes

- All configuration is loaded from environment variables (with `.env` fallback via `dotenvy`).
- `APP_ADMIN_PASSWORD_HASH` is **required** — the app will not authenticate without it.
- Session cookies are HTTP-only and server-side managed. Set `APP_SESSION_COOKIE_SECURE=true` in production.
- See `.env.example` for all available variables and their defaults.
