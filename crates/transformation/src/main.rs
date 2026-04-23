mod bronze_to_silver;
mod silver_to_gold;
mod compute;

use dotenvy::dotenv;
use std::env;
use tracing_subscriber::EnvFilter;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env()
            .add_directive("transformation=info".parse()?))
        .init();

    dotenv().ok();

    let bronze_path = env::var("BRONZE_DATA_PATH")
        .unwrap_or_else(|_| "./data/bronze".to_string());
    let silver_path = env::var("SILVER_DATA_PATH")
        .unwrap_or_else(|_| "./data/silver".to_string());
    let gold_path = env::var("GOLD_DATA_PATH")
        .unwrap_or_else(|_| "./data/gold".to_string());

    // Bronze → Silver
    bronze_to_silver::run(&bronze_path, &silver_path)?;

    println!("\n🥈 Silver layer complete.");
    println!("   data/silver/sales_enriched.parquet");
    println!("   data/silver/controlling_enriched.parquet");
    println!("   data/silver/delivery_enriched.parquet");

    // Silver → Gold
    silver_to_gold::run(&silver_path, &gold_path)?;

    println!("\n🥇 Gold layer complete.");
    println!("   data/gold/margin_by_material.parquet");
    println!("   data/gold/margin_by_channel.parquet");
    println!("   data/gold/margin_by_sales_org.parquet");
    println!("   data/gold/margin_by_segment.parquet");
    println!("   data/gold/budget_variance.parquet");
    println!("   data/gold/delivery_performance.parquet");

    Ok(())
}