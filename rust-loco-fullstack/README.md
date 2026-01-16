# rust-loco-fullstack

A fullstack web application template built with Rust, Loco framework, and HTMX for dynamic server-rendered UI.

## Stack

- **Runtime**: Rust 1.85+
- **Framework**: Loco 0.14 (Rails-like for Rust)
- **ORM**: SeaORM
- **Database**: PostgreSQL
- **Templating**: Tera
- **Frontend**: HTMX 2.0 (dynamic HTML over the wire)

## Quick Start

1. Copy `.env.example` to `.env`:

   ```bash
   cp .env.example .env
   ```

2. Start PostgreSQL (using Docker):

   ```bash
   docker run -d --name postgres -e POSTGRES_PASSWORD=postgres -e POSTGRES_DB=app -p 5432:5432 postgres:16-alpine
   ```

3. Run the application:

   ```bash
   cargo run -- start
   ```

4. The app will be available at `http://localhost:3000`

## Loco CLI Commands

```bash
# Start the server
cargo run -- start

# Run migrations
cargo run -- db migrate

# Reset database
cargo run -- db reset

# Generate controller
cargo run -- generate controller <name>

# Generate model
cargo run -- generate model <name>
```

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `LOCO_ENV` | Environment (development/production) | `development` |
| `PORT` | Server port | `3000` |
| `DATABASE_URL` | PostgreSQL connection string | `postgres://postgres:postgres@localhost:5432/app` |

## Project Structure

```
.
├── assets/
│   ├── static/             # Static files (CSS, JS, images)
│   └── views/              # Tera HTML templates
│       ├── layouts/
│       │   └── base.html   # Base layout template
│       └── items/
│           ├── index.html  # Items list page
│           ├── _row.html   # Item row partial
│           └── _edit_row.html  # Edit row partial
├── config/
│   ├── development.yaml    # Development config
│   └── production.yaml     # Production config
├── migration/
│   └── src/                # Database migrations
├── src/
│   ├── app.rs              # Application hooks
│   ├── controllers/        # Route handlers
│   │   ├── health.rs       # Health check endpoint
│   │   ├── items.rs        # REST API endpoints
│   │   └── items_ui.rs     # HTMX UI endpoints
│   ├── initializers/
│   │   └── view_engine.rs  # Tera view engine setup
│   ├── models/             # Database models
│   │   └── _entities/
│   │       └── items.rs
│   ├── views/              # View rendering helpers
│   │   └── items.rs
│   ├── lib.rs
│   └── main.rs
├── Cargo.toml
└── README.md
```

## Endpoints

### Web UI (HTMX)

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/` | Items list page (server-rendered) |
| `POST` | `/` | Create item (returns HTML row) |
| `GET` | `/:id/edit` | Get edit form row |
| `GET` | `/:id/row` | Get item row (cancel edit) |
| `PUT` | `/:id` | Update item (returns HTML row) |
| `DELETE` | `/:id` | Delete item |

### REST API (JSON)

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/health` | Health check |
| `GET` | `/api/items` | Get all items |
| `GET` | `/api/items/:id` | Get item by ID |
| `POST` | `/api/items` | Create new item |
| `PUT` | `/api/items/:id` | Update item |
| `DELETE` | `/api/items/:id` | Delete item |

## Request/Response Examples (REST API)

### Create Item

```bash
curl -X POST http://localhost:3000/api/items \
  -H "Content-Type: application/json" \
  -d '{"name": "My Item", "description": "A sample item"}'
```

### Get All Items

```bash
curl http://localhost:3000/api/items
```

### Get Item by ID

```bash
curl http://localhost:3000/api/items/<id>
```

### Update Item

```bash
curl -X PUT http://localhost:3000/api/items/<id> \
  -H "Content-Type: application/json" \
  -d '{"name": "Updated Name"}'
```

### Delete Item

```bash
curl -X DELETE http://localhost:3000/api/items/<id>
```

## Docker

Build and run with Docker Compose (includes PostgreSQL):

```bash
docker-compose up --build
```

Or build manually:

```bash
docker build -t rust-loco-fullstack .
docker run -p 3000:3000 -e DATABASE_URL=postgres://user:pass@host:5432/db rust-loco-fullstack
```

## License

MIT
