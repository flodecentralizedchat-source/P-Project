use chrono::Utc;
use p_project_core::models::{TokenTransaction, TransactionType};
use p_project_core::utils::generate_id;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Custom error types for token operations
#[derive(Debug, Clone, PartialEq)]
pub enum TokenError {
    InsufficientBalance,
    InsufficientFrozenBalance,
    TransferLimitExceeded(f64),
    InvalidAmount,
    DatabaseError(String),
    SerializationError(String),
}

impl std::fmt::Display for TokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenError::InsufficientBalance => write!(f, "Insufficient balance"),
            TokenError::InsufficientFrozenBalance => write!(f, "Insufficient frozen balance"),
            TokenError::TransferLimitExceeded(limit) => {
                write!(f, "Transfer amount exceeds maximum limit of {}", limit)
            }
            TokenError::InvalidAmount => write!(f, "Amount must be positive"),
            TokenError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            TokenError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
        }
    }
}

impl std::error::Error for TokenError {}

// Event structure for token operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenEvent {
    pub event_type: String,
    pub user_id: String,
    pub amount: f64,
    pub timestamp: chrono::NaiveDateTime,
    pub details: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PProjectToken {
    total_supply: f64,
    balances: HashMap<String, f64>,         // user_id -> balance
    frozen_balances: HashMap<String, f64>,  // user_id -> frozen balance
    burn_rate: f64,                         // percentage to burn on each transaction
    reward_rate: f64,                       // percentage to distribute to holders
    holders: Vec<String>,                   // list of user_ids who hold tokens
    max_transfer_limit: f64,                // anti-whale mechanism
    transaction_log: Vec<TokenTransaction>, // audit trail
    event_log: Vec<TokenEvent>,             // event logging
    liquidity_pools: HashMap<String, f64>,  // pool_id -> liquidity amount
}

impl PProjectToken {
    pub fn new(total_supply: f64, burn_rate: f64, reward_rate: f64) -> Self {
        Self {
            total_supply,
            balances: HashMap::new(),
            frozen_balances: HashMap::new(),
            burn_rate,
            reward_rate,
            holders: Vec::new(),
            max_transfer_limit: total_supply * 0.05, // 5% of total supply as default limit
            transaction_log: Vec::new(),
            event_log: Vec::new(),
            liquidity_pools: HashMap::new(),
        }
    }

    /// Set maximum transfer limit for anti-whale mechanism
    pub fn set_max_transfer_limit(&mut self, limit: f64) {
        self.max_transfer_limit = limit;
        self.log_event(
            "CONFIG_UPDATE".to_string(),
            "SYSTEM".to_string(),
            0.0,
            format!("Max transfer limit set to {}", limit),
        );
    }

    /// Get maximum transfer limit
    pub fn get_max_transfer_limit(&self) -> f64 {
        self.max_transfer_limit
    }

    /// Log token events
    fn log_event(&mut self, event_type: String, user_id: String, amount: f64, details: String) {
        let event = TokenEvent {
            event_type,
            user_id,
            amount,
            timestamp: Utc::now().naive_utc(),
            details,
        };
        self.event_log.push(event);
    }

    /// Get event log
    pub fn get_event_log(&self) -> &Vec<TokenEvent> {
        &self.event_log
    }

    /// Add liquidity to a pool
    pub fn add_liquidity(
        &mut self,
        pool_id: String,
        user_id: &str,
        amount: f64,
    ) -> Result<(), TokenError> {
        // Check if user has enough balance
        let balance = self.balances.get(user_id).unwrap_or(&0.0);
        if *balance < amount {
            return Err(TokenError::InsufficientBalance);
        }

        // Update user balance
        let new_balance = balance - amount;
        self.balances.insert(user_id.to_string(), new_balance);

        // Update liquidity pool
        let current_liquidity = self.liquidity_pools.get(&pool_id).unwrap_or(&0.0);
        let new_liquidity = current_liquidity + amount;
        self.liquidity_pools.insert(pool_id.clone(), new_liquidity);

        // Log event
        self.log_event(
            "LIQUIDITY_ADDED".to_string(),
            user_id.to_string(),
            amount,
            format!("Added liquidity to pool {}", pool_id),
        );

        Ok(())
    }

    /// Remove liquidity from a pool
    pub fn remove_liquidity(
        &mut self,
        pool_id: String,
        user_id: &str,
        amount: f64,
    ) -> Result<(), TokenError> {
        // Check if pool has enough liquidity
        let liquidity = self.liquidity_pools.get(&pool_id).unwrap_or(&0.0);
        if *liquidity < amount {
            return Err(TokenError::InsufficientBalance);
        }

        // Update liquidity pool
        let new_liquidity = liquidity - amount;
        self.liquidity_pools.insert(pool_id.clone(), new_liquidity);

        // Update user balance
        let balance = self.balances.get(user_id).unwrap_or(&0.0);
        let new_balance = balance + amount;
        self.balances.insert(user_id.to_string(), new_balance);

        // Log event
        self.log_event(
            "LIQUIDITY_REMOVED".to_string(),
            user_id.to_string(),
            amount,
            format!("Removed liquidity from pool {}", pool_id),
        );

        Ok(())
    }

    /// Get liquidity in a pool
    pub fn get_pool_liquidity(&self, pool_id: &str) -> f64 {
        *self.liquidity_pools.get(pool_id).unwrap_or(&0.0)
    }

    /// Initialize token distribution to users
    pub fn initialize_distribution(&mut self, allocations: Vec<(String, f64)>) {
        for (user_id, amount) in allocations {
            self.balances.insert(user_id.clone(), amount);
            if amount > 0.0 && !self.holders.contains(&user_id) {
                self.holders.push(user_id);
            }
        }
    }

    /// Transfer tokens from one user to another
    pub fn transfer(
        &mut self,
        from_user_id: &str,
        to_user_id: &str,
        amount: f64,
    ) -> Result<(), TokenError> {
        // Anti-whale check
        if amount > self.max_transfer_limit {
            return Err(TokenError::TransferLimitExceeded(self.max_transfer_limit));
        }

        // Check if sender has enough balance
        let sender_balance = self.balances.get(from_user_id).unwrap_or(&0.0);
        if *sender_balance < amount {
            return Err(TokenError::InsufficientBalance);
        }

        // Calculate burn amount
        let burn_amount = amount * self.burn_rate;
        let transfer_amount = amount - burn_amount;

        // Update balances
        let new_sender_balance = sender_balance - amount;
        self.balances
            .insert(from_user_id.to_string(), new_sender_balance);

        // Add to recipient balance
        let recipient_balance = self.balances.get(to_user_id).unwrap_or(&0.0);
        let new_recipient_balance = recipient_balance + transfer_amount;
        self.balances
            .insert(to_user_id.to_string(), new_recipient_balance);

        // Update total supply (burn)
        self.total_supply -= burn_amount;

        // Add recipient to holders if not already present
        if new_recipient_balance > 0.0 && !self.holders.contains(&to_user_id.to_string()) {
            self.holders.push(to_user_id.to_string());
        }

        // Distribute rewards to holders
        self.distribute_rewards(burn_amount * self.reward_rate);

        // Log transaction
        let transaction = TokenTransaction {
            id: generate_id(),
            from_user_id: from_user_id.to_string(),
            to_user_id: to_user_id.to_string(),
            amount,
            transaction_type: TransactionType::Transfer,
            timestamp: Utc::now().naive_utc(),
        };
        self.transaction_log.push(transaction);

        // Log event
        self.log_event(
            "TRANSFER".to_string(),
            from_user_id.to_string(),
            amount,
            format!("Transferred to {}", to_user_id),
        );

        Ok(())
    }

    /// Distribute rewards to all holders with weighted distribution
    fn distribute_rewards(&mut self, reward_pool: f64) {
        if self.holders.is_empty() || reward_pool <= 0.0 {
            return;
        }

        // Calculate total holdings for weighted distribution
        let total_holdings: f64 = self
            .holders
            .iter()
            .map(|holder_id| *self.balances.get(holder_id).unwrap_or(&0.0))
            .sum();

        if total_holdings <= 0.0 {
            return;
        }

        // Collect reward information first to avoid borrowing issues
        let mut rewards_to_distribute = Vec::new();
        for holder_id in self.holders.clone() {
            let holder_balance = *self.balances.get(&holder_id).unwrap_or(&0.0);
            if holder_balance > 0.0 {
                let reward_amount = reward_pool * (holder_balance / total_holdings);
                rewards_to_distribute.push((holder_id, reward_amount));
            }
        }

        // Distribute rewards
        for (holder_id, reward_amount) in rewards_to_distribute {
            if let Some(balance) = self.balances.get_mut(&holder_id) {
                *balance += reward_amount;
            }

            // Log reward event
            self.log_event(
                "REWARD_DISTRIBUTED".to_string(),
                holder_id,
                reward_amount,
                "Holder reward distribution".to_string(),
            );
        }
    }

    /// Get user balance
    pub fn get_balance(&self, user_id: &str) -> f64 {
        *self.balances.get(user_id).unwrap_or(&0.0)
    }

    /// Get total supply
    pub fn get_total_supply(&self) -> f64 {
        self.total_supply
    }

    /// Freeze user tokens (for staking or other purposes) with event logging
    pub fn freeze_tokens(&mut self, user_id: &str, amount: f64) -> Result<(), TokenError> {
        let balance = self.balances.get(user_id).unwrap_or(&0.0);
        if *balance < amount {
            return Err(TokenError::InsufficientBalance);
        }

        // Update balances
        let new_balance = balance - amount;
        self.balances.insert(user_id.to_string(), new_balance);

        // Update frozen balance
        let frozen_balance = self.frozen_balances.get(user_id).unwrap_or(&0.0);
        let new_frozen_balance = frozen_balance + amount;
        self.frozen_balances
            .insert(user_id.to_string(), new_frozen_balance);

        // Log freeze event
        self.log_event(
            "TOKENS_FROZEN".to_string(),
            user_id.to_string(),
            amount,
            "Tokens frozen for staking or other purposes".to_string(),
        );

        Ok(())
    }

    /// Unfreeze user tokens with event logging
    pub fn unfreeze_tokens(&mut self, user_id: &str, amount: f64) -> Result<(), TokenError> {
        let frozen_balance = self.frozen_balances.get(user_id).unwrap_or(&0.0);
        if *frozen_balance < amount {
            return Err(TokenError::InsufficientFrozenBalance);
        }

        // Update frozen balance
        let new_frozen_balance = frozen_balance - amount;
        self.frozen_balances
            .insert(user_id.to_string(), new_frozen_balance);

        // Update available balance
        let balance = self.balances.get(user_id).unwrap_or(&0.0);
        let new_balance = balance + amount;
        self.balances.insert(user_id.to_string(), new_balance);

        // Log unfreeze event
        self.log_event(
            "TOKENS_UNFROZEN".to_string(),
            user_id.to_string(),
            amount,
            "Tokens unfrozen".to_string(),
        );

        Ok(())
    }

    /// Get frozen balance for a user
    pub fn get_frozen_balance(&self, user_id: &str) -> f64 {
        *self.frozen_balances.get(user_id).unwrap_or(&0.0)
    }

    /// Get transaction log for audit trails
    pub fn get_transaction_log(&self) -> &Vec<TokenTransaction> {
        &self.transaction_log
    }
}
