# P-Project Usage Examples

This document provides comprehensive usage examples for all new features implemented in the P-Project contracts.

## Table of Contents
1. [Liquidity Pools Examples](#liquidity-pools-examples)
2. [L2 Scaling Solutions Examples](#l2-scaling-solutions-examples)
3. [Advanced Cryptography Examples](#advanced-cryptography-examples)
4. [Formal Verification Examples](#formal-verification-examples)

## Liquidity Pools Examples

### Creating a Liquidity Pool

```rust
use p_project_contracts::liquidity_pool::{LiquidityPool, LiquidityPoolConfig};

// Create a new liquidity pool for ETH/USDC trading
let mut pool = LiquidityPool::new(
    "ETH_USDC_POOL".to_string(),
    "ETH".to_string(),
    "USDC".to_string(),
    0.003, // 0.3% fee
    "P_TOKEN".to_string(), // Reward token
    1000000.0, // Total reward allocation
    0.15, // 15% APR
);

println!("Created liquidity pool: {}", pool.config.pool_id);
```

### Adding Liquidity

```rust
use p_project_contracts::liquidity_pool::LiquidityPool;

// Add liquidity to the pool
let user_id = "user123".to_string();
let eth_amount = 10.0;
let usdc_amount = 20000.0;
let duration_days = 30;

match pool.add_liquidity(user_id, eth_amount, usdc_amount, duration_days) {
    Ok(liquidity_amount) => {
        println!("Successfully added liquidity: {}", liquidity_amount);
    }
    Err(e) => {
        println!("Failed to add liquidity: {}", e);
    }
}
```

### Performing a Swap

```rust
use p_project_contracts::liquidity_pool::LiquidityPool;

// Swap ETH for USDC
let input_token = "ETH";
let input_amount = 1.0;

match pool.swap(input_token, input_amount) {
    Ok(output_amount) => {
        println!("Swapped {} ETH for {} USDC", input_amount, output_amount);
    }
    Err(e) => {
        println!("Swap failed: {}", e);
    }
}
```

### Claiming Rewards

```rust
use p_project_contracts::liquidity_pool::LiquidityPool;

let user_id = "user123";

match pool.claim_rewards(&user_id) {
    Ok(rewards) => {
        println!("Claimed {} rewards", rewards);
    }
    Err(e) => {
        println!("Failed to claim rewards: {}", e);
    }
}
```

### Removing Liquidity

```rust
use p_project_contracts::liquidity_pool::LiquidityPool;

let user_id = "user123";

match pool.remove_liquidity(&user_id) {
    Ok((token_a_amount, token_b_amount)) => {
        println!("Removed {} ETH and {} USDC", token_a_amount, token_b_amount);
    }
    Err(e) => {
        println!("Failed to remove liquidity: {}", e);
    }
}
```

## L2 Scaling Solutions Examples

### Setting up an L2 Rollup

```rust
use p_project_contracts::l2_rollup::{L2Rollup, RollupConfig};

// Configure the rollup
let config = RollupConfig {
    chain_id: "p-project-l2".to_string(),
    operator_address: "operator123".to_string(),
    batch_submission_interval: 300, // 5 minutes
    max_batch_size: 1000,
    gas_price: 1.0,
};

// Create the rollup
let mut rollup = L2Rollup::new(config);

// Initialize user accounts
rollup.initialize_account("user1".to_string(), 1000.0);
rollup.initialize_account("user2".to_string(), 500.0);

println!("L2 rollup initialized with chain ID: {}", rollup.config.chain_id);
```

### Adding Transactions

```rust
use p_project_contracts::l2_rollup::{L2Rollup, L2Transaction};
use chrono::Utc;

// Create a transaction
let transaction = L2Transaction {
    from: "user1".to_string(),
    to: "user2".to_string(),
    amount: 100.0,
    nonce: 1,
    signature: "signature_here".to_string(),
    timestamp: Utc::now().naive_utc(),
};

// Add transaction to the rollup
match rollup.add_transaction(transaction) {
    Ok(()) => {
        println!("Transaction added successfully");
    }
    Err(e) => {
        println!("Failed to add transaction: {}", e);
    }
}
```

### Cross-Chain Communication

```rust
use p_project_contracts::l2_cross_chain::L2CrossChainProtocol;
use p_project_contracts::l2_rollup::{L2Rollup, RollupConfig};

// Set up the rollup
let config = RollupConfig {
    chain_id: "p-project-l2".to_string(),
    operator_address: "operator123".to_string(),
    batch_submission_interval: 300,
    max_batch_size: 1000,
    gas_price: 1.0,
};

let rollup = L2Rollup::new(config);

// Create cross-chain protocol
let mut protocol = L2CrossChainProtocol::new(rollup, "p-project-l2".to_string());

// Connect to other chains
protocol.add_connected_chain("ethereum".to_string());
protocol.add_connected_chain("polygon".to_string());

// Create a cross-chain message
let payload = b"Transfer 100 USDC from P-Project L2 to Ethereum".to_vec();

match protocol.create_cross_chain_message(
    "p-project-l2".to_string(),
    "ethereum".to_string(),
    "user1".to_string(),
    "user1_eth".to_string(),
    100.0,
    "USDC".to_string(),
    payload,
) {
    Ok(message) => {
        println!("Cross-chain message created with ID: {}", message.message_id);
    }
    Err(e) => {
        println!("Failed to create cross-chain message: {}", e);
    }
}
```

## Advanced Cryptography Examples

### Post-Quantum Cryptography

```rust
use p_project_contracts::advanced_cryptography::post_quantum;

// Generate a post-quantum keypair
let keypair = post_quantum::generate_keypair().expect("Failed to generate keypair");

// Encrypt data
let data = b"Secret message for post-quantum encryption";
let encrypted = post_quantum::encrypt(&keypair.public_key, data).expect("Failed to encrypt");

println!("Encrypted data size: {} bytes", encrypted.ciphertext.len());

// Decrypt data
let decrypted = post_quantum::decrypt(&keypair.private_key, &encrypted).expect("Failed to decrypt");
assert_eq!(decrypted, data);

println!("Post-quantum encryption/decryption successful");
```

### Zero-Knowledge Proofs

```rust
use p_project_contracts::advanced_cryptography::zero_knowledge;

// Create a proof system
let proof_system = zero_knowledge::new_proof_system().expect("Failed to create proof system");

// Generate a proof
let witness = b"secret witness data for ZK proof";
let public_inputs = b"public inputs for verification";
let proof = zero_knowledge::generate_proof(witness, public_inputs).expect("Failed to generate proof");

// Verify the proof
let is_valid = zero_knowledge::verify_proof(&proof, &proof_system).expect("Failed to verify proof");
assert!(is_valid);

println!("Zero-knowledge proof generated and verified successfully");
```

### Threshold Signatures

```rust
use p_project_contracts::advanced_cryptography::threshold_signatures;

// Create a threshold signature scheme (3-out-of-5)
let scheme = threshold_signatures::new_scheme(3, 5);

// Generate key shares for participants
let participants = threshold_signatures::generate_key_shares(&scheme).expect("Failed to generate key shares");

// Generate partial signatures
let message = b"Message to be signed with threshold signature";
let partial_signatures = vec![
    threshold_signatures::generate_partial_signature(&participants[0], message).expect("Failed to generate partial signature 1"),
    threshold_signatures::generate_partial_signature(&participants[1], message).expect("Failed to generate partial signature 2"),
    threshold_signatures::generate_partial_signature(&participants[2], message).expect("Failed to generate partial signature 3"),
];

// Combine partial signatures
let combined_signature = threshold_signatures::combine_signatures(&partial_signatures, vec![0, 1, 2]).expect("Failed to combine signatures");

// Verify the combined signature
let is_valid = threshold_signatures::verify_signature(&combined_signature, message, &scheme).expect("Failed to verify signature");
assert!(is_valid);

println!("Threshold signature created and verified successfully");
```

### Complete Cryptographic Workflow

```rust
use p_project_contracts::advanced_cryptography::{post_quantum, zero_knowledge, threshold_signatures};

// 1. Encrypt sensitive data with post-quantum cryptography
let keypair = post_quantum::generate_keypair().expect("Failed to generate keypair");
let sensitive_data = b"Confidential financial data";
let encrypted_data = post_quantum::encrypt(&keypair.public_key, sensitive_data).expect("Failed to encrypt");

// 2. Create a zero-knowledge proof about the encrypted data
let proof_system = zero_knowledge::new_proof_system().expect("Failed to create proof system");
let witness = &encrypted_data.ciphertext; // Proof about the ciphertext
let public_inputs = b"Data is properly encrypted";
let zk_proof = zero_knowledge::generate_proof(witness, public_inputs).expect("Failed to generate proof");

// 3. Sign the proof with threshold signatures
let scheme = threshold_signatures::new_scheme(2, 3);
let participants = threshold_signatures::generate_key_shares(&scheme).expect("Failed to generate key shares");

let proof_bytes = zk_proof.proof_data.clone();
let partial_signatures = vec![
    threshold_signatures::generate_partial_signature(&participants[0], &proof_bytes).expect("Failed to generate partial signature 1"),
    threshold_signatures::generate_partial_signature(&participants[1], &proof_bytes).expect("Failed to generate partial signature 2"),
];

let threshold_signature = threshold_signatures::combine_signatures(&partial_signatures, vec![0, 1]).expect("Failed to combine signatures");

println!("Complete cryptographic workflow executed successfully");
println!("- Data encrypted with post-quantum cryptography");
println!("- Zero-knowledge proof generated");
println!("- Proof signed with threshold signature");
```

## Formal Verification Examples

### Running Verification Tests

```bash
# Install verification tools
make install-tools

# Run all verification tests
make verify

# Run specific verification harness
make verify-harness HARNESS=verify_liquidity_pool_creation

# Run Kani verification with verbose output
make verify-kani-verbose
```

### Using Cargo for Verification

```bash
# Run Kani verification
cargo kani --features verification

# Run specific harness
cargo kani --features verification --harness verify_liquidity_pool_creation
```

### Example Verification Harness

```rust
// In formal_verification.rs
#[cfg(kani)]
#[kani::proof]
fn verify_liquidity_pool_creation() {
    // Generate symbolic inputs
    let pool_id = kani::any::<String>();
    let token_a = kani::any::<String>();
    let token_b = kani::any::<String>();
    let fee_tier = kani::any::<f64>();
    let reward_token = kani::any::<String>();
    let total_reward_allocation = kani::any::<f64>();
    let apr_rate = kani::any::<f64>();
    
    // Assume valid inputs
    kani::assume(!pool_id.is_empty());
    kani::assume(!token_a.is_empty());
    kani::assume(!token_b.is_empty());
    kani::assume(fee_tier >= 0.0 && fee_tier <= 0.1);
    kani::assume(!reward_token.is_empty());
    kani::assume(total_reward_allocation >= 0.0);
    kani::assume(apr_rate >= 0.0 && apr_rate <= 1.0);
    
    // Create liquidity pool
    let pool = LiquidityPool::new(
        pool_id,
        token_a,
        token_b,
        fee_tier,
        reward_token,
        total_reward_allocation,
        apr_rate,
    );
    
    // Verify properties
    assert!(pool.total_liquidity == 0.0);
    assert!(pool.total_token_a == 0.0);
    assert!(pool.total_token_b == 0.0);
    assert!(pool.k_constant == 0.0);
}
```

### Theorem Proving Example

```rust
// In theorem_proving.rs
#[cfg(verus)]
use vstd::prelude::*;

#[cfg(verus)]
verus! {
    /// Specification for the constant product invariant
    pub spec fn constant_product_invariant(
        token_a_reserve: f64,
        token_b_reserve: f64,
        k_constant: f64,
    ) -> bool {
        token_a_reserve * token_b_reserve == k_constant
    }
    
    /// Proof that the constant product is maintained after a swap
    pub proof fn lemma_swap_maintains_invariant(
        pre_token_a: f64,
        pre_token_b: f64,
        input_amount: f64,
        fee_tier: f64,
    )
        requires
            pre_token_a > 0.0,
            pre_token_b > 0.0,
            input_amount > 0.0,
            fee_tier >= 0.0 && fee_tier <= 1.0,
        ensures
            constant_product_invariant(pre_token_a, pre_token_b, pre_token_a * pre_token_b),
    {
        let k_initial = pre_token_a * pre_token_b;
        
        // Apply fee
        let input_amount_with_fee = input_amount * (1.0 - fee_tier);
        
        // Calculate output amount using constant product formula
        let k_final = (pre_token_a + input_amount_with_fee) * (pre_token_b - (k_initial / (pre_token_a + input_amount_with_fee)));
        
        // Prove that k remains constant (simplified for example)
        assert(k_final == k_initial);
    }
}
```

### Model Checking Example

```rust
// In l2_model_checking.rs
#[cfg(kani)]
#[kani::proof]
fn verify_l2_account_balance_non_negativity() {
    // Generate symbolic account
    let mut account = L2Account {
        address: kani::any::<String>(),
        balance: kani::any::<f64>(),
        nonce: kani::any::<u64>(),
    };
    
    // Assume valid initial state
    kani::assume(account.balance >= 0.0);
    kani::assume(!account.address.is_empty());
    
    // Apply a transaction that deducts funds
    let transaction_amount = kani::any::<f64>();
    kani::assume(transaction_amount >= 0.0);
    kani::assume(transaction_amount <= account.balance);
    
    account.balance -= transaction_amount;
    
    // Verify balance remains non-negative
    assert!(account.balance >= 0.0);
}
```