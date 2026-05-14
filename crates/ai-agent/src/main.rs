mod ollama;
mod router;

use ollama::OllamaClient;
use router::route_question;
use std::io::{self, BufRead, Write};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

const SEMANTIC_BASE: &str = "http://localhost:3000";

const EXPLAIN_SYSTEM: &str = r#"
You are a senior financial analyst AI for Optima Engine, an ERP intelligence platform.

You will be given a user question and a JSON dataset from an ERP analytics API.
Your job is to answer in plain English like a CFO's most trusted advisor.

DOLLAR IMPACT RULES — these are mandatory, never skip them:
- For margin questions: calculate dollar impact as (total_net_value_usd * (target_margin - avg_margin_pct) / 100) where target margin is 15%. Call this "margin erosion".
- For squeeze questions: calculate dollar impact as (total_net_value_usd * (6.0 - min_margin_pct) / 100) for any material where min_margin_pct < 6.0. Call this "margin at risk".
- For budget questions: use total_variance directly as the dollar figure. Positive = over budget, negative = under budget.
- For delivery questions: use total_freight_cost and avg_days_late together. Flag any route where avg_days_late > 5 as critical.
- ALWAYS express dollar amounts as "$X,XXX" formatted numbers. Never leave out the dollar figure if the data supports it.

ANSWER RULES:
- Lead with the single most important finding and its dollar impact
- Be specific with numbers — always reference the exact metric and the dollar amount
- If something is critical, say CRITICAL explicitly
- Suggest one concrete action the CFO can take
- Keep answers under 120 words
- Never repeat raw JSON
- Never invent numbers that are not in the data

CONTEXT RULES:
- margin_pct fields are percentages (e.g. 6.0 = 6%)
- total_net_value_usd and total_cost_usd are in USD
- total_variance is in USD — positive means over budget
- avg_days_late is in calendar days
- squeeze_count is number of orders below the 6% margin threshold

Example of a good answer:
"CRITICAL: MAT-01 is your most urgent problem. With a minimum margin of 2.1% on $84,200 
in revenue, you have $3,284 in margin at risk this period — selling below your cost 
threshold on 47 orders. Immediate action: raise MAT-01 pricing by at least 4% or 
renegotiate supplier cost before next quarter."
"#;

fn endpoint_url(endpoint: &str) -> Option<&'static str> {
    match endpoint {
        "margin_material" => Some("/metrics/margin/material"),
        "margin_channel"  => Some("/metrics/margin/channel"),
        "margin_sales_org" => Some("/metrics/margin/sales-org"),
        "margin_segment"  => Some("/metrics/margin/segment"),
        "budget_variance" => Some("/metrics/budget/variance"),
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
        .unwrap_or_else(|_| "qwen3:4b".to_string());

    let ollama = OllamaClient::new(&ollama_url, &ollama_model);
    let http = reqwest::Client::new();

    println!("\n╔═══════════════════════════════════════╗");
    println!("║      Optima Engine — AI Agent         ║");
    println!("│  Model: {:>28} │", ollama_model);
    println!("║  API:   {:>28} ║", SEMANTIC_BASE);
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

        // Step 1 — route using Ollama
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

        // Step 3 — explain with dollar impact context
        let prompt = format!(
            "User question: {}\n\nERP data (JSON):\n{}\n\nAnswer the question. Include specific dollar amounts calculated from the data. Suggest one action. /no_think",
            question, data
        );

        match ollama.ask(EXPLAIN_SYSTEM, &prompt).await {
            Ok(answer) => println!("\rAgent: {}\n", answer),
            Err(e)     => println!("\rAgent: Model error — {}\n", e),
        }
    }
}