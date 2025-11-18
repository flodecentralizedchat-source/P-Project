#[cfg(test)]
mod tests {
    use super::super::dex_listing::DexListingManager;
    use super::super::liquidity_pool::LiquidityPool;

    #[test]
    fn seeds_deep_liquidity_for_primary_pairs() {
        let mut mgr = DexListingManager::new();

        // Create constant product pool P-COIN/USDC and seed deep vs shallow
        mgr.create_constant_pool(
            "pcoin_usdc",
            "P-COIN",
            "USDC",
            0.003,
            "REWARD",
            100_000.0,
            0.12,
        );

        // Shallow: 10k/10k
        let mut shallow = LiquidityPool::new(
            "shallow".to_string(),
            "P-COIN".to_string(),
            "USDC".to_string(),
            0.003,
            "REWARD".to_string(),
            100_000.0,
            0.12,
        );
        shallow
            .add_liquidity("lp".into(), 10_000.0, 10_000.0, 30)
            .unwrap();

        // Deep: 1,000,000 / 1,000,000
        mgr.seed_constant_liquidity("pcoin_usdc", "lp", 1_000_000.0, 1_000_000.0, 30)
            .unwrap();

        let out_shallow = shallow.calculate_swap_output("P-COIN", 10_000.0).unwrap();
        let out_deep = mgr
            .constant_pool("pcoin_usdc")
            .unwrap()
            .calculate_swap_output("P-COIN", 10_000.0)
            .unwrap();

        assert!(
            out_deep > out_shallow,
            "deep liquidity must reduce slippage"
        );
    }

    #[test]
    fn prepares_stable_pool_for_primary_pair() {
        let mut mgr = DexListingManager::new();
        mgr.create_stable_pool(
            "pcoin_usdc_stable",
            "P-COIN",
            "USDC",
            0.0005,
            100.0,
            "REWARD",
            100_000.0,
            0.10,
        )
        .unwrap();

        mgr.seed_stable_liquidity("pcoin_usdc_stable", "lp1", 100_000.0, 100_000.0, 30)
            .unwrap();

        let sp = mgr.stable_pool("pcoin_usdc_stable").unwrap();
        // sanity: reserves updated and output is sensible
        let out = sp.calculate_swap_output("P-COIN", 10_000.0).unwrap();
        assert!(out > 0.0);
    }
}
