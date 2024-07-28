use std::net::SocketAddr;

use axum_test::TestServer;
use world_wonders_api::get_app;

/// Get a test server using the router that will be used for the actual server
pub fn get_server() -> TestServer {
    let app = get_app().into_make_service_with_connect_info::<SocketAddr>();
    TestServer::new(app).unwrap()
}
