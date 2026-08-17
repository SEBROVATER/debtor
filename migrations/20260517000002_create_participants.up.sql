CREATE TABLE participants (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    group_id    INTEGER NOT NULL CHECK (group_id > 0)
                REFERENCES groups(id) ON DELETE CASCADE,
    name        TEXT    NOT NULL CHECK (length(name) BETWEEN 1 AND 100),
    color       TEXT    NOT NULL CHECK (
        length(color) = 7
        AND color GLOB '#[0-9A-Fa-f][0-9A-Fa-f][0-9A-Fa-f][0-9A-Fa-f][0-9A-Fa-f][0-9A-Fa-f]'
    ),
    is_archived INTEGER NOT NULL DEFAULT 0 CHECK (is_archived IN (0, 1)),
    created_at  TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE TRIGGER participants_owner_immutable
BEFORE UPDATE OF group_id ON participants
WHEN OLD.group_id <> NEW.group_id
BEGIN
    SELECT RAISE(ABORT, 'participant ownership is immutable');
END;
