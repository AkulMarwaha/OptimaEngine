use anyhow::Result;
use polars::prelude::*;
use std::path::Path;

/// Returns Vec<(table_name, insight_sentence)> across all 6 Gold tables.
pub fn build_all(gold_path: &str) -> Result<Vec<(String, String)>> {
    let gold = Path::new(gold_path);
    let mut out = Vec::new();

    out.extend(margin_by_material(gold)?);
    out.extend(margin_by_channel(gold)?);
    out.extend(margin_by_sales_org(gold)?);
    out.extend(margin_by_segment(gold)?);
    out.extend(budget_variance(gold)?);
    out.extend(delivery_performance(gold)?);

    tracing::info!("Built {} insight sentences from Gold tables", out.len());
    Ok(out)
}

// ── helpers ──────────────────────────────────────────────────────────────────

fn read_parquet(path: &Path) -> Result<DataFrame> {
    let file = std::fs::File::open(path)?;
    Ok(ParquetReader::new(file).finish()?)
}

fn f64_val(df: &DataFrame, col: &str, row: usize) -> f64 {
    df.column(col)
        .ok()
        .and_then(|s| s.cast(&DataType::Float64).ok())
        .and_then(|s| s.f64().ok().and_then(|ca| ca.get(row)))
        .unwrap_or(0.0)
}

fn i64_val(df: &DataFrame, col: &str, row: usize) -> i64 {
    df.column(col)
        .ok()
        .and_then(|s| s.cast(&DataType::Int64).ok())
        .and_then(|s| s.i64().ok().and_then(|ca| ca.get(row)))
        .unwrap_or(0)
}

fn str_val(df: &DataFrame, col: &str, row: usize) -> String {
    df.column(col)
        .ok()
        .and_then(|s| s.cast(&DataType::String).ok())
        .and_then(|s| s.str().ok().and_then(|ca| ca.get(row).map(|v| v.to_string())))
        .unwrap_or_default()
}

// ── per-table insight builders ────────────────────────────────────────────────

fn margin_by_material(gold: &Path) -> Result<Vec<(String, String)>> {
    let df = read_parquet(&gold.join("margin_by_material.parquet"))?;
    let mut rows = Vec::new();
    for i in 0..df.height() {
        let material = str_val(&df, "ssi_material_id", i);
        let avg      = f64_val(&df, "avg_margin_pct", i);
        let squeeze  = i64_val(&df, "squeeze_count", i);
        let revenue  = f64_val(&df, "total_net_value_usd", i);
        let tag = if avg < 6.0 { " — critically below the 6% threshold" } else { "" };
        rows.push((
            "margin_by_material".to_string(),
            format!(
                "{} has an average margin of {:.1}% with {} squeeze orders \
                 and ${:.0} total revenue{}.",
                material, avg, squeeze, revenue, tag
            ),
        ));
    }
    Ok(rows)
}

fn margin_by_channel(gold: &Path) -> Result<Vec<(String, String)>> {
    let df = read_parquet(&gold.join("margin_by_channel.parquet"))?;
    let mut rows = Vec::new();
    for i in 0..df.height() {
        let channel = str_val(&df, "ssh_distribution_channel", i);
        let avg     = f64_val(&df, "avg_margin_pct", i);
        let orders  = i64_val(&df, "order_count", i);
        let revenue = f64_val(&df, "total_net_value_usd", i);
        rows.push((
            "margin_by_channel".to_string(),
            format!(
                "The {} channel has an average margin of {:.1}% across {} orders \
                 with ${:.0} total revenue.",
                channel, avg, orders, revenue
            ),
        ));
    }
    Ok(rows)
}

fn margin_by_sales_org(gold: &Path) -> Result<Vec<(String, String)>> {
    let df = read_parquet(&gold.join("margin_by_sales_org.parquet"))?;
    let mut rows = Vec::new();
    for i in 0..df.height() {
        let org    = str_val(&df, "ssh_sales_org", i);
        let avg    = f64_val(&df, "avg_margin_pct", i);
        let orders = i64_val(&df, "order_count", i);
        rows.push((
            "margin_by_sales_org".to_string(),
            format!(
                "Sales org {} has an average margin of {:.1}% across {} orders.",
                org, avg, orders
            ),
        ));
    }
    Ok(rows)
}

fn margin_by_segment(gold: &Path) -> Result<Vec<(String, String)>> {
    let df = read_parquet(&gold.join("margin_by_segment.parquet"))?;
    let mut rows = Vec::new();
    for i in 0..df.height() {
        let industry = str_val(&df, "scm_industry", i);
        let region   = str_val(&df, "scm_region_group", i);
        let avg      = f64_val(&df, "avg_margin_pct", i);
        let revenue  = f64_val(&df, "total_net_value_usd", i);
        rows.push((
            "margin_by_segment".to_string(),
            format!(
                "The {} segment in {} has an average margin of {:.1}% with ${:.0} revenue.",
                industry, region, avg, revenue
            ),
        ));
    }
    Ok(rows)
}

fn budget_variance(gold: &Path) -> Result<Vec<(String, String)>> {
    let df = read_parquet(&gold.join("budget_variance.parquet"))?;
    let mut rows = Vec::new();
    for i in 0..df.height() {
        let dept     = str_val(&df, "sco_department", i);
        let year     = str_val(&df, "sco_fiscal_year", i);
        let variance = f64_val(&df, "total_variance", i);
        let status   = if variance > 0.0 { "over budget" } else { "under budget" };
        let tag      = if variance > 100_000.0 { " — critical overspend" } else { "" };
        rows.push((
            "budget_variance".to_string(),
            format!(
                "The {} department is ${:.0} {} in fiscal year {}{}.",
                dept, variance.abs(), status, year, tag
            ),
        ));
    }
    Ok(rows)
}

fn delivery_performance(gold: &Path) -> Result<Vec<(String, String)>> {
    let df = read_parquet(&gold.join("delivery_performance.parquet"))?;
    let mut rows = Vec::new();
    for i in 0..df.height() {
        let route      = str_val(&df, "sdl_route", i);
        let days_late  = f64_val(&df, "avg_days_late", i);
        let deliveries = i64_val(&df, "delivery_count", i);
        let freight    = f64_val(&df, "total_freight_cost", i);
        let tag        = if days_late > 5.0 { " — critical delay" } else { "" };
        rows.push((
            "delivery_performance".to_string(),
            format!(
                "Route {} averages {:.0} days late across {} deliveries \
                 with ${:.0} total freight cost{}.",
                route, days_late, deliveries, freight, tag
            ),
        ));
    }
    Ok(rows)
}
