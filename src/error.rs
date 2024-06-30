use std::fmt::Display;

use aide::OperationIo;
use axum::{
    extract::rejection::JsonRejection,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use axum_jsonschema::JsonSchemaRejection;
use schemars::JsonSchema;
use serde::Serialize;

use crate::extractor::AppJson;

pub type Result<T> = core::result::Result<T, Error>;

/// Represents all errors the API can return
#[derive(thiserror::Error, Debug, OperationIo, JsonSchema)]
#[aide(
    input_with = "axum_jsonschema::Json<Error>",
    output_with = "axum_jsonschema::Json<Error>",
    json_schema
)]
pub enum Error {
    #[error("No wonder matching the given filters was found")]
    NoWondersLeft,
    #[error("No wonder found matching the name '{0}'")]
    NoMatchingName(String),
    #[error("The provided lower limit of {0} is greater than the provided upper limit of {1}")]
    ConflictingLimitParams(i16, i16),
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    // Don't expose additional details about server errors to the client
    #[error("Something went wrong")]
    Internal(String),
}

// For serialising error response into specific format
#[derive(Serialize, Debug, OperationIo, JsonSchema)]
#[aide(
    input_with = "axum_jsonschema::Json<ErrorResponse>",
    output_with = "axum_jsonschema::Json<ErrorResponse>",
    json_schema
)]
pub struct ErrorResponse {
    pub message: String,
}
impl ErrorResponse {
    pub fn new(message: impl Display) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let message = self.to_string();
        let status = match &self {
            Self::NoWondersLeft => StatusCode::BAD_REQUEST,
            Self::NoMatchingName(_) => StatusCode::BAD_REQUEST,
            Self::ConflictingLimitParams(_, _) => StatusCode::BAD_REQUEST,
            Self::InvalidRequest(_) => StatusCode::BAD_REQUEST,

            Self::Internal(s) => {
                tracing::error!("Internal server error: {s}");
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };

        (status, AppJson(ErrorResponse::new(message))).into_response()
    }
}

impl From<JsonSchemaRejection> for Error {
    fn from(rejection: JsonSchemaRejection) -> Self {
        match rejection {
            JsonSchemaRejection::Json(s) => Self::InvalidRequest(s.to_string()),
            JsonSchemaRejection::Serde(s) => Self::InvalidRequest(s.to_string()),
            JsonSchemaRejection::Schema(_) => {
                Self::InvalidRequest("Failed to validate schema".to_string())
            }
        }
    }
}

impl From<JsonRejection> for Error {
    fn from(rejection: JsonRejection) -> Self {
        Self::InvalidRequest(rejection.to_string())
    }
}
