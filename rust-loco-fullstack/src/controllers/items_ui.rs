use axum::http::StatusCode;
use loco_rs::prelude::*;
use sea_orm::{ActiveModelTrait, EntityTrait, QueryOrder, Set};
use serde::Deserialize;
use uuid::Uuid;

use crate::models::_entities::items::{ActiveModel, Column, Entity as Items};
use crate::views;

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

/// GET / - Show items list page
pub async fn index(
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
) -> Result<impl IntoResponse> {
    let items = Items::find()
        .order_by_desc(Column::CreatedAt)
        .all(&ctx.db)
        .await?;
    views::items::index(v, items)
}

/// POST / - Create item and return new row HTML
pub async fn create(
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
    Form(params): Form<CreateItemParams>,
) -> Result<impl IntoResponse> {
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

    views::items::row(v, &item)
}

/// GET /:id/edit - Return edit form row
pub async fn edit(
    ViewEngine(v): ViewEngine<TeraView>,
    Path(id): Path<String>,
    State(ctx): State<AppContext>,
) -> Result<impl IntoResponse> {
    let item = Items::find_by_id(&id).one(&ctx.db).await?;
    match item {
        Some(item) => views::items::edit_row(v, &item),
        None => Err(Error::NotFound),
    }
}

/// GET /:id/row - Return normal row (cancel edit)
pub async fn row(
    ViewEngine(v): ViewEngine<TeraView>,
    Path(id): Path<String>,
    State(ctx): State<AppContext>,
) -> Result<impl IntoResponse> {
    let item = Items::find_by_id(&id).one(&ctx.db).await?;
    match item {
        Some(item) => views::items::row(v, &item),
        None => Err(Error::NotFound),
    }
}

/// PUT /:id - Update item and return updated row
pub async fn update(
    ViewEngine(v): ViewEngine<TeraView>,
    Path(id): Path<String>,
    State(ctx): State<AppContext>,
    Form(params): Form<UpdateItemParams>,
) -> Result<impl IntoResponse> {
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
            views::items::row(v, &updated)
        }
        None => Err(Error::NotFound),
    }
}

/// DELETE /:id - Delete item and return empty (removes row)
pub async fn remove(
    Path(id): Path<String>,
    State(ctx): State<AppContext>,
) -> Result<impl IntoResponse> {
    let result = Items::delete_by_id(&id).exec(&ctx.db).await?;
    if result.rows_affected > 0 {
        Ok(StatusCode::OK)
    } else {
        Err(Error::NotFound)
    }
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/")
        .add("/", get(index))
        .add("/", post(create))
        .add("/{id}/edit", get(edit))
        .add("/{id}/row", get(row))
        .add("/{id}", put(update))
        .add("/{id}", delete(remove))
}
