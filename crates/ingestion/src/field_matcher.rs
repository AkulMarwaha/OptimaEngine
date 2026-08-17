use std::collections::HashMap;

/// Which matching tier produced this result.
#[derive(Debug, PartialEq)]
pub enum MatchTier {
    /// Matched against a known ERP field code (e.g. SAP VBELN → order_id).
    Known,
    /// Matched via plain-English synonym heuristics.
    Heuristic,
    /// No confident match found.
    NoMatch,
}

/// Proposed mapping for one input column header.
#[derive(Debug)]
pub struct FieldMatch {
    pub canonical: Option<&'static str>,
    pub tier: MatchTier,
}

/// Build a flat uppercase-code → canonical-name lookup from per-ERP tables.
///
/// The outer slice is keyed by ERP name so additional ERP sets can be added
/// later (e.g. NetSuite, Oracle, Dynamics) without restructuring — only the
/// SAP set is populated until those field codes are verified.
fn known_codes() -> HashMap<String, &'static str> {
    let erp_tables: &[(&str, &[(&str, &str)])] = &[(
        "SAP",
        &[
            ("VBELN", "order_id"),
            ("KUNNR", "customer_id"),
            ("MATNR", "material_id"),
            ("NETWR", "net_value"),
            ("WAERK", "currency"),
            ("VKORG", "sales_org"),
            ("VTWEG", "distribution_channel"),
            ("KOSTL", "department"),
            ("GJAHR", "fiscal_year"),
            ("ERDAT", "order_date"),
            ("BUKRS", "company_code"),
            ("WERKS", "plant"),
        ],
    )];

    let mut map = HashMap::new();
    for (_erp, codes) in erp_tables {
        for &(code, canonical) in *codes {
            map.insert(code.to_uppercase(), canonical);
        }
    }
    map
}

/// Build a normalized-text → canonical-name lookup for Tier 2 heuristics.
fn synonym_map() -> HashMap<String, &'static str> {
    let entries: &[(&[&str], &str)] = &[
        (
            &[
                "order", "ordernumber", "orderid", "orderno",
                "ponumber", "po", "purchaseorder", "salesorder",
            ],
            "order_id",
        ),
        (
            &[
                "customer", "customername", "customerid", "customernumber",
                "client", "clientid", "account", "accountid",
                "custid", "custno",
            ],
            "customer_id",
        ),
        (
            &[
                "material", "materialid", "itemnumber", "item",
                "sku", "skunumber", "part", "partnumber", "product",
            ],
            "material_id",
        ),
        (
            &[
                "amount", "netamount", "netvalue", "salesamount",
                "revenue", "value", "price",
            ],
            "net_value",
        ),
        (
            &["estimatedcost", "unitcost", "cogs", "costofgoods"],
            "estimated_cost",
        ),
        (&["currency", "curr", "currencycode"], "currency"),
        (
            &["distributionchannel", "saleschannel", "distchannel", "channel"],
            "distribution_channel",
        ),
        (&["salesorg", "salesorganization"], "sales_org"),
        (&["industry", "sector", "industrytype"], "industry"),
        (&["regiongroup", "region", "geography", "area"], "region_group"),
        (&["department", "dept", "costcenter", "division"], "department"),
        (&["fiscalyear", "fy", "fiscalyr"], "fiscal_year"),
        (&["actualcost", "actuals", "actual", "spend"], "actual_cost"),
        (
            &["budgetamount", "budget", "plannedcost", "planned", "plan"],
            "budget_amount",
        ),
        (&["budgetvariance", "variance", "var"], "budget_variance"),
        (&["route", "routeid", "shippingroute", "deliveryroute"], "route"),
        (
            &["transporttype", "shipmenttype", "carriertype", "transport"],
            "transport_type",
        ),
        (
            &["dayslate", "latedays", "delay", "deliverydelay", "lateness"],
            "days_late",
        ),
        (
            &["freightcost", "freightcostusd", "freight", "shippingcost", "freightcharge"],
            "freight_cost_usd",
        ),
        (
            &["deliveryid", "deliverynumber", "delivery", "shipmentid", "shipment"],
            "delivery_id",
        ),
    ];

    let mut map = HashMap::new();
    for &(synonyms, canonical) in entries {
        for &syn in synonyms {
            map.insert(syn.to_string(), canonical);
        }
    }
    map
}

/// Strip non-alphanumeric characters and lowercase.
fn normalize(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_alphanumeric())
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

/// Propose a canonical field mapping for each input column header.
///
/// Returns one [`FieldMatch`] per header in input order. Tier 1 (Known) takes
/// priority over Tier 2 (Heuristic). The confirm screen should flag Heuristic
/// and NoMatch results for human review before the pipeline runs.
pub fn match_headers(headers: &[&str]) -> Vec<FieldMatch> {
    let codes = known_codes();
    let synonyms = synonym_map();

    headers
        .iter()
        .map(|h| {
            // Tier 1: case-insensitive exact match against known ERP field codes.
            if let Some(&canonical) = codes.get(&h.trim().to_uppercase()) {
                return FieldMatch {
                    canonical: Some(canonical),
                    tier: MatchTier::Known,
                };
            }

            // Tier 2: normalize whitespace/punctuation and check synonym table.
            if let Some(&canonical) = synonyms.get(&normalize(h)) {
                return FieldMatch {
                    canonical: Some(canonical),
                    tier: MatchTier::Heuristic,
                };
            }

            FieldMatch { canonical: None, tier: MatchTier::NoMatch }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tier1_sap_field_codes() {
        let headers = &["VBELN", "KUNNR", "MATNR", "NETWR", "WAERK", "VKORG", "GJAHR"];
        let results = match_headers(headers);

        assert_eq!(results[0].tier, MatchTier::Known);
        assert_eq!(results[0].canonical, Some("order_id"));

        assert_eq!(results[1].tier, MatchTier::Known);
        assert_eq!(results[1].canonical, Some("customer_id"));

        assert_eq!(results[2].tier, MatchTier::Known);
        assert_eq!(results[2].canonical, Some("material_id"));

        assert_eq!(results[3].tier, MatchTier::Known);
        assert_eq!(results[3].canonical, Some("net_value"));

        assert_eq!(results[4].tier, MatchTier::Known);
        assert_eq!(results[4].canonical, Some("currency"));

        assert_eq!(results[5].tier, MatchTier::Known);
        assert_eq!(results[5].canonical, Some("sales_org"));

        assert_eq!(results[6].tier, MatchTier::Known);
        assert_eq!(results[6].canonical, Some("fiscal_year"));
    }

    #[test]
    fn tier1_sap_codes_are_case_insensitive() {
        let results = match_headers(&["vbeln", "kunnr", "matnr"]);
        assert!(results.iter().all(|r| r.tier == MatchTier::Known));
        assert_eq!(results[0].canonical, Some("order_id"));
        assert_eq!(results[1].canonical, Some("customer_id"));
        assert_eq!(results[2].canonical, Some("material_id"));
    }

    #[test]
    fn tier2_plain_english_headers() {
        let headers = &[
            "Order Number",
            "Customer Name",
            "SKU",
            "Part Number",
            "Distribution Channel",
            "Days Late",
            "Freight Cost",
        ];
        let results = match_headers(headers);

        assert_eq!(results[0].tier, MatchTier::Heuristic);
        assert_eq!(results[0].canonical, Some("order_id"));

        assert_eq!(results[1].tier, MatchTier::Heuristic);
        assert_eq!(results[1].canonical, Some("customer_id"));

        assert_eq!(results[2].tier, MatchTier::Heuristic);
        assert_eq!(results[2].canonical, Some("material_id"));

        assert_eq!(results[3].tier, MatchTier::Heuristic);
        assert_eq!(results[3].canonical, Some("material_id"));

        assert_eq!(results[4].tier, MatchTier::Heuristic);
        assert_eq!(results[4].canonical, Some("distribution_channel"));

        assert_eq!(results[5].tier, MatchTier::Heuristic);
        assert_eq!(results[5].canonical, Some("days_late"));

        assert_eq!(results[6].tier, MatchTier::Heuristic);
        assert_eq!(results[6].canonical, Some("freight_cost_usd"));
    }

    #[test]
    fn tier2_ignores_punctuation_and_case() {
        let results = match_headers(&["order_number", "ORDER-NUMBER", "  Order Number  "]);
        assert!(results.iter().all(|r| r.tier == MatchTier::Heuristic));
        assert!(results.iter().all(|r| r.canonical == Some("order_id")));
    }

    #[test]
    fn no_match_for_unrecognised_headers() {
        let headers = &["XZQ42", "foobar", "not_a_field", "column1", "!!!"];
        let results = match_headers(headers);
        assert!(results.iter().all(|r| r.tier == MatchTier::NoMatch));
        assert!(results.iter().all(|r| r.canonical.is_none()));
    }
}
