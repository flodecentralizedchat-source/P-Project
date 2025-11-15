#[cfg(test)]
mod tests {
    use super::super::game_currency_service::{
        GameCurrencyConfig, GameCurrencyService, PeacefulMission, PositiveBehavior,
    };
    use std::collections::HashMap;

    fn config() -> GameCurrencyConfig {
        let mut behavior_rewards = HashMap::new();
        behavior_rewards.insert(PositiveBehavior::HelpingHands, 15.0);
        behavior_rewards.insert(PositiveBehavior::EnvironmentalCare, 12.0);
        GameCurrencyConfig {
            base_mission_reward: 20.0,
            behavior_rewards,
        }
    }

    fn stub_mission(id: &str) -> PeacefulMission {
        PeacefulMission {
            mission_id: id.to_string(),
            name: "Peace Walk".to_string(),
            description: "Walk through the campus picking up waste".to_string(),
            reward_multiplier: 1.5,
            tags: vec!["exploration".to_string(), "community".to_string()],
            is_nonviolent: true,
            created_at: chrono::Utc::now().naive_utc(),
        }
    }

    #[test]
    fn completing_mission_awards_tokens_once() {
        let mut service = GameCurrencyService::new(config());
        let mission = stub_mission("mission-1");
        let mission = service.register_mission(mission).unwrap();

        let receipt = service
            .complete_mission("player1", &mission.mission_id)
            .unwrap();
        assert_eq!(receipt.tokens_awarded, 20.0 * 1.5);
        assert_eq!(receipt.new_balance, 20.0 * 1.5);
        assert!(receipt.reason.contains("Peace Walk"));

        let err = service.complete_mission("player1", &mission.mission_id);
        assert!(err.is_err());
    }

    #[test]
    fn behavior_rewards_track_balance() {
        let mut service = GameCurrencyService::new(config());
        let receipt = service.record_behavior("player2", PositiveBehavior::HelpingHands);
        assert_eq!(receipt.tokens_awarded, 15.0);
        assert_eq!(service.get_balance("player2"), 15.0);

        let receipt2 = service.record_behavior("player2", PositiveBehavior::EnvironmentalCare);
        assert_eq!(receipt2.tokens_awarded, 12.0);
        assert_eq!(service.get_balance("player2"), 27.0);
    }

    #[test]
    fn mission_registration_validates_nature() {
        let mut service = GameCurrencyService::new(config());
        let mut mission = stub_mission("mission-2");
        mission.is_nonviolent = false;
        assert!(service.register_mission(mission).is_err());
    }
}
