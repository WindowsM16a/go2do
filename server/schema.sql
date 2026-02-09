DROP TABLE IF EXISTS tasks;
CREATE TABLE tasks (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    deleted_at INTEGER,
    content TEXT NOT NULL,
    completed BOOLEAN NOT NULL DEFAULT 0,
    pinned BOOLEAN NOT NULL DEFAULT 0,
    version INTEGER NOT NULL DEFAULT 1,
    device_id TEXT NOT NULL
);
CREATE INDEX idx_user_updated ON tasks(user_id, updated_at);
