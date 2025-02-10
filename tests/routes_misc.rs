use pretty_assertions::assert_eq;
use world_wonders_api::{HEALTH_ROUTE, METRICS_ROUTE};

mod common;
use common::get_server;

#[tokio::test]
async fn test_routes_misc() {
    let server = get_server();

    // HEALTH
    let response = server.get(HEALTH_ROUTE).await;
    response.assert_status_ok();
    response.assert_text_contains("Healthy!");

    // 404
    let response = server.get("not-a-route").await;
    response.assert_status_not_found();
    response.assert_text_contains("Route not found");

    // METRICS
    let response = server.get(METRICS_ROUTE).await;
    response.assert_status_ok();
    assert_eq!(response.header("content-type"), "text/plain; charset=utf-8");
    assert!(
        response
            .header("content-length")
            .to_str()
            .unwrap()
            .parse::<usize>()
            .unwrap()
            > 1000
    );
}
