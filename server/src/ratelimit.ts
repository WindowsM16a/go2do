import { Context, Next } from 'hono';

export async function rateLimit(c: Context, next: Next) {
    const userId = c.get('userId') || c.req.header('cf-connecting-ip') || 'anon';
    const key = `ratelimit:${userId}`;
    
    // Limits
    const LIMIT = 20; // 20 requests
    const WINDOW = 60; // per 60 seconds

    const kv = c.env.GO2DO_KV;
    
    // Get current count
    const countStr = await kv.get(key);
    let count = countStr ? parseInt(countStr) : 0;

    if (count >= LIMIT) {
        return c.json({ error: 'calm down. rate limit exceeded.' }, 429);
    }

    // Increment
    // Ideally this would be atomic, but KV eventual consistency is fine for rate limiting
    // We set TTL to window size if it's a new key
    if (count === 0) {
        await kv.put(key, (count + 1).toString(), { expirationTtl: WINDOW });
    } else {
        await kv.put(key, (count + 1).toString(), { expirationTtl: WINDOW }); // Resetting TTL is lazy but works for sliding window approx
    }

    await next();
}
