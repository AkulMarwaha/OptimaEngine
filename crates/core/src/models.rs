use serde::Deserialize;

/// VBAK — SAP Sales Header (enriched)
#[derive(Debug, Deserialize)]
pub struct SalesHeader {
    pub vbeln: u64,
    pub erdat: Option<String>,
    pub kunnr: u32,
    pub waerk: String,
    pub vtweg: String,
    pub common_batch_id: u32,
    pub auart: String,
    pub vkorg: String,
    pub spart: String,
    pub bstnk: String,
    pub vsbed: String,
    pub zterm: String,
    pub lifsk: String,
    pub faksk: String,
    pub pay_method: String,
    pub pay_status: String,
    pub auth_status: String,
    pub discount_code: String,
}

/// VBAP — SAP Sales Items (enriched)
#[derive(Debug, Deserialize)]
pub struct SalesItem {
    pub vbeln: u64,
    pub posnr: u32,
    pub matnr: String,
    pub zmeng: u32,
    pub netpr: f64,
    pub netwr: f64,
    pub estimated_cost: f64,
    pub werks: String,
    pub lgort: String,
    pub pstyv: String,
    pub abgru: String,
    pub bonus: f64,
    pub mwsbp: f64,
    pub kwmeng: u32,
    pub lmeng: u32,
    pub route: String,
    pub vstel: String,
}

/// KNA1 — SAP Customer Master
#[derive(Debug, Deserialize)]
pub struct CustomerMaster {
    pub kunnr: u32,
    pub name1: String,
    pub land1: String,
    pub regio: String,
    pub kukla: String,
    pub kraus: String,
    pub klkla: String,
    pub waers: String,
    pub zterm: String,
    pub vkorg: String,
    pub industry: String,
    pub region_grp: String,
    pub since_year: u32,
}

/// MARA — SAP Material Master
#[derive(Debug, Deserialize)]
pub struct MaterialMaster {
    pub matnr: String,
    pub maktx: String,
    pub matkl: String,
    pub mtart: String,
    pub meins: String,
    pub mvgr1: String,
    pub labor_cost_pct: f64,
    pub overhead_pct: f64,
    pub brgew: f64,
    pub mtpos_mara: String,
    pub mfrnr: String,
    pub ersda: String,
}

/// COPA — SAP Controlling / Profitability Analysis
#[derive(Debug, Deserialize)]
pub struct Controlling {
    pub vbeln: u64,
    pub kostl: u32,
    pub prctr: String,
    pub kokrs: String,
    pub gjahr: String,
    pub poper: u32,
    pub kstar: String,
    pub wkgbtr: f64,
    pub budget_amt: f64,
    pub variance_amt: f64,
    pub abtei: String,
    pub segment: String,
}

/// LIKP — SAP Delivery
#[derive(Debug, Deserialize)]
pub struct Delivery {
    pub vbeln_delivery: u64,
    pub vbeln_order: u64,
    pub kunnr: u32,
    pub wadat_ist: String,
    pub lddat: String,
    pub tddat: String,
    pub vstel: String,
    pub route: String,
    pub traty: String,
    pub freight_cost: f64,
    pub delivery_status: String,
    pub days_late: f64,
}