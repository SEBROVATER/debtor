<!--
=============================================================================
SYNC IMPACT REPORT
=============================================================================
Version change: 2.3.0 → 3.0.0
  Bump rationale: MAJOR — Data access layer redefined: Sea-ORM replaced
  with SQLx. This is a backward-incompatible change that invalidates all
  existing SeaORM-based entities, repositories, and migrations.

Modified principles:
  - (none — principle text unchanged)

Modified sections:
  - Technology Stack: "Data access" row changed from "Sea-ORM (primary ORM)"
    to "SQLx (compile-time verified queries)".

Added sections/principles:
  - (none)

Removed principles:
  - (none)

Templates checked:
  ✅ .specify/templates/plan-template.md  — Technology-agnostic; no changes
     needed.
  ✅ .specify/templates/spec-template.md  — Technology-agnostic; no changes
     needed.
  ✅ .specify/templates/tasks-template.md — Technology-agnostic; no changes
     needed.
  ✅ .specify/templates/constitution-template.md — Source template; unchanged.

Deferred TODOs:
  - specs/003-complete-main-http-server/ artifacts (plan.md, research.md,
    data-model.md, quickstart.md, contracts/) reference SeaORM and must be
    regenerated after this constitution amendment takes effect.
  - AGENTS.md references "sea-orm-migration crate" in project structure and
    "sea-orm" in commit examples — should be updated to reflect SQLx.
  - .kilocode/rules/specify-rules.md contains auto-generated SeaORM
    references from prior features — will be overwritten on next
    update-agent-context.sh run.
=============================================================================
-->

# debtor Constitution

## Core Principles

### I. JavaScript-Free Frontend

The frontend MUST be implemented without JavaScript except for the single
permitted library: [htmx](https://htmx.org). No JavaScript frameworks,
bundlers, build steps, or custom JS scripts are allowed. All interactivity
MUST be expressed through hypermedia (HTMX attributes, HTML forms, server
responses). Browser-native behaviour is preferred over any programmatic
workaround.

**Rationale**: A JavaScript-free approach keeps the frontend auditable,
dependency-free, and aligned with the project's personal/local-use nature.
HTMX is the single pragmatic exception that enables modern UX without a JS
ecosystem.

### II. Rust Backend (acton-htmx)

The backend MUST be implemented in Rust using the [acton-htmx](https://github.com/Govcraft/acton-dx)
(acton-dx) framework. Development MUST follow the [official acton-htmx guides](https://github.com/Govcraft/acton-dx/tree/main/docs/guides)
to ensure consistency with hypermedia-first patterns. Code MUST follow idiomatic
Rust patterns. Use of `unsafe` blocks MUST be documented with a clear
justification comment. External crates MAY be used freely provided they are
reliable, actively maintained, and have a meaningful user base.

**Rationale**: `acton-htmx` is an opinionated framework designed specifically
for server-rendered hypermedia applications, perfectly aligning with the
project's JS-free and Rust-based goals.

### III. Vanilla CSS & Semantic HTML

Styling MUST use modern, vanilla CSS only. CSS frameworks (Bootstrap, Tailwind,
Bulma, etc.) are NOT permitted. HTML MUST be semantic — use the correct
element for the correct purpose (`<nav>`, `<main>`, `<section>`, `<article>`,
`<time>`, etc.). CSS custom properties MUST be used for design tokens (colours,
spacing, typography). Layouts MUST use CSS Grid or Flexbox.

**Rationale**: Keeping CSS framework-free preserves full control over the
visual output and avoids bloated, opaque stylesheets. Semantic HTML improves
accessibility and maintainability.

### IV. Single-User Secured Access

Authentication MUST be implemented. The system MUST support exactly one user
account (the owner). There MUST be no self-registration flow. All
expense-related routes MUST be behind authentication. Credentials MUST be
stored securely (hashed with a modern algorithm, e.g., Argon2). Sessions
MUST be managed server-side with secure, HTTP-only cookies.

**Rationale**: Although this is a personal project, expenses are private
financial data. Proper auth prevents accidental or malicious exposure if the
service is reachable on a network.

### V. Simplicity & Personal-First

This project is for personal use by a single owner. MUST NOT introduce
premature abstractions, over-engineered patterns, or unnecessary complexity.
Apply YAGNI: build what is needed now. Complexity additions MUST be justified
against a concrete current need.

**Rationale**: A personal expense-sharing tool does not need enterprise
patterns. Keeping it simple makes it easier to maintain, extend, and enjoy
working on.

### VI. Test-Driven Development (NON-NEGOTIABLE)

TDD is the primary development paradigm. Tests MUST be written before
implementation code. The Red-Green-Refactor cycle MUST be followed strictly:

1. **Red**: Write a failing test that defines the desired behaviour.
2. **Green**: Write the minimal implementation to make the test pass.
3. **Refactor**: Clean up code while keeping all tests passing.

No feature MUST be considered complete unless it has corresponding tests that
were written first. Skipping the Red phase (writing tests after implementation)
is NOT permitted.

**Rationale**: TDD forces a clear specification of behaviour before coding
begins, reduces defects, and keeps the codebase safe to refactor — especially
valuable for a long-lived personal project maintained by a single developer.

## Technology Stack

| Layer       | Technology                                      |
|-------------|-------------------------------------------------|
| Framework   | acton-htmx (acton-dx)                           |
| Backend     | Rust                                            |
| Frontend    | HTMX + server-rendered HTML                     |
| Styling     | Vanilla CSS (no frameworks)                     |
| Database    | SQLite (primary)                                |
| Data access | SQLx (compile-time verified queries)            |
| Auth        | Single-user; server-side sessions; secure hash  |
| Testing     | TDD (mandatory); Rust built-in test framework   |
| Deployment  | Docker Compose (allowed; decision per feature)  |

Technology additions SHOULD be documented in the relevant feature plan and
validated against the Simplicity principle before adoption.

## Governance

This constitution supersedes all other development practices for the debtor
project. Amendments MUST be documented as a new version in this file with an
updated Sync Impact Report.

**Amendment procedure**:
1. Identify which principle(s) or sections change.
2. Determine version bump (MAJOR/MINOR/PATCH per semantic rules below).
3. Update this file; update the Sync Impact Report comment.
4. Commit with message: `docs: amend constitution to vX.Y.Z (<summary>)`

**Versioning policy**:
- MAJOR: Principle removed, redefined, or governance fundamentally changed.
- MINOR: New principle or section added; material expansion of guidance.
- PATCH: Clarification, wording improvement, typo fix.

All feature specifications, plans, and task lists produced by the speckit
workflows MUST reference and comply with the active version of this
constitution.

**Version**: 3.0.0 | **Ratified**: 2026-02-23 | **Last Amended**: 2026-04-12
