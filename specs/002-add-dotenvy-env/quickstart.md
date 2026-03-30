# Quickstart: Running debtor with .env Configuration

## Prerequisites

- Rust toolchain installed (1.94+)
- Project cloned and dependencies built

## Setup

1. **Create your `.env` file** from the template:

   ```bash
   cp .env.example .env
   ```

2. **Edit `.env`** with your values:

   ```bash
   # Required: set a password hash for the admin account
   APP_ADMIN_PASSWORD_HASH=your_argon2_hash_here

   # Optional: override defaults
   # APP_DATABASE_URL=sqlite://debtor.db?mode=rwc
   # APP_SESSION_COOKIE_NAME=debtor_session
   # APP_ADMIN_USERNAME=owner
   # APP_SESSION_COOKIE_SECURE=false
   # APP_EXCHANGE_BASE_URL=https://api.frankfurter.app
   ```

3. **Run the application**:

   ```bash
   cargo run
   ```

   The application will:
   - Load `.env` into the process environment
   - Read configuration via `AppConfig::from_env()`
   - Initialize the database and start the server

## Verification

- **Success**: Application prints `debtor bootstrap initialized` and starts serving
- **No `.env` file**: Application falls back to system environment and defaults
- **Malformed `.env`**: Application exits with a clear error message about the parse failure

## Without .env File

You can also set environment variables directly in your shell:

```bash
export APP_ADMIN_PASSWORD_HASH=your_hash
cargo run
```

System environment variables take precedence over `.env` file values.

## Compile-Time Embedding

For deployment scenarios where you want config baked into the binary:

```bash
# Ensure env vars are set during compilation
export APP_ADMIN_PASSWORD_HASH=your_hash
cargo build --release
# The binary can now run without a .env file
```
