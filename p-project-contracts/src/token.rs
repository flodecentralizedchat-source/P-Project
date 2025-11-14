use chrono::{NaiveDateTime, Utc};
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
    LiquidityLocked, // Added for liquidity locking mechanism
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
            TokenError::LiquidityLocked => write!(f, "Liquidity is locked"),
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

// Liquidity lock structure for LP token locking mechanism
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityLock {
    pub pool_id: String,
    pub amount: f64,
    pub lock_start_date: NaiveDateTime,
    pub lock_duration_months: i64,
    pub is_unlocked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PProjectToken {
    total_supply: f64,
    balances: HashMap<String, f64>,         // user_id -> balance
    frozen_balances: HashMap<String, f64>,  // user_id -> frozen balance
    base_burn_rate: f64,                    // base percentage to burn on each transaction
    reward_rate: f64,                       // percentage to distribute to holders
    holders: Vec<String>,                   // list of user_ids who hold tokens
    max_transfer_limit: f64,                // anti-whale mechanism
    transaction_log: Vec<TokenTransaction>, // audit trail
    event_log: Vec<TokenEvent>,             // event logging
    liquidity_pools: HashMap<String, f64>,  // pool_id -> liquidity amount
    liquidity_locks: HashMap<String, LiquidityLock>, // pool_id -> liquidity lock
    activity_tracker: HashMap<String, i64>, // user_id -> transaction count (for dynamic burn)
    total_transactions: u64,                // total transactions for activity tracking
    // New fields for additional protection mechanisms
    daily_transfer_limits: HashMap<String, (f64, NaiveDateTime)>, // user_id -> (amount, last_reset_date)
    max_daily_transfer_percent: f64,        // max daily transfer as percentage of total supply
    bot_protection_enabled: bool,           // anti-bot protection flag
    bot_cooldown_period: i64,               // cooldown period in seconds
    user_last_transaction: HashMap<String, NaiveDateTime>, // user_id -> last transaction time
    restricted_wallets: HashMap<String, bool>, // user_id -> is_restricted (for team wallets)
}

impl PProjectToken {
    pub fn new(total_supply: f64, burn_rate: f64, reward_rate: f64) -> Self {
        Self {
            total_supply,
            balances: HashMap::new(),
            frozen_balances: HashMap::new(),
            base_burn_rate: burn_rate,
            reward_rate,
            holders: Vec::new(),
            max_transfer_limit: total_supply * 0.05, // 5% of total supply as default limit
            transaction_log: Vec::new(),
            event_log: Vec::new(),
            liquidity_pools: HashMap::new(),
            liquidity_locks: HashMap::new(), // Initialize liquidity locks
            activity_tracker: HashMap::new(),
            total_transactions: 0,
            // Initialize new fields
            daily_transfer_limits: HashMap::new(),
            max_daily_transfer_percent: 0.03, // 3% of total supply as default daily limit
            bot_protection_enabled: true,     // Enable bot protection by default
            bot_cooldown_period: 60,          // 60 seconds cooldown by default
            user_last_transaction: HashMap::new(),
            restricted_wallets: HashMap::new(), // Initialize restricted wallets
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

        // Check if liquidity is locked
        if self.is_liquidity_locked(&pool_id) {
            return Err(TokenError::LiquidityLocked);
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

    /// Lock liquidity for 24 months as per tokenomics
    pub fn lock_liquidity(&mut self, pool_id: String, amount: f64) -> Result<(), TokenError> {
        let liquidity = self.liquidity_pools.get(&pool_id).unwrap_or(&0.0);
        if *liquidity < amount {
            return Err(TokenError::InsufficientBalance);
        }

        let lock = LiquidityLock {
            pool_id: pool_id.clone(),
            amount,
            lock_start_date: Utc::now().naive_utc(),
            lock_duration_months: 24, // 24 months as per tokenomics
            is_unlocked: false,
        };

        self.liquidity_locks.insert(pool_id, lock);
        Ok(())
    }

    /// Check if liquidity is locked
    pub fn is_liquidity_locked(&self, pool_id: &str) -> bool {
        if let Some(lock) = self.liquidity_locks.get(pool_id) {
            if lock.is_unlocked {
                return false;
            }

            let now = Utc::now().naive_utc();
            let elapsed_duration = now - lock.lock_start_date;
            let elapsed_months = elapsed_duration.num_days() / 30; // Approximate months

            elapsed_months < lock.lock_duration_months
        } else {
            false
        }
    }

    /// Unlock liquidity (for emergency purposes)
    pub fn unlock_liquidity(&mut self, pool_id: &str) -> Result<(), TokenError> {
        if let Some(lock) = self.liquidity_locks.get_mut(pool_id) {
            lock.is_unlocked = true;
            
            let lock_amount = lock.amount; // Capture the amount before borrowing self again
            
            // Log event
            self.log_event(
                "LIQUIDITY_UNLOCKED".to_string(),
                "SYSTEM".to_string(),
                lock_amount,
                "Liquidity unlocked".to_string(),
            );
            
            Ok(())
        } else {
            Err(TokenError::InsufficientBalance)
        }
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

    /// Get dynamic burn rate based on network activity
    fn get_dynamic_burn_rate(&self, user_id: &str) -> f64 {
        // Base burn rate
        let mut burn_rate = self.base_burn_rate;
        
        // Increase burn rate during high activity periods
        // Check if this user has been very active (more than 10 transactions)
        if let Some(transaction_count) = self.activity_tracker.get(user_id) {
            if *transaction_count > 10 {
                // Increase burn rate by up to 50% for highly active users
                let activity_multiplier = (*transaction_count as f64 / 100.0).min(0.5);
                burn_rate += self.base_burn_rate * activity_multiplier;
            }
        }
        
        // Global activity factor - increase burn rate when network is very active
        if self.total_transactions > 10000 {
            // Increase burn rate by up to 30% during high network activity
            let network_activity_multiplier = (self.total_transactions as f64 / 100000.0).min(0.3);
            burn_rate += self.base_burn_rate * network_activity_multiplier;
        }
        
        burn_rate
    }

    /// Set maximum daily transfer limit as percentage of total supply
    pub fn set_max_daily_transfer_percent(&mut self, percent: f64) {
        self.max_daily_transfer_percent = percent;
        self.log_event(
            "CONFIG_UPDATE".to_string(),
            "SYSTEM".to_string(),
            0.0,
            format!("Max daily transfer percent set to {}%", percent * 100.0),
        );
    }

    /// Get maximum daily transfer limit
    pub fn get_max_daily_transfer_limit(&self) -> f64 {
        self.total_supply * self.max_daily_transfer_percent
    }

    /// Enable or disable bot protection
    pub fn set_bot_protection(&mut self, enabled: bool) {
        self.bot_protection_enabled = enabled;
        self.log_event(
            "CONFIG_UPDATE".to_string(),
            "SYSTEM".to_string(),
            0.0,
            format!("Bot protection {}", if enabled { "enabled" } else { "disabled" }),
        );
    }

    /// Set bot cooldown period in seconds
    pub fn set_bot_cooldown_period(&mut self, seconds: i64) {
        self.bot_cooldown_period = seconds;
        self.log_event(
            "CONFIG_UPDATE".to_string(),
            "SYSTEM".to_string(),
            0.0,
            format!("Bot cooldown period set to {} seconds", seconds),
        );
    }

    /// Restrict a wallet (e.g., team wallets)
    pub fn restrict_wallet(&mut self, user_id: String, restricted: bool) {
        self.restricted_wallets.insert(user_id.clone(), restricted);
        self.log_event(
            "WALLET_RESTRICTION".to_string(),
            "SYSTEM".to_string(),
            0.0,
            format!("Wallet {} {}", user_id, if restricted { "restricted" } else { "unrestricted" }),
        );
    }

    /// Check if a wallet is restricted
    pub fn is_wallet_restricted(&self, user_id: &str) -> bool {
        *self.restricted_wallets.get(user_id).unwrap_or(&false)
    }

    /// Check daily transfer limit for a user
    fn check_daily_transfer_limit(&mut self, user_id: &str, amount: f64) -> Result<(), TokenError> {
        let now = Utc::now().naive_utc();
        let max_daily_limit = self.get_max_daily_transfer_limit();
        
        // Reset daily limit if it's a new day
        if let Some((daily_amount, last_reset)) = self.daily_transfer_limits.get_mut(user_id) {
            let elapsed_duration = now - *last_reset;
            if elapsed_duration.num_days() >= 1 {
                // Reset daily limit
                *daily_amount = 0.0;
                *last_reset = now;
            }
            
            // Check if adding this amount would exceed daily limit
            if *daily_amount + amount > max_daily_limit {
                return Err(TokenError::TransferLimitExceeded(max_daily_limit));
            }
            
            // Update daily amount
            *daily_amount += amount;
        } else {
            // First transfer of the day
            if amount > max_daily_limit {
                return Err(TokenError::TransferLimitExceeded(max_daily_limit));
            }
            self.daily_transfer_limits.insert(user_id.to_string(), (amount, now));
        }
        
        Ok(())
    }

    /// Check bot protection cooldown
    fn check_bot_protection(&self, user_id: &str) -> Result<(), TokenError> {
        if !self.bot_protection_enabled {
            return Ok(());
        }
        
        if let Some(last_transaction) = self.user_last_transaction.get(user_id) {
            let now = Utc::now().naive_utc();
            let elapsed_duration = now - *last_transaction;
            let elapsed_seconds = elapsed_duration.num_seconds();
            
            if elapsed_seconds < self.bot_cooldown_period {
                return Err(TokenError::TransferLimitExceeded(
                    self.bot_cooldown_period as f64 - elapsed_seconds as f64
                ));
            }
        }
        
        Ok(())
    }

    /// Transfer tokens from one user to another with enhanced protection
    pub fn transfer(
        &mut self,
        from_user_id: &str,
        to_user_id: &str,
        amount: f64,
    ) -> Result<(), TokenError> {
        // Check if sender wallet is restricted
        if self.is_wallet_restricted(from_user_id) {
            return Err(TokenError::TransferLimitExceeded(0.0));
        }

        // Anti-whale check
        if amount > self.max_transfer_limit {
            return Err(TokenError::TransferLimitExceeded(self.max_transfer_limit));
        }

        // Daily transfer limit check
        self.check_daily_transfer_limit(from_user_id, amount)?;

        // Bot protection check
        self.check_bot_protection(from_user_id)?;

        // Check if sender has enough balance
        let sender_balance = self.balances.get(from_user_id).unwrap_or(&0.0);
        if *sender_balance < amount {
            return Err(TokenError::InsufficientBalance);
        }

        // Track activity for dynamic burn rate
        let sender_activity = self.activity_tracker.entry(from_user_id.to_string()).or_insert(0);
        *sender_activity += 1;
        self.total_transactions += 1;

        // Update last transaction time for bot protection
        self.user_last_transaction.insert(from_user_id.to_string(), Utc::now().naive_utc());

        // Calculate dynamic burn amount
        let dynamic_burn_rate = self.get_dynamic_burn_rate(from_user_id);
        let burn_amount = amount * dynamic_burn_rate;
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
            format!("Transferred to {} with dynamic burn rate of {}%", to_user_id, dynamic_burn_rate * 100.0),
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

    /// Burn tokens directly (for buyback programs)
    pub fn burn_tokens(&mut self, amount: f64) -> Result<(), TokenError> {
        if amount <= 0.0 {
            return Err(TokenError::InvalidAmount);
        }

        // Burn from total supply
        self.total_supply -= amount;

        // Log burn event
        self.log_event(
            "TOKENS_BURNED".to_string(),
            "SYSTEM".to_string(),
            amount,
            "Tokens burned through buyback program".to_string(),
        );

        Ok(())
    }

    /// Get activity tracker for a user
    pub fn get_user_activity(&self, user_id: &str) -> i64 {
        *self.activity_tracker.get(user_id).unwrap_or(&0)
    }

    /// Get total transactions
    pub fn get_total_transactions(&self) -> u64 {
        self.total_transactions
    }
}