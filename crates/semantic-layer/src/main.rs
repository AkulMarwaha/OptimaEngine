mod routes;

use axum::{routing::get, Router};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone)]
pub struct AppState {
    pub gold_path: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::new(
                std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
            ),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    dotenvy::dotenv().ok();

    let gold_path = std::env::var("GOLD_DATA_PATH")
        .unwrap_or_else(|_| "./data/gold".to_string());

    tracing::info!("Gold data path: {}", gold_path);

    let state = Arc::new(AppState { gold_path });

    let app = Router::new()
        .route("/health",                       get(routes::health))
        .route("/metrics/margin/material",      get(routes::margin_by_material))
        .route("/metrics/margin/channel",       get(routes::margin_by_channel))
        .route("/metrics/margin/sales-org",     get(routes::margin_by_sales_org))
        .route("/metrics/margin/segment",       get(routes::margin_by_segment))
        .route("/metrics/budget/variance",      get(routes::budget_variance))
        .route("/metrics/delivery/performance", get(routes::delivery_performance))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind port 3000");

    tracing::info!("Semantic Layer API listening on http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}