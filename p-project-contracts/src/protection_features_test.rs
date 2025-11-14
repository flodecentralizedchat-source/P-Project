use super::staking::StakingContract;
use super::token::{PProjectToken, TokenError};
use super::treasury::Treasury;

#[test]
fn test_wallet_restrictions() {
    let mut token = PProjectToken::new(1000000.0, 0.01, 0.005);

    // Initialize some balances
    let allocations = vec![
        ("user1".to_string(), 100000.0),
        ("user2".to_string(), 100000.0),
    ];
    token.initialize_distribution(allocations);

    // Restrict user1's wallet
    token.restrict_wallet("user1".to_string(), true);

    // Try to transfer from restricted wallet - should fail
    let result = token.transfer("user1", "user2", 1000.0);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), TokenError::TransferLimitExceeded(0.0));

    // Try to transfer from unrestricted wallet - should succeed
    let result = token.transfer("user2", "user1", 1000.0);
    assert!(result.is_ok());

    // Unrestrict user1's wallet
    token.restrict_wallet("user1".to_string(), false);

    // Try to transfer from now unrestricted wallet - should succeed
    let result = token.transfer("user1", "user2", 1000.0);
    assert!(result.is_ok());
}

#[test]
fn test_max_daily_transfer_limit() {
    let mut token = PProjectToken::new(1000000.0, 0.01, 0.005);

    // Set daily transfer limit to 1% of total supply (10,000 tokens)
    token.set_max_daily_transfer_percent(0.01);

    // Disable bot protection for this test to avoid interference
    token.set_bot_protection(false);

    // Initialize some balances
    let allocations = vec![("user1".to_string(), 100000.0)];
    token.initialize_distribution(allocations);

    // Check that the daily limit is correctly set
    assert_eq!(token.get_max_daily_transfer_limit(), 10000.0);

    // Check the anti-whale limit
    println!("Max transfer limit: {}", token.get_max_transfer_limit());

    // First transfer within daily limit - should succeed
    let result = token.transfer("user1", "user2", 5000.0);
    assert!(result.is_ok());

    // Second transfer that would exceed daily limit - should fail
    let result = token.transfer("user1", "user2", 6000.0);
    assert!(result.is_err());
    // Check that it's a TransferLimitExceeded error (exact value may have precision issues)
    match result.unwrap_err() {
        TokenError::TransferLimitExceeded(_) => {} // Expected error type
        _ => panic!("Expected TransferLimitExceeded error"),
    }

    // Transfer within remaining daily limit - should succeed
    let result = token.transfer("user1", "user2", 4000.0); // 5000 already transferred, 4000 more is within 10000 limit
                                                           // Let's check what the actual error is
    if let Err(e) = &result {
        println!("Error: {:?}", e);
    }
    assert!(result.is_ok());
}

#[test]
fn test_bot_protection() {
    let mut token = PProjectToken::new(1000000.0, 0.01, 0.005);

    // Set bot cooldown period to 2 seconds for testing
    token.set_bot_cooldown_period(2);

    // Initialize some balances
    let allocations = vec![("user1".to_string(), 100000.0)];
    token.initialize_distribution(allocations);

    // First transfer - should succeed
    let result = token.transfer("user1", "user2", 1000.0);
    assert!(result.is_ok());

    // Immediate second transfer - should fail due to bot protection
    let result = token.transfer("user1", "user2", 1000.0);
    assert!(result.is_err());

    // Note: We're not testing the cooldown period in this test because
    // it would require sleeping, which isn't ideal for unit tests
}

#[test]
fn test_multisig_approval() {
    let mut treasury = Treasury::new();

    // Add funds to treasury
    treasury.add_funds("USD".to_string(), 100000.0).unwrap();

    // Set custom multi-sig signers
    let signers = vec![
        "alice".to_string(),
        "bob".to_string(),
        "charlie".to_string(),
        "david".to_string(),
        "eve".to_string(),
    ];
    treasury.set_multisig_signers(signers, 3); // 3-of-5 multi-sig

    // Create a pending transaction
    let tx_id = treasury
        .create_pending_transaction(
            "Test transaction".to_string(),
            5000.0,
            "USD".to_string(),
            "recipient".to_string(),
            "creator".to_string(),
        )
        .unwrap();

    // Check that transaction was created
    let pending_txs = treasury.get_pending_transactions();
    assert!(pending_txs.contains_key(&tx_id));

    // First signature
    let result = treasury.sign_transaction(&tx_id, "alice".to_string());
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), false); // Not enough signatures yet

    // Second signature
    let result = treasury.sign_transaction(&tx_id, "bob".to_string());
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), false); // Not enough signatures yet

    // Third signature - should execute transaction
    let result = treasury.sign_transaction(&tx_id, "charlie".to_string());
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), true); // Executed

    // Check that funds were deducted
    assert_eq!(treasury.get_balance("USD"), 95000.0);
}

#[test]
fn test_team_staking_incentives() {
    let mut staking_contract = StakingContract::new();

    // Add user1 as a team member
    staking_contract.add_team_member("team_member".to_string());

    // Check that user is recognized as team member
    assert!(staking_contract.is_team_member("team_member"));
    assert!(!staking_contract.is_team_member("regular_user"));

    // Team members get a boost to their APY
    let team_boost = staking_contract.get_team_member_boost();
    assert_eq!(team_boost, 0.05); // 5% boost
}

#[test]
fn test_enhanced_buyback_engine() {
    let mut treasury = Treasury::new();
    let mut token = PProjectToken::new(1000000.0, 0.01, 0.005);

    // Add funds to treasury
    treasury.add_funds("USD".to_string(), 100000.0).unwrap();

    // Execute buyback
    let tokens_bought = treasury.execute_buyback(&mut token, 10000.0, 0.1).unwrap();

    // Check that tokens were "bought"
    assert_eq!(tokens_bought, 100000.0); // 10000 USD / 0.1 USD per token = 100000 tokens

    // Check that funds were deducted from treasury
    assert_eq!(treasury.get_balance("USD"), 90000.0);
}
