use std::{net::SocketAddr, sync::Arc, time::Duration};

use axum::{
    extract::{MatchedPath, Request},
    Router,
};
use tower_governor::{
    governor::GovernorConfigBuilder, key_extractor::SmartIpKeyExtractor, GovernorLayer,
};
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use world_wonders_api::{
    shutdown_signal,
    web::{handler_404, handlers_categories, handlers_wonders},
    PORT,
};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "example_error_handling=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Governor configuration for rate-limiting
    let governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .burst_size(10)
            .per_millisecond(200)
            .key_extractor(SmartIpKeyExtractor)
            .finish()
            .expect("Failed setting up `tower_governor` configuration"),
    );

    let app = Router::new()
        .nest("/v0/wonders", handlers_wonders::routes())
        .nest("/v0/categories", handlers_categories::routes())
        .fallback(handler_404)
        // Rate-limiting
        .layer(GovernorLayer {
            config: governor_conf,
        })
        // Logging
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|req: &Request| {
                    let method = req.method();
                    let uri = req.uri();

                    let matched_path = req
                        .extensions()
                        .get::<MatchedPath>()
                        .map(|matched_path| matched_path.as_str());

                    tracing::debug_span!("request", %method, %uri, matched_path)
                })
                // By default `TraceLayer` will log 5xx responses but we're doing our specific
                // logging of errors so disable that
                .on_failure(()),
        )
        // Timeout
        .layer(TimeoutLayer::new(Duration::from_secs(10)));

    let addr = SocketAddr::from(([127, 0, 0, 1], PORT));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed binding listener");
    tracing::debug!("listening on {}", listener.local_addr().unwrap());

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    .expect("Failed to start server");
}
