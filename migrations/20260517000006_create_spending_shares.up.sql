CREATE TABLE spending_shares (
    spending_id    INTEGER NOT NULL REFERENCES spendings(id) ON DELETE CASCADE,
    participant_id INTEGER NOT NULL REFERENCES participants(id) ON DELETE CASCADE,
    share_amount   TEXT    NOT NULL,
    PRIMARY KEY (spending_id, participant_id)
);
