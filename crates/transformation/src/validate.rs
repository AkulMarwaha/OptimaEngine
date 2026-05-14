use anyhow::anyhow;
use polars::prelude::*;
use tracing::{error, info, warn};

/// Validation result with details about what failed
#[derive(Debug)]
pub struct ValidationReport {
    pub passed: bool,
    pub checks: Vec<CheckResult>,
}

#[derive(Debug)]
pub struct CheckResult {
    pub name: &'static str,
    pub passed: bool,
    #[allow(dead_code)]
    pub message: String,
}

impl ValidationReport {
    fn new() -> Self {
        Self { passed: true, checks: Vec::new() }
    }

    fn record(&mut self, name: &'static str, passed: bool, message: String) {
        if !passed {
            self.passed = false;
            error!("❌ VALIDATION FAILED [{}]: {}", name, message);
        } else {
            info!("✅ VALIDATION PASSED [{}]: {}", name, message);
        }
        self.checks.push(CheckResult { name, passed, message });
    }
}

/// Validate the enriched Silver sales DataFrame before writing.
/// Returns Err if any hard-fail check fails — pipeline must stop.
pub fn validate_silver_sales(df: &DataFrame) -> anyhow::Result<()> {
    let mut report = ValidationReport::new();
    let height = df.height();

    // --- Check 1: Row count sanity ---
    let min_expected_rows = 500;
    let max_expected_rows = 5000;
    report.record(
        "row_count_range",
        height >= min_expected_rows && height <= max_expected_rows,
        format!(
            "{} rows (expected {min_expected_rows}–{max_expected_rows})",
            height
        ),
    );

    // --- Check 2: No nulls in join keys ---
    for col_name in &["ssh_order_id", "ssh_customer_id", "ssi_material_id"] {
        let null_count = df.column(col_name)?.null_count();
        report.record(
            "join_key_no_nulls",
            null_count == 0,
            format!("column '{}' has {} null(s)", col_name, null_count),
        );
    }

    // --- Check 3: netwr_usd must be positive ---
    let netwr = df.column("netwr_usd")?.f64()?;
    let non_positive: usize = netwr
        .into_iter()
        .filter(|v| v.map(|x| x <= 0.0).unwrap_or(true))
        .count();
    report.record(
        "netwr_usd_positive",
        non_positive == 0,
        format!("{} row(s) with netwr_usd <= 0", non_positive),
    );

    // --- Check 4: margin_pct in valid range ---
    let margins = df.column("margin_pct")?.f64()?;
    let above_100: usize = margins
        .into_iter()
        .filter(|v| v.map(|x| x > 100.0).unwrap_or(false))
        .count();
    let negative: usize = margins
        .into_iter()
        .filter(|v| v.map(|x| x < 0.0).unwrap_or(false))
        .count();

    report.record(
        "margin_pct_not_above_100",
        above_100 == 0,
        format!("{} row(s) with margin_pct > 100% (formula error)", above_100),
    );

    if negative > 0 {
        warn!(
            "⚠️  VALIDATION WARNING [negative_margins]: {} row(s) with margin_pct < 0 — possible loss-leader or data issue",
            negative
        );
    } else {
        info!("✅ VALIDATION PASSED [negative_margins]: no negative margins");
    }

    // --- Check 5: estimated_cost_usd must be positive ---
    let costs = df.column("estimated_cost_usd")?.f64()?;
    let non_positive_cost: usize = costs
        .into_iter()
        .filter(|v| v.map(|x| x <= 0.0).unwrap_or(true))
        .count();
    report.record(
        "estimated_cost_usd_positive",
        non_positive_cost == 0,
        format!("{} row(s) with estimated_cost_usd <= 0", non_positive_cost),
    );

    // --- Check 6: Margin cross-validation ---
    let netwr_arr = df.column("netwr_usd")?.f64()?;
    let cost_arr = df.column("estimated_cost_usd")?.f64()?;
    let margin_arr = df.column("margin_pct")?.f64()?;

    let inconsistent: usize = netwr_arr
        .into_iter()
        .zip(cost_arr.into_iter())
        .zip(margin_arr.into_iter())
        .filter(|((n, c), m)| {
            match (n, c, m) {
                (Some(netwr), Some(cost), Some(margin)) if *netwr > 0.0 => {
                    let expected = ((netwr - cost) / netwr) * 100.0;
                    (expected - margin).abs() > 0.01
                }
                _ => false,
            }
        })
        .count();
    report.record(
        "margin_cross_validation",
        inconsistent == 0,
        format!(
            "{} row(s) where margin_pct doesn't match (netwr - cost) / netwr",
            inconsistent
        ),
    );

    // --- Check 7: No nulls in enrichment columns ---
    for col_name in &["netwr_usd", "estimated_cost_usd", "margin_pct", "is_margin_squeeze"] {
        let null_count = df.column(col_name)?.null_count();
        report.record(
            "enrichment_col_no_nulls",
            null_count == 0,
            format!("enrichment column '{}' has {} null(s)", col_name, null_count),
        );
    }

    // --- Final gate ---
    if !report.passed {
        let failed: Vec<&str> = report
            .checks
            .iter()
            .filter(|c| !c.passed)
            .map(|c| c.name)
            .collect();
        return Err(anyhow!(
            "Silver validation failed — {} check(s) failed: {:?}. Pipeline halted. No Silver file written.",
            failed.len(),
            failed
        ));
    }

    info!("✅ All Silver sales validation checks passed ({} rows)", height);
    Ok(())
}

/// Validate the Controlling DataFrame (pass-through to Silver).
pub fn validate_silver_controlling(df: &DataFrame) -> anyhow::Result<()> {
    let mut report = ValidationReport::new();
    let height = df.height();

    report.record(
        "controlling_row_count",
        height >= 50,
        format!("{} rows (expected >= 50)", height),
    );

    let variance = df.column("sco_budget_variance")?.f64()?;
    let null_count = variance.null_count();
    report.record(
        "budget_variance_no_nulls",
        null_count == 0,
        format!("sco_budget_variance has {} null(s)", null_count),
    );

    let actual = df.column("sco_actual_cost")?.f64()?;
    let budget = df.column("sco_budget_amount")?.f64()?;
    let inconsistent: usize = actual
        .into_iter()
        .zip(budget.into_iter())
        .zip(variance.into_iter())
        .filter(|((a, b), v)| match (a, b, v) {
            (Some(act), Some(bud), Some(var)) => ((*act - *bud) - *var).abs() > 0.01,
            _ => false,
        })
        .count();
    report.record(
        "budget_variance_cross_validation",
        inconsistent == 0,
        format!(
            "{} row(s) where sco_budget_variance != actual - budget",
            inconsistent
        ),
    );

    if !report.passed {
        let failed: Vec<&str> = report.checks.iter().filter(|c| !c.passed).map(|c| c.name).collect();
        return Err(anyhow!(
            "Controlling validation failed — {:?}. Pipeline halted.",
            failed
        ));
    }

    info!("✅ All Controlling validation checks passed ({} rows)", height);
    Ok(())
}

/// Validate the Delivery DataFrame (pass-through to Silver).
pub fn validate_silver_delivery(df: &DataFrame) -> anyhow::Result<()> {
    let mut report = ValidationReport::new();
    let height = df.height();

    report.record(
        "delivery_row_count",
        height >= 50,
        format!("{} rows (expected >= 50)", height),
    );

    let days_late = df.column("sdl_days_late")?.f64()?;
    let negative_delays: usize = days_late
        .into_iter()
        .filter(|v| v.map(|x| x < 0.0).unwrap_or(false))
        .count();
    report.record(
        "days_late_non_negative",
        negative_delays == 0,
        format!("{} row(s) with sdl_days_late < 0 (impossible value)", negative_delays),
    );

    let freight = df.column("sdl_freight_cost_usd")?.f64()?;
    let non_positive_freight: usize = freight
        .into_iter()
        .filter(|v| v.map(|x| x <= 0.0).unwrap_or(true))
        .count();
    report.record(
        "freight_cost_positive",
        non_positive_freight == 0,
        format!("{} row(s) with sdl_freight_cost_usd <= 0", non_positive_freight),
    );

    if !report.passed {
        let failed: Vec<&str> = report.checks.iter().filter(|c| !c.passed).map(|c| c.name).collect();
        return Err(anyhow!(
            "Delivery validation failed — {:?}. Pipeline halted.",
            failed
        ));
    }

    info!("✅ All Delivery validation checks passed ({} rows)", height);
    Ok(())
}

// ============================================================
// UNIT TESTS
// ============================================================
#[cfg(test)]
mod tests {
    use crate::compute::{is_margin_squeeze, margin_pct, to_usd};

    #[test]
    fn test_margin_pct_mat02_formula() {
        let price = 100.0_f64;
        let cost = price * 0.94;
        let margin = margin_pct(price, cost);
        assert!(
            (margin - 6.0).abs() < 0.001,
            "MAT-02 margin should be ~6%, got {margin}"
        );
    }

    #[test]
    fn test_margin_pct_mat01_formula() {
        let price = 100.0_f64;
        let cost = price * 0.98;
        let margin = margin_pct(price, cost);
        assert!(
            (margin - 2.0).abs() < 0.001,
            "MAT-01 margin should be ~2%, got {margin}"
        );
    }

    #[test]
    fn test_margin_pct_mat04_formula() {
        let price = 100.0_f64;
        let cost = price * 0.72;
        let margin = margin_pct(price, cost);
        assert!(
            (margin - 28.0).abs() < 0.001,
            "MAT-04 margin should be ~28%, got {margin}"
        );
    }

    #[test]
    fn test_margin_pct_zero_revenue() {
        let margin = margin_pct(0.0, 50.0);
        assert_eq!(margin, 0.0, "Zero revenue should return 0.0 margin");
    }

    #[test]
    fn test_margin_pct_100_percent() {
        let margin = margin_pct(100.0, 0.0);
        assert!((margin - 100.0).abs() < 0.001, "Zero cost should be 100% margin");
    }

    #[test]
    fn test_squeeze_fires_below_6_pct() {
        assert!(is_margin_squeeze(5.9), "5.9% should be a squeeze");
        assert!(is_margin_squeeze(2.0), "2.0% should be a squeeze");
        assert!(is_margin_squeeze(0.0), "0.0% should be a squeeze");
    }

    #[test]
    fn test_squeeze_does_not_fire_at_6_pct() {
        assert!(!is_margin_squeeze(6.0), "Exactly 6.0% should NOT be a squeeze");
        assert!(!is_margin_squeeze(6.1), "6.1% should NOT be a squeeze");
        assert!(!is_margin_squeeze(28.0), "28.0% should NOT be a squeeze");
    }

    #[test]
    fn test_mat01_triggers_squeeze() {
        let margin = margin_pct(100.0, 98.0);
        assert!(is_margin_squeeze(margin), "MAT-01 at 2% should trigger squeeze");
    }

    #[test]
    fn test_mat02_triggers_squeeze_at_threshold() {
        let margin = margin_pct(100.0, 94.0);
        assert!(
            !is_margin_squeeze(margin),
            "MAT-02 at exactly 6.0% should NOT trigger squeeze (< not <=)"
        );
    }

    #[test]
    fn test_to_usd_passthrough() {
        assert_eq!(to_usd(100.0, "USD"), 100.0);
    }

    #[test]
    fn test_to_usd_eur_conversion() {
        let result = to_usd(100.0, "EUR");
        assert!((result - 108.0).abs() < 0.001, "EUR→USD should be 1.08x");
    }

    #[test]
    fn test_to_usd_gbp_conversion() {
        let result = to_usd(100.0, "GBP");
        assert!((result - 127.0).abs() < 0.001, "GBP→USD should be 1.27x");
    }

    #[test]
    fn test_to_usd_unknown_currency_no_conversion() {
        let result = to_usd(100.0, "JPY");
        assert_eq!(result, 100.0, "Unknown currency should pass through unchanged");
    }
}