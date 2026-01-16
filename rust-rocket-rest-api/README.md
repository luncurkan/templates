# rust-rocket-rest-api

A simple REST API starter template built with Rust and the Rocket framework.

## Stack

- **Runtime**: Rust 1.85+
- **Framework**: Rocket 0.5
- **Database**: PostgreSQL (via SQLx)
- **Language**: Rust (Edition 2021)

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
   cargo run
   ```

4. The API will be available at `http://localhost:3000`

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `HOST` | Server host address | `0.0.0.0` |
| `PORT` | Server port | `3000` |
| `DATABASE_URL` | PostgreSQL connection string | `postgres://postgres:postgres@localhost:5432/app` |

## API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/health` | Health check |
| `GET` | `/api/items` | Get all items |
| `GET` | `/api/items/:id` | Get item by ID |
| `POST` | `/api/items` | Create new item |
| `PUT` | `/api/items/:id` | Update item |
| `DELETE` | `/api/items/:id` | Delete item |

## Request/Response Examples

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
docker build -t rust-rocket-rest-api .
docker run -p 3000:3000 -e DATABASE_URL=postgres://user:pass@host:5432/db rust-rocket-rest-api
```

## License

MIT
