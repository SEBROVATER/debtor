CREATE TABLE spending_payers (
    spending_id    INTEGER NOT NULL REFERENCES spendings(id) ON DELETE CASCADE,
    participant_id INTEGER NOT NULL REFERENCES participants(id) ON DELETE CASCADE,
    paid_amount    TEXT    NOT NULL,
    PRIMARY KEY (spending_id, participant_id)
);
