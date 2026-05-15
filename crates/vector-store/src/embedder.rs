use anyhow::Result;
use serde::{Deserialize, Serialize};

pub const EMBED_DIM: usize = 768;
const EMBED_MODEL: &str = "nomic-embed-text";

#[derive(Serialize)]
struct EmbedRequest<'a> {
    model: &'a str,
    prompt: &'a str,
}

#[derive(Deserialize)]
struct EmbedResponse {
    embedding: Vec<f32>,
}

pub struct Embedder {
    client: reqwest::Client,
    base_url: String,
}

impl Embedder {
    pub fn new(base_url: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.to_string(),
        }
    }

    pub async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let req = EmbedRequest {
            model: EMBED_MODEL,
            prompt: text,
        };

        let res = self
            .client
            .post(format!("{}/api/embeddings", self.base_url))
            .json(&req)
            .send()
            .await?
            .json::<EmbedResponse>()
            .await?;

        anyhow::ensure!(
            res.embedding.len() == EMBED_DIM,
            "Expected {} dimensions from {}, got {}",
            EMBED_DIM,
            EMBED_MODEL,
            res.embedding.len()
        );

        Ok(res.embedding)
    }
}
