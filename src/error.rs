use std::fmt::Display;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use thiserror::Error;

use crate::AppJson;

pub type Result<T> = core::result::Result<T, Error>;

/// Errors specific to the server
#[derive(Error, Debug)]
pub enum ServerError {
    #[error("Wonders list is empty")]
    WondersEmpty,
}
impl From<ServerError> for Error {
    fn from(value: ServerError) -> Self {
        Self::Server(value)
    }
}

/// Errors specific to the client / requests made by the client
#[derive(Error, Debug)]
pub enum ClientError {
    #[error("No wonder matching the given filters was found")]
    NoWondersLeft,
    #[error("No wonder found matching the name '{0}'")]
    NoMatchingName(String),
    #[error("The provided lower limit of {0} is greater than the provided upper limit of {1}")]
    ConflictingLimitParams(i16, i16),
}
impl From<ClientError> for Error {
    fn from(value: ClientError) -> Self {
        Self::Client(value)
    }
}

/// Represents all errors the API can return
#[derive(Error, Debug)]
pub enum Error {
    Server(ServerError),
    Client(ClientError),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> core::result::Result<(), std::fmt::Error> {
        write!(
            f,
            "{}",
            match self {
                Self::Client(c) => c.to_string(),
                // Don't expose additional details about server errors to the client
                Self::Server(_) => "Something went wrong".to_owned(),
            }
        )
    }
}

// For serialising error response into specific format
#[derive(Serialize)]
struct ErrorResponse {
    message: String,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let message = self.to_string();
        let status = match &self {
            Self::Client(c) => match c {
                ClientError::NoWondersLeft => StatusCode::BAD_REQUEST,
                ClientError::NoMatchingName(_) => StatusCode::BAD_REQUEST,
                ClientError::ConflictingLimitParams(_, _) => StatusCode::BAD_REQUEST,
            },

            Self::Server(s) => {
                tracing::error!("{s}");

                StatusCode::INTERNAL_SERVER_ERROR
            }
        };

        (status, AppJson(ErrorResponse { message })).into_response()
    }
}
