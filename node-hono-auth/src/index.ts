import { serve } from "@hono/node-server";
import { Hono } from "hono";
import { cors } from "hono/cors";
import { logger } from "hono/logger";
import Database from "better-sqlite3";
import bcrypt from "bcryptjs";
import * as jose from "jose";
import { z } from "zod";

const app = new Hono();

// Environment
const HOST = process.env.HOST || "0.0.0.0";
const PORT = parseInt(process.env.PORT || "3000");
const DATABASE_PATH = process.env.DATABASE_PATH || "./data.db";
const JWT_SECRET = new TextEncoder().encode(process.env.JWT_SECRET || "change-me");
const JWT_EXPIRES_IN = process.env.JWT_EXPIRES_IN || "15m";
const REFRESH_TOKEN_EXPIRES_IN = process.env.REFRESH_TOKEN_EXPIRES_IN || "7d";

// Database setup
const db = new Database(DATABASE_PATH);
db.exec(`
  CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    email TEXT UNIQUE NOT NULL,
    password TEXT NOT NULL,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
  )
`);

// Schemas
const registerSchema = z.object({
  email: z.string().email(),
  password: z.string().min(8).max(100),
});

const loginSchema = z.object({
  email: z.string().email(),
  password: z.string(),
});

const refreshSchema = z.object({
  refreshToken: z.string(),
});

// Types
interface User {
  id: number;
  email: string;
  password: string;
  created_at: string;
}

// Helpers
function parseExpiry(expiry: string): string {
  const match = expiry.match(/^(\d+)([smhd])$/);
  if (!match) return "15m";
  const [, value, unit] = match;
  const units: Record<string, string> = { s: "seconds", m: "minutes", h: "hours", d: "days" };
  return `${value} ${units[unit]}`;
}

async function generateTokens(userId: number, email: string) {
  const accessToken = await new jose.SignJWT({ userId, email })
    .setProtectedHeader({ alg: "HS256" })
    .setExpirationTime(parseExpiry(JWT_EXPIRES_IN))
    .sign(JWT_SECRET);

  const refreshToken = await new jose.SignJWT({ userId, type: "refresh" })
    .setProtectedHeader({ alg: "HS256" })
    .setExpirationTime(parseExpiry(REFRESH_TOKEN_EXPIRES_IN))
    .sign(JWT_SECRET);

  return { accessToken, refreshToken };
}

// Middleware
app.use("*", logger());
app.use("*", cors());

// Health check
app.get("/health", (c) => c.json({ status: "ok" }));

// Register
app.post("/api/auth/register", async (c) => {
  const body = await c.req.json();
  const result = registerSchema.safeParse(body);

  if (!result.success) {
    return c.json({ error: result.error.flatten() }, 400);
  }

  const { email, password } = result.data;

  const existing = db.prepare("SELECT id FROM users WHERE email = ?").get(email);
  if (existing) {
    return c.json({ error: "Email already registered" }, 409);
  }

  const hashedPassword = await bcrypt.hash(password, 10);
  const stmt = db.prepare("INSERT INTO users (email, password) VALUES (?, ?)");
  const info = stmt.run(email, hashedPassword);

  const tokens = await generateTokens(Number(info.lastInsertRowid), email);

  return c.json({
    ...tokens,
    user: { id: info.lastInsertRowid, email },
  }, 201);
});

// Login
app.post("/api/auth/login", async (c) => {
  const body = await c.req.json();
  const result = loginSchema.safeParse(body);

  if (!result.success) {
    return c.json({ error: result.error.flatten() }, 400);
  }

  const { email, password } = result.data;

  const user = db.prepare("SELECT * FROM users WHERE email = ?").get(email) as User | undefined;
  if (!user) {
    return c.json({ error: "Invalid credentials" }, 401);
  }

  const valid = await bcrypt.compare(password, user.password);
  if (!valid) {
    return c.json({ error: "Invalid credentials" }, 401);
  }

  const tokens = await generateTokens(user.id, user.email);

  return c.json({
    ...tokens,
    user: { id: user.id, email: user.email },
  });
});

// Refresh token
app.post("/api/auth/refresh", async (c) => {
  const body = await c.req.json();
  const result = refreshSchema.safeParse(body);

  if (!result.success) {
    return c.json({ error: result.error.flatten() }, 400);
  }

  try {
    const { payload } = await jose.jwtVerify(result.data.refreshToken, JWT_SECRET);

    if (payload.type !== "refresh") {
      return c.json({ error: "Invalid token type" }, 401);
    }

    const user = db.prepare("SELECT * FROM users WHERE id = ?").get(payload.userId) as User | undefined;
    if (!user) {
      return c.json({ error: "User not found" }, 401);
    }

    const tokens = await generateTokens(user.id, user.email);
    return c.json(tokens);
  } catch {
    return c.json({ error: "Invalid or expired token" }, 401);
  }
});

// Get current user (protected)
app.get("/api/auth/me", async (c) => {
  const authHeader = c.req.header("Authorization");
  if (!authHeader?.startsWith("Bearer ")) {
    return c.json({ error: "Missing authorization header" }, 401);
  }

  const token = authHeader.slice(7);

  try {
    const { payload } = await jose.jwtVerify(token, JWT_SECRET);

    const user = db.prepare("SELECT id, email, created_at FROM users WHERE id = ?").get(payload.userId) as Omit<User, "password"> | undefined;
    if (!user) {
      return c.json({ error: "User not found" }, 401);
    }

    return c.json({ user });
  } catch {
    return c.json({ error: "Invalid or expired token" }, 401);
  }
});

// Start server
console.log(`Server starting on http://${HOST}:${PORT}`);
serve({ fetch: app.fetch, hostname: HOST, port: PORT });
