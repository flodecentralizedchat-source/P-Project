use super::token::{PProjectToken, TokenError};

#[test]
fn test_token_initialization() {
    let total_supply = 350000000.0; // 350M tokens
    let burn_rate = 0.01; // 1% burn rate
    let reward_rate = 0.005; // 0.5% reward rate
    
    let token = PProjectToken::new(total_supply, burn_rate, reward_rate);
    
    assert_eq!(token.get_total_supply(), total_supply);
    assert_eq!(token.get_max_transfer_limit(), total_supply * 0.05);
}

#[test]
fn test_token_transfer() {
    let total_supply = 350000000.0;
    let burn_rate = 0.01;
    let reward_rate = 0.005;
    
    let mut token = PProjectToken::new(total_supply, burn_rate, reward_rate);
    
    // Initialize some balances
    let allocations = vec![
        ("user1".to_string(), 10000.0),
        ("user2".to_string(), 5000.0),
    ];
    token.initialize_distribution(allocations);
    
    // Transfer tokens
    let result = token.transfer("user1", "user2", 1000.0);
    assert!(result.is_ok());
    
    // Check that balances changed (exact values are hard to predict due to rewards)
    assert!(token.get_balance("user1") < 10000.0);
    assert!(token.get_balance("user2") > 5000.0);
}

#[test]
fn test_liquidity_locking() {
    let mut token = PProjectToken::new(350000000.0, 0.01, 0.005);
    
    let pool_id = "pool1".to_string();
    let user_id = "user1";
    let amount = 10000.0;
    
    // Add some balance to user first
    let allocations = vec![(user_id.to_string(), amount * 2.0)];
    token.initialize_distribution(allocations);
    
    // Add liquidity
    let add_result = token.add_liquidity(pool_id.clone(), user_id, amount);
    assert!(add_result.is_ok());
    
    // Check pool liquidity
    assert_eq!(token.get_pool_liquidity(&pool_id), amount);
    
    // Lock liquidity
    let lock_result = token.lock_liquidity(pool_id.clone(), amount);
    assert!(lock_result.is_ok());
    
    // Check that liquidity is locked
    assert!(token.is_liquidity_locked(&pool_id));
    
    // Try to remove locked liquidity (should fail)
    let remove_result = token.remove_liquidity(pool_id.clone(), user_id, amount);
    assert!(remove_result.is_err());
    assert_eq!(remove_result.unwrap_err(), TokenError::LiquidityLocked);
    
    // Unlock liquidity
    let unlock_result = token.unlock_liquidity(&pool_id);
    assert!(unlock_result.is_ok());
    
    // Now we should be able to remove liquidity
    let remove_result = token.remove_liquidity(pool_id.clone(), user_id, amount);
    assert!(remove_result.is_ok());
    
    // Check that liquidity is no longer in pool
    assert_eq!(token.get_pool_liquidity(&pool_id), 0.0);
}