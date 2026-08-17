use anyhow::Result;
use polars::prelude::*;
use std::io::Cursor;

pub fn load_gold_context(gold_path: &str) -> Result<String> {
    let mut ctx = String::new();

    append_section(&mut ctx, gold_path, "margin_by_material.parquet",   "MARGIN BY MATERIAL")?;
    append_section(&mut ctx, gold_path, "margin_by_channel.parquet",    "MARGIN BY CHANNEL")?;
    append_budget_section(&mut ctx, gold_path)?;
    append_section(&mut ctx, gold_path, "delivery_performance.parquet", "DELIVERY PERFORMANCE")?;
    append_section(&mut ctx, gold_path, "margin_by_sales_org.parquet",  "MARGIN BY SALES ORG")?;
    append_section(&mut ctx, gold_path, "margin_by_segment.parquet",    "MARGIN BY SEGMENT")?;

    Ok(ctx)
}

/// Budget section with pre-computed OVER / UNDER labels so the model doesn't need to infer signs.
fn append_budget_section(ctx: &mut String, gold_path: &str) -> Result<()> {
    let path = format!("{}/budget_variance.parquet", gold_path);
    let mut df = LazyFrame::scan_parquet(&path, Default::default())?.collect()?;

    let mut buf = Cursor::new(Vec::new());
    JsonWriter::new(&mut buf)
        .with_json_format(JsonFormat::JsonLines)
        .finish(&mut df)?;
    let raw = String::from_utf8(buf.into_inner())?;

    struct BudgetRow {
        dept: String,
        year: i64,
        budget: f64,
        actual: f64,
        variance: f64,
    }

    let mut rows: Vec<BudgetRow> = Vec::new();
    for line in raw.lines().filter(|l| !l.trim().is_empty()) {
        if let Ok(serde_json::Value::Object(map)) = serde_json::from_str::<serde_json::Value>(line) {
            let dept     = map.get("department").and_then(|v| v.as_str()).unwrap_or("?").to_string();
            let year     = map.get("fiscal_year").and_then(|v| v.as_i64()).unwrap_or(0);
            let budget   = map.get("total_budget").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let actual   = map.get("total_actual_cost").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let variance = map.get("total_variance").and_then(|v| v.as_f64()).unwrap_or(0.0);
            rows.push(BudgetRow { dept, year, budget, actual, variance });
        }
    }

    // Sort: over-budget first (descending by overrun), then under-budget (ascending by savings)
    rows.sort_by(|a, b| b.variance.partial_cmp(&a.variance).unwrap_or(std::cmp::Ordering::Equal));

    ctx.push_str("=== BUDGET VARIANCE ===\n");

    let over: Vec<&BudgetRow> = rows.iter().filter(|r| r.variance > 0.0).collect();
    let under: Vec<&BudgetRow> = rows.iter().filter(|r| r.variance <= 0.0).collect();

    if !over.is_empty() {
        ctx.push_str("OVER BUDGET (actual > planned — cost overrun, action required):\n");
        for r in &over {
            ctx.push_str(&format!(
                "  {} {}: budget=${:.2} | actual=${:.2} | overrun=${:.2}\n",
                r.dept, r.year, r.budget, r.actual, r.variance
            ));
        }
        ctx.push('\n');
    }

    if !under.is_empty() {
        ctx.push_str("UNDER BUDGET (actual < planned — savings, on track):\n");
        for r in &under {
            ctx.push_str(&format!(
                "  {} {}: budget=${:.2} | actual=${:.2} | savings=${:.2}\n",
                r.dept, r.year, r.budget, r.actual, r.variance.abs()
            ));
        }
        ctx.push('\n');
    }

    Ok(())
}

fn append_section(ctx: &mut String, gold_path: &str, filename: &str, title: &str) -> Result<()> {
    let path = format!("{}/{}", gold_path, filename);
    let mut df = LazyFrame::scan_parquet(&path, Default::default())?.collect()?;

    ctx.push_str(&format!("=== {} ===\n", title));

    let mut buf = Cursor::new(Vec::new());
    JsonWriter::new(&mut buf)
        .with_json_format(JsonFormat::JsonLines)
        .finish(&mut df)?;

    let raw = String::from_utf8(buf.into_inner())?;

    for line in raw.lines().filter(|l| !l.trim().is_empty()) {
        if let Ok(serde_json::Value::Object(map)) = serde_json::from_str::<serde_json::Value>(line) {
            let parts: Vec<String> = map.iter()
                .map(|(k, v)| format!("{}: {}", strip_prefix(k), fmt_value(v)))
                .collect();
            ctx.push_str(&parts.join(" | "));
            ctx.push('\n');
        }
    }

    ctx.push('\n');
    Ok(())
}

fn strip_prefix(col: &str) -> &str {
    col
}

fn fmt_value(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::Number(n) => {
            if let Some(f) = n.as_f64() {
                if f.fract() == 0.0 {
                    format!("{}", f as i64)
                } else {
                    format!("{:.2}", f)
                }
            } else {
                n.to_string()
            }
        }
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Null => "N/A".to_string(),
        other => other.to_string(),
    }
}
