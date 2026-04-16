use dotenvy::dotenv;
use std::env;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub mockaroo_api_key: String,
    pub mockaroo_base_url: String,
    pub bronze_data_path: String,
    pub schema_sales_header: String,
    pub schema_sales_items: String,
}

impl AppConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        dotenv().ok();

        Ok(Self {
            mockaroo_api_key: env::var("MOCKAROO_API_KEY")
                .map_err(|_| anyhow::anyhow!("MOCKAROO_API_KEY not set"))?,
            mockaroo_base_url: env::var("MOCKAROO_BASE_URL")
                .unwrap_or_else(|_| "https://api.mockaroo.com/api".to_string()),
            bronze_data_path: env::var("BRONZE_DATA_PATH")
                .unwrap_or_else(|_| "./data/bronze".to_string()),
            schema_sales_header: env::var("MOCKAROO_SCHEMA_SALES_HEADER")
                .map_err(|_| anyhow::anyhow!("MOCKAROO_SCHEMA_SALES_HEADER not set"))?,
            schema_sales_items: env::var("MOCKAROO_SCHEMA_SALES_ITEMS")
                .map_err(|_| anyhow::anyhow!("MOCKAROO_SCHEMA_SALES_ITEMS not set"))?,
        })
    }
}