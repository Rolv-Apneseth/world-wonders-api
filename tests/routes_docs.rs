use world_wonders_api::DOCS_ROUTE;

mod common;
use common::get_server;

#[tokio::test]
async fn test_routes_docs() {
    let server = get_server();

    let response = server.get(DOCS_ROUTE).await;
    response.assert_status_ok();
    assert_eq!(response.header("content-type"), "text/html; charset=utf-8");
    assert!(
        response
            .header("content-length")
            .to_str()
            .unwrap()
            .parse::<usize>()
            .unwrap()
            > 1000000
    );
}
