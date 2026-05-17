CREATE TABLE spendings (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    group_id       INTEGER NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
    description    TEXT    NOT NULL,
    total_amount   TEXT    NOT NULL,
    currency       TEXT    NOT NULL,
    spending_type  TEXT    NOT NULL DEFAULT 'other',
    spent_date     TEXT    NOT NULL,
    created_at     TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at     TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_spendings_group ON spendings(group_id);
CREATE INDEX idx_spendings_spent_date ON spendings(spent_date);
CREATE INDEX idx_spendings_type ON spendings(spending_type);
CREATE INDEX idx_spendings_group_date ON spendings(group_id, spent_date);
