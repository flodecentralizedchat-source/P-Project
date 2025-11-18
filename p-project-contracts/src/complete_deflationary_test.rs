use crate::token::PProjectToken;
use crate::treasury::{BuybackCondition, MarketSnapshot, Treasury};
use chrono::{Utc, Duration};

#[test]
fn test_complete_deflationary_mechanisms() {
    // Initialize token with 1% base burn rate and 0.5% reward rate
    let mut token = PProjectToken::new(350000000.0, 0.01, 0.005);
    
    // Initialize treasury
    let mut treasury = Treasury::new();
    
    // Add funds to treasury for buybacks
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
    
    // Record initial state
    let initial_supply = token.get_total_supply();
    let initial_treasury_balance = treasury.get_balance("USD");
    
    println!("=== Testing Dynamic Burn Rates ===");
    
    // Normal transfer with base burn rate
    token.transfer("user1", "user2", 10000.0).unwrap();
    let supply_after_normal_transfer = token.get_total_supply();
    let normal_burn = initial_supply - supply_after_normal_transfer;
    println!("Normal burn amount: {}", normal_burn);
    
    // Simulate high activity for user1 to trigger increased burn rate
    for i in 0..15 {
        token.transfer("user1", "user2", 1000.0).unwrap();
    }
    
    // Transfer with increased burn rate due to high activity
    let supply_before_high_activity = token.get_total_supply();
    token.transfer("user1", "user2", 10000.0).unwrap();
    let supply_after_high_activity = token.get_total_supply();
    let high_activity_burn = supply_before_high_activity - supply_after_high_activity;
    println!("High activity burn amount: {}", high_activity_burn);
    
    // Verify that high activity burn is greater than normal burn
    assert!(high_activity_burn > normal_burn);
    
    println!("\n=== Testing Scheduled Burns ===");
    
    // Add scheduled burns
    let now = Utc::now().naive_utc();
    let past_burn_time = now - Duration::days(1);
    token.add_scheduled_burn(past_burn_time, 100000.0);
    token.set_burn_schedule_enabled(true);
    
    // Execute scheduled burns
    let scheduled_burn_amount = token.execute_scheduled_burns().unwrap();
    println!("Scheduled burn amount: {}", scheduled_burn_amount);
    assert_eq!(scheduled_burn_amount, 100000.0);
    
    println!("\n=== Testing Milestone Burns ===");
    
    // Add milestone burns
    token.add_milestone_burn(
        "10k_holders".to_string(),
        "holders_count".to_string(),
        10000.0, // Target: 10,000 holders
        500000.0, // Burn 500,000 tokens when reached
    );
    
    // Simulate creating many holders to reach milestone
    // We already have 3 holders, so we need to create more
    for i in 0..10000 {
        let user_id = format!("holder_{}", i);
        token.transfer("liquidity", &user_id, 10.0).unwrap();
    }
    
    // Check that we've exceeded the holder count milestone
    assert!(token.holders.len() > 10000);
    
    // Execute milestone burns
    let milestone_burn_amount = token.check_milestone_burns().unwrap();
    println!("Milestone burn amount: {}", milestone_burn_amount);
    assert_eq!(milestone_burn_amount, 500000.0);
    
    println!("\n=== Testing Revenue-Linked Burns ===");
    
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
    let revenue_burn_amount = token.execute_revenue_linked_burns().unwrap();
    let expected_revenue_burn = 50000.0 * 0.10 + 100000.0 * 0.05; // $5,000 + $5,000 = $10,000
    println!("Revenue-linked burn amount: {}", revenue_burn_amount);
    assert_eq!(revenue_burn_amount, expected_revenue_burn);
    
    println!("\n=== Testing Treasury Buybacks ===");
    
    // Execute treasury buyback
    let tokens_bought = treasury.execute_buyback(&mut token, 100000.0, 0.01).unwrap();
    let expected_tokens_bought = 100000.0 / 0.01; // 10,000,000 tokens
    println!("Tokens bought and burned in buyback: {}", tokens_bought);
    assert_eq!(tokens_bought, expected_tokens_bought);
    
    println!("\n=== Testing Scheduled Buybacks ===");
    
    // Add scheduled buybacks
    treasury.add_scheduled_buyback(now - Duration::hours(1), 50000.0, 0.01).unwrap();
    treasury.set_auto_buyback_enabled(true);
    
    // Execute scheduled buybacks
    let scheduled_buyback_tokens = treasury.execute_scheduled_buybacks(&mut token, 0.01).unwrap();
    let expected_scheduled_buyback_tokens = 50000.0 / 0.01; // 5,000,000 tokens
    println!("Tokens bought and burned in scheduled buyback: {}", scheduled_buyback_tokens);
    assert_eq!(scheduled_buyback_tokens, expected_scheduled_buyback_tokens);
    
    println!("\n=== Testing Trigger-Based Buybacks ===");
    
    // Add buyback triggers
    treasury.add_buyback_trigger(
        "price_drop".to_string(),
        BuybackCondition::PriceBelow,
        0.005,    // Trigger when price drops to $0.005 or below
        25000.0,  // Spend $25,000 on buyback
    ).unwrap();
    
    // Test price drop trigger - current price is $0.01, which is above threshold
    // So no trigger should execute
    let trigger_buyback_tokens = treasury.check_buyback_triggers(&mut token, &MarketSnapshot {
        price: 0.01, // Current price above threshold
        volume: 1_000_000.0,
        price_change_percentage: -0.5,
        volume_change_percentage: 0.0,
    }).unwrap();
    println!("Trigger buyback tokens (should be 0): {}", trigger_buyback_tokens);
    assert_eq!(trigger_buyback_tokens, 0.0);
    
    // Test with price below threshold
    let trigger_buyback_tokens_below = treasury.check_buyback_triggers(&mut token, &MarketSnapshot {
        price: 0.001, // Current price below threshold
        volume: 450_000.0,
        price_change_percentage: -5.0,
        volume_change_percentage: 0.0,
    }).unwrap();
    let expected_trigger_buyback_tokens = 25000.0 / 0.001; // 25,000,000 tokens
    println!("Trigger buyback tokens (price below threshold): {}", trigger_buyback_tokens_below);
    assert_eq!(trigger_buyback_tokens_below, expected_trigger_buyback_tokens);
    
    // Verify final state
    let final_supply = token.get_total_supply();
    let final_treasury_balance = treasury.get_balance("USD");
    
    // Calculate total burned tokens
    let total_burned = normal_burn + high_activity_burn + scheduled_burn_amount + 
                      milestone_burn_amount + revenue_burn_amount;
    
    // Calculate total tokens bought and burned through buybacks
    let total_buyback_tokens = tokens_bought + scheduled_buyback_tokens + trigger_buyback_tokens_below;
    
    println!("\n=== Final Results ===");
    println!("Initial supply: {}", initial_supply);
    println!("Final supply: {}", final_supply);
    println!("Total burned through token burns: {}", total_burned);
    println!("Total burned through buybacks: {}", total_buyback_tokens);
    println!("Total supply reduction: {}", initial_supply - final_supply);
    println!("Initial treasury balance: ${}", initial_treasury_balance);
    println!("Final treasury balance: ${}", final_treasury_balance);
    println!("Total treasury spent on buybacks: ${}", initial_treasury_balance - final_treasury_balance);
    
    // Verify supply was reduced
    assert!(final_supply < initial_supply);
    
    // Verify treasury balance was reduced by buyback spending
    let total_buyback_spending = 100000.0 + 50000.0 + 25000.0; // Direct buyback + scheduled + trigger
    assert_eq!(final_treasury_balance, initial_treasury_balance - total_buyback_spending);
    
    println!("\nAll deflationary mechanisms tested successfully!");
}
