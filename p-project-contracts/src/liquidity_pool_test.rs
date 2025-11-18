#[cfg(test)]
mod tests {
    use super::super::liquidity_pool::{LiquidityMechanisms, LiquidityPool, LiquidityPoolError};
    use chrono::Utc;
    use std::collections::HashMap;

    #[test]
    fn test_create_liquidity_pool() {
        let pool = LiquidityPool::new(
            "pool1".to_string(),
            "TOKENA".to_string(),
            "TOKENB".to_string(),
            0.003, // 0.3% fee
            "REWARD".to_string(),
            100000.0, // 100,000 reward tokens
            0.12,     // 12% APR
        );

        assert_eq!(pool.config.pool_id, "pool1");
        assert_eq!(pool.config.token_a, "TOKENA");
        assert_eq!(pool.config.token_b, "TOKENB");
        assert_eq!(pool.config.fee_tier, 0.003);
        assert_eq!(pool.config.reward_token, "REWARD");
        assert_eq!(pool.config.total_reward_allocation, 100000.0);
        assert_eq!(pool.config.apr_rate, 0.12);
        assert_eq!(pool.total_liquidity, 0.0);
        assert_eq!(pool.total_token_a, 0.0);
        assert_eq!(pool.total_token_b, 0.0);
    }

    #[test]
    fn test_add_liquidity() {
        let mut pool = LiquidityPool::new(
            "pool1".to_string(),
            "TOKENA".to_string(),
            "TOKENB".to_string(),
            0.003,
            "REWARD".to_string(),
            100000.0,
            0.12,
        );

        let result = pool.add_liquidity(
            "user1".to_string(),
            1000.0, // 1000 TOKENA
            2000.0, // 2000 TOKENB
            30,     // 30 days
        );

        assert!(result.is_ok());
        let liquidity_amount = result.unwrap();
        assert_eq!(liquidity_amount, (1000.0f64 * 2000.0f64).sqrt());
        assert_eq!(pool.total_token_a, 1000.0);
        assert_eq!(pool.total_token_b, 2000.0);
        assert_eq!(pool.total_liquidity, liquidity_amount);
        assert_eq!(pool.k_constant, 1000.0 * 2000.0);

        // Check position was created
        let position = pool.get_position("user1").unwrap();
        assert_eq!(position.user_id, "user1");
        assert_eq!(position.liquidity_amount, liquidity_amount);
        assert_eq!(position.token_a_amount, 1000.0);
        assert_eq!(position.token_b_amount, 2000.0);
        assert_eq!(position.duration_days, 30);
    }

    #[test]
    fn test_add_liquidity_enforces_minimum_lock() {
        let mut pool = LiquidityPool::new(
            "pool1".to_string(),
            "TOKENA".to_string(),
            "TOKENB".to_string(),
            0.003,
            "REWARD".to_string(),
            100000.0,
            0.12,
        );

        pool.add_liquidity("user1".to_string(), 1000.0, 2000.0, 7)
            .unwrap();
        let position = pool.get_position("user1").unwrap();
        assert!(position.duration_days >= pool.mechanisms.lp_lock_days);
    }

    #[test]
    fn test_add_liquidity_invalid_amount() {
        let mut pool = LiquidityPool::new(
            "pool1".to_string(),
            "TOKENA".to_string(),
            "TOKENB".to_string(),
            0.003,
            "REWARD".to_string(),
            100000.0,
            0.12,
        );

        // Test negative amount
        let result = pool.add_liquidity("user1".to_string(), -1000.0, 2000.0, 30);
        assert_eq!(result, Err(LiquidityPoolError::InvalidAmount));

        // Test zero amount
        let result = pool.add_liquidity("user1".to_string(), 0.0, 2000.0, 30);
        assert_eq!(result, Err(LiquidityPoolError::InvalidAmount));

        // Test negative second token amount
        let result = pool.add_liquidity("user1".to_string(), 1000.0, -2000.0, 30);
        assert_eq!(result, Err(LiquidityPoolError::InvalidAmount));
    }

    #[test]
    fn test_add_liquidity_invalid_duration() {
        let mut pool = LiquidityPool::new(
            "pool1".to_string(),
            "TOKENA".to_string(),
            "TOKENB".to_string(),
            0.003,
            "REWARD".to_string(),
            100000.0,
            0.12,
        );

        let result = pool.add_liquidity(
            "user1".to_string(),
            1000.0,
            2000.0,
            0, // Invalid duration
        );
        assert_eq!(result, Err(LiquidityPoolError::InvalidDuration));

        let result = pool.add_liquidity(
            "user1".to_string(),
            1000.0,
            2000.0,
            -30, // Negative duration
        );
        assert_eq!(result, Err(LiquidityPoolError::InvalidDuration));
    }

    #[test]
    fn test_remove_liquidity() {
        let mut pool = LiquidityPool::new(
            "pool1".to_string(),
            "TOKENA".to_string(),
            "TOKENB".to_string(),
            0.003,
            "REWARD".to_string(),
            100000.0,
            0.12,
        );

        // Add liquidity first
        pool.add_liquidity("user1".to_string(), 1000.0, 2000.0, 30)
            .unwrap();

        // Fast-forward lock by adjusting start_time for testing removal
        {
            let pos = pool.liquidity_positions.get_mut("user1").unwrap();
            pos.start_time = pos.start_time - chrono::Duration::days(pos.duration_days + 1);
        }

        // Remove liquidity
        let result = pool.remove_liquidity("user1");
        assert!(result.is_ok());
        let (token_a_return, token_b_return) = result.unwrap();
        assert_eq!(token_a_return, 1000.0);
        assert_eq!(token_b_return, 2000.0);
        assert_eq!(pool.total_token_a, 0.0);
        assert_eq!(pool.total_token_b, 0.0);
        assert_eq!(pool.total_liquidity, 0.0);
        assert_eq!(pool.k_constant, 0.0);

        // Check position was removed
        assert!(pool.get_position("user1").is_none());
    }

    #[test]
    fn test_remove_liquidity_user_not_in_pool() {
        let mut pool = LiquidityPool::new(
            "pool1".to_string(),
            "TOKENA".to_string(),
            "TOKENB".to_string(),
            0.003,
            "REWARD".to_string(),
            100000.0,
            0.12,
        );

        let result = pool.remove_liquidity("user1");
        assert_eq!(result, Err(LiquidityPoolError::UserNotInPool));
    }

    #[test]
    fn test_remove_liquidity_locked() {
        let mut pool = LiquidityPool::new(
            "pool1".to_string(),
            "TOKENA".to_string(),
            "TOKENB".to_string(),
            0.003,
            "REWARD".to_string(),
            100000.0,
            0.12,
        );

        pool.add_liquidity("user1".to_string(), 1000.0, 2000.0, 30)
            .unwrap();

        // Removing immediately should be blocked by lock
        let result = pool.remove_liquidity("user1");
        assert_eq!(result, Err(LiquidityPoolError::LiquidityLocked));
    }

    #[test]
    fn test_deep_liquidity_reduces_slippage() {
        // Shallow pool
        let mut shallow = LiquidityPool::new(
            "shallow".to_string(),
            "A".to_string(),
            "B".to_string(),
            0.003,
            "REWARD".to_string(),
            1000.0,
            0.10,
        );
        shallow
            .add_liquidity("lp".to_string(), 1_000.0, 1_000.0, 7)
            .unwrap();

        // Deep pool
        let mut deep = LiquidityPool::new(
            "deep".to_string(),
            "A".to_string(),
            "B".to_string(),
            0.003,
            "REWARD".to_string(),
            1000.0,
            0.10,
        );
        deep.add_liquidity("lp".to_string(), 1_000_000.0, 1_000_000.0, 7)
            .unwrap();

        let input = 10_000.0;
        let out_shallow = shallow.calculate_swap_output("A", input).unwrap();
        let out_deep = deep.calculate_swap_output("A", input).unwrap();

        // Deep pool should have materially lower slippage => higher output
        assert!(out_deep > out_shallow);
    }

    #[test]
    fn test_calculate_swap_output() {
        let mut pool = LiquidityPool::new(
            "pool1".to_string(),
            "TOKENA".to_string(),
            "TOKENB".to_string(),
            0.003, // 0.3% fee
            "REWARD".to_string(),
            100000.0,
            0.12,
        );

        // Add initial liquidity
        pool.add_liquidity(
            "user1".to_string(),
            1000.0, // 1000 TOKENA
            2000.0, // 2000 TOKENB
            30,
        )
        .unwrap();

        // Calculate swap output
        let result = pool.calculate_swap_output("TOKENA", 100.0);
        assert!(result.is_ok());
        let output_amount = result.unwrap();

        // With 0.3% fee, we should get less than the simple ratio would suggest
        // Simple ratio would be: 100 * 2000 / 1000 = 200
        // With fee: should be less than 200
        assert!(output_amount < 200.0);
        assert!(output_amount > 0.0);
    }

    #[test]
    fn test_swap() {
        let mut pool = LiquidityPool::new(
            "pool1".to_string(),
            "TOKENA".to_string(),
            "TOKENB".to_string(),
            0.003, // 0.3% fee
            "REWARD".to_string(),
            100000.0,
            0.12,
        );

        // Add initial liquidity
        pool.add_liquidity("user1".to_string(), 1000.0, 2000.0, 30)
            .unwrap();

        let initial_volume = pool.total_volume;
        let initial_fees = pool.total_fees;

        // Execute swap
        let result = pool.swap("TOKENA", 100.0);
        assert!(result.is_ok());
        let output_amount = result.unwrap();

        // Check reserves updated
        assert_eq!(pool.total_token_a, 1100.0); // 1000 + 100
        assert!(pool.total_token_b < 2000.0); // 2000 - output_amount

        // Check volume and fees tracking
        assert_eq!(pool.total_volume, initial_volume + 100.0);
        assert_eq!(pool.total_fees, initial_fees + (100.0 * 0.003));
    }

    #[test]
    fn test_swap_auto_liquidity_growth() {
        let mut pool = LiquidityPool::new(
            "pool1".to_string(),
            "TOKENA".to_string(),
            "TOKENB".to_string(),
            0.003,
            "REWARD".to_string(),
            100000.0,
            0.12,
        );

        let mut mech = LiquidityMechanisms::default();
        mech.auto_liquidity_rate_bps = 5000; // 50% of fees recycled
        pool.set_mechanisms(mech);

        pool.add_liquidity("user1".to_string(), 1000.0, 2000.0, 30)
            .unwrap();

        let before_total_liquidity = pool.total_liquidity;
        pool.swap("TOKENA", 100.0).unwrap();
        assert!(pool.total_liquidity > before_total_liquidity);
    }

    #[test]
    fn test_swap_with_slippage_success_and_failure() {
        let mut pool = LiquidityPool::new(
            "pool1".to_string(),
            "TOKENA".to_string(),
            "TOKENB".to_string(),
            0.003,
            "REWARD".to_string(),
            100000.0,
            0.12,
        );

        pool.add_liquidity("user1".to_string(), 1000.0, 2000.0, 30)
            .unwrap();

        let expected = pool.calculate_swap_output("TOKENA", 100.0).unwrap();
        let result = pool.swap_with_slippage("TOKENA", 100.0, expected, Some(10));
        assert!(result.is_ok());

        let mut pool_bad = LiquidityPool::new(
            "pool2".to_string(),
            "TOKENA".to_string(),
            "TOKENB".to_string(),
            0.003,
            "REWARD".to_string(),
            100000.0,
            0.12,
        );
        pool_bad
            .add_liquidity("user2".to_string(), 1000.0, 2000.0, 30)
            .unwrap();

        let expected = pool_bad.calculate_swap_output("TOKENA", 100.0).unwrap();
        let result = pool_bad.swap_with_slippage("TOKENA", 100.0, expected * 1.1, Some(10));
        assert_eq!(result, Err(LiquidityPoolError::SlippageExceeded));
    }

    #[test]
    fn test_calculate_yield() {
        let mut pool = LiquidityPool::new(
            "pool1".to_string(),
            "TOKENA".to_string(),
            "TOKENB".to_string(),
            0.003,
            "REWARD".to_string(),
            100000.0,
            0.12, // 12% APR
        );

        // Add liquidity
        pool.add_liquidity("user1".to_string(), 1000.0, 2000.0, 30)
            .unwrap();

        // Get initial yield (should be 0 or very small since no time has passed)
        let position = pool.get_position("user1").unwrap();
        let initial_yield = pool.calculate_yield(position);
        assert!(initial_yield >= 0.0);

        // For a more meaningful test, we would need to mock time or sleep
        // This test just verifies the function doesn't crash
    }

    #[test]
    fn test_calculate_projected_yield() {
        let pool = LiquidityPool::new(
            "pool1".to_string(),
            "TOKENA".to_string(),
            "TOKENB".to_string(),
            0.003,
            "REWARD".to_string(),
            100000.0,
            0.12, // 12% APR
        );

        // Calculate projected yield for 1000 liquidity over 365 days at 12% APR
        let projected_yield = pool.calculate_projected_yield(1000.0, 365.0);

        // With daily compounding, we should get approximately 12% yield
        // But not exactly due to compounding effects
        assert!(projected_yield > 110.0); // Should be more than 11% (simple interest)
        assert!(projected_yield < 130.0); // Should be less than 13% (reasonable compounding)
    }

    #[test]
    fn test_update_rewards() {
        let mut pool = LiquidityPool::new(
            "pool1".to_string(),
            "TOKENA".to_string(),
            "TOKENB".to_string(),
            0.003,
            "REWARD".to_string(),
            100000.0,
            0.12,
        );

        // Add liquidity
        pool.add_liquidity("user1".to_string(), 1000.0, 2000.0, 30)
            .unwrap();

        // Update rewards
        let result = pool.update_rewards("user1");
        assert!(result.is_ok());

        // Check that accumulated rewards increased
        let position = pool.get_position("user1").unwrap();
        assert!(position.accumulated_rewards >= 0.0);
    }

    #[test]
    fn test_update_rewards_user_not_in_pool() {
        let mut pool = LiquidityPool::new(
            "pool1".to_string(),
            "TOKENA".to_string(),
            "TOKENB".to_string(),
            0.003,
            "REWARD".to_string(),
            100000.0,
            0.12,
        );

        let result = pool.update_rewards("user1");
        assert_eq!(result, Err(LiquidityPoolError::UserNotInPool));
    }

    #[test]
    fn test_claim_rewards() {
        let mut pool = LiquidityPool::new(
            "pool1".to_string(),
            "TOKENA".to_string(),
            "TOKENB".to_string(),
            0.003,
            "REWARD".to_string(),
            100000.0,
            0.12,
        );

        // Add liquidity
        pool.add_liquidity("user1".to_string(), 1000.0, 2000.0, 30)
            .unwrap();

        // Update rewards to have some accumulated
        pool.update_rewards("user1").unwrap();

        // Claim rewards
        let result = pool.claim_rewards("user1");
        assert!(result.is_ok());

        let claimed_amount = result.unwrap();
        assert!(claimed_amount >= 0.0);

        // Check that claimed rewards are updated
        let position = pool.get_position("user1").unwrap();
        assert_eq!(position.claimed_rewards, claimed_amount);
    }

    #[test]
    fn test_get_claimable_rewards() {
        let mut pool = LiquidityPool::new(
            "pool1".to_string(),
            "TOKENA".to_string(),
            "TOKENB".to_string(),
            0.003,
            "REWARD".to_string(),
            100000.0,
            0.12,
        );

        // Add liquidity
        pool.add_liquidity("user1".to_string(), 1000.0, 2000.0, 30)
            .unwrap();

        // Get claimable rewards
        let result = pool.get_claimable_rewards("user1");
        assert!(result.is_ok());

        let claimable_amount = result.unwrap();
        assert!(claimable_amount >= 0.0);
    }

    #[test]
    fn test_distribute_fees() {
        let mut pool = LiquidityPool::new(
            "pool1".to_string(),
            "TOKENA".to_string(),
            "TOKENB".to_string(),
            0.003,
            "REWARD".to_string(),
            100000.0,
            0.12,
        );

        // Add liquidity from multiple users
        pool.add_liquidity("user1".to_string(), 1000.0, 2000.0, 30)
            .unwrap();

        pool.add_liquidity("user2".to_string(), 2000.0, 4000.0, 30)
            .unwrap();

        // Execute some swaps to generate fees
        pool.swap("TOKENA", 100.0).unwrap();
        pool.swap("TOKENB", 200.0).unwrap();

        // Distribute fees
        let result = pool.distribute_fees();
        assert!(result.is_ok());

        let fee_distribution = result.unwrap();
        assert_eq!(fee_distribution.len(), 2); // Two users

        // Check that both users get some fees
        assert!(fee_distribution.get("user1").is_some());
        assert!(fee_distribution.get("user2").is_some());

        // User2 should get more fees since they provided more liquidity
        let user1_fees = fee_distribution.get("user1").unwrap();
        let user2_fees = fee_distribution.get("user2").unwrap();
        assert!(user2_fees > user1_fees);
    }

    #[test]
    fn test_get_pool_stats() {
        let mut pool = LiquidityPool::new(
            "pool1".to_string(),
            "TOKENA".to_string(),
            "TOKENB".to_string(),
            0.003,
            "REWARD".to_string(),
            100000.0,
            0.12,
        );

        // Add liquidity from multiple users
        pool.add_liquidity("user1".to_string(), 1000.0, 2000.0, 30)
            .unwrap();

        pool.add_liquidity("user2".to_string(), 2000.0, 4000.0, 30)
            .unwrap();

        // Execute some swaps to generate volume and fees
        pool.swap("TOKENA", 100.0).unwrap();
        pool.swap("TOKENB", 200.0).unwrap();

        // Get pool stats
        let stats = pool.get_pool_stats();

        assert_eq!(stats.total_providers, 2);
        assert_eq!(stats.apr_rate, 0.12);
        assert_eq!(stats.total_volume, 300.0); // 100 + 200
        assert!(stats.total_fees > 0.0);
        assert!(stats.total_liquidity > 0.0);
        assert!(stats.avg_liquidity > 0.0);
    }

    #[test]
    fn test_pcoin_usdc_pool_creation() {
        let pool = LiquidityPool::new(
            "pcoin_usdc".to_string(),
            "P-COIN".to_string(),
            "USDC".to_string(),
            0.003,
            "REWARD".to_string(),
            50000.0,
            0.10,
        );
        assert_eq!(pool.config.pool_id, "pcoin_usdc");
        assert_eq!(pool.config.token_a, "P-COIN");
        assert_eq!(pool.config.token_b, "USDC");
    }
}
