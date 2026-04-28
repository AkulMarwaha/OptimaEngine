mod ollama;
mod router;

use ollama::OllamaClient;
use router::route_question;
use std::io::{self, BufRead, Write};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

const SEMANTIC_BASE: &str = "http://localhost:3000";

const EXPLAIN_SYSTEM: &str = r#"
You are an ERP analytics assistant for Optima Engine.
You will be given a user question and a JSON dataset from an ERP analytics API.
Your job is to answer the question clearly and concisely in plain English.
Focus on the most important insight. Highlight anything that looks like a problem.
- The key margin fields are: avg_margin_pct, min_margin_pct, squeeze_count
- If min_margin_pct is below 10%, flag it as a CRITICAL margin squeeze risk
- If squeeze_count is above 0, flag it as at risk
- If budget variance is positive (over budget), flag it with the dollar amount
- If avg_days_late is above 5, flag the route as critical
- Always reference min_margin_pct when discussing squeeze risk, not avg_margin_pct
Keep your answer under 100 words. Be specific with numbers. Do not repeat raw JSON.
"#;

fn endpoint_url(endpoint: &str) -> Option<&'static str> {
    match endpoint {
        "margin_material"      => Some("/metrics/margin/material"),
        "margin_channel"       => Some("/metrics/margin/channel"),
        "margin_sales_org"     => Some("/metrics/margin/sales-org"),
        "margin_segment"       => Some("/metrics/margin/segment"),
        "budget_variance"      => Some("/metrics/budget/variance"),
        "delivery_performance" => Some("/metrics/delivery/performance"),
        _ => None,
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "warn".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    dotenvy::dotenv().ok();

    let ollama_url = std::env::var("AI_AGENT_BASE_URL")
        .unwrap_or_else(|_| "http://localhost:11434".to_string());
    let ollama_model = std::env::var("AI_AGENT_MODEL")
        .unwrap_or_else(|_| "qwen3:1.7b".to_string());

    let ollama = OllamaClient::new(&ollama_url, &ollama_model);
    let http   = reqwest::Client::new();

    println!("\n╔═══════════════════════════════════════╗");
    println!("║     Optima Engine — AI Agent          ║");
    println!("│     Model:   {:>26} │", ollama_model);
    println!("║     API:   {:>26} ║", SEMANTIC_BASE);
    println!("│     Backend: {:>24} │", "ollama");
    println!("╚═══════════════════════════════════════╝\n");
    println!("Ask me anything about your ERP data. Ctrl+C to exit.\n");

    let stdin = io::stdin();
    loop {
        print!("You: ");
        io::stdout().flush()?;

        let mut question = String::new();
        stdin.lock().read_line(&mut question)?;
        let question = question.trim();
        if question.is_empty() { continue; }

        print!("Agent: thinking...");
        io::stdout().flush()?;

        // Step 1 — route using Ollama (fast, one word response)
        let endpoint_key = match route_question(&ollama, question).await {
            Ok(k) => k,
            Err(e) => {
                println!("\rAgent: Could not reach Ollama — is it running? ({})\n", e);
                continue;
            }
        };

        let Some(path) = endpoint_url(&endpoint_key) else {
            println!("\rAgent: I can answer questions about margins, budgets, and delivery performance. Could you rephrase?\n");
            continue;
        };

        // Step 2 — fetch data from semantic layer
        let url = format!("{}{}", SEMANTIC_BASE, path);
        let data = match http.get(&url).send().await {
            Ok(r)  => r.text().await.unwrap_or_default(),
            Err(e) => {
                println!("\rAgent: Could not reach the semantic layer — is it running on port 3000? ({})\n", e);
                continue;
            }
        };

        // Step 3 — explain using selected backend
        let prompt = format!(
            "User question: {}\n\nERP data (JSON):\n{}\n\nAnswer the question based on this data. /no_think",
            question, data
        );

       let result = ollama.ask(EXPLAIN_SYSTEM, &prompt).await;

        match result {
            Ok(answer) => println!("\rAgent: {}\n", answer),
            Err(e)     => println!("\rAgent: Model error — {}\n", e),
        }
    }
}