use anyhow::Context;
use polars::prelude::*;
use std::path::Path;
use tracing::info;

use crate::compute::{is_margin_squeeze, margin_pct, to_usd};

/// Run the Bronze → Silver transformation.
///
/// Reads SAP_Sales_Header.csv and SAP_Sales_Items.csv from the bronze path,
/// performs an inner join on `vbeln`, enriches with margin and currency columns,
/// and writes the result to data/silver/sales_enriched.parquet.
pub fn run(bronze_path: &str, silver_path: &str) -> anyhow::Result<()> {
    info!("Starting Bronze → Silver transformation");

    // --- Read Bronze CSVs ---
    let header_path = Path::new(bronze_path).join("SAP_Sales_Header.csv");
    let items_path = Path::new(bronze_path).join("SAP_Sales_Items.csv");

    info!("Reading SAP_Sales_Header from {:?}", header_path);
    let header_df = CsvReadOptions::default()
        .with_has_header(true)
        .try_into_reader_with_file_path(Some(header_path))?
        .finish()
        .context("Failed to read SAP_Sales_Header.csv")?;

    info!("Reading SAP_Sales_Items from {:?}", items_path);
    let items_df = CsvReadOptions::default()
        .with_has_header(true)
        .try_into_reader_with_file_path(Some(items_path))?
        .finish()
        .context("Failed to read SAP_Sales_Items.csv")?;

    info!(
        "Bronze loaded — Header: {} rows, Items: {} rows",
        header_df.height(),
        items_df.height()
    );

    // --- Inner Join on vbeln ---
    let joined = header_df
        .join(
            &items_df,
            ["vbeln"],
            ["vbeln"],
            JoinArgs::new(JoinType::Inner),
            None,
        )
        .context("Failed to join Header and Items on vbeln")?;

    info!("Joined DataFrame: {} rows", joined.height());

    // --- Enrich: Currency Normalization + Margin Calculation ---
    // Extract columns needed for enrichment
    let netwr_ca = joined.column("netwr")?.f64()?;
    let estimated_cost_ca = joined.column("estimated_cost")?.f64()?;
    let waerk_ca = joined.column("waerk")?.str()?;
    let matnr_ca = joined.column("matnr")?.str()?;

    // Compute netwr_usd — currency normalized net value
    let netwr_usd: Vec<f64> = netwr_ca
        .into_iter()
        .zip(waerk_ca.into_iter())
        .map(|(netwr, currency)| {
            to_usd(netwr.unwrap_or(0.0), currency.unwrap_or("USD"))
        })
        .collect();

    // Compute estimated_cost_usd
    let cost_usd: Vec<f64> = estimated_cost_ca
        .into_iter()
        .zip(waerk_ca.into_iter())
        .map(|(cost, currency)| {
            to_usd(cost.unwrap_or(0.0), currency.unwrap_or("USD"))
        })
        .collect();

    // Compute margin_pct column
    let margins: Vec<f64> = netwr_usd
        .iter()
        .zip(cost_usd.iter())
        .map(|(netwr, cost)| margin_pct(*netwr, *cost))
        .collect();

    // Compute margin_squeeze flag
    let squeeze_flags: Vec<bool> = margins
        .iter()
        .map(|m| is_margin_squeeze(*m))
        .collect();

    // Compute margin_squeeze_material flag (MAT-01 specific)
    let mat01_squeeze: Vec<bool> = matnr_ca
        .into_iter()
        .zip(margins.iter())
        .map(|(matnr, margin)| matnr.unwrap_or("") == "MAT-01" && margin < &6.0)
        .collect();

    // --- Build enriched DataFrame ---
    let mut enriched = joined;
    enriched.with_column(Series::new("netwr_usd".into(), netwr_usd))?;
    enriched.with_column(Series::new("estimated_cost_usd".into(), cost_usd))?;
    enriched.with_column(Series::new("margin_pct".into(), margins))?;
    enriched.with_column(Series::new("is_margin_squeeze".into(), squeeze_flags))?;
    enriched.with_column(Series::new("is_mat01_squeeze".into(), mat01_squeeze))?;

    info!(
        "Enriched DataFrame: {} rows, {} columns",
        enriched.height(),
        enriched.width()
    );

    // --- Write Silver Parquet ---
    let silver_dir = Path::new(silver_path);
    std::fs::create_dir_all(silver_dir)?;

    let output_path = silver_dir.join("sales_enriched.parquet");
    let mut file = std::fs::File::create(&output_path)?;

    ParquetWriter::new(&mut file)
        .finish(&mut enriched)
        .context("Failed to write silver Parquet file")?;

    info!("✅ Silver layer written → {:?}", output_path);
    Ok(())
}