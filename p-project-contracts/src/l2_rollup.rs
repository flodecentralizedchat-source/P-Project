use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};
use std::collections::HashMap;

// Custom error types for L2 rollup operations
#[derive(Debug, Clone, PartialEq)]
pub enum RollupError {
    InvalidTransaction,
    InsufficientBalance,
    InvalidBlock,
    BatchSubmissionFailed,
    StateRootMismatch,
    SerializationError(String),
    MerkleTreeError,
}

impl std::fmt::Display for RollupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RollupError::InvalidTransaction => write!(f, "Invalid transaction"),
            RollupError::InsufficientBalance => write!(f, "Insufficient balance"),
            RollupError::InvalidBlock => write!(f, "Invalid block"),
            RollupError::BatchSubmissionFailed => write!(f, "Batch submission failed"),
            RollupError::StateRootMismatch => write!(f, "State root mismatch"),
            RollupError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            RollupError::MerkleTreeError => write!(f, "Merkle tree error"),
        }
    }
}

impl std::error::Error for RollupError {}

// Transaction structure for L2
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct L2Transaction {
    pub from: String,
    pub to: String,
    pub amount: f64,
    pub nonce: u64,
    pub signature: String,
    pub timestamp: NaiveDateTime,
}

// Block structure for L2
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct L2Block {
    pub block_number: u64,
    pub transactions: Vec<L2Transaction>,
    pub state_root: String,
    pub previous_block_hash: String,
    pub timestamp: NaiveDateTime,
    pub batch_id: Option<String>,
}

// Account structure for L2 state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct L2Account {
    pub address: String,
    pub balance: f64,
    pub nonce: u64,
}

// Rollup configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollupConfig {
    pub chain_id: String,
    pub operator_address: String,
    pub batch_submission_interval: u64, // in seconds
    pub max_batch_size: usize,
    pub gas_price: f64,
}

// Merkle tree node for state commitments
#[derive(Debug, Clone)]
pub struct MerkleNode {
    pub hash: String,
    pub left: Option<Box<MerkleNode>>,
    pub right: Option<Box<MerkleNode>>,
}

// Rollup state manager
pub struct RollupStateManager {
    pub accounts: HashMap<String, L2Account>,
    pub state_root: String,
    pub merkle_tree: Option<MerkleNode>,
}

impl RollupStateManager {
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
            state_root: String::new(),
            merkle_tree: None,
        }
    }

    pub fn get_account(&self, address: &str) -> Option<&L2Account> {
        self.accounts.get(address)
    }

    pub fn update_account(&mut self, account: L2Account) {
        self.accounts.insert(account.address.clone(), account);
        self.update_state_root();
    }

    pub fn update_state_root(&mut self) {
        // Create a simple state root based on account data
        let mut hasher = Keccak256::new();

        // Sort accounts by address for consistent hashing
        let mut accounts: Vec<(&String, &L2Account)> = self.accounts.iter().collect();
        accounts.sort_by(|a, b| a.0.cmp(b.0));

        for (_, account) in accounts {
            hasher.update(account.address.as_bytes());
            hasher.update(&account.balance.to_le_bytes());
            hasher.update(&account.nonce.to_le_bytes());
        }

        let result = hasher.finalize();
        self.state_root = format!("{:x}", result);
    }

    pub fn get_state_root(&self) -> &str {
        &self.state_root
    }
}

// Batch structure for transaction batching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct L2Batch {
    pub batch_id: String,
    pub transactions: Vec<L2Transaction>,
    pub state_root_before: String,
    pub state_root_after: String,
    pub timestamp: NaiveDateTime,
    pub operator_signature: String,
}

// Main rollup structure
pub struct L2Rollup {
    pub config: RollupConfig,
    pub state_manager: RollupStateManager,
    pub blocks: Vec<L2Block>,
    pub pending_transactions: Vec<L2Transaction>,
    pub batches: Vec<L2Batch>,
    pub latest_block_number: u64,
    pub latest_batch_id: u64,
}

impl L2Rollup {
    pub fn new(config: RollupConfig) -> Self {
        Self {
            config,
            state_manager: RollupStateManager::new(),
            blocks: Vec::new(),
            pending_transactions: Vec::new(),
            batches: Vec::new(),
            latest_block_number: 0,
            latest_batch_id: 0,
        }
    }

    /// Add a transaction to the pending queue
    pub fn add_transaction(&mut self, transaction: L2Transaction) -> Result<(), RollupError> {
        // Validate transaction
        if transaction.amount <= 0.0 {
            return Err(RollupError::InvalidTransaction);
        }

        // Check sender has sufficient balance
        if let Some(account) = self.state_manager.get_account(&transaction.from) {
            if account.balance < transaction.amount {
                return Err(RollupError::InsufficientBalance);
            }
        } else {
            // New account, check if it's trying to send tokens
            if transaction.amount > 0.0 {
                return Err(RollupError::InsufficientBalance);
            }
        }

        self.pending_transactions.push(transaction);
        Ok(())
    }

    /// Create a new block from pending transactions
    pub fn create_block(&mut self) -> Result<L2Block, RollupError> {
        if self.pending_transactions.is_empty() {
            return Err(RollupError::InvalidBlock);
        }

        let state_root_before = self.state_manager.get_state_root().to_string();

        // Process transactions and update state
        let transactions = self.pending_transactions.clone();
        self.pending_transactions.clear();
        for tx in &transactions {
            self.process_transaction(tx)?;
        }

        let state_root_after = self.state_manager.get_state_root().to_string();

        self.latest_block_number += 1;

        let block = L2Block {
            block_number: self.latest_block_number,
            transactions: transactions.clone(),
            state_root: state_root_after.clone(),
            previous_block_hash: self.get_latest_block_hash(),
            timestamp: Utc::now().naive_utc(),
            batch_id: None,
        };

        self.blocks.push(block.clone());
        self.pending_transactions.clear();

        Ok(block)
    }

    /// Process a single transaction and update state
    fn process_transaction(&mut self, transaction: &L2Transaction) -> Result<(), RollupError> {
        // Deduct from sender
        if let Some(mut sender_account) = self.state_manager.get_account(&transaction.from).cloned()
        {
            if sender_account.balance < transaction.amount {
                return Err(RollupError::InsufficientBalance);
            }
            sender_account.balance -= transaction.amount;
            sender_account.nonce += 1;
            self.state_manager.update_account(sender_account);
        } else {
            return Err(RollupError::InsufficientBalance);
        }

        // Add to receiver
        let receiver_balance =
            if let Some(receiver_account) = self.state_manager.get_account(&transaction.to) {
                receiver_account.balance
            } else {
                0.0
            };

        let receiver_account = L2Account {
            address: transaction.to.clone(),
            balance: receiver_balance + transaction.amount,
            nonce: if let Some(receiver_account) = self.state_manager.get_account(&transaction.to) {
                receiver_account.nonce
            } else {
                0
            },
        };

        self.state_manager.update_account(receiver_account);

        Ok(())
    }

    /// Submit a batch of transactions to L1
    pub fn submit_batch(&mut self) -> Result<L2Batch, RollupError> {
        if self.pending_transactions.is_empty() {
            return Err(RollupError::BatchSubmissionFailed);
        }

        // Process all pending transactions
        let state_root_before = self.state_manager.get_state_root().to_string();

        let transactions = self.pending_transactions.clone();
        self.pending_transactions.clear();
        for tx in &transactions {
            self.process_transaction(tx)?;
        }

        let state_root_after = self.state_manager.get_state_root().to_string();

        self.latest_batch_id += 1;
        let batch_id = format!("batch_{}", self.latest_batch_id);

        let batch = L2Batch {
            batch_id: batch_id.clone(),
            transactions: transactions.clone(),
            state_root_before,
            state_root_after: state_root_after.clone(),
            timestamp: Utc::now().naive_utc(),
            operator_signature: String::new(), // In a real implementation, this would be a cryptographic signature
        };

        // Update all pending transactions with batch ID
        for block in self.blocks.iter_mut() {
            if block.batch_id.is_none() {
                block.batch_id = Some(batch_id.clone());
            }
        }

        self.batches.push(batch.clone());
        self.pending_transactions.clear();

        Ok(batch)
    }

    /// Get the hash of the latest block
    fn get_latest_block_hash(&self) -> String {
        if let Some(latest_block) = self.blocks.last() {
            let mut hasher = Keccak256::new();
            hasher.update(latest_block.block_number.to_le_bytes());
            hasher.update(latest_block.state_root.as_bytes());
            hasher.update(latest_block.previous_block_hash.as_bytes());
            let result = hasher.finalize();
            format!("{:x}", result)
        } else {
            "genesis".to_string()
        }
    }

    /// Get current state root
    pub fn get_state_root(&self) -> &str {
        self.state_manager.get_state_root()
    }

    /// Get account balance
    pub fn get_balance(&self, address: &str) -> f64 {
        if let Some(account) = self.state_manager.get_account(address) {
            account.balance
        } else {
            0.0
        }
    }

    /// Initialize an account with balance
    pub fn initialize_account(&mut self, address: String, balance: f64) {
        let account = L2Account {
            address,
            balance,
            nonce: 0,
        };
        self.state_manager.update_account(account);
    }
}
