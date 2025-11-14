use super::airdrop::{AirdropContract, AirdropError};

#[test]
fn test_airdrop_creation() {
    let total_amount = 52500000.0; // 52.5M tokens
    let airdrop_contract = AirdropContract::new(total_amount);
    
    // Check that the airdrop was created successfully
    assert_eq!(airdrop_contract.get_status().total_amount, total_amount);
}

#[test]
fn test_airdrop_recipient_management() {
    let mut airdrop_contract = AirdropContract::new(52500000.0);
    
    let recipients = vec![
        ("user1".to_string(), 1000.0),
        ("user2".to_string(), 2000.0),
    ];
    
    // Add recipients
    let result = airdrop_contract.add_recipients(recipients);
    assert!(result.is_ok());
    
    // Check status
    let status = airdrop_contract.get_status();
    assert_eq!(status.total_recipients, 2);
    assert_eq!(status.distributed_amount, 3000.0);
}

#[test]
fn test_airdrop_claiming() {
    let mut airdrop_contract = AirdropContract::new(52500000.0);
    
    let recipients = vec![("user1".to_string(), 1000.0)];
    airdrop_contract.add_recipients(recipients).unwrap();
    
    // Claim airdrop
    let result = airdrop_contract.claim("user1");
    assert!(result.is_ok());
    
    let claimed_amount = result.unwrap();
    assert_eq!(claimed_amount, 1000.0);
    
    // Check that it's marked as claimed
    assert!(airdrop_contract.is_claimed("user1"));
}

#[test]
fn test_airdrop_insufficient_tokens() {
    let mut airdrop_contract = AirdropContract::new(1000.0); // Only 1000 tokens
    
    let recipients = vec![("user1".to_string(), 2000.0)]; // Trying to allocate 2000 tokens
    
    // This should fail due to insufficient tokens
    let result = airdrop_contract.add_recipients(recipients);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), AirdropError::InsufficientTokens);
}

#[test]
fn test_airdrop_status_tracking() {
    let mut airdrop_contract = AirdropContract::new(10000.0);
    
    let recipients = vec![
        ("user1".to_string(), 1000.0),
        ("user2".to_string(), 2000.0),
        ("user3".to_string(), 1500.0),
    ];
    
    airdrop_contract.add_recipients(recipients).unwrap();
    
    // Initially no one has claimed
    let status = airdrop_contract.get_status();
    assert_eq!(status.total_recipients, 3);
    assert_eq!(status.claimed_recipients, 0);
    assert_eq!(status.distributed_amount, 4500.0);
    
    // User 1 claims
    airdrop_contract.claim("user1").unwrap();
    let status = airdrop_contract.get_status();
    assert_eq!(status.claimed_recipients, 1);
    
    // User 2 claims
    airdrop_contract.claim("user2").unwrap();
    let status = airdrop_contract.get_status();
    assert_eq!(status.claimed_recipients, 2);
}

#[test]
fn test_batch_claiming() {
    let mut airdrop_contract = AirdropContract::new(10000.0);
    
    let recipients = vec![
        ("user1".to_string(), 1000.0),
        ("user2".to_string(), 2000.0),
        ("user3".to_string(), 1500.0),
    ];
    
    airdrop_contract.add_recipients(recipients).unwrap();
    
    // Batch claim for user1 and user3
    let users_to_claim = vec!["user1".to_string(), "user3".to_string()];
    let result = airdrop_contract.batch_claim(users_to_claim);
    assert!(result.is_ok());
    
    let claimed_amounts = result.unwrap();
    assert_eq!(claimed_amounts.len(), 2);
    
    // Check that the right amounts were claimed
    assert_eq!(claimed_amounts[0].1, 1000.0); // user1
    assert_eq!(claimed_amounts[1].1, 1500.0); // user3
    
    // Check that user2 hasn't claimed yet
    assert!(!airdrop_contract.is_claimed("user2"));
    
    // Check that user1 and user3 have claimed
    assert!(airdrop_contract.is_claimed("user1"));
    assert!(airdrop_contract.is_claimed("user3"));
}

#[test]
fn test_airdrop_with_categories() {
    let mut airdrop_contract = AirdropContract::new(10000.0);
    
    let recipients = vec![
        ("user1".to_string(), 1000.0),
        ("user2".to_string(), 2000.0),
    ];
    
    // Add recipients with category
    let result = airdrop_contract.add_recipients_with_category(recipients, Some("early_supporter".to_string()));
    assert!(result.is_ok());
    
    // Check that categories are set
    assert_eq!(airdrop_contract.get_user_category("user1"), Some(&"early_supporter".to_string()));
    assert_eq!(airdrop_contract.get_user_category("user2"), Some(&"early_supporter".to_string()));
}

#[test]
fn test_airdrop_pause_functionality() {
    let mut airdrop_contract = AirdropContract::new(10000.0);
    
    let recipients = vec![("user1".to_string(), 1000.0)];
    airdrop_contract.add_recipients(recipients).unwrap();
    
    // Initially not paused
    assert!(!airdrop_contract.is_paused());
    
    // Pause the airdrop
    airdrop_contract.pause();
    assert!(airdrop_contract.is_paused());
    
    // Try to claim while paused - should fail
    let result = airdrop_contract.claim("user1");
    assert!(result.is_err());
}