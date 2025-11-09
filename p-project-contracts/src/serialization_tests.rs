//! Comprehensive tests for data serialization/deserialization

#[cfg(test)]
mod tests {
    use crate::token::{PProjectToken, TokenEvent};
    use crate::staking::StakingContract;
    use crate::airdrop::AirdropContract;
    use p_project_core::models::{TokenTransaction, TransactionType, StakingInfo as CoreStakingInfo};
    use serde_json;

    #[test]
    fn test_token_serialization() {
        // Create a token instance
        let mut token = PProjectToken::new(1000000.0, 0.01, 0.02);
        token.initialize_distribution(vec![("user1".to_string(), 1000.0), ("user2".to_string(), 500.0)]);
        
        // Serialize to JSON
        let json = serde_json::to_string(&token).expect("Failed to serialize token");
        assert!(!json.is_empty());
        
        // Deserialize from JSON
        let deserialized_token: PProjectToken = serde_json::from_str(&json).expect("Failed to deserialize token");
        
        // Verify the deserialized token has the same properties
        assert_eq!(deserialized_token.get_total_supply(), token.get_total_supply());
        assert_eq!(deserialized_token.get_balance("user1"), token.get_balance("user1"));
        assert_eq!(deserialized_token.get_balance("user2"), token.get_balance("user2"));
    }

    #[test]
    fn test_token_event_serialization() {
        // Create a token event
        let event = TokenEvent {
            event_type: "TRANSFER".to_string(),
            user_id: "user1".to_string(),
            amount: 100.0,
            timestamp: chrono::Utc::now().naive_utc(),
            details: "Test transfer".to_string(),
        };
        
        // Serialize to JSON
        let json = serde_json::to_string(&event).expect("Failed to serialize token event");
        assert!(!json.is_empty());
        
        // Deserialize from JSON
        let deserialized_event: TokenEvent = serde_json::from_str(&json).expect("Failed to deserialize token event");
        
        // Verify the deserialized event has the same properties
        assert_eq!(deserialized_event.event_type, event.event_type);
        assert_eq!(deserialized_event.user_id, event.user_id);
        assert_eq!(deserialized_event.amount, event.amount);
        assert_eq!(deserialized_event.details, event.details);
    }

    #[test]
    fn test_token_transaction_serialization() {
        // Create a token transaction
        let transaction = TokenTransaction {
            id: "tx1".to_string(),
            from_user_id: "user1".to_string(),
            to_user_id: "user2".to_string(),
            amount: 100.0,
            transaction_type: TransactionType::Transfer,
            timestamp: chrono::Utc::now().naive_utc(),
        };
        
        // Serialize to JSON
        let json = serde_json::to_string(&transaction).expect("Failed to serialize token transaction");
        assert!(!json.is_empty());
        
        // Deserialize from JSON
        let deserialized_transaction: TokenTransaction = serde_json::from_str(&json).expect("Failed to deserialize token transaction");
        
        // Verify the deserialized transaction has the same properties
        assert_eq!(deserialized_transaction.id, transaction.id);
        assert_eq!(deserialized_transaction.from_user_id, transaction.from_user_id);
        assert_eq!(deserialized_transaction.to_user_id, transaction.to_user_id);
        assert_eq!(deserialized_transaction.amount, transaction.amount);
        // Note: We can't directly compare TransactionType enums without PartialEq
        assert_eq!(deserialized_transaction.timestamp, transaction.timestamp);
    }

    #[test]
    fn test_staking_contract_serialization() {
        // Create a staking contract
        let staking = StakingContract::new();
        
        // Serialize to JSON
        let json = serde_json::to_string(&staking).expect("Failed to serialize staking contract");
        assert!(!json.is_empty());
        
        // Deserialize from JSON
        let deserialized_staking: StakingContract = serde_json::from_str(&json).expect("Failed to deserialize staking contract");
        
        // Verify the deserialized staking contract has the same properties
        assert_eq!(deserialized_staking.get_staking_tiers().len(), staking.get_staking_tiers().len());
    }

    #[test]
    fn test_core_staking_info_serialization() {
        // Create a core staking info
        let staking_info = CoreStakingInfo {
            user_id: "user1".to_string(),
            amount: 1000.0,
            start_time: chrono::Utc::now().naive_utc(),
            end_time: None,
            rewards_earned: 50.0,
            tier_name: Some("Basic".to_string()),
            is_compounding: true,
        };
        
        // Serialize to JSON
        let json = serde_json::to_string(&staking_info).expect("Failed to serialize core staking info");
        assert!(!json.is_empty());
        
        // Deserialize from JSON
        let deserialized_staking_info: CoreStakingInfo = serde_json::from_str(&json).expect("Failed to deserialize core staking info");
        
        // Verify the deserialized staking info has the same properties
        assert_eq!(deserialized_staking_info.user_id, staking_info.user_id);
        assert_eq!(deserialized_staking_info.amount, staking_info.amount);
        assert_eq!(deserialized_staking_info.rewards_earned, staking_info.rewards_earned);
        assert_eq!(deserialized_staking_info.tier_name, staking_info.tier_name);
        assert_eq!(deserialized_staking_info.is_compounding, staking_info.is_compounding);
    }

    #[test]
    fn test_airdrop_contract_serialization() {
        // Create an airdrop contract
        let mut airdrop = AirdropContract::new(100000.0);
        airdrop.pause(); // Pause it to test the paused field
        
        // Serialize to JSON
        let json = serde_json::to_string(&airdrop).expect("Failed to serialize airdrop contract");
        assert!(!json.is_empty());
        
        // Deserialize from JSON
        let deserialized_airdrop: AirdropContract = serde_json::from_str(&json).expect("Failed to deserialize airdrop contract");
        
        // Verify the deserialized airdrop contract has the same properties
        assert_eq!(deserialized_airdrop.is_paused(), airdrop.is_paused());
    }

    #[test]
    fn test_complex_nested_serialization() {
        // Test serialization of complex nested structures
        let mut token = PProjectToken::new(1000000.0, 0.01, 0.02);
        token.initialize_distribution(vec![("user1".to_string(), 1000.0), ("user2".to_string(), 500.0)]);
        
        // Add some transactions and events
        token.transfer("user1", "user2", 100.0).unwrap();
        
        // Serialize the entire token state
        let json = serde_json::to_string(&token).expect("Failed to serialize complex token");
        assert!(!json.is_empty());
        
        // Deserialize and verify
        let deserialized_token: PProjectToken = serde_json::from_str(&json).expect("Failed to deserialize complex token");
        assert_eq!(deserialized_token.get_total_supply(), token.get_total_supply());
        assert_eq!(deserialized_token.get_transaction_log().len(), token.get_transaction_log().len());
        assert_eq!(deserialized_token.get_event_log().len(), token.get_event_log().len());
    }
}