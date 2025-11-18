#[cfg(test)]
mod tests {
    use super::super::liquidity_pool::LiquidityPool;
    use super::super::stable_liquidity_pool::StableLiquidityPool;
    use super::super::router::{AutoProvisionConfig, PoolKind, Router, RouterError};

    #[test]
    fn picks_best_route_between_constant_and_stable() {
        let mut router = Router::new();

        let mut cp = LiquidityPool::new(
            "cp_usdc".to_string(),
            "P-COIN".to_string(),
            "USDC".to_string(),
            0.003,
            "RWD".to_string(),
            100_000.0,
            0.10,
        );
        cp.add_liquidity("lp1".into(), 1_000_000.0, 1_000_000.0, 30).unwrap();
        router.register_constant_pool("cp".to_string(), cp);

        let mut sp = StableLiquidityPool::new(
            "sp_usdc".to_string(),
            "P-COIN".to_string(),
            "USDC".to_string(),
            0.0005,
            100.0,
            "RWD".to_string(),
            100_000.0,
            0.10,
        )
        .unwrap();
        sp.add_liquidity("lp2".into(), 1_000_000.0, 1_000_000.0, 30)
            .unwrap();
        router.register_stable_pool("sp".to_string(), sp);

        let q = router
            .best_quote("P-COIN", "USDC", 10_000.0)
            .expect("quote must exist");

        // With lower fee and amplification, stable should typically provide better output.
        assert_eq!(q.kind, PoolKind::Stable);
    }

    #[test]
    fn auto_provisions_when_liquidity_insufficient_then_swaps() {
        let mut router = Router::new();

        let cp = LiquidityPool::new(
            "cp_usdc".to_string(),
            "P-COIN".to_string(),
            "USDC".to_string(),
            0.003,
            "RWD".to_string(),
            100_000.0,
            0.10,
        );
        // No initial liquidity registered
        router.register_constant_pool("cp".to_string(), cp);

        // Without auto-provision, there is no route
        let no_route = router.best_quote("P-COIN", "USDC", 10_000.0);
        assert_eq!(no_route.err(), Some(RouterError::NoRoute));

        // Enable auto-provision with a reserve floor large enough
        let cfg = AutoProvisionConfig {
            lp_user: "router_lp".to_string(),
            min_reserve_per_side: 100_000.0,
            duration_days: 30,
        };

        let res = router.swap_best_route("P-COIN", "USDC", 10_000.0, 1.0, Some(cfg));
        assert!(res.is_ok(), "swap should succeed after auto-provision");

        // Verify pool now has reserves > 0
        let p = router.get_constant_pool("cp").unwrap();
        let (ra, rb) = p.get_reserves();
        assert!(ra >= 100_000.0 && rb >= 100_000.0);
    }
}

