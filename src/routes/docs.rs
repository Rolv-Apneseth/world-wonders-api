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

use crate::{extractors::Json, DOCS_ROUTE};

const API_FILE_ROUTE: &str = "/api.json";

pub fn routes() -> ApiRouter {
    let router: ApiRouter = ApiRouter::new()
        // Serve the documentation JSON file
        .route(API_FILE_ROUTE, get(serve_docs))
        // Use documentation JSON file with `scalar` (make sure the path matches the nested path)
        .api_route_with(
            "/",
            get_with(
                Scalar::new(format!("{DOCS_ROUTE}{API_FILE_ROUTE}"))
                    .with_title("World Wonder API Docs")
                    .axum_handler(),
                |op| op.description("This documentation page."),
            ),
            |p| p,
        );

    router
}

async fn serve_docs(Extension(api): Extension<Arc<OpenApi>>) -> impl IntoApiResponse {
    Json(api)
}
