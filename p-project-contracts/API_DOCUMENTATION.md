# P-Project API Documentation

This document provides comprehensive API documentation for all new features implemented in the P-Project contracts.

## Table of Contents
1. [Liquidity Pools API](#liquidity-pools-api)
2. [L2 Scaling Solutions API](#l2-scaling-solutions-api)
3. [Advanced Cryptography API](#advanced-cryptography-api)
4. [Formal Verification API](#formal-verification-api)

## Liquidity Pools API

### LiquidityPoolConfig
Configuration structure for liquidity pools.

```rust
pub struct LiquidityPoolConfig {
    pub pool_id: String,
    pub token_a: String,
    pub token_b: String,
    pub fee_tier: f64,
    pub start_date: NaiveDateTime,
    pub reward_token: String,
    pub total_reward_allocation: f64,
    pub distributed_rewards: f64,
    pub apr_rate: f64,
}
```

**Fields:**
- `pool_id`: Unique identifier for the pool
- `token_a`: First token in the pair
- `token_b`: Second token in the pair
- `fee_tier`: Fee tier as percentage (e.g., 0.003 for 0.3%)
- `start_date`: Pool creation date
- `reward_token`: Token used for rewards
- `total_reward_allocation`: Total rewards allocated to this pool
- `distributed_rewards`: Rewards already distributed
- `apr_rate`: Annual percentage rate for yield farming

### LiquidityPosition
Represents a liquidity provider's position in a pool.

```rust
pub struct LiquidityPosition {
    pub user_id: String,
    pub pool_id: String,
    pub liquidity_amount: f64,
    pub token_a_amount: f64,
    pub token_b_amount: f64,
    pub start_time: NaiveDateTime,
    pub duration_days: i64,
    pub accumulated_rewards: f64,
    pub last_reward_time: NaiveDateTime,
    pub claimed_rewards: f64,
}
```

### LiquidityPool
Main liquidity pool structure.

```rust
pub struct LiquidityPool {
    pub config: LiquidityPoolConfig,
    pub total_liquidity: f64,
    pub total_token_a: f64,
    pub total_token_b: f64,
    pub liquidity_positions: HashMap<String, LiquidityPosition>,
    pub k_constant: f64,
    pub total_volume: f64,
    pub total_fees: f64,
}
```

### LiquidityPool Methods

#### new
Create a new liquidity pool.

```rust
pub fn new(
    pool_id: String,
    token_a: String,
    token_b: String,
    fee_tier: f64,
    reward_token: String,
    total_reward_allocation: f64,
    apr_rate: f64,
) -> Self
```

#### add_liquidity
Add liquidity to the pool.

```rust
pub fn add_liquidity(
    &mut self,
    user_id: String,
    token_a_amount: f64,
    token_b_amount: f64,
    duration_days: i64,
) -> Result<f64, LiquidityPoolError>
```

#### remove_liquidity
Remove liquidity from the pool.

```rust
pub fn remove_liquidity(&mut self, user_id: &str) -> Result<(f64, f64), LiquidityPoolError>
```

#### swap
Execute a token swap.

```rust
pub fn swap(&mut self, input_token: &str, input_amount: f64) -> Result<f64, LiquidityPoolError>
```

#### claim_rewards
Claim accumulated rewards.

```rust
pub fn claim_rewards(&mut self, user_id: &str) -> Result<f64, LiquidityPoolError>
```

## L2 Scaling Solutions API

### L2Transaction
Represents an L2 transaction.

```rust
pub struct L2Transaction {
    pub from: String,
    pub to: String,
    pub amount: f64,
    pub nonce: u64,
    pub signature: String,
    pub timestamp: NaiveDateTime,
}
```

### L2Block
Represents an L2 block.

```rust
pub struct L2Block {
    pub block_number: u64,
    pub transactions: Vec<L2Transaction>,
    pub state_root: String,
    pub previous_block_hash: String,
    pub timestamp: NaiveDateTime,
    pub batch_id: Option<String>,
}
```

### L2Account
Represents an L2 account.

```rust
pub struct L2Account {
    pub address: String,
    pub balance: f64,
    pub nonce: u64,
}
```

### RollupConfig
Configuration for the L2 rollup.

```rust
pub struct RollupConfig {
    pub chain_id: String,
    pub operator_address: String,
    pub batch_submission_interval: u64,
    pub max_batch_size: usize,
    pub gas_price: f64,
}
```

### L2Rollup
Main L2 rollup structure.

```rust
pub struct L2Rollup {
    pub config: RollupConfig,
    pub state_manager: RollupStateManager,
    pub blocks: Vec<L2Block>,
    pub pending_transactions: Vec<L2Transaction>,
    pub batches: Vec<L2Batch>,
    pub latest_block_number: u64,
    pub latest_batch_id: u64,
}
```

### L2Rollup Methods

#### new
Create a new L2 rollup.

```rust
pub fn new(config: RollupConfig) -> Self
```

#### add_transaction
Add a transaction to the pending queue.

```rust
pub fn add_transaction(&mut self, transaction: L2Transaction) -> Result<(), RollupError>
```

#### initialize_account
Initialize an account with balance.

```rust
pub fn initialize_account(&mut self, address: String, balance: f64)
```

### CrossChainMessage
Represents a cross-chain message.

```rust
pub struct CrossChainMessage {
    pub message_id: String,
    pub source_chain: String,
    pub destination_chain: String,
    pub sender: String,
    pub recipient: String,
    pub amount: f64,
    pub token: String,
    pub payload: Vec<u8>,
    pub timestamp: u64,
    pub signature: String,
}
```

### L2CrossChainProtocol
Cross-chain communication protocol.

```rust
pub struct L2CrossChainProtocol {
    pub rollup: L2Rollup,
    pub bridge_state: CrossChainBridgeState,
    pub chain_id: String,
    pub connected_chains: Vec<String>,
}
```

### L2CrossChainProtocol Methods

#### new
Create a new cross-chain protocol instance.

```rust
pub fn new(rollup: L2Rollup, chain_id: String) -> Self
```

#### add_connected_chain
Add a connected chain.

```rust
pub fn add_connected_chain(&mut self, chain_id: String)
```

#### create_cross_chain_message
Create a cross-chain message.

```rust
pub fn create_cross_chain_message(
    &mut self,
    source_chain: String,
    destination_chain: String,
    sender: String,
    recipient: String,
    amount: f64,
    token: String,
    payload: Vec<u8>,
) -> Result<CrossChainMessage, RollupError>
```

## Advanced Cryptography API

### Post-Quantum Cryptography

#### PQKeyPair
Post-quantum keypair structure.

```rust
pub struct PQKeyPair {
    pub public_key: Vec<u8>,
    pub private_key: Vec<u8>,
}
```

#### PQEncryptedData
Encrypted data structure.

```rust
pub struct PQEncryptedData {
    pub ciphertext: Vec<u8>,
    pub shared_secret: Vec<u8>,
}
```

#### generate_keypair
Generate a post-quantum keypair.

```rust
pub fn generate_keypair() -> Result<PQKeyPair, Box<dyn std::error::Error>>
```

#### encrypt
Encrypt data using post-quantum cryptography.

```rust
pub fn encrypt(_public_key: &[u8], data: &[u8]) -> Result<PQEncryptedData, Box<dyn std::error::Error>>
```

#### decrypt
Decrypt data using post-quantum cryptography.

```rust
pub fn decrypt(_private_key: &[u8], encrypted_data: &PQEncryptedData) -> Result<Vec<u8>, Box<dyn std::error::Error>>
```

### Zero-Knowledge Proofs

#### ZKProof
Zero-knowledge proof structure.

```rust
pub struct ZKProof {
    pub proof_data: Vec<u8>,
    pub public_inputs: Vec<u8>,
}
```

#### ZKProofSystem
ZK proof system structure.

```rust
pub struct ZKProofSystem {
    pub system_parameters: Vec<u8>,
}
```

#### new_proof_system
Create a new ZK proof system.

```rust
pub fn new_proof_system() -> Result<ZKProofSystem, Box<dyn std::error::Error>>
```

#### generate_proof
Generate a zero-knowledge proof.

```rust
pub fn generate_proof(witness: &[u8], public_inputs: &[u8]) -> Result<ZKProof, Box<dyn std::error::Error>>
```

#### verify_proof
Verify a zero-knowledge proof.

```rust
pub fn verify_proof(_proof: &ZKProof, _proof_system: &ZKProofSystem) -> Result<bool, Box<dyn std::error::Error>>
```

### Threshold Signatures

#### ThresholdSignatureScheme
Threshold signature scheme structure.

```rust
pub struct ThresholdSignatureScheme {
    pub threshold: usize,
    pub total_parties: usize,
}
```

#### Participant
Participant in a threshold signature scheme.

```rust
pub struct Participant {
    pub id: usize,
    pub public_key: Vec<u8>,
    pub private_key_share: Vec<u8>,
}
```

#### ThresholdSignature
Threshold signature structure.

```rust
pub struct ThresholdSignature {
    pub signature_data: Vec<u8>,
    pub participants: Vec<usize>,
}
```

#### new_scheme
Create a new threshold signature scheme.

```rust
pub fn new_scheme(threshold: usize, total_parties: usize) -> ThresholdSignatureScheme
```

#### generate_key_shares
Generate key shares for participants.

```rust
pub fn generate_key_shares(scheme: &ThresholdSignatureScheme) -> Result<Vec<Participant>, Box<dyn std::error::Error>>
```

#### generate_partial_signature
Generate a partial signature.

```rust
pub fn generate_partial_signature(participant: &Participant, message: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>>
```

#### combine_signatures
Combine partial signatures into a threshold signature.

```rust
pub fn combine_signatures(partial_signatures: &[Vec<u8>], participants: Vec<usize>) -> Result<ThresholdSignature, Box<dyn std::error::Error>>
```

## Formal Verification API

### Verification Modules

The formal verification system includes several modules:

1. **formal_verification.rs** - Basic verification harnesses
2. **l2_model_checking.rs** - Model checking specifications for L2
3. **theorem_proving.rs** - Mathematical proofs for critical functions
4. **comprehensive_verification.rs** - Additional verification harnesses

### Verification Properties

#### Liquidity Pool Properties
- Constant Product Invariant: k = x * y is maintained after swaps
- Balance Non-Negativity: Account balances never go negative
- Fee Calculation Accuracy: Fees are calculated correctly
- Liquidity Provision Safety: Only positive amounts can be added

#### L2 Rollup Properties
- State Consistency: State root correctly represents account state
- Transaction Safety: Transactions are processed correctly
- Cross-Chain Message Integrity: Messages are processed correctly
- Batch Submission Correctness: Batches are submitted correctly

#### General Properties
- Memory Safety: No undefined behavior or memory errors
- Panic Freedom: No unexpected panics in normal operation
- Arithmetic Safety: No overflows or underflows
- Specification Compliance: Code adheres to formal specifications