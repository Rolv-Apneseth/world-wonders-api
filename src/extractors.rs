use aide::operation::OperationIo;
use axum::{
    extract::{FromRequest, FromRequestParts},
    response::IntoResponse,
};
use axum_valid::HasValidate;
use serde::Serialize;

use crate::error::Error;

// MAIN JSON EXTRACTOR ----------------------------------------------------------------------------
#[derive(FromRequest, OperationIo)]
#[from_request(via(axum_jsonschema::Json), rejection(Error))]
#[aide(
    input_with = "axum_jsonschema::Json<T>",
    output_with = "axum_jsonschema::Json<T>",
    json_schema
)]
pub struct Json<T>(pub T);

impl<T> IntoResponse for Json<T>
where
    T: Serialize,
{
    fn into_response(self) -> axum::response::Response {
        axum_jsonschema::Json(self.0).into_response()
    }
}

// QUERY EXTRACTOR --------------------------------------------------------------------------------
#[derive(FromRequestParts, OperationIo)]
#[from_request(via(axum::extract::Query), rejection(Error))]
#[aide(
    input_with = "axum::extract::Query<T>",
    output_with = "axum_jsonschema::Json<T>",
    json_schema
)]
pub struct Query<T>(pub T);

impl<T> HasValidate for Query<T> {
    type Validate = T;
    fn get_validate(&self) -> &T {
        &self.0
    }
}

// PATH EXTRACTOR ---------------------------------------------------------------------------------
#[derive(FromRequestParts, OperationIo)]
#[from_request(via(axum::extract::Path), rejection(Error))]
#[aide(
    input_with = "axum::extract::Path<T>",
    output_with = "axum_jsonschema::Json<T>",
    json_schema
)]
pub struct Path<T>(pub T);
