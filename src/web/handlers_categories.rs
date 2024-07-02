use aide::{
    axum::{routing::get_with, ApiRouter, IntoApiResponse},
    transform::TransformOperation,
};
use axum::response::IntoResponse;
use schemars::JsonSchema;
use serde::Deserialize;
use strum::IntoEnumIterator;

use crate::{
    data::Category,
    error::{Error, ErrorResponse},
    extractor::{AppJson, Query},
};

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CategoriesParams {
    exclude_games: Option<bool>,
}

// ROUTES -----------------------------------------------------------------------------------------
pub fn routes() -> ApiRouter {
    ApiRouter::new().api_route("/", get_with(get_all_categories, get_all_categories_docs))
}

// HANDLERS ----------------------------------------------------------------------------------------
// GET ALL CATEGORIES
async fn get_all_categories(Query(params): Query<CategoriesParams>) -> impl IntoApiResponse {
    let categories: Vec<Category> = Category::iter()
        // Filter out game categories :(
        .filter(|c| {
            !matches!(
                (params.exclude_games, c),
                (Some(true), Category::Civ5 | Category::Civ6)
            )
        })
        .collect();

    AppJson(categories).into_response()
}
fn get_all_categories_docs(op: TransformOperation) -> TransformOperation {
    op.summary("Wonder categories")
        .description("Get all available wonder categories")
        .response_with::<200, AppJson<Vec<Category>>, _>(|res| {
            res.example(vec![Category::SevenWonders, Category::Civ5])
        })
        .response_with::<400, ErrorResponse, _>(|res| {
            res.description("Bad request")
                .example(ErrorResponse::new(Error::InvalidRequest(
                    "Failed to deserialize query string".to_string(),
                )))
        })
}
