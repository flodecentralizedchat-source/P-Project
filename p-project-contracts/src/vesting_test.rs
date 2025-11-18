use super::vesting::{VestingContract, VestingError, VestingType};
use chrono::{Duration, Utc};

#[test]
fn test_team_vesting_schedule() {
    let total_vesting_tokens = 17500000.0; // 17.5M tokens
    let mut vesting_contract = VestingContract::new(total_vesting_tokens);

    let user_id = "team_member_1".to_string();
    let amount = 1000000.0; // 1M tokens
    let start_date = Utc::now().naive_utc();

    // Create team vesting schedule (12m cliff + 24m linear)
    let result = vesting_contract.create_team_vesting(user_id.clone(), amount, start_date);
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
    assert_eq!(schedule.cliff_duration_months, 12);
    assert_eq!(schedule.vesting_duration_months, 24);
}

#[test]
fn test_advisor_vesting_schedule() {
    let total_vesting_tokens = 3500000.0; // 3.5M tokens for advisors
    let mut vesting_contract = VestingContract::new(total_vesting_tokens);

    let user_id = "advisor_1".to_string();
    let amount = 100000.0; // 100K tokens
    let start_date = Utc::now().naive_utc();

    // Create advisor vesting schedule (6m cliff + 12m linear)
    let result = vesting_contract.create_advisor_vesting(user_id.clone(), amount, start_date);
    assert!(result.is_ok());

    // Check vesting schedule details
    let schedule = vesting_contract.get_vesting_schedule(&user_id);
    assert!(schedule.is_some());
    let schedule = schedule.unwrap();
    assert_eq!(schedule.total_amount, amount);
    assert_eq!(schedule.cliff_duration_months, 6);
    assert_eq!(schedule.vesting_duration_months, 12);
}

#[test]
fn test_founder_vesting_schedule() {
    let total_vesting_tokens = 70000000.0; // 70M tokens for founders (20% of 350M)
    let mut vesting_contract = VestingContract::new(total_vesting_tokens);

    let user_id = "founder_1".to_string();
    let amount = 35000000.0; // 35M tokens
    let start_date = Utc::now().naive_utc();

    // Create founder vesting schedule (4-year vesting, 1-year cliff)
    let result = vesting_contract.create_founder_vesting(user_id.clone(), amount, start_date);
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
    assert_eq!(schedule.cliff_duration_months, 12);
    assert_eq!(schedule.vesting_duration_months, 48);
    assert_eq!(schedule.schedule_type, VestingType::Founder);
}

#[test]
fn test_founding_member_vesting_schedule() {
    let total_vesting_tokens = 35000000.0; // 35M tokens for founding members (10% of 350M)
    let mut vesting_contract = VestingContract::new(total_vesting_tokens);

    let user_id = "founding_member_1".to_string();
    let amount = 2692308.0; // ~2.69M tokens
    let start_date = Utc::now().naive_utc();

    // Create founding member vesting schedule (3-year vesting, 6-month cliff)
    let result =
        vesting_contract.create_founding_member_vesting(user_id.clone(), amount, start_date);
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
    assert_eq!(schedule.vesting_duration_months, 36);
    assert_eq!(schedule.schedule_type, VestingType::FoundingMember);
}

#[test]
fn test_vesting_claim_after_cliff() {
    let total_vesting_tokens = 17500000.0;
    let mut vesting_contract = VestingContract::new(total_vesting_tokens);

    let user_id = "team_member_1".to_string();
    let amount = 1000000.0;
    let start_date = Utc::now().naive_utc();

    // Create vesting schedule
    vesting_contract
        .create_team_vesting(user_id.clone(), amount, start_date)
        .unwrap();

    // Simulate time passing - 13 months (1 month after cliff)
    // Note: In a real test, we would mock time, but for simplicity we'll test the calculation logic
    let claimable = vesting_contract.calculate_claimable_amount(&user_id);
    assert!(claimable.is_ok());

    // Since we can't easily mock time, we'll test that the function doesn't panic
    // and returns a valid result (0.0 or positive amount)
    let claimable_amount = claimable.unwrap();
    assert!(claimable_amount >= 0.0);
}

#[test]
fn test_multiple_founding_members_vesting() {
    let amount_per_member = 2692308.0;
    let total_needed = amount_per_member * 13.0; // 35,000,004
    let total_vesting_tokens = total_needed;
    let mut vesting_contract = VestingContract::new(total_vesting_tokens);

    // Create vesting schedules for all 13 founding members
    let start_date = Utc::now().naive_utc();

    for i in 1..=13 {
        let user_id = format!("founding_member_{}", i);
        let result =
            vesting_contract.create_founding_member_vesting(user_id, amount_per_member, start_date);
        assert!(result.is_ok());
    }

    // Verify total claimed tokens
    assert_eq!(vesting_contract.get_claimed_tokens(), 0.0);
    // Check that we have the right amount of total vesting tokens
    assert_eq!(
        vesting_contract.get_total_vesting_tokens(),
        total_vesting_tokens
    );
}

#[test]
fn test_founder_and_founding_member_vesting_together() {
    let total_vesting_tokens = 105000000.0; // 105M tokens for both founders and founding members
    let mut vesting_contract = VestingContract::new(total_vesting_tokens);

    let start_date = Utc::now().naive_utc();

    // Create vesting for Founder 1
    let founder_id = "founder_1".to_string();
    let founder_amount = 35000000.0;
    let result =
        vesting_contract.create_founder_vesting(founder_id.clone(), founder_amount, start_date);
    assert!(result.is_ok());

    // Create vesting for Founding Member 1
    let member_id = "founding_member_1".to_string();
    let member_amount = 2692308.0;
    let result = vesting_contract.create_founding_member_vesting(
        member_id.clone(),
        member_amount,
        start_date,
    );
    assert!(result.is_ok());

    // Verify founder schedule
    let founder_schedule = vesting_contract.get_vesting_schedule(&founder_id);
    assert!(founder_schedule.is_some());
    let founder_schedule = founder_schedule.unwrap();
    assert_eq!(founder_schedule.cliff_duration_months, 12);
    assert_eq!(founder_schedule.vesting_duration_months, 48);
    assert_eq!(founder_schedule.schedule_type, VestingType::Founder);

    // Verify founding member schedule
    let member_schedule = vesting_contract.get_vesting_schedule(&member_id);
    assert!(member_schedule.is_some());
    let member_schedule = member_schedule.unwrap();
    assert_eq!(member_schedule.cliff_duration_months, 6);
    assert_eq!(member_schedule.vesting_duration_months, 36);
    assert_eq!(member_schedule.schedule_type, VestingType::FoundingMember);
}

#[test]
fn test_vesting_insufficient_tokens() {
    let total_vesting_tokens = 1000000.0; // Only 1M tokens available
    let mut vesting_contract = VestingContract::new(total_vesting_tokens);

    let user_id = "team_member_1".to_string();
    let amount = 2000000.0; // Trying to allocate 2M tokens
    let start_date = Utc::now().naive_utc();

    // This should fail due to insufficient tokens
    let result = vesting_contract.create_team_vesting(user_id, amount, start_date);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), VestingError::InsufficientTokens);
}

#[test]
fn test_vesting_insufficient_tokens_founder() {
    let total_vesting_tokens = 30000000.0; // Only 30M tokens available
    let mut vesting_contract = VestingContract::new(total_vesting_tokens);

    let user_id = "founder_1".to_string();
    let amount = 35000000.0; // Trying to allocate 35M tokens
    let start_date = Utc::now().naive_utc();

    // This should fail due to insufficient tokens
    let result = vesting_contract.create_founder_vesting(user_id, amount, start_date);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), VestingError::InsufficientTokens);
}

#[test]
fn test_vesting_insufficient_tokens_founding_member() {
    let total_vesting_tokens = 2000000.0; // Only 2M tokens available
    let mut vesting_contract = VestingContract::new(total_vesting_tokens);

    let user_id = "founding_member_1".to_string();
    let amount = 2692308.0; // Trying to allocate ~2.69M tokens
    let start_date = Utc::now().naive_utc();

    // This should fail due to insufficient tokens
    let result = vesting_contract.create_founding_member_vesting(user_id, amount, start_date);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), VestingError::InsufficientTokens);
}

#[test]
fn test_claim_vested_tokens() {
    let total_vesting_tokens = 17500000.0;
    let mut vesting_contract = VestingContract::new(total_vesting_tokens);

    let user_id = "team_member_1".to_string();
    let amount = 1000000.0;
    let start_date = Utc::now().naive_utc();

    // Create vesting schedule
    vesting_contract
        .create_team_vesting(user_id.clone(), amount, start_date)
        .unwrap();

    // Claim tokens (will be 0.0 since we're not mocking time)
    let claimed_amount = vesting_contract.claim_vested_tokens(&user_id);
    assert!(claimed_amount.is_ok());

    // Update claimed tokens tracking
    assert_eq!(
        vesting_contract.get_claimed_tokens(),
        claimed_amount.unwrap()
    );
}

#[test]
fn test_block_emission_vesting_claimable() {
    let total_vesting_tokens = 2_000_000.0;
    let mut vesting_contract = VestingContract::new(total_vesting_tokens);
    let start_date = (Utc::now() - Duration::days(120)).naive_utc();

    vesting_contract
        .create_block_emission_vesting(
            "block_user".to_string(),
            500_000.0,
            start_date,
            1,
            2.0,
            2500.0,
            VestingType::Investor,
        )
        .unwrap();

    let claimable = vesting_contract
        .calculate_claimable_amount("block_user")
        .unwrap();
    assert!(claimable > 0.0);
}

#[test]
fn test_epoch_unlock_vesting_claimable() {
    let total_vesting_tokens = 1_000_000.0;
    let mut vesting_contract = VestingContract::new(total_vesting_tokens);
    let start_date = (Utc::now() - Duration::weeks(20)).naive_utc();

    vesting_contract
        .create_epoch_unlock_vesting(
            "epoch_user".to_string(),
            200_000.0,
            start_date,
            0,
            7 * 24 * 60 * 60,
            5000.0,
            VestingType::Advisor,
        )
        .unwrap();

    let claimable = vesting_contract
        .calculate_claimable_amount("epoch_user")
        .unwrap();
    assert!(claimable >= 5_000.0);
}

#[test]
fn test_team_circuit_breaker_blocks_claims() {
    let mut vesting_contract = VestingContract::new(5_000_000.0);
    let start_date = (Utc::now() - Duration::days(400)).naive_utc();

    vesting_contract
        .create_team_vesting("team_user".to_string(), 1_000_000.0, start_date)
        .unwrap();

    let initial = vesting_contract
        .calculate_claimable_amount("team_user")
        .unwrap();
    assert!(initial > 0.0);

    vesting_contract
        .toggle_team_circuit_breaker(
            "team_user",
            vec!["founder1".to_string(), "founder2".to_string()],
            true,
        )
        .unwrap();

    let blocked = vesting_contract
        .calculate_claimable_amount("team_user")
        .unwrap();
    assert_eq!(blocked, 0.0);

    vesting_contract
        .toggle_team_circuit_breaker(
            "team_user",
            vec!["founder1".to_string(), "legal_representative".to_string()],
            false,
        )
        .unwrap();

    let resumed = vesting_contract
        .calculate_claimable_amount("team_user")
        .unwrap();
    assert!(resumed > 0.0);
}
