# luncurkan.dev templates

Ready-to-deploy project templates for [luncurkan.dev](https://luncurkan.dev) deployment platform.

## Naming Convention

Templates follow the pattern: `{language}-{framework}-{usecase}`

## Available Templates

### URL Shorteners

| Template | Stack | Features |
|----------|-------|----------|
| [rust-axum-url-shortener](./rust-axum-url-shortener) | Rust, Axum 0.8, PostgreSQL | Custom codes, click tracking, stats API, web UI |
| [go-fiber-url-shortener](./go-fiber-url-shortener) | Go 1.21+, Fiber v2, HTMX | In-memory storage, visit tracking, reactive UI |

### REST API Starters

| Template | Stack | Features |
|----------|-------|----------|
| [node-hono-rest-api](./node-hono-rest-api) | Node.js, Hono, TypeScript | Zod validation, SQLite, CRUD operations |
| [python-fastapi-rest-api](./python-fastapi-rest-api) | Python, FastAPI, SQLAlchemy | Pydantic models, async, CRUD operations |
| [bun-elysia-rest-api](./bun-elysia-rest-api) | Bun, Elysia, TypeScript | Validation, SQLite, CRUD operations |

### Authentication Services

| Template | Stack | Features |
|----------|-------|----------|
| [node-hono-auth](./node-hono-auth) | Node.js, Hono, TypeScript | JWT, bcrypt, refresh tokens |
| [python-fastapi-auth](./python-fastapi-auth) | Python, FastAPI | JWT, passlib, OAuth2 |
| [go-fiber-auth](./go-fiber-auth) | Go, Fiber v2 | JWT middleware, bcrypt |

### WebSocket/Realtime

| Template | Stack | Features |
|----------|-------|----------|
| [node-hono-websocket](./node-hono-websocket) | Node.js, Hono, TypeScript | WebSocket upgrade, chat rooms |
| [bun-elysia-websocket](./bun-elysia-websocket) | Bun, Elysia | Native Bun WebSocket, rooms |
| [go-fiber-websocket](./go-fiber-websocket) | Go, Fiber v2 | WebSocket middleware, broadcast |

## Usage

Deploy any template directly to luncurkan.dev or use as a starting point for your own projects.

## License

MIT
