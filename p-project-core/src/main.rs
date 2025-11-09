use p_project_core::{
    models::User,
    utils::{generate_id, shorten_wallet_address},
};

fn main() {
    println!("Testing P-Project Core Components");

    // Test ID generation
    let user_id = generate_id();
    println!("Generated User ID: {}", user_id);

    // Test wallet address shortening
    let wallet_address = "0x742d35Cc6634C0532925a3b844Bc454e4438f44e".to_string();
    let short_address = shorten_wallet_address(&wallet_address);
    println!("Full wallet address: {}", wallet_address);
    println!("Short wallet address: {}", short_address);

    // Test user model
    let user = User {
        id: user_id,
        username: "testuser".to_string(),
        wallet_address,
        created_at: chrono::Utc::now().naive_utc(),
    };

    println!(
        "User: {} ({})",
        user.username,
        shorten_wallet_address(&user.wallet_address)
    );

    println!("Core components test completed successfully!");
}
