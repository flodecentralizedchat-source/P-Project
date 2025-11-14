#[cfg(test)]
mod tests {
    use super::super::l2_rollup::{L2Rollup, L2Transaction, RollupConfig, RollupError};
    use chrono::Utc;

    #[test]
    fn test_create_rollup() {
        let config = RollupConfig {
            chain_id: "test-chain".to_string(),
            operator_address: "operator1".to_string(),
            batch_submission_interval: 300,
            max_batch_size: 100,
            gas_price: 0.001,
        };

        let rollup = L2Rollup::new(config.clone());

        assert_eq!(rollup.config.chain_id, "test-chain");
        assert_eq!(rollup.config.operator_address, "operator1");
        assert_eq!(rollup.config.batch_submission_interval, 300);
        assert_eq!(rollup.config.max_batch_size, 100);
        assert_eq!(rollup.config.gas_price, 0.001);
        assert_eq!(rollup.latest_block_number, 0);
        assert_eq!(rollup.latest_batch_id, 0);
        assert_eq!(rollup.pending_transactions.len(), 0);
        assert_eq!(rollup.blocks.len(), 0);
        assert_eq!(rollup.batches.len(), 0);
    }

    #[test]
    fn test_initialize_account() {
        let config = RollupConfig {
            chain_id: "test-chain".to_string(),
            operator_address: "operator1".to_string(),
            batch_submission_interval: 300,
            max_batch_size: 100,
            gas_price: 0.001,
        };

        let mut rollup = L2Rollup::new(config);
        rollup.initialize_account("user1".to_string(), 1000.0);

        assert_eq!(rollup.get_balance("user1"), 1000.0);
        assert_eq!(rollup.get_balance("user2"), 0.0);
    }

    #[test]
    fn test_add_transaction() {
        let config = RollupConfig {
            chain_id: "test-chain".to_string(),
            operator_address: "operator1".to_string(),
            batch_submission_interval: 300,
            max_batch_size: 100,
            gas_price: 0.001,
        };

        let mut rollup = L2Rollup::new(config);
        rollup.initialize_account("user1".to_string(), 1000.0);

        let transaction = L2Transaction {
            from: "user1".to_string(),
            to: "user2".to_string(),
            amount: 100.0,
            nonce: 0,
            signature: "sig1".to_string(),
            timestamp: Utc::now().naive_utc(),
        };

        let result = rollup.add_transaction(transaction);
        assert!(result.is_ok());
        assert_eq!(rollup.pending_transactions.len(), 1);
    }

    #[test]
    fn test_add_invalid_transaction() {
        let config = RollupConfig {
            chain_id: "test-chain".to_string(),
            operator_address: "operator1".to_string(),
            batch_submission_interval: 300,
            max_batch_size: 100,
            gas_price: 0.001,
        };

        let mut rollup = L2Rollup::new(config);

        // Test negative amount
        let transaction = L2Transaction {
            from: "user1".to_string(),
            to: "user2".to_string(),
            amount: -100.0,
            nonce: 0,
            signature: "sig1".to_string(),
            timestamp: Utc::now().naive_utc(),
        };

        let result = rollup.add_transaction(transaction);
        assert_eq!(result, Err(RollupError::InvalidTransaction));

        // Test insufficient balance
        let transaction = L2Transaction {
            from: "user1".to_string(),
            to: "user2".to_string(),
            amount: 100.0,
            nonce: 0,
            signature: "sig1".to_string(),
            timestamp: Utc::now().naive_utc(),
        };

        let result = rollup.add_transaction(transaction);
        assert_eq!(result, Err(RollupError::InsufficientBalance));
    }

    #[test]
    fn test_create_block() {
        let config = RollupConfig {
            chain_id: "test-chain".to_string(),
            operator_address: "operator1".to_string(),
            batch_submission_interval: 300,
            max_batch_size: 100,
            gas_price: 0.001,
        };

        let mut rollup = L2Rollup::new(config);
        rollup.initialize_account("user1".to_string(), 1000.0);

        let transaction = L2Transaction {
            from: "user1".to_string(),
            to: "user2".to_string(),
            amount: 100.0,
            nonce: 0,
            signature: "sig1".to_string(),
            timestamp: Utc::now().naive_utc(),
        };

        rollup.add_transaction(transaction).unwrap();
        assert_eq!(rollup.pending_transactions.len(), 1);

        let result = rollup.create_block();
        assert!(result.is_ok());
        assert_eq!(rollup.pending_transactions.len(), 0);
        assert_eq!(rollup.blocks.len(), 1);
        assert_eq!(rollup.latest_block_number, 1);
    }

    #[test]
    fn test_create_block_empty() {
        let config = RollupConfig {
            chain_id: "test-chain".to_string(),
            operator_address: "operator1".to_string(),
            batch_submission_interval: 300,
            max_batch_size: 100,
            gas_price: 0.001,
        };

        let mut rollup = L2Rollup::new(config);
        let result = rollup.create_block();
        assert_eq!(result, Err(RollupError::InvalidBlock));
    }

    #[test]
    fn test_submit_batch() {
        let config = RollupConfig {
            chain_id: "test-chain".to_string(),
            operator_address: "operator1".to_string(),
            batch_submission_interval: 300,
            max_batch_size: 100,
            gas_price: 0.001,
        };

        let mut rollup = L2Rollup::new(config);
        rollup.initialize_account("user1".to_string(), 1000.0);

        let transaction = L2Transaction {
            from: "user1".to_string(),
            to: "user2".to_string(),
            amount: 100.0,
            nonce: 0,
            signature: "sig1".to_string(),
            timestamp: Utc::now().naive_utc(),
        };

        rollup.add_transaction(transaction).unwrap();
        assert_eq!(rollup.pending_transactions.len(), 1);

        let state_root_before = rollup.get_state_root().to_string();
        let result = rollup.submit_batch();
        assert!(result.is_ok());
        assert_eq!(rollup.pending_transactions.len(), 0);
        assert_eq!(rollup.batches.len(), 1);
        assert_eq!(rollup.latest_batch_id, 1);

        let batch = result.unwrap();
        assert_eq!(batch.batch_id, "batch_1");
        assert_eq!(batch.transactions.len(), 1);
        assert_eq!(batch.state_root_before, state_root_before);
    }

    #[test]
    fn test_process_transaction() {
        let config = RollupConfig {
            chain_id: "test-chain".to_string(),
            operator_address: "operator1".to_string(),
            batch_submission_interval: 300,
            max_batch_size: 100,
            gas_price: 0.001,
        };

        let mut rollup = L2Rollup::new(config);
        rollup.initialize_account("user1".to_string(), 1000.0);
        rollup.initialize_account("user2".to_string(), 0.0);

        let transaction = L2Transaction {
            from: "user1".to_string(),
            to: "user2".to_string(),
            amount: 100.0,
            nonce: 0,
            signature: "sig1".to_string(),
            timestamp: Utc::now().naive_utc(),
        };

        rollup.add_transaction(transaction).unwrap();
        let block = rollup.create_block().unwrap();

        assert_eq!(rollup.get_balance("user1"), 900.0);
        assert_eq!(rollup.get_balance("user2"), 100.0);
        assert_eq!(block.transactions.len(), 1);
    }
}