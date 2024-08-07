use axum::{http::StatusCode, response::IntoResponse};

pub mod docs;
pub mod wonders;

pub async fn handler_404() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        "Whoops! Route not found. Nothing to see here",
    )
}
