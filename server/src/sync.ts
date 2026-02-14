import { Bindings, Task } from './types';

// Helper to batch DB write operations on D1
export async function batchWriteTasks(db: D1Database, userId: string, tasks: Task[]): Promise<void> {
    if (tasks.length === 0) return;

    // Use a single prepared statement for all inserts
    const stmt = db.prepare(`
      INSERT INTO tasks (id, user_id, created_at, updated_at, deleted_at, content, completed, pinned, version, device_id)
      VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
      ON CONFLICT(id) DO UPDATE SET
        content = excluded.content,
        updated_at = excluded.updated_at,
        deleted_at = excluded.deleted_at,
        completed = excluded.completed,
        pinned = excluded.pinned,
        version = excluded.version,
        device_id = excluded.device_id
      WHERE excluded.version > tasks.version
    `);

    // Create batch array
    const batch = tasks.map(task => stmt.bind(
      task.id,
      userId,
      task.created_at,
      task.updated_at,
      task.deleted_at,
      task.content,
      task.completed ? 1 : 0, 
      task.pinned ? 1 : 0,
      task.version,
      task.device_id
    ));

    await db.batch(batch);
}

// Get changes for a user since a given timestamp
export async function getChanges(db: D1Database, userId: string, lastSync: number): Promise<Task[]> {
    const result = await db.prepare(`
        SELECT * FROM tasks
        WHERE user_id = ? AND updated_at > ?
    `).bind(userId, lastSync).all<Task>();
    
    // Convert SQLite 1/0 booleans back to true/false if needed (D1 typed return might handle this but being safe)
    return result.results.map(t => ({
        ...t,
        completed: Boolean(t.completed),
        pinned: Boolean(t.pinned)
    }));
}

// Calculate a lightweight hash of the user's current data state
// This allows the client to do a quick "HEAD" check before downloading
export async function calculateDataHash(db: D1Database, userId: string): Promise<string> {
    // We sum the versions and max update time to create a unique signature
    // If anything changes (add, update, delete), this signature changes.
    const result = await db.prepare(`
        SELECT 
            COUNT(*) as count, 
            MAX(updated_at) as max_updated, 
            SUM(version) as sum_version 
        FROM tasks 
        WHERE user_id = ?
    `).bind(userId).first<{ count: number, max_updated: number, sum_version: number }>();

    if (!result) return 'empty';

    const rawString = `${result.count || 0}-${result.max_updated || 0}-${result.sum_version || 0}`;
    
    // Hash it (SHA-1 is fine for ETag purposes, fast)
    const encoder = new TextEncoder();
    const data = encoder.encode(rawString);
    const hashBuffer = await crypto.subtle.digest('SHA-1', data);
    const hashArray = Array.from(new Uint8Array(hashBuffer));
    return hashArray.map(b => b.toString(16).padStart(2, '0')).join('');
}
