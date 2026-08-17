mod mockaroo_client;
mod pipeline;
mod connectors;
pub mod field_matcher;

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
        (config.schema_sales_header.as_str(),    "SAP_Sales_Header",    500u32),
        (config.schema_sales_items.as_str(),     "SAP_Sales_Items",     500u32),
        (config.schema_customer_master.as_str(), "SAP_Customer_Master", 100u32),
        (config.schema_material_master.as_str(), "SAP_Material_Master",  50u32),
        (config.schema_controlling.as_str(),     "SAP_Controlling",     500u32),
        (config.schema_delivery.as_str(),        "SAP_Delivery",        500u32),
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
    println!("   SAP_Sales_Header    → data/bronze/SAP_Sales_Header.csv");
    println!("   SAP_Sales_Items     → data/bronze/SAP_Sales_Items.csv");
    println!("   SAP_Customer_Master → data/bronze/SAP_Customer_Master.csv");
    println!("   SAP_Material_Master → data/bronze/SAP_Material_Master.csv");
    println!("   SAP_Controlling     → data/bronze/SAP_Controlling.csv");
    println!("   SAP_Delivery        → data/bronze/SAP_Delivery.csv");

    Ok(())
}