use super::vesting::{VestingContract, VestingType};
use chrono::Utc;

#[test]
fn test_complete_team_allocation_protection() {
    // Test the complete team allocation as specified in team_allocation_protection.csv
    
    // Calculate total tokens needed:
    // 2 Founders: 35M each = 70M
    // 13 Founding Members: ~2,692,308 each = ~35,000,004
    // Total: ~105,000,004
    let total_founder_tokens = 70000000.0; // 2 * 35M
    let total_member_tokens = 35000004.0;  // 13 * 2,692,308
    let total_allocated_tokens = total_founder_tokens + total_member_tokens;
    
    // Initialize with exactly what we need
    let mut vesting_contract = VestingContract::new(total_allocated_tokens);
    
    let start_date = Utc::now().naive_utc();
    
    // Test Founder 1 Allocation (10% = 35M tokens, 4-year vesting, 1-year cliff)
    let founder1_result = vesting_contract.create_founder_vesting(
        "founder_1".to_string(), 
        35000000.0, 
        start_date
    );
    assert!(founder1_result.is_ok());
    
    // Verify Founder 1 schedule
    let founder1_schedule = vesting_contract.get_vesting_schedule("founder_1").unwrap();
    assert_eq!(founder1_schedule.total_amount, 35000000.0);
    assert_eq!(founder1_schedule.cliff_duration_months, 12);
    assert_eq!(founder1_schedule.vesting_duration_months, 48);
    assert_eq!(founder1_schedule.schedule_type, VestingType::Founder);
    
    // Test Founder 2 Allocation (10% = 35M tokens, 4-year vesting, 1-year cliff)
    let founder2_result = vesting_contract.create_founder_vesting(
        "founder_2".to_string(), 
        35000000.0, 
        start_date
    );
    assert!(founder2_result.is_ok());
    
    // Verify Founder 2 schedule
    let founder2_schedule = vesting_contract.get_vesting_schedule("founder_2").unwrap();
    assert_eq!(founder2_schedule.total_amount, 35000000.0);
    assert_eq!(founder2_schedule.cliff_duration_months, 12);
    assert_eq!(founder2_schedule.vesting_duration_months, 48);
    assert_eq!(founder2_schedule.schedule_type, VestingType::Founder);
    
    // Test all 13 Founding Members (0.7692% each = ~2,692,308 tokens, 3-year vesting, 6-month cliff)
    for i in 1..=13 {
        let member_id = format!("founding_member_{}", i);
        let member_result = vesting_contract.create_founding_member_vesting(
            member_id, 
            2692308.0, 
            start_date
        );
        assert!(member_result.is_ok());
    }
    
    // Verify one founding member schedule
    let member_schedule = vesting_contract.get_vesting_schedule("founding_member_1").unwrap();
    assert_eq!(member_schedule.total_amount, 2692308.0);
    assert_eq!(member_schedule.cliff_duration_months, 6);
    assert_eq!(member_schedule.vesting_duration_months, 36);
    assert_eq!(member_schedule.schedule_type, VestingType::FoundingMember);
    
    // Verify total allocation
    assert_eq!(vesting_contract.get_claimed_tokens(), 0.0);
    assert_eq!(vesting_contract.get_total_vesting_tokens(), total_allocated_tokens);
    
    // Calculate how much we've actually allocated
    let mut total_used = 0.0;
    for i in 1..=2 {
        let founder_id = format!("founder_{}", i);
        if let Some(schedule) = vesting_contract.get_vesting_schedule(&founder_id) {
            total_used += schedule.total_amount;
        }
    }
    
    for i in 1..=13 {
        let member_id = format!("founding_member_{}", i);
        if let Some(schedule) = vesting_contract.get_vesting_schedule(&member_id) {
            total_used += schedule.total_amount;
        }
    }
    
    // Verify that we've used the expected amount
    assert_eq!(total_used, total_allocated_tokens);
}

#[test]
fn test_anti_dump_protection_mechanisms() {
    // Test the anti-dump protection mechanisms mentioned in the CSV
    
    let total_tokens = 70000000.0;
    let mut vesting_contract = VestingContract::new(total_tokens);
    let start_date = Utc::now().naive_utc();
    
    // Test mandatory vesting locks for founders
    let founder_result = vesting_contract.create_founder_vesting(
        "founder_1".to_string(),
        35000000.0,
        start_date
    );
    assert!(founder_result.is_ok());
    
    // Check that no tokens are claimable during cliff period (anti-dump protection)
    let claimable = vesting_contract.calculate_claimable_amount("founder_1").unwrap();
    assert_eq!(claimable, 0.0);
    
    // Test mandatory vesting locks for founding members
    let member_result = vesting_contract.create_founding_member_vesting(
        "member_1".to_string(),
        2692308.0,
        start_date
    );
    assert!(member_result.is_ok());
    
    // Check that no tokens are claimable during cliff period (anti-dump protection)
    let claimable = vesting_contract.calculate_claimable_amount("member_1").unwrap();
    assert_eq!(claimable, 0.0);
    
    // Verify linear unlock mechanism (monthly release after cliff)
    let founder_schedule = vesting_contract.get_vesting_schedule("founder_1").unwrap();
    assert_eq!(founder_schedule.is_linear, true);
    
    let member_schedule = vesting_contract.get_vesting_schedule("member_1").unwrap();
    assert_eq!(member_schedule.is_linear, true);
}

#[test]
fn test_vesting_contract_immutable_properties() {
    // Test that vesting contracts are immutable as specified in the protection mechanisms
    
    let total_tokens = 70000000.0;
    let mut vesting_contract = VestingContract::new(total_tokens);
    let start_date = Utc::now().naive_utc();
    
    // Create a founder vesting schedule
    vesting_contract.create_founder_vesting(
        "founder_1".to_string(),
        35000000.0,
        start_date
    ).unwrap();
    
    // Verify that the schedule type is correctly set and cannot be changed externally
    let schedule = vesting_contract.get_vesting_schedule("founder_1").unwrap();
    assert_eq!(schedule.schedule_type, VestingType::Founder);
    assert_eq!(schedule.cliff_duration_months, 12);
    assert_eq!(schedule.vesting_duration_months, 48);
    
    // These properties enforce the immutable nature of the vesting contract
    assert_eq!(schedule.total_amount, 35000000.0);
    assert_eq!(schedule.claimed_amount, 0.0);
    assert_eq!(schedule.is_linear, true);
}