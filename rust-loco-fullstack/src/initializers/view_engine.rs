use async_trait::async_trait;
use axum::Extension;
use loco_rs::{
    app::{AppContext, Initializer},
    controller::views::{engines, ViewEngine},
    Result,
};

pub struct ViewEngineInitializer;

#[async_trait]
impl Initializer for ViewEngineInitializer {
    fn name(&self) -> String {
        "view-engine".to_string()
    }

    async fn after_routes(
        &self,
        router: axum::routing::Router,
        _ctx: &AppContext,
    ) -> Result<axum::routing::Router> {
        let tera_engine = engines::TeraView::build()?;
        Ok(router.layer(Extension(ViewEngine::from(tera_engine))))
    }
}
