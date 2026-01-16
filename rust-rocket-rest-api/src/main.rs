#[macro_use]
extern crate rocket;

use rocket::serde::json::Json;
use rocket::{Build, Rocket, State};
use serde::{Deserialize, Serialize};
use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::FromRow;
use std::env;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow)]
struct Item {
    id: String,
    name: String,
    description: Option<String>,
    created_at: String,
}

#[derive(Debug, Deserialize)]
struct CreateItem {
    name: String,
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UpdateItem {
    name: Option<String>,
    description: Option<String>,
}

#[derive(Debug, Serialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

impl<T> ApiResponse<T> {
    fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    fn error(message: &str) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message.to_string()),
        }
    }
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: String,
    version: String,
}

// Health check endpoint
#[get("/health")]
fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

// Get all items
#[get("/api/items")]
async fn get_items(pool: &State<PgPool>) -> Json<ApiResponse<Vec<Item>>> {
    match sqlx::query_as::<_, Item>(
        "SELECT id, name, description, created_at FROM items ORDER BY created_at DESC",
    )
    .fetch_all(pool.inner())
    .await
    {
        Ok(items) => Json(ApiResponse::success(items)),
        Err(e) => Json(ApiResponse::error(&e.to_string())),
    }
}

// Get single item by ID
#[get("/api/items/<id>")]
async fn get_item(id: &str, pool: &State<PgPool>) -> Json<ApiResponse<Item>> {
    match sqlx::query_as::<_, Item>(
        "SELECT id, name, description, created_at FROM items WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool.inner())
    .await
    {
        Ok(Some(item)) => Json(ApiResponse::success(item)),
        Ok(None) => Json(ApiResponse::error("Item not found")),
        Err(e) => Json(ApiResponse::error(&e.to_string())),
    }
}

// Create new item
#[post("/api/items", data = "<input>")]
async fn create_item(input: Json<CreateItem>, pool: &State<PgPool>) -> Json<ApiResponse<Item>> {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    match sqlx::query(
        "INSERT INTO items (id, name, description, created_at) VALUES ($1, $2, $3, $4)",
    )
    .bind(&id)
    .bind(&input.name)
    .bind(&input.description)
    .bind(&now)
    .execute(pool.inner())
    .await
    {
        Ok(_) => {
            let item = Item {
                id,
                name: input.name.clone(),
                description: input.description.clone(),
                created_at: now,
            };
            Json(ApiResponse::success(item))
        }
        Err(e) => Json(ApiResponse::error(&e.to_string())),
    }
}

// Update item
#[put("/api/items/<id>", data = "<input>")]
async fn update_item(
    id: &str,
    input: Json<UpdateItem>,
    pool: &State<PgPool>,
) -> Json<ApiResponse<Item>> {
    // First check if item exists
    let existing = sqlx::query_as::<_, Item>(
        "SELECT id, name, description, created_at FROM items WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool.inner())
    .await;

    match existing {
        Ok(Some(item)) => {
            let name = input.name.clone().unwrap_or(item.name);
            let description = input.description.clone().or(item.description);

            match sqlx::query("UPDATE items SET name = $1, description = $2 WHERE id = $3")
                .bind(&name)
                .bind(&description)
                .bind(id)
                .execute(pool.inner())
                .await
            {
                Ok(_) => {
                    let updated = Item {
                        id: id.to_string(),
                        name,
                        description,
                        created_at: item.created_at,
                    };
                    Json(ApiResponse::success(updated))
                }
                Err(e) => Json(ApiResponse::error(&e.to_string())),
            }
        }
        Ok(None) => Json(ApiResponse::error("Item not found")),
        Err(e) => Json(ApiResponse::error(&e.to_string())),
    }
}

// Delete item
#[delete("/api/items/<id>")]
async fn delete_item(id: &str, pool: &State<PgPool>) -> Json<ApiResponse<String>> {
    match sqlx::query("DELETE FROM items WHERE id = $1")
        .bind(id)
        .execute(pool.inner())
        .await
    {
        Ok(result) => {
            if result.rows_affected() > 0 {
                Json(ApiResponse::success("Item deleted".to_string()))
            } else {
                Json(ApiResponse::error("Item not found"))
            }
        }
        Err(e) => Json(ApiResponse::error(&e.to_string())),
    }
}

async fn init_db(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS items (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT,
            created_at TEXT NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await?;
    Ok(())
}

#[launch]
async fn rocket() -> Rocket<Build> {
    dotenvy::dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/app".to_string());
    let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .unwrap_or(3000);

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create pool");

    init_db(&pool).await.expect("Failed to initialize database");

    let config = rocket::Config {
        address: host.parse().unwrap(),
        port,
        ..rocket::Config::default()
    };

    rocket::custom(config).manage(pool).mount(
        "/",
        routes![
            health,
            get_items,
            get_item,
            create_item,
            update_item,
            delete_item
        ],
    )
}
