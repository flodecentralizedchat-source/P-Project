use crate::liquidity_pool::{LiquidityPool, LiquidityPoolError};
use crate::stable_liquidity_pool::{StableLiquidityPool, StablePoolError};
use std::collections::HashMap;

/// Simple manager to orchestrate DEX listing steps: create pools and seed liquidity.
#[derive(Default)]
pub struct DexListingManager {
    pools: HashMap<String, LiquidityPool>,
    stable_pools: HashMap<String, StableLiquidityPool>,
}

impl DexListingManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_constant_pool(
        &mut self,
        pool_id: &str,
        token_a: &str,
        token_b: &str,
        fee_tier: f64,
        reward_token: &str,
        total_reward_allocation: f64,
        apr_rate: f64,
    ) {
        let p = LiquidityPool::new(
            pool_id.to_string(),
            token_a.to_string(),
            token_b.to_string(),
            fee_tier,
            reward_token.to_string(),
            total_reward_allocation,
            apr_rate,
        );
        self.pools.insert(pool_id.to_string(), p);
    }

    pub fn create_stable_pool(
        &mut self,
        pool_id: &str,
        token_a: &str,
        token_b: &str,
        fee_tier: f64,
        amplification: f64,
        reward_token: &str,
        total_reward_allocation: f64,
        apr_rate: f64,
    ) -> Result<(), StablePoolError> {
        let p = StableLiquidityPool::new(
            pool_id.to_string(),
            token_a.to_string(),
            token_b.to_string(),
            fee_tier,
            amplification,
            reward_token.to_string(),
            total_reward_allocation,
            apr_rate,
        )?;
        self.stable_pools.insert(pool_id.to_string(), p);
        Ok(())
    }

    pub fn seed_constant_liquidity(
        &mut self,
        pool_id: &str,
        lp_user: &str,
        token_a_amount: f64,
        token_b_amount: f64,
        duration_days: i64,
    ) -> Result<f64, LiquidityPoolError> {
        let p = self
            .pools
            .get_mut(pool_id)
            .expect("pool must be created before seeding");
        p.add_liquidity(
            lp_user.to_string(),
            token_a_amount,
            token_b_amount,
            duration_days,
        )
    }

    pub fn seed_stable_liquidity(
        &mut self,
        pool_id: &str,
        lp_user: &str,
        token_a_amount: f64,
        token_b_amount: f64,
        duration_days: i64,
    ) -> Result<f64, StablePoolError> {
        let p = self
            .stable_pools
            .get_mut(pool_id)
            .expect("stable pool must be created before seeding");
        p.add_liquidity(
            lp_user.to_string(),
            token_a_amount,
            token_b_amount,
            duration_days,
        )
    }

    pub fn constant_pool(&self, pool_id: &str) -> Option<&LiquidityPool> {
        self.pools.get(pool_id)
    }

    pub fn stable_pool(&self, pool_id: &str) -> Option<&StableLiquidityPool> {
        self.stable_pools.get(pool_id)
    }
}
