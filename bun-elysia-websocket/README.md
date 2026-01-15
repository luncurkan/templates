# bun-elysia-websocket

A minimal WebSocket chat server built with Bun and Elysia.

## Stack

- **Runtime:** Bun
- **Framework:** Elysia
- **Language:** TypeScript

## Quick Start

```bash
# Install dependencies
bun install

# Copy environment file
cp .env.example .env

# Run development server
bun dev
```

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `HOST` | Server host | `0.0.0.0` |
| `PORT` | Server port | `3000` |

## Endpoints

| Type | Endpoint | Description |
|------|----------|-------------|
| HTTP | `GET /` | Web UI for chat |
| HTTP | `GET /health` | Health check |
| WS | `GET /ws` | WebSocket connection |
| WS | `GET /ws/:room` | WebSocket connection to specific room |

## WebSocket Messages

### Send Message

```json
{
  "type": "message",
  "content": "Hello, world!"
}
```

### Set Username

```json
{
  "type": "username",
  "username": "Alice"
}
```

### Received Message

```json
{
  "type": "message",
  "username": "Alice",
  "content": "Hello, world!",
  "timestamp": "2024-01-01T12:00:00.000Z"
}
```

## Testing with wscat

```bash
# Install wscat
npm install -g wscat

# Connect to default room
wscat -c ws://localhost:3000/ws

# Connect to specific room
wscat -c ws://localhost:3000/ws/my-room
```

## Docker

```bash
# Build and run
docker-compose up --build

# Or build manually
docker build -t bun-elysia-websocket .
docker run -p 3000:3000 bun-elysia-websocket
```

## License

MIT
