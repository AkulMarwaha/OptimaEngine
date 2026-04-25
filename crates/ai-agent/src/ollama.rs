use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct OllamaRequest {
    pub model: String,
    pub prompt: String,
    pub stream: bool,
    pub system: Option<String>,
}

#[derive(Deserialize)]
pub struct OllamaResponse {
    pub response: String,
}

pub struct OllamaClient {
    pub base_url: String,
    pub model: String,
    pub client: reqwest::Client,
}

impl OllamaClient {
    pub fn new(base_url: &str, model: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            model: model.to_string(),
            client: reqwest::Client::new(),
        }
    }

    pub async fn ask(&self, system: &str, prompt: &str) -> anyhow::Result<String> {
        let req = OllamaRequest {
            model: self.model.clone(),
            prompt: prompt.to_string(),
            stream: false,
            system: Some(system.to_string()),
        };

        let res = self.client
            .post(format!("{}/api/generate", self.base_url))
            .json(&req)
            .send()
            .await?
            .json::<OllamaResponse>()
            .await?;

        Ok(res.response.trim().to_string())
    }
}