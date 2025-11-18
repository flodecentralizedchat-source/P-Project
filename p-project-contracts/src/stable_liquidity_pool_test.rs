#[cfg(test)]
mod tests {
    use super::super::liquidity_pool::LiquidityPool;
    use super::super::stable_liquidity_pool::StableLiquidityPool;

    #[test]
    fn test_create_stable_pool() {
        let pool = StableLiquidityPool::new(
            "pcoin_usdc_stable".to_string(),
            "P-COIN".to_string(),
            "USDC".to_string(),
            0.0005, // 5 bps
            100.0,  // high amplification
            "REWARD".to_string(),
            100000.0,
            0.10,
        )
        .unwrap();

        assert_eq!(pool.config.pool_id, "pcoin_usdc_stable");
        assert_eq!(pool.config.token_a, "P-COIN");
        assert_eq!(pool.config.token_b, "USDC");
        assert_eq!(pool.config.fee_tier, 0.0005);
        assert_eq!(pool.config.amplification, 100.0);
        assert_eq!(pool.config.apr_rate, 0.10);
    }

    #[test]
    fn test_stable_pool_slippage_vs_constant() {
        // Normal constant product pool
        let mut cp = LiquidityPool::new(
            "pcoin_usdc".to_string(),
            "P-COIN".to_string(),
            "USDC".to_string(),
            0.003,
            "REWARD".to_string(),
            100000.0,
            0.10,
        );
        cp.add_liquidity("lp1".to_string(), 100000.0, 100000.0, 30)
            .unwrap();

        // Stable pool with amplification
        let mut sp = StableLiquidityPool::new(
            "pcoin_usdc_stable".to_string(),
            "P-COIN".to_string(),
            "USDC".to_string(),
            0.0005,
            100.0,
            "REWARD".to_string(),
            100000.0,
            0.10,
        )
        .unwrap();
        sp.add_liquidity("lp1".to_string(), 100000.0, 100000.0, 30)
            .unwrap();

        // Swap 10k P-COIN in each pool
        let out_cp = cp.calculate_swap_output("P-COIN", 10000.0).unwrap();
        let out_sp = sp.calculate_swap_output("P-COIN", 10000.0).unwrap();

        // Stable pool should have lower slippage => higher output
        assert!(out_sp > out_cp);

        // Execute swap and ensure reserves update
        let o1 = sp.swap("P-COIN", 10000.0).unwrap();
        assert!((o1 - out_sp).abs() < 1e-8);
        let (ra, rb) = sp.get_reserves();
        assert!(ra > 100000.0 && rb < 100000.0);
    }

    #[test]
    fn test_stable_pool_liquidity_lock() {
        let mut sp = StableLiquidityPool::new(
            "pcoin_usdc_stable".to_string(),
            "P-COIN".to_string(),
            "USDC".to_string(),
            0.0005,
            50.0,
            "REWARD".to_string(),
            100000.0,
            0.10,
        )
        .unwrap();

        sp.add_liquidity("lp1".to_string(), 10_000.0, 10_000.0, 30)
            .unwrap();

        // Removing immediately should be blocked by lock
        let locked = sp.remove_liquidity("lp1");
        assert!(locked.is_err());

        // Fast-forward by adjusting start_time to past lock
        {
            let pos = sp.positions.get_mut("lp1").unwrap();
            pos.start_time = pos.start_time - chrono::Duration::days(pos.duration_days + 1);
        }

        let ok = sp.remove_liquidity("lp1");
        assert!(ok.is_ok());
    }
}
