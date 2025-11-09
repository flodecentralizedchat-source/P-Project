use chrono::Utc;
use p_project_contracts::airdrop::{AirdropContract, MerkleTree};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing Airdrop Functionality");

    // Create a new airdrop
    let total_amount = 10000.0;
    let mut airdrop = AirdropContract::new(total_amount);

    println!("Created airdrop with ID: {}", airdrop.get_airdrop_id());

    // Add recipients
    let recipients = vec![
        ("user1".to_string(), 100.0),
        ("user2".to_string(), 200.0),
        ("user3".to_string(), 150.0),
    ];

    airdrop.add_recipients(recipients.clone()).unwrap();
    println!("Added {} recipients to airdrop", recipients.len());

    // Test claiming
    match airdrop.claim("user1") {
        Ok(amount) => println!("User1 claimed {} tokens", amount),
        Err(e) => println!("Failed to claim: {}", e),
    }

    // Check if claimed
    let is_claimed = airdrop.is_claimed("user1");
    println!("User1 claimed status: {}", is_claimed);

    // Get status
    let status = airdrop.get_status();
    println!(
        "Airdrop status: {} total, {} distributed, {} recipients, {} claimed",
        status.total_amount,
        status.distributed_amount,
        status.total_recipients,
        status.claimed_recipients
    );

    // Test time-limited airdrop
    let start_time = Utc::now().naive_utc();
    let end_time = start_time + chrono::Duration::days(7);
    let timed_airdrop = AirdropContract::new_timed(total_amount, start_time, end_time);
    println!(
        "Created time-limited airdrop. Active: {}",
        timed_airdrop.is_active()
    );

    // Test Merkle tree
    let leaves = vec![
        "user1".to_string(),
        "user2".to_string(),
        "user3".to_string(),
    ];
    let merkle_tree = MerkleTree::new(leaves);

    println!("Merkle tree root: {}", merkle_tree.get_root());

    // Test proof generation
    if let Some(proof) = merkle_tree.get_proof(0) {
        println!("Proof for user1: {:?}", proof);
    }

    println!("Airdrop tests completed successfully!");
    Ok(())
}
