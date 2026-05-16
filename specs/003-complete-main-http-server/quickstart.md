# Quickstart: Complete Main Function with HTTP Server

**Branch**: `003-complete-main-http-server` | **Date**: 2026-04-12

## Prerequisites

- Rust 1.94.1+ (Edition 2024)
- A valid `.env` file (copy from `.env.example`)
- An Argon2 password hash for the admin account

## Setup

1. **Copy environment file**:
   ```bash
   cp .env.example .env
   ```

2. **Generate admin password hash**:
   ```bash
   echo -n "yourpassword" | argon2 somesalt -e
   ```

3. **Set the password hash in `.env`**:
   ```bash
   APP_ADMIN_PASSWORD_HASH='$argon2id$v=19$m=19456,t=2,p=1$...'
   ```

4. **Optional — customize host and port** (defaults: `127.0.0.1:3000`):
   ```bash
   APP_HOST=0.0.0.0
   APP_PORT=8080
   ```

## Run

```bash
cargo run
```

Expected output:
```
debtor listening on 127.0.0.1:3000
```

## Verify

```bash
# Health check
curl http://127.0.0.1:3000/health
# Expected: "ok"

# Login page
curl http://127.0.0.1:3000/login
# Expected: HTML login form

# Dashboard (redirects to login without session)
curl -v http://127.0.0.1:3000/dashboard
# Expected: 303 redirect to /login
```

## Run Tests

```bash
# All tests
cargo test

# Only unit tests
cargo test --test unit

# Only integration tests
cargo test --test integration

# Only contract tests
cargo test --test contract
```

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `APP_HOST` | `127.0.0.1` | Network interface to bind |
| `APP_PORT` | `3000` | TCP port to listen on |
| `APP_DATABASE_URL` | `sqlite://debtor.db?mode=rwc` | SQLite connection URL |
| `APP_SESSION_COOKIE_NAME` | `debtor_session` | Name of the session cookie |
| `APP_ADMIN_USERNAME` | `owner` | Admin account username |
| `APP_ADMIN_PASSWORD_HASH` | *(required)* | Argon2 hash of admin password |
| `APP_SESSION_COOKIE_SECURE` | `false` | Require HTTPS for session cookie |
| `APP_EXCHANGE_BASE_URL` | `https://api.frankfurter.app` | Exchange rate API base URL |

## Shutdown

Send `SIGINT` (Ctrl+C) or `SIGTERM` to gracefully shut down the server. In-flight requests will be completed before exit.
