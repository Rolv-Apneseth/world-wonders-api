use crate::{
    data::{Category, TimePeriod, Wonder, WONDERS},
    error::{Error, ErrorResponse, Result},
    extractor::AppJson,
};
use aide::{
    axum::{routing::get_with, ApiRouter, IntoApiResponse},
    transform::TransformOperation,
};
use axum::{
    extract::{Path, Query},
    response::IntoResponse,
    Json,
};
use rand::prelude::*;
use schemars::JsonSchema;
use serde::Deserialize;

use super::utils::empty_string_as_none;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct WonderParamsFiltering {
    #[serde(default, deserialize_with = "empty_string_as_none")]
    name: Option<String>,
    location: Option<String>,
    time_period: Option<TimePeriod>,
    lower_limit: Option<i16>,
    upper_limit: Option<i16>,
    category: Option<Category>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct WonderParamsSorting {
    sort_by: Option<SortBy>,
    sort_reverse: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub enum SortBy {
    BuildYear,
    Alphabetical,
}

// ROUTES -----------------------------------------------------------------------------------------
pub fn routes() -> ApiRouter {
    ApiRouter::new()
        .api_route("/", get_with(get_all_wonders, get_all_wonders_docs))
        .api_route(
            "/count",
            get_with(get_count_wonders, get_count_wonders_docs),
        )
        .api_route(
            "/random",
            get_with(get_random_wonder, get_random_wonder_docs),
        )
        .api_route(
            "/oldest",
            get_with(get_oldest_wonder, get_oldest_wonder_docs),
        )
        .api_route(
            "/youngest",
            get_with(get_youngest_wonder, get_youngest_wonder_docs),
        )
        .api_route(
            "/name/:name",
            get_with(get_wonder_by_name, get_wonder_by_name_docs),
        )
}

// UTILS ------------------------------------------------------------------------------------------
/// Filters wonders based on given [`WondersParams`]
fn filter_wonders(wonders: &mut Vec<&'static Wonder>, params: WonderParamsFiltering) -> Result<()> {
    if let Some(name) = params.name.as_deref() {
        wonders.retain(|w| w.name.to_lowercase().contains(&name.to_lowercase()));
    };
    if let Some(location) = params.location.as_deref() {
        wonders.retain(|w| w.location.to_lowercase().contains(&location.to_lowercase()));
    };
    if let Some(time_period) = params.time_period.as_ref() {
        wonders.retain(|w| w.time_period == *time_period);
    };
    if let Some(category) = params.category.as_ref() {
        wonders.retain(|w| w.categories.contains(category));
    };

    // Handle upper and lower limits for build year
    match (params.lower_limit.as_ref(), params.upper_limit.as_ref()) {
        (Some(l), Some(u)) => {
            if l > u {
                return Err(Error::ConflictingLimitParams(*l, *u));
            };
            wonders.retain(|w| w.build_year >= *l && w.build_year <= *u);
        }
        (Some(l), None) => wonders.retain(|w| w.build_year >= *l),
        (None, Some(u)) => wonders.retain(|w| w.build_year <= *u),
        (None, None) => {}
    };

    if wonders.is_empty() {
        return Err(Error::NoWondersLeft);
    }

    Ok(())
}

/// Filters wonders based on given [`WondersParams`], but ignores the no wonders error.
///
/// Intended for endpoints which should instead return an empty vec
fn filter_wonders_ignore_empty(
    wonders: &mut Vec<&'static Wonder>,
    params: WonderParamsFiltering,
) -> Result<()> {
    let res = filter_wonders(wonders, params);
    // Ignore no wonders error, just continue with the empty vec
    if matches!(res, Err(Error::NoWondersLeft)) {
        Ok(())
    } else {
        res
    }
}

/// Sorts wonders based on given [`WondersParams`]
fn sort_wonders(wonders: &mut [&'static Wonder], params: WonderParamsSorting) {
    if let Some(sort_by) = params.sort_by.as_ref() {
        match sort_by {
            SortBy::Alphabetical => wonders.sort_by(|a, b| a.name.cmp(&b.name)),
            SortBy::BuildYear => wonders.sort_by(|a, b| a.build_year.cmp(&b.build_year)),
        }

        // `sort_reverse` only matters when `sort_by` is provided, and `sort_reverse = true`
        if matches!(params.sort_reverse, Some(true)) {
            wonders.reverse();
        };
    };
}

// HANDLERS ----------------------------------------------------------------------------------------
// GET ALL WONDERS
async fn get_all_wonders(
    Query(filtering_params): Query<WonderParamsFiltering>,
    Query(sorting_params): Query<WonderParamsSorting>,
) -> impl IntoApiResponse {
    let mut wonders: Vec<&Wonder> = WONDERS.iter().collect();

    if let Err(e) = filter_wonders_ignore_empty(&mut wonders, filtering_params) {
        return e.into_response();
    };
    sort_wonders(&mut wonders, sorting_params);

    AppJson(wonders).into_response()
}
fn get_all_wonders_docs(op: TransformOperation) -> TransformOperation {
    op.summary("All wonders")
        .description(
            "Get all wonders after applying filters and sort methods defined by query parameters",
        )
        .response_with::<200, Json<Vec<&'static Wonder>>, _>(|res| res.example(vec![&WONDERS[0]]))
        .response_with::<400, ErrorResponse, _>(|res| {
            res.description("Bad request")
                .example(ErrorResponse::new(Error::ConflictingLimitParams(1000, 400)))
        })
}

// GET NUM WONDERS
async fn get_count_wonders(
    Query(filtering_params): Query<WonderParamsFiltering>,
) -> impl IntoApiResponse {
    let mut wonders: Vec<&Wonder> = WONDERS.iter().collect();
    if let Err(e) = filter_wonders_ignore_empty(&mut wonders, filtering_params) {
        return e.into_response();
    };
    AppJson(wonders.len()).into_response()
}
fn get_count_wonders_docs(op: TransformOperation) -> TransformOperation {
    op.summary("Number of wonders")
        .description(
            "Get the total number of wonders, after applying filters defined by query parameters",
        )
        .response_with::<200, Json<usize>, _>(|res| res.example(WONDERS.len()))
        .response_with::<400, ErrorResponse, _>(|res| {
            res.description("Bad request")
                .example(ErrorResponse::new(Error::ConflictingLimitParams(1000, 400)))
        })
}

// GET WONDER BY NAME
async fn get_wonder_by_name(Path(name): Path<String>) -> impl IntoApiResponse {
    let Some(wonder) = WONDERS
        .iter()
        .find(|w| w.name.to_ascii_lowercase().replace(' ', "-") == name)
    else {
        return Error::NoMatchingName(name).into_response();
    };

    AppJson(wonder).into_response()
}
fn get_wonder_by_name_docs(op: TransformOperation) -> TransformOperation {
    op.summary("Specific wonder - by name")
        .description("Get a specific wonder matching the name defined by the path")
        .response_with::<200, Json<&'static Wonder>, _>(|res| res.example(&WONDERS[1]))
        .response_with::<400, ErrorResponse, _>(|res| {
            res.description("Bad request")
                .example(ErrorResponse::new(Error::NoMatchingName("...".to_owned())))
        })
}

// GET RANDOM WONDER
async fn get_random_wonder(
    Query(filtering_params): Query<WonderParamsFiltering>,
) -> impl IntoApiResponse {
    assert!(WONDERS.len() > 0);

    let mut rng = rand::thread_rng();

    let mut wonders: Vec<&Wonder> = WONDERS.iter().collect();
    if let Err(e) = filter_wonders(&mut wonders, filtering_params) {
        return e.into_response();
    };

    AppJson(wonders.choose(&mut rng).unwrap()).into_response()
}
fn get_random_wonder_docs(op: TransformOperation) -> TransformOperation {
    op.summary("Specific wonder - random")
        .description(
            "Get a random wonder, after filtering wonders based on provided query parameters",
        )
        .summary("Random wonder")
        .response_with::<200, Json<&'static Wonder>, _>(|res| res.example(&WONDERS[20]))
        .response_with::<400, ErrorResponse, _>(|res| {
            res.description("Bad request")
                .example(ErrorResponse::new(Error::NoWondersLeft))
        })
}

// GET OLDEST WONDER
async fn get_oldest_wonder(
    Query(filtering_params): Query<WonderParamsFiltering>,
) -> impl IntoApiResponse {
    assert!(WONDERS.len() > 0);

    let mut wonders: Vec<&Wonder> = WONDERS.iter().collect();
    if let Err(e) = filter_wonders(&mut wonders, filtering_params) {
        return e.into_response();
    };

    AppJson(
        wonders
            .iter()
            .reduce(|a, b| if a.build_year < b.build_year { a } else { b })
            .unwrap(),
    )
    .into_response()
}
fn get_oldest_wonder_docs(op: TransformOperation) -> TransformOperation {
    op.summary("Specific wonder - oldest")
        .description("Get the oldest (least recently built) wonder, after filtering wonders based on provided query parameters")
        .response_with::<200, Json<&'static Wonder>, _>(|res| res.example(&WONDERS[2]))
        .response_with::<400, ErrorResponse, _>(|res| {
            res.description("Bad request")
                .example(ErrorResponse::new(Error::NoWondersLeft))
        })
}

// GET YOUNGEST WONDER
async fn get_youngest_wonder(
    Query(filtering_params): Query<WonderParamsFiltering>,
) -> impl IntoApiResponse {
    assert!(WONDERS.len() > 0);

    let mut wonders: Vec<&Wonder> = WONDERS.iter().collect();
    if let Err(e) = filter_wonders(&mut wonders, filtering_params) {
        return e.into_response();
    };

    AppJson(
        wonders
            .iter()
            .reduce(|a, b| if a.build_year > b.build_year { a } else { b })
            .unwrap(),
    )
    .into_response()
}
fn get_youngest_wonder_docs(op: TransformOperation) -> TransformOperation {
    op.summary("Specific wonder - youngest")
        .description("Get the youngest (most recently built) wonder, after filtering wonders based on provided query parameters")
        .response_with::<200, Json<&'static Wonder>, _>(|res| res.example(&WONDERS[10]))
        .response_with::<400, ErrorResponse, _>(|res| {
            res.description("Bad request")
                .example(ErrorResponse::new(Error::NoWondersLeft))
        })
}
