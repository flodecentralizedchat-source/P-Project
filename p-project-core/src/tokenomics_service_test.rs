#[cfg(test)]
mod tests {
    use super::super::tokenomics_service::TokenomicsService;
    use std::io::Cursor;

    const CSV_SAMPLE: &str = r#"section,item,value,extra1,extra2,extra3,extra4
TOKENOMICS,Total Supply,350000000,,,,
TOKENOMICS,Circulating at Launch,70000000,20%,,,
TOKENOMICS,Team Allocation,70000000,20%,4-year vesting,1-year cliff,
VESTING,Team Vesting,20%,48 months,12-month cliff,Linear,
LP_RATIOS,Initial LP USDT,250000,,,,
LAUNCH_STRATEGY,DEX Listing,Monkol-DEX,Primary listing,,
PRICE_MODEL,Target Price,0.20,Requires: MC 14M,Increase users + LP,,
PRICE_MODEL,Target Price,0.50,Requires: MC 35M,Staking + burns,,
"#;

    #[test]
    fn parses_tokenomics_csv() {
        let service = TokenomicsService::from_reader(Cursor::new(CSV_SAMPLE)).expect("parse");
        let summary = service.summary();
        assert_eq!(summary.total_supply, 350_000_000.0);
        assert_eq!(summary.circulating_at_launch, Some(70_000_000.0));
        assert_eq!(summary.allocations.len(), 1);
        let team = &summary.allocations[0];
        assert_eq!(team.name, "Team Allocation");
        assert_eq!(team.amount, 70_000_000.0);
        assert_eq!(team.percent.as_deref(), Some("20%"));

        assert_eq!(summary.vesting.len(), 1);
        assert_eq!(summary.launch_strategy.len(), 1);
        assert_eq!(summary.price_targets.len(), 2);
        let price = &summary.price_targets[0];
        assert_eq!(price.target_price, 0.20);
        assert!(price.requirement.contains("MC 14M"));
        assert_eq!(price.market_cap, 0.20 * summary.total_supply);
    }

    #[test]
    fn computes_price_roundtrip() {
        let service = TokenomicsService::from_reader(Cursor::new(CSV_SAMPLE)).expect("parse");
        let summary = service.summary();
        let cap = summary.market_cap_for_price(1.0);
        assert_eq!(cap, summary.total_supply);
        let price = summary.price_for_market_cap(140_000_000.0);
        assert_eq!(price, 0.4);
    }
}
