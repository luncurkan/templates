# go-fiber-auth

A minimal JWT authentication service built with Go, Fiber, and SQLite.

## Stack

- **Runtime:** Go 1.21+
- **Framework:** Fiber v2
- **Database:** SQLite
- **Auth:** golang-jwt, bcrypt

## Quick Start

```bash
# Download dependencies
go mod download

# Copy environment file
cp .env.example .env

# Run development server
go run main.go
```

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `HOST` | Server host | `0.0.0.0` |
| `PORT` | Server port | `3000` |
| `DATABASE_PATH` | SQLite database path | `./data.db` |
| `JWT_SECRET` | Secret key for JWT signing | (required) |
| `ACCESS_TOKEN_EXPIRE_MINUTES` | Access token expiry in minutes | `15` |
| `REFRESH_TOKEN_EXPIRE_DAYS` | Refresh token expiry in days | `7` |

## API Endpoints

| Method | Endpoint | Description | Auth |
|--------|----------|-------------|------|
| `GET` | `/health` | Health check | No |
| `POST` | `/api/auth/register` | Register new user | No |
| `POST` | `/api/auth/login` | Login user | No |
| `POST` | `/api/auth/refresh` | Refresh access token | No |
| `GET` | `/api/auth/me` | Get current user | Yes |

## Request/Response Examples

### Register

```bash
curl -X POST http://localhost:3000/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email": "user@example.com", "password": "password123"}'
```

### Login

```bash
curl -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email": "user@example.com", "password": "password123"}'
```

Response:
```json
{
  "access_token": "eyJhbGc...",
  "refresh_token": "eyJhbGc...",
  "user": { "id": 1, "email": "user@example.com" }
}
```

### Protected Route

```bash
curl http://localhost:3000/api/auth/me \
  -H "Authorization: Bearer eyJhbGc..."
```

### Refresh Token

```bash
curl -X POST http://localhost:3000/api/auth/refresh \
  -H "Content-Type: application/json" \
  -d '{"refresh_token": "eyJhbGc..."}'
```

## Docker

```bash
# Build and run
docker-compose up --build

# Or build manually
docker build -t go-fiber-auth .
docker run -p 3000:3000 -e JWT_SECRET=your-secret go-fiber-auth
```

## License

MIT
