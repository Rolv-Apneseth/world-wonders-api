use std::sync::Arc;

use aide::{
    axum::{
        routing::{get, get_with},
        ApiRouter, IntoApiResponse,
    },
    openapi::OpenApi,
    scalar::Scalar,
};
use axum::Extension;

use crate::extractor::AppJson;

pub fn routes() -> ApiRouter {
    let router: ApiRouter = ApiRouter::new()
        // Serve the documentation JSON file
        .route("/api.json", get(serve_docs))
        // Use documentation JSON file with `scalar` (make sure the path matches the nested path)
        .api_route_with(
            "/",
            get_with(
                Scalar::new("/docs/api.json")
                    .with_title("World Wonder API Docs")
                    .axum_handler(),
                |op| op.description("This documentation page."),
            ),
            |p| p,
        );

    router
}

async fn serve_docs(Extension(api): Extension<Arc<OpenApi>>) -> impl IntoApiResponse {
    AppJson(api)
}
