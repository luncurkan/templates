# luncurkan.dev templates

Ready-to-deploy project templates for [luncurkan.dev](https://luncurkan.dev) deployment platform.

## Naming Convention

Templates follow the pattern: `{language}-{framework}-{usecase}`

## Available Templates

### URL Shorteners

| Template | Stack | Database | Docker | Features |
|----------|-------|----------|--------|----------|
| [rust-axum-url-shortener](./rust-axum-url-shortener) | Rust, Axum 0.8 | PostgreSQL | ❌ | Custom codes, click tracking, stats API, web UI |
| [go-fiber-url-shortener](./go-fiber-url-shortener) | Go 1.21+, Fiber v2, HTMX | In-memory | ✅ | Visit tracking, reactive UI |

### REST API Starters

| Template | Stack | Database | Docker | Features |
|----------|-------|----------|--------|----------|
| [node-hono-rest-api](./node-hono-rest-api) | Node.js, Hono, TypeScript | SQLite | ✅ | Zod validation, CRUD operations |
| [python-fastapi-rest-api](./python-fastapi-rest-api) | Python, FastAPI, SQLAlchemy | SQLite | ✅ | Pydantic models, async, CRUD operations |
| [bun-elysia-rest-api](./bun-elysia-rest-api) | Bun, Elysia, TypeScript | SQLite | ✅ | Validation, CRUD operations |

### Authentication Services

| Template | Stack | Database | Docker | Features |
|----------|-------|----------|--------|----------|
| [node-hono-auth](./node-hono-auth) | Node.js, Hono, TypeScript | In-memory | ✅ | JWT, bcrypt, refresh tokens |
| [python-fastapi-auth](./python-fastapi-auth) | Python, FastAPI | In-memory | ✅ | JWT, passlib, OAuth2 |
| [go-fiber-auth](./go-fiber-auth) | Go, Fiber v2 | In-memory | ✅ | JWT middleware, bcrypt |

### WebSocket/Realtime

| Template | Stack | Database | Docker | Features |
|----------|-------|----------|--------|----------|
| [node-hono-websocket](./node-hono-websocket) | Node.js, Hono, TypeScript | None | ✅ | WebSocket upgrade, chat rooms |
| [bun-elysia-websocket](./bun-elysia-websocket) | Bun, Elysia | None | ✅ | Native Bun WebSocket, rooms |
| [go-fiber-websocket](./go-fiber-websocket) | Go, Fiber v2 | None | ✅ | WebSocket middleware, broadcast |

## Usage

Deploy any template directly to luncurkan.dev or use as a starting point for your own projects.

## Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `PORT` | Yes (Docker) | Default port is `3000`. All Dockerized templates expose this port. |
| `DATABASE_URL` | Yes (PostgreSQL) | PostgreSQL connection string. Example: `postgres://user:password@host:5432/dbname` |

## Notes

- **Dockerfile**: If a template includes a Dockerfile, [luncurkan.dev](https://luncurkan.dev) will use it as a reference for deployment and will not auto-generate a Dockerfile.

## License

MIT
