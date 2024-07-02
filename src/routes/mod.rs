use axum::{http::StatusCode, response::IntoResponse};

pub mod categories;
pub mod docs;
mod utils;
pub mod wonders;

pub async fn handler_404() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        "Whoops! Route not found. Nothing to see here",
    )
}
