/// Normalize a value to USD based on currency code.
/// Exchange rates are hardcoded for now — will be replaced
/// with a live rates API in a future iteration.
///
/// EUR → USD: 1.08
/// GBP → USD: 1.27
/// USD → USD: 1.00 (no conversion)
pub fn to_usd(amount: f64, currency: &str) -> f64 {
    match currency {
        "EUR" => amount * 1.08,
        "GBP" => amount * 1.27,
        "USD" => amount,
        other => {
            tracing::warn!("Unknown currency '{}' — defaulting to no conversion", other);
            amount
        }
    }
}

/// Compute margin percentage between net value and estimated cost.
/// Returns percentage as a float e.g. 5.26 means 5.26%
pub fn margin_pct(netwr: f64, estimated_cost: f64) -> f64 {
    if netwr == 0.0 {
        return 0.0;
    }
    ((netwr - estimated_cost) / netwr) * 100.0
}

/// Flag whether a row represents a margin squeeze.
/// Definition: margin < 6% on any material.
pub fn is_margin_squeeze(margin: f64) -> bool {
    margin < 6.0
}