mod bronze_to_silver;
mod silver_to_gold;
mod compute;

use dotenvy::dotenv;
use std::env;
use tracing_subscriber::EnvFilter;

fn main() -> anyhow::Result<()> {
    // Structured logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env()
            .add_directive("transformation=info".parse()?))
        .init();

    dotenv().ok();

    let bronze_path = env::var("BRONZE_DATA_PATH")
        .unwrap_or_else(|_| "./data/bronze".to_string());
    let silver_path = env::var("SILVER_DATA_PATH")
        .unwrap_or_else(|_| "./data/silver".to_string());

    // Run Bronze → Silver transformation
    bronze_to_silver::run(&bronze_path, &silver_path)?;

    println!("\n🥈 Silver layer complete.");
    println!("   data/silver/sales_enriched.parquet");

    Ok(())
}