# debtor

A personal, single-owner expense-sharing web application built with Rust.

## Architecture

`debtor` is a [Cargo workspace] with the binary crate at the root and three library crates:

```
debtor (root) → debtor-web → debtor-domain ← debtor-infra
```

| Crate | Purpose |
|---|---|
| Root (`debtor`) | Binary — composition root that wires concrete implementations and starts the server. |
| **debtor-domain** | Pure business logic — balance calculation, debt simplification, share splitting. Zero I/O dependencies; fully unit-testable without async runtimes. |
| **debtor-infra** | Infrastructure adapters — `SQLx` repositories, `reqwest` HTTP clients, `argon2` password hashing. Implements traits defined in `debtor-domain`. |
| **debtor-web** | HTTP layer — Axum handlers, `Askama` templates, `axum-htmx` response helpers, middleware (auth, CSRF). |

### Dependency Inversion

`debtor-domain` defines trait interfaces for external dependencies (repositories, providers). `debtor-web` handlers receive `Arc<dyn Trait>` references, enabling straightforward mocking in tests without infrastructure.

## Tech Stack

| Layer | Technology |
|---|---|
| HTTP Framework | [Axum] 0.8 |
| HTMX Integration | [axum-htmx] |
| Templates | [Askama] (compile-time type-safe) |
| Database | [SQLite] via [SQLx] (compile-time verified queries) |
| Sessions | [tower-sessions] (cookie-based, server-side) |
| Auth | [Argon2id] password hashing, HTTP-only cookies |
| Frontend | [HTMX] — zero custom JavaScript |
| Styling | Vanilla CSS (no frameworks) |
| HTTP Client | [reqwest] with rustls-tls (exchange rates) |

## Quick Start

```bash
# Copy environment config
cp .env.example .env

# Generate an Argon2 password hash
echo -n "yourpassword" | argon2 somesalt -e

# Edit .env and set APP_ADMIN_PASSWORD_HASH to the generated hash
$EDITOR .env

# Build and run
cargo run
```

Visit `http://localhost:3000` (default).

## Development

```bash
# Fast compile check
cargo check

# Lint and auto-fix (pedantic + nursery clippy)
cargo clippy --fix --allow-dirty

# Format
cargo fmt
```

### Project Structure

```
debtor/
├── Cargo.toml                  # Workspace root + binary crate
├── src/
│   └── main.rs                 # Application entry point (composition root)
├── migrations/                 # SQLx migrations (workspace-level)
├── debtor-domain/              # Pure domain logic
├── debtor-infra/               # Infrastructure adapters
├── debtor-web/                 # HTTP layer
├── static/css/                 # Vanilla CSS
└── specs/                      # Feature specs (speckit workflow)
```

## Principles

- **No custom JavaScript** — HTMX attributes only; no JS frameworks, bundlers, or scripts
- **Axum + composable crates** — mature, independently-maintained components (axum-htmx, Askama, SQLx, tower-sessions, argon2)
- **Modern HTML & CSS** — prefer modern web platform features over deprecated patterns (e.g., CSS logical properties, `:has()`, container queries, `<dialog>`, `popover`, `inert` attribute); avoid legacy workarounds when native solutions exist
- **Vanilla CSS & semantic HTML** — no CSS frameworks; CSS custom properties for design tokens; Grid/Flexbox layouts; correct semantic elements (`<nav>`, `<main>`, `<time>`, etc.)
- **Single-user auth** — one owner account, no self-registration, Argon2-hashed password, HTTP-only server-side sessions
- **Workspace architecture** — 4-crate Cargo workspace with unidirectional dependency flow and `Arc<dyn Trait>` dependency inversion

## License

MIT OR Apache-2.0

[Axum]: https://github.com/tokio-rs/axum
[axum-htmx]: https://github.com/robertwayne/axum-htmx
[Askama]: https://github.com/djc/askama
[SQLite]: https://sqlite.org
[SQLx]: https://github.com/launchbadge/sqlx
[tower-sessions]: https://github.com/maxcountryman/tower-sessions
[Argon2id]: https://en.wikipedia.org/wiki/Argon2
[HTMX]: https://htmx.org
[reqwest]: https://github.com/seanmonstar/reqwest
[Cargo workspace]: https://doc.rust-lang.org/cargo/reference/workspaces.html
