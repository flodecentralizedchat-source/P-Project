use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

/// Configuration for the in-game currency ecosystem.
#[derive(Debug, Clone)]
pub struct GameCurrencyConfig {
    pub base_mission_reward: f64,
    pub behavior_rewards: HashMap<PositiveBehavior, f64>,
}

/// Representation of a peaceful mission playable in a game.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeacefulMission {
    pub mission_id: String,
    pub name: String,
    pub description: String,
    pub reward_multiplier: f64,
    pub tags: Vec<String>,
    pub is_nonviolent: bool,
    pub created_at: NaiveDateTime,
}

/// Enum capturing the kinds of positive behavior we reward.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PositiveBehavior {
    HelpingHands,
    EnvironmentalCare,
    ConflictResolution,
    EducationChampion,
}

impl PositiveBehavior {
    pub fn description(&self) -> &'static str {
        match self {
            PositiveBehavior::HelpingHands => "Helping others in the community",
            PositiveBehavior::EnvironmentalCare => "Protecting the environment",
            PositiveBehavior::ConflictResolution => "Mediating peacefully",
            PositiveBehavior::EducationChampion => "Promoting learning",
        }
    }
}

/// Receipt returned after awarding tokens.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardReceipt {
    pub player_id: String,
    pub tokens_awarded: f64,
    pub new_balance: f64,
    pub reason: String,
    pub timestamp: NaiveDateTime,
}

/// Tracks completions and balances for players.
pub struct GameCurrencyService {
    config: GameCurrencyConfig,
    missions: HashMap<String, PeacefulMission>,
    player_balances: HashMap<String, f64>,
    completions: HashMap<String, HashSet<String>>,
}

impl GameCurrencyService {
    pub fn new(config: GameCurrencyConfig) -> Self {
        Self {
            config,
            missions: HashMap::new(),
            player_balances: HashMap::new(),
            completions: HashMap::new(),
        }
    }

    pub fn register_mission(
        &mut self,
        mut mission: PeacefulMission,
    ) -> Result<PeacefulMission, Box<dyn std::error::Error>> {
        if !mission.is_nonviolent {
            return Err("Mission must be non-violent".into());
        }
        if mission.reward_multiplier <= 0.0 {
            return Err("Reward multiplier must be positive".into());
        }
        if mission.mission_id.is_empty() {
            mission.mission_id = format!("mission_{}", Uuid::new_v4());
        }
        mission.created_at = Utc::now().naive_utc();
        self.missions
            .insert(mission.mission_id.clone(), mission.clone());
        Ok(mission)
    }

    pub fn get_mission(&self, mission_id: &str) -> Option<&PeacefulMission> {
        self.missions.get(mission_id)
    }

    pub fn complete_mission(
        &mut self,
        player_id: &str,
        mission_id: &str,
    ) -> Result<RewardReceipt, Box<dyn std::error::Error>> {
        let mission = self.missions.get(mission_id).ok_or("Mission not found")?;
        let completed = self
            .completions
            .entry(player_id.to_string())
            .or_insert_with(HashSet::new);
        if completed.contains(mission_id) {
            return Err("Mission already completed".into());
        }
        let reward = self.config.base_mission_reward * mission.reward_multiplier;
        let balance = self
            .player_balances
            .entry(player_id.to_string())
            .or_insert(0.0);
        *balance += reward;
        completed.insert(mission_id.to_string());
        Ok(RewardReceipt {
            player_id: player_id.to_string(),
            tokens_awarded: reward,
            new_balance: *balance,
            reason: format!("Completed mission {}", mission.name),
            timestamp: Utc::now().naive_utc(),
        })
    }

    pub fn record_behavior(
        &mut self,
        player_id: &str,
        behavior: PositiveBehavior,
    ) -> RewardReceipt {
        let award = *self.config.behavior_rewards.get(&behavior).unwrap_or(&10.0);
        let balance = self
            .player_balances
            .entry(player_id.to_string())
            .or_insert(0.0);
        *balance += award;
        RewardReceipt {
            player_id: player_id.to_string(),
            tokens_awarded: award,
            new_balance: *balance,
            reason: format!("Positive behavior: {}", behavior.description()),
            timestamp: Utc::now().naive_utc(),
        }
    }

    pub fn get_balance(&self, player_id: &str) -> f64 {
        *self.player_balances.get(player_id).unwrap_or(&0.0)
    }
}
