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
    LP_RATIOS,Initial LP Tokens,25000000,,,,
    LP_RATIOS,Starting Price,0.01,,,,
    LAUNCH_STRATEGY,DEX Listing,Monkol-DEX,Primary listing,,
    PRICE_MODEL,Target Price,0.20,$14M,Users + LP growth,Adoption phase,Adoption push
    PRICE_MODEL,Target Price,0.50,$35M,Staking + burns,Ecosystem expansion,
    "#;

    const CSV_LP_INCONSISTENT_PRICE: &str = r#"section,item,value,extra1,extra2,extra3,extra4
    TOKENOMICS,Total Supply,350000000,,,,
    TOKENOMICS,Circulating at Launch,70000000,20%,,,
    TOKENOMICS,Team Allocation,70000000,20%,4-year vesting,1-year cliff,
    VESTING,Team Vesting,20%,48 months,12-month cliff,Linear,
    LP_RATIOS,Initial LP USDT,250000,,,,
    LP_RATIOS,Initial LP Tokens,25000000,,,,
    LP_RATIOS,Starting Price,0.02,,,,
    LAUNCH_STRATEGY,DEX Listing,Monkol-DEX,Primary listing,,
    PRICE_MODEL,Target Price,0.20,$14M,Users + LP growth,Adoption phase,Adoption push
    PRICE_MODEL,Target Price,0.50,$35M,Staking + burns,Ecosystem expansion,
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
        assert_eq!(price.market_cap_required, Some(14_000_000.0));
        assert_eq!(price.mechanisms.as_deref(), Some("Users + LP growth"));
        assert_eq!(price.components.as_deref(), Some("Adoption phase"));
        assert_eq!(price.note.as_deref(), Some("Adoption push"));
        assert_eq!(price.market_cap, 0.20 * summary.total_supply);

        let ratios = summary.lp_ratios.as_ref().expect("lp ratios available");
        assert_eq!(ratios.initial_lp_usdt, 250_000.0);
        assert_eq!(ratios.initial_lp_tokens, 25_000_000.0);
        assert!((ratios.computed_price - 0.01).abs() < f64::EPSILON);
        assert!(ratios.ratio_consistent);
        assert!(ratios
            .notes
            .iter()
            .any(|note| note.contains("Derived formula")));
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

    #[test]
    fn price_model_helpers() {
        let service = TokenomicsService::from_reader(Cursor::new(CSV_SAMPLE)).expect("parse");
        let summary = service.summary();

        assert!(summary.price_target_for_price(0.20).is_some());
        assert!(summary.price_target_for_price(0.33).is_none());

        assert_eq!(
            summary.market_cap_gap_to_target(0.20, 10_000_000.0),
            Some(4_000_000.0)
        );
        assert_eq!(
            summary.market_cap_gap_to_target(0.50, 30_000_000.0),
            Some(5_000_000.0)
        );
        assert!(summary
            .market_cap_gap_to_target(0.20, 20_000_000.0)
            .is_some());

        let next_target = summary.next_price_target(0.25).expect("next target exists");
        assert_eq!(next_target.target_price, 0.50);

        assert!(summary.next_price_target(0.50).is_none());
    }

    #[test]
    fn lp_ratios_detects_inconsistent_starting_price() {
        let service =
            TokenomicsService::from_reader(Cursor::new(CSV_LP_INCONSISTENT_PRICE)).expect("parse");
        let ratios = service
            .summary()
            .lp_ratios
            .as_ref()
            .expect("lp ratios available");
        assert_eq!(ratios.computed_price, 0.01);
        assert_eq!(ratios.starting_price, 0.02);
        assert!(!ratios.ratio_consistent);
    }
}
