//! Sui Relayer Binary
//! 
//! This binary handles relaying events from the Ethereum Bridge contract to the Sui chain
//! by minting or burning P-Project tokens.

use p_project_bridge::SuiRelayer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Sui Relayer...");
    
    // Get configuration from environment variables
    let rpc_url = std::env::var("SUI_RPC_URL")
        .unwrap_or_else(|_| "https://fullnode.devnet.sui.io:443".to_string());
    
    let bridge_authority = std::env::var("SUI_BRIDGE_AUTHORITY")
        .unwrap_or_else(|_| "0x0000000000000000000000000000000000000000000000000000000000000000".to_string());
    
    // Create Sui relayer
    let relayer = SuiRelayer::new(&rpc_url, &bridge_authority);
    
    // For now, just print a message indicating the relayer is ready
    println!("Sui Relayer is ready to process events");
    println!("RPC URL: {}", rpc_url);
    println!("Bridge Authority: {}", bridge_authority);
    
    // In a real implementation, you would:
    // 1. Connect to a message queue or database to get events
    // 2. Process each event by calling the appropriate methods on the relayer
    // 3. Handle errors and retries
    
    // Keep the process running
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    }
}