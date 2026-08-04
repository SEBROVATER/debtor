CREATE TABLE spending_shares (
    spending_id    INTEGER NOT NULL REFERENCES spendings(id) ON DELETE CASCADE,
    participant_id INTEGER NOT NULL REFERENCES participants(id) ON DELETE RESTRICT,
    share_amount   TEXT    NOT NULL CHECK (typeof(share_amount) = 'text'),
    PRIMARY KEY (spending_id, participant_id)
);

CREATE INDEX idx_spending_shares_participant ON spending_shares(participant_id);
