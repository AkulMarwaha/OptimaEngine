mod context;
mod ollama;

use context::load_gold_context;
use ollama::OllamaClient;
use std::io::{self, BufRead, Write};
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
        .unwrap_or_else(|_| "qwen2.5:1.5b".to_string());
    let gold_path = std::env::var("GOLD_DATA_PATH")
        .unwrap_or_else(|_| "./data/gold".to_string());

    // Load all Gold context once at startup
    let context = load_gold_context(&gold_path)?;
    let system_prompt = SYSTEM_PROMPT_TEMPLATE.replace("{context}", &context);

    let ollama = OllamaClient::new(&ollama_url, &ollama_model);

    println!("\n╔═══════════════════════════════════════╗");
    println!("║      Optima Engine — AI Agent         ║");
    println!("│  Model: {:>28} │", ollama_model);
    println!("╚═══════════════════════════════════════╝\n");
    println!("Ask me anything about your ERP data. Ctrl+C to exit.\n");

    let stdin = io::stdin();

    loop {
        print!("You: ");
        io::stdout().flush()?;

        let mut question = String::new();
        let n = stdin.lock().read_line(&mut question)?;
        if n == 0 { break; }
        let question = question.trim();
        if question.is_empty() { continue; }

        print!("Agent: ");
        io::stdout().flush()?;

        let prompt = format!("Question: {}", question);
        match ollama.ask_streaming(&system_prompt, &prompt).await {
            Ok(_)  => { println!(); println!(); }
            Err(e) => println!("\nCould not reach Ollama — is it running? ({})\n", e),
        }
    }

    Ok(())
}
