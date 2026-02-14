import { Hono } from 'hono';
import { cors } from 'hono/cors';
import { Bindings, SyncRequest } from './types';
import { hashPassword, verifyPassword, createToken } from './auth';
import { authMiddleware } from './middleware';
import { batchWriteTasks, getChanges, calculateDataHash } from './sync';
import { rateLimit } from './ratelimit';
import * as cookie from 'cookie';

type Variables = {
  userId: string;
};

const app = new Hono<{ Bindings: Bindings; Variables: Variables }>();

app.use('*', cors({
  origin: '*', // TODO: Lock this down in prod? nah, desktop apps need it.
  allowMethods: ['POST', 'GET', 'OPTIONS'],
  allowHeaders: ['Content-Type', 'Authorization', 'x-user-id'],
  exposeHeaders: ['Content-Length'],
  maxAge: 600,
  credentials: true,
}));

app.get('/', (c) => c.text('Go2Do Sync Server Running. go away unless you are a client.'));

// --- AUTH ROUTES ---

app.post('/auth/register', async (c) => {
  const { email, password } = await c.req.json();
  
  if (!email || !password || password.length < 8) {
    return c.json({ error: 'invalid inputs. password needs 8 chars minimum.' }, 400);
  }

  // Check existing
  const existing = await c.env.DB.prepare('SELECT id FROM users WHERE email = ?').bind(email).first();
  if (existing) {
    return c.json({ error: 'email taken. be original.' }, 409);
  }

  const id = crypto.randomUUID();
  const pwdHash = await hashPassword(password);
  const now = Date.now();

  try {
    await c.env.DB.prepare(
      'INSERT INTO users (id, email, password_hash, created_at) VALUES (?, ?, ?, ?)'
    ).bind(id, email, pwdHash, now).run();

    const token = await createToken(id, c.env.JWT_SECRET);
    
    // Set HttpOnly Cookie
    c.header('Set-Cookie', cookie.serialize('auth_token', token, {
      httpOnly: true,
      secure: true, // Always true on Cloudflare
      sameSite: 'strict',
      maxAge: 60 * 60 * 24 * 30, // 30 days
      path: '/',
    }));

    return c.json({ message: 'welcome aboard.', token, user_id: id }, 201);
  } catch (e) {
    return c.json({ error: 'db exploded', details: (e as Error).message }, 500);
  }
});

app.post('/auth/login', async (c) => {
  const { email, password } = await c.req.json();
  const user = await c.env.DB.prepare('SELECT * FROM users WHERE email = ?').bind(email).first<any>();

  if (!user || !(await verifyPassword(password, user.password_hash))) {
    return c.json({ error: 'nice try. invalid credentials.' }, 401);
  }

  const token = await createToken(user.id, c.env.JWT_SECRET);

  c.header('Set-Cookie', cookie.serialize('auth_token', token, {
    httpOnly: true,
    secure: true,
    sameSite: 'strict',
    maxAge: 60 * 60 * 24 * 30, // 30 days
    path: '/',
  }));

  return c.json({ message: 'you are in.', token, user_id: user.id }, 200);
});

// --- PROTECTED ROUTES ---

app.use('/sync/*', authMiddleware);
app.use('/sync/*', rateLimit);

app.get('/sync/check', async (c) => {
  const userId = c.get('userId');
  
  // 1. Check KV Cache (Fastest)
  const cachedHash = await c.env.GO2DO_KV.get(`hash:${userId}`);
  if (cachedHash) {
      if (c.req.header('If-None-Match') === cachedHash) {
          return c.body(null, 304);
      }
      c.header('ETag', cachedHash);
      return c.json({ hash: cachedHash, source: 'cache' });
  }

  // 2. Computed Hash (D1)
  const hash = await calculateDataHash(c.env.DB, userId);
  
  // Update Cache (TTL 5 minutes so we don't hammer D1 on heavy polling)
  await c.env.GO2DO_KV.put(`hash:${userId}`, hash, { expirationTtl: 300 });

  if (c.req.header('If-None-Match') === hash) {
      return c.body(null, 304);
  }

  c.header('ETag', hash);
  return c.json({ hash, source: 'db' });
});

app.post('/sync', async (c) => {
  const userId = c.get('userId');
  let body: SyncRequest;
  
  try {
    body = await c.req.json<SyncRequest>();
  } catch (e) {
    return c.json({ error: 'invalid json, get it together client' }, 400);
  }

  const { last_sync, changes } = body;
  const db = c.env.DB;

  // 1. Apply Changes (Write)
  if (changes && changes.length > 0) {
      try {
          await batchWriteTasks(db, userId, changes);
          // Invalidate Cache immediately on write
          await c.env.GO2DO_KV.delete(`hash:${userId}`);
      } catch (e) {
          console.error('sync write failed:', e);
          return c.json({ error: 'database write failed', details: (e as Error).message }, 500);
      }
  }

  // 2. Fetch Updates (Read)
  // Optimization: If client sent changes, they likely need the latest state anyway.
  // If no changes sent, and client provided `If-None-Match`, they should have used GET /sync/check first.
  
  const updates = await getChanges(db, userId, last_sync);

  return c.json({
    updates,
    server_time: Date.now(),
  });
});

export default app;
