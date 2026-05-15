use anyhow::Result;
use arrow_array::{
    Array, FixedSizeListArray, Float32Array, RecordBatch, StringArray,
};
use arrow_schema::{DataType, Field, Schema};
use futures::TryStreamExt;
use lancedb::query::{ExecutableQuery, QueryBase};
use std::sync::Arc;

const TABLE_NAME: &str = "insights";
pub const EMBED_DIM: i32 = 768;

/// Write (or overwrite) the vector store from insight sentences and their embeddings.
pub async fn write(
    db_path: &str,
    insights: &[(String, String)], // (table_name, text)
    embeddings: Vec<Vec<f32>>,
) -> Result<()> {
    std::fs::create_dir_all(db_path)?;

    let db = lancedb::connect(db_path).execute().await?;

    // Drop old table if it exists — pass empty slice for root namespace
    let _ = db.drop_table(TABLE_NAME, &[]).await;

    let schema = build_schema();
    let batch = build_batch(&schema, insights, embeddings)?;

    // RecordBatch implements Scannable in lancedb 0.29
    db.create_table(TABLE_NAME, batch).execute().await?;

    tracing::info!("Vector store written to {} ({} rows)", db_path, insights.len());
    Ok(())
}

/// Search the vector store by cosine similarity, returning the top-`limit` insight texts.
/// Returns an empty Vec (not an error) when the store doesn't exist yet.
pub async fn search(db_path: &str, query_vec: Vec<f32>, limit: usize) -> Result<Vec<String>> {
    // Graceful fallback: store not built yet
    if !std::path::Path::new(db_path)
        .join(format!("{}.lance", TABLE_NAME))
        .exists()
    {
        tracing::warn!("Vector store not found at {} — skipping semantic search", db_path);
        return Ok(Vec::new());
    }

    let db = lancedb::connect(db_path).execute().await?;
    let table = db.open_table(TABLE_NAME).execute().await?;

    // Bind in a local so the borrow for execute() outlives the await
    let query = table
        .query()
        .nearest_to(query_vec)?
        .limit(limit);
    let stream = query.execute().await?;

    let batches: Vec<RecordBatch> = stream.try_collect().await?;

    let mut results = Vec::new();
    for batch in &batches {
        if let Some(col) = batch.column_by_name("text") {
            if let Some(strings) = col.as_any().downcast_ref::<StringArray>() {
                for i in 0..strings.len() {
                    if !strings.is_null(i) {
                        results.push(strings.value(i).to_string());
                    }
                }
            }
        }
    }

    Ok(results)
}

// ── schema / batch builders ───────────────────────────────────────────────────

fn build_schema() -> Arc<Schema> {
    Arc::new(Schema::new(vec![
        Field::new("text", DataType::Utf8, false),
        Field::new("table_name", DataType::Utf8, false),
        Field::new(
            "vector",
            DataType::FixedSizeList(
                Arc::new(Field::new("item", DataType::Float32, true)),
                EMBED_DIM,
            ),
            false,
        ),
    ]))
}

fn build_batch(
    schema: &Arc<Schema>,
    insights: &[(String, String)],
    embeddings: Vec<Vec<f32>>,
) -> Result<RecordBatch> {
    let texts  = StringArray::from_iter_values(insights.iter().map(|(_, t)| t.as_str()));
    let tables = StringArray::from_iter_values(insights.iter().map(|(tn, _)| tn.as_str()));

    let flat: Vec<f32> = embeddings.into_iter().flatten().collect();
    let values: Arc<dyn Array> = Arc::new(Float32Array::from(flat));
    let item_field = Arc::new(Field::new("item", DataType::Float32, true));
    let vectors = FixedSizeListArray::try_new(item_field, EMBED_DIM, values, None)?;

    Ok(RecordBatch::try_new(
        schema.clone(),
        vec![Arc::new(texts), Arc::new(tables), Arc::new(vectors)],
    )?)
}
