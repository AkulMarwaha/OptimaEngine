use axum::{extract::{Multipart, State}, http::StatusCode, Json};
use calamine::{open_workbook_from_rs, Reader, Xls, Xlsx};
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

/// Extract the header row from a CSV or Excel file supplied as raw bytes.
fn extract_headers(filename: &str, data: &[u8]) -> Result<Vec<String>, String> {
    let ext = std::path::Path::new(filename)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "xlsx" => extract_headers_xlsx(data),
        "xls"  => extract_headers_xls(data),
        _      => Ok(extract_headers_csv(data)),
    }
}

fn extract_headers_csv(data: &[u8]) -> Vec<String> {
    // Strip UTF-8 BOM (present in many Excel-exported CSVs).
    let data = data.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(data);
    let text = std::str::from_utf8(data).unwrap_or("");
    text.lines()
        .next()
        .unwrap_or("")
        .split(',')
        .map(|h| h.trim().trim_matches('"').to_string())
        .filter(|h| !h.is_empty())
        .collect()
}

fn extract_headers_xlsx(data: &[u8]) -> Result<Vec<String>, String> {
    let cursor = Cursor::new(data.to_vec());
    let mut wb: Xlsx<_> = open_workbook_from_rs(cursor).map_err(|e: calamine::XlsxError| e.to_string())?;
    let sheets = wb.sheet_names();
    let sheet = sheets.first().cloned().ok_or("No sheets found in workbook")?;
    let range = wb.worksheet_range(&sheet).map_err(|e| e.to_string())?;
    Ok(range
        .rows()
        .next()
        .unwrap_or_default()
        .iter()
        .map(|c| c.to_string())
        .collect())
}

fn extract_headers_xls(data: &[u8]) -> Result<Vec<String>, String> {
    let cursor = Cursor::new(data.to_vec());
    let mut wb: Xls<_> = open_workbook_from_rs(cursor).map_err(|e: calamine::XlsError| e.to_string())?;
    let sheets = wb.sheet_names();
    let sheet = sheets.first().cloned().ok_or("No sheets found in workbook")?;
    let range = wb.worksheet_range(&sheet).map_err(|e| e.to_string())?;
    Ok(range
        .rows()
        .next()
        .unwrap_or_default()
        .iter()
        .map(|c| c.to_string())
        .collect())
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

        // Save raw bytes to Bronze.
        std::fs::create_dir_all(&state.bronze_path)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        let dest = std::path::Path::new(&state.bronze_path).join(&filename);
        std::fs::write(&dest, &data)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        // Extract headers and propose a field mapping.
        let headers = extract_headers(&filename, &data)
            .map_err(|e| (StatusCode::UNPROCESSABLE_ENTITY, e))?;
        let header_refs: Vec<&str> = headers.iter().map(String::as_str).collect();
        let matches = ingestion::field_matcher::match_headers(&header_refs);

        let mapping: Vec<serde_json::Value> = headers
            .iter()
            .zip(matches.iter())
            .map(|(header, m)| {
                let tier = match m.tier {
                    ingestion::field_matcher::MatchTier::Known     => "Known",
                    ingestion::field_matcher::MatchTier::Heuristic => "Heuristic",
                    ingestion::field_matcher::MatchTier::NoMatch   => "NoMatch",
                };
                serde_json::json!({
                    "header":    header,
                    "canonical": m.canonical,
                    "tier":      tier,
                })
            })
            .collect();

        tracing::info!(
            "Uploaded {} ({} bytes) → {:?} — {} headers, {} matched",
            filename, byte_count, dest,
            headers.len(),
            matches.iter().filter(|m| m.canonical.is_some()).count(),
        );

        return Ok(Json(serde_json::json!({
            "status":   "ok",
            "filename": filename,
            "bytes":    byte_count,
            "mapping":  mapping,
        })));
    }

    Err((StatusCode::BAD_REQUEST, "No file field found in upload".to_string()))
}

#[derive(serde::Deserialize)]
pub struct MappingEntry {
    pub header: String,
    pub canonical: Option<String>,
}

pub async fn confirm_mapping(
    State(state): State<Arc<AppState>>,
    Json(entries): Json<Vec<MappingEntry>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    std::fs::create_dir_all(&state.config_path)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let path = std::path::Path::new(&state.config_path).join("field_mapping.json");

    let json_body: Vec<serde_json::Value> = entries
        .iter()
        .map(|e| serde_json::json!({ "header": e.header, "canonical": e.canonical }))
        .collect();

    let pretty = serde_json::to_string_pretty(&json_body)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    std::fs::write(&path, &pretty)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    tracing::info!("Field mapping saved to {:?} ({} entries)", path, entries.len());

    Ok(Json(serde_json::json!({
        "status":  "ok",
        "path":    path.to_string_lossy(),
        "entries": entries.len(),
    })))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{to_bytes, Body},
        http::{Request, StatusCode},
        routing::post,
        Router,
    };
    use tower::ServiceExt;

    fn make_multipart(boundary: &str, filename: &str, csv: &str) -> String {
        format!(
            "--{b}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"{f}\"\r\nContent-Type: text/csv\r\n\r\n{c}\r\n--{b}--\r\n",
            b = boundary, f = filename, c = csv,
        )
    }

    fn test_state(bronze_path: String, config_path: String) -> std::sync::Arc<crate::AppState> {
        std::sync::Arc::new(crate::AppState {
            gold_path: "/tmp".to_string(),
            bronze_path,
            config_path,
            system_prompt: String::new(),
            ollama_url: "http://localhost:11434".to_string(),
            ollama_model: "test".to_string(),
        })
    }

    #[tokio::test]
    async fn upload_csv_lands_in_bronze_and_returns_mapping() {
        let bronze_dir = std::env::temp_dir().join("optima_upload_test");
        std::fs::create_dir_all(&bronze_dir).unwrap();

        let app = Router::new()
            .route("/ingest/upload", post(upload_erp_file))
            .with_state(test_state(
                bronze_dir.to_str().unwrap().to_string(),
                "/tmp".to_string(),
            ));

        let boundary = "testboundary9000";
        let csv = "VBELN,KUNNR,MATNR,UNKNOWN_COL\n001,C001,M001,foo\n";
        let body = make_multipart(boundary, "sap_export.csv", csv);

        let request = Request::builder()
            .method("POST")
            .uri("/ingest/upload")
            .header("content-type", format!("multipart/form-data; boundary={}", boundary))
            .body(Body::from(body))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        let status = response.status();
        let body_bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(status, StatusCode::OK);

        // File landed in Bronze with content intact.
        let landed = bronze_dir.join("sap_export.csv");
        assert!(landed.exists(), "file should be in bronze dir");
        assert!(std::fs::read_to_string(&landed).unwrap().contains("VBELN"));

        // Response contains the field mapping.
        let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        let mapping = json["mapping"].as_array().expect("mapping should be an array");
        assert_eq!(mapping.len(), 4);

        assert_eq!(mapping[0]["header"],    "VBELN");
        assert_eq!(mapping[0]["canonical"], "order_id");
        assert_eq!(mapping[0]["tier"],      "Known");

        assert_eq!(mapping[1]["header"],    "KUNNR");
        assert_eq!(mapping[1]["canonical"], "customer_id");
        assert_eq!(mapping[1]["tier"],      "Known");

        assert_eq!(mapping[2]["header"],    "MATNR");
        assert_eq!(mapping[2]["canonical"], "material_id");
        assert_eq!(mapping[2]["tier"],      "Known");

        assert_eq!(mapping[3]["header"],    "UNKNOWN_COL");
        assert_eq!(mapping[3]["canonical"], serde_json::Value::Null);
        assert_eq!(mapping[3]["tier"],      "NoMatch");

        std::fs::remove_dir_all(&bronze_dir).ok();
    }

    #[tokio::test]
    async fn confirm_mapping_writes_config_file() {
        let config_dir = std::env::temp_dir().join("optima_confirm_test");
        std::fs::create_dir_all(&config_dir).unwrap();

        let app = Router::new()
            .route("/ingest/confirm-mapping", post(confirm_mapping))
            .with_state(test_state(
                "/tmp".to_string(),
                config_dir.to_str().unwrap().to_string(),
            ));

        let payload = serde_json::json!([
            { "header": "VBELN",         "canonical": "order_id" },
            { "header": "KUNNR",         "canonical": "customer_id" },
            { "header": "UNKNOWN_FIELD", "canonical": null },
        ]);

        let request = Request::builder()
            .method("POST")
            .uri("/ingest/confirm-mapping")
            .header("content-type", "application/json")
            .body(Body::from(payload.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        let status = response.status();
        let body_bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(status, StatusCode::OK);

        let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(json["status"],  "ok");
        assert_eq!(json["entries"], 3);

        // Config file exists and has the right contents.
        let mapping_file = config_dir.join("field_mapping.json");
        assert!(mapping_file.exists(), "field_mapping.json should exist");
        let content = std::fs::read_to_string(&mapping_file).unwrap();
        let written: serde_json::Value = serde_json::from_str(&content).unwrap();
        let arr = written.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0]["header"],    "VBELN");
        assert_eq!(arr[0]["canonical"], "order_id");
        assert_eq!(arr[1]["header"],    "KUNNR");
        assert_eq!(arr[1]["canonical"], "customer_id");
        assert_eq!(arr[2]["header"],    "UNKNOWN_FIELD");
        assert_eq!(arr[2]["canonical"], serde_json::Value::Null);

        std::fs::remove_dir_all(&config_dir).ok();
    }
}