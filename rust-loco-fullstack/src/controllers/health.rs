use loco_rs::prelude::*;
use serde::Serialize;

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    version: String,
}

async fn health() -> Result<Response> {
    format::json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

pub fn routes() -> Routes {
    Routes::new().add("/health", get(health))
}
