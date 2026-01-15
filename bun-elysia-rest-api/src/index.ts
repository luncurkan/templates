import { Elysia, t } from "elysia";
import Database from "better-sqlite3";

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

// Types
interface Item {
  id: number;
  name: string;
  description: string | null;
  created_at: string;
  updated_at: string;
}

// App
const app = new Elysia()
  // Health check
  .get("/health", () => ({ status: "ok" }))

  // List items
  .get("/api/items", () => {
    const items = db.prepare("SELECT * FROM items ORDER BY created_at DESC").all() as Item[];
    return items;
  })

  // Get item by ID
  .get(
    "/api/items/:id",
    ({ params, error }) => {
      const item = db.prepare("SELECT * FROM items WHERE id = ?").get(params.id) as Item | undefined;
      if (!item) {
        return error(404, { error: "Item not found" });
      }
      return item;
    },
    {
      params: t.Object({
        id: t.Numeric(),
      }),
    }
  )

  // Create item
  .post(
    "/api/items",
    ({ body }) => {
      const stmt = db.prepare("INSERT INTO items (name, description) VALUES (?, ?)");
      const info = stmt.run(body.name, body.description || null);
      const item = db.prepare("SELECT * FROM items WHERE id = ?").get(info.lastInsertRowid) as Item;
      return item;
    },
    {
      body: t.Object({
        name: t.String({ minLength: 1, maxLength: 255 }),
        description: t.Optional(t.String({ maxLength: 1000 })),
      }),
    }
  )

  // Update item
  .put(
    "/api/items/:id",
    ({ params, body, error }) => {
      const existing = db.prepare("SELECT * FROM items WHERE id = ?").get(params.id) as Item | undefined;
      if (!existing) {
        return error(404, { error: "Item not found" });
      }

      const stmt = db.prepare(`
        UPDATE items
        SET name = COALESCE(?, name),
            description = COALESCE(?, description),
            updated_at = CURRENT_TIMESTAMP
        WHERE id = ?
      `);
      stmt.run(body.name || null, body.description || null, params.id);

      const item = db.prepare("SELECT * FROM items WHERE id = ?").get(params.id) as Item;
      return item;
    },
    {
      params: t.Object({
        id: t.Numeric(),
      }),
      body: t.Object({
        name: t.Optional(t.String({ minLength: 1, maxLength: 255 })),
        description: t.Optional(t.String({ maxLength: 1000 })),
      }),
    }
  )

  // Delete item
  .delete(
    "/api/items/:id",
    ({ params, error }) => {
      const existing = db.prepare("SELECT * FROM items WHERE id = ?").get(params.id) as Item | undefined;
      if (!existing) {
        return error(404, { error: "Item not found" });
      }

      db.prepare("DELETE FROM items WHERE id = ?").run(params.id);
      return { message: "Item deleted" };
    },
    {
      params: t.Object({
        id: t.Numeric(),
      }),
    }
  )

  .listen({ hostname: HOST, port: PORT });

console.log(`Server running on http://${HOST}:${PORT}`);
