mod embedder;
mod insight_builder;
mod store;

use embedder::Embedder;

/// Build (or rebuild) the vector store from the Gold Parquet tables.
///
/// Called at the end of the transformation pipeline after Gold is written.
/// `gold_path`   – directory containing the 6 Gold .parquet files
/// `vector_path` – where LanceDB stores its data (VECTOR_DATA_PATH, default ./data/vector)
/// `ollama_url`  – base URL for Ollama (AI_AGENT_BASE_URL, default http://localhost:11434)
pub async fn build_store(
    gold_path: &str,
    vector_path: &str,
    ollama_url: &str,
) -> anyhow::Result<()> {
    tracing::info!("Building vector store from Gold tables at {}", gold_path);

    let insights = insight_builder::build_all(gold_path)?;
    tracing::info!("Embedding {} insight sentences with nomic-embed-text…", insights.len());

    let embedder = Embedder::new(ollama_url);
    let mut embeddings: Vec<Vec<f32>> = Vec::with_capacity(insights.len());

    for (i, (_, text)) in insights.iter().enumerate() {
        let vec = embedder.embed(text).await.map_err(|e| {
            anyhow::anyhow!("Failed to embed insight #{}: {} — is Ollama running with nomic-embed-text? ({})", i, text, e)
        })?;
        embeddings.push(vec);
    }

    store::write(vector_path, &insights, embeddings).await?;
    println!("\n🔮 Vector store complete — {} sentences embedded at {}", insights.len(), vector_path);
    Ok(())
}

/// Search the vector store with a plain-English query.
///
/// Returns the top-`limit` most semantically relevant insight sentences.
/// Returns an empty Vec (no error) when the store hasn't been built yet.
pub async fn search_store(
    query: &str,
    vector_path: &str,
    ollama_url: &str,
    limit: usize,
) -> anyhow::Result<Vec<String>> {
    let embedder = Embedder::new(ollama_url);
    let query_vec = match embedder.embed(query).await {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!("Could not embed search query ({}), skipping vector search", e);
            return Ok(Vec::new());
        }
    };

    store::search(vector_path, query_vec, limit).await
}
