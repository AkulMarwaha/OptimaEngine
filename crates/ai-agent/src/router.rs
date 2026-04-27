use crate::ollama::OllamaClient;

const ROUTING_SYSTEM: &str = r#"
You are a routing agent for Optima Engine, an ERP analytics system.
Your ONLY job is to read a user question and return exactly one of these endpoint names:

- margin_material       (questions about product/material margins, MAT-01 through MAT-05)
- margin_channel        (questions about wholesale vs retail channel performance)
- margin_sales_org      (questions about sales organisations 1000, 2000, 3000)
- margin_segment        (questions about industry segments or regions: Americas, EMEA, APAC)
- budget_variance       (questions about budgets, costs, spend, departments, fiscal year)
- delivery_performance  (questions about delivery, freight, shipping, late orders, routes)
- unknown               (if the question does not match any of the above)

Respond with ONLY the endpoint name. No explanation. No punctuation. Just the name.

Examples:
Q: Which material has the worst margin? → margin_material
Q: How is the retail channel performing? → margin_channel
Q: Is the logistics department over budget? → budget_variance
Q: How many deliveries were late on route R002? → delivery_performance
Q: Which region has the highest revenue? → margin_segment
Q: What would you tell our board of directors? → budget_variance
Q: Give me a full business health check → margin_material
Q: What is our biggest risk? → margin_material
Q: Where are we losing the most money? → margin_material
Q: What should our CEO or CFO know? → budget_variance
Q: Summarise our financial performance → budget_variance
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