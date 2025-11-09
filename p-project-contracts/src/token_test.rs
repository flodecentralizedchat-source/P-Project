//! Tests for token contract with database integration

#[cfg(test)]
mod tests {
    use crate::token::{PProjectToken, TokenError};
    use p_project_core::models::{TokenTransaction, TransactionType};
    use std::collections::HashMap;

    #[test]
    fn test_token_creation() {
        let token = PProjectToken::new(1000000.0, 0.01, 0.02);
        assert_eq!(token.get_total_supply(), 1000000.0);
        assert_eq!(token.get_max_transfer_limit(), 50000.0); // 5% of total supply
    }

    #[test]
    fn test_transfer_success() {
        let mut token = PProjectToken::new(1000000.0, 0.01, 0.02);
        token.initialize_distribution(vec![("user1".to_string(), 1000.0), ("user2".to_string(), 500.0)]);
        
        let result = token.transfer("user1", "user2", 100.0);
        assert!(result.is_ok());
        
        // Check balances after transfer
        // User1 should have 900 (1000 - 100)
        // User2 should have 599 (500 + 99, 1 burned)
        assert_eq!(token.get_balance("user1"), 900.0);
        assert_eq!(token.get_balance("user2"), 599.0);
    }

    #[test]
    fn test_transfer_exceeds_limit() {
        let mut token = PProjectToken::new(1000000.0, 0.01, 0.02);
        token.initialize_distribution(vec![("user1".to_string(), 100000.0)]);
        token.set_max_transfer_limit(1000.0);
        
        let result = token.transfer("user1", "user2", 1500.0);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), TokenError::TransferLimitExceeded(1000.0));
    }

    #[test]
    fn test_transfer_insufficient_balance() {
        let mut token = PProjectToken::new(1000000.0, 0.01, 0.02);
        token.initialize_distribution(vec![("user1".to_string(), 100.0)]);
        
        let result = token.transfer("user1", "user2", 200.0);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), TokenError::InsufficientBalance);
    }

    #[test]
    fn test_freeze_unfreeze_tokens() {
        let mut token = PProjectToken::new(1000000.0, 0.01, 0.02);
        token.initialize_distribution(vec![("user1".to_string(), 1000.0)]);
        
        // Freeze tokens
        let result = token.freeze_tokens("user1", 200.0);
        assert!(result.is_ok());
        assert_eq!(token.get_balance("user1"), 800.0);
        assert_eq!(token.get_frozen_balance("user1"), 200.0);
        
        // Unfreeze tokens
        let result = token.unfreeze_tokens("user1", 100.0);
        assert!(result.is_ok());
        assert_eq!(token.get_balance("user1"), 900.0);
        assert_eq!(token.get_frozen_balance("user1"), 100.0);
    }

    #[test]
    fn test_freeze_insufficient_balance() {
        let mut token = PProjectToken::new(1000000.0, 0.01, 0.02);
        token.initialize_distribution(vec![("user1".to_string(), 100.0)]);
        
        let result = token.freeze_tokens("user1", 200.0);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), TokenError::InsufficientBalance);
    }

    #[test]
    fn test_unfreeze_insufficient_frozen_balance() {
        let mut token = PProjectToken::new(1000000.0, 0.01, 0.02);
        token.initialize_distribution(vec![("user1".to_string(), 1000.0)]);
        token.freeze_tokens("user1", 200.0).unwrap();
        
        let result = token.unfreeze_tokens("user1", 300.0);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), TokenError::InsufficientFrozenBalance);
    }

    #[test]
    fn test_liquidity_operations() {
        let mut token = PProjectToken::new(1000000.0, 0.01, 0.02);
        token.initialize_distribution(vec![("user1".to_string(), 1000.0)]);
        
        // Add liquidity
        let result = token.add_liquidity("pool1".to_string(), "user1", 200.0);
        assert!(result.is_ok());
        assert_eq!(token.get_balance("user1"), 800.0);
        assert_eq!(token.get_pool_liquidity("pool1"), 200.0);
        
        // Remove liquidity
        let result = token.remove_liquidity("pool1".to_string(), "user1", 100.0);
        assert!(result.is_ok());
        assert_eq!(token.get_balance("user1"), 900.0);
        assert_eq!(token.get_pool_liquidity("pool1"), 100.0);
    }

    #[test]
    fn test_liquidity_insufficient_balance() {
        let mut token = PProjectToken::new(1000000.0, 0.01, 0.02);
        token.initialize_distribution(vec![("user1".to_string(), 100.0)]);
        
        let result = token.add_liquidity("pool1".to_string(), "user1", 200.0);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), TokenError::InsufficientBalance);
    }

    #[test]
    fn test_liquidity_insufficient_pool() {
        let mut token = PProjectToken::new(1000000.0, 0.01, 0.02);
        token.initialize_distribution(vec![("user1".to_string(), 1000.0)]);
        token.add_liquidity("pool1".to_string(), "user1", 200.0).unwrap();
        
        let result = token.remove_liquidity("pool1".to_string(), "user1", 300.0);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), TokenError::InsufficientBalance);
    }

    #[test]
    fn test_event_logging() {
        let mut token = PProjectToken::new(1000000.0, 0.01, 0.02);
        token.initialize_distribution(vec![("user1".to_string(), 1000.0)]);
        
        token.transfer("user1", "user2", 100.0).unwrap();
        
        let events = token.get_event_log();
        assert!(!events.is_empty());
        assert_eq!(events[0].event_type, "TRANSFER");
        assert_eq!(events[0].user_id, "user1");
        assert_eq!(events[0].amount, 100.0);
    }

    #[test]
    fn test_transaction_logging() {
        let mut token = PProjectToken::new(1000000.0, 0.01, 0.02);
        token.initialize_distribution(vec![("user1".to_string(), 1000.0)]);
        
        token.transfer("user1", "user2", 100.0).unwrap();
        
        let transactions = token.get_transaction_log();
        assert!(!transactions.is_empty());
        assert_eq!(transactions[0].from_user_id, "user1");
        assert_eq!(transactions[0].to_user_id, "user2");
        assert_eq!(transactions[0].amount, 100.0);
        assert_eq!(transactions[0].transaction_type, TransactionType::Transfer);
    }
}