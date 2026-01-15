# node-hono-rest-api

A minimal REST API template built with Node.js, Hono, and SQLite.

## Stack

- **Runtime:** Node.js 22+
- **Framework:** Hono
- **Database:** SQLite (better-sqlite3)
- **Validation:** Zod
- **Language:** TypeScript

## Quick Start

```bash
# Install dependencies
npm install

# Copy environment file
cp .env.example .env

# Run development server
npm run dev
```

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `HOST` | Server host | `0.0.0.0` |
| `PORT` | Server port | `3000` |
| `DATABASE_PATH` | SQLite database path | `./data.db` |

## API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/health` | Health check |
| `GET` | `/api/items` | List all items |
| `GET` | `/api/items/:id` | Get item by ID |
| `POST` | `/api/items` | Create new item |
| `PUT` | `/api/items/:id` | Update item |
| `DELETE` | `/api/items/:id` | Delete item |

## Request/Response Examples

### Create Item

```bash
curl -X POST http://localhost:3000/api/items \
  -H "Content-Type: application/json" \
  -d '{"name": "Example Item", "description": "A sample item"}'
```

### List Items

```bash
curl http://localhost:3000/api/items
```

## Docker

```bash
# Build and run
docker-compose up --build

# Or build manually
docker build -t node-hono-rest-api .
docker run -p 3000:3000 node-hono-rest-api
```

## License

MIT
