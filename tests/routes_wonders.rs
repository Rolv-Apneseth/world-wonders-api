use pretty_assertions::assert_eq;
use strum::IntoEnumIterator;
use world_wonders_api::{
    data::{Category, TimePeriod, Wonder, WONDERS},
    WONDERS_ROUTE,
};

mod common;
use common::get_server;

#[tokio::test]
async fn test_routes_wonders() {
    let server = get_server();

    // All wonders
    let response = server.get(WONDERS_ROUTE).await;
    response.assert_status_ok();
    let wonders = response.json::<Vec<Wonder>>();
    assert_eq!(wonders.len(), WONDERS.len());
    assert_eq!(wonders[0], WONDERS[0]);
    assert_eq!(wonders[12], WONDERS[12]);

    // Count
    let response = server
        .get(&format!("{WONDERS_ROUTE}/count?category=SevenWonders"))
        .await;
    response.assert_status_ok();
    response.assert_json::<u16>(&7);

    // Categories
    let response = server.get(&format!("{WONDERS_ROUTE}/categories")).await;
    response.assert_status_ok();
    response.assert_json::<Vec<Category>>(&Category::iter().collect::<Vec<Category>>());

    // Time periods
    let response = server.get(&format!("{WONDERS_ROUTE}/time-periods")).await;
    response.assert_status_ok();
    response.assert_json::<Vec<TimePeriod>>(&TimePeriod::iter().collect::<Vec<TimePeriod>>());

    // Specific wonder - by name
    let response = server.get(&format!("{WONDERS_ROUTE}/name/alhambra")).await;
    response.assert_status_ok();
    response.assert_json::<Wonder>(WONDERS.iter().find(|w| w.name == "Alhambra").unwrap());

    // Specific wonder - random
    let response = server.get(&format!("{WONDERS_ROUTE}/name/alhambra")).await;
    response.assert_status_ok();
    let _ = response.json::<Wonder>();

    // Specific wonder - oldest
    let response = server.get(&format!("{WONDERS_ROUTE}/oldest")).await;
    response.assert_status_ok();
    response.assert_json::<Wonder>(
        WONDERS
            .iter()
            .reduce(|a, b| if a.build_year < b.build_year { a } else { b })
            .unwrap(),
    );

    // Specific wonder - youngest
    let response = server.get(&format!("{WONDERS_ROUTE}/youngest")).await;
    response.assert_status_ok();
    response.assert_json::<Wonder>(
        WONDERS
            .iter()
            .reduce(|a, b| if a.build_year > b.build_year { a } else { b })
            .unwrap(),
    );
}
