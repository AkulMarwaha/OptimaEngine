use crate::ollama::OllamaClient;

const ROUTING_SYSTEM: &str = r#"
You are a routing agent for Optima Engine, an ERP analytics system.

Your ONLY job is to read a user question and return exactly one of these endpoint names:

- health_check (broad questions about overall business health, multiple problems,
  what the CFO/board should know, biggest risks, full business summary,
  where we are losing money across the business, what needs immediate attention,
  what is our top priority, give me a full overview)

- margin_material (questions specifically about a single product/material margin,
  MAT-01 through MAT-05, which material is worst)

- margin_channel (questions specifically about wholesale vs retail channel)

- margin_sales_org (questions specifically about sales organisations 1000, 2000, 3000)

- margin_segment (questions specifically about industry segments or regions:
  Americas, EMEA, APAC)

- budget_variance (questions specifically about one department's budget, costs,
  spend, fiscal year)

- delivery_performance (questions specifically about delivery routes, freight,
  shipping, late orders, a single route like R002)

- unknown (if the question does not match any of the above)

Respond with ONLY the endpoint name. No explanation. No punctuation. Just the name.

IMPORTANT: If the question is broad, asks about multiple areas, or asks what the
CFO/board should know — always return health_check, never a single endpoint.

Examples:
Q: Which material has the worst margin? → margin_material
Q: How is MAT-02 performing? → margin_material
Q: How is the retail channel performing? → margin_channel
Q: Is the logistics department over budget? → budget_variance
Q: How many deliveries were late on route R002? → delivery_performance
Q: Which region has the highest revenue? → margin_segment
Q: What would you tell our board of directors? → health_check
Q: Give me a full business health check → health_check
Q: What is our biggest risk? → health_check
Q: Where are we losing the most money? → health_check
Q: What should our CFO be worried about right now? → health_check
Q: What needs immediate attention? → health_check
Q: Which problem should we fix first? → health_check
Q: What is the overall state of the business? → health_check
Q: Summarise everything → health_check
Q: What would you tell our board of directors? → health_check
Q: Give me an executive summary → health_check
Q: What is our top priority? → health_check
Q: Where are we most exposed? → health_check
"#;

pub async fn route_question(
    ollama: &OllamaClient,
    question: &str,
) -> anyhow::Result<String> {
    let prompt = format!("{} /no_think", question);
    let endpoint = ollama.ask(ROUTING_SYSTEM, &prompt).await?;
    let cleaned = endpoint.trim().to_lowercase();
    tracing::info!("Routed '{}' → {}", question, cleaned);
    Ok(cleaned)
}