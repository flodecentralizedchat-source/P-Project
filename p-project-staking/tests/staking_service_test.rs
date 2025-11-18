use p_project_contracts::staking::StakingContract;
use p_project_staking::{StakeDb, StakingService};
use rust_decimal::prelude::*;
use rust_decimal::Decimal;

struct FakeDb {
    pub saved_states: usize,
    pub last_stake: Option<(String, String, Decimal, i64)>,
    pub last_unstake: Option<(String, Option<String>)>,
}

#[async_trait::async_trait]
impl StakeDb for FakeDb {
    async fn stake_tokens(
        &self,
        _stake_id: &str,
        _user_id: &str,
        _amount: Decimal,
        _duration_days: i64,
    ) -> Result<p_project_core::models::StakingInfo, String> {
        // Return a minimal staking info; callers don't rely on it in these tests
        Ok(p_project_core::models::StakingInfo {
            user_id: _user_id.to_string(),
            amount: _amount,
            start_time: chrono::Utc::now().naive_utc(),
            end_time: None,
            rewards_earned: Decimal::ZERO,
            tier_name: None,
            is_compounding: false,
        })
    }

    async fn unstake_tokens(
        &self,
        _user_id: &str,
        _stake_id: Option<&str>,
    ) -> Result<p_project_core::models::StakingInfo, String> {
        Ok(p_project_core::models::StakingInfo {
            user_id: _user_id.to_string(),
            amount: Decimal::ZERO,
            start_time: chrono::Utc::now().naive_utc(),
            end_time: Some(chrono::Utc::now().naive_utc()),
            rewards_earned: Decimal::ZERO,
            tier_name: None,
            is_compounding: false,
        })
    }

    async fn save_staking_state(&self, _state_data: &str) -> Result<(), String> {
        Ok(())
    }
}

#[tokio::test]
async fn stake_and_unstake_happy_path() {
    let contract = StakingContract::new();
    let db = FakeDb {
        saved_states: 0,
        last_stake: None,
        last_unstake: None,
    };
    let mut svc = StakingService::new(contract, db);

    // 1000 tokens for 90 days should fall into Silver tier (10% APY)
    let tier = svc
        .stake_tokens("user-1".to_string(), 1000.0, 90)
        .await
        .expect("stake ok");
    assert_eq!(tier, "Silver");

    // Contract should reflect a staking position
    let info = svc
        .get_staking_info("user-1")
        .expect("staking info present");
    assert_eq!(info.user_id, "user-1");
    assert_eq!(info.amount, Decimal::from_f64(1000.0).unwrap());

    // Unstake and verify amounts are returned; rewards come from contract calc
    let (amount, rewards) = svc.unstake_tokens("user-1").await.expect("unstake ok");
    assert!(amount > 0.0);
    assert!(rewards >= 0.0);

    // After unstake, info should be gone
    assert!(svc.get_staking_info("user-1").is_none());
}

#[tokio::test]
async fn stake_rejects_invalid_inputs() {
    let contract = StakingContract::new();
    let db = FakeDb {
        saved_states: 0,
        last_stake: None,
        last_unstake: None,
    };
    let mut svc = StakingService::new(contract, db);

    // Negative amount
    assert!(svc.stake_tokens("u".to_string(), -10.0, 30).await.is_err());

    // Zero duration
    assert!(svc.stake_tokens("u".to_string(), 10.0, 0).await.is_err());
}
