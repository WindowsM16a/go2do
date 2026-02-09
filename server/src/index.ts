import { Hono } from 'hono';
import { cors } from 'hono/cors';

type Bindings = {
  DB: D1Database;
};

type Task = {
  id: string;
  user_id: string;
  created_at: number;
  updated_at: number;
  deleted_at: number | null;
  content: string;
  completed: boolean;
  pinned: boolean;
  version: number;
  device_id: string;
};

type SyncRequest = {
  last_sync: number;
  changes: Task[];
};

type Variables = {
  userId: string;
};

const app = new Hono<{ Bindings: Bindings; Variables: Variables }>();

app.use('*', cors());

// basic auth middleware (placeholder)
app.use(async (c, next) => {
  // in a real app, verify jwt here.
  // for now, we trust the header 'x-user-id' because we're optimistic like that.
  const userId = c.req.header('x-user-id') || 'dev-user';
  c.set('userId', userId);
  await next();
});

app.get('/', (c) => c.text('Go2Do Sync Server Running. go away unless you are a client.'));

app.post('/sync', async (c) => {
  const userId = c.get('userId') as string;
  let body: SyncRequest;
  
  try {
    body = await c.req.json<SyncRequest>();
  } catch (e) {
    return c.json({ error: 'invalid json, get it together client' }, 400);
  }

  const { last_sync, changes } = body;
  const db = c.env.DB;

  // 1. apply changes (last write wins per field via sql upsert)
  // logic: insert or replace. in a real system we'd check versions properly.
  // for v1: we trust the client's 'version' is newer if they sent it.
  
  if (changes && changes.length > 0) {
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

    const batch = changes.map(task => stmt.bind(
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

    try {
      await db.batch(batch);
    } catch (e) {
      console.error('batch insert failed:', e);
      return c.json({ error: 'database threw a fit', details: (e as Error).message }, 500);
    }
  }

  // 2. fetch updates
  // we return everything that changed since the last sync.
  // if you just wrote it, we might return it back to you. deal with it.
  const updatesFn = async () => {
    try {
      const result = await db.prepare(`
        SELECT * FROM tasks
        WHERE user_id = ? AND updated_at > ?
      `).bind(userId, last_sync).all();
      return result.results;
    } catch (e) {
        console.error('fetch updates failed:', e);
        return [];
    }
  };
  
  const updates = await updatesFn();

  return c.json({
    updates,
    server_time: Date.now(),
  });
});

export default app;
