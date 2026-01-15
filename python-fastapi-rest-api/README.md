# python-fastapi-rest-api

A minimal REST API template built with Python, FastAPI, and SQLite.

## Stack

- **Runtime:** Python 3.12+
- **Framework:** FastAPI
- **Database:** SQLite (async with aiosqlite)
- **ORM:** SQLAlchemy 2.0
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
| `PORT` | Server port | `8000` |
| `DATABASE_URL` | SQLAlchemy database URL | `sqlite+aiosqlite:///./data.db` |

## API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/health` | Health check |
| `GET` | `/api/items` | List all items |
| `GET` | `/api/items/{id}` | Get item by ID |
| `POST` | `/api/items` | Create new item |
| `PUT` | `/api/items/{id}` | Update item |
| `DELETE` | `/api/items/{id}` | Delete item |

## Request/Response Examples

### Create Item

```bash
curl -X POST http://localhost:8000/api/items \
  -H "Content-Type: application/json" \
  -d '{"name": "Example Item", "description": "A sample item"}'
```

### List Items

```bash
curl http://localhost:8000/api/items
```

## Interactive Docs

- Swagger UI: http://localhost:8000/docs
- ReDoc: http://localhost:8000/redoc

## Docker

```bash
# Build and run
docker-compose up --build

# Or build manually
docker build -t python-fastapi-rest-api .
docker run -p 8000:8000 python-fastapi-rest-api
```

## License

MIT
