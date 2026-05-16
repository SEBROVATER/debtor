CREATE TABLE IF NOT EXISTS admin_users (
    id            INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    username      TEXT    NOT NULL UNIQUE,
    password_hash TEXT    NOT NULL,
    created_at    TEXT    NOT NULL,
    updated_at    TEXT    NOT NULL
);

CREATE TABLE IF NOT EXISTS auth_state (
    id                    INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    failed_attempt_count  INTEGER NOT NULL DEFAULT 0,
    lockout_until         TEXT,
    last_failed_at        TEXT,
    updated_at            TEXT    NOT NULL
);

CREATE TABLE IF NOT EXISTS sessions (
    id           TEXT    PRIMARY KEY NOT NULL,
    user_id      INTEGER NOT NULL REFERENCES admin_users(id) ON DELETE CASCADE,
    token_hash   TEXT    NOT NULL UNIQUE,
    created_at   TEXT    NOT NULL,
    last_seen_at TEXT    NOT NULL,
    expires_at   TEXT    NOT NULL,
    revoked_at   TEXT
);

CREATE TABLE IF NOT EXISTS groups (
    id              TEXT PRIMARY KEY NOT NULL,
    name            TEXT NOT NULL,
    target_currency TEXT NOT NULL,
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS members (
    id           TEXT    PRIMARY KEY NOT NULL,
    group_id     TEXT    NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
    display_name TEXT    NOT NULL,
    is_active    INTEGER NOT NULL DEFAULT 1,
    created_at   TEXT    NOT NULL,
    updated_at   TEXT    NOT NULL,
    removed_at   TEXT
);

CREATE TABLE IF NOT EXISTS expenses (
    id               TEXT PRIMARY KEY NOT NULL,
    group_id         TEXT NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
    payer_member_id  TEXT NOT NULL REFERENCES members(id) ON DELETE RESTRICT,
    amount           TEXT NOT NULL,
    currency         TEXT NOT NULL,
    note             TEXT,
    expense_date     TEXT NOT NULL,
    created_at       TEXT NOT NULL,
    updated_at       TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS expense_shares (
    id              TEXT PRIMARY KEY NOT NULL,
    expense_id      TEXT NOT NULL REFERENCES expenses(id) ON DELETE CASCADE,
    member_id       TEXT NOT NULL REFERENCES members(id) ON DELETE RESTRICT,
    share_mode      TEXT NOT NULL,
    share_value     TEXT NOT NULL,
    computed_amount TEXT NOT NULL,
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS exchange_rates (
    id            TEXT PRIMARY KEY NOT NULL,
    from_currency TEXT NOT NULL,
    to_currency   TEXT NOT NULL,
    rate          TEXT NOT NULL,
    fetched_at    TEXT NOT NULL,
    rate_date     TEXT NOT NULL,
    provider      TEXT NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_exchange_rates_unique
    ON exchange_rates (from_currency, to_currency, rate_date);
