CREATE TABLE spendings (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    group_id       INTEGER NOT NULL REFERENCES groups(id) ON DELETE RESTRICT,
    description    TEXT    NOT NULL CHECK (length(description) BETWEEN 1 AND 200),
    total_amount   TEXT    NOT NULL CHECK (typeof(total_amount) = 'text'),
    currency       TEXT    NOT NULL CHECK (currency IN ('USD', 'EUR', 'RUB', 'KGS', 'TRY', 'KZT', 'UZS', 'CNY', 'KRW', 'JPY', 'OMR', 'TJS')),
    spending_type  TEXT    NOT NULL DEFAULT 'other' CHECK (spending_type IN ('food', 'transport', 'housing', 'fun', 'shopping', 'bills', 'health', 'other')),
    spent_date     TEXT    NOT NULL CHECK (
        spent_date GLOB '[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]'
        AND date(spent_date) = spent_date
        AND spent_date >= '2025-01-01'
    ),
    created_at     TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at     TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_spendings_group ON spendings(group_id);
CREATE INDEX idx_spendings_spent_date ON spendings(spent_date);
CREATE INDEX idx_spendings_type ON spendings(spending_type);
CREATE INDEX idx_spendings_group_date ON spendings(group_id, spent_date);
