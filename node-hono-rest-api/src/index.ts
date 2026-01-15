import { serve } from "@hono/node-server";
import { Hono } from "hono";
import { cors } from "hono/cors";
import { logger } from "hono/logger";
import Database from "better-sqlite3";
import { z } from "zod";

const app = new Hono();

// Environment
const HOST = process.env.HOST || "0.0.0.0";
const PORT = parseInt(process.env.PORT || "3000");
const DATABASE_PATH = process.env.DATABASE_PATH || "./data.db";

// Database setup
const db = new Database(DATABASE_PATH);
db.exec(`
  CREATE TABLE IF NOT EXISTS items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    description TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP
  )
`);

// Schemas
const createItemSchema = z.object({
  name: z.string().min(1).max(255),
  description: z.string().max(1000).optional(),
});

const updateItemSchema = z.object({
  name: z.string().min(1).max(255).optional(),
  description: z.string().max(1000).optional(),
});

// Types
interface Item {
  id: number;
  name: string;
  description: string | null;
  created_at: string;
  updated_at: string;
}

// Middleware
app.use("*", logger());
app.use("*", cors());

// Health check
app.get("/health", (c) => c.json({ status: "ok" }));

// List items
app.get("/api/items", (c) => {
  const items = db.prepare("SELECT * FROM items ORDER BY created_at DESC").all() as Item[];
  return c.json(items);
});

// Get item by ID
app.get("/api/items/:id", (c) => {
  const id = parseInt(c.req.param("id"));
  const item = db.prepare("SELECT * FROM items WHERE id = ?").get(id) as Item | undefined;

  if (!item) {
    return c.json({ error: "Item not found" }, 404);
  }

  return c.json(item);
});

// Create item
app.post("/api/items", async (c) => {
  const body = await c.req.json();
  const result = createItemSchema.safeParse(body);

  if (!result.success) {
    return c.json({ error: result.error.flatten() }, 400);
  }

  const { name, description } = result.data;
  const stmt = db.prepare("INSERT INTO items (name, description) VALUES (?, ?)");
  const info = stmt.run(name, description || null);

  const item = db.prepare("SELECT * FROM items WHERE id = ?").get(info.lastInsertRowid) as Item;
  return c.json(item, 201);
});

// Update item
app.put("/api/items/:id", async (c) => {
  const id = parseInt(c.req.param("id"));
  const existing = db.prepare("SELECT * FROM items WHERE id = ?").get(id) as Item | undefined;

  if (!existing) {
    return c.json({ error: "Item not found" }, 404);
  }

  const body = await c.req.json();
  const result = updateItemSchema.safeParse(body);

  if (!result.success) {
    return c.json({ error: result.error.flatten() }, 400);
  }

  const { name, description } = result.data;
  const stmt = db.prepare(`
    UPDATE items
    SET name = COALESCE(?, name),
        description = COALESCE(?, description),
        updated_at = CURRENT_TIMESTAMP
    WHERE id = ?
  `);
  stmt.run(name || null, description || null, id);

  const item = db.prepare("SELECT * FROM items WHERE id = ?").get(id) as Item;
  return c.json(item);
});

// Delete item
app.delete("/api/items/:id", (c) => {
  const id = parseInt(c.req.param("id"));
  const existing = db.prepare("SELECT * FROM items WHERE id = ?").get(id) as Item | undefined;

  if (!existing) {
    return c.json({ error: "Item not found" }, 404);
  }

  db.prepare("DELETE FROM items WHERE id = ?").run(id);
  return c.json({ message: "Item deleted" });
});

// Start server
console.log(`Server starting on http://${HOST}:${PORT}`);
serve({ fetch: app.fetch, hostname: HOST, port: PORT });
