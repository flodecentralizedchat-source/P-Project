use crate::airdrop::AirdropContract;
use crate::staking::StakingContract;
use crate::token::PProjectToken;
use crate::treasury::{LiquidityMiningProgram, Treasury};
use crate::vesting::VestingContract;
use chrono::Utc;

#[test]
fn test_full_tokenomics_implementation() {
    // Initialize all components
    let mut token = PProjectToken::new(350000000.0, 0.01, 0.005); // 350M tokens, 1% burn rate, 0.5% reward rate
                                                                  // Disable bot protection to avoid interference with rapid transfers in this test
    token.set_bot_protection(false);

    let mut treasury = Treasury::new();
    let mut vesting_contract = VestingContract::new(105000000.0); // 30% for vesting
    let mut staking_contract = StakingContract::new();
    let mut airdrop_contract = AirdropContract::new(52500000.0); // 15% for airdrop/community

    // Initialize token distribution
    let initial_allocations = vec![
        ("treasury".to_string(), 70000000.0),  // 20% for treasury
        ("dao".to_string(), 35000000.0),       // 10% for DAO
        ("liquidity".to_string(), 35000000.0), // 10% for initial liquidity (more than needed for tests)
    ];
    token.initialize_distribution(initial_allocations);

    // Test 1: Vesting Contracts Implementation
    println!("Testing Vesting Contracts...");

    // Create team vesting (12m cliff + 24m linear)
    let team_member = "team_member_1".to_string();
    let team_amount = 1000000.0;
    let start_date = Utc::now().naive_utc();
    // Check if we have enough tokens before creating vesting schedule
    assert!(vesting_contract.get_total_vesting_tokens() >= team_amount);
    vesting_contract
        .create_team_vesting(team_member.clone(), team_amount, start_date)
        .unwrap();

    // Create advisor vesting (6m cliff + 12m linear)
    let advisor = "advisor_1".to_string();
    let advisor_amount = 500000.0;
    // Check if we have enough tokens before creating vesting schedule
    assert!(vesting_contract.get_total_vesting_tokens() >= team_amount + advisor_amount);
    vesting_contract
        .create_advisor_vesting(advisor.clone(), advisor_amount, start_date)
        .unwrap();

    // Verify vesting schedules
    let team_schedule = vesting_contract.get_vesting_schedule(&team_member).unwrap();
    assert_eq!(team_schedule.cliff_duration_months, 12);
    assert_eq!(team_schedule.vesting_duration_months, 24);

    let advisor_schedule = vesting_contract.get_vesting_schedule(&advisor).unwrap();
    assert_eq!(advisor_schedule.cliff_duration_months, 6);
    assert_eq!(advisor_schedule.vesting_duration_months, 12);

    // Test 2: Staking Rewards Implementation
    println!("Testing Staking Rewards...");

    // Stake tokens with different durations to test reward calculation
    staking_contract
        .stake_tokens("staker1".to_string(), 100000.0, 365)
        .unwrap(); // 1 year
    staking_contract
        .stake_tokens("staker2".to_string(), 200000.0, 730)
        .unwrap(); // 2 years

    // Verify staking info
    let staker1_info = staking_contract.get_staking_info("staker1").unwrap();
    assert_eq!(staker1_info.amount, 100000.0);

    let staker2_info = staking_contract.get_staking_info("staker2").unwrap();
    assert_eq!(staker2_info.amount, 200000.0);

    // Test 3: Community Incentive Distribution
    println!("Testing Community Incentive Distribution...");

    // Add recipients for airdrop/community incentives
    let recipients = vec![
        ("community_user1".to_string(), 10000.0),
        ("community_user2".to_string(), 15000.0),
        ("community_user3".to_string(), 20000.0),
    ];
    airdrop_contract.add_recipients(recipients).unwrap();

    // Verify airdrop status
    let airdrop_status = airdrop_contract.get_status();
    assert_eq!(airdrop_status.total_recipients, 3);
    assert_eq!(airdrop_status.distributed_amount, 45000.0);

    // Test 4: DAO-controlled Allocations
    println!("Testing DAO-controlled Allocations...");

    // Add funds to treasury
    treasury.add_funds("USD".to_string(), 1000000.0).unwrap();

    // Allocate funds for different purposes (leave some for buyback)
    treasury
        .allocate_funds(
            "Ecosystem Grants".to_string(),
            200000.0,
            "Funding ecosystem development".to_string(),
        )
        .unwrap();
    treasury
        .allocate_funds(
            "Charity Endowment".to_string(),
            200000.0,
            "Charitable initiatives".to_string(),
        )
        .unwrap();
    treasury
        .allocate_funds(
            "Treasury Reserve".to_string(),
            500000.0,
            "General treasury reserve".to_string(),
        )
        .unwrap();

    // Verify allocations
    assert_eq!(treasury.get_balance("USD"), 100000.0); // 100K left for buyback

    // Test 5: Liquidity Locking
    println!("Testing Liquidity Locking...");

    // Add liquidity to a pool (use only part of the liquidity allocation)
    token
        .add_liquidity("pool1".to_string(), "liquidity", 17500000.0)
        .unwrap();

    // Lock liquidity for 24 months
    token
        .lock_liquidity("pool1".to_string(), 17500000.0)
        .unwrap();

    // Verify liquidity is locked
    assert!(token.is_liquidity_locked("pool1"));

    // Test 6: Deflationary Pressure (Dynamic Burn Rates)
    println!("Testing Deflationary Pressure...");

    // Give user1 enough tokens for all transfers
    token.transfer("liquidity", "user1", 200000.0).unwrap();

    // Check initial state
    let initial_supply = token.get_total_supply();

    // Transfer with base burn rate (user1 has no activity yet)
    token.transfer("user1", "user2", 10000.0).unwrap(); // Normal transfer from user1 to user2
    let supply_after_normal_transfer = token.get_total_supply();

    // Check user1's activity count
    let user1_activity_before = token.get_user_activity("user1");
    println!(
        "User1 activity before high activity transfers: {}",
        user1_activity_before
    );

    // Simulate high activity for a user (more than 10 transactions to trigger increased burn rate)
    for _ in 0..20 {
        // Increase to 20 transactions to ensure we exceed the threshold
        token.transfer("user1", "user2", 1000.0).unwrap();
    }

    // Check user1's activity count after transfers
    let user1_activity_after = token.get_user_activity("user1");
    println!(
        "User1 activity after high activity transfers: {}",
        user1_activity_after
    );

    // Check total transactions
    let total_transactions = token.get_total_transactions();
    println!("Total transactions: {}", total_transactions);

    // Transfer same amount with increased burn rate due to high activity
    let supply_before_high_activity_transfer = supply_after_normal_transfer;
    token.transfer("user1", "user2", 10000.0).unwrap(); // Same amount as normal transfer
    let supply_after_high_activity_transfer = token.get_total_supply();

    // Verify that burn rate increased during high activity
    let normal_burn = initial_supply - supply_after_normal_transfer;
    let high_activity_burn =
        supply_before_high_activity_transfer - supply_after_high_activity_transfer;
    println!(
        "Normal burn: {}, High activity burn: {}",
        normal_burn, high_activity_burn
    );

    // The high activity burn should be greater than the normal burn
    assert!(
        high_activity_burn > normal_burn,
        "Expected high activity burn ({}) to be greater than normal burn ({}), but it wasn't",
        high_activity_burn,
        normal_burn
    );

    // Test 7: Buyback Programs
    println!("Testing Buyback Programs...");

    // Execute buyback with treasury funds
    let tokens_bought = treasury
        .execute_buyback(&mut token, 100000.0, 0.001)
        .unwrap();
    assert_eq!(tokens_bought, 100000000.0); // 100000 USD / 0.001 per token

    // Verify treasury balance decreased
    assert_eq!(treasury.get_balance("USD"), 0.0); // All funds spent

    // Test 8: Liquidity Mining
    println!("Testing Liquidity Mining...");

    // Create liquidity mining program
    let mut mining_program = LiquidityMiningProgram::new(
        "pool1".to_string(),
        "P".to_string(),
        1000.0,  // 1000 tokens per day
        30,      // 30 days
        30000.0, // 30000 total tokens
    );

    // Add participants with different liquidity amounts
    mining_program.add_participant("miner1".to_string(), 10000.0);
    mining_program.add_participant("miner2".to_string(), 20000.0);
    mining_program.add_participant("miner3".to_string(), 30000.0);

    // Calculate rewards for different participants
    let miner1_rewards = mining_program.calculate_rewards("miner1", 7.0); // 7 days
    let miner2_rewards = mining_program.calculate_rewards("miner2", 7.0); // 7 days
    let miner3_rewards = mining_program.calculate_rewards("miner3", 7.0); // 7 days

    // Verify proportional rewards
    assert!(miner2_rewards > miner1_rewards);
    assert!(miner3_rewards > miner2_rewards);

    // Test 9: Partnership Integrations
    println!("Testing Partnership Integrations...");

    // Simulate partnership integration by transferring tokens to partner platforms
    // Account for 1% burn rate on transfers
    token
        .transfer("liquidity", "partner_platform_1", 500000.0)
        .unwrap();
    token
        .transfer("liquidity", "partner_platform_2", 300000.0)
        .unwrap();

    // Verify partner platforms received tokens (accounting for 1% burn rate)
    let partner1_balance = token.get_balance("partner_platform_1");
    let partner2_balance = token.get_balance("partner_platform_2");
    println!(
        "Partner 1 balance: {}, Partner 2 balance: {}",
        partner1_balance, partner2_balance
    );
    assert!((partner1_balance - 495000.0).abs() < 1.0); // 500,000 - 1% burn = 495,000
    assert!((partner2_balance - 297000.0).abs() < 1.0); // 300,000 - 1% burn = 297,000

    // Test 10: Real-World Use Cases
    println!("Testing Real-World Use Cases...");

    // Simulate real-world use case: User pays for service with tokens
    token.transfer("user1", "service_provider", 5000.0).unwrap();

    // Service provider stakes earnings
    staking_contract
        .stake_tokens("service_provider".to_string(), 5000.0, 180)
        .unwrap();

    // Verify service provider has staked tokens
    let provider_staking = staking_contract
        .get_staking_info("service_provider")
        .unwrap();
    assert_eq!(provider_staking.amount, 5000.0);

    println!("All features tested successfully!");

    // Final verification of total supply reduction due to burn mechanisms
    let final_supply = token.get_total_supply();
    assert!(final_supply < 350000000.0); // Should be less due to burns

    println!("Initial supply: 350,000,000");
    println!("Final supply: {}", final_supply);
    println!("Tokens burned: {}", 350000000.0 - final_supply);
}
