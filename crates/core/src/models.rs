use serde::Deserialize;

/// VBAK — SAP Sales Header
/// Matches Mockaroo schema: SAP_Sales_Header
/// Fields: vbeln | erdat | kunnr | waerk | vtweg | common_batch_id
#[derive(Debug, Deserialize)]
pub struct SalesHeader {
    /// Row Number — primary key / join key to SalesItem
    pub vbeln: u64,

    /// Order creation date, format mm/dd/yyyy — 2% of rows will be blank
    pub erdat: Option<String>,

    /// Customer number, range 10000–99999, no decimals
    pub kunnr: u32,

    /// Currency code — one of: "USD", "EUR", "GBP"
    pub waerk: String,

    /// Distribution channel — one of: "10 (Wholesale)", "20 (Retail)"
    pub vtweg: String,

    /// Batch correlation ID, range 5000–6000, no decimals
    pub common_batch_id: u32,
}

/// VBAP — SAP Sales Items
/// Matches Mockaroo schema: SAP_Sales_Items
/// Fields: vbeln | posnr | matnr | zmeng | netpr | netwr | estimated_cost
#[derive(Debug, Deserialize)]
pub struct SalesItem {
    /// Row Number — foreign key → SalesHeader.vbeln
    pub vbeln: u64,

    /// Line item position — one of: 10, 20, 30
    pub posnr: u32,

    /// Material number — one of: "MAT-01", "MAT-02", "MAT-03"
    pub matnr: String,

    /// Order quantity, range 1–100, no decimals
    pub zmeng: u32,

    /// Net price per unit, range 50.00–1000.00, 2 decimal places
    pub netpr: f64,

    /// Net value — Mockaroo formula: zmeng * netpr
    pub netwr: f64,

    /// Estimated cost — Mockaroo formula:
    /// MAT-01 → netpr * 0.95  (5% margin squeeze simulation)
    /// MAT-02 → netpr * 0.88  (truncated in UI, assumed standard discount)
    /// MAT-03 → netpr * 0.80  (assumed — full formula hidden in screenshot)
    pub estimated_cost: f64,
}