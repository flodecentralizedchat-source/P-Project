use crate::liquidity_pool::{LiquidityPool, LiquidityPoolError};
use crate::stable_liquidity_pool::{StableLiquidityPool, StablePoolError};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum RouterError {
    NoRoute,
    InvalidAmount,
    InsufficientLiquidity,
    SlippageExceeded,
    PoolError(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum PoolKind {
    Constant,
    Stable,
}

#[derive(Debug, Clone)]
pub struct RouterQuote {
    pub pool_id: String,
    pub kind: PoolKind,
    pub input_token: String,
    pub output_token: String,
    pub input_amount: f64,
    pub output_amount: f64,
}

#[derive(Debug, Clone)]
pub struct SwapResult {
    pub pool_id: String,
    pub kind: PoolKind,
    pub input_token: String,
    pub output_token: String,
    pub input_amount: f64,
    pub output_amount: f64,
}

#[derive(Debug, Clone)]
pub struct AutoProvisionConfig {
    pub lp_user: String,
    pub min_reserve_per_side: f64,
    pub duration_days: i64,
}

#[derive(Default)]
pub struct Router {
    constant_pools: HashMap<String, LiquidityPool>,
    stable_pools: HashMap<String, StableLiquidityPool>,
}

impl Router {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_constant_pool(&mut self, id: String, pool: LiquidityPool) {
        self.constant_pools.insert(id, pool);
    }

    pub fn register_stable_pool(&mut self, id: String, pool: StableLiquidityPool) {
        self.stable_pools.insert(id, pool);
    }

    pub fn get_constant_pool(&self, id: &str) -> Option<&LiquidityPool> {
        self.constant_pools.get(id)
    }

    pub fn get_stable_pool(&self, id: &str) -> Option<&StableLiquidityPool> {
        self.stable_pools.get(id)
    }

    fn matching_constant_pool_ids(&self, a: &str, b: &str) -> Vec<String> {
        self.constant_pools
            .iter()
            .filter(|(_, p)| {
                (p.config.token_a == a && p.config.token_b == b)
                    || (p.config.token_a == b && p.config.token_b == a)
            })
            .map(|(id, _)| id.clone())
            .collect()
    }

    fn matching_stable_pool_ids(&self, a: &str, b: &str) -> Vec<String> {
        self.stable_pools
            .iter()
            .filter(|(_, p)| {
                (p.get_config().token_a == a && p.get_config().token_b == b)
                    || (p.get_config().token_a == b && p.get_config().token_b == a)
            })
            .map(|(id, _)| id.clone())
            .collect()
    }

    pub fn best_quote(
        &self,
        input_token: &str,
        output_token: &str,
        input_amount: f64,
    ) -> Result<RouterQuote, RouterError> {
        if input_amount <= 0.0 {
            return Err(RouterError::InvalidAmount);
        }

        let mut best: Option<RouterQuote> = None;

        for (id, p) in &self.constant_pools {
            let ta = &p.config.token_a;
            let tb = &p.config.token_b;
            let out = if input_token == ta && output_token == tb {
                p.calculate_swap_output(input_token, input_amount)
            } else if input_token == tb && output_token == ta {
                p.calculate_swap_output(input_token, input_amount)
            } else {
                continue;
            };
            if let Ok(o) = out {
                let q = RouterQuote {
                    pool_id: id.clone(),
                    kind: PoolKind::Constant,
                    input_token: input_token.to_string(),
                    output_token: output_token.to_string(),
                    input_amount,
                    output_amount: o,
                };
                if best.as_ref().map(|b| o > b.output_amount).unwrap_or(true) {
                    best = Some(q);
                }
            }
        }

        for (id, p) in &self.stable_pools {
            let ta = &p.get_config().token_a;
            let tb = &p.get_config().token_b;
            let out = if input_token == ta && output_token == tb {
                p.calculate_swap_output(input_token, input_amount)
            } else if input_token == tb && output_token == ta {
                p.calculate_swap_output(input_token, input_amount)
            } else {
                continue;
            };
            if let Ok(o) = out {
                let q = RouterQuote {
                    pool_id: id.clone(),
                    kind: PoolKind::Stable,
                    input_token: input_token.to_string(),
                    output_token: output_token.to_string(),
                    input_amount,
                    output_amount: o,
                };
                if best.as_ref().map(|b| o > b.output_amount).unwrap_or(true) {
                    best = Some(q);
                }
            }
        }

        best.ok_or(RouterError::NoRoute)
    }

    fn ensure_liquidity(
        &mut self,
        pool_id: &str,
        kind: &PoolKind,
        input_token: &str,
        output_token: &str,
        cfg: &AutoProvisionConfig,
    ) -> Result<bool, RouterError> {
        match kind {
            PoolKind::Constant => {
                let p = self
                    .constant_pools
                    .get_mut(pool_id)
                    .ok_or(RouterError::NoRoute)?;
                let (mut ra, mut rb) = p.get_reserves();
                let add_a = if ra < cfg.min_reserve_per_side {
                    cfg.min_reserve_per_side - ra
                } else {
                    0.0
                };
                let add_b = if rb < cfg.min_reserve_per_side {
                    cfg.min_reserve_per_side - rb
                } else {
                    0.0
                };
                if add_a > 0.0 || add_b > 0.0 {
                    // Map amounts to token order
                    let (ta, tb) = (&p.config.token_a, &p.config.token_b);
                    let (mut tok_a_amt, mut tok_b_amt) = (add_a, add_b);
                    if !(ta == input_token && tb == output_token) && !(ta == output_token && tb == input_token) {
                        // Keep provision symmetric if pair order differs
                        tok_a_amt = add_a;
                        tok_b_amt = add_b;
                    }
                    p.add_liquidity(
                        cfg.lp_user.clone(),
                        tok_a_amt,
                        tok_b_amt,
                        cfg.duration_days,
                    )
                    .map_err(|e| RouterError::PoolError(format!("{:?}", e)))?;
                    let rs = p.get_reserves();
                    ra = rs.0;
                    rb = rs.1;
                    return Ok(true);
                }
                Ok(false)
            }
            PoolKind::Stable => {
                let p = self
                    .stable_pools
                    .get_mut(pool_id)
                    .ok_or(RouterError::NoRoute)?;
                let (ra, rb) = p.get_reserves();
                let add_a = if ra < cfg.min_reserve_per_side {
                    cfg.min_reserve_per_side - ra
                } else {
                    0.0
                };
                let add_b = if rb < cfg.min_reserve_per_side {
                    cfg.min_reserve_per_side - rb
                } else {
                    0.0
                };
                if add_a > 0.0 || add_b > 0.0 {
                    p.add_liquidity(
                        cfg.lp_user.clone(),
                        add_a,
                        add_b,
                        cfg.duration_days,
                    )
                    .map_err(|e| RouterError::PoolError(format!("{:?}", e)))?;
                    return Ok(true);
                }
                Ok(false)
            }
        }
    }

    fn pick_preferred_matching_pool(
        &self,
        input_token: &str,
        output_token: &str,
    ) -> Option<(PoolKind, String)> {
        let st = self.matching_stable_pool_ids(input_token, output_token);
        if let Some(id) = st.first() {
            return Some((PoolKind::Stable, id.clone()));
        }
        let ct = self.matching_constant_pool_ids(input_token, output_token);
        ct.first().map(|id| (PoolKind::Constant, id.clone()))
    }

    pub fn swap_best_route(
        &mut self,
        input_token: &str,
        output_token: &str,
        input_amount: f64,
        min_output: f64,
        auto: Option<AutoProvisionConfig>,
    ) -> Result<SwapResult, RouterError> {
        if input_amount <= 0.0 {
            return Err(RouterError::InvalidAmount);
        }

        let mut quote = self.best_quote(input_token, output_token, input_amount);

        if quote.is_err() {
            if let Some(cfg) = &auto {
                if let Some((kind, id)) = self.pick_preferred_matching_pool(input_token, output_token)
                {
                    let _ = self.ensure_liquidity(&id, &kind, input_token, output_token, cfg)?;
                    quote = self.best_quote(input_token, output_token, input_amount);
                }
            }
        }

        let q = quote?;

        match q.kind {
            PoolKind::Constant => {
                let p = self
                    .constant_pools
                    .get_mut(&q.pool_id)
                    .ok_or(RouterError::NoRoute)?;

                let expected = p
                    .calculate_swap_output(input_token, input_amount)
                    .map_err(|e| match e {
                        LiquidityPoolError::InsufficientLiquidity => RouterError::InsufficientLiquidity,
                        LiquidityPoolError::InvalidAmount => RouterError::InvalidAmount,
                        _ => RouterError::PoolError(format!("{:?}", e)),
                    })?;

                if expected < min_output {
                    if let Some(cfg) = &auto {
                        let changed = self.ensure_liquidity(&q.pool_id, &q.kind, input_token, output_token, cfg)?;
                        if changed {
                            let re = p
                                .calculate_swap_output(input_token, input_amount)
                                .map_err(|e| match e {
                                    LiquidityPoolError::InsufficientLiquidity => RouterError::InsufficientLiquidity,
                                    LiquidityPoolError::InvalidAmount => RouterError::InvalidAmount,
                                    _ => RouterError::PoolError(format!("{:?}", e)),
                                })?;
                            if re < min_output {
                                return Err(RouterError::SlippageExceeded);
                            }
                        } else {
                            return Err(RouterError::SlippageExceeded);
                        }
                    } else {
                        return Err(RouterError::SlippageExceeded);
                    }
                }

                let out = p.swap(input_token, input_amount).map_err(|e| match e {
                    LiquidityPoolError::InsufficientLiquidity => RouterError::InsufficientLiquidity,
                    LiquidityPoolError::InvalidAmount => RouterError::InvalidAmount,
                    _ => RouterError::PoolError(format!("{:?}", e)),
                })?;

                Ok(SwapResult {
                    pool_id: q.pool_id,
                    kind: q.kind,
                    input_token: input_token.to_string(),
                    output_token: output_token.to_string(),
                    input_amount,
                    output_amount: out,
                })
            }
            PoolKind::Stable => {
                let p = self
                    .stable_pools
                    .get_mut(&q.pool_id)
                    .ok_or(RouterError::NoRoute)?;

                let expected = p
                    .calculate_swap_output(input_token, input_amount)
                    .map_err(|e| match e {
                        StablePoolError::InsufficientLiquidity => RouterError::InsufficientLiquidity,
                        StablePoolError::InvalidAmount => RouterError::InvalidAmount,
                        _ => RouterError::PoolError(format!("{:?}", e)),
                    })?;

                if expected < min_output {
                    if let Some(cfg) = &auto {
                        let changed = self.ensure_liquidity(&q.pool_id, &q.kind, input_token, output_token, cfg)?;
                        if changed {
                            let re = p
                                .calculate_swap_output(input_token, input_amount)
                                .map_err(|e| match e {
                                    StablePoolError::InsufficientLiquidity => RouterError::InsufficientLiquidity,
                                    StablePoolError::InvalidAmount => RouterError::InvalidAmount,
                                    _ => RouterError::PoolError(format!("{:?}", e)),
                                })?;
                            if re < min_output {
                                return Err(RouterError::SlippageExceeded);
                            }
                        } else {
                            return Err(RouterError::SlippageExceeded);
                        }
                    } else {
                        return Err(RouterError::SlippageExceeded);
                    }
                }

                let out = p.swap(input_token, input_amount).map_err(|e| match e {
                    StablePoolError::InsufficientLiquidity => RouterError::InsufficientLiquidity,
                    StablePoolError::InvalidAmount => RouterError::InvalidAmount,
                    _ => RouterError::PoolError(format!("{:?}", e)),
                })?;

                Ok(SwapResult {
                    pool_id: q.pool_id,
                    kind: q.kind,
                    input_token: input_token.to_string(),
                    output_token: output_token.to_string(),
                    input_amount,
                    output_amount: out,
                })
            }
        }
    }
}
