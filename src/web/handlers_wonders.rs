use crate::{
    data::{Category, TimePeriod, Wonder, WONDERS},
    error::{ClientError, Error, Result, ServerError},
    AppJson,
};
use axum::{
    extract::{Path, Query},
    routing::get,
    Router,
};
use rand::prelude::*;
use serde::Deserialize;

use super::utils::empty_string_as_none;

#[derive(Debug, Deserialize)]
pub struct WondersParams {
    #[serde(default, deserialize_with = "empty_string_as_none")]
    // FILTERING
    name: Option<String>,
    location: Option<String>,
    time_period: Option<TimePeriod>,
    lower_limit: Option<i16>,
    upper_limit: Option<i16>,
    category: Option<Category>,
    // SORTING
    sort_by: Option<SortBy>,
    sort_reverse: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub enum SortBy {
    BuildYear,
    Alphabetical,
}

// HELPERS

// ROUTES -----------------------------------------------------------------------------------------
pub fn routes() -> Router {
    Router::new()
        .route("/", get(all_wonders))
        .route("/count", get(count_wonders))
        .route("/random", get(random_wonder))
        .route("/oldest", get(oldest_wonder))
        .route("/youngest", get(youngest_wonder))
        .route("/name/:name", get(specific_wonder_by_name))
}

// UTILS ------------------------------------------------------------------------------------------
/// Filters wonders based on given [`WondersParams`]
fn filter_wonders(wonders: &mut Vec<&'static Wonder>, params: &WondersParams) -> Result<()> {
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
                return Err(ClientError::ConflictingLimitParams(*l, *u).into());
            };
            wonders.retain(|w| w.build_year >= *l && w.build_year <= *u);
        }
        (Some(l), None) => wonders.retain(|w| w.build_year >= *l),
        (None, Some(u)) => wonders.retain(|w| w.build_year <= *u),
        (None, None) => {}
    };

    if wonders.is_empty() {
        return Err(ClientError::NoWondersLeft.into());
    }

    Ok(())
}

/// Filters wonders based on given [`WondersParams`], but ignores the no wonders error.
///
/// Intended for endpoints which should instead return an empty vec
fn filter_wonders_ignore_empty(
    wonders: &mut Vec<&'static Wonder>,
    params: &WondersParams,
) -> Result<()> {
    let res = filter_wonders(wonders, params);
    // Ignore no wonders error, just continue with the empty vec
    if matches!(res, Err(Error::Client(ClientError::NoWondersLeft))) {
        Ok(())
    } else {
        res
    }
}

/// Sorts wonders based on given [`WondersParams`]
fn sort_wonders(wonders: &mut [&'static Wonder], params: &WondersParams) {
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
/// Returns a random wonder, after filtering wonders based on provided query parameters
pub async fn random_wonder(
    Query(params): Query<WondersParams>,
) -> Result<AppJson<&'static Wonder>> {
    if WONDERS.is_empty() {
        return Err(ServerError::WondersEmpty.into());
    }

    let mut rng = rand::thread_rng();

    let mut wonders: Vec<&Wonder> = WONDERS.iter().collect();

    filter_wonders(&mut wonders, &params)?;

    Ok(AppJson(
        wonders.choose(&mut rng).ok_or(ClientError::NoWondersLeft)?,
    ))
}

/// Returns the oldest wonder in the data
pub async fn oldest_wonder() -> Result<AppJson<&'static Wonder>> {
    Ok(AppJson(
        WONDERS
            .iter()
            .reduce(|a, b| if a.build_year < b.build_year { a } else { b })
            .ok_or(ServerError::WondersEmpty)?,
    ))
}

/// Returns the youngest wonder in the data
pub async fn youngest_wonder() -> Result<AppJson<&'static Wonder>> {
    Ok(AppJson(
        WONDERS
            .iter()
            .reduce(|a, b| if a.build_year > b.build_year { a } else { b })
            .ok_or(ServerError::WondersEmpty)?,
    ))
}

/// Returns all wonders after applying filters and sort methods defined by query parameters
pub async fn all_wonders(
    Query(params): Query<WondersParams>,
) -> Result<AppJson<Vec<&'static Wonder>>> {
    let mut wonders: Vec<&Wonder> = WONDERS.iter().collect();

    filter_wonders_ignore_empty(&mut wonders, &params)?;
    sort_wonders(&mut wonders, &params);

    Ok(AppJson(wonders))
}

/// Returns the number of wonders, after applying filters defined by query parameters
pub async fn count_wonders(Query(params): Query<WondersParams>) -> Result<AppJson<usize>> {
    let mut wonders: Vec<&Wonder> = WONDERS.iter().collect();
    filter_wonders_ignore_empty(&mut wonders, &params)?;
    Ok(AppJson(wonders.len()))
}

/// Returns a specific wonder matching the name defined by the path
pub async fn specific_wonder_by_name(Path(name): Path<String>) -> Result<AppJson<&'static Wonder>> {
    let wonder = WONDERS
        .iter()
        .find(|w| w.name.to_ascii_lowercase().replace(' ', "-") == name)
        .ok_or(ClientError::NoMatchingName(name))?;

    Ok(AppJson(wonder))
}
