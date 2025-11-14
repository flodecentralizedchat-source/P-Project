use crate::token::PProjectToken;
use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Custom error types for treasury operations
#[derive(Debug, Clone, PartialEq)]
pub enum TreasuryError {
    InsufficientFunds,
    InvalidAmount,
    TokenOperationFailed(String),
    DatabaseError(String),
    SerializationError(String),
}

impl std::fmt::Display for TreasuryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TreasuryError::InsufficientFunds => write!(f, "Insufficient funds in treasury"),
            TreasuryError::InvalidAmount => write!(f, "Amount must be positive"),
            TreasuryError::TokenOperationFailed(msg) => {
                write!(f, "Token operation failed: {}", msg)
            }
            TreasuryError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            TreasuryError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
        }
    }
}

impl std::error::Error for TreasuryError {}

// Treasury allocation structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreasuryAllocation {
    pub name: String,
    pub amount: f64,
    pub purpose: String,
}

// Buyback record structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuybackRecord {
    pub timestamp: NaiveDateTime,
    pub amount_spent: f64,
    pub tokens_bought: f64,
    pub price_per_token: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Treasury {
    reserves: HashMap<String, f64>, // asset -> amount
    allocations: Vec<TreasuryAllocation>,
    buyback_records: Vec<BuybackRecord>,
    total_buybacks: f64,
    dao_controlled: bool,
    // New fields for multi-sig approval
    multisig_signers: Vec<String>, // List of authorized signers
    multisig_required: usize,      // Number of signatures required
    pending_transactions: HashMap<String, PendingTransaction>, // tx_id -> transaction
}

// Structure for pending multi-sig transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingTransaction {
    pub tx_id: String,
    pub description: String,
    pub amount: f64,
    pub asset: String,
    pub destination: String,
    pub creator: String,
    pub signatures: Vec<String>, // List of signers who approved
    pub created_at: NaiveDateTime,
    pub executed: bool,
}

impl Treasury {
    pub fn new() -> Self {
        Self {
            reserves: HashMap::new(),
            allocations: Vec::new(),
            buyback_records: Vec::new(),
            total_buybacks: 0.0,
            dao_controlled: true, // Controlled by DAO by default
            multisig_signers: vec![
                "signer1".to_string(),
                "signer2".to_string(),
                "signer3".to_string(),
                "signer4".to_string(),
                "signer5".to_string(),
            ], // Default signers
            multisig_required: 3, // Default 3-of-5 multi-sig
            pending_transactions: HashMap::new(),
        }
    }

    /// Set multi-sig signers
    pub fn set_multisig_signers(&mut self, signers: Vec<String>, required: usize) {
        self.multisig_signers = signers;
        self.multisig_required = required;
    }

    /// Get multi-sig signers
    pub fn get_multisig_signers(&self) -> &Vec<String> {
        &self.multisig_signers
    }

    /// Get required signatures
    pub fn get_required_signatures(&self) -> usize {
        self.multisig_required
    }

    /// Create a pending transaction that requires multi-sig approval
    pub fn create_pending_transaction(
        &mut self,
        description: String,
        amount: f64,
        asset: String,
        destination: String,
        creator: String,
    ) -> Result<String, TreasuryError> {
        if amount <= 0.0 {
            return Err(TreasuryError::InvalidAmount);
        }

        if amount > self.get_balance(&asset) {
            return Err(TreasuryError::InsufficientFunds);
        }

        let tx_id = format!("tx_{}", Utc::now().timestamp());
        let pending_tx = PendingTransaction {
            tx_id: tx_id.clone(),
            description,
            amount,
            asset: asset.clone(),
            destination,
            creator,
            signatures: Vec::new(),
            created_at: Utc::now().naive_utc(),
            executed: false,
        };

        self.pending_transactions.insert(tx_id.clone(), pending_tx);
        Ok(tx_id)
    }

    /// Sign a pending transaction
    pub fn sign_transaction(&mut self, tx_id: &str, signer: String) -> Result<bool, TreasuryError> {
        // Check if signer is authorized
        if !self.multisig_signers.contains(&signer) {
            return Err(TreasuryError::InvalidAmount); // Using this for unauthorized signer
        }

        // Check if transaction exists and is not executed
        if let Some(pending_tx) = self.pending_transactions.get_mut(tx_id) {
            if pending_tx.executed {
                return Err(TreasuryError::InvalidAmount); // Using this for already executed
            }

            // Check if signer already signed
            if pending_tx.signatures.contains(&signer) {
                return Ok(false); // Already signed
            }

            // Add signature
            pending_tx.signatures.push(signer);

            // Check if we have enough signatures to execute
            if pending_tx.signatures.len() >= self.multisig_required {
                // Execute the transaction
                self.execute_pending_transaction(tx_id)?;
                Ok(true) // Executed
            } else {
                Ok(false) // Not enough signatures yet
            }
        } else {
            Err(TreasuryError::InvalidAmount) // Using this for transaction not found
        }
    }

    /// Execute a pending transaction
    fn execute_pending_transaction(&mut self, tx_id: &str) -> Result<(), TreasuryError> {
        // First, check if the transaction exists and is not executed
        let (asset_name, amount) = {
            let pending_tx = match self.pending_transactions.get(tx_id) {
                Some(tx) => tx,
                None => return Err(TreasuryError::InvalidAmount), // Transaction not found
            };

            if pending_tx.executed {
                return Err(TreasuryError::InvalidAmount); // Already executed
            }

            (pending_tx.asset.clone(), pending_tx.amount)
        };

        // Now we can borrow self immutably to get the balance
        let current_balance = self.get_balance(&asset_name);

        if current_balance < amount {
            return Err(TreasuryError::InsufficientFunds);
        }

        // Deduct funds from treasury
        self.reserves.insert(asset_name, current_balance - amount);

        // Mark as executed
        if let Some(pending_tx) = self.pending_transactions.get_mut(tx_id) {
            pending_tx.executed = true;
        }

        Ok(())
    }

    /// Get pending transactions
    pub fn get_pending_transactions(&self) -> &HashMap<String, PendingTransaction> {
        &self.pending_transactions
    }

    /// Add funds to treasury
    pub fn add_funds(&mut self, asset: String, amount: f64) -> Result<(), TreasuryError> {
        if amount <= 0.0 {
            return Err(TreasuryError::InvalidAmount);
        }

        let current_balance = self.reserves.get(&asset).unwrap_or(&0.0);
        self.reserves.insert(asset, current_balance + amount);
        Ok(())
    }

    /// Get treasury balance for an asset
    pub fn get_balance(&self, asset: &str) -> f64 {
        *self.reserves.get(asset).unwrap_or(&0.0)
    }

    /// Allocate funds for specific purposes
    pub fn allocate_funds(
        &mut self,
        name: String,
        amount: f64,
        purpose: String,
    ) -> Result<(), TreasuryError> {
        if amount <= 0.0 {
            return Err(TreasuryError::InvalidAmount);
        }

        if amount > self.get_balance("USD") {
            return Err(TreasuryError::InsufficientFunds);
        }

        let allocation = TreasuryAllocation {
            name,
            amount,
            purpose,
        };

        self.allocations.push(allocation);

        // Deduct from reserves
        let current_balance = self.get_balance("USD");
        self.reserves
            .insert("USD".to_string(), current_balance - amount);

        Ok(())
    }

    /// Execute token buyback program with enhanced functionality
    pub fn execute_buyback(
        &mut self,
        token: &mut PProjectToken,
        amount_to_spend: f64,
        current_token_price: f64,
    ) -> Result<f64, TreasuryError> {
        if amount_to_spend <= 0.0 {
            return Err(TreasuryError::InvalidAmount);
        }

        if amount_to_spend > self.get_balance("USD") {
            return Err(TreasuryError::InsufficientFunds);
        }

        // Calculate how many tokens we can buy
        let tokens_to_buy = amount_to_spend / current_token_price;

        // For this implementation, we'll simulate buying tokens from the market
        // In a real implementation, this would interact with a DEX or market
        let buyback_record = BuybackRecord {
            timestamp: Utc::now().naive_utc(),
            amount_spent: amount_to_spend,
            tokens_bought: tokens_to_buy,
            price_per_token: current_token_price,
        };

        self.buyback_records.push(buyback_record);
        self.total_buybacks += amount_to_spend;

        // Deduct funds from treasury
        let current_balance = self.get_balance("USD");
        self.reserves
            .insert("USD".to_string(), current_balance - amount_to_spend);

        // Burn the tokens to reduce supply (deflationary mechanism)
        // In a real implementation, we would acquire tokens and burn them
        match token.burn_tokens(tokens_to_buy) {
            Ok(_) => {
                println!(
                    "Treasury bought and burned {} tokens at ${} each, spending ${}",
                    tokens_to_buy, current_token_price, amount_to_spend
                );
            }
            Err(e) => {
                println!("Warning: Failed to burn tokens: {}", e);
            }
        }

        Ok(tokens_to_buy)
    }

    /// Get total buybacks executed
    pub fn get_total_buybacks(&self) -> f64 {
        self.total_buybacks
    }

    /// Get all buyback records
    pub fn get_buyback_records(&self) -> &Vec<BuybackRecord> {
        &self.buyback_records
    }

    /// Check if treasury is DAO controlled
    pub fn is_dao_controlled(&self) -> bool {
        self.dao_controlled
    }

    /// Set DAO control status
    pub fn set_dao_controlled(&mut self, controlled: bool) {
        self.dao_controlled = controlled;
    }
}

// Liquidity mining program structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityMiningProgram {
    pub pool_id: String,
    pub reward_token: String,
    pub reward_rate: f64, // rewards per day
    pub start_date: NaiveDateTime,
    pub end_date: NaiveDateTime,
    pub total_rewards: f64,
    pub distributed_rewards: f64,
    pub participants: HashMap<String, f64>, // user_id -> liquidity_amount
}

impl LiquidityMiningProgram {
    pub fn new(
        pool_id: String,
        reward_token: String,
        reward_rate: f64,
        duration_days: i64,
        total_rewards: f64,
    ) -> Self {
        let start_date = Utc::now().naive_utc();
        let end_date = start_date + chrono::Duration::days(duration_days);

        Self {
            pool_id,
            reward_token,
            reward_rate,
            start_date,
            end_date,
            total_rewards,
            distributed_rewards: 0.0,
            participants: HashMap::new(),
        }
    }

    /// Add a participant to the liquidity mining program
    pub fn add_participant(&mut self, user_id: String, liquidity_amount: f64) {
        self.participants.insert(user_id, liquidity_amount);
    }

    /// Remove a participant from the liquidity mining program
    pub fn remove_participant(&mut self, user_id: &str) -> Option<f64> {
        self.participants.remove(user_id)
    }

    /// Calculate rewards for a participant
    pub fn calculate_rewards(&self, user_id: &str, days_participated: f64) -> f64 {
        if let Some(liquidity_amount) = self.participants.get(user_id) {
            // Rewards based on liquidity amount and time participated
            let daily_reward_per_token = self.reward_rate / self.get_total_liquidity();
            daily_reward_per_token * liquidity_amount * days_participated
        } else {
            0.0
        }
    }

    /// Get total liquidity in the program
    pub fn get_total_liquidity(&self) -> f64 {
        self.participants.values().sum()
    }

    /// Check if the program is active
    pub fn is_active(&self) -> bool {
        let now = Utc::now().naive_utc();
        now >= self.start_date && now <= self.end_date
    }
}
