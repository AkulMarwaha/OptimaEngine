use anyhow::Context;
use polars::prelude::*;
use std::path::Path;
use tracing::info;

pub fn run(silver_path: &str, gold_path: &str) -> anyhow::Result<()> {
    info!("Starting Silver → Gold transformation");

    let silver = Path::new(silver_path);
    let gold = Path::new(gold_path);
    std::fs::create_dir_all(gold)?;

    // --- Read Silver Parquet files ---
    let sales = read_parquet(&silver.join("sales_enriched.parquet"))?;
    let controlling = read_parquet(&silver.join("controlling_enriched.parquet"))?;
    let delivery = read_parquet(&silver.join("delivery_enriched.parquet"))?;

    info!(
        "Silver loaded — Sales: {} rows, Controlling: {} rows, Delivery: {} rows",
        sales.height(), controlling.height(), delivery.height()
    );

    // --- Gold Table 1: Margin by Material ---
    let margin_by_material = sales
        .clone()
        .lazy()
        .group_by([col("material_id")])
        .agg([
            col("margin_pct").mean().alias("avg_margin_pct"),
            col("margin_pct").min().alias("min_margin_pct"),
            col("margin_pct").max().alias("max_margin_pct"),
            col("net_value").sum().alias("total_net_value_usd"),
            col("estimated_cost").sum().alias("total_cost_usd"),
            col("is_margin_squeeze").sum().alias("squeeze_count"),
            col("order_id").count().alias("order_count"),
        ])
        .sort(["avg_margin_pct"], SortMultipleOptions::default())
        .collect()
        .context("Failed to compute margin_by_material")?;

    info!("margin_by_material: {} rows", margin_by_material.height());
    write_parquet(&mut margin_by_material.clone(), &gold.join("margin_by_material.parquet"))?;
    info!("✅ margin_by_material.parquet written");

    // --- Gold Table 2: Margin by Distribution Channel ---
    let margin_by_channel = sales
        .clone()
        .lazy()
        .group_by([col("distribution_channel")])
        .agg([
            col("margin_pct").mean().alias("avg_margin_pct"),
            col("margin_pct").min().alias("min_margin_pct"),
            col("margin_pct").max().alias("max_margin_pct"),
            col("net_value").sum().alias("total_net_value_usd"),
            col("is_margin_squeeze").sum().alias("squeeze_count"),
            col("order_id").count().alias("order_count"),
        ])
        .sort(["avg_margin_pct"], SortMultipleOptions::default())
        .collect()
        .context("Failed to compute margin_by_channel")?;

    info!("margin_by_channel: {} rows", margin_by_channel.height());
    write_parquet(&mut margin_by_channel.clone(), &gold.join("margin_by_channel.parquet"))?;
    info!("✅ margin_by_channel.parquet written");

    // --- Gold Table 3: Margin by Sales Organization ---
    let margin_by_sales_org = sales
        .clone()
        .lazy()
        .group_by([col("sales_org")])
        .agg([
            col("margin_pct").mean().alias("avg_margin_pct"),
            col("margin_pct").min().alias("min_margin_pct"),
            col("margin_pct").max().alias("max_margin_pct"),
            col("net_value").sum().alias("total_net_value_usd"),
            col("is_margin_squeeze").sum().alias("squeeze_count"),
            col("order_id").count().alias("order_count"),
        ])
        .sort(["avg_margin_pct"], SortMultipleOptions::default())
        .collect()
        .context("Failed to compute margin_by_sales_org")?;

    info!("margin_by_sales_org: {} rows", margin_by_sales_org.height());
    write_parquet(&mut margin_by_sales_org.clone(), &gold.join("margin_by_sales_org.parquet"))?;
    info!("✅ margin_by_sales_org.parquet written");

    // --- Gold Table 4: Margin by Customer Segment ---
    let margin_by_segment = sales
        .clone()
        .lazy()
        .group_by([col("industry"), col("region_group")])
        .agg([
            col("margin_pct").mean().alias("avg_margin_pct"),
            col("net_value").sum().alias("total_net_value_usd"),
            col("is_margin_squeeze").sum().alias("squeeze_count"),
            col("order_id").count().alias("order_count"),
        ])
        .sort(["avg_margin_pct"], SortMultipleOptions::default())
        .collect()
        .context("Failed to compute margin_by_segment")?;

    info!("margin_by_segment: {} rows", margin_by_segment.height());
    write_parquet(&mut margin_by_segment.clone(), &gold.join("margin_by_segment.parquet"))?;
    info!("✅ margin_by_segment.parquet written");

    // --- Gold Table 5: Budget Variance by Department ---
    let budget_variance = controlling
        .clone()
        .lazy()
        .group_by([col("department"), col("fiscal_year")])
        .agg([
            col("actual_cost").sum().alias("total_actual_cost"),
            col("budget_amount").sum().alias("total_budget"),
            col("budget_variance").sum().alias("total_variance"),
            col("budget_variance").mean().alias("avg_variance"),
            col("order_id").count().alias("record_count"),
        ])
        .sort(["total_variance"], SortMultipleOptions::default())
        .collect()
        .context("Failed to compute budget_variance")?;

    info!("budget_variance: {} rows", budget_variance.height());
    write_parquet(&mut budget_variance.clone(), &gold.join("budget_variance.parquet"))?;
    info!("✅ budget_variance.parquet written");

    // --- Gold Table 6: Delivery Performance by Route ---
    let delivery_performance = delivery
        .clone()
        .lazy()
        .group_by([col("route"), col("transport_type")])
        .agg([
            col("days_late").mean().alias("avg_days_late"),
            col("freight_cost_usd").mean().alias("avg_freight_cost"),
            col("freight_cost_usd").sum().alias("total_freight_cost"),
            col("delivery_id").count().alias("delivery_count"),
        ])
        .sort(["avg_days_late"], SortMultipleOptions::default())
        .collect()
        .context("Failed to compute delivery_performance")?;

    info!("delivery_performance: {} rows", delivery_performance.height());
    write_parquet(&mut delivery_performance.clone(), &gold.join("delivery_performance.parquet"))?;
    info!("✅ delivery_performance.parquet written");

    Ok(())
}

fn read_parquet(path: &Path) -> anyhow::Result<DataFrame> {
    let file = std::fs::File::open(path)
        .with_context(|| format!("Failed to open Parquet: {:?}", path))?;
    ParquetReader::new(file)
        .finish()
        .with_context(|| format!("Failed to read Parquet: {:?}", path))
}

fn write_parquet(df: &mut DataFrame, path: &Path) -> anyhow::Result<()> {
    let mut file = std::fs::File::create(path)?;
    ParquetWriter::new(&mut file)
        .finish(df)
        .with_context(|| format!("Failed to write Parquet: {:?}", path))?;
    Ok(())
}