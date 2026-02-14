import { Bindings } from './types';
import * as bcrypt from 'bcryptjs';
import * as jose from 'jose';

const SALT_ROUNDS = 10;
const JWT_ALG = 'HS256';

export async function hashPassword(password: string): Promise<string> {
  return await bcrypt.hash(password, SALT_ROUNDS);
}

export async function verifyPassword(password: string, hash: string): Promise<boolean> {
  return await bcrypt.compare(password, hash);
}

export async function createToken(userId: string, secret: string): Promise<string> {
  const secretKey = new TextEncoder().encode(secret);
  return await new jose.SignJWT({ sub: userId })
    .setProtectedHeader({ alg: JWT_ALG })
    .setIssuedAt()
    .setExpirationTime('30d') // Long-lived session (Retention friendly)
    .sign(secretKey);
}

export async function verifyToken(token: string, secret: string): Promise<string | null> {
  try {
    const secretKey = new TextEncoder().encode(secret);
    const { payload } = await jose.jwtVerify(token, secretKey);
    return payload.sub || null;
  } catch (e) {
    return null; // Invalid or expired
  }
}
