DROP TABLE IF EXISTS tasks;
DROP TABLE IF EXISTS users;

CREATE TABLE users (
    id TEXT PRIMARY KEY,
    email TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    created_at INTEGER NOT NULL
);

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
    device_id TEXT NOT NULL,
    FOREIGN KEY(user_id) REFERENCES users(id)
);

-- optimized indexes for "smart sync" logic
-- allows filtering by user AND updated_at efficiently
CREATE INDEX idx_user_updated ON tasks(user_id, updated_at);
-- allows finding deleted items (tombstones) efficiently
CREATE INDEX idx_user_deleted ON tasks(user_id, deleted_at);
