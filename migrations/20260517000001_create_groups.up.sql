CREATE TABLE groups (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT    NOT NULL CHECK (length(name) BETWEEN 1 AND 100),
    currency    TEXT    NOT NULL CHECK (currency IN ('USD', 'EUR', 'RUB', 'KGS', 'TRY', 'KZT', 'UZS', 'CNY', 'KRW', 'JPY', 'OMR', 'TJS')),
    is_archived INTEGER NOT NULL DEFAULT 0 CHECK (is_archived IN (0, 1)),
    created_at  TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT    NOT NULL DEFAULT (datetime('now'))
);
