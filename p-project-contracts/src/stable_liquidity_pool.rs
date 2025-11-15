use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum StablePoolError {
    InvalidAmount,
    InsufficientLiquidity,
    UserNotInPool,
    InvalidDuration,
    InvalidAmplification,
}

impl fmt::Display for StablePoolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StablePoolError::InvalidAmount => write!(f, "Amount must be positive"),
            StablePoolError::InsufficientLiquidity => write!(f, "Insufficient liquidity in pool"),
            StablePoolError::UserNotInPool => write!(f, "User not in pool"),
            StablePoolError::InvalidDuration => write!(f, "Invalid duration specified"),
            StablePoolError::InvalidAmplification => write!(f, "Amplification must be >= 1.0"),
        }
    }
}

impl std::error::Error for StablePoolError {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StablePoolConfig {
    pub pool_id: String,
    pub token_a: String, // e.g., P-COIN
    pub token_b: String, // e.g., USDC
    pub fee_tier: f64,   // e.g., 0.0005 for 5 bps
    pub start_date: NaiveDateTime,
    pub amplification: f64, // amplification factor (A >= 1) to reduce slippage near peg
    pub reward_token: String,
    pub total_reward_allocation: f64,
    pub distributed_rewards: f64,
    pub apr_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StableLiquidityPosition {
    pub user_id: String,
    pub pool_id: String,
    pub liquidity_amount: f64,
    pub token_a_amount: f64,
    pub token_b_amount: f64,
    pub start_time: NaiveDateTime,
    pub duration_days: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StableLiquidityPool {
    pub config: StablePoolConfig,
    pub total_liquidity: f64,
    pub total_token_a: f64,
    pub total_token_b: f64,
    pub positions: HashMap<String, StableLiquidityPosition>,
    pub k_constant: f64,
    pub total_volume: f64,
    pub total_fees: f64,
}

impl StableLiquidityPool {
    pub fn new(
        pool_id: String,
        token_a: String,
        token_b: String,
        fee_tier: f64,
        amplification: f64,
        reward_token: String,
        total_reward_allocation: f64,
        apr_rate: f64,
    ) -> Result<Self, StablePoolError> {
        if amplification < 1.0 {
            return Err(StablePoolError::InvalidAmplification);
        }
        let now = Utc::now().naive_utc();
        let config = StablePoolConfig {
            pool_id,
            token_a,
            token_b,
            fee_tier,
            start_date: now,
            amplification,
            reward_token,
            total_reward_allocation,
            distributed_rewards: 0.0,
            apr_rate,
        };

        Ok(Self {
            config,
            total_liquidity: 0.0,
            total_token_a: 0.0,
            total_token_b: 0.0,
            positions: HashMap::new(),
            k_constant: 0.0,
            total_volume: 0.0,
            total_fees: 0.0,
        })
    }

    pub fn add_liquidity(
        &mut self,
        user_id: String,
        token_a_amount: f64,
        token_b_amount: f64,
        duration_days: i64,
    ) -> Result<f64, StablePoolError> {
        if token_a_amount <= 0.0 || token_b_amount <= 0.0 {
            return Err(StablePoolError::InvalidAmount);
        }
        if duration_days <= 0 {
            return Err(StablePoolError::InvalidDuration);
        }

        let liquidity_amount = (token_a_amount * token_b_amount).sqrt();

        if self.total_liquidity == 0.0 {
            self.k_constant = token_a_amount * token_b_amount;
        }

        self.total_token_a += token_a_amount;
        self.total_token_b += token_b_amount;
        self.total_liquidity += liquidity_amount;

        let start_time = Utc::now().naive_utc();
        let position = if let Some(existing) = self.positions.get_mut(&user_id) {
            existing.liquidity_amount += liquidity_amount;
            existing.token_a_amount += token_a_amount;
            existing.token_b_amount += token_b_amount;
            existing.clone()
        } else {
            StableLiquidityPosition {
                user_id: user_id.clone(),
                pool_id: self.config.pool_id.clone(),
                liquidity_amount,
                token_a_amount,
                token_b_amount,
                start_time,
                duration_days,
            }
        };
        self.positions.insert(user_id, position);
        Ok(liquidity_amount)
    }

    pub fn remove_liquidity(&mut self, user_id: &str) -> Result<(f64, f64), StablePoolError> {
        let position = self
            .positions
            .get(user_id)
            .ok_or(StablePoolError::UserNotInPool)?
            .clone();

        let liquidity_ratio = position.liquidity_amount / self.total_liquidity;
        let token_a_return = self.total_token_a * liquidity_ratio;
        let token_b_return = self.total_token_b * liquidity_ratio;

        self.total_token_a -= token_a_return;
        self.total_token_b -= token_b_return;
        self.total_liquidity -= position.liquidity_amount;

        if self.total_liquidity > 0.0 {
            self.k_constant = self.total_token_a * self.total_token_b;
        } else {
            self.k_constant = 0.0;
        }

        self.positions.remove(user_id);
        Ok((token_a_return, token_b_return))
    }

    pub fn get_reserves(&self) -> (f64, f64) {
        (self.total_token_a, self.total_token_b)
    }

    pub fn get_config(&self) -> &StablePoolConfig {
        &self.config
    }

    /// Stable swap approximation: output = dx * y / (A*x + dx), with fee applied to dx
    pub fn calculate_swap_output(
        &self,
        input_token: &str,
        input_amount: f64,
    ) -> Result<f64, StablePoolError> {
        if input_amount <= 0.0 {
            return Err(StablePoolError::InvalidAmount);
        }

        let (input_reserve, output_reserve) = if input_token == self.config.token_a {
            (self.total_token_a, self.total_token_b)
        } else if input_token == self.config.token_b {
            (self.total_token_b, self.total_token_a)
        } else {
            return Err(StablePoolError::InvalidAmount);
        };

        if input_reserve == 0.0 || output_reserve == 0.0 {
            return Err(StablePoolError::InsufficientLiquidity);
        }

        let dx = input_amount * (1.0 - self.config.fee_tier);
        let a = self.config.amplification;
        let out = (dx * output_reserve) / (a * input_reserve + dx);
        if out <= 0.0 || out > output_reserve {
            return Err(StablePoolError::InsufficientLiquidity);
        }
        Ok(out)
    }

    pub fn swap(&mut self, input_token: &str, input_amount: f64) -> Result<f64, StablePoolError> {
        let output_amount = self.calculate_swap_output(input_token, input_amount)?;

        if input_token == self.config.token_a {
            self.total_token_a += input_amount;
            self.total_token_b -= output_amount;
        } else if input_token == self.config.token_b {
            self.total_token_b += input_amount;
            self.total_token_a -= output_amount;
        } else {
            return Err(StablePoolError::InvalidAmount);
        }

        self.k_constant = self.total_token_a * self.total_token_b;
        self.total_volume += input_amount;
        self.total_fees += input_amount * self.config.fee_tier;
        Ok(output_amount)
    }
}
