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
    PresaleClosed,
    PresaleWhitelistRequired,
    PresaleAllocationExceeded,
    PresaleTargetReached,
    ReserveNotLocked,
    DevelopmentMilestoneNotFound,
    DevelopmentMilestoneAlreadyReleased,
    DatabaseError(String),
    SerializationError(String),
    NGOAccountNotFound,
}

impl std::fmt::Display for TreasuryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TreasuryError::InsufficientFunds => write!(f, "Insufficient funds in treasury"),
            TreasuryError::InvalidAmount => write!(f, "Amount must be positive"),
            TreasuryError::TokenOperationFailed(msg) => {
                write!(f, "Token operation failed: {}", msg)
            }
            TreasuryError::PresaleClosed => write!(f, "Presale is not accepting new contributions"),
            TreasuryError::PresaleWhitelistRequired => {
                write!(
                    f,
                    "Address must be whitelisted before contributing to the presale"
                )
            }
            TreasuryError::PresaleAllocationExceeded => {
                write!(f, "Presale allocation has been depleted")
            }
            TreasuryError::PresaleTargetReached => {
                write!(f, "Presale funding target has already been met")
            }
            TreasuryError::ReserveNotLocked => {
                write!(f, "Reserve allocation is not currently locked for release")
            }
            TreasuryError::DevelopmentMilestoneNotFound => {
                write!(f, "Development milestone not found")
            }
            TreasuryError::DevelopmentMilestoneAlreadyReleased => {
                write!(f, "Development milestone already released")
            }
            TreasuryError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            TreasuryError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            TreasuryError::NGOAccountNotFound => write!(f, "NGO treasury account not found"),
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

/// Configuration for the presale/funding raise
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresaleProgram {
    pub allocation_percent: f64,
    pub token_allocation: f64,
    pub tokens_remaining: f64,
    pub funding_target: f64,
    pub price_per_token: f64,
    pub total_raised: f64,
    pub whitelist: HashMap<String, bool>,
    pub contributions: HashMap<String, f64>,
    pub public_phase: bool,
}

impl PresaleProgram {
    pub fn new() -> Self {
        Self {
            allocation_percent: 0.10,
            token_allocation: 0.0,
            tokens_remaining: 0.0,
            funding_target: 200000.0,
            price_per_token: 0.0,
            total_raised: 0.0,
            whitelist: HashMap::new(),
            contributions: HashMap::new(),
            public_phase: false,
        }
    }

    pub fn configure(&mut self, total_supply: f64, funding_target: f64) {
        self.funding_target = funding_target;
        let allocation = total_supply * self.allocation_percent;
        self.token_allocation = allocation;
        self.tokens_remaining = allocation;
        self.price_per_token = if allocation > 0.0 {
            funding_target / allocation
        } else {
            0.0
        };
        self.total_raised = 0.0;
        self.whitelist.clear();
        self.contributions.clear();
        self.public_phase = false;
    }

    pub fn can_contribute(&self, user_id: &str) -> bool {
        self.public_phase || self.whitelist.contains_key(user_id)
    }

    pub fn whitelist_user(&mut self, user_id: &str) {
        self.whitelist.insert(user_id.to_string(), true);
    }

    pub fn remove_whitelist_user(&mut self, user_id: &str) {
        self.whitelist.remove(user_id);
    }
}

/// Emergency reserve allocation locked for crash protection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReserveAllocation {
    pub locked_tokens: f64,
    pub lock_reason: Option<String>,
    pub is_locked: bool,
}

impl ReserveAllocation {
    pub fn new() -> Self {
        Self {
            locked_tokens: 0.0,
            lock_reason: None,
            is_locked: false,
        }
    }
}

/// Development fund with milestone-based releases
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevelopmentMilestone {
    pub name: String,
    pub description: String,
    pub release_amount: f64,
    pub completed: bool,
    pub completed_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevelopmentFund {
    pub total_allocation: f64,
    pub released_amount: f64,
    pub milestones: Vec<DevelopmentMilestone>,
}

impl DevelopmentFund {
    pub fn new() -> Self {
        Self {
            total_allocation: 0.0,
            released_amount: 0.0,
            milestones: Vec::new(),
        }
    }

    pub fn configure(&mut self, total_supply: f64) {
        self.total_allocation = total_supply * 0.03;
        self.released_amount = 0.0;
        self.milestones.clear();
    }

    pub fn remaining(&self) -> f64 {
        (self.total_allocation - self.released_amount).max(0.0)
    }

    pub fn schedule_milestone(&mut self, name: String, description: String, release_amount: f64) {
        let milestone = DevelopmentMilestone {
            name,
            description,
            release_amount,
            completed: false,
            completed_at: None,
        };
        self.milestones.push(milestone);
    }
}

// Buyback record structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuybackRecord {
    pub timestamp: NaiveDateTime,
    pub amount_spent: f64,
    pub tokens_bought: f64,
    pub price_per_token: f64,
}

// Structure for scheduled buybacks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuybackSchedule {
    pub timestamp: NaiveDateTime,
    pub amount: f64,
    pub target_price: f64,
    pub executed: bool,
}

/// Supported conditions that can trigger a buyback event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BuybackCondition {
    PriceDrop,   // Threshold measured in percentage drop (e.g., 5.0 => price dropped by 5%)
    VolumeSpike, // Threshold measured in percentage gain
    PriceBelow,  // Threshold measured in absolute price
    PriceAbove,  // Threshold measured in absolute price
}

/// Snapshot of market data provided to the treasury when evaluating triggers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketSnapshot {
    pub price: f64,
    pub volume: f64,
    pub price_change_percentage: f64,
    pub volume_change_percentage: f64,
}

// Structure for trigger-based buybacks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuybackTrigger {
    pub trigger_name: String,
    pub condition: BuybackCondition,
    pub threshold: f64, // Threshold value for the condition
    pub amount: f64,    // Amount to spend on buyback
    pub executed: bool,
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
    ngo_treasuries: HashMap<String, NGOTreasuryAccount>,
    // New fields for enhanced buyback mechanisms
    scheduled_buybacks: Vec<BuybackSchedule>, // Scheduled buyback programs
    auto_buyback_enabled: bool,               // Whether auto buybacks are enabled
    buyback_triggers: Vec<BuybackTrigger>,    // Trigger-based buyback programs
    // Funding controls
    presale_program: PresaleProgram,
    reserve_allocation: ReserveAllocation,
    development_fund: DevelopmentFund,
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

/// Records for NGO treasury operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NGOTreasuryRecord {
    pub timestamp: NaiveDateTime,
    pub amount: f64,
    pub record_type: String,
    pub description: String,
}

/// On-chain representation of an NGO treasury bucket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NGOTreasuryAccount {
    pub ngo_address: String,
    pub balance: f64,
    pub purpose: String,
    pub last_update: NaiveDateTime,
    pub records: Vec<NGOTreasuryRecord>,
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
            ngo_treasuries: HashMap::new(),
            // Initialize new fields for enhanced buyback mechanisms
            scheduled_buybacks: Vec::new(),
            auto_buyback_enabled: false,
            buyback_triggers: Vec::new(),
            presale_program: PresaleProgram::new(),
            reserve_allocation: ReserveAllocation::new(),
            development_fund: DevelopmentFund::new(),
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

    /// Register an NGO treasury account for on-chain budgeting
    pub fn register_ngo_treasury(
        &mut self,
        ngo_address: String,
        purpose: String,
    ) -> Result<(), TreasuryError> {
        if self.ngo_treasuries.contains_key(&ngo_address) {
            return Err(TreasuryError::InvalidAmount);
        }

        let now = Utc::now().naive_utc();
        self.ngo_treasuries.insert(
            ngo_address.clone(),
            NGOTreasuryAccount {
                ngo_address,
                balance: 0.0,
                purpose,
                last_update: now,
                records: Vec::new(),
            },
        );

        Ok(())
    }

    /// Fund an existing NGO treasury from the USD reserves
    pub fn fund_ngo_treasury(
        &mut self,
        ngo_address: &str,
        amount: f64,
    ) -> Result<(), TreasuryError> {
        if amount <= 0.0 {
            return Err(TreasuryError::InvalidAmount);
        }

        let current_balance = self.get_balance("USD");
        if amount > current_balance {
            return Err(TreasuryError::InsufficientFunds);
        }

        let account = self
            .ngo_treasuries
            .get_mut(ngo_address)
            .ok_or(TreasuryError::NGOAccountNotFound)?;

        self.reserves
            .insert("USD".to_string(), current_balance - amount);
        account.balance += amount;
        account.last_update = Utc::now().naive_utc();
        account.records.push(NGOTreasuryRecord {
            timestamp: Utc::now().naive_utc(),
            amount,
            record_type: "deposit".to_string(),
            description: "Funded NGO treasury".to_string(),
        });

        Ok(())
    }

    /// Withdraw tokens from an NGO treasury allocation for spending
    pub fn withdraw_from_ngo_treasury(
        &mut self,
        ngo_address: &str,
        amount: f64,
    ) -> Result<f64, TreasuryError> {
        if amount <= 0.0 {
            return Err(TreasuryError::InvalidAmount);
        }

        let account = self
            .ngo_treasuries
            .get_mut(ngo_address)
            .ok_or(TreasuryError::NGOAccountNotFound)?;

        if amount > account.balance {
            return Err(TreasuryError::InsufficientFunds);
        }

        account.balance -= amount;
        account.last_update = Utc::now().naive_utc();
        account.records.push(NGOTreasuryRecord {
            timestamp: Utc::now().naive_utc(),
            amount,
            record_type: "withdraw".to_string(),
            description: "NGO treasury withdrawal".to_string(),
        });

        Ok(amount)
    }

    /// Get a reference to an NGO treasury account
    pub fn get_ngo_treasury(&self, ngo_address: &str) -> Option<&NGOTreasuryAccount> {
        self.ngo_treasuries.get(ngo_address)
    }

    /// Retrieve all NGO treasuries managed by the DAO
    pub fn get_all_ngo_treasuries(&self) -> &HashMap<String, NGOTreasuryAccount> {
        &self.ngo_treasuries
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

    /// Add a scheduled buyback
    pub fn add_scheduled_buyback(
        &mut self,
        timestamp: NaiveDateTime,
        amount: f64,
        target_price: f64,
    ) -> Result<(), TreasuryError> {
        if amount <= 0.0 {
            return Err(TreasuryError::InvalidAmount);
        }

        if amount > self.get_balance("USD") {
            return Err(TreasuryError::InsufficientFunds);
        }

        let scheduled_buyback = BuybackSchedule {
            timestamp,
            amount,
            target_price,
            executed: false,
        };
        self.scheduled_buybacks.push(scheduled_buyback);

        Ok(())
    }

    /// Execute scheduled buybacks that are due
    pub fn execute_scheduled_buybacks(
        &mut self,
        token: &mut PProjectToken,
        current_token_price: f64,
    ) -> Result<f64, TreasuryError> {
        if !self.auto_buyback_enabled {
            return Ok(0.0);
        }

        let now = Utc::now().naive_utc();
        let mut total_tokens_bought = 0.0;

        for i in 0..self.scheduled_buybacks.len() {
            if !self.scheduled_buybacks[i].executed && self.scheduled_buybacks[i].timestamp <= now {
                if self.scheduled_buybacks[i].amount > 0.0
                    && self.scheduled_buybacks[i].amount <= self.get_balance("USD")
                {
                    // Calculate how many tokens we can buy
                    let tokens_to_buy =
                        self.scheduled_buybacks[i].amount / self.scheduled_buybacks[i].target_price;

                    // Record the buyback
                    let buyback_record = BuybackRecord {
                        timestamp: Utc::now().naive_utc(),
                        amount_spent: self.scheduled_buybacks[i].amount,
                        tokens_bought: tokens_to_buy,
                        price_per_token: self.scheduled_buybacks[i].target_price,
                    };

                    self.buyback_records.push(buyback_record);
                    self.total_buybacks += self.scheduled_buybacks[i].amount;
                    total_tokens_bought += tokens_to_buy;
                    self.scheduled_buybacks[i].executed = true;

                    // Deduct funds from treasury
                    let current_balance = self.get_balance("USD");
                    self.reserves.insert(
                        "USD".to_string(),
                        current_balance - self.scheduled_buybacks[i].amount,
                    );

                    // Burn the tokens to reduce supply (deflationary mechanism)
                    match token.burn_tokens(tokens_to_buy) {
                        Ok(_) => {
                            println!(
                                "Treasury bought and burned {} tokens at ${} each, spending ${}",
                                tokens_to_buy,
                                self.scheduled_buybacks[i].target_price,
                                self.scheduled_buybacks[i].amount
                            );
                        }
                        Err(e) => {
                            println!("Warning: Failed to burn tokens: {}", e);
                        }
                    }
                }
            }
        }

        Ok(total_tokens_bought)
    }

    /// Add a buyback trigger
    pub fn add_buyback_trigger(
        &mut self,
        trigger_name: String,
        condition: BuybackCondition,
        threshold: f64,
        amount: f64,
    ) -> Result<(), TreasuryError> {
        if amount <= 0.0 {
            return Err(TreasuryError::InvalidAmount);
        }

        if amount > self.get_balance("USD") {
            return Err(TreasuryError::InsufficientFunds);
        }

        if threshold < 0.0 {
            return Err(TreasuryError::InvalidAmount);
        }

        let buyback_trigger = BuybackTrigger {
            trigger_name,
            condition,
            threshold,
            amount,
            executed: false,
        };
        self.buyback_triggers.push(buyback_trigger);

        Ok(())
    }

    /// Check and execute buyback triggers based on market conditions
    pub fn check_buyback_triggers(
        &mut self,
        token: &mut PProjectToken,
        snapshot: &MarketSnapshot,
    ) -> Result<f64, TreasuryError> {
        if !self.auto_buyback_enabled {
            return Ok(0.0);
        }

        let mut total_tokens_bought = 0.0;

        for i in 0..self.buyback_triggers.len() {
            if !self.buyback_triggers[i].executed {
                let should_execute = match self.buyback_triggers[i].condition {
                    BuybackCondition::PriceDrop => {
                        snapshot.price_change_percentage <= -self.buyback_triggers[i].threshold
                    }
                    BuybackCondition::VolumeSpike => {
                        snapshot.volume_change_percentage >= self.buyback_triggers[i].threshold
                    }
                    BuybackCondition::PriceBelow => {
                        snapshot.price <= self.buyback_triggers[i].threshold
                    }
                    BuybackCondition::PriceAbove => {
                        snapshot.price >= self.buyback_triggers[i].threshold
                    }
                };

                if should_execute {
                    if self.buyback_triggers[i].amount > 0.0
                        && self.buyback_triggers[i].amount <= self.get_balance("USD")
                    {
                        // Calculate how many tokens we can buy
                        let tokens_to_buy = self.buyback_triggers[i].amount / snapshot.price;

                        // Record the buyback
                        let buyback_record = BuybackRecord {
                            timestamp: Utc::now().naive_utc(),
                            amount_spent: self.buyback_triggers[i].amount,
                            tokens_bought: tokens_to_buy,
                            price_per_token: snapshot.price,
                        };

                        self.buyback_records.push(buyback_record);
                        self.total_buybacks += self.buyback_triggers[i].amount;
                        total_tokens_bought += tokens_to_buy;
                        self.buyback_triggers[i].executed = true;

                        // Deduct funds from treasury
                        let current_balance = self.get_balance("USD");
                        self.reserves.insert(
                            "USD".to_string(),
                            current_balance - self.buyback_triggers[i].amount,
                        );

                        // Burn the tokens to reduce supply (deflationary mechanism)
                        match token.burn_tokens(tokens_to_buy) {
                            Ok(_) => {
                                println!(
                                    "Treasury bought and burned {} tokens at ${} each, spending ${}",
                                    tokens_to_buy, snapshot.price, self.buyback_triggers[i].amount
                                );
                            }
                            Err(e) => {
                                println!("Warning: Failed to burn tokens: {}", e);
                            }
                        }
                    }
                }
            }
        }

        Ok(total_tokens_bought)
    }

    /// Enable or disable auto buybacks
    pub fn set_auto_buyback_enabled(&mut self, enabled: bool) {
        self.auto_buyback_enabled = enabled;
    }

    /// Get all scheduled buybacks
    pub fn get_scheduled_buybacks(&self) -> &Vec<BuybackSchedule> {
        &self.scheduled_buybacks
    }

    /// Get all buyback triggers
    pub fn get_buyback_triggers(&self) -> &Vec<BuybackTrigger> {
        &self.buyback_triggers
    }

    /// Get total buybacks executed
    pub fn get_total_buybacks(&self) -> f64 {
        self.total_buybacks
    }

    /// Get all buyback records
    pub fn get_buyback_records(&self) -> &Vec<BuybackRecord> {
        &self.buyback_records
    }

    /// Read current reserve balance for a specific asset (e.g., USD)
    pub fn get_reserve_balance(&self, asset: &str) -> f64 {
        *self.reserves.get(asset).unwrap_or(&0.0)
    }

    /// Check if treasury is DAO controlled
    pub fn is_dao_controlled(&self) -> bool {
        self.dao_controlled
    }

    /// Set DAO control status
    pub fn set_dao_controlled(&mut self, controlled: bool) {
        self.dao_controlled = controlled;
    }

    /// Configure the presale/funding raise parameters
    pub fn configure_presale(&mut self, total_supply: f64, funding_target: f64) {
        self.presale_program.configure(total_supply, funding_target);
    }

    /// Add a user to the presale whitelist
    pub fn add_presale_whitelist(&mut self, user_id: &str) {
        self.presale_program.whitelist_user(user_id);
    }

    /// Remove a user from the presale whitelist
    pub fn remove_presale_whitelist(&mut self, user_id: &str) {
        self.presale_program.remove_whitelist_user(user_id);
    }

    /// Open the presale to the public
    pub fn open_presale_public_phase(&mut self) {
        self.presale_program.public_phase = true;
    }

    /// Attempt to contribute to the presale
    pub fn contribute_to_presale(
        &mut self,
        user_id: &str,
        amount: f64,
    ) -> Result<f64, TreasuryError> {
        if amount <= 0.0 {
            return Err(TreasuryError::InvalidAmount);
        }

        if self.presale_program.price_per_token <= 0.0 {
            return Err(TreasuryError::InvalidAmount);
        }

        if self.presale_program.total_raised >= self.presale_program.funding_target {
            return Err(TreasuryError::PresaleTargetReached);
        }

        if self.presale_program.tokens_remaining <= 0.0 {
            return Err(TreasuryError::PresaleAllocationExceeded);
        }

        if !self.presale_program.can_contribute(user_id) {
            return Err(TreasuryError::PresaleWhitelistRequired);
        }

        let tokens = amount / self.presale_program.price_per_token;
        if tokens > self.presale_program.tokens_remaining {
            return Err(TreasuryError::PresaleAllocationExceeded);
        }

        *self
            .presale_program
            .contributions
            .entry(user_id.to_string())
            .or_insert(0.0) += amount;
        self.presale_program.total_raised += amount;
        self.presale_program.tokens_remaining -= tokens;
        *self.reserves.entry("USD".to_string()).or_insert(0.0) += amount;

        Ok(tokens)
    }

    /// Get presale program details for telemetry
    pub fn get_presale_program(&self) -> &PresaleProgram {
        &self.presale_program
    }

    /// Lock the crash reserve allocation
    pub fn lock_reserve(&mut self, total_supply: f64, reason: String) {
        self.reserve_allocation.locked_tokens = total_supply * 0.15;
        self.reserve_allocation.lock_reason = Some(reason);
        self.reserve_allocation.is_locked = true;
    }

    /// Release the locked reserve allocation (only once)
    pub fn release_reserve(&mut self) -> Result<f64, TreasuryError> {
        if !self.reserve_allocation.is_locked || self.reserve_allocation.locked_tokens <= 0.0 {
            return Err(TreasuryError::ReserveNotLocked);
        }

        let amount = self.reserve_allocation.locked_tokens;
        self.reserve_allocation.locked_tokens = 0.0;
        self.reserve_allocation.is_locked = false;
        self.reserve_allocation.lock_reason = Some("Released after emergency".to_string());
        Ok(amount)
    }

    /// Check whether reserve is locked
    pub fn is_reserve_locked(&self) -> bool {
        self.reserve_allocation.is_locked
    }

    /// Configure the development fund allocation
    pub fn configure_development_fund(&mut self, total_supply: f64) {
        self.development_fund.configure(total_supply);
    }

    /// Schedule a milestone-based release for the development fund
    pub fn schedule_development_milestone(
        &mut self,
        name: String,
        description: String,
        release_amount: f64,
    ) {
        self.development_fund
            .schedule_milestone(name, description, release_amount);
    }

    /// Release tokens for a completed milestone
    pub fn release_development_milestone(&mut self, name: &str) -> Result<f64, TreasuryError> {
        let milestone = self
            .development_fund
            .milestones
            .iter_mut()
            .find(|milestone| milestone.name == name)
            .ok_or(TreasuryError::DevelopmentMilestoneNotFound)?;

        if milestone.completed {
            return Err(TreasuryError::DevelopmentMilestoneAlreadyReleased);
        }

        if milestone.release_amount > self.development_fund.remaining() {
            return Err(TreasuryError::InvalidAmount);
        }

        milestone.completed = true;
        milestone.completed_at = Some(Utc::now().naive_utc());
        self.development_fund.released_amount += milestone.release_amount;
        Ok(milestone.release_amount)
    }

    /// Get current development fund snapshot
    pub fn get_development_fund(&self) -> &DevelopmentFund {
        &self.development_fund
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

    /// Remaining rewards that can still be distributed
    pub fn remaining_rewards(&self) -> f64 {
        (self.total_rewards - self.distributed_rewards).max(0.0)
    }

    /// Distribute rewards for the given number of days to all participants.
    /// Caps distribution so total distributed never exceeds total_rewards.
    pub fn distribute_rewards(&mut self, days: f64) -> HashMap<String, f64> {
        let mut rewards: HashMap<String, f64> = HashMap::new();

        if !self.is_active() || self.participants.is_empty() || days <= 0.0 {
            return rewards;
        }

        let total_liquidity = self.get_total_liquidity();
        if total_liquidity <= 0.0 {
            return rewards;
        }

        // Compute raw rewards per participant
        let daily_reward_per_token = self.reward_rate / total_liquidity;
        let mut total_required = 0.0;
        let mut raw: Vec<(String, f64)> = Vec::new();
        for (user, liq) in self.participants.iter() {
            let r = daily_reward_per_token * *liq * days;
            if r > 0.0 {
                total_required += r;
                raw.push((user.clone(), r));
            }
        }

        let remaining = self.remaining_rewards();
        if total_required <= 0.0 || remaining <= 0.0 {
            return rewards;
        }

        // Scale rewards if needed to not exceed remaining
        let scale = if total_required > remaining {
            remaining / total_required
        } else {
            1.0
        };

        let mut actually_distributed = 0.0;
        for (user, r) in raw.into_iter() {
            let amt = r * scale;
            if amt > 0.0 {
                actually_distributed += amt;
                rewards.insert(user, amt);
            }
        }

        self.distributed_rewards += actually_distributed;
        rewards
    }
}

/// Summary report for publishing treasury state and allocations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreasuryReport {
    pub generated_at: NaiveDateTime,
    pub reserves: HashMap<String, f64>,
    pub allocations: Vec<TreasuryAllocation>,
    pub buyback_records: Vec<BuybackRecord>,
    pub total_buybacks: f64,
    pub pending_transactions: Vec<PendingTransaction>,
    pub ngo_treasuries: HashMap<String, NGOTreasuryAccount>,
}

impl Treasury {
    /// Generate a snapshot report of the current treasury state
    pub fn generate_report(&self) -> Result<TreasuryReport, TreasuryError> {
        let reserves = self.reserves.clone();
        let allocations = self.allocations.clone();
        let buyback_records = self.buyback_records.clone();
        let total_buybacks = self.total_buybacks;
        let pending_transactions = self
            .pending_transactions
            .values()
            .cloned()
            .collect::<Vec<_>>();
        let ngo_treasuries = self.ngo_treasuries.clone();

        Ok(TreasuryReport {
            generated_at: Utc::now().naive_utc(),
            reserves,
            allocations,
            buyback_records,
            total_buybacks,
            pending_transactions,
            ngo_treasuries,
        })
    }
}
