use axum::{
    Json, Router,
    extract::{Path, State},
    http::{header, HeaderMap, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
    routing::get,
};
use tower_http::trace::TraceLayer;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool, postgres::PgPoolOptions};
use std::sync::Arc;
use thiserror::Error;
use uuid::Uuid;

// ============================================================================
// Application State
// ============================================================================

#[derive(Clone)]
struct AppState {
    db: PgPool,
}

// ============================================================================
// Error Handling
// ============================================================================

#[derive(Error, Debug)]
enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("URL not found")]
    NotFound,

    #[error("Invalid URL provided")]
    InvalidUrl,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::Database(e) => {
                tracing::error!("Database error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
            AppError::NotFound => (StatusCode::NOT_FOUND, "URL not found"),
            AppError::InvalidUrl => (StatusCode::BAD_REQUEST, "Invalid URL provided"),
        };

        let body = serde_json::json!({
            "error": message
        });

        (status, Json(body)).into_response()
    }
}

// ============================================================================
// Models
// ============================================================================

#[derive(Debug, Serialize, FromRow)]
struct Url {
    id: Uuid,
    short_code: String,
    original_url: String,
    clicks: i64,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
struct CreateUrlRequest {
    url: String,
    #[serde(default)]
    custom_code: Option<String>,
}

#[derive(Debug, Serialize)]
struct CreateUrlResponse {
    id: Uuid,
    short_code: String,
    short_url: String,
    original_url: String,
}

#[derive(Debug, Serialize)]
struct UrlStatsResponse {
    id: Uuid,
    short_code: String,
    short_url: String,
    original_url: String,
    clicks: i64,
    created_at: DateTime<Utc>,
}

// ============================================================================
// Handlers
// ============================================================================

async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy"
    }))
}

async fn index() -> Html<&'static str> {
    Html(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>URL Shortener</title>
    <style>
        * { box-sizing: border-box; margin: 0; padding: 0; }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
            display: flex;
            align-items: center;
            justify-content: center;
            padding: 20px;
        }
        .container {
            background: white;
            padding: 40px;
            border-radius: 16px;
            box-shadow: 0 20px 60px rgba(0,0,0,0.3);
            width: 100%;
            max-width: 500px;
        }
        h1 {
            color: #333;
            margin-bottom: 8px;
            font-size: 28px;
        }
        .subtitle {
            color: #666;
            margin-bottom: 30px;
            font-size: 14px;
        }
        .form-group {
            margin-bottom: 20px;
        }
        label {
            display: block;
            margin-bottom: 8px;
            color: #333;
            font-weight: 500;
        }
        input[type="text"] {
            width: 100%;
            padding: 14px 16px;
            border: 2px solid #e1e1e1;
            border-radius: 8px;
            font-size: 16px;
            transition: border-color 0.2s;
        }
        input[type="text"]:focus {
            outline: none;
            border-color: #667eea;
        }
        button {
            width: 100%;
            padding: 14px;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            border: none;
            border-radius: 8px;
            font-size: 16px;
            font-weight: 600;
            cursor: pointer;
            transition: transform 0.2s, box-shadow 0.2s;
        }
        button:hover {
            transform: translateY(-2px);
            box-shadow: 0 4px 12px rgba(102, 126, 234, 0.4);
        }
        button:disabled {
            opacity: 0.7;
            cursor: not-allowed;
            transform: none;
        }
        .result {
            margin-top: 24px;
            padding: 20px;
            background: #f8f9fa;
            border-radius: 8px;
            display: none;
        }
        .result.show { display: block; }
        .result.error {
            background: #fee;
            border: 1px solid #fcc;
        }
        .result-label {
            font-size: 12px;
            color: #666;
            margin-bottom: 8px;
        }
        .short-url {
            display: flex;
            align-items: center;
            gap: 10px;
        }
        .short-url a {
            color: #667eea;
            font-weight: 600;
            word-break: break-all;
            flex: 1;
        }
        .copy-btn {
            width: auto;
            padding: 8px 16px;
            font-size: 14px;
        }
        .error-msg { color: #c00; }
    </style>
</head>
<body>
    <div class="container">
        <h1>URL Shortener</h1>
        <p class="subtitle">Create short, memorable links instantly</p>

        <form id="shorten-form">
            <div class="form-group">
                <label for="url">Enter your long URL</label>
                <input type="text" id="url" name="url" placeholder="https://example.com/very/long/url" required>
            </div>
            <div class="form-group">
                <label for="custom_code">Custom code (optional)</label>
                <input type="text" id="custom_code" name="custom_code" placeholder="my-link">
            </div>
            <button type="submit" id="submit-btn">Shorten URL</button>
        </form>

        <div class="result" id="result">
            <div class="result-label">Your shortened URL:</div>
            <div class="short-url">
                <a href="#" id="short-url-link" target="_blank"></a>
                <button class="copy-btn" id="copy-btn">Copy</button>
            </div>
        </div>

        <div class="result error" id="error-result">
            <span class="error-msg" id="error-msg"></span>
        </div>
    </div>

    <script>
        const form = document.getElementById('shorten-form');
        const result = document.getElementById('result');
        const errorResult = document.getElementById('error-result');
        const shortUrlLink = document.getElementById('short-url-link');
        const submitBtn = document.getElementById('submit-btn');
        const copyBtn = document.getElementById('copy-btn');
        const errorMsg = document.getElementById('error-msg');

        form.addEventListener('submit', async (e) => {
            e.preventDefault();

            const url = document.getElementById('url').value;
            const customCode = document.getElementById('custom_code').value;

            result.classList.remove('show');
            errorResult.classList.remove('show');
            submitBtn.disabled = true;
            submitBtn.textContent = 'Shortening...';

            try {
                const body = { url };
                if (customCode) body.custom_code = customCode;

                const response = await fetch('/api/urls', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify(body)
                });

                const data = await response.json();

                if (response.ok) {
                    shortUrlLink.href = data.short_url;
                    shortUrlLink.textContent = data.short_url;
                    result.classList.add('show');
                } else {
                    errorMsg.textContent = data.error || 'Failed to shorten URL';
                    errorResult.classList.add('show');
                }
            } catch (err) {
                errorMsg.textContent = 'Network error. Please try again.';
                errorResult.classList.add('show');
            } finally {
                submitBtn.disabled = false;
                submitBtn.textContent = 'Shorten URL';
            }
        });

        copyBtn.addEventListener('click', async () => {
            try {
                await navigator.clipboard.writeText(shortUrlLink.href);
                copyBtn.textContent = 'Copied!';
                setTimeout(() => copyBtn.textContent = 'Copy', 2000);
            } catch (err) {
                copyBtn.textContent = 'Failed';
                setTimeout(() => copyBtn.textContent = 'Copy', 2000);
            }
        });
    </script>
</body>
</html>"##,
    )
}

async fn create_short_url(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<CreateUrlRequest>,
) -> Result<Json<CreateUrlResponse>, AppError> {
    // Validate URL
    if !is_valid_url(&payload.url) {
        tracing::warn!(url = %payload.url, "Invalid URL provided");
        return Err(AppError::InvalidUrl);
    }

    // Generate or use custom short code
    let short_code = match payload.custom_code {
        Some(code) if !code.is_empty() => code,
        _ => generate_short_code(),
    };

    // Insert into database
    let record: Url = sqlx::query_as(
        r#"
        INSERT INTO urls (short_code, original_url)
        VALUES ($1, $2)
        RETURNING id, short_code, original_url, clicks, created_at
        "#,
    )
    .bind(&short_code)
    .bind(&payload.url)
    .fetch_one(&state.db)
    .await?;

    let base_url = get_base_url(&headers);
    let short_url = format!("{}/{}", base_url, record.short_code);

    tracing::info!(
        short_code = %record.short_code,
        original_url = %record.original_url,
        "URL shortened"
    );

    Ok(Json(CreateUrlResponse {
        id: record.id,
        short_code: record.short_code,
        short_url,
        original_url: record.original_url,
    }))
}

async fn redirect_to_url(
    State(state): State<Arc<AppState>>,
    Path(code): Path<String>,
) -> Result<Redirect, AppError> {
    // Find and update click count
    let record: Option<Url> = sqlx::query_as(
        r#"
        UPDATE urls
        SET clicks = clicks + 1, updated_at = NOW()
        WHERE short_code = $1
        RETURNING id, short_code, original_url, clicks, created_at
        "#,
    )
    .bind(&code)
    .fetch_optional(&state.db)
    .await?;

    let record = match record {
        Some(r) => r,
        None => {
            tracing::warn!(short_code = %code, "Short URL not found");
            return Err(AppError::NotFound);
        }
    };

    tracing::info!(
        short_code = %record.short_code,
        target = %record.original_url,
        clicks = record.clicks,
        "Redirecting"
    );

    Ok(Redirect::temporary(&record.original_url))
}

async fn get_url_stats(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(code): Path<String>,
) -> Result<Json<UrlStatsResponse>, AppError> {
    let record: Option<Url> = sqlx::query_as(
        r#"
        SELECT id, short_code, original_url, clicks, created_at
        FROM urls
        WHERE short_code = $1
        "#,
    )
    .bind(&code)
    .fetch_optional(&state.db)
    .await?;

    let record = record.ok_or(AppError::NotFound)?;
    let base_url = get_base_url(&headers);
    let short_url = format!("{}/{}", base_url, record.short_code);

    Ok(Json(UrlStatsResponse {
        id: record.id,
        short_code: record.short_code,
        short_url,
        original_url: record.original_url,
        clicks: record.clicks,
        created_at: record.created_at,
    }))
}

async fn list_urls(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<UrlStatsResponse>>, AppError> {
    let records: Vec<Url> = sqlx::query_as(
        r#"
        SELECT id, short_code, original_url, clicks, created_at
        FROM urls
        ORDER BY created_at DESC
        LIMIT 100
        "#,
    )
    .fetch_all(&state.db)
    .await?;

    let base_url = get_base_url(&headers);
    let urls: Vec<UrlStatsResponse> = records
        .into_iter()
        .map(|r| UrlStatsResponse {
            id: r.id,
            short_code: r.short_code.clone(),
            short_url: format!("{}/{}", base_url, r.short_code),
            original_url: r.original_url,
            clicks: r.clicks,
            created_at: r.created_at,
        })
        .collect();

    Ok(Json(urls))
}

async fn delete_url(
    State(state): State<Arc<AppState>>,
    Path(code): Path<String>,
) -> Result<StatusCode, AppError> {
    let result = sqlx::query(
        r#"
        DELETE FROM urls
        WHERE short_code = $1
        "#,
    )
    .bind(&code)
    .execute(&state.db)
    .await?;

    if result.rows_affected() == 0 {
        tracing::warn!(short_code = %code, "URL not found for deletion");
        return Err(AppError::NotFound);
    }

    tracing::info!(short_code = %code, "URL deleted");

    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Utility Functions
// ============================================================================

fn generate_short_code() -> String {
    let uuid = Uuid::new_v4();
    let encoded = uuid.to_string().replace("-", "");
    encoded[..7].to_string()
}

fn is_valid_url(url: &str) -> bool {
    url.starts_with("http://") || url.starts_with("https://")
}

fn get_base_url(headers: &HeaderMap) -> String {
    let host = headers
        .get(header::HOST)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("localhost:3000");

    format!("http://{}", host)
}

// ============================================================================
// Main
// ============================================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    // Get configuration from environment
    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let max_connections: u32 = std::env::var("DATABASE_MAX_CONNECTIONS")
        .unwrap_or_else(|_| "5".to_string())
        .parse()
        .expect("DATABASE_MAX_CONNECTIONS must be a number");

    // Create database connection pool
    tracing::info!("Connecting to database...");
    let pool = PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(&database_url)
        .await?;

    // Run migrations
    tracing::info!("Running database migrations...");
    sqlx::migrate!("./migrations").run(&pool).await?;

    // Create application state
    let state = Arc::new(AppState { db: pool });

    // Build router
    let app = Router::new()
        .route("/", get(index))
        .route("/health", get(health_check))
        .route("/api/urls", get(list_urls).post(create_short_url))
        .route("/api/urls/{code}", get(get_url_stats).delete(delete_url))
        .route("/{code}", get(redirect_to_url))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // Start server
    let addr = format!("{}:{}", host, port);
    tracing::info!("Starting server at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
