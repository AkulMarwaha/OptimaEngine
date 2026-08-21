mod ask;
mod context;
mod routes;
mod ui;

use axum::{routing::{get, post}, Router};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

const SYSTEM_PROMPT_TEMPLATE: &str = r#"You are Optima Engine, an AI assistant that answers questions about company ERP data.

You have access to the following pre-loaded data context which contains aggregated financial and operational metrics from the company's ERP system. Answer any question the user asks based on this data.

CRITICAL RULES:
- Always interpret the most likely intended meaning of a question even if it contains spelling mistakes, grammar errors, abbreviations, or informal language
- NEVER ask the user to rephrase due to spelling or grammar issues
- Only ask for clarification if the business intent is genuinely ambiguous and cannot be reasonably inferred
- Always include specific dollar amounts and percentages in your answers
- Always end your answer with one clear recommended action
- If data quality issues were found and fixed during processing, include a brief note about what was fixed and how it affects the answer
- Keep answers concise and direct — lead with the number, then the explanation, then the action
- If a question cannot be answered from the available data, say so clearly and explain what data would be needed

DATA INTERPRETATION:
- In BUDGET VARIANCE data: positive total_variance = OVER budget (cost overrun, BAD). Negative = under budget (savings, good).
- When asked what to worry about or what is over budget: focus ONLY on departments with POSITIVE total_variance.
- total_budget is the planned spend. total_actual_cost is what was spent. total_variance = actual minus budget.

REASONING INSTRUCTIONS:
Before answering, scan all sections of the data context. Identify which section is most relevant to the question. Check every numeric value in that section. Find the single most important finding. Only then write your answer.

Your answer must:
- Lead with the most critical finding and its exact dollar amount from the data
- Include specific numbers for every claim — never round or estimate
- End with one concrete recommended action
- Never ask the user to rephrase

COMPANY DATA CONTEXT:
{context}"#;

#[derive(Clone)]
pub struct AppState {
    pub gold_path: String,
    pub bronze_path: String,
    pub config_path: String,
    pub system_prompt: String,
    pub ollama_url: String,
    pub ollama_model: String,
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
    let bronze_path = std::env::var("BRONZE_DATA_PATH")
        .unwrap_or_else(|_| "./data/bronze".to_string());
    let config_path = std::env::var("DATA_CONFIG_PATH")
        .unwrap_or_else(|_| "./data/config".to_string());
    let ollama_url = std::env::var("AI_AGENT_BASE_URL")
        .unwrap_or_else(|_| "http://localhost:11434".to_string());
    let ollama_model = std::env::var("AI_AGENT_MODEL")
        .unwrap_or_else(|_| "qwen2.5:1.5b".to_string());

    tracing::info!("Loading Gold context from {}", gold_path);
    let gold_context = context::load_gold_context(&gold_path)
        .expect("Failed to load Gold context — run the transformation pipeline first");
    tracing::info!("Gold context loaded ({} bytes)", gold_context.len());

    let system_prompt = SYSTEM_PROMPT_TEMPLATE.replace("{context}", &gold_context);

    let model_name = ollama_model.clone();
    let state = Arc::new(AppState {
        gold_path,
        bronze_path,
        config_path,
        system_prompt,
        ollama_url,
        ollama_model,
    });

    let app = Router::new()
        .route("/",                             get(ui::index))
        .route("/health",                       get(routes::health))
        .route("/ask",                          post(ask::ask))
        .route("/metrics/margin/material",      get(routes::margin_by_material))
        .route("/metrics/margin/channel",       get(routes::margin_by_channel))
        .route("/metrics/margin/sales-org",     get(routes::margin_by_sales_org))
        .route("/metrics/margin/segment",       get(routes::margin_by_segment))
        .route("/metrics/budget/variance",      get(routes::budget_variance))
        .route("/metrics/delivery/performance", get(routes::delivery_performance))
        .route("/ingest/upload",                post(routes::upload_erp_file))
        .route("/ingest/confirm-mapping",       post(routes::confirm_mapping))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind port 3000");

    tracing::info!("Optima Engine listening on http://localhost:3000");
    tracing::info!("Model: {}", model_name);
    axum::serve(listener, app).await.unwrap();
}
