use super::token::{PProjectToken, TokenError};
use chrono::{Duration, Utc};

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

#[test]
fn test_scheduled_burns_execute_when_due_and_can_be_toggled() {
    let mut token = PProjectToken::new(350000000.0, 0.01, 0.005);

    let past = Utc::now().naive_utc() - Duration::minutes(5);
    token.add_scheduled_burn(past, 1_000.0);
    let supply_before = token.get_total_supply();

    let burned = token.execute_scheduled_burns().unwrap();
    assert_eq!(burned, 1_000.0);
    assert_eq!(token.get_total_supply(), supply_before - 1_000.0);

    let future = Utc::now().naive_utc() + Duration::hours(1);
    token.add_scheduled_burn(future, 2_000.0);
    token.set_burn_schedule_enabled(false);
    let burned = token.execute_scheduled_burns().unwrap();
    assert_eq!(burned, 0.0);

    token.set_burn_schedule_enabled(true);
    let burned = token.execute_scheduled_burns().unwrap();
    assert_eq!(burned, 2_000.0);
}

#[test]
fn test_milestone_burns_trigger_on_targets() {
    let mut token = PProjectToken::new(350000000.0, 0.01, 0.005);
    token.initialize_distribution(vec![
        ("user1".to_string(), 10_000.0),
        ("user2".to_string(), 5_000.0),
        ("user3".to_string(), 3_000.0),
    ]);

    token.add_milestone_burn(
        "holder_target".to_string(),
        "holders_count".to_string(),
        3.0,
        1_000.0,
    );
    token.add_milestone_burn(
        "transaction_target".to_string(),
        "transactions_count".to_string(),
        3.0,
        500.0,
    );
    token.add_milestone_burn(
        "supply_target".to_string(),
        "supply_reduction".to_string(),
        0.0005,
        1_500.0,
    );

    let first_burn = token.check_milestone_burns().unwrap();
    assert_eq!(first_burn, 1_000.0);
    assert!(token.get_milestone_burns()[0].executed);
    assert!(!token.get_milestone_burns()[1].executed);
    assert!(!token.get_milestone_burns()[2].executed);

    for _ in 0..3 {
        token.transfer("user1", "user2", 500.0).unwrap();
    }

    let second_burn = token.check_milestone_burns().unwrap();
    assert_eq!(second_burn, 500.0);
    assert!(token.get_milestone_burns()[1].executed);
    assert!(!token.get_milestone_burns()[2].executed);

    token.add_scheduled_burn(Utc::now().naive_utc() - Duration::minutes(1), 4_000.0);
    token.execute_scheduled_burns().unwrap();

    let third_burn = token.check_milestone_burns().unwrap();
    assert_eq!(third_burn, 1_500.0);
    assert!(token.get_milestone_burns()[2].executed);
}
