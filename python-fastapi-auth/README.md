# python-fastapi-auth

A minimal JWT authentication service built with Python, FastAPI, and SQLite.

## Stack

- **Runtime:** Python 3.12+
- **Framework:** FastAPI
- **Database:** SQLite (async with aiosqlite)
- **ORM:** SQLAlchemy 2.0
- **Auth:** python-jose (JWT), passlib (bcrypt)
- **Validation:** Pydantic

## Quick Start

```bash
# Create virtual environment
python -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate

# Install dependencies
pip install -r requirements.txt

# Copy environment file
cp .env.example .env

# Run development server
uvicorn app.main:app --reload
```

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `HOST` | Server host | `0.0.0.0` |
| `PORT` | Server port | `3000` |
| `DATABASE_URL` | SQLAlchemy database URL | `sqlite+aiosqlite:///./data.db` |
| `JWT_SECRET` | Secret key for JWT signing | (required) |
| `JWT_ALGORITHM` | JWT algorithm | `HS256` |
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
  "token_type": "bearer",
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

## Interactive Docs

- Swagger UI: http://localhost:3000/docs
- ReDoc: http://localhost:3000/redoc

## Docker

```bash
# Build and run
docker-compose up --build

# Or build manually
docker build -t python-fastapi-auth .
docker run -p 3000:3000 -e JWT_SECRET=your-secret python-fastapi-auth
```

## License

MIT
