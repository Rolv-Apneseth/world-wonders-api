use std::{sync::Arc, time::Duration};

use aide::{axum::ApiRouter, openapi::OpenApi, transform::TransformOpenApi};
use axum::{
    extract::{MatchedPath, Request},
    Extension, Router,
};
use routes::{docs, handler_404, wonders};
use tokio::signal;
use tower_governor::{
    governor::GovernorConfigBuilder, key_extractor::SmartIpKeyExtractor, GovernorLayer,
};
use tower_http::{timeout::TimeoutLayer, trace::TraceLayer};

pub mod data;
pub mod error;
pub mod extractors;
pub mod routes;

// If changing, remember to update the Docker files
pub const PORT: u16 = 8138;
pub const DOCS_ROUTE: &str = "/v0/docs";
pub const WONDERS_ROUTE: &str = "/v0/wonders";

pub fn get_app() -> Router {
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

    ApiRouter::new()
        .nest_api_service(WONDERS_ROUTE, wonders::routes())
        .nest_api_service(DOCS_ROUTE, docs::routes())
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
        .layer(TimeoutLayer::new(Duration::from_secs(10)))
}
fn api_docs(api: TransformOpenApi) -> TransformOpenApi {
    api.title("World Wonders API")
        .description("Free and open source API providing information about world wonders")
}

// For graceful shutdown
pub async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
