use axum::{extract::Query, routing::get, Json, Router};
use serde::Deserialize;
use strum::IntoEnumIterator;

use crate::data::Category;

#[derive(Debug, Deserialize)]
pub struct CategoriesParams {
    exclude_games: Option<bool>,
}

pub fn routes() -> Router {
    Router::new().route("/", get(all_categories))
}

/// Fetch all available categories for wonders
pub async fn all_categories(Query(params): Query<CategoriesParams>) -> axum::Json<Vec<Category>> {
    let mut categories: Vec<Category> = Category::iter().collect();

    // Filter out game categories :(
    if matches!(params.exclude_games, Some(true)) {
        categories.retain(|c| !matches!(c, Category::Civ5 | Category::Civ6));
    }

    Json(categories)
}
