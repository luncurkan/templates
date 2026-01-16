# rust-axum-url-shortener

A fast, lightweight URL shortener service built with Rust and Axum.

## Features

- Create short URLs with auto-generated or custom codes
- Redirect short URLs to original destinations
- Track click statistics for each shortened URL
- RESTful API for programmatic access
- Built-in web UI for easy URL shortening
- PostgreSQL database with automatic migrations

## Requirements

- Rust 2024 edition
- PostgreSQL database

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection string | **Required** |
| `HOST` | Server bind address | `0.0.0.0` |
| `PORT` | Server port | `3000` |
| `BASE_URL` | Public URL for generated short links | `http://{HOST}:{PORT}` |
| `DATABASE_MAX_CONNECTIONS` | Max database pool connections | `5` |

## Getting Started

1. Create a `.env` file:

```env
DATABASE_URL=postgres://user:password@localhost:5432/shorturl
HOST=0.0.0.0
PORT=3000
BASE_URL=http://localhost:3000
```

1. Run the application:

```bash
cargo run
```

The server will automatically run database migrations on startup.

## API Endpoints

### Health Check

```
GET /health
```

Returns service health status.

### Web UI

```
GET /
```

Interactive web interface for creating short URLs.

### Create Short URL

```
POST /api/urls
Content-Type: application/json

{
  "url": "https://example.com/very/long/url",
  "custom_code": "my-link"  // optional
}
```

**Response:**

```json
{
  "id": "uuid",
  "short_code": "my-link",
  "short_url": "http://localhost:3000/my-link",
  "original_url": "https://example.com/very/long/url"
}
```

### List URLs

```
GET /api/urls
```

Returns the 100 most recent shortened URLs.

### Get URL Stats

```
GET /api/urls/{code}
```

Returns statistics for a specific short URL including click count.

### Delete URL

```
DELETE /api/urls/{code}
```

Deletes a shortened URL.

### Redirect

```
GET /{code}
```

Redirects to the original URL and increments the click counter.

## Database Schema

The `urls` table stores all shortened URLs:

| Column | Type | Description |
|--------|------|-------------|
| `id` | UUID | Primary key |
| `short_code` | VARCHAR(10) | Unique short code |
| `original_url` | TEXT | Original destination URL |
| `clicks` | BIGINT | Number of redirects |
| `created_at` | TIMESTAMPTZ | Creation timestamp |
| `updated_at` | TIMESTAMPTZ | Last update timestamp |

## License

MIT
