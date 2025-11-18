use crate::{
    liquidity_pool::{LiquidityPool, LiquidityPoolError},
    stable_liquidity_pool::{StableLiquidityPool, StablePoolError},
};
use chrono::{Duration, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{self, Display};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Chain {
    Ethereum,
    BSC,
    Solana,
}

impl Display for Chain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Chain::Ethereum => write!(f, "Ethereum"),
            Chain::BSC => write!(f, "BSC"),
            Chain::Solana => write!(f, "Solana"),
        }
    }
}

#[derive(Debug)]
pub enum CrossChainLiquidityError {
    PoolAlreadyExists,
    PoolNotFound,
    UnsupportedOperation,
    InvalidAmount,
    InvalidDuration,
    InsufficientLiquidity,
}

impl Display for CrossChainLiquidityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CrossChainLiquidityError::PoolAlreadyExists => write!(f, "Pool already exists"),
            CrossChainLiquidityError::PoolNotFound => write!(f, "Pool not found"),
            CrossChainLiquidityError::UnsupportedOperation => write!(f, "Unsupported operation"),
            CrossChainLiquidityError::InvalidAmount => write!(f, "Amount must be positive"),
            CrossChainLiquidityError::InvalidDuration => write!(f, "Invalid duration"),
            CrossChainLiquidityError::InsufficientLiquidity => write!(f, "Insufficient liquidity"),
        }
    }
}

impl std::error::Error for CrossChainLiquidityError {}

#[derive(Debug)]
pub enum AnyPool {
    Constant(LiquidityPool),
    Stable(StableLiquidityPool),
}

impl AnyPool {
    fn tokens(&self) -> (String, String) {
        match self {
            AnyPool::Constant(p) => (p.config.token_a.clone(), p.config.token_b.clone()),
            AnyPool::Stable(p) => (p.config.token_a.clone(), p.config.token_b.clone()),
        }
    }

    fn add_liquidity(
        &mut self,
        user_id: String,
        token_a_amount: f64,
        token_b_amount: f64,
        duration_days: i64,
    ) -> Result<f64, CrossChainLiquidityError> {
        match self {
            AnyPool::Constant(p) => p
                .add_liquidity(user_id, token_a_amount, token_b_amount, duration_days)
                .map_err(map_lp_err),
            AnyPool::Stable(p) => p
                .add_liquidity(user_id, token_a_amount, token_b_amount, duration_days)
                .map_err(map_sp_err),
        }
    }

    fn reserves(&self) -> (f64, f64) {
        match self {
            AnyPool::Constant(p) => (p.total_token_a, p.total_token_b),
            AnyPool::Stable(p) => (p.total_token_a, p.total_token_b),
        }
    }
}

fn map_lp_err(e: LiquidityPoolError) -> CrossChainLiquidityError {
    match e {
        LiquidityPoolError::InvalidAmount => CrossChainLiquidityError::InvalidAmount,
        LiquidityPoolError::InvalidDuration => CrossChainLiquidityError::InvalidDuration,
        LiquidityPoolError::InsufficientLiquidity => {
            CrossChainLiquidityError::InsufficientLiquidity
        }
        _ => CrossChainLiquidityError::UnsupportedOperation,
    }
}

fn map_sp_err(e: StablePoolError) -> CrossChainLiquidityError {
    match e {
        StablePoolError::InvalidAmount => CrossChainLiquidityError::InvalidAmount,
        StablePoolError::InvalidDuration => CrossChainLiquidityError::InvalidDuration,
        StablePoolError::InsufficientLiquidity => CrossChainLiquidityError::InsufficientLiquidity,
        _ => CrossChainLiquidityError::UnsupportedOperation,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolDeployment {
    pub pool_id: String,
    pub token_a: String,
    pub token_b: String,
    pub fee_tier: f64,
    pub reward_token: String,
    pub total_reward_allocation: f64,
    pub apr_rate: f64,
    pub amplification: Option<f64>,
}

pub struct CrossChainLiquidityManager {
    pools: HashMap<(Chain, String), AnyPool>,
    started_at: NaiveDateTime,
}

impl CrossChainLiquidityManager {
    pub fn new() -> Self {
        Self {
            pools: HashMap::new(),
            started_at: Utc::now().naive_utc(),
        }
    }

    pub fn started_at(&self) -> NaiveDateTime {
        self.started_at
    }

    pub fn deploy_pool(
        &mut self,
        chain: Chain,
        cfg: PoolDeployment,
    ) -> Result<(), CrossChainLiquidityError> {
        let key = (chain, cfg.pool_id.clone());
        if self.pools.contains_key(&key) {
            return Err(CrossChainLiquidityError::PoolAlreadyExists);
        }

        let pool = if let Some(a) = cfg.amplification {
            AnyPool::Stable(
                StableLiquidityPool::new(
                    cfg.pool_id,
                    cfg.token_a,
                    cfg.token_b,
                    cfg.fee_tier,
                    a,
                    cfg.reward_token,
                    cfg.total_reward_allocation,
                    cfg.apr_rate,
                )
                .map_err(map_sp_err)?,
            )
        } else {
            AnyPool::Constant(LiquidityPool::new(
                cfg.pool_id,
                cfg.token_a,
                cfg.token_b,
                cfg.fee_tier,
                cfg.reward_token,
                cfg.total_reward_allocation,
                cfg.apr_rate,
            ))
        };

        self.pools.insert(key, pool);
        Ok(())
    }

    pub fn deploy_constant_product_pool(
        &mut self,
        chain: Chain,
        pool_id: String,
        token_a: String,
        token_b: String,
        fee_tier: f64,
        reward_token: String,
        total_reward_allocation: f64,
        apr_rate: f64,
    ) -> Result<(), CrossChainLiquidityError> {
        self.deploy_pool(
            chain,
            PoolDeployment {
                pool_id,
                token_a,
                token_b,
                fee_tier,
                reward_token,
                total_reward_allocation,
                apr_rate,
                amplification: None,
            },
        )
    }

    pub fn deploy_stable_pool(
        &mut self,
        chain: Chain,
        pool_id: String,
        token_a: String,
        token_b: String,
        fee_tier: f64,
        amplification: f64,
        reward_token: String,
        total_reward_allocation: f64,
        apr_rate: f64,
    ) -> Result<(), CrossChainLiquidityError> {
        self.deploy_pool(
            chain,
            PoolDeployment {
                pool_id,
                token_a,
                token_b,
                fee_tier,
                reward_token,
                total_reward_allocation,
                apr_rate,
                amplification: Some(amplification),
            },
        )
    }

    pub fn add_liquidity(
        &mut self,
        chain: Chain,
        pool_id: &str,
        user_id: String,
        token_a_amount: f64,
        token_b_amount: f64,
        duration_days: i64,
    ) -> Result<f64, CrossChainLiquidityError> {
        let key = (chain, pool_id.to_string());
        let pool = self
            .pools
            .get_mut(&key)
            .ok_or(CrossChainLiquidityError::PoolNotFound)?;
        pool.add_liquidity(user_id, token_a_amount, token_b_amount, duration_days)
    }

    pub fn get_reserves(
        &self,
        chain: Chain,
        pool_id: &str,
    ) -> Result<(f64, f64), CrossChainLiquidityError> {
        let key = (chain, pool_id.to_string());
        let pool = self
            .pools
            .get(&key)
            .ok_or(CrossChainLiquidityError::PoolNotFound)?;
        Ok(pool.reserves())
    }

    pub fn total_liquidity_for_pair(&self, token_a: &str, token_b: &str) -> f64 {
        let mut total = 0.0;
        for (_key, pool) in self.pools.iter() {
            let (a, b) = pool.tokens();
            if (a == token_a && b == token_b) || (a == token_b && b == token_a) {
                let (ra, rb) = pool.reserves();
                total += (ra * rb).sqrt();
            }
        }
        total
    }

    /// Convenience: deploy the same stable pair across multiple chains
    pub fn deploy_stable_pair_across_chains(
        &mut self,
        base_token: &str,
        stable_token: &str,
        chains: &[Chain],
        fee_tier: f64,
        amplification: f64,
        reward_token: &str,
        total_reward_allocation: f64,
        apr_rate: f64,
    ) -> Result<Vec<String>, CrossChainLiquidityError> {
        let mut ids = Vec::new();
        for c in chains {
            let pid = format!(
                "{}_{}_{}",
                base_token,
                stable_token,
                match c {
                    Chain::Ethereum => "eth",
                    Chain::BSC => "bsc",
                    Chain::Solana => "sol",
                }
            );
            self.deploy_stable_pool(
                *c,
                pid.clone(),
                base_token.to_string(),
                stable_token.to_string(),
                fee_tier,
                amplification,
                reward_token.to_string(),
                total_reward_allocation,
                apr_rate,
            )?;
            ids.push(pid);
        }
        Ok(ids)
    }

    /// Distribute initial liquidity to a given pool across chains
    pub fn distribute_initial_liquidity(
        &mut self,
        pool_id: &str,
        allocations: &[(Chain, f64, f64, i64)],
    ) -> Result<(), CrossChainLiquidityError> {
        for (chain, a, b, days) in allocations.iter().copied() {
            let _ =
                self.add_liquidity(chain, pool_id, format!("treasury@{}", chain), a, b, days)?;
        }
        Ok(())
    }
}
