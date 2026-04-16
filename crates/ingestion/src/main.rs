mod mockaroo_client;
mod pipeline;
mod connectors;

use optima_core::config::AppConfig;
use mockaroo_client::MockarooClient;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env()
            .add_directive("ingestion=info".parse()?))
        .init();

    let config = AppConfig::from_env()?;
    let client = MockarooClient::new(config.clone());

    let schemas = [
        (config.schema_sales_header.as_str(), "SAP_Sales_Header", 500u32),
        (config.schema_sales_items.as_str(), "SAP_Sales_Items",  500u32),
    ];

    for (schema_id, schema_name, count) in &schemas {
        match client.ingest_schema(schema_id, schema_name, *count).await {
            Ok(path) => info!("✅ {} → {:?}", schema_name, path),
            Err(e) => {
                tracing::error!("❌ Failed to ingest '{}': {}", schema_name, e);
                return Err(e);
            }
        }
    }

    println!("\n🏁 Bronze ingestion complete.");
    println!("   data/bronze/SAP_Sales_Header.csv");
    println!("   data/bronze/SAP_Sales_Items.csv");

    Ok(())
}
