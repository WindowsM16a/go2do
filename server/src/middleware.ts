import { MiddlewareHandler } from 'hono';
import { verifyToken } from './auth';
import * as cookie from 'cookie';

export const authMiddleware: MiddlewareHandler = async (c, next) => {
  const cookieHeader = c.req.header('Cookie');
  const cookies = cookie.parse(cookieHeader || '');
  const token = cookies.auth_token;

  if (!token) {
    // If no cookie, try 'Authorization: Bearer <token>' for CLI/API clients
    const authHeader = c.req.header('Authorization');
    if (authHeader && authHeader.startsWith('Bearer ')) {
       const bearerToken = authHeader.split(' ')[1];
       const userId = await verifyToken(bearerToken, c.env.JWT_SECRET);
       if (userId) {
          c.set('userId', userId);
          return await next();
       }
    }
    return c.json({ error: 'Unauthorized: No token provided' }, 401);
  }

  const userId = await verifyToken(token, c.env.JWT_SECRET);
  if (!userId) {
    return c.json({ error: 'Unauthorized: Invalid token' }, 401);
  }

  c.set('userId', userId);
  await next();
};
