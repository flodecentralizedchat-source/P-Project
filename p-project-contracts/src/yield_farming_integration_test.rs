#[cfg(test)]
mod tests {
    use super::super::{
        liquidity_pool::{LiquidityPool, LiquidityPoolError},
        token::PProjectToken,
    };
    use chrono::{Duration, Utc};

    #[test]
    fn test_complete_yield_farming_workflow() {
        // Step 1: Create tokens
        let mut token_a = PProjectToken::new(1000000.0, 0.01, 0.005);
        let mut token_b = PProjectToken::new(1000000.0, 0.01, 0.005);
        let mut reward_token = PProjectToken::new(1000000.0, 0.01, 0.005);

        // Step 2: Initialize user balances
        let users = vec!["user1", "user2", "user3"];
        let initial_balance = 100000.0;
        
        for user in &users {
            let allocations_a = vec![(user.to_string(), initial_balance)];
            token_a.initialize_distribution(allocations_a);
            
            let allocations_b = vec![(user.to_string(), initial_balance)];
            token_b.initialize_distribution(allocations_b);
            
            let allocations_reward = vec![(user.to_string(), initial_balance)];
            reward_token.initialize_distribution(allocations_reward);
        }

        // Step 3: Create liquidity pool
        let mut pool = LiquidityPool::new(
            "pool1".to_string(),
            "TOKENA".to_string(),
            "TOKENB".to_string(),
            0.003, // 0.3% fee
            "REWARD".to_string(),
            50000.0, // 50,000 reward tokens allocated
            0.15, // 15% APR
        );

        // Step 4: Users add liquidity
        let liquidity_amounts = vec![
            (10000.0, 20000.0), // user1
            (15000.0, 30000.0), // user2
            (20000.0, 40000.0), // user3
        ];

        for (i, (token_a_amount, token_b_amount)) in liquidity_amounts.iter().enumerate() {
            let user_id = users[i];
            let result = pool.add_liquidity(
                user_id.to_string(),
                *token_a_amount,
                *token_b_amount,
                90, // 90 days
            );
            assert!(result.is_ok());
        }

        // Step 5: Verify pool state after adding liquidity
        assert_eq!(pool.total_token_a, 45000.0); // 10000 + 15000 + 20000
        assert_eq!(pool.total_token_b, 90000.0); // 20000 + 30000 + 40000
        assert_eq!(pool.liquidity_positions.len(), 3);

        // Step 6: Execute some swaps
        let swap_result = pool.swap("TOKENA", 5000.0);
        assert!(swap_result.is_ok());
        let output_b = swap_result.unwrap();
        assert!(output_b > 0.0);

        let swap_result = pool.swap("TOKENB", 3000.0);
        assert!(swap_result.is_ok());
        let output_a = swap_result.unwrap();
        assert!(output_a > 0.0);

        // Step 7: Check pool stats
        let stats = pool.get_pool_stats();
        assert_eq!(stats.total_providers, 3);
        assert_eq!(stats.total_volume, 8000.0); // 5000 + 3000
        assert!(stats.total_fees > 0.0);
        assert_eq!(stats.apr_rate, 0.15);

        // Step 8: Update rewards for users
        for user in &users {
            let result = pool.update_rewards(user);
            assert!(result.is_ok());
        }

        // Step 9: Check claimable rewards
        for user in &users {
            let claimable = pool.get_claimable_rewards(user);
            assert!(claimable.is_ok());
            let amount = claimable.unwrap();
            assert!(amount >= 0.0);
        }

        // Step 10: Users claim rewards
        for user in &users {
            let result = pool.claim_rewards(user);
            assert!(result.is_ok());
            let claimed_amount = result.unwrap();
            assert!(claimed_amount >= 0.0);
        }

        // Step 11: Distribute fees
        let fee_distribution = pool.distribute_fees();
        assert!(fee_distribution.is_ok());
        let fee_map = fee_distribution.unwrap();
        assert_eq!(fee_map.len(), 3);

        // Step 12: Users with more liquidity should get more fees
        let user1_fees = fee_map.get("user1").unwrap();
        let user3_fees = fee_map.get("user3").unwrap();
        assert!(user3_fees > user1_fees);

        // Step 13: Users remove liquidity
        for user in &users {
            let result = pool.remove_liquidity(user);
            assert!(result.is_ok());
        }

        // Step 14: Verify pool is empty
        assert!(pool.total_liquidity.abs() < 1e-10); // Use epsilon comparison for floating point
        assert!(pool.total_token_a.abs() < 1e-10);
        assert!(pool.total_token_b.abs() < 1e-10);
        assert_eq!(pool.liquidity_positions.len(), 0);
        assert!(pool.k_constant.abs() < 1e-10);

        println!("Complete yield farming workflow test passed!");
    }

    #[test]
    fn test_multiple_pools_integration() {
        // Create multiple pools
        let mut pool1 = LiquidityPool::new(
            "eth_usdc_pool".to_string(),
            "ETH".to_string(),
            "USDC".to_string(),
            0.003,
            "REWARD".to_string(),
            25000.0,
            0.12,
        );

        let mut pool2 = LiquidityPool::new(
            "btc_usdt_pool".to_string(),
            "BTC".to_string(),
            "USDT".to_string(),
            0.003,
            "REWARD".to_string(),
            25000.0,
            0.10,
        );

        // Add liquidity to both pools
        let result1 = pool1.add_liquidity(
            "user1".to_string(),
            10.0,    // 10 ETH
            20000.0, // 20000 USDC
            30,
        );
        assert!(result1.is_ok());

        let result2 = pool2.add_liquidity(
            "user1".to_string(),
            1.0,     // 1 BTC
            50000.0, // 50000 USDT
            30,
        );
        assert!(result2.is_ok());

        // Execute swaps in both pools
        let swap1 = pool1.swap("ETH", 1.0);
        assert!(swap1.is_ok());

        let swap2 = pool2.swap("BTC", 0.1);
        assert!(swap2.is_ok());

        // Check pool stats
        let stats1 = pool1.get_pool_stats();
        let stats2 = pool2.get_pool_stats();

        assert_eq!(stats1.total_volume, 1.0);
        assert_eq!(stats2.total_volume, 0.1);

        // Check APR rates are different
        assert_eq!(stats1.apr_rate, 0.12);
        assert_eq!(stats2.apr_rate, 0.10);

        println!("Multiple pools integration test passed!");
    }

    #[test]
    fn test_yield_calculation_accuracy() {
        let pool = LiquidityPool::new(
            "test_pool".to_string(),
            "TOKENA".to_string(),
            "TOKENB".to_string(),
            0.003,
            "REWARD".to_string(),
            10000.0,
            0.10, // 10% APR
        );

        // Calculate projected yield for 1000 tokens over 365 days
        let projected_yield = pool.calculate_projected_yield(1000.0, 365.0);
        
        // With 10% APR and daily compounding, we should get approximately 10%
        // The exact value with daily compounding is: 1000 * (1 + 0.10/365)^365 - 1000
        // Which is approximately 105.15
        assert!(projected_yield > 100.0);
        assert!(projected_yield < 110.0);

        println!("Yield calculation accuracy test passed! Projected yield: {}", projected_yield);
    }

    #[test]
    fn test_error_handling() {
        let mut pool = LiquidityPool::new(
            "test_pool".to_string(),
            "TOKENA".to_string(),
            "TOKENB".to_string(),
            0.003,
            "REWARD".to_string(),
            10000.0,
            0.10,
        );

        // Try to remove liquidity for non-existent user
        let result = pool.remove_liquidity("nonexistent_user");
        assert_eq!(result, Err(LiquidityPoolError::UserNotInPool));

        // Try to update rewards for non-existent user
        let result = pool.update_rewards("nonexistent_user");
        assert_eq!(result, Err(LiquidityPoolError::UserNotInPool));

        // Try to claim rewards for non-existent user
        let result = pool.claim_rewards("nonexistent_user");
        assert_eq!(result, Err(LiquidityPoolError::UserNotInPool));

        // Try to get claimable rewards for non-existent user
        let result = pool.get_claimable_rewards("nonexistent_user");
        assert_eq!(result, Err(LiquidityPoolError::UserNotInPool));

        println!("Error handling test passed!");
    }
}