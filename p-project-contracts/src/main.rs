use p_project_contracts::{PProjectToken, StakingContract, AirdropContract};

fn main() {
    println!("Testing P-Project Contracts");
    
    // Test token contract
    let mut token_contract = PProjectToken::new(1000000.0, 0.01, 0.02); // 1M total supply, 1% burn, 2% rewards
    println!("Token contract created with total supply: {}", token_contract.get_total_supply());
    
    // Initialize distribution
    let allocations = vec![
        ("user1".to_string(), 100000.0),
        ("user2".to_string(), 200000.0),
        ("user3".to_string(), 150000.0),
    ];
    token_contract.initialize_distribution(allocations);
    
    println!("Initial balances:");
    println!("User 1: {}", token_contract.get_balance("user1"));
    println!("User 2: {}", token_contract.get_balance("user2"));
    println!("User 3: {}", token_contract.get_balance("user3"));
    
    // Test transfer
    match token_contract.transfer("user1", "user2", 5000.0) {
        Ok(_) => println!("Transfer successful"),
        Err(e) => println!("Transfer failed: {}", e),
    }
    
    println!("Balances after transfer:");
    println!("User 1: {}", token_contract.get_balance("user1"));
    println!("User 2: {}", token_contract.get_balance("user2"));
    
    // Test staking contract
    let mut staking_contract = StakingContract::new(0.1); // 10% annual reward rate
    match staking_contract.stake_tokens("user1".to_string(), 50000.0, 30) {
        Ok(_) => println!("Staking successful"),
        Err(e) => println!("Staking failed: {}", e),
    }
    
    println!("Total staked: {}", staking_contract.get_total_staked());
    
    // Test airdrop contract
    let mut airdrop_contract = AirdropContract::new(100000.0); // 100k tokens for airdrop
    let recipients = vec![
        ("user1".to_string(), 10000.0),
        ("user2".to_string(), 15000.0),
        ("user3".to_string(), 20000.0),
    ];
    
    match airdrop_contract.add_recipients(recipients) {
        Ok(_) => println!("Airdrop recipients added successfully"),
        Err(e) => println!("Failed to add airdrop recipients: {}", e),
    }
    
    println!("Airdrop status: {:?}", airdrop_contract.get_status());
    
    println!("Contracts test completed successfully!");
}