use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

// Custom error types for liquidity pool operations
#[derive(Debug, Clone, PartialEq)]
pub enum LiquidityPoolError {
    InvalidAmount,
    InsufficientLiquidity,
    PoolNotFound,
    UserNotInPool,
    PoolAlreadyExists,
    InvalidDuration,
    SerializationError(String),
    InsufficientRewards,
}

impl fmt::Display for LiquidityPoolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LiquidityPoolError::InvalidAmount => write!(f, "Amount must be positive"),
            LiquidityPoolError::InsufficientLiquidity => write!(f, "Insufficient liquidity in pool"),
            LiquidityPoolError::PoolNotFound => write!(f, "Pool not found"),
            LiquidityPoolError::UserNotInPool => write!(f, "User not in pool"),
            LiquidityPoolError::PoolAlreadyExists => write!(f, "Pool already exists"),
            LiquidityPoolError::InvalidDuration => write!(f, "Invalid duration specified"),
            LiquidityPoolError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            LiquidityPoolError::InsufficientRewards => write!(f, "Insufficient rewards available"),
        }
    }
}

impl std::error::Error for LiquidityPoolError {}

// Liquidity pool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityPoolConfig {
    pub pool_id: String,
    pub token_a: String, // First token in the pair
    pub token_b: String, // Second token in the pair
    pub fee_tier: f64,   // Fee tier as percentage (e.g., 0.003 for 0.3%)
    pub start_date: NaiveDateTime,
    pub reward_token: String, // Token used for rewards
    pub total_reward_allocation: f64, // Total rewards allocated to this pool
    pub distributed_rewards: f64, // Rewards already distributed
    pub apr_rate: f64, // Annual percentage rate for yield farming
}

// Liquidity provider position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityPosition {
    pub user_id: String,
    pub pool_id: String,
    pub liquidity_amount: f64,
    pub token_a_amount: f64,
    pub token_b_amount: f64,
    pub start_time: NaiveDateTime,
    pub duration_days: i64,
    pub accumulated_rewards: f64,
    pub last_reward_time: NaiveDateTime,
    pub claimed_rewards: f64, // Rewards already claimed by the user
}

// Liquidity pool struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityPool {
    pub config: LiquidityPoolConfig,
    pub total_liquidity: f64,
    pub total_token_a: f64,
    pub total_token_b: f64,
    pub liquidity_positions: HashMap<String, LiquidityPosition>, // user_id -> position
    pub k_constant: f64, // Constant product formula: x * y = k
    pub total_volume: f64, // Total volume traded through the pool
    pub total_fees: f64, // Total fees collected
}

impl LiquidityPool {
    /// Create a new liquidity pool
    pub fn new(
        pool_id: String,
        token_a: String,
        token_b: String,
        fee_tier: f64,
        reward_token: String,
        total_reward_allocation: f64,
        apr_rate: f64,
    ) -> Self {
        let now = Utc::now().naive_utc();
        let config = LiquidityPoolConfig {
            pool_id,
            token_a,
            token_b,
            fee_tier,
            start_date: now,
            reward_token,
            total_reward_allocation,
            distributed_rewards: 0.0,
            apr_rate,
        };

        Self {
            config,
            total_liquidity: 0.0,
            total_token_a: 0.0,
            total_token_b: 0.0,
            liquidity_positions: HashMap::new(),
            k_constant: 0.0,
            total_volume: 0.0,
            total_fees: 0.0,
        }
    }

    /// Add liquidity to the pool
    pub fn add_liquidity(
        &mut self,
        user_id: String,
        token_a_amount: f64,
        token_b_amount: f64,
        duration_days: i64,
    ) -> Result<f64, LiquidityPoolError> {
        if token_a_amount <= 0.0 || token_b_amount <= 0.0 {
            return Err(LiquidityPoolError::InvalidAmount);
        }

        if duration_days <= 0 {
            return Err(LiquidityPoolError::InvalidDuration);
        }

        let liquidity_amount = (token_a_amount * token_b_amount).sqrt();

        // If this is the first liquidity added, initialize the k constant
        if self.total_liquidity == 0.0 {
            self.k_constant = token_a_amount * token_b_amount;
        }

        // Update pool totals
        self.total_token_a += token_a_amount;
        self.total_token_b += token_b_amount;
        self.total_liquidity += liquidity_amount;

        // Create or update liquidity position
        let start_time = Utc::now().naive_utc();
        
        // Check if user already has a position
        let position = if let Some(existing_position) = self.liquidity_positions.get_mut(&user_id) {
            // Update existing position
            existing_position.liquidity_amount += liquidity_amount;
            existing_position.token_a_amount += token_a_amount;
            existing_position.token_b_amount += token_b_amount;
            existing_position.clone()
        } else {
            // Create new position
            LiquidityPosition {
                user_id: user_id.clone(),
                pool_id: self.config.pool_id.clone(),
                liquidity_amount,
                token_a_amount,
                token_b_amount,
                start_time,
                duration_days,
                accumulated_rewards: 0.0,
                last_reward_time: start_time,
                claimed_rewards: 0.0,
            }
        };

        self.liquidity_positions.insert(user_id, position);

        Ok(liquidity_amount)
    }

    /// Remove liquidity from the pool
    pub fn remove_liquidity(&mut self, user_id: &str) -> Result<(f64, f64), LiquidityPoolError> {
        // Returns (token_a_amount, token_b_amount)
        let position = self
            .liquidity_positions
            .get(user_id)
            .ok_or(LiquidityPoolError::UserNotInPool)?
            .clone();

        // Calculate proportional amounts to return
        let liquidity_ratio = position.liquidity_amount / self.total_liquidity;
        let token_a_return = self.total_token_a * liquidity_ratio;
        let token_b_return = self.total_token_b * liquidity_ratio;

        // Update pool totals
        self.total_token_a -= token_a_return;
        self.total_token_b -= token_b_return;
        self.total_liquidity -= position.liquidity_amount;

        // Update k constant
        if self.total_liquidity > 0.0 {
            self.k_constant = self.total_token_a * self.total_token_b;
        } else {
            self.k_constant = 0.0;
        }

        // Remove position
        self.liquidity_positions.remove(user_id);

        Ok((token_a_return, token_b_return))
    }

    /// Get liquidity position for a user
    pub fn get_position(&self, user_id: &str) -> Option<&LiquidityPosition> {
        self.liquidity_positions.get(user_id)
    }

    /// Get all liquidity positions
    pub fn get_all_positions(&self) -> &HashMap<String, LiquidityPosition> {
        &self.liquidity_positions
    }

    /// Calculate swap output amount
    pub fn calculate_swap_output(&self, input_token: &str, input_amount: f64) -> Result<f64, LiquidityPoolError> {
        if input_amount <= 0.0 {
            return Err(LiquidityPoolError::InvalidAmount);
        }

        let (input_reserve, output_reserve) = if input_token == self.config.token_a {
            (self.total_token_a, self.total_token_b)
        } else if input_token == self.config.token_b {
            (self.total_token_b, self.total_token_a)
        } else {
            return Err(LiquidityPoolError::InvalidAmount);
        };

        if input_reserve == 0.0 || output_reserve == 0.0 {
            return Err(LiquidityPoolError::InsufficientLiquidity);
        }

        // Apply fee
        let input_amount_with_fee = input_amount * (1.0 - self.config.fee_tier);
        
        // Constant product formula: (x + dx) * (y - dy) = x * y
        // Solving for dy: dy = y - (x * y) / (x + dx)
        let output_amount = output_reserve - (self.k_constant / (input_reserve + input_amount_with_fee));

        if output_amount <= 0.0 || output_amount > output_reserve {
            return Err(LiquidityPoolError::InsufficientLiquidity);
        }

        Ok(output_amount)
    }

    /// Execute a swap
    pub fn swap(&mut self, input_token: &str, input_amount: f64) -> Result<f64, LiquidityPoolError> {
        let output_amount = self.calculate_swap_output(input_token, input_amount)?;

        // Update reserves
        if input_token == self.config.token_a {
            self.total_token_a += input_amount;
            self.total_token_b -= output_amount;
        } else if input_token == self.config.token_b {
            self.total_token_b += input_amount;
            self.total_token_a -= output_amount;
        } else {
            return Err(LiquidityPoolError::InvalidAmount);
        }

        // Update k constant
        self.k_constant = self.total_token_a * self.total_token_b;

        // Update volume and fees tracking
        self.total_volume += input_amount;
        self.total_fees += input_amount * self.config.fee_tier;

        Ok(output_amount)
    }

    /// Get pool reserves
    pub fn get_reserves(&self) -> (f64, f64) {
        (self.total_token_a, self.total_token_b)
    }

    /// Get pool configuration
    pub fn get_config(&self) -> &LiquidityPoolConfig {
        &self.config
    }

    /// Calculate yield for a liquidity position based on time and APR
    pub fn calculate_yield(&self, position: &LiquidityPosition) -> f64 {
        let now = Utc::now().naive_utc();
        let duration = now - position.last_reward_time;
        let days = duration.num_seconds() as f64 / 86400.0; // Convert to days
        
        // Calculate yield using compound interest formula
        // A = P(1 + r/n)^(nt) where n = 365 (daily compounding)
        let principal = position.liquidity_amount;
        let rate = self.config.apr_rate;
        let n = 365.0; // Daily compounding
        let t = days / 365.0; // Time in years
        
        let yield_amount = principal * ((1.0 + rate / n).powf(n * t) - 1.0);
        
        yield_amount
    }

    /// Calculate projected yield for a given liquidity amount and duration
    pub fn calculate_projected_yield(&self, liquidity_amount: f64, days: f64) -> f64 {
        // Calculate yield using compound interest formula
        let principal = liquidity_amount;
        let rate = self.config.apr_rate;
        let n = 365.0; // Daily compounding
        let t = days / 365.0; // Time in years
        
        let yield_amount = principal * ((1.0 + rate / n).powf(n * t) - 1.0);
        
        yield_amount
    }

    /// Update rewards for a specific user
    pub fn update_rewards(&mut self, user_id: &str) -> Result<f64, LiquidityPoolError> {
        // Clone the position to avoid borrowing issues
        let position = self
            .liquidity_positions
            .get(user_id)
            .ok_or(LiquidityPoolError::UserNotInPool)?
            .clone();

        let yield_amount = self.calculate_yield(&position);
        
        // Update the position in the map
        if let Some(pos) = self.liquidity_positions.get_mut(user_id) {
            pos.accumulated_rewards += yield_amount;
            pos.last_reward_time = Utc::now().naive_utc();
        }

        Ok(yield_amount)
    }

    /// Get total accumulated rewards for a user
    pub fn get_user_rewards(&self, user_id: &str) -> Result<f64, LiquidityPoolError> {
        let position = self
            .liquidity_positions
            .get(user_id)
            .ok_or(LiquidityPoolError::UserNotInPool)?;
        
        Ok(position.accumulated_rewards)
    }

    /// Get claimable rewards for a user
    pub fn get_claimable_rewards(&self, user_id: &str) -> Result<f64, LiquidityPoolError> {
        let position = self
            .liquidity_positions
            .get(user_id)
            .ok_or(LiquidityPoolError::UserNotInPool)?;

        let new_rewards = self.calculate_yield(position);
        let total_accumulated = position.accumulated_rewards + new_rewards;
        let claimable = total_accumulated - position.claimed_rewards;

        Ok(claimable)
    }

    /// Claim rewards for a user
    pub fn claim_rewards(&mut self, user_id: &str) -> Result<f64, LiquidityPoolError> {
        // Clone the position to avoid borrowing issues
        let position = self
            .liquidity_positions
            .get(user_id)
            .ok_or(LiquidityPoolError::UserNotInPool)?
            .clone();

        // Update rewards to get the latest amount
        let new_rewards = self.calculate_yield(&position);
        let total_accumulated = position.accumulated_rewards + new_rewards;
        let claimable = total_accumulated - position.claimed_rewards;

        // Check if there are enough rewards in the pool allocation
        if self.config.distributed_rewards + claimable > self.config.total_reward_allocation {
            return Err(LiquidityPoolError::InsufficientRewards);
        }

        // Claim the rewards
        if let Some(pos) = self.liquidity_positions.get_mut(user_id) {
            pos.accumulated_rewards = total_accumulated;
            pos.claimed_rewards = total_accumulated;
            pos.last_reward_time = Utc::now().naive_utc();
        }
        
        self.config.distributed_rewards += claimable;

        Ok(claimable)
    }

    /// Distribute fees proportionally to liquidity providers
    pub fn distribute_fees(&self) -> Result<HashMap<String, f64>, LiquidityPoolError> {
        let mut fee_distribution = HashMap::new();
        
        // Distribute fees proportionally based on liquidity contribution
        for (user_id, position) in &self.liquidity_positions {
            let share = position.liquidity_amount / self.total_liquidity;
            let fee_share = self.total_fees * share;
            fee_distribution.insert(user_id.clone(), fee_share);
        }
        
        Ok(fee_distribution)
    }

    /// Get pool statistics
    pub fn get_pool_stats(&self) -> PoolStats {
        let total_providers = self.liquidity_positions.len() as f64;
        let avg_liquidity = if total_providers > 0.0 {
            self.total_liquidity / total_providers
        } else {
            0.0
        };

        PoolStats {
            total_liquidity: self.total_liquidity,
            total_volume: self.total_volume,
            total_fees: self.total_fees,
            total_providers: total_providers as usize,
            avg_liquidity,
            apr_rate: self.config.apr_rate,
        }
    }
}

// Pool statistics structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolStats {
    pub total_liquidity: f64,
    pub total_volume: f64,
    pub total_fees: f64,
    pub total_providers: usize,
    pub avg_liquidity: f64,
    pub apr_rate: f64,
}