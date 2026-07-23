use anyhow::Result;
use polars::prelude::*;
use std::io::Cursor;

pub fn load_gold_context(gold_path: &str) -> Result<String> {
    let mut ctx = String::new();

    append_section(&mut ctx, gold_path, "margin_by_material.parquet",   "MARGIN BY MATERIAL")?;
    append_section(&mut ctx, gold_path, "margin_by_channel.parquet",    "MARGIN BY CHANNEL")?;
    append_section(&mut ctx, gold_path, "budget_variance.parquet",      "BUDGET VARIANCE")?;
    append_section(&mut ctx, gold_path, "delivery_performance.parquet", "DELIVERY PERFORMANCE")?;
    append_section(&mut ctx, gold_path, "margin_by_sales_org.parquet",  "MARGIN BY SALES ORG")?;
    append_section(&mut ctx, gold_path, "margin_by_segment.parquet",    "MARGIN BY SEGMENT")?;

    Ok(ctx)
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
    for prefix in &["ssi_", "ssh_", "scm_", "sco_", "sdl_"] {
        if let Some(stripped) = col.strip_prefix(prefix) {
            return stripped;
        }
    }
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
