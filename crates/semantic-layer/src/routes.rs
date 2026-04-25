use polars::prelude::*;
use axum::{extract::State, http::StatusCode, Json};
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