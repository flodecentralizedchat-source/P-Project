use super::vesting::{VestingContract, VestingError, VestingType};
use chrono::Utc;

#[test]
fn test_seed_investor_vesting_schedule() {
    let total_vesting_tokens = 24500000.0; // 24.5M tokens for seed round (7% of 350M)
    let mut vesting_contract = VestingContract::new(total_vesting_tokens);
    
    let user_id = "seed_investor_1".to_string();
    let amount = 1000000.0; // 1M tokens
    let start_date = Utc::now().naive_utc();
    
    // Create seed investor vesting schedule (18-month vesting, 3-month cliff)
    let result = vesting_contract.create_seed_investor_vesting(user_id.clone(), amount, start_date);
    assert!(result.is_ok());
    
    // Check that no tokens are claimable during cliff period
    let claimable = vesting_contract.calculate_claimable_amount(&user_id);
    assert!(claimable.is_ok());
    assert_eq!(claimable.unwrap(), 0.0);
    
    // Check vesting schedule details
    let schedule = vesting_contract.get_vesting_schedule(&user_id);
    assert!(schedule.is_some());
    let schedule = schedule.unwrap();
    assert_eq!(schedule.total_amount, amount);
    assert_eq!(schedule.cliff_duration_months, 3);
    assert_eq!(schedule.vesting_duration_months, 18);
    assert_eq!(schedule.schedule_type, VestingType::Investor);
}

#[test]
fn test_private_investor_vesting_schedule() {
    let total_vesting_tokens = 17500000.0; // 17.5M tokens for private round (5% of 350M)
    let mut vesting_contract = VestingContract::new(total_vesting_tokens);
    
    let user_id = "private_investor_1".to_string();
    let amount = 500000.0; // 500K tokens
    let start_date = Utc::now().naive_utc();
    
    // Create private investor vesting schedule (24-month vesting, 6-month cliff)
    let result = vesting_contract.create_private_investor_vesting(user_id.clone(), amount, start_date);
    assert!(result.is_ok());
    
    // Check that no tokens are claimable during cliff period
    let claimable = vesting_contract.calculate_claimable_amount(&user_id);
    assert!(claimable.is_ok());
    assert_eq!(claimable.unwrap(), 0.0);
    
    // Check vesting schedule details
    let schedule = vesting_contract.get_vesting_schedule(&user_id);
    assert!(schedule.is_some());
    let schedule = schedule.unwrap();
    assert_eq!(schedule.total_amount, amount);
    assert_eq!(schedule.cliff_duration_months, 6);
    assert_eq!(schedule.vesting_duration_months, 24);
    assert_eq!(schedule.schedule_type, VestingType::Investor);
}

#[test]
fn test_public_sale_vesting_schedule() {
    let total_vesting_tokens = 10500000.0; // 10.5M tokens for public sale (3% of 350M)
    let mut vesting_contract = VestingContract::new(total_vesting_tokens);
    
    let user_id = "public_sale_investor_1".to_string();
    let amount = 100000.0; // 100K tokens
    let start_date = Utc::now().naive_utc();
    
    // Create public sale vesting schedule (12-month linear vesting, 10% TGE)
    let result = vesting_contract.create_public_sale_vesting(user_id.clone(), amount, start_date);
    assert!(result.is_ok());
    
    // For public sale, there's no cliff so tokens should be claimable immediately
    // But since we're not mocking time, we'll just check the schedule details
    
    // Check vesting schedule details
    let schedule = vesting_contract.get_vesting_schedule(&user_id);
    assert!(schedule.is_some());
    let schedule = schedule.unwrap();
    assert_eq!(schedule.total_amount, amount);
    assert_eq!(schedule.cliff_duration_months, 0); // No cliff for public sale
    assert_eq!(schedule.vesting_duration_months, 12);
    assert_eq!(schedule.schedule_type, VestingType::Investor);
}

#[test]
fn test_multiple_investor_types() {
    let total_vesting_tokens = 52500000.0; // Total for all investor types
    let mut vesting_contract = VestingContract::new(total_vesting_tokens);
    
    let start_date = Utc::now().naive_utc();
    
    // Create seed investor vesting schedule
    let seed_investor_result = vesting_contract.create_seed_investor_vesting(
        "seed_investor".to_string(),
        1000000.0,
        start_date
    );
    assert!(seed_investor_result.is_ok());
    
    // Create private investor vesting schedule
    let private_investor_result = vesting_contract.create_private_investor_vesting(
        "private_investor".to_string(),
        500000.0,
        start_date
    );
    assert!(private_investor_result.is_ok());
    
    // Create public sale vesting schedule
    let public_sale_result = vesting_contract.create_public_sale_vesting(
        "public_sale_investor".to_string(),
        100000.0,
        start_date
    );
    assert!(public_sale_result.is_ok());
    
    // Verify all schedules
    let seed_schedule = vesting_contract.get_vesting_schedule("seed_investor").unwrap();
    assert_eq!(seed_schedule.cliff_duration_months, 3);
    assert_eq!(seed_schedule.vesting_duration_months, 18);
    assert_eq!(seed_schedule.schedule_type, VestingType::Investor);
    
    let private_schedule = vesting_contract.get_vesting_schedule("private_investor").unwrap();
    assert_eq!(private_schedule.cliff_duration_months, 6);
    assert_eq!(private_schedule.vesting_duration_months, 24);
    assert_eq!(private_schedule.schedule_type, VestingType::Investor);
    
    let public_schedule = vesting_contract.get_vesting_schedule("public_sale_investor").unwrap();
    assert_eq!(public_schedule.cliff_duration_months, 0);
    assert_eq!(public_schedule.vesting_duration_months, 12);
    assert_eq!(public_schedule.schedule_type, VestingType::Investor);
}

#[test]
fn test_investor_vesting_insufficient_tokens() {
    let total_vesting_tokens = 1000000.0; // Only 1M tokens available
    let mut vesting_contract = VestingContract::new(total_vesting_tokens);
    
    let user_id = "seed_investor_1".to_string();
    let amount = 2000000.0; // Trying to allocate 2M tokens
    let start_date = Utc::now().naive_utc();
    
    // This should fail due to insufficient tokens
    let result = vesting_contract.create_seed_investor_vesting(user_id, amount, start_date);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), VestingError::InsufficientTokens);
}

#[test]
fn test_founder_multisig_features() {
    let total_vesting_tokens = 70000000.0; // 70M tokens for founders (20% of 350M)
    let mut vesting_contract = VestingContract::new(total_vesting_tokens);
    
    // Test default multisig settings
    let (signers, required) = vesting_contract.get_founder_multisig_signers();
    assert_eq!(signers.len(), 3);
    assert_eq!(required, 2);
    assert!(vesting_contract.is_founder_signer("founder1"));
    assert!(!vesting_contract.is_founder_signer("unauthorized_user"));
    
    // Test updating multisig settings
    let new_signers = vec!["founder1".to_string(), "founder2".to_string(), "advisor".to_string()];
    vesting_contract.set_founder_multisig_signers(new_signers.clone(), 2);
    
    let (updated_signers, updated_required) = vesting_contract.get_founder_multisig_signers();
    assert_eq!(updated_signers, &new_signers);
    assert_eq!(updated_required, 2);
    assert!(vesting_contract.is_founder_signer("advisor"));
}
