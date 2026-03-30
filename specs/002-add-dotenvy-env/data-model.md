# Data Model: Dotenvy Environment Variable Configuration

## Entities

### Environment Variable (conceptual)

Not a persisted entity — environment variables are process-level configuration loaded from the `.env` file or system environment.

| Variable | Type | Required | Default | Description |
|----------|------|----------|---------|-------------|
| APP_DATABASE_URL | String | No | `sqlite://debtor.db?mode=rwc` | SQLite database connection URL |
| APP_SESSION_COOKIE_NAME | String | No | `debtor_session` | Name of the HTTP session cookie |
| APP_ADMIN_USERNAME | String | No | `owner` | Username for the single admin account |
| APP_ADMIN_PASSWORD_HASH | String | Yes* | None | Argon2-hashed password for admin login |
| APP_SESSION_COOKIE_SECURE | Boolean | No | `false` | Whether session cookie requires HTTPS |
| APP_EXCHANGE_BASE_URL | String | No | `https://api.frankfurter.app` | Base URL for the exchange rate API |

*APP_ADMIN_PASSWORD_HASH is conditionally required — the app will not bootstrap an admin user if absent, but the application still starts.

### .env File

A plain-text file in the project root containing `KEY=VALUE` pairs, one per line.

**Format rules**:
- Lines starting with `#` are comments
- Empty lines are ignored
- Values are strings; no quoting required (but double-quotes are stripped)
- Whitespace around `=` is not supported (standard dotenv format)

### .env.example File

Template file tracked in version control. Same format as `.env` but with placeholder values and comments explaining each variable.

## Relationships

```
.env.example  --(copy & edit)--> .env  --(dotenvy::dotenv())--> Process Environment  --(std::env::var)--> AppConfig
                                    ^                                        ^
                                    |                                        |
                             system env vars                          compile-time defaults
                             (take precedence)                        (dotenvy_macro)
```

## Validation Rules

- `APP_SESSION_COOKIE_SECURE` must parse as a boolean (`true`/`false`/`1`/`0`/`yes`/`no`/`on`/`off`)
- `APP_DATABASE_URL` is a valid SQLite connection string (defaults validate)
- No validation needed for other string variables at the env-loading level

## State Transitions

N/A — Environment variables are loaded once at startup and remain immutable for the process lifetime.
