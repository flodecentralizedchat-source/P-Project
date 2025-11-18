use crate::cross_chain_liquidity::Chain;
use crate::treasury::LiquidityMiningProgram;
use chrono::{Duration, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityLiquidityProgram {
    pub program_id: String,
    pub reward_token: String,
    pub early_window_days: i64,
    pub early_multiplier: f64,
    pub chain_weights: HashMap<Chain, f64>,
    pub created_at: NaiveDateTime,
    base_program: LiquidityMiningProgram,
    user_chain_liquidity: HashMap<String, HashMap<Chain, f64>>, // user -> (chain -> amount)
}

impl CommunityLiquidityProgram {
    pub fn new(
        program_id: String,
        reward_token: String,
        reward_rate_per_day: f64,
        duration_days: i64,
        total_rewards: f64,
    ) -> Self {
        let created_at = Utc::now().naive_utc();
        Self {
            program_id: program_id.clone(),
            reward_token: reward_token.clone(),
            early_window_days: 0,
            early_multiplier: 1.0,
            chain_weights: HashMap::new(),
            created_at,
            base_program: LiquidityMiningProgram::new(
                program_id,
                reward_token,
                reward_rate_per_day,
                duration_days,
                total_rewards,
            ),
            user_chain_liquidity: HashMap::new(),
        }
    }

    pub fn set_early_window(&mut self, days: i64, multiplier: f64) {
        self.early_window_days = days.max(0);
        self.early_multiplier = if multiplier >= 1.0 { multiplier } else { 1.0 };
    }

    pub fn set_chain_weight(&mut self, chain: Chain, weight: f64) {
        self.chain_weights
            .insert(chain, if weight > 0.0 { weight } else { 1.0 });
    }

    /// Record or update a user's liquidity contribution on a given chain.
    /// Applies early participation multiplier if within the early window.
    pub fn record_contribution(&mut self, user_id: String, chain: Chain, amount: f64) {
        if amount <= 0.0 {
            return;
        }
        let entry = self
            .user_chain_liquidity
            .entry(user_id.clone())
            .or_insert_with(HashMap::new);
        let v = entry.entry(chain).or_insert(0.0);
        *v += amount;

        // Recompute the weighted aggregate for the participant inside the base program
        let weighted = self.get_user_weighted_liquidity(&user_id);
        self.base_program.add_participant(user_id, weighted);
    }

    pub fn get_user_weighted_liquidity(&self, user_id: &str) -> f64 {
        let now = Utc::now().naive_utc();
        let is_early = now <= (self.created_at + Duration::days(self.early_window_days));
        let early = if is_early { self.early_multiplier } else { 1.0 };

        if let Some(per_chain) = self.user_chain_liquidity.get(user_id) {
            per_chain
                .iter()
                .map(|(ch, amt)| {
                    let w = *self.chain_weights.get(ch).unwrap_or(&1.0);
                    amt * w * early
                })
                .sum()
        } else {
            0.0
        }
    }

    /// Calculate estimated rewards after `days_participated`, proportional to weighted liquidity.
    pub fn estimated_rewards(&self, user_id: &str, days_participated: f64) -> f64 {
        self.base_program
            .calculate_rewards(user_id, days_participated)
    }

    /// Return a leaderboard of top-N users by estimated rewards for a given time window.
    pub fn leaderboard(&self, days_participated: f64, top_n: usize) -> Vec<(String, f64)> {
        let mut items: Vec<(String, f64)> = self
            .base_program
            .participants
            .iter()
            .map(|(u, _)| (u.clone(), self.estimated_rewards(u, days_participated)))
            .collect();
        items.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        items.truncate(top_n);
        items
    }
}
