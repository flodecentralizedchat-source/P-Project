#[cfg(test)]
mod tests {
    use super::super::partnerships_service::*;
    use super::super::tokenomics_service::TokenomicsService;
    use std::io::Cursor;

    const CSV_WITH_STEPS: &str = r#"section,item,value,extra1,extra2
TOKENOMICS,Total Supply,350000000,,
LAUNCH_STRATEGY,DEX Listing,Start with permissionless DEXs,,
LAUNCH_STRATEGY,CEX Listing,Pursue after DEX credibility,,
LAUNCH_STRATEGY,Market Making,Partner with MM to ensure liquidity,,
"#;

    #[test]
    fn default_contains_expected_platforms() {
        let s = ExchangeListings::default();
        let names: Vec<_> = s
            .dex_strategy
            .platforms
            .iter()
            .map(|p| p.name.as_str())
            .collect();
        assert!(names.contains(&"Uniswap"));
        assert!(names.contains(&"PancakeSwap"));
        assert!(names.contains(&"Raydium"));
        assert!(s.dex_strategy.start_with_permissionless);
        assert!(s.market_makers.partner_with_mm);
        assert!(s.cex_progression.pursue_after_dex);
    }

    #[test]
    fn builds_from_tokenomics_summary_flags() {
        let service = TokenomicsService::from_reader(Cursor::new(CSV_WITH_STEPS)).expect("parse");
        let s = ExchangeListings::from_tokenomics_service(&service);
        assert!(s.dex_strategy.start_with_permissionless);
        assert!(s.cex_progression.pursue_after_dex);
        assert!(s.market_makers.partner_with_mm);
    }
}

