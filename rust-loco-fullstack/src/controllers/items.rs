use loco_rs::prelude::*;
use sea_orm::{ActiveModelTrait, EntityTrait, QueryOrder, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::_entities::items::{ActiveModel, Column, Entity as Items, Model};

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

#[derive(Debug, Deserialize)]
pub struct CreateItemParams {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateItemParams {
    pub name: Option<String>,
    pub description: Option<String>,
}

async fn list(State(ctx): State<AppContext>) -> Result<Response> {
    let items = Items::find()
        .order_by_desc(Column::CreatedAt)
        .all(&ctx.db)
        .await?;
    format::json(ApiResponse::success(items))
}

async fn get_one(Path(id): Path<String>, State(ctx): State<AppContext>) -> Result<Response> {
    let item = Items::find_by_id(&id).one(&ctx.db).await?;
    match item {
        Some(item) => format::json(ApiResponse::success(item)),
        None => format::json(ApiResponse::<Model>::error("Item not found")),
    }
}

async fn create(
    State(ctx): State<AppContext>,
    Json(params): Json<CreateItemParams>,
) -> Result<Response> {
    let now = chrono::Utc::now().naive_utc();
    let item = ActiveModel {
        id: Set(Uuid::new_v4().to_string()),
        name: Set(params.name),
        description: Set(params.description),
        created_at: Set(now),
        updated_at: Set(now),
    }
    .insert(&ctx.db)
    .await?;

    format::json(ApiResponse::success(item))
}

async fn update(
    Path(id): Path<String>,
    State(ctx): State<AppContext>,
    Json(params): Json<UpdateItemParams>,
) -> Result<Response> {
    let item = Items::find_by_id(&id).one(&ctx.db).await?;
    match item {
        Some(item) => {
            let mut active: ActiveModel = item.into();
            if let Some(name) = params.name {
                active.name = Set(name);
            }
            if let Some(description) = params.description {
                active.description = Set(Some(description));
            }
            active.updated_at = Set(chrono::Utc::now().naive_utc());

            let updated = active.update(&ctx.db).await?;
            format::json(ApiResponse::success(updated))
        }
        None => format::json(ApiResponse::<Model>::error("Item not found")),
    }
}

async fn remove(Path(id): Path<String>, State(ctx): State<AppContext>) -> Result<Response> {
    let result = Items::delete_by_id(&id).exec(&ctx.db).await?;
    if result.rows_affected > 0 {
        format::json(ApiResponse::success("Item deleted"))
    } else {
        format::json(ApiResponse::<String>::error("Item not found"))
    }
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("api/items")
        .add("/", get(list))
        .add("/", post(create))
        .add("/{id}", get(get_one))
        .add("/{id}", put(update))
        .add("/{id}", delete(remove))
}
