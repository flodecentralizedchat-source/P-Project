//! Solana Relayer Binary
//! 
//! This binary handles relaying events from the Ethereum Bridge contract to the Solana chain
//! by minting SPL tokens.

use p_project_bridge::SolanaRelayer;
use solana_sdk::{
    signature::{read_keypair_file, Keypair},
};
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Solana Relayer...");
    
    // Get configuration from environment variables
    let rpc_url = std::env::var("SOLANA_RPC_URL")
        .unwrap_or_else(|_| "http://localhost:8899".to_string());
    
    let payer_keypair_path = std::env::var("SOLANA_PAYER_KEYPAIR")
        .unwrap_or_else(|_| "~/.config/solana/id.json".to_string());
    
    let token_mint = std::env::var("SPL_TOKEN_MINT")
        .unwrap_or_else(|_| "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb".to_string());
    
    let bridge_authority = std::env::var("SOLANA_BRIDGE_AUTHORITY")
        .unwrap_or_else(|_| "BridgeAuthority11111111111111111111111111111111".to_string());
    
    // Load payer keypair
    let payer_keypair = read_keypair_file(&payer_keypair_path)
        .map_err(|e| format!("Failed to read keypair file: {}", e))?;
    
    // Create Solana relayer
    let relayer = SolanaRelayer::new(
        &rpc_url,
        payer_keypair,
        &token_mint,
        &bridge_authority,
    )?;
    
    // For now, just print a message indicating the relayer is ready
    println!("Solana Relayer is ready to process events");
    println!("RPC URL: {}", rpc_url);
    println!("Token Mint: {}", token_mint);
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