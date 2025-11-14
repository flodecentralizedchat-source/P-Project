#[cfg(test)]
mod tests {
    use super::super::l2_rollup::{L2Rollup, RollupConfig};
    use super::super::l2_cross_chain::{L2CrossChainProtocol, CrossChainMessage};

    #[test]
    fn test_create_cross_chain_protocol() {
        let config = RollupConfig {
            chain_id: "test-chain".to_string(),
            operator_address: "operator1".to_string(),
            batch_submission_interval: 300,
            max_batch_size: 100,
            gas_price: 0.001,
        };

        let rollup = L2Rollup::new(config);
        let protocol = L2CrossChainProtocol::new(rollup, "test-chain".to_string());

        assert_eq!(protocol.chain_id, "test-chain");
        assert_eq!(protocol.connected_chains.len(), 0);
        assert_eq!(protocol.bridge_state.locked_tokens.len(), 0);
        assert_eq!(protocol.bridge_state.processed_messages.len(), 0);
        assert_eq!(protocol.bridge_state.pending_messages.len(), 0);
    }

    #[test]
    fn test_add_connected_chain() {
        let config = RollupConfig {
            chain_id: "test-chain".to_string(),
            operator_address: "operator1".to_string(),
            batch_submission_interval: 300,
            max_batch_size: 100,
            gas_price: 0.001,
        };

        let rollup = L2Rollup::new(config);
        let mut protocol = L2CrossChainProtocol::new(rollup, "test-chain".to_string());

        protocol.add_connected_chain("ethereum".to_string());
        protocol.add_connected_chain("polygon".to_string());
        protocol.add_connected_chain("ethereum".to_string()); // Duplicate

        assert_eq!(protocol.connected_chains.len(), 2);
        assert!(protocol.connected_chains.contains(&"ethereum".to_string()));
        assert!(protocol.connected_chains.contains(&"polygon".to_string()));
    }

    #[test]
    fn test_lock_tokens() {
        let config = RollupConfig {
            chain_id: "test-chain".to_string(),
            operator_address: "operator1".to_string(),
            batch_submission_interval: 300,
            max_batch_size: 100,
            gas_price: 0.001,
        };

        let mut rollup = L2Rollup::new(config);
        rollup.initialize_account("user1".to_string(), 1000.0);

        let mut protocol = L2CrossChainProtocol::new(rollup, "test-chain".to_string());

        let result = protocol.lock_tokens("user1".to_string(), "P".to_string(), 100.0);
        assert!(result.is_ok());

        let lock_id = result.unwrap();
        assert!(!lock_id.is_empty());

        let bridge_status = protocol.get_bridge_status();
        assert_eq!(*bridge_status.locked_tokens.get("P").unwrap(), 100.0);
    }

    #[test]
    fn test_lock_tokens_insufficient_balance() {
        let config = RollupConfig {
            chain_id: "test-chain".to_string(),
            operator_address: "operator1".to_string(),
            batch_submission_interval: 300,
            max_batch_size: 100,
            gas_price: 0.001,
        };

        let mut rollup = L2Rollup::new(config);
        rollup.initialize_account("user1".to_string(), 50.0);

        let mut protocol = L2CrossChainProtocol::new(rollup, "test-chain".to_string());

        let result = protocol.lock_tokens("user1".to_string(), "P".to_string(), 100.0);
        assert_eq!(result, Err(super::super::l2_rollup::RollupError::InsufficientBalance));
    }

    #[test]
    fn test_create_cross_chain_message() {
        let config = RollupConfig {
            chain_id: "test-chain".to_string(),
            operator_address: "operator1".to_string(),
            batch_submission_interval: 300,
            max_batch_size: 100,
            gas_price: 0.001,
        };

        let mut rollup = L2Rollup::new(config);
        rollup.initialize_account("user1".to_string(), 1000.0);

        let mut protocol = L2CrossChainProtocol::new(rollup, "test-chain".to_string());
        protocol.add_connected_chain("ethereum".to_string());

        let result = protocol.create_cross_chain_message(
            "test-chain".to_string(),
            "ethereum".to_string(),
            "user1".to_string(),
            "user2".to_string(),
            100.0,
            "P".to_string(),
            vec![1, 2, 3, 4],
        );

        assert!(result.is_ok());
        assert_eq!(protocol.bridge_state.pending_messages.len(), 1);

        let message = result.unwrap();
        assert_eq!(message.source_chain, "test-chain");
        assert_eq!(message.destination_chain, "ethereum");
        assert_eq!(message.sender, "user1");
        assert_eq!(message.recipient, "user2");
        assert_eq!(message.amount, 100.0);
        assert_eq!(message.token, "P");
        assert_eq!(message.payload, vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_create_cross_chain_message_disconnected_chain() {
        let config = RollupConfig {
            chain_id: "test-chain".to_string(),
            operator_address: "operator1".to_string(),
            batch_submission_interval: 300,
            max_batch_size: 100,
            gas_price: 0.001,
        };

        let mut rollup = L2Rollup::new(config);
        rollup.initialize_account("user1".to_string(), 1000.0);

        let mut protocol = L2CrossChainProtocol::new(rollup, "test-chain".to_string());

        let result = protocol.create_cross_chain_message(
            "test-chain".to_string(),
            "ethereum".to_string(),
            "user1".to_string(),
            "user2".to_string(),
            100.0,
            "P".to_string(),
            vec![1, 2, 3, 4],
        );

        assert_eq!(result, Err(super::super::l2_rollup::RollupError::InvalidTransaction));
    }

    #[test]
    fn test_process_incoming_message() {
        let config = RollupConfig {
            chain_id: "test-chain".to_string(),
            operator_address: "operator1".to_string(),
            batch_submission_interval: 300,
            max_batch_size: 100,
            gas_price: 0.001,
        };

        let mut rollup = L2Rollup::new(config);
        rollup.initialize_account("bridge".to_string(), 10000.0); // Initialize bridge with funds

        let mut protocol = L2CrossChainProtocol::new(rollup, "test-chain".to_string());

        let message = CrossChainMessage {
            message_id: "msg1".to_string(),
            source_chain: "ethereum".to_string(),
            destination_chain: "test-chain".to_string(),
            sender: "user1".to_string(),
            recipient: "user2".to_string(),
            amount: 100.0,
            token: "P".to_string(),
            payload: vec![1, 2, 3, 4],
            timestamp: 1234567890,
            signature: "sig1".to_string(),
        };

        let result = protocol.process_incoming_message(message);
        assert!(result.is_ok());

        let bridge_status = protocol.get_bridge_status();
        assert!(bridge_status.processed_messages.contains_key("msg1"));
        assert_eq!(*bridge_status.processed_messages.get("msg1").unwrap(), true);
    }

    #[test]
    fn test_release_tokens() {
        let config = RollupConfig {
            chain_id: "test-chain".to_string(),
            operator_address: "operator1".to_string(),
            batch_submission_interval: 300,
            max_batch_size: 100,
            gas_price: 0.001,
        };

        let mut rollup = L2Rollup::new(config);
        rollup.initialize_account("user1".to_string(), 1000.0);
        rollup.initialize_account("bridge".to_string(), 10000.0);

        let mut protocol = L2CrossChainProtocol::new(rollup, "test-chain".to_string());

        // Lock some tokens first
        protocol.lock_tokens("user1".to_string(), "P".to_string(), 100.0).unwrap();

        let result = protocol.release_tokens("user1".to_string(), "P".to_string(), 50.0);
        assert!(result.is_ok());

        let bridge_status = protocol.get_bridge_status();
        assert_eq!(*bridge_status.locked_tokens.get("P").unwrap(), 50.0);
    }

    #[test]
    fn test_submit_message_batch() {
        let config = RollupConfig {
            chain_id: "test-chain".to_string(),
            operator_address: "operator1".to_string(),
            batch_submission_interval: 300,
            max_batch_size: 100,
            gas_price: 0.001,
        };

        let mut rollup = L2Rollup::new(config);
        rollup.initialize_account("user1".to_string(), 1000.0);

        let mut protocol = L2CrossChainProtocol::new(rollup, "test-chain".to_string());
        protocol.add_connected_chain("ethereum".to_string());

        // Create a message
        protocol.create_cross_chain_message(
            "test-chain".to_string(),
            "ethereum".to_string(),
            "user1".to_string(),
            "user2".to_string(),
            100.0,
            "P".to_string(),
            vec![1, 2, 3, 4],
        ).unwrap();

        assert_eq!(protocol.bridge_state.pending_messages.len(), 1);

        let result = protocol.submit_message_batch();
        assert!(result.is_ok());

        let messages = result.unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(protocol.bridge_state.pending_messages.len(), 0);
    }
}