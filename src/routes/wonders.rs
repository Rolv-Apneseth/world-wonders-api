use aide::{
    axum::{routing::get_with, ApiRouter, IntoApiResponse},
    transform::TransformOperation,
};
use axum::{extract::Path, response::IntoResponse};
use axum_valid::Garde;
use garde::Validate;
use rand::prelude::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::{
    data::{Category, TimePeriod, Wonder, WONDERS},
    error::{Error, ErrorResponse, Result},
    extractors::{Json, Query},
};

#[derive(Debug, Deserialize, JsonSchema, Default, Validate)]
#[garde(allow_unvalidated)]
pub struct WonderParamsFiltering {
    #[garde(length(min = 1, max = 150))]
    name: Option<String>,
    #[garde(length(min = 1, max = 150))]
    location: Option<String>,
    time_period: Option<TimePeriod>,
    lower_limit: Option<i16>,
    upper_limit: Option<i16>,
    category: Option<Category>,
}

#[derive(Debug, Deserialize, JsonSchema, Validate)]
#[garde(allow_unvalidated)]
pub struct WonderParamsSorting {
    sort_by: Option<SortBy>,
    sort_reverse: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CategoriesParams {
    exclude_games: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq, PartialOrd, Ord, EnumIter)]
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
            "/categories",
            get_with(get_wonder_categories, get_wonder_categories_docs),
        )
        .api_route(
            "/time-periods",
            get_with(get_wonder_time_periods, get_wonder_time_periods_docs),
        )
        .api_route(
            "/sort-by",
            get_with(get_wonder_sort_by, get_wonder_sort_by_docs),
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
    Garde(Query(filtering_params)): Garde<Query<WonderParamsFiltering>>,
    Garde(Query(sorting_params)): Garde<Query<WonderParamsSorting>>,
) -> impl IntoApiResponse {
    let mut wonders: Vec<&Wonder> = WONDERS.iter().collect();

    if let Err(e) = filter_wonders_ignore_empty(&mut wonders, filtering_params) {
        return e.into_response();
    };
    sort_wonders(&mut wonders, sorting_params);

    Json(wonders).into_response()
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
    Garde(Query(filtering_params)): Garde<Query<WonderParamsFiltering>>,
) -> impl IntoApiResponse {
    let mut wonders: Vec<&Wonder> = WONDERS.iter().collect();
    if let Err(e) = filter_wonders_ignore_empty(&mut wonders, filtering_params) {
        return e.into_response();
    };
    Json(wonders.len()).into_response()
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

// GET WONDER CATEGORIES
async fn get_wonder_categories(Query(params): Query<CategoriesParams>) -> impl IntoApiResponse {
    let categories: Vec<Category> = Category::iter()
        // Filter out game categories :(
        .filter(|c| {
            !matches!(
                (params.exclude_games, c),
                (Some(true), Category::Civ5 | Category::Civ6)
            )
        })
        .collect();

    Json(categories).into_response()
}
fn get_wonder_categories_docs(op: TransformOperation) -> TransformOperation {
    op.summary("Wonder categories")
        .description("Get all available wonder categories")
        .response_with::<200, Json<Vec<Category>>, _>(|res| {
            res.example(Category::iter().collect::<Vec<Category>>())
        })
        .response_with::<400, ErrorResponse, _>(|res| {
            res.description("Bad request")
                .example(ErrorResponse::new(Error::InvalidRequest(
                    "Failed to deserialize query string".to_string(),
                )))
        })
}

// GET WONDER TIME PERIODS
async fn get_wonder_time_periods() -> impl IntoApiResponse {
    Json(TimePeriod::iter().collect::<Vec<TimePeriod>>()).into_response()
}
fn get_wonder_time_periods_docs(op: TransformOperation) -> TransformOperation {
    op.summary("Wonder time periods")
        .description("Get all available human history time periods for wonders' construction times")
        .response_with::<200, Json<Vec<TimePeriod>>, _>(|res| {
            res.example(TimePeriod::iter().collect::<Vec<TimePeriod>>())
        })
}

// GET WONDER SORT BY OPTIONS
async fn get_wonder_sort_by() -> impl IntoApiResponse {
    Json(SortBy::iter().collect::<Vec<SortBy>>()).into_response()
}
fn get_wonder_sort_by_docs(op: TransformOperation) -> TransformOperation {
    op.summary("Wonder sort by options")
        .description("Get all valid options for sorting wonders")
        .response_with::<200, Json<Vec<SortBy>>, _>(|res| {
            res.example(SortBy::iter().collect::<Vec<SortBy>>())
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

    Json(wonder).into_response()
}
fn get_wonder_by_name_docs(op: TransformOperation) -> TransformOperation {
    op.summary("Specific wonder - by name")
        .description(
            "Get a specific wonder matching the name defined by the path. Note that
the name will be parsed as lowercase letters with spaces replaced with '-'.",
        )
        .response_with::<200, Json<&'static Wonder>, _>(|res| res.example(&WONDERS[1]))
        .response_with::<400, ErrorResponse, _>(|res| {
            res.description("Bad request")
                .example(ErrorResponse::new(Error::NoMatchingName("...".to_owned())))
        })
}

// GET RANDOM WONDER
async fn get_random_wonder(
    Garde(Query(filtering_params)): Garde<Query<WonderParamsFiltering>>,
) -> impl IntoApiResponse {
    assert!(WONDERS.len() > 0);

    let mut rng = rand::thread_rng();

    let mut wonders: Vec<&Wonder> = WONDERS.iter().collect();
    if let Err(e) = filter_wonders(&mut wonders, filtering_params) {
        return e.into_response();
    };

    Json(wonders.choose(&mut rng).unwrap()).into_response()
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
    Garde(Query(filtering_params)): Garde<Query<WonderParamsFiltering>>,
) -> impl IntoApiResponse {
    assert!(WONDERS.len() > 0);

    let mut wonders: Vec<&Wonder> = WONDERS.iter().collect();
    if let Err(e) = filter_wonders(&mut wonders, filtering_params) {
        return e.into_response();
    };

    Json(
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
    Garde(Query(filtering_params)): Garde<Query<WonderParamsFiltering>>,
) -> impl IntoApiResponse {
    assert!(WONDERS.len() > 0);

    let mut wonders: Vec<&Wonder> = WONDERS.iter().collect();
    if let Err(e) = filter_wonders(&mut wonders, filtering_params) {
        return e.into_response();
    };

    Json(
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

#[cfg(test)]
mod tests {
    use axum::{routing::get, Router};
    use axum_test::TestServer;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::{extract_response, get_route_server};

    // UNIT TESTS - HELPERS
    #[test]
    fn test_filter_wonders_ok() {
        let mut wonders: Vec<&Wonder> = WONDERS.iter().collect();
        assert!(filter_wonders(
            &mut wonders,
            WonderParamsFiltering {
                category: Some(Category::SevenWonders),
                ..Default::default()
            },
        )
        .is_ok());
        assert_eq!(wonders.len(), 7);

        let mut wonders: Vec<&Wonder> = WONDERS.iter().collect();
        assert!(filter_wonders(
            &mut wonders,
            WonderParamsFiltering {
                name: Some("al".to_string()),
                time_period: Some(TimePeriod::PostClassical),
                ..Default::default()
            },
        )
        .is_ok());
        wonders.iter().for_each(|w| {
            assert_eq!(w.time_period, TimePeriod::PostClassical);
            assert!(w.name.to_lowercase().contains("al"));
        });

        let mut wonders: Vec<&Wonder> = WONDERS.iter().collect();
        assert!(filter_wonders(
            &mut wonders,
            WonderParamsFiltering {
                location: Some("ro".to_string()),
                lower_limit: Some(-200),
                upper_limit: Some(1000),
                ..Default::default()
            },
        )
        .is_ok());
        wonders.iter().for_each(|w| {
            assert!(w.build_year >= -200);
            assert!(w.build_year <= 1000);
            assert!(w.location.to_lowercase().contains("ro"));
        });
    }

    #[test]
    fn test_filter_wonders_errors() {
        // Empty
        let mut wonders: Vec<&Wonder> = WONDERS.iter().collect();
        assert!(filter_wonders(
            &mut wonders,
            WonderParamsFiltering {
                name: Some("abcdefghijk".to_string()),
                ..Default::default()
            },
        )
        .is_err_and(|e| matches!(e, Error::NoWondersLeft)));
        assert_eq!(wonders.len(), 0);

        // Conflicting limits
        let mut wonders: Vec<&Wonder> = WONDERS.iter().collect();
        assert!(filter_wonders(
            &mut wonders,
            WonderParamsFiltering {
                lower_limit: Some(500),
                upper_limit: Some(400),
                ..Default::default()
            },
        )
        .is_err_and(|e| matches!(e, Error::ConflictingLimitParams(500, 400))));
    }

    #[test]
    fn test_filter_wonders_ignore_empty() {
        let mut wonders: Vec<&Wonder> = WONDERS.iter().collect();
        assert!(filter_wonders_ignore_empty(
            &mut wonders,
            WonderParamsFiltering {
                name: Some("abcdefghijk".to_string()),
                ..Default::default()
            },
        )
        .is_ok());
        assert_eq!(wonders.len(), 0);

        // Other errors should not be ignored
        let mut wonders: Vec<&Wonder> = WONDERS.iter().collect();
        assert!(filter_wonders(
            &mut wonders,
            WonderParamsFiltering {
                lower_limit: Some(1000),
                upper_limit: Some(-400),
                ..Default::default()
            },
        )
        .is_err_and(|e| matches!(e, Error::ConflictingLimitParams(1000, -400))));
    }

    #[test]
    fn test_sort_wonders() {
        let mut wonders: Vec<&Wonder> = WONDERS.iter().collect();

        sort_wonders(
            &mut wonders,
            WonderParamsSorting {
                sort_by: Some(SortBy::BuildYear),
                sort_reverse: None,
            },
        );
        wonders.iter().reduce(|a, b| {
            assert!(a.build_year <= b.build_year);
            b
        });

        // Reset working `Vec`, as sorting is stable
        let mut wonders: Vec<&Wonder> = WONDERS.iter().collect();

        sort_wonders(
            &mut wonders,
            WonderParamsSorting {
                sort_by: Some(SortBy::Alphabetical),
                sort_reverse: Some(true),
            },
        );
        wonders.iter().reduce(|a, b| {
            assert!(a.name > b.name);
            b
        });
    }

    // UNIT TESTS - ROUTES
    #[tokio::test]
    async fn test_get_all_wonders() {
        let server = get_route_server!(get_all_wonders);

        let wonders = extract_response!(server, Vec<Wonder>);
        assert_eq!(wonders.len(), WONDERS.len());

        let wonders = extract_response!(server, Vec<Wonder>, "/?category=SevenWonders");
        assert_eq!(wonders.len(), 7);
    }

    #[tokio::test]
    async fn test_get_count_wonders() {
        let server = get_route_server!(get_count_wonders);

        let count = extract_response!(server, u16);
        assert_eq!(count as usize, WONDERS.len());

        let count = extract_response!(server, u16, "/?category=SevenWonders");
        assert_eq!(count as usize, 7);
    }

    #[tokio::test]
    async fn test_get_wonder_categories() {
        let server = get_route_server!(get_wonder_categories);

        let categories = extract_response!(server, Vec<Category>);
        assert_eq!(categories, Category::iter().collect::<Vec<Category>>());
    }

    #[tokio::test]
    async fn test_get_wonder_time_periods() {
        let server = get_route_server!(get_wonder_time_periods);

        let time_periods = extract_response!(server, Vec<TimePeriod>);
        assert_eq!(
            time_periods,
            TimePeriod::iter().collect::<Vec<TimePeriod>>()
        );
    }

    #[tokio::test]
    async fn test_get_wonder_sort_by() {
        let server = get_route_server!(get_wonder_sort_by);

        let sort_by = extract_response!(server, Vec<SortBy>);
        assert_eq!(sort_by, SortBy::iter().collect::<Vec<SortBy>>());
    }

    #[tokio::test]
    async fn test_get_wonder_by_name() {
        let app = Router::new().route("/:name", get(get_wonder_by_name));
        let server = TestServer::new(app).unwrap();

        let expected = &WONDERS[0];
        let wonder = extract_response!(
            server,
            Wonder,
            &format!("/{}", expected.name.to_lowercase().replace(' ', "-"))
        );
        assert_eq!(&wonder, expected);

        let error_response = server.get("/").await;
        error_response.assert_status_not_found();
        let error_response = server.get("/a").await;
        error_response.assert_status_bad_request();
    }

    #[tokio::test]
    async fn test_get_random_wonder() {
        let server = get_route_server!(get_random_wonder);

        let response = server.get("/").await;
        let wonder = response.json::<Wonder>();
        assert!(WONDERS.contains(&wonder));
    }

    #[tokio::test]
    async fn test_get_oldest_wonder() {
        let server = get_route_server!(get_oldest_wonder);

        let expected = WONDERS
            .iter()
            .min_by(|a, b| a.build_year.cmp(&b.build_year))
            .unwrap();
        let wonder = extract_response!(server, Wonder);
        assert_eq!(&wonder, expected);
    }

    #[tokio::test]
    async fn test_get_youngest_wonder() {
        let server = get_route_server!(get_youngest_wonder);

        let expected = WONDERS
            .iter()
            .max_by(|a, b| a.build_year.cmp(&b.build_year))
            .unwrap();
        let wonder = extract_response!(server, Wonder);
        assert_eq!(&wonder, expected);
    }
}
