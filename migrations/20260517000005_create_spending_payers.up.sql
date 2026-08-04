CREATE TABLE spending_payers (
    spending_id    INTEGER NOT NULL REFERENCES spendings(id) ON DELETE CASCADE,
    participant_id INTEGER NOT NULL REFERENCES participants(id) ON DELETE RESTRICT,
    paid_amount    TEXT    NOT NULL CHECK (typeof(paid_amount) = 'text'),
    PRIMARY KEY (spending_id, participant_id)
);

CREATE INDEX idx_spending_payers_participant ON spending_payers(participant_id);
