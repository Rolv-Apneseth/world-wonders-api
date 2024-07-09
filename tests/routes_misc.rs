use pretty_assertions::assert_eq;

use world_wonders_api::DOCS_ROUTE;

mod common;
use common::get_server;

#[tokio::test]
async fn test_404() {
    let server = get_server();

    let response = server.get("not-a-route").await;
    response.assert_status_not_found();
    response.assert_text_contains("Route not found");
}

#[tokio::test]
async fn test_docs() {
    let server = get_server();

    let response = server.get(DOCS_ROUTE).await;
    response.assert_status_ok();
    assert!(
        response
            .header("content-length")
            .to_str()
            .unwrap()
            .parse::<usize>()
            .unwrap()
            > 1000000
    );
    assert_eq!(response.header("content-type"), "text/html; charset=utf-8");
}
