#[cfg(test)]
mod tests {
    use super::super::community_liquidity::CommunityLiquidityProgram;
    use super::super::cross_chain_liquidity::{Chain, CrossChainLiquidityManager};

    fn approx_eq(a: f64, b: f64, eps: f64) -> bool {
        (a - b).abs() <= eps
    }

    #[test]
    fn test_deploy_and_add_liquidity_across_chains() {
        let mut mgr = CrossChainLiquidityManager::new();

        // Deploy P-COIN/USDC stable pairs on Ethereum, BSC, Solana
        let chains = vec![Chain::Ethereum, Chain::BSC, Chain::Solana];
        let ids = mgr
            .deploy_stable_pair_across_chains(
                "P-COIN", "USDC", &chains, 0.0005, 100.0, "REWARD", 100_000.0, 0.10,
            )
            .unwrap();
        assert_eq!(ids.len(), 3);

        // Provide initial liquidity allocations
        // ETH: 50k/50k, BSC: 25k/25k, SOL: 25k/25k
        mgr.add_liquidity(
            Chain::Ethereum,
            &ids[0],
            "user_eth".into(),
            50_000.0,
            50_000.0,
            30,
        )
        .unwrap();
        mgr.add_liquidity(
            Chain::BSC,
            &ids[1],
            "user_bsc".into(),
            25_000.0,
            25_000.0,
            30,
        )
        .unwrap();
        mgr.add_liquidity(
            Chain::Solana,
            &ids[2],
            "user_sol".into(),
            25_000.0,
            25_000.0,
            30,
        )
        .unwrap();

        // Aggregate total liquidity across chains for the same pair
        let total = mgr.total_liquidity_for_pair("P-COIN", "USDC");
        let expected = (50_000.0_f64 * 50_000.0_f64).sqrt()
            + (25_000.0_f64 * 25_000.0_f64).sqrt()
            + (25_000.0_f64 * 25_000.0_f64).sqrt();
        assert!(approx_eq(total, expected, 1e-8));

        // Spot check reserves
        let r_eth = mgr.get_reserves(Chain::Ethereum, &ids[0]).unwrap();
        assert_eq!(r_eth, (50_000.0, 50_000.0));
    }

    #[test]
    fn test_stable_pair_helper_creates_per_chain_pools() {
        let mut mgr = CrossChainLiquidityManager::new();
        let chains = vec![Chain::Ethereum, Chain::BSC];
        let ids = mgr
            .deploy_stable_pair_across_chains(
                "P-COIN", "USDT", &chains, 0.0005, 75.0, "REWARD", 50_000.0, 0.08,
            )
            .unwrap();
        assert_eq!(ids.len(), 2);
        // Reserves should start at zero
        for (idx, ch) in chains.iter().enumerate() {
            let r = mgr.get_reserves(*ch, &ids[idx]).unwrap();
            assert_eq!(r, (0.0, 0.0));
        }
    }

    #[test]
    fn test_community_liquidity_incentives_and_leaderboard() {
        // Create program with generous early window to ensure multiplier applies
        let mut prog = CommunityLiquidityProgram::new(
            "lp-community".into(),
            "P".into(),
            1000.0, // per day
            30,     // duration
            30_000.0,
        );
        prog.set_early_window(100, 1.5);
        prog.set_chain_weight(Chain::Ethereum, 1.0);
        prog.set_chain_weight(Chain::BSC, 1.2);
        prog.set_chain_weight(Chain::Solana, 0.8);

        // Record contributions (weighted + early multiplier expected):
        // user1: 10k on ETH => 10k * 1.0 * 1.5 = 15k
        prog.record_contribution("user1".into(), Chain::Ethereum, 10_000.0);
        // user2: 8k on BSC + 4k on SOL => 8k*1.2*1.5 + 4k*0.8*1.5 = 19.2k
        prog.record_contribution("user2".into(), Chain::BSC, 8_000.0);
        prog.record_contribution("user2".into(), Chain::Solana, 4_000.0);
        // user3: 5k on ETH + 5k on BSC => 5k*1.0*1.5 + 5k*1.2*1.5 = 16.5k
        prog.record_contribution("user3".into(), Chain::Ethereum, 5_000.0);
        prog.record_contribution("user3".into(), Chain::BSC, 5_000.0);

        // 7-day leaderboard based on proportional rewards
        let board = prog.leaderboard(7.0, 3);
        assert_eq!(board.len(), 3);
        // Expect ordering: user2 > user3 > user1
        assert_eq!(board[0].0, "user2");
        assert_eq!(board[1].0, "user3");
        assert_eq!(board[2].0, "user1");
        assert!(board[0].1 > board[1].1 && board[1].1 > board[2].1);
    }
}
