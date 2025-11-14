#[cfg(test)]
mod tests {
    use super::super::l2_batching::{TransactionBatch, BatchConfig, BatchSubmissionResult};
    use super::super::l2_rollup::{L2Transaction, TransactionType};

    #[test]
    fn test_create_batch_config() {
        let config = BatchConfig {
            max_batch_size: 100,
            batch_timeout: 300,
            gas_limit_per_batch: 1000000,
            max_transactions_per_batch: 50,
        };

        assert_eq!(config.max_batch_size, 100);
        assert_eq!(config.batch_timeout, 300);
        assert_eq!(config.gas_limit_per_batch, 1000000);
        assert_eq!(config.max_transactions_per_batch, 50);
    }

    #[test]
    fn test_create_transaction_batch() {
        let tx = L2Transaction {
            tx_id: "tx1".to_string(),
            from: "user1".to_string(),
            to: "user2".to_string(),
            amount: 100.0,
            fee: 0.001,
            nonce: 1,
            timestamp: 1234567890,
            signature: "sig1".to_string(),
            tx_type: TransactionType::Transfer,
        };

        let batch = TransactionBatch {
            batch_id: "batch1".to_string(),
            transactions: vec![tx.clone()],
            block_numbers: vec![1],
            gas_used: 50000,
            timestamp: 1234567890,
            submitter: "operator1".to_string(),
            signature: "batch_sig1".to_string(),
            state_root_before: "root_before".to_string(),
            state_root_after: "root_after".to_string(),
        };

        assert_eq!(batch.batch_id, "batch1");
        assert_eq!(batch.transactions.len(), 1);
        assert_eq!(batch.block_numbers, vec![1]);
        assert_eq!(batch.gas_used, 50000);
        assert_eq!(batch.submitter, "operator1");
        assert_eq!(batch.signature, "batch_sig1");
        assert_eq!(batch.state_root_before, "root_before");
        assert_eq!(batch.state_root_after, "root_after");
    }

    #[test]
    fn test_batch_submission_result() {
        let result = BatchSubmissionResult {
            batch_id: "batch1".to_string(),
            transaction_hash: "hash1".to_string(),
            block_number: 100,
            gas_used: 75000,
            success: true,
            error_message: None,
        };

        assert_eq!(result.batch_id, "batch1");
        assert_eq!(result.transaction_hash, "hash1");
        assert_eq!(result.block_number, 100);
        assert_eq!(result.gas_used, 75000);
        assert_eq!(result.success, true);
        assert_eq!(result.error_message, None);
    }
}