use std::net::SocketAddr;

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use world_wonders_api::{config::get_config, get_app, shutdown_signal, DOCS_ROUTE};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "example_error_handling=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app = get_app();

    let config = get_config().expect("Failed to read configuration");

    let addr = SocketAddr::from((config.network.host, config.network.port));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed binding listener");
    tracing::debug!("Listening at http://{}", listener.local_addr().unwrap());
    tracing::debug!(
        "Example docs available at http://{}{DOCS_ROUTE}",
        listener.local_addr().unwrap()
    );

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    .expect("Failed to start server");
}
