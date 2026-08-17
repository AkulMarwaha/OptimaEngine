use axum::{extract::{Multipart, State}, http::StatusCode, Json};
use polars::prelude::*;
use std::{io::Cursor, sync::Arc};

use crate::AppState;

type ApiResponse = Result<Json<serde_json::Value>, (StatusCode, String)>;

fn map_err(e: impl std::fmt::Display) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}

fn read_gold(gold_path: &str, filename: &str) -> Result<DataFrame, PolarsError> {
    let path = format!("{}/{}", gold_path, filename);
    LazyFrame::scan_parquet(&path, Default::default())?.collect()
}

fn df_to_json(df: &mut DataFrame) -> Result<serde_json::Value, String> {
    let mut buf = Cursor::new(Vec::new());
    JsonWriter::new(&mut buf)
        .with_json_format(JsonFormat::JsonLines)
        .finish(df)
        .map_err(|e| e.to_string())?;

    let raw = String::from_utf8(buf.into_inner()).map_err(|e| e.to_string())?;

    // Parse newline-delimited JSON into a Vec of objects
    let records: Vec<serde_json::Value> = raw
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(serde_json::Value::Array(records))
}

pub async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "optima-engine-semantic-layer",
        "version": "1.0"
    }))
}

pub async fn margin_by_material(State(state): State<Arc<AppState>>) -> ApiResponse {
    let mut df = read_gold(&state.gold_path, "margin_by_material.parquet").map_err(map_err)?;
    tracing::info!("margin_by_material: {} rows", df.height());
    let json = df_to_json(&mut df).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(json))
}

pub async fn margin_by_channel(State(state): State<Arc<AppState>>) -> ApiResponse {
    let mut df = read_gold(&state.gold_path, "margin_by_channel.parquet").map_err(map_err)?;
    tracing::info!("margin_by_channel: {} rows", df.height());
    let json = df_to_json(&mut df).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(json))
}

pub async fn margin_by_sales_org(State(state): State<Arc<AppState>>) -> ApiResponse {
    let mut df = read_gold(&state.gold_path, "margin_by_sales_org.parquet").map_err(map_err)?;
    tracing::info!("margin_by_sales_org: {} rows", df.height());
    let json = df_to_json(&mut df).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(json))
}

pub async fn margin_by_segment(State(state): State<Arc<AppState>>) -> ApiResponse {
    let mut df = read_gold(&state.gold_path, "margin_by_segment.parquet").map_err(map_err)?;
    tracing::info!("margin_by_segment: {} rows", df.height());
    let json = df_to_json(&mut df).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(json))
}

pub async fn budget_variance(State(state): State<Arc<AppState>>) -> ApiResponse {
    let mut df = read_gold(&state.gold_path, "budget_variance.parquet").map_err(map_err)?;
    tracing::info!("budget_variance: {} rows", df.height());
    let json = df_to_json(&mut df).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(json))
}

pub async fn delivery_performance(State(state): State<Arc<AppState>>) -> ApiResponse {
    let mut df = read_gold(&state.gold_path, "delivery_performance.parquet").map_err(map_err)?;
    tracing::info!("delivery_performance: {} rows", df.height());
    let json = df_to_json(&mut df).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(json))
}

pub async fn upload_erp_file(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?
    {
        if field.name() != Some("file") {
            continue;
        }

        let raw_name = field.file_name().unwrap_or("upload.csv").to_string();

        // Strip any directory components to prevent path traversal.
        let filename = std::path::Path::new(&raw_name)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("upload.csv")
            .to_string();

        let data = field
            .bytes()
            .await
            .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
        let byte_count = data.len();

        std::fs::create_dir_all(&state.bronze_path)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        let dest = std::path::Path::new(&state.bronze_path).join(&filename);
        std::fs::write(&dest, &data)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        tracing::info!("Uploaded {} ({} bytes) → {:?}", filename, byte_count, dest);

        return Ok(Json(serde_json::json!({
            "status": "ok",
            "filename": filename,
            "bytes": byte_count,
        })));
    }

    Err((StatusCode::BAD_REQUEST, "No file field found in upload".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        routing::post,
        Router,
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn upload_csv_lands_in_bronze() {
        let bronze_dir = std::env::temp_dir().join("optima_upload_test");
        std::fs::create_dir_all(&bronze_dir).unwrap();
        let bronze_path = bronze_dir.to_str().unwrap().to_string();

        let state = std::sync::Arc::new(crate::AppState {
            gold_path: "/tmp".to_string(),
            bronze_path: bronze_path.clone(),
            system_prompt: String::new(),
            ollama_url: "http://localhost:11434".to_string(),
            ollama_model: "test".to_string(),
        });

        let app = Router::new()
            .route("/ingest/upload", post(upload_erp_file))
            .with_state(state);

        let boundary = "testboundary9000";
        let csv = "order_id,customer_id,net_value\nO-001,C-001,9500.00\n";
        let body = format!(
            "--{b}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"test_upload.csv\"\r\nContent-Type: text/csv\r\n\r\n{c}\r\n--{b}--\r\n",
            b = boundary,
            c = csv,
        );

        let request = Request::builder()
            .method("POST")
            .uri("/ingest/upload")
            .header("content-type", format!("multipart/form-data; boundary={}", boundary))
            .body(Body::from(body))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let landed = bronze_dir.join("test_upload.csv");
        assert!(landed.exists(), "uploaded file should be present in bronze dir");
        let saved = std::fs::read_to_string(&landed).unwrap();
        assert!(saved.contains("order_id"), "CSV content should be preserved intact");

        std::fs::remove_dir_all(&bronze_dir).ok();
    }
}