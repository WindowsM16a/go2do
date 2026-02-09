use rusqlite::{params, Connection, Result};
use crate::sync::Task;

pub fn init() -> Result<Connection> {
    let path = "go2do.db";
    let conn = Connection::open(path)?;
    
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tasks (
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
        )",
        [],
    )?;
    
    Ok(conn)
}

pub fn create_task(conn: &Connection, task: &Task) -> Result<()> {
    conn.execute(
        "INSERT INTO tasks (id, user_id, created_at, updated_at, deleted_at, content, completed, pinned, version, device_id)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            task.id, task.user_id, task.created_at, task.updated_at, task.deleted_at,
            task.content, task.completed, task.pinned, task.version, task.device_id
        ],
    )?;
    Ok(())
}

pub fn get_tasks(conn: &Connection) -> Result<Vec<Task>> {
    let mut stmt = conn.prepare("SELECT id, user_id, created_at, updated_at, deleted_at, content, completed, pinned, version, device_id FROM tasks WHERE deleted_at IS NULL ORDER BY created_at DESC")?;
    let task_iter = stmt.query_map([], |row| {
        Ok(Task {
            id: row.get(0)?,
            user_id: row.get(1)?,
            created_at: row.get(2)?,
            updated_at: row.get(3)?,
            deleted_at: row.get(4)?,
            content: row.get(5)?,
            completed: row.get(6)?,
            pinned: row.get(7)?,
            version: row.get(8)?,
            device_id: row.get(9)?,
        })
    })?;

    let mut tasks = Vec::new();
    for task in task_iter {
        tasks.push(task?);
    }
    Ok(tasks)
}

pub fn soft_delete_old_completed_tasks(conn: &Connection) -> Result<usize> {
    let limit_ts = chrono::Utc::now().timestamp_millis() - (1 * 60 * 1000); // 1 minute ago
    let now = chrono::Utc::now().timestamp_millis();
    conn.execute(
        "UPDATE tasks SET deleted_at = ?1, updated_at = ?2, version = version + 1 WHERE completed = 1 AND deleted_at IS NULL AND updated_at < ?3",
        params![now, now, limit_ts],
    )
}

pub fn update_task_completion(conn: &Connection, id: &str, completed: bool) -> Result<()> {
    let now = chrono::Utc::now().timestamp_millis();
    conn.execute(
        "UPDATE tasks SET completed = ?1, updated_at = ?2, version = version + 1 WHERE id = ?3",
        params![completed, now, id],
    )?;
    Ok(())
}

pub fn get_changes_since(conn: &Connection, last_sync: i64) -> Result<Vec<Task>> {
    let mut stmt = conn.prepare("SELECT id, user_id, created_at, updated_at, deleted_at, content, completed, pinned, version, device_id FROM tasks WHERE updated_at > ?1")?;
    let task_iter = stmt.query_map(params![last_sync], |row| {
        Ok(Task {
            id: row.get(0)?,
            user_id: row.get(1)?,
            created_at: row.get(2)?,
            updated_at: row.get(3)?,
            deleted_at: row.get(4)?,
            content: row.get(5)?,
            completed: row.get(6)?,
            pinned: row.get(7)?,
            version: row.get(8)?,
            device_id: row.get(9)?,
        })
    })?;

    let mut tasks = Vec::new();
    for task in task_iter {
        tasks.push(task?);
    }
    Ok(tasks)
}

pub fn upsert_task(conn: &Connection, task: &Task) -> Result<()> {
    conn.execute(
        "INSERT INTO tasks (id, user_id, created_at, updated_at, deleted_at, content, completed, pinned, version, device_id)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
         ON CONFLICT(id) DO UPDATE SET
            content = excluded.content,
            updated_at = excluded.updated_at,
            deleted_at = excluded.deleted_at,
            completed = excluded.completed,
            pinned = excluded.pinned,
            version = excluded.version,
            device_id = excluded.device_id
         WHERE excluded.updated_at > tasks.updated_at",
        params![
            task.id, task.user_id, task.created_at, task.updated_at, task.deleted_at,
            task.content, task.completed, task.pinned, task.version, task.device_id
        ],
    )?;
    Ok(())
}

use rusqlite::OptionalExtension;
// Helper to store/get last sync time
pub fn get_last_sync_timestamp(conn: &Connection) -> Result<i64> {
    conn.execute("CREATE TABLE IF NOT EXISTS meta (key TEXT PRIMARY KEY, value TEXT)", [])?;
    let mut stmt = conn.prepare("SELECT value FROM meta WHERE key = 'last_sync'")?;
    let timestamp: Option<String> = stmt.query_row([], |row| row.get(0)).optional()?;
    
    Ok(timestamp.and_then(|t| t.parse::<i64>().ok()).unwrap_or(0))
}

pub fn set_last_sync_timestamp(conn: &Connection, ts: i64) -> Result<()> {
    conn.execute("INSERT OR REPLACE INTO meta (key, value) VALUES ('last_sync', ?1)", params![ts.to_string()])?;
    Ok(())
}
