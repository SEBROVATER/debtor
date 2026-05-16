# Research: Complete Main Function with HTTP Server

**Branch**: `003-complete-main-http-server` | **Date**: 2026-04-12

## R1: HTTP Framework Selection

### Decision: Axum 0.8 (direct dependency)

### Rationale

The constitution mandates acton-htmx (acton-dx) as the backend framework. However, acton-dx v1.0.0-beta.10 is currently included with `default-features = false`, which disables all HTTP functionality. When its `htmx` feature is enabled, acton-dx wraps Axum 0.8 internally and adds its own opinionated layers (SQLx for database, actor-based sessions, Askama templates).

The existing codebase uses:
- **SeaORM** (not SQLx) for database access — all entities, repos, and migrations are SeaORM-based
- **Custom session management** — `SessionRepo` with SHA-256 hashed tokens in a `sessions` table via SeaORM
- **Custom auth state** — `AuthStateRepo` with lockout logic via SeaORM
- **Handlebars-style templates** — `{{#each}}`, `{{#if}}`, `{{> partial}}` syntax (not Askama's Jinja2 syntax)

Enabling acton-dx's full stack would require replacing SeaORM with SQLx, rewriting session management, and converting all templates from Handlebars to Askama — a rewrite far exceeding the scope of "complete main."

Using Axum directly is consistent with acton-dx's architecture (it is the same HTTP layer) and avoids the impedance mismatch with existing infrastructure.

### Alternatives Considered

| Alternative | Why Rejected |
|-------------|-------------|
| acton-dx with `default-features = true` | Requires replacing SeaORM → SQLx, rewriting sessions → actor-based, converting templates → Askama. Massive scope expansion. |
| actix-web | Different runtime model (actix vs tokio). The project already uses tokio. Axum's tower middleware model is simpler. |
| warp | Less maintained, smaller ecosystem than Axum. No clear advantage. |
| rocket | Requires nightly Rust (or specific stable features). Less ecosystem alignment with tokio/tower. |

### Dependencies to Add

```toml
axum = { version = "0.8", features = ["macros"] }
tower = "0.5"
tower-http = { version = "0.6", features = ["fs", "trace"] }
```

---

## R2: Template Engine Selection

### Decision: handlebars (handlebars-rs)

### Rationale

The existing 7 template files use Handlebars/Mustache syntax throughout:
- Variable interpolation: `{{variable}}`
- Loops: `{{#each items}}...{{/each}}`
- Conditionals: `{{#if condition}}...{{else}}...{{/if}}`
- Partials: `{{> path/to/partial}}`

handlebars-rs supports all of these constructs natively with zero template modifications. It loads templates at runtime from files, supports template directories with automatic partial registration, and has a mature, well-maintained Rust implementation.

### Alternatives Considered

| Alternative | Why Rejected |
|-------------|-------------|
| minijinja | Jinja2 syntax (`{% for %}`, `{% if %}`) — would require rewriting all templates. |
| askama | Jinja2 syntax, compile-time — would require rewriting all templates AND adding Rust struct annotations for each template. |
| tera | Jinja2 syntax — would require rewriting all templates. |

### Dependencies to Add

```toml
handlebars = "6"
```

---

## R3: Axum Middleware Strategy for Auth & CSRF

### Decision: Axum middleware layers using `axum::middleware::from_fn_with_state`

### Rationale

The existing codebase provides framework-agnostic middleware logic:
- `enforce_auth(route: &RouteSpec, is_authenticated: bool) -> AuthOutcome` — determines if a request should proceed or redirect to login
- `extract_session_cookie(cookie_header: Option<&str>, cookie_name: &str) -> Option<String>` — extracts session token from cookie header
- `validate_csrf(method: RouteMethod, provided: Option<&CsrfToken>, expected: Option<&CsrfToken>) -> Result<(), CsrfError>` — validates CSRF tokens on state-changing methods

The adapter approach:
1. An Axum middleware function extracts the session cookie from the request headers using `extract_session_cookie`
2. It validates the session via `SessionRepo::find_active_session` and optionally touches it (sliding expiry)
3. It stores the `SessionContext` in request extensions
4. A second middleware layer (or per-route logic) calls `enforce_auth` using the route's `requires_auth` flag
5. CSRF validation is applied as a middleware or extractor for routes with `csrf_protected: true`

This approach reuses all existing logic without modification and keeps the Axum-specific code in a thin adapter layer.

### Implementation Pattern

```
Request → Extract Session Cookie → Validate Session → Store SessionContext in Extensions
        → Check Auth (redirect if needed) → Check CSRF (reject if invalid) → Handler
```

---

## R4: Route Wiring Strategy

### Decision: Build Axum router programmatically from `route_specs()`

### Rationale

The existing `route_specs()` returns a `Vec<RouteSpec>` with 16 route definitions. Each `RouteSpec` has a `handler` field (a string name like `"health_handler"`, `"auth_login_page"`, etc.).

The adapter layer will:
1. Iterate over `route_specs()`
2. Match each `handler` string to a concrete Axum handler function (a thin adapter that bridges Axum extractors to the existing domain handler)
3. Register the route with the appropriate HTTP method on the Axum `Router`

This preserves the declarative route table as the single source of truth for routes while adding the transport layer.

### Adapter Handler Pattern

Each adapter handler:
1. Extracts Axum request data (path params, form body, query params, extensions)
2. Converts to the existing domain request struct (e.g., `CreateGroupRequest`)
3. Calls the existing domain handler (e.g., `handle_create_group`)
4. Converts the domain response to an Axum HTTP response (rendered HTML or redirect)

---

## R5: Static File Serving

### Decision: `tower-http::services::ServeDir`

### Rationale

The project has a single static directory (`static/css/app.css`) referenced in `layout.html` as `/static/css/app.css`. tower-http's `ServeDir` maps a URL path prefix to a filesystem directory with proper MIME types, caching headers, and 404 handling. It is the standard approach for Axum applications and requires no additional dependencies beyond `tower-http` (already needed for tracing middleware).

---

## R6: Host/Port Configuration

### Decision: Add `APP_HOST` and `APP_PORT` environment variables to `AppConfig`

### Rationale

The existing `AppConfig::from_env()` pattern reads environment variables with fallback defaults. Two new fields:
- `host: String` — from `APP_HOST`, default `"127.0.0.1"`
- `port: u16` — from `APP_PORT`, default `3000`

These follow the same pattern as existing config fields (`APP_DATABASE_URL`, `APP_SESSION_COOKIE_NAME`, etc.) and will be documented in `.env.example`.

---

## R7: Graceful Shutdown

### Decision: `tokio::signal` for SIGTERM/SIGINT handling with Axum's `with_graceful_shutdown`

### Rationale

Axum's `serve()` returns a `Serve` handle that supports `.with_graceful_shutdown(signal)`. The signal is a future that completes when a shutdown is requested. Using `tokio::signal::ctrl_c()` (for SIGINT) combined with a SIGTERM handler covers both interactive and deployment scenarios. SeaORM's `DatabaseConnection` implements `Drop` for connection cleanup, so no explicit teardown is needed beyond stopping the server.

---

## R8: Method Override for HTML Forms

### Decision: Support `_method` hidden field for PATCH/DELETE from HTML forms

### Rationale

HTML forms only support GET and POST methods. The existing templates use a pattern of `<input type="hidden" name="_method" value="PATCH">` (and DELETE) inside POST forms. The Axum adapter layer needs to:
1. For POST requests, check for a `_method` form field
2. If present, treat the request as the overridden method for routing purposes

This can be implemented as a middleware or as logic within the adapter handler dispatch.
