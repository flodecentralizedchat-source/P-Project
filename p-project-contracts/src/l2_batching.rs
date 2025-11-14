use crate::l2_rollup::{L2Transaction, L2Block, RollupError};
use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};
use std::collections::HashMap;

// Batch configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchConfig {
    pub max_batch_size: usize,
    pub batch_timeout: u64, // in seconds
    pub gas_limit_per_batch: u64,
    pub max_transactions_per_batch: usize,
}

// Transaction batch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionBatch {
    pub batch_id: String,
    pub transactions: Vec<L2Transaction>,
    pub block_numbers: Vec<u64>,
    pub gas_used: u64,
    pub timestamp: u64,
    pub submitter: String,
    pub signature: String,
    pub state_root_before: String,
    pub state_root_after: String,
}

// Batch submission result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchSubmissionResult {
    pub batch_id: String,
    pub transaction_hash: String,
    pub gas_used: u64,
    pub success: bool,
    pub error_message: Option<String>,
}

// Batch validator
pub struct BatchValidator {
    pub config: BatchConfig,
}

impl BatchValidator {
    pub fn new(config: BatchConfig) -> Self {
        Self { config }
    }

    /// Validate a batch of transactions
    pub fn validate_batch(&self, batch: &TransactionBatch) -> Result<(), RollupError> {
        // Check batch size
        if batch.transactions.len() > self.config.max_transactions_per_batch {
            return Err(RollupError::BatchSubmissionFailed);
        }

        // Check gas limit
        if batch.gas_used > self.config.gas_limit_per_batch {
            return Err(RollupError::BatchSubmissionFailed);
        }

        // Check timestamp is not in the future
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        if batch.timestamp > current_time + 60 {
            return Err(RollupError::BatchSubmissionFailed);
        }

        // Validate each transaction in the batch
        for transaction in &batch.transactions {
            self.validate_transaction(transaction)?;
        }

        Ok(())
    }

    /// Validate a single transaction
    fn validate_transaction(&self, transaction: &L2Transaction) -> Result<(), RollupError> {
        // Check amount is positive
        if transaction.amount <= 0.0 {
            return Err(RollupError::InvalidTransaction);
        }

        // Check addresses are not empty
        if transaction.from.is_empty() || transaction.to.is_empty() {
            return Err(RollupError::InvalidTransaction);
        }

        // Check signature is not empty
        if transaction.signature.is_empty() {
            return Err(RollupError::InvalidTransaction);
        }

        Ok(())
    }
}

// Batch aggregator
pub struct BatchAggregator {
    pub pending_transactions: Vec<L2Transaction>,
    pub batch_config: BatchConfig,
    pub batch_counter: u64,
}

impl BatchAggregator {
    pub fn new(batch_config: BatchConfig) -> Self {
        Self {
            pending_transactions: Vec::new(),
            batch_config,
            batch_counter: 0,
        }
    }

    /// Add a transaction to the pending queue
    pub fn add_transaction(&mut self, transaction: L2Transaction) -> Result<(), RollupError> {
        // Validate transaction first
        let validator = BatchValidator::new(self.batch_config.clone());
        validator.validate_transaction(&transaction)?;
        
        self.pending_transactions.push(transaction);
        Ok(())
    }

    /// Create a batch from pending transactions
    pub fn create_batch(
        &mut self,
        state_root_before: String,
        state_root_after: String,
        submitter: String,
    ) -> Result<TransactionBatch, RollupError> {
        if self.pending_transactions.is_empty() {
            return Err(RollupError::BatchSubmissionFailed);
        }

        // Limit batch size
        let batch_size = std::cmp::min(
            self.pending_transactions.len(),
            self.batch_config.max_transactions_per_batch,
        );
        
        let transactions: Vec<L2Transaction> = self.pending_transactions
            .drain(0..batch_size)
            .collect();

        self.batch_counter += 1;
        let batch_id = format!("batch_{}", self.batch_counter);
        
        // Calculate gas used (simplified)
        let gas_used = transactions.len() as u64 * 21000; // Base gas per transaction

        let batch = TransactionBatch {
            batch_id,
            transactions,
            block_numbers: Vec::new(), // Will be filled when blocks are created
            gas_used,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            submitter,
            signature: String::new(), // Will be filled with actual signature
            state_root_before,
            state_root_after,
        };

        Ok(batch)
    }

    /// Check if a batch should be created based on size or timeout
    pub fn should_create_batch(&self, last_batch_time: u64) -> bool {
        // Check if we have enough transactions
        if self.pending_transactions.len() >= self.batch_config.max_transactions_per_batch {
            return true;
        }

        // Check if timeout has been reached
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        if current_time - last_batch_time >= self.batch_config.batch_timeout {
            return !self.pending_transactions.is_empty();
        }

        false
    }

    /// Get number of pending transactions
    pub fn pending_transaction_count(&self) -> usize {
        self.pending_transactions.len()
    }

    /// Clear pending transactions
    pub fn clear_pending(&mut self) {
        self.pending_transactions.clear();
    }
}

// Batch submitter
pub struct BatchSubmitter {
    pub validator: BatchValidator,
    pub submitted_batches: HashMap<String, BatchSubmissionResult>,
}

impl BatchSubmitter {
    pub fn new(batch_config: BatchConfig) -> Self {
        Self {
            validator: BatchValidator::new(batch_config),
            submitted_batches: HashMap::new(),
        }
    }

    /// Submit a batch to the L1
    pub fn submit_batch(&mut self, batch: TransactionBatch) -> Result<BatchSubmissionResult, RollupError> {
        // Validate batch
        self.validator.validate_batch(&batch)?;

        // Simulate batch submission to L1
        let transaction_hash = self.simulate_batch_submission(&batch)?;
        
        let result = BatchSubmissionResult {
            batch_id: batch.batch_id.clone(),
            transaction_hash,
            gas_used: batch.gas_used,
            success: true,
            error_message: None,
        };

        // Store result
        self.submitted_batches.insert(batch.batch_id.clone(), result.clone());
        
        Ok(result)
    }

    /// Simulate batch submission (in a real implementation, this would interact with L1)
    fn simulate_batch_submission(&self, batch: &TransactionBatch) -> Result<String, RollupError> {
        // Create a hash of the batch data
        let mut hasher = Keccak256::new();
        hasher.update(batch.batch_id.as_bytes());
        hasher.update(batch.state_root_before.as_bytes());
        hasher.update(batch.state_root_after.as_bytes());
        hasher.update(&batch.gas_used.to_le_bytes());
        hasher.update(&batch.timestamp.to_le_bytes());
        
        for tx in &batch.transactions {
            hasher.update(tx.from.as_bytes());
            hasher.update(tx.to.as_bytes());
            hasher.update(&tx.amount.to_le_bytes());
        }
        
        let result = hasher.finalize();
        Ok(format!("0x{:x}", result))
    }

    /// Get submission result for a batch
    pub fn get_submission_result(&self, batch_id: &str) -> Option<&BatchSubmissionResult> {
        self.submitted_batches.get(batch_id)
    }

    /// Get all submission results
    pub fn get_all_results(&self) -> &HashMap<String, BatchSubmissionResult> {
        &self.submitted_batches
    }
}

// Batch compression for efficient L1 submission
pub struct BatchCompressor {
    pub compression_ratio: f64,
}

impl BatchCompressor {
    pub fn new() -> Self {
        Self {
            compression_ratio: 0.5, // 50% compression
        }
    }

    /// Compress batch data
    pub fn compress_batch(&self, batch: &TransactionBatch) -> Result<Vec<u8>, RollupError> {
        // Serialize batch to JSON
        let json_data = serde_json::to_string(batch)
            .map_err(|e| RollupError::SerializationError(e.to_string()))?;
        
        // In a real implementation, we would use actual compression algorithms
        // For now, we'll just convert to bytes
        Ok(json_data.into_bytes())
    }

    /// Decompress batch data
    pub fn decompress_batch(&self, compressed_data: Vec<u8>) -> Result<TransactionBatch, RollupError> {
        // Convert bytes back to string
        let json_data = String::from_utf8(compressed_data)
            .map_err(|e| RollupError::SerializationError(e.to_string()))?;
        
        // Deserialize JSON to batch
        serde_json::from_str(&json_data)
            .map_err(|e| RollupError::SerializationError(e.to_string()))
    }

    /// Estimate compressed size
    pub fn estimate_compressed_size(&self, batch: &TransactionBatch) -> usize {
        // Estimate size based on number of transactions and compression ratio
        let uncompressed_size = batch.transactions.len() * 200; // Rough estimate per transaction
        (uncompressed_size as f64 * self.compression_ratio) as usize
    }
}

// Batch fee calculator
pub struct BatchFeeCalculator {
    pub base_fee_per_batch: f64,
    pub fee_per_transaction: f64,
    pub gas_price: f64,
}

impl BatchFeeCalculator {
    pub fn new(base_fee_per_batch: f64, fee_per_transaction: f64, gas_price: f64) -> Self {
        Self {
            base_fee_per_batch,
            fee_per_transaction,
            gas_price,
        }
    }

    /// Calculate fee for a batch
    pub fn calculate_batch_fee(&self, transaction_count: usize, gas_used: u64) -> f64 {
        let transaction_fees = transaction_count as f64 * self.fee_per_transaction;
        let gas_fees = gas_used as f64 * self.gas_price;
        self.base_fee_per_batch + transaction_fees + gas_fees
    }

    /// Calculate fee for a single transaction
    pub fn calculate_transaction_fee(&self) -> f64 {
        self.fee_per_transaction + (21000.0 * self.gas_price)
    }
}