use serde::Deserialize;

/// SAP_Sales_Header (SSH)
#[derive(Debug, Deserialize)]
pub struct SalesHeader {
    pub ssh_order_id: u64,
    pub ssh_order_date: Option<String>,
    pub ssh_customer_id: u32,
    pub ssh_currency: String,
    pub ssh_distribution_channel: String,
    pub ssh_batch_id: u32,
    pub ssh_order_type: String,
    pub ssh_sales_org: String,
    pub ssh_division: String,
    pub ssh_customer_po_number: String,
    pub ssh_shipping_condition: String,
    pub ssh_payment_terms: String,
    pub ssh_delivery_block: String,
    pub ssh_billing_block: String,
    pub ssh_payment_method: String,
    pub ssh_payment_status: String,
    pub ssh_authorization_status: String,
    pub ssh_discount_code: String,
}

/// SAP_Sales_Items (SSI)
#[derive(Debug, Deserialize)]
pub struct SalesItem {
    pub ssi_order_id: u64,
    pub ssi_line_item_number: u32,
    pub ssi_material_id: String,
    pub ssi_order_quantity: u32,
    pub ssi_unit_price: f64,
    pub ssi_net_value: f64,
    pub ssi_estimated_cost: f64,
    pub ssi_plant: String,
    pub ssi_storage_location: String,
    pub ssi_item_category: String,
    pub ssi_rejection_reason: String,
    pub ssi_rebate_pct: f64,
    pub ssi_tax_amount: f64,
    pub ssi_confirmed_quantity: u32,
    pub ssi_delivered_quantity: u32,
    pub ssi_delivery_route: String,
    pub ssi_shipping_point: String,
}

/// SAP_Customer_Master (SCM)
#[derive(Debug, Deserialize)]
pub struct CustomerMaster {
    pub scm_customer_id: u32,
    pub scm_company_name: String,
    pub scm_country: String,
    pub scm_region: String,
    pub scm_customer_class: String,
    pub scm_credit_rating: String,
    pub scm_credit_risk_class: String,
    pub scm_currency: String,
    pub scm_payment_terms: String,
    pub scm_sales_org: String,
    pub scm_industry: String,
    pub scm_region_group: String,
    pub scm_customer_since_year: u32,
}

/// SAP_Material_Master (SMM)
#[derive(Debug, Deserialize)]
pub struct MaterialMaster {
    pub smm_material_id: String,
    pub smm_material_description: String,
    pub smm_material_group: String,
    pub smm_material_type: String,
    pub smm_unit_of_measure: String,
    pub smm_pricing_group: String,
    pub smm_labor_cost_pct: f64,
    pub smm_overhead_pct: f64,
    pub smm_gross_weight: f64,
    pub smm_item_category_group: String,
    pub smm_manufacturer: String,
    pub smm_creation_date: String,
}

/// SAP_Controlling (SCO)
#[derive(Debug, Deserialize)]
pub struct Controlling {
    pub sco_order_id: u64,
    pub sco_cost_center_id: u32,
    pub sco_profit_center: String,
    pub sco_controlling_area: String,
    pub sco_fiscal_year: String,
    pub sco_posting_period: u32,
    pub sco_cost_element: String,
    pub sco_actual_cost: f64,
    pub sco_budget_amount: f64,
    pub sco_budget_variance: f64,
    pub sco_department: String,
    pub sco_market_segment: String,
}

/// SAP_Delivery (SDL)
#[derive(Debug, Deserialize)]
pub struct Delivery {
    pub sdl_delivery_id: u64,
    pub sdl_order_id: u64,
    pub sdl_customer_id: u32,
    pub sdl_actual_goods_issue_date: String,
    pub sdl_loading_date: String,
    pub sdl_transport_planning_date: String,
    pub sdl_shipping_point: String,
    pub sdl_route: String,
    pub sdl_transport_type: String,
    pub sdl_freight_cost_usd: f64,
    pub sdl_delivery_status: String,
    pub sdl_days_late: f64,
}