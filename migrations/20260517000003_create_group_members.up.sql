CREATE TABLE group_members (
    group_id       INTEGER NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
    participant_id INTEGER NOT NULL REFERENCES participants(id) ON DELETE CASCADE,
    is_active      INTEGER NOT NULL DEFAULT 1,
    joined_at      TEXT    NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (group_id, participant_id)
);

CREATE INDEX idx_group_members_group ON group_members(group_id);
CREATE INDEX idx_group_members_participant ON group_members(participant_id);
