use optima_core::config::AppConfig;
use reqwest::Client;
use std::path::PathBuf;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tracing::{info, error};

pub struct MockarooClient {
    client: Client,
    config: AppConfig,
}

impl MockarooClient {
    pub fn new(config: AppConfig) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }

    /// Fetches `count` rows using the Mockaroo schema ID and writes to
    /// /data/bronze/<schema_name>.csv
    pub async fn ingest_schema(
        &self,
        schema_id: &str,
        schema_name: &str,
        count: u32,
    ) -> anyhow::Result<PathBuf> {
        let url = format!(
            "{}/{}?count={}&key={}",
            self.config.mockaroo_base_url,
            schema_id,
            count,
            self.config.mockaroo_api_key
        );

        info!("→ Requesting {} rows from schema '{}' (id: {}) ...", count, schema_name, schema_id);

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("Mockaroo API error {}: {}", status, body);
            anyhow::bail!("Mockaroo returned HTTP {}: {}", status, body);
        }

        let output_dir = PathBuf::from(&self.config.bronze_data_path);
        fs::create_dir_all(&output_dir).await?;

        let output_path = output_dir.join(format!("{}.csv", schema_name));
        let bytes = response.bytes().await?;

        let mut file = fs::File::create(&output_path).await?;
        file.write_all(&bytes).await?;
        file.flush().await?;

        info!("✓ Wrote {} bytes → {:?}", bytes.len(), output_path);

        Ok(output_path)
    }
}