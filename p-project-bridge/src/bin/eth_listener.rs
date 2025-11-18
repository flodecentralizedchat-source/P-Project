//! Ethereum Event Listener Binary
//! 
//! This binary listens for events from the Ethereum Bridge contract and processes them
//! for cross-chain transfers.

use p_project_bridge::EthEventListener;
use std::str::FromStr;
use web3::{
    transports::Http,
    types::Address,
    Web3,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Ethereum Event Listener...");
    
    // Get configuration from environment variables
    let rpc_url = std::env::var("ETHEREUM_RPC_URL")
        .unwrap_or_else(|_| "http://localhost:8545".to_string());
    
    let bridge_contract_address = std::env::var("BRIDGE_CONTRACT_ADDRESS")
        .unwrap_or_else(|_| "0x0000000000000000000000000000000000000000".to_string());
    
    // Create HTTP transport
    let transport = Http::new(&rpc_url)?;
    let web3 = Web3::new(transport);
    
    // Parse bridge contract address
    let contract_address = Address::from_str(&bridge_contract_address)?;
    
    // Create event listener
    let mut listener = EthEventListener::new(web3, contract_address);
    
    // Run the listener loop
    listener.run_loop().await?;
    
    Ok(())
}