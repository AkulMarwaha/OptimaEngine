use anyhow::Context;
use polars::prelude::*;
use std::collections::HashSet;
use std::path::Path;
use tracing::info;

use crate::compute::{is_margin_squeeze, margin_pct, to_usd};
use crate::validate::{validate_silver_sales, validate_silver_controlling, validate_silver_delivery};

const REQUIRED_TYPES: &[&str] = &[
    "sales_header", "sales_items", "customer_master", "controlling", "delivery",
];

/// Mapping-driven Bronze → Silver pipeline.
/// Returns `true` if the pipeline ran, `false` if the config file is absent
/// or a required data-type entry is missing (caller should produce no Gold output).
pub fn run_from_mapping(
    config_path: &str,
    bronze_path: &str,
    silver_path: &str,
) -> anyhow::Result<bool> {
    let mapping_file = Path::new(config_path).join("field_mapping.json");

    if !mapping_file.exists() {
        info!("No field_mapping.json at {:?} — pipeline skipped", mapping_file);
        return Ok(false);
    }

    let raw = std::fs::read_to_string(&mapping_file)
        .context("Failed to read field_mapping.json")?;
    let root: serde_json::Map<String, serde_json::Value> =
        serde_json::from_str(&raw).context("field_mapping.json is not a valid JSON object")?;

    for dt in REQUIRED_TYPES {
        if !root.contains_key(*dt) {
            info!("field_mapping.json missing required type '{}' — pipeline skipped", dt);
            return Ok(false);
        }
    }

    let bronze = Path::new(bronze_path);
    let silver = Path::new(silver_path);
    std::fs::create_dir_all(silver)?;

    let header_df     = load_and_rename(&root, "sales_header",   bronze)?;
    let items_df      = load_and_rename(&root, "sales_items",    bronze)?;
    let customer_df   = load_and_rename(&root, "customer_master", bronze)?
        .lazy()
        .group_by([col("customer_id")])
        .agg([col("*").first()])
        .collect()
        .context("Failed to deduplicate Customer Master on customer_id")?;
    let controlling_df = load_and_rename(&root, "controlling", bronze)?;
    let delivery_df    = load_and_rename(&root, "delivery",      bronze)?;

    let material_df = if root.contains_key("material_master") {
        info!("Material Master found — loading");
        Some(
            load_and_rename(&root, "material_master", bronze)?
                .lazy()
                .group_by([col("material_id")])
                .agg([col("*").first()])
                .collect()
                .context("Failed to deduplicate Material Master on material_id")?,
        )
    } else {
        None
    };

    info!(
        "ERP Bronze loaded — Header: {} rows, Items: {} rows, Customer: {} rows (deduped), \
         Controlling: {} rows, Delivery: {} rows",
        header_df.height(), items_df.height(), customer_df.height(),
        controlling_df.height(), delivery_df.height()
    );

    // --- Step 1: Inner join Sales Header + Items on order_id ---
    let sales = header_df
        .join(&items_df, ["order_id"], ["order_id"], JoinArgs::new(JoinType::Inner), None)
        .context("Failed to inner join Header and Items on order_id")?;
    info!("After Header ⋈ Items (inner): {} rows", sales.height());

    // --- Step 2: Left join with Customer Master on customer_id ---
    let sales = sales
        .join(&customer_df, ["customer_id"], ["customer_id"], JoinArgs::new(JoinType::Left), None)
        .context("Failed to left join with Customer Master on customer_id")?;
    info!("After ⋈ Customer Master (left): {} rows", sales.height());

    // --- Step 3 (optional): Left join with Material Master on material_id ---
    let sales = if let Some(mat_df) = material_df {
        let j = sales
            .join(&mat_df, ["material_id"], ["material_id"], JoinArgs::new(JoinType::Left), None)
            .context("Failed to left join with Material Master on material_id")?;
        info!("After ⋈ Material Master (left): {} rows", j.height());
        j
    } else {
        sales
    };

    // --- Step 4: Compute enrichment columns ---
    let netwr_ca         = sales.column("net_value")?.f64()?;
    let estimated_cost_ca = sales.column("estimated_cost")?.f64()?;
    let waerk_ca         = sales.column("currency")?.str()?;
    let matnr_ca         = sales.column("material_id")?.str()?;

    let netwr_usd: Vec<f64> = netwr_ca.into_iter()
        .zip(waerk_ca.into_iter())
        .map(|(n, c)| to_usd(n.unwrap_or(0.0), c.unwrap_or("USD")))
        .collect();

    let cost_usd: Vec<f64> = estimated_cost_ca.into_iter()
        .zip(waerk_ca.into_iter())
        .map(|(c, cur)| to_usd(c.unwrap_or(0.0), cur.unwrap_or("USD")))
        .collect();

    let margins: Vec<f64> = netwr_usd.iter()
        .zip(cost_usd.iter())
        .map(|(n, c)| margin_pct(*n, *c))
        .collect();

    let squeeze_flags: Vec<bool> = margins.iter().map(|m| is_margin_squeeze(*m)).collect();

    let mat01_squeeze: Vec<bool> = matnr_ca.into_iter()
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
    info!("Sales enriched: {} rows, {} columns", enriched.height(), enriched.width());

    // --- Step 6: Validate ---
    validate_silver_sales(&enriched)
        .context("Sales Silver validation failed")?;
    validate_silver_controlling(&controlling_df)
        .context("Controlling Silver validation failed")?;
    validate_silver_delivery(&delivery_df)
        .context("Delivery Silver validation failed")?;

    // --- Write Silver Parquet ---
    write_parquet(&mut enriched, &silver.join("sales_enriched.parquet"))?;
    info!("✅ sales_enriched.parquet written");
    write_parquet(&mut controlling_df.clone(), &silver.join("controlling_enriched.parquet"))?;
    info!("✅ controlling_enriched.parquet written");
    write_parquet(&mut delivery_df.clone(), &silver.join("delivery_enriched.parquet"))?;
    info!("✅ delivery_enriched.parquet written");

    Ok(true)
}

/// Load a Bronze CSV and rename its columns to canonical names per the saved mapping entry.
fn load_and_rename(
    root: &serde_json::Map<String, serde_json::Value>,
    data_type: &str,
    bronze: &Path,
) -> anyhow::Result<DataFrame> {
    let entry = root.get(data_type)
        .ok_or_else(|| anyhow::anyhow!("no mapping entry for '{}'", data_type))?;
    let file = entry.get("file")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("'{}' mapping missing 'file' key", data_type))?;
    let mapping = entry.get("mapping")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow::anyhow!("'{}' mapping missing 'mapping' array", data_type))?;

    let mut df = read_csv(&bronze.join(file))?;

    let existing: HashSet<String> =
        df.get_column_names().iter().map(|s| s.to_string()).collect();

    let mut frm: Vec<String> = Vec::new();
    let mut to:  Vec<String> = Vec::new();

    for item in mapping {
        let header    = item.get("header").and_then(|v| v.as_str()).unwrap_or("");
        let canonical = item.get("canonical").and_then(|v| v.as_str()).unwrap_or("");
        if !canonical.is_empty() && header != canonical && existing.contains(header) {
            frm.push(header.to_string());
            to.push(canonical.to_string());
        }
    }

    if !frm.is_empty() {
        df = df.lazy().rename(frm, to, true).collect()
            .with_context(|| format!("Failed to rename columns for '{}'", data_type))?;
    }

    Ok(df)
}

#[allow(dead_code)]
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

    // --- Step 6: Rename to generic column names ---
    enriched = enriched
        .lazy()
        .rename(
            ["ssh_order_id", "ssh_customer_id", "ssh_currency",
             "ssh_distribution_channel", "ssh_sales_org",
             "ssi_material_id", "ssi_net_value", "ssi_estimated_cost",
             "scm_industry", "scm_region_group"],
            ["order_id", "customer_id", "currency",
             "distribution_channel", "sales_org",
             "material_id", "net_value", "estimated_cost",
             "industry", "region_group"],
            true,
        )
        .collect()
        .context("Failed to rename sales columns to generic names")?;

    let controlling_df = controlling_df
        .lazy()
        .rename(
            ["sco_department", "sco_fiscal_year", "sco_actual_cost",
             "sco_budget_amount", "sco_budget_variance", "sco_order_id"],
            ["department", "fiscal_year", "actual_cost",
             "budget_amount", "budget_variance", "order_id"],
            true,
        )
        .collect()
        .context("Failed to rename controlling columns to generic names")?;

    let delivery_df = delivery_df
        .lazy()
        .rename(
            ["sdl_route", "sdl_transport_type", "sdl_days_late",
             "sdl_freight_cost_usd", "sdl_delivery_id"],
            ["route", "transport_type", "days_late",
             "freight_cost_usd", "delivery_id"],
            true,
        )
        .collect()
        .context("Failed to rename delivery columns to generic names")?;

    // --- Step 7: Validate before writing ---
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

#[cfg(test)]
mod tests {
    use super::*;

    fn csv_rows(header: &str, row_fn: impl Fn(usize) -> String, n: usize) -> String {
        let mut s = header.to_string();
        s.push('\n');
        for i in 1..=n {
            s.push_str(&row_fn(i));
            s.push('\n');
        }
        s
    }

    #[test]
    fn run_from_mapping_returns_false_with_no_config() {
        let result = run_from_mapping("/tmp/optima_no_such_config", "/tmp", "/tmp");
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn run_from_mapping_produces_silver_with_canonical_columns() {
        let tmp = std::env::temp_dir().join("optima_pipeline_test");
        std::fs::remove_dir_all(&tmp).ok();
        let bronze  = tmp.join("bronze");
        let silver  = tmp.join("silver");
        let config  = tmp.join("config");
        for d in [&bronze, &config] { std::fs::create_dir_all(d).unwrap(); }

        // 600 rows — passes the 500-row minimum in validate_silver_sales.
        // Unique order IDs ensure the inner join is 1:1 (not Cartesian).
        std::fs::write(bronze.join("hdr.csv"), csv_rows(
            "VBELN,KUNNR,WAERK,VTWEG,VKORG",
            |i| format!("ORD-{i:04},CUST-{i:04},USD,D1,SO01"),
            600,
        )).unwrap();

        std::fs::write(bronze.join("itm.csv"), csv_rows(
            "VBELN,MATNR,NETWR,EST_COST",
            |i| format!("ORD-{i:04},MAT-01,1000.0,700.0"),
            600,
        )).unwrap();

        std::fs::write(bronze.join("cust.csv"), csv_rows(
            "KUNNR,INDUSTRY,REGION",
            |i| format!("CUST-{i:04},Automotive,North"),
            600,
        )).unwrap();

        // controlling: budget_variance must equal actual_cost - budget_amount.
        // 900 - 1000 = -100 ✓
        std::fs::write(bronze.join("ctrl.csv"), csv_rows(
            "KOSTL,GJAHR,ACTUAL,BUDGET,VARIANCE,ORD_REF",
            |i| format!("Engineering,2024,900.0,1000.0,-100.0,ORD-{i:04}"),
            60,
        )).unwrap();

        std::fs::write(bronze.join("dlv.csv"), csv_rows(
            "ROUTE,TRANSTYPE,DLATE,FREIGHT,DELV_ID",
            |i| format!("Route-A,Truck,2,150.0,DEL-{i:04}"),
            60,
        )).unwrap();

        let mapping = serde_json::json!({
            "sales_header": {
                "file": "hdr.csv",
                "mapping": [
                    {"header":"VBELN","canonical":"order_id"},
                    {"header":"KUNNR","canonical":"customer_id"},
                    {"header":"WAERK","canonical":"currency"},
                    {"header":"VTWEG","canonical":"distribution_channel"},
                    {"header":"VKORG","canonical":"sales_org"}
                ]
            },
            "sales_items": {
                "file": "itm.csv",
                "mapping": [
                    {"header":"VBELN","canonical":"order_id"},
                    {"header":"MATNR","canonical":"material_id"},
                    {"header":"NETWR","canonical":"net_value"},
                    {"header":"EST_COST","canonical":"estimated_cost"}
                ]
            },
            "customer_master": {
                "file": "cust.csv",
                "mapping": [
                    {"header":"KUNNR","canonical":"customer_id"},
                    {"header":"INDUSTRY","canonical":"industry"},
                    {"header":"REGION","canonical":"region_group"}
                ]
            },
            "controlling": {
                "file": "ctrl.csv",
                "mapping": [
                    {"header":"KOSTL","canonical":"department"},
                    {"header":"GJAHR","canonical":"fiscal_year"},
                    {"header":"ACTUAL","canonical":"actual_cost"},
                    {"header":"BUDGET","canonical":"budget_amount"},
                    {"header":"VARIANCE","canonical":"budget_variance"},
                    {"header":"ORD_REF","canonical":"order_id"}
                ]
            },
            "delivery": {
                "file": "dlv.csv",
                "mapping": [
                    {"header":"ROUTE","canonical":"route"},
                    {"header":"TRANSTYPE","canonical":"transport_type"},
                    {"header":"DLATE","canonical":"days_late"},
                    {"header":"FREIGHT","canonical":"freight_cost_usd"},
                    {"header":"DELV_ID","canonical":"delivery_id"}
                ]
            }
        });
        std::fs::write(
            config.join("field_mapping.json"),
            serde_json::to_string_pretty(&mapping).unwrap(),
        ).unwrap();

        let result = run_from_mapping(
            config.to_str().unwrap(),
            bronze.to_str().unwrap(),
            silver.to_str().unwrap(),
        );
        assert!(result.is_ok(), "run_from_mapping failed: {:?}", result.err());
        assert!(result.unwrap(), "run_from_mapping returned false — mapping not found");

        // Silver parquets exist.
        assert!(silver.join("sales_enriched.parquet").exists());
        assert!(silver.join("controlling_enriched.parquet").exists());
        assert!(silver.join("delivery_enriched.parquet").exists());

        // Sales parquet has canonical column names, not ERP codes.
        let f = std::fs::File::open(silver.join("sales_enriched.parquet")).unwrap();
        let df = ParquetReader::new(f).finish().unwrap();
        let cols = df.get_column_names();
        for expected in &["order_id", "material_id", "net_value", "margin_pct"] {
            assert!(
                cols.iter().any(|n| n.as_str() == *expected),
                "missing canonical column '{expected}'"
            );
        }
        for erp in &["VBELN", "MATNR", "NETWR"] {
            assert!(
                !cols.iter().any(|n| n.as_str() == *erp),
                "ERP code '{erp}' should not be present"
            );
        }

        std::fs::remove_dir_all(&tmp).ok();
    }
}