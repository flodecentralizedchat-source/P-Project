use super::token::PProjectToken;
use super::treasury::{BuybackCondition, MarketSnapshot, Treasury};
use chrono::{Utc, Duration};

#[test]
fn test_dynamic_burn_rates() {
    let mut token = PProjectToken::new(350000000.0, 0.01, 0.005);
    
    // Initialize some balances
    let allocations = vec![
        ("user1".to_string(), 1000000.0),
        ("user2".to_string(), 1000000.0),
    ];
    token.initialize_distribution(allocations);
    
    // Normal transfer with base burn rate
    let initial_supply = token.get_total_supply();
    token.transfer("user1", "user2", 10000.0).unwrap();
    let supply_after_normal_transfer = token.get_total_supply();
    
    // Simulate high activity for user1
    for _ in 0..15 {
        token.transfer("user1", "user2", 1000.0).unwrap();
    }
    
    // Transfer with increased burn rate due to high activity
    let supply_before_high_activity = supply_after_normal_transfer;
    token.transfer("user1", "user2", 10000.0).unwrap();
    let supply_after_high_activity = token.get_total_supply();
    
    // The burn should be higher during high activity
    let normal_burn = initial_supply - supply_after_normal_transfer;
    let high_activity_burn = supply_before_high_activity - supply_after_high_activity;
    
    // High activity burn should be higher than normal burn
    assert!(high_activity_burn > normal_burn);
    
    println!("Normal burn: {}, High activity burn: {}", normal_burn, high_activity_burn);
}

#[test]
fn test_scheduled_burns() {
    let mut token = PProjectToken::new(350000000.0, 0.01, 0.005);
    
    // Add scheduled burns
    let now = Utc::now().naive_utc();
    let future_burn_time = now + Duration::days(1);
    let past_burn_time = now - Duration::days(1);
    
    // Add a burn that should execute now (past date)
    token.add_scheduled_burn(past_burn_time, 100000.0);
    
    // Add a burn that should not execute yet (future date)
    token.add_scheduled_burn(future_burn_time, 200000.0);
    
    // Enable burn schedule
    token.set_burn_schedule_enabled(true);
    
    // Execute scheduled burns
    let burned_amount = token.execute_scheduled_burns().unwrap();
    
    // Only the past burn should have executed
    assert_eq!(burned_amount, 100000.0);
    
    // Check that total supply decreased by the burned amount
    assert_eq!(token.get_total_supply(), 350000000.0 - 100000.0);
    
    // Execute again - no additional burns should happen
    let burned_amount = token.execute_scheduled_burns().unwrap();
    assert_eq!(burned_amount, 0.0);
}

#[test]
fn test_milestone_burns() {
    let mut token = PProjectToken::new(350000000.0, 0.01, 0.005);
    
    // Add milestone burns
    token.add_milestone_burn(
        "10k_holders".to_string(),
        "holders_count".to_string(),
        10000.0, // Target: 10,000 holders
        500000.0, // Burn 500,000 tokens when reached
    );
    
    token.add_milestone_burn(
        "100k_transactions".to_string(),
        "transactions_count".to_string(),
        100000.0, // Target: 100,000 transactions
        1000000.0, // Burn 1,000,000 tokens when reached
    );
    
    // Initialize some balances to create holders
    let allocations = vec![
        ("user1".to_string(), 1000000.0),
        ("user2".to_string(), 1000000.0),
        ("user3".to_string(), 1000000.0),
    ];
    token.initialize_distribution(allocations);
    
    // Simulate transactions to reach the milestone
    for i in 0..100001 {
        let from_user = format!("user{}", (i % 3) + 1);
        let to_user = format!("user{}", ((i + 1) % 3) + 1);
        token.transfer(&from_user, &to_user, 100.0).unwrap();
    }
    
    // Check that we've exceeded the transaction count milestone
    assert!(token.get_total_transactions() > 100000);
    
    // Execute milestone burns
    let burned_amount = token.check_milestone_burns().unwrap();
    
    // The transaction milestone burn should have executed
    assert_eq!(burned_amount, 1000000.0);
    
    // Check that total supply decreased by the burned amount
    assert_eq!(token.get_total_supply(), 350000000.0 - 1000000.0);
}

#[test]
fn test_revenue_linked_burns() {
    let mut token = PProjectToken::new(350000000.0, 0.01, 0.005);
    
    // Add revenue-linked burns
    token.add_revenue_linked_burn(
        "staking_rewards".to_string(),
        50000.0, // $50,000 in staking rewards
        10.0,    // Burn 10% of revenue
    );
    
    token.add_revenue_linked_burn(
        "partnerships".to_string(),
        100000.0, // $100,000 from partnerships
        5.0,      // Burn 5% of revenue
    );
    
    // Execute revenue-linked burns
    let burned_amount = token.execute_revenue_linked_burns().unwrap();
    
    // Calculate expected burn amount: 10% of $50,000 + 5% of $100,000 = $5,000 + $5,000 = $10,000
    let expected_burn = 50000.0 * 0.10 + 100000.0 * 0.05;
    assert_eq!(burned_amount, expected_burn);
    
    // Check that total supply decreased by the burned amount
    assert_eq!(token.get_total_supply(), 350000000.0 - expected_burn);
}

#[test]
fn test_treasury_scheduled_buybacks() {
    let mut treasury = Treasury::new();
    let mut token = PProjectToken::new(350000000.0, 0.01, 0.005);
    
    // Add funds to treasury
    treasury.add_funds("USD".to_string(), 1000000.0).unwrap();
    
    // Add scheduled buybacks
    let now = Utc::now().naive_utc();
    let past_buyback_time = now - Duration::days(1);
    
    // Add a buyback that should execute now (past date)
    treasury.add_scheduled_buyback(past_buyback_time, 100000.0, 0.01).unwrap();
    
    // Enable auto buybacks
    treasury.set_auto_buyback_enabled(true);
    
    // Execute scheduled buybacks
    let tokens_bought = treasury.execute_scheduled_buybacks(&mut token, 0.01).unwrap();
    
    // Calculate expected tokens bought: $100,000 / $0.01 = 10,000,000 tokens
    let expected_tokens = 100000.0 / 0.01;
    assert_eq!(tokens_bought, expected_tokens);
    
    // Verify treasury balance decreased
    assert_eq!(treasury.get_balance("USD"), 900000.0);
    
    // Verify buyback was recorded
    assert_eq!(treasury.get_total_buybacks(), 100000.0);
    assert_eq!(treasury.get_buyback_records().len(), 1);
}

#[test]
fn test_treasury_trigger_buybacks() {
    let mut treasury = Treasury::new();
    let mut token = PProjectToken::new(350000000.0, 0.01, 0.005);
    
    // Add funds to treasury
    treasury.add_funds("USD".to_string(), 1000000.0).unwrap();
    
    // Add buyback triggers
    treasury.add_buyback_trigger(
        "price_floor".to_string(),
        BuybackCondition::PriceBelow,
        0.005,    // Trigger when price drops to $0.005 or below
        50000.0,  // Spend $50,000 on buyback
    ).unwrap();
    
    treasury.add_buyback_trigger(
        "volume_surge".to_string(),
        BuybackCondition::VolumeSpike,
        10.0,     // Trigger when volume increases by 10% or more
        100000.0, // Spend $100,000 on buyback
    ).unwrap();
    
    // Enable auto buybacks
    treasury.set_auto_buyback_enabled(true);
    
    // Test price floor trigger using a snapshot with price below the threshold
    let price_snapshot = MarketSnapshot {
        price: 0.004,
        volume: 500_000.0,
        price_change_percentage: -2.0,
        volume_change_percentage: 0.0,
    };
    let tokens_bought = treasury.check_buyback_triggers(&mut token, &price_snapshot).unwrap();

    // Calculate expected tokens bought: $50,000 / $0.004 = 12,500,000 tokens
    let expected_tokens = 50000.0 / 0.004;
    assert_eq!(tokens_bought, expected_tokens);

    // Verify treasury balance decreased
    assert_eq!(treasury.get_balance("USD"), 950000.0);

    // Volume spike should trigger after the price-based trigger was executed
    let volume_snapshot = MarketSnapshot {
        price: 0.006,
        volume: 3_000_000.0,
        price_change_percentage: 1.5,
        volume_change_percentage: 15.0,
    };
    let tokens_bought_volume = treasury.check_buyback_triggers(&mut token, &volume_snapshot).unwrap();
    let expected_tokens_volume = 100000.0 / 0.006;
    assert_eq!(tokens_bought_volume, expected_tokens_volume);
    assert_eq!(treasury.get_balance("USD"), 850000.0);

    assert_eq!(treasury.get_buyback_records().len(), 2);
    assert_eq!(treasury.get_total_buybacks(), 150000.0);
}

#[test]
fn test_price_drop_percentage_trigger() {
    let mut treasury = Treasury::new();
    let mut token = PProjectToken::new(350000000.0, 0.01, 0.005);

    treasury.add_funds("USD".to_string(), 100000.0).unwrap();
    treasury.set_auto_buyback_enabled(true);

    treasury.add_buyback_trigger(
        "momentum".to_string(),
        BuybackCondition::PriceDrop,
        5.0,     // Trigger when price drops by 5% or more
        25000.0, // Spend $25,000 on buyback
    ).unwrap();

    let snapshot = MarketSnapshot {
        price: 0.008,
        volume: 400_000.0,
        price_change_percentage: -6.0,
        volume_change_percentage: 2.0,
    };
    let tokens_bought = treasury.check_buyback_triggers(&mut token, &snapshot).unwrap();
    let expected_tokens = 25000.0 / 0.008;
    assert_eq!(tokens_bought, expected_tokens);
    assert_eq!(treasury.get_balance("USD"), 75000.0);
}

#[test]
fn test_transaction_based_burns() {
    let mut token = PProjectToken::new(350000000.0, 0.02, 0.005); // 2% base burn rate
    
    // Initialize some balances
    let allocations = vec![
        ("user1".to_string(), 1000000.0),
        ("user2".to_string(), 1000000.0),
    ];
    token.initialize_distribution(allocations);
    
    // Disable bot protection for predictable testing
    token.set_bot_protection(false);
    
    // Transfer tokens and check burn amount
    let initial_supply = token.get_total_supply();
    let transfer_amount = 10000.0;
    token.transfer("user1", "user2", transfer_amount).unwrap();
    
    // Calculate expected burn: 2% of 10,000 = 200 tokens
    let expected_burn = transfer_amount * 0.02;
    let actual_supply = token.get_total_supply();
    let actual_burn = initial_supply - actual_supply;
    
    assert_eq!(actual_burn, expected_burn);
    
    // Check that the recipient received the correct amount (transfer amount - burn)
    let recipient_balance = token.get_balance("user2");
    assert_eq!(recipient_balance, 1000000.0 + (transfer_amount - expected_burn));
}

#[test]
fn test_deflationary_mechanisms_integration() {
    let mut token = PProjectToken::new(350000000.0, 0.01, 0.005);
    let mut treasury = Treasury::new();
    
    // Add funds to treasury
    treasury.add_funds("USD".to_string(), 1000000.0).unwrap();
    
    // Initialize some balances
    let allocations = vec![
        ("user1".to_string(), 1000000.0),
        ("user2".to_string(), 1000000.0),
        ("liquidity".to_string(), 1000000.0),
    ];
    token.initialize_distribution(allocations);
    
    // Disable bot protection for predictable testing
    token.set_bot_protection(false);
    
    // Add scheduled burns
    let now = Utc::now().naive_utc();
    let past_burn_time = now - Duration::days(1);
    token.add_scheduled_burn(past_burn_time, 100000.0);
    token.set_burn_schedule_enabled(true);
    
    // Add milestone burns
    token.add_milestone_burn(
        "5k_holders".to_string(),
        "holders_count".to_string(),
        5000.0, // Target: 5,000 holders
        250000.0, // Burn 250,000 tokens when reached
    );
    
    // Add revenue-linked burns
    token.add_revenue_linked_burn(
        "transaction_fees".to_string(),
        25000.0, // $25,000 in transaction fees
        20.0,    // Burn 20% of revenue
    );
    
    // Add scheduled buybacks
    treasury.add_scheduled_buyback(now - Duration::hours(1), 50000.0, 0.01).unwrap();
    treasury.set_auto_buyback_enabled(true);
    
    // Add buyback triggers
    treasury.add_buyback_trigger(
        "price_drop".to_string(),
        0.005,   // Trigger when price drops to $0.005 or below
        25000.0, // Spend $25,000 on buyback
    ).unwrap();
    
    // Record initial state
    let initial_supply = token.get_total_supply();
    let initial_treasury_balance = treasury.get_balance("USD");
    
    // Execute transfers to trigger dynamic burns and milestone burns
    for i in 0..6000 {
        let from_user = if i % 2 == 0 { "user1" } else { "user2" };
        let to_user = if i % 2 == 0 { "user2" } else { "user1" };
        token.transfer(from_user, to_user, 1000.0).unwrap();
    }
    
    // Execute scheduled burns
    let scheduled_burn_amount = token.execute_scheduled_burns().unwrap();
    
    // Execute milestone burns
    let milestone_burn_amount = token.check_milestone_burns().unwrap();
    
    // Execute revenue-linked burns
    let revenue_burn_amount = token.execute_revenue_linked_burns().unwrap();
    
    // Execute scheduled buybacks
    let buyback_tokens = treasury.execute_scheduled_buybacks(&mut token, 0.01).unwrap();
    
    // Execute trigger buybacks (price is $0.01, which is above threshold of $0.005, so no trigger)
    let trigger_buyback_tokens = treasury.check_buyback_triggers(
        &mut token,
        0.01, // Current price
        "",   // No specific market condition
        0.0,  // No condition value
    ).unwrap();
    
    // Verify all mechanisms worked
    assert_eq!(scheduled_burn_amount, 100000.0);
    assert_eq!(milestone_burn_amount, 250000.0);
    assert_eq!(revenue_burn_amount, 25000.0 * 0.20);
    assert_eq!(buyback_tokens, 50000.0 / 0.01);
    assert_eq!(trigger_buyback_tokens, 0.0); // No trigger executed
    
    // Verify final state
    let final_supply = token.get_total_supply();
    let final_treasury_balance = treasury.get_balance("USD");
    
    // Supply should be reduced by all burn mechanisms
    let total_burned = scheduled_burn_amount + milestone_burn_amount + revenue_burn_amount;
    assert_eq!(final_supply, initial_supply - total_burned);
    
    // Treasury balance should be reduced by buybacks
    let total_spent = 50000.0; // Only scheduled buyback executed
    assert_eq!(final_treasury_balance, initial_treasury_balance - total_spent);
    
    println!("Initial supply: {}, Final supply: {}", initial_supply, final_supply);
    println!("Total burned: {}", total_burned);
    println!("Initial treasury: ${}, Final treasury: ${}", initial_treasury_balance, final_treasury_balance);
}
