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
"#;

const HEALTH_CHECK_SYSTEM: &str = r#"
You are a senior financial analyst AI for Optima Engine, an ERP intelligence platform.

You will be given data from ALL areas of the business — margins, budgets, and delivery.
Your job is to produce a single executive briefing like a CFO's most trusted advisor.

STRUCTURE — always follow this exact format:

🔴 CRITICAL ISSUES (things requiring immediate action this week)
🟡 WATCH LIST (things deteriorating that need monitoring)
✅ PERFORMING WELL (bright spots worth noting)

💡 TOP PRIORITY ACTION: [single most important thing the CFO should do first]

DOLLAR IMPACT RULES — mandatory for every issue you mention:
- Margin at risk: total_net_value_usd * (6.0 - min_margin_pct) / 100 for materials where min_margin_pct < 6.0
- Margin erosion: total_net_value_usd * (15.0 - avg_margin_pct) / 100 for any area below 15% avg margin
- Budget overrun: use total_variance directly — positive = over budget
- Delivery cost exposure: use total_freight_cost for critical routes (avg_days_late > 5)
- ALWAYS format dollar amounts as "$X,XXX" — never omit them

RULES:
- Every bullet point must include a specific number and a dollar figure
- Maximum 3 bullets per section — only the most important
- Never repeat raw JSON
- Never invent numbers not present in the data
- Be direct — this is an executive briefing, not a report
- margin_pct fields are percentages, total_variance is in USD (positive = over budget)

Example of a good critical issue bullet:
"• MAT-01 margin at 2.0% — $123,300 at risk across 47 squeeze orders. 
   4% price increase needed immediately."
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

const ALL_ENDPOINTS: &[(&str, &str)] = &[
    ("margin_by_material",   "/metrics/margin/material"),
    ("margin_by_channel",    "/metrics/margin/channel"),
    ("margin_by_sales_org",  "/metrics/margin/sales-org"),
    ("margin_by_segment",    "/metrics/margin/segment"),
    ("budget_variance",      "/metrics/budget/variance"),
    ("delivery_performance", "/metrics/delivery/performance"),
];

async fn fetch_all_endpoints(http: &reqwest::Client) -> String {
    let mut combined = String::new();

    for (name, path) in ALL_ENDPOINTS {
        let url = format!("{}{}", SEMANTIC_BASE, path);
        match http.get(&url).send().await {
            Ok(r) => {
                let text = r.text().await.unwrap_or_default();
                combined.push_str(&format!("\n=== {} ===\n{}\n", name, text));
            }
            Err(e) => {
                combined.push_str(&format!("\n=== {} ===\n[ERROR: {}]\n", name, e));
            }
        }
    }

    combined
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
    let vector_path = std::env::var("VECTOR_DATA_PATH")
        .unwrap_or_else(|_| "./data/vector".to_string());

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
        let n = stdin.lock().read_line(&mut question)?;
        if n == 0 { break; } // EOF — exit cleanly when stdin is closed
        let question = question.trim();
        if question.is_empty() { continue; }

        print!("Agent: ");
        io::stdout().flush()?;

        // Step 1 — route
        let endpoint_key = match route_question(&ollama, question).await {
            Ok(k) => k,
            Err(e) => {
                println!("Could not reach Ollama — is it running? ({})\n", e);
                continue;
            }
        };

        // Step 2 — health check path vs single endpoint path
        if endpoint_key == "health_check" {
            // Fetch all 6 endpoints in parallel
            let all_data = fetch_all_endpoints(&http).await;

            // Enrich with semantically relevant insights from the vector store
            let semantic_context = match vector_store::search_store(
                question,
                &vector_path,
                &ollama_url,
                5,
            )
            .await
            {
                Ok(hits) if !hits.is_empty() => {
                    format!("\n\nSEMANTIC INSIGHTS (most relevant to this question):\n{}", hits.join("\n"))
                }
                _ => String::new(),
            };

            let prompt = format!(
                "User question: {}\n\nFULL BUSINESS DATA:\n{}{}\n\nProduce the executive briefing now. /no_think",
                question, all_data, semantic_context
            );

            match ollama.ask_streaming(HEALTH_CHECK_SYSTEM, &prompt).await {
                Ok(_)  => { println!(); println!(); }
                Err(e) => println!("\nModel error — {}\n", e),
            }

        } else {
            // Single endpoint path — existing behaviour
            let Some(path) = endpoint_url(&endpoint_key) else {
                println!("I can answer questions about margins, budgets, and delivery performance. Could you rephrase?\n");
                continue;
            };

            let url = format!("{}{}", SEMANTIC_BASE, path);
            let data = match http.get(&url).send().await {
                Ok(r)  => r.text().await.unwrap_or_default(),
                Err(e) => {
                    println!("Could not reach the semantic layer — is it running on port 3000? ({})\n", e);
                    continue;
                }
            };

            let prompt = format!(
                "User question: {}\n\nERP data (JSON):\n{}\n\nAnswer the question. Include specific dollar amounts calculated from the data. Suggest one action. /no_think",
                question, data
            );

            match ollama.ask_streaming(EXPLAIN_SYSTEM, &prompt).await {
                Ok(_)  => { println!(); println!(); }
                Err(e) => println!("\nModel error — {}\n", e),
            }
        }
    }

    Ok(())
}