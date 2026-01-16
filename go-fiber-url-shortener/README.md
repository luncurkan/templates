# go-fiber-url-shortener

A simple URL shortener built with [Go Fiber](https://gofiber.io/) featuring an HTMX-powered web UI and in-memory storage.

## Features

- Shorten long URLs
- Redirect to original URLs
- View URL statistics (visit count, creation time)
- List all shortened URLs
- HTMX-powered web interface
- JSON API for programmatic access
- In-memory storage (no database required)

## Requirements

- Go 1.21+ (or Docker)

## Getting Started

### Using Docker

```bash
# Build the image
docker build -t url-shortener .

# Run the container
docker run -p 3000:3000 url-shortener
```

### Using Go

```bash
# Install dependencies
go mod tidy

# Run the server
go run main.go
```

The server will start on `http://localhost:3000`.

## Web Interface

Visit `http://localhost:3000` in your browser to access the web UI where you can:

- Shorten URLs with a simple form
- View all shortened URLs
- Copy short URLs to clipboard
- See visit statistics

## API Endpoints

### Health Check

```bash
GET /api/health
```

**Response:**

```json
{
  "status": "ok",
  "message": "Go Fiber URL Shortener is running"
}
```

### Shorten URL

```bash
POST /shorten
Content-Type: application/json

{
  "url": "https://example.com/very/long/url"
}
```

**Response:**

```json
{
  "short_url": "http://localhost:3000/abc123",
  "short_code": "abc123",
  "original_url": "https://example.com/very/long/url"
}
```

### Redirect to Original URL

```bash
GET /:code
```

Redirects to the original URL with a 301 status code.

### Get URL Statistics

```bash
GET /:code/stats
```

**Response:**

```json
{
  "original_url": "https://example.com/very/long/url",
  "short_code": "abc123",
  "created_at": "2024-01-14T12:00:00Z",
  "visits": 42
}
```

### List All URLs

```bash
GET /api/urls
```

**Response:**

```json
{
  "urls": [
    {
      "original_url": "https://example.com",
      "short_code": "abc123",
      "created_at": "2024-01-14T12:00:00Z",
      "visits": 10
    }
  ],
  "count": 1
}
```

## Example Usage

```bash
# Shorten a URL
curl -X POST http://localhost:3000/shorten \
  -H "Content-Type: application/json" \
  -d '{"url": "https://github.com"}'

# Get statistics
curl http://localhost:3000/abc123/stats

# List all URLs
curl http://localhost:3000/api/urls
```

## Notes

- Data is stored in-memory and will be lost when the server restarts
- For production use, consider adding a database backend
