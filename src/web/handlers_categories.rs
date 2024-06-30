use aide::{
    axum::{routing::get_with, ApiRouter},
    transform::TransformOperation,
};
use axum::extract::Query;
use schemars::JsonSchema;
use serde::Deserialize;
use strum::IntoEnumIterator;

use crate::{data::Category, extractor::AppJson};

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
async fn get_all_categories(Query(params): Query<CategoriesParams>) -> AppJson<Vec<Category>> {
    let mut categories: Vec<Category> = Category::iter().collect();

    // Filter out game categories :(
    if matches!(params.exclude_games, Some(true)) {
        categories.retain(|c| !matches!(c, Category::Civ5 | Category::Civ6));
    }

    AppJson(categories)
}
fn get_all_categories_docs(op: TransformOperation) -> TransformOperation {
    op.summary("Wonder categories")
        .description("Get all available wonder categories")
}
