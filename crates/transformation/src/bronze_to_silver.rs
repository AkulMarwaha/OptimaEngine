use anyhow::Context;
use polars::prelude::*;
use std::path::Path;
use tracing::info;

use crate::compute::{is_margin_squeeze, margin_pct, to_usd};
use crate::validate::{validate_silver_sales, validate_silver_controlling, validate_silver_delivery};

pub fn run(bronze_path: &str, silver_path: &str) -> anyhow::Result<()> {
    info!("Starting Bronze → Silver transformation");

    let bronze = Path::new(bronze_path);
    let silver = Path::new(silver_path);
    std::fs::create_dir_all(silver)?;

    // --- Read all Bronze CSVs ---
    let header_df = read_csv(&bronze.join("SAP_Sales_Header.csv"))?;
    let items_df = read_csv(&bronze.join("SAP_Sales_Items.csv"))?;

    let customer_df = read_csv(&bronze.join("SAP_Customer_Master.csv"))?
        .lazy()
        .group_by([col("scm_customer_id")])
        .agg([col("*").first()])
        .collect()
        .context("Failed to deduplicate Customer Master on scm_customer_id")?;

    let material_df = read_csv(&bronze.join("SAP_Material_Master.csv"))?
        .lazy()
        .group_by([col("smm_material_id")])
        .agg([col("*").first()])
        .collect()
        .context("Failed to deduplicate Material Master on smm_material_id")?;

    let controlling_df = read_csv(&bronze.join("SAP_Controlling.csv"))?;
    let delivery_df = read_csv(&bronze.join("SAP_Delivery.csv"))?;

    info!(
        "Bronze loaded — Header: {} rows, Items: {} rows, Customer: {} rows (deduped), Material: {} rows (deduped), Controlling: {} rows, Delivery: {} rows",
        header_df.height(), items_df.height(),
        customer_df.height(), material_df.height(),
        controlling_df.height(), delivery_df.height()
    );

    // --- Step 1: Inner join Sales Header + Items on order_id ---
    let sales = header_df
        .join(
            &items_df,
            ["ssh_order_id"],
            ["ssi_order_id"],
            JoinArgs::new(JoinType::Inner),
            None,
        )
        .context("Failed to inner join Header and Items on order_id")?;

    info!("After Header ⋈ Items (inner): {} rows", sales.height());

    // --- Step 2: Left join with Customer Master on customer_id ---
    let sales = sales
        .join(
            &customer_df,
            ["ssh_customer_id"],
            ["scm_customer_id"],
            JoinArgs::new(JoinType::Left),
            None,
        )
        .context("Failed to left join with Customer Master on customer_id")?;

    info!("After ⋈ Customer Master (left): {} rows", sales.height());

    // --- Step 3: Left join with Material Master on material_id ---
    let sales = sales
        .join(
            &material_df,
            ["ssi_material_id"],
            ["smm_material_id"],
            JoinArgs::new(JoinType::Left),
            None,
        )
        .context("Failed to left join with Material Master on material_id")?;

    info!("After ⋈ Material Master (left): {} rows", sales.height());

    // --- Step 4: Compute enrichment columns ---
    let netwr_ca = sales.column("ssi_net_value")?.f64()?;
    let estimated_cost_ca = sales.column("ssi_estimated_cost")?.f64()?;
    let waerk_ca = sales.column("ssh_currency")?.str()?;
    let matnr_ca = sales.column("ssi_material_id")?.str()?;

    let netwr_usd: Vec<f64> = netwr_ca
        .into_iter()
        .zip(waerk_ca.into_iter())
        .map(|(n, c)| to_usd(n.unwrap_or(0.0), c.unwrap_or("USD")))
        .collect();

    let cost_usd: Vec<f64> = estimated_cost_ca
        .into_iter()
        .zip(waerk_ca.into_iter())
        .map(|(c, cur)| to_usd(c.unwrap_or(0.0), cur.unwrap_or("USD")))
        .collect();

    let margins: Vec<f64> = netwr_usd
        .iter()
        .zip(cost_usd.iter())
        .map(|(n, c)| margin_pct(*n, *c))
        .collect();

    let squeeze_flags: Vec<bool> = margins
        .iter()
        .map(|m| is_margin_squeeze(*m))
        .collect();

    let mat01_squeeze: Vec<bool> = matnr_ca
        .into_iter()
        .zip(margins.iter())
        .map(|(m, margin)| m.unwrap_or("") == "MAT-01" && *margin < 6.0)
        .collect();

    // --- Step 5: Add enrichment columns ---
    let mut enriched = sales;
    enriched.with_column(Series::new("netwr_usd".into(), netwr_usd))?;
    enriched.with_column(Series::new("estimated_cost_usd".into(), cost_usd))?;
    enriched.with_column(Series::new("margin_pct".into(), margins))?;
    enriched.with_column(Series::new("is_margin_squeeze".into(), squeeze_flags))?;
    enriched.with_column(Series::new("is_mat01_squeeze".into(), mat01_squeeze))?;

    info!(
        "Sales enriched: {} rows, {} columns",
        enriched.height(),
        enriched.width()
    );

    // --- Step 6: Validate before writing ---
    // If any check fails, the pipeline halts here and nothing is written to Silver.
    validate_silver_sales(&enriched)
        .context("Sales Silver validation failed — pipeline halted, no files written")?;
    validate_silver_controlling(&controlling_df)
        .context("Controlling Silver validation failed — pipeline halted, no files written")?;
    validate_silver_delivery(&delivery_df)
        .context("Delivery Silver validation failed — pipeline halted, no files written")?;

    // --- Write Silver Parquet files ---
    // Only reached if all three validation blocks above pass.
    write_parquet(&mut enriched, &silver.join("sales_enriched.parquet"))?;
    info!("✅ sales_enriched.parquet written");

    write_parquet(
        &mut controlling_df.clone(),
        &silver.join("controlling_enriched.parquet"),
    )?;
    info!("✅ controlling_enriched.parquet written");

    write_parquet(
        &mut delivery_df.clone(),
        &silver.join("delivery_enriched.parquet"),
    )?;
    info!("✅ delivery_enriched.parquet written");

    Ok(())
}

fn read_csv(path: &Path) -> anyhow::Result<DataFrame> {
    CsvReadOptions::default()
        .with_has_header(true)
        .try_into_reader_with_file_path(Some(path.to_path_buf()))?
        .finish()
        .with_context(|| format!("Failed to read CSV: {:?}", path))
}

fn write_parquet(df: &mut DataFrame, path: &Path) -> anyhow::Result<()> {
    let mut file = std::fs::File::create(path)?;
    ParquetWriter::new(&mut file)
        .finish(df)
        .with_context(|| format!("Failed to write Parquet: {:?}", path))?;
    Ok(())
}