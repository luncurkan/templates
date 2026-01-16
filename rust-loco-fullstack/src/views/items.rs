use loco_rs::prelude::*;
use serde_json::json;

use crate::models::_entities::items::Model;

pub fn index(v: impl ViewRenderer, items: Vec<Model>) -> Result<impl IntoResponse> {
    format::render().view(&v, "items/index.html", json!({ "items": items }))
}

pub fn row(v: impl ViewRenderer, item: &Model) -> Result<impl IntoResponse> {
    format::render().view(&v, "items/_row.html", json!({ "item": item }))
}

pub fn edit_row(v: impl ViewRenderer, item: &Model) -> Result<impl IntoResponse> {
    format::render().view(&v, "items/_edit_row.html", json!({ "item": item }))
}
