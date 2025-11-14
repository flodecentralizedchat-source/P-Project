use crate::l2_rollup::{L2Rollup, L2Transaction, RollupError};
use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};
use std::collections::HashMap;

// Message structure for cross-chain communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossChainMessage {
    pub message_id: String,
    pub source_chain: String,
    pub destination_chain: String,
    pub sender: String,
    pub recipient: String,
    pub amount: f64,
    pub token: String,
    pub payload: Vec<u8>,
    pub timestamp: u64,
    pub signature: String,
}

// Cross-chain bridge state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossChainBridgeState {
    pub locked_tokens: HashMap<String, f64>, // token_address -> amount
    pub processed_messages: HashMap<String, bool>, // message_id -> processed
    pub pending_messages: Vec<CrossChainMessage>,
}

// Cross-chain communication protocol
pub struct L2CrossChainProtocol {
    pub rollup: L2Rollup,
    pub bridge_state: CrossChainBridgeState,
    pub chain_id: String,
    pub connected_chains: Vec<String>,
}

impl L2CrossChainProtocol {
    pub fn new(rollup: L2Rollup, chain_id: String) -> Self {
        Self {
            rollup,
            bridge_state: CrossChainBridgeState {
                locked_tokens: HashMap::new(),
                processed_messages: HashMap::new(),
                pending_messages: Vec::new(),
            },
            chain_id,
            connected_chains: Vec::new(),
        }
    }

    /// Add a connected chain
    pub fn add_connected_chain(&mut self, chain_id: String) {
        if !self.connected_chains.contains(&chain_id) {
            self.connected_chains.push(chain_id);
        }
    }

    /// Lock tokens for cross-chain transfer
    pub fn lock_tokens(&mut self, user: String, token: String, amount: f64) -> Result<String, RollupError> {
        // Check user balance
        let user_balance = self.rollup.get_balance(&user);
        if user_balance < amount {
            return Err(RollupError::InsufficientBalance);
        }

        // Deduct tokens from user
        let mut hasher = Keccak256::new();
        hasher.update(user.as_bytes());
        hasher.update(token.as_bytes());
        hasher.update(&amount.to_le_bytes());
        let lock_id = format!("{:x}", hasher.finalize());

        // Update user balance
        // In a real implementation, we would update the rollup state
        // For now, we'll just track the locked tokens
        let current_locked = self.bridge_state.locked_tokens.get(&token).unwrap_or(&0.0);
        self.bridge_state.locked_tokens.insert(token.clone(), current_locked + amount);

        Ok(lock_id)
    }

    /// Create cross-chain message
    pub fn create_cross_chain_message(
        &mut self,
        source_chain: String,
        destination_chain: String,
        sender: String,
        recipient: String,
        amount: f64,
        token: String,
        payload: Vec<u8>,
    ) -> Result<CrossChainMessage, RollupError> {
        // Verify chains are connected
        if !self.connected_chains.contains(&destination_chain) {
            return Err(RollupError::InvalidTransaction);
        }

        // Lock tokens
        let lock_id = self.lock_tokens(sender.clone(), token.clone(), amount)?;

        let message = CrossChainMessage {
            message_id: lock_id,
            source_chain,
            destination_chain,
            sender,
            recipient,
            amount,
            token,
            payload,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            signature: String::new(), // In a real implementation, this would be a cryptographic signature
        };

        self.bridge_state.pending_messages.push(message.clone());
        Ok(message)
    }

    /// Process incoming cross-chain message
    pub fn process_incoming_message(&mut self, message: CrossChainMessage) -> Result<(), RollupError> {
        // Check if message was already processed
        if self.bridge_state.processed_messages.contains_key(&message.message_id) {
            return Err(RollupError::InvalidTransaction);
        }

        // Verify message signature
        // In a real implementation, we would verify the cryptographic signature
        // For now, we'll assume it's valid

        // Mint tokens to recipient
        let recipient_balance = self.rollup.get_balance(&message.recipient);
        
        // Create a transaction to mint tokens
        let transaction = L2Transaction {
            from: "bridge".to_string(), // Special bridge address
            to: message.recipient.clone(),
            amount: message.amount,
            nonce: 0, // Bridge transactions don't use nonce
            signature: message.signature.clone(),
            timestamp: chrono::Utc::now().naive_utc(),
        };

        // Add transaction to rollup
        self.rollup.add_transaction(transaction)?;

        // Mark message as processed
        self.bridge_state.processed_messages.insert(message.message_id, true);

        Ok(())
    }

    /// Release locked tokens (in case of failed transfer)
    pub fn release_tokens(&mut self, user: String, token: String, amount: f64) -> Result<(), RollupError> {
        let current_locked = self.bridge_state.locked_tokens.get(&token).unwrap_or(&0.0);
        if *current_locked < amount {
            return Err(RollupError::InsufficientBalance);
        }

        self.bridge_state.locked_tokens.insert(token.clone(), current_locked - amount);

        // Add tokens back to user balance
        let transaction = L2Transaction {
            from: "bridge".to_string(), // Special bridge address
            to: user,
            amount,
            nonce: 0,
            signature: String::new(),
            timestamp: chrono::Utc::now().naive_utc(),
        };

        self.rollup.add_transaction(transaction)?;
        Ok(())
    }

    /// Get bridge status
    pub fn get_bridge_status(&self) -> CrossChainBridgeState {
        self.bridge_state.clone()
    }

    /// Submit batch of cross-chain messages
    pub fn submit_message_batch(&mut self) -> Result<Vec<CrossChainMessage>, RollupError> {
        if self.bridge_state.pending_messages.is_empty() {
            return Err(RollupError::BatchSubmissionFailed);
        }

        let messages = self.bridge_state.pending_messages.clone();
        self.bridge_state.pending_messages.clear();

        // In a real implementation, these messages would be submitted to the destination chains
        // For now, we'll just return them

        Ok(messages)
    }

    /// Verify message integrity
    pub fn verify_message(&self, message: &CrossChainMessage) -> bool {
        // In a real implementation, we would verify the cryptographic signature
        // and check that the message matches the lock event on the source chain
        // For now, we'll just do a basic validation
        
        !message.sender.is_empty() && 
        !message.recipient.is_empty() && 
        message.amount > 0.0 &&
        !message.token.is_empty()
    }
}