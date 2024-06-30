use std::{net::SocketAddr, sync::Arc, time::Duration};

use aide::{axum::ApiRouter, openapi::OpenApi, transform::TransformOpenApi};
use axum::{
    extract::{MatchedPath, Request},
    Extension,
};
use tower_governor::{
    governor::GovernorConfigBuilder, key_extractor::SmartIpKeyExtractor, GovernorLayer,
};
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use world_wonders_api::{
    shutdown_signal,
    web::{docs, handler_404, handlers_categories, handlers_wonders},
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

    // API docs generation
    aide::gen::on_error(|error| {
        tracing::error!("Api generation error: {error}");
    });
    aide::gen::extract_schemas(true);
    let mut api = OpenApi::default();

    // Governor configuration for rate-limiting
    let governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .burst_size(10)
            .per_millisecond(200)
            .key_extractor(SmartIpKeyExtractor)
            .finish()
            .expect("Failed setting up `tower_governor` configuration"),
    );

    let app = ApiRouter::new()
        .nest_api_service("/v0/wonders", handlers_wonders::routes())
        .nest_api_service("/v0/categories", handlers_categories::routes())
        .nest_api_service("/docs", docs::routes())
        .fallback(handler_404)
        .finish_api_with(&mut api, api_docs)
        // Docs generation
        .layer(Extension(Arc::new(api)))
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
    tracing::debug!("Listening at http://{}", listener.local_addr().unwrap());
    tracing::debug!(
        "Example docs available at http://{}/docs",
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

fn api_docs(api: TransformOpenApi) -> TransformOpenApi {
    api.title("World Wonders API")
        .description("Free and open source API providing information about world wonders")
}
