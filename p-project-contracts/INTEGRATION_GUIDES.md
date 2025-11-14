# P-Project Integration Guides for Third-Party Developers

This document provides comprehensive integration guides for third-party developers who want to integrate with P-Project contracts.

## Table of Contents
1. [Integration Overview](#integration-overview)
2. [Liquidity Pool Integration](#liquidity-pool-integration)
3. [L2 Scaling Integration](#l2-scaling-integration)
4. [Cross-Chain Integration](#cross-chain-integration)
5. [Advanced Cryptography Integration](#advanced-cryptography-integration)
6. [API Integration Patterns](#api-integration-patterns)
7. [Security Considerations](#security-considerations)
8. [Troubleshooting](#troubleshooting)

## Integration Overview

### Supported Integration Methods
1. **Direct Smart Contract Interaction** - Interact directly with P-Project contracts
2. **SDK Integration** - Use official SDKs for simplified integration
3. **API Services** - Use REST/gRPC APIs for off-chain integration
4. **Subgraph Integration** - Query indexed data using The Graph

### Prerequisites
- Understanding of Rust and smart contract development
- Basic knowledge of DeFi concepts
- Familiarity with blockchain networks (Ethereum, Polygon, etc.)
- Access to a compatible blockchain node or RPC provider

### Environment Setup
```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install P-Project CLI tools
cargo install p-project-cli

# Set up development environment
p-project init my-integration-project
cd my-integration-project
```

## Liquidity Pool Integration

### Basic Pool Interaction
```rust
use p_project_contracts::liquidity_pool::{LiquidityPool, LiquidityPoolConfig};

// Connect to an existing pool
let pool_address = "0x1234..."; // Contract address
let pool = LiquidityPool::at(pool_address);

// Check pool status
println!("Pool ID: {}", pool.config.pool_id);
println!("Token A: {}", pool.config.token_a);
println!("Token B: {}", pool.config.token_b);
println!("Fee Tier: {}%", pool.config.fee_tier * 100.0);

// Get current reserves
let (token_a_reserve, token_b_reserve) = pool.get_reserves();
println!("Reserves: {} {} / {} {}", token_a_reserve, pool.config.token_a, token_b_reserve, pool.config.token_b);
```

### Providing Liquidity
```rust
// Calculate optimal liquidity amounts
let desired_token_a = 1000.0;
let token_b_amount = pool.calculate_optimal_token_b_amount(desired_token_a);

// Approve token transfers
token_a_contract.approve(pool_address, desired_token_a).await?;
token_b_contract.approve(pool_address, token_b_amount).await?;

// Add liquidity
let user_id = "your_user_id";
let duration_days = 30;
let liquidity_amount = pool.add_liquidity(
    user_id.to_string(),
    desired_token_a,
    token_b_amount,
    duration_days,
).await?;

println!("Added {} liquidity tokens", liquidity_amount);
```

### Performing Swaps
```rust
// Calculate output amount before swap
let input_token = "TOKEN_A";
let input_amount = 100.0;
let expected_output = pool.calculate_swap_output(input_token, input_amount)?;

println!("Expected output: {} TOKEN_B", expected_output);

// Approve input token transfer
token_a_contract.approve(pool_address, input_amount).await?;

// Execute swap
let actual_output = pool.swap(input_token, input_amount).await?;

println!("Swapped {} TOKEN_A for {} TOKEN_B", input_amount, actual_output);
```

### Building a Trading Interface
```rust
pub struct TradingInterface {
    pool: LiquidityPool,
    slippage_tolerance: f64, // e.g., 0.005 for 0.5%
}

impl TradingInterface {
    pub fn new(pool: LiquidityPool, slippage_tolerance: f64) -> Self {
        Self { pool, slippage_tolerance }
    }
    
    pub async fn safe_swap(&self, input_token: &str, input_amount: f64) -> Result<f64, Box<dyn std::error::Error>> {
        // Calculate expected output
        let expected_output = self.pool.calculate_swap_output(input_token, input_amount)?;
        
        // Calculate minimum output with slippage tolerance
        let minimum_output = expected_output * (1.0 - self.slippage_tolerance);
        
        // Check pool has sufficient liquidity
        if expected_output > self.get_max_output() {
            return Err("Insufficient liquidity".into());
        }
        
        // Execute swap with slippage protection
        let actual_output = self.pool.swap_with_slippage_protection(
            input_token,
            input_amount,
            minimum_output,
        ).await?;
        
        Ok(actual_output)
    }
    
    fn get_max_output(&self) -> f64 {
        // Implement logic to determine maximum safe output
        let (_, token_b_reserve) = self.pool.get_reserves();
        token_b_reserve * 0.1 // Max 10% of reserve
    }
}
```

### Yield Farming Integration
```rust
// Monitor rewards
let user_id = "your_user_id";
let current_rewards = pool.get_user_rewards(user_id)?;
println!("Current rewards: {} {}", current_rewards, pool.config.reward_token);

// Claim rewards
let claimed_amount = pool.claim_rewards(user_id).await?;
println!("Claimed {} rewards", claimed_amount);

// Calculate projected yields
let liquidity_amount = 10000.0;
let projected_30_days = pool.calculate_projected_yield(liquidity_amount, 30.0);
let projected_1_year = pool.calculate_projected_yield(liquidity_amount, 365.0);

println!("Projected yield (30 days): {} {}", projected_30_days, pool.config.reward_token);
println!("Projected yield (1 year): {} {}", projected_1_year, pool.config.reward_token);
```

## L2 Scaling Integration

### Connecting to L2 Network
```rust
use p_project_contracts::l2_rollup::{L2Rollup, RollupConfig};

// Connect to L2 network
let rollup_rpc_url = "https://l2.p-project.network";
let rollup = L2Rollup::connect(rollup_rpc_url).await?;

// Get network information
println!("Chain ID: {}", rollup.config.chain_id);
println!("Latest block: {}", rollup.latest_block_number);
println!("Gas price: {}", rollup.config.gas_price);
```

### Submitting Transactions
```rust
use p_project_contracts::l2_rollup::L2Transaction;

// Create and sign transaction
let transaction = L2Transaction {
    from: "your_address".to_string(),
    to: "recipient_address".to_string(),
    amount: 100.0,
    nonce: rollup.get_account_nonce("your_address").await?,
    signature: sign_transaction(&transaction_data, &private_key)?,
    timestamp: chrono::Utc::now().naive_utc(),
};

// Submit transaction
let tx_hash = rollup.submit_transaction(transaction).await?;
println!("Transaction submitted: {}", tx_hash);

// Wait for confirmation
let receipt = rollup.wait_for_transaction_receipt(&tx_hash).await?;
if receipt.success {
    println!("Transaction confirmed in block {}", receipt.block_number);
} else {
    println!("Transaction failed: {}", receipt.error_message.unwrap_or("Unknown error".to_string()));
}
```

### Batch Transaction Processing
```rust
// Create batch processor
let mut batch_processor = BatchProcessor::new(rollup);

// Add multiple transactions to batch
for transfer in transfers {
    let tx = L2Transaction {
        from: transfer.from.clone(),
        to: transfer.to.clone(),
        amount: transfer.amount,
        nonce: rollup.get_account_nonce(&transfer.from).await?,
        signature: sign_transaction(&transfer, &private_keys[&transfer.from])?,
        timestamp: chrono::Utc::now().naive_utc(),
    };
    
    batch_processor.add_transaction(tx)?;
}

// Submit batch
let batch_result = batch_processor.submit_batch().await?;
println!("Batch submitted with {} transactions", batch_result.transactions.len());
```

### Monitoring Rollup State
```rust
// Monitor account balances
let balance = rollup.get_balance("your_address").await?;
println!("Account balance: {}", balance);

// Monitor block production
let latest_block = rollup.get_latest_block().await?;
println!("Latest block: #{}, {} transactions", latest_block.block_number, latest_block.transactions.len());

// Monitor batch submissions
let pending_batches = rollup.get_pending_batches().await?;
println!("Pending batches: {}", pending_batches.len());
```

## Cross-Chain Integration

### Setting up Cross-Chain Communication
```rust
use p_project_contracts::l2_cross_chain::L2CrossChainProtocol;

// Initialize cross-chain protocol
let mut cross_chain = L2CrossChainProtocol::new(rollup, "p-project-l2".to_string());

// Connect to destination chains
cross_chain.add_connected_chain("ethereum".to_string());
cross_chain.add_connected_chain("polygon".to_string());
cross_chain.add_connected_chain("bsc".to_string());

println!("Connected to {} chains", cross_chain.connected_chains.len());
```

### Sending Cross-Chain Messages
```rust
// Prepare cross-chain transfer
let transfer_data = CrossChainTransfer {
    source_chain: "p-project-l2".to_string(),
    destination_chain: "ethereum".to_string(),
    sender: "your_address".to_string(),
    recipient: "ethereum_address".to_string(),
    amount: 1000.0,
    token: "USDC".to_string(),
    memo: "Cross-chain transfer".to_string(),
};

// Serialize transfer data
let payload = serde_json::to_vec(&transfer_data)?;

// Lock tokens on source chain
let message_id = cross_chain.lock_tokens(
    transfer_data.sender.clone(),
    transfer_data.token.clone(),
    transfer_data.amount,
)?;

// Create cross-chain message
let message = cross_chain.create_cross_chain_message(
    transfer_data.source_chain,
    transfer_data.destination_chain,
    transfer_data.sender,
    transfer_data.recipient,
    transfer_data.amount,
    transfer_data.token,
    payload,
).await?;

println!("Cross-chain message created: {}", message.message_id);
```

### Receiving Cross-Chain Messages
```rust
// Process incoming messages
impl CrossChainHandler {
    pub async fn handle_incoming_message(&mut self, message: CrossChainMessage) -> Result<(), Box<dyn std::error::Error>> {
        // Verify message integrity
        if !self.cross_chain.verify_message(&message) {
            return Err("Invalid message signature".into());
        }
        
        // Check if message was already processed
        if self.is_message_processed(&message.message_id) {
            return Err("Message already processed".into());
        }
        
        // Parse payload
        let transfer_data: CrossChainTransfer = serde_json::from_slice(&message.payload)?;
        
        // Validate transfer
        if !self.validate_transfer(&transfer_data) {
            return Err("Invalid transfer data".into());
        }
        
        // Mint tokens on destination chain
        self.mint_tokens(&transfer_data.recipient, &transfer_data.token, transfer_data.amount).await?;
        
        // Mark message as processed
        self.mark_message_processed(&message.message_id);
        
        // Emit event
        self.emit_transfer_completed_event(&message.message_id, &transfer_data);
        
        Ok(())
    }
}
```

### Building a Cross-Chain Bridge Interface
```rust
pub struct CrossChainBridge {
    protocol: L2CrossChainProtocol,
    fee_collector: FeeCollector,
}

impl CrossChainBridge {
    pub async fn transfer_tokens(
        &mut self,
        source_chain: &str,
        destination_chain: &str,
        amount: f64,
        token: &str,
        recipient: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // Calculate fees
        let fee = self.calculate_transfer_fee(amount, source_chain, destination_chain);
        let total_amount = amount + fee;
        
        // Check user balance
        let user_balance = self.protocol.rollup.get_balance("user_address").await?;
        if user_balance < total_amount {
            return Err("Insufficient balance".into());
        }
        
        // Lock tokens
        let message_id = self.protocol.lock_tokens("user_address".to_string(), token.to_string(), total_amount)?;
        
        // Create message
        let payload = self.create_transfer_payload(amount, token, recipient)?;
        let message = self.protocol.create_cross_chain_message(
            source_chain.to_string(),
            destination_chain.to_string(),
            "user_address".to_string(),
            recipient.to_string(),
            amount,
            token.to_string(),
            payload,
        ).await?;
        
        // Collect fees
        self.fee_collector.collect_fee(fee, token).await?;
        
        Ok(message.message_id)
    }
    
    fn calculate_transfer_fee(&self, amount: f64, source: &str, destination: &str) -> f64 {
        // Implement fee calculation logic
        let base_fee = 0.1;
        let chain_multiplier = self.get_chain_fee_multiplier(source, destination);
        let amount_percentage = amount * 0.001; // 0.1%
        
        base_fee + (chain_multiplier * amount_percentage)
    }
}
```

## Advanced Cryptography Integration

### Post-Quantum Cryptography Integration
```rust
use p_project_contracts::advanced_cryptography::post_quantum;

// Generate post-quantum keypair for secure communication
let keypair = post_quantum::generate_keypair()?;

// Encrypt sensitive data
let sensitive_data = b"Confidential business information";
let encrypted = post_quantum::encrypt(&keypair.public_key, sensitive_data)?;

// Store encrypted data securely
self.storage.store_encrypted_data("document_id", &encrypted)?;

// Decrypt data when needed
let stored_encrypted = self.storage.get_encrypted_data("document_id")?;
let decrypted = post_quantum::decrypt(&keypair.private_key, &stored_encrypted)?;
```

### Zero-Knowledge Proof Integration
```rust
use p_project_contracts::advanced_cryptography::zero_knowledge;

// Create proof system for compliance verification
let proof_system = zero_knowledge::new_proof_system()?;

// Generate proof of compliance without revealing sensitive data
let compliance_data = self.get_compliance_data();
let public_inputs = self.get_public_compliance_inputs();

let proof = zero_knowledge::generate_proof(&compliance_data, &public_inputs)?;

// Submit proof to compliance service
self.compliance_service.submit_proof(proof, proof_system).await?;

// Verify proof for audit purposes
let is_valid = zero_knowledge::verify_proof(&proof, &proof_system)?;
if is_valid {
    println!("Compliance verified");
} else {
    println!("Compliance verification failed");
}
```

### Threshold Signature Integration
```rust
use p_project_contracts::advanced_cryptography::threshold_signatures;

// Set up threshold signature scheme for multi-sig wallet
let scheme = threshold_signatures::new_scheme(3, 5); // 3-of-5 multisig
let participants = threshold_signatures::generate_key_shares(&scheme)?;

// Distribute key shares to participants
for (index, participant) in participants.iter().enumerate() {
    self.key_management.distribute_key_share(index, participant)?;
}

// Coordinate multi-party signing
let message = b"Transaction to be signed";
let signature = self.threshold_coordinator.coordinate_signing(message, &participants).await?;

// Verify threshold signature
let is_valid = threshold_signatures::verify_signature(&signature, message, &scheme)?;
if is_valid {
    println!("Threshold signature verified");
}
```

### Building a Secure Communication Layer
```rust
pub struct SecureCommunicationLayer {
    pq_keypair: PQKeyPair,
    zk_proof_system: ZKProofSystem,
    threshold_scheme: ThresholdSignatureScheme,
}

impl SecureCommunicationLayer {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let pq_keypair = post_quantum::generate_keypair()?;
        let zk_proof_system = zero_knowledge::new_proof_system()?;
        let threshold_scheme = threshold_signatures::new_scheme(2, 3);
        
        Ok(Self {
            pq_keypair,
            zk_proof_system,
            threshold_scheme,
        })
    }
    
    pub fn secure_message(&self, plaintext: &[u8], recipient_pubkey: &[u8]) -> Result<SecureMessage, Box<dyn std::error::Error>> {
        // Encrypt message with post-quantum cryptography
        let encrypted = post_quantum::encrypt(recipient_pubkey, plaintext)?;
        
        // Create zero-knowledge proof of message integrity
        let witness = &encrypted.ciphertext;
        let public_inputs = b"Message integrity verified";
        let proof = zero_knowledge::generate_proof(witness, public_inputs)?;
        
        // Sign with threshold signature for authenticity
        let participants = self.get_signing_participants();
        let partial_signatures = self.generate_partial_signatures(&proof.proof_data, &participants)?;
        let threshold_signature = threshold_signatures::combine_signatures(&partial_signatures, self.get_participant_indices())?;
        
        Ok(SecureMessage {
            encrypted_data: encrypted,
            integrity_proof: proof,
            authenticity_signature: threshold_signature,
            timestamp: chrono::Utc::now().timestamp(),
        })
    }
    
    pub fn verify_secure_message(&self, message: &SecureMessage) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        // Verify threshold signature
        let is_authentic = threshold_signatures::verify_signature(
            &message.authenticity_signature,
            &message.integrity_proof.proof_data,
            &self.threshold_scheme,
        )?;
        
        if !is_authentic {
            return Err("Invalid authenticity signature".into());
        }
        
        // Verify zero-knowledge proof
        let is_valid_proof = zero_knowledge::verify_proof(
            &message.integrity_proof,
            &self.zk_proof_system,
        )?;
        
        if !is_valid_proof {
            return Err("Invalid integrity proof".into());
        }
        
        // Decrypt message
        let plaintext = post_quantum::decrypt(&self.pq_keypair.private_key, &message.encrypted_data)?;
        
        Ok(plaintext)
    }
}
```

## API Integration Patterns

### REST API Integration
```rust
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct PoolInfo {
    id: String,
    token_a: String,
    token_b: String,
    fee_tier: f64,
    tvl: f64,
    apr: f64,
}

// Fetch pool information
async fn get_pool_info(client: &Client, pool_id: &str) -> Result<PoolInfo, Box<dyn std::error::Error>> {
    let url = format!("https://api.p-project.network/v1/pools/{}", pool_id);
    let response = client.get(&url).send().await?;
    let pool_info: PoolInfo = response.json().await?;
    Ok(pool_info)
}

// Submit transaction
async fn submit_transaction(client: &Client, tx_data: &TransactionData) -> Result<String, Box<dyn std::error::Error>> {
    let url = "https://api.p-project.network/v1/transactions";
    let response = client.post(url).json(tx_data).send().await?;
    let tx_response: TransactionResponse = response.json().await?;
    Ok(tx_response.tx_hash)
}
```

### GraphQL Integration
```rust
use graphql_client::{GraphQLQuery, Response};

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/pool_data.graphql",
    response_derives = "Debug"
)]
struct PoolDataQuery;

// Query pool data
async fn query_pool_data(client: &Client, pool_id: &str) -> Result<pool_data_query::ResponseData, Box<dyn std::error::Error>> {
    let variables = pool_data_query::Variables {
        pool_id: pool_id.to_string(),
    };
    
    let request_body = PoolDataQuery::build_query(variables);
    let response = client.post("https://api.p-project.network/graphql").json(&request_body).send().await?;
    let response_body: Response<pool_data_query::ResponseData> = response.json().await?;
    
    Ok(response_body.data.unwrap())
}
```

### WebSocket Integration
```rust
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::{SinkExt, StreamExt};

// Subscribe to real-time events
async fn subscribe_to_events(pool_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!("wss://api.p-project.network/ws/pools/{}", pool_id);
    let (ws_stream, _) = connect_async(url).await?;
    let (mut write, mut read) = ws_stream.split();
    
    // Send subscription message
    let subscription = r#"{"type": "subscribe", "event": "swap"}"#;
    write.send(Message::Text(subscription.to_string())).await?;
    
    // Listen for events
    while let Some(msg) = read.next().await {
        let msg = msg?;
        if let Message::Text(text) = msg {
            let event: SwapEvent = serde_json::from_str(&text)?;
            println!("Swap event: {:?}", event);
        }
    }
    
    Ok(())
}
```

### Event-Driven Architecture
```rust
pub struct EventProcessor {
    client: Client,
    event_handlers: HashMap<String, Box<dyn EventHandler>>,
}

impl EventProcessor {
    pub async fn process_events(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let events = self.fetch_new_events().await?;
        
        for event in events {
            if let Some(handler) = self.event_handlers.get(&event.event_type) {
                handler.handle_event(&event).await?;
            }
        }
        
        Ok(())
    }
    
    async fn fetch_new_events(&self) -> Result<Vec<Event>, Box<dyn std::error::Error>> {
        let url = "https://api.p-project.network/v1/events?since={}&limit=100";
        let response = self.client.get(url).send().await?;
        let events: Vec<Event> = response.json().await?;
        Ok(events)
    }
}

// Custom event handler
pub struct SwapEventHandler {
    analytics_service: AnalyticsService,
    notification_service: NotificationService,
}

#[async_trait]
impl EventHandler for SwapEventHandler {
    async fn handle_event(&self, event: &Event) -> Result<(), Box<dyn std::error::Error>> {
        let swap_data: SwapData = serde_json::from_value(event.data.clone())?;
        
        // Update analytics
        self.analytics_service.record_swap(&swap_data).await?;
        
        // Send notifications
        self.notification_service.send_swap_notification(&swap_data).await?;
        
        Ok(())
    }
}
```

## Security Considerations

### Authentication and Authorization
```rust
// Implement proper authentication
pub struct AuthService {
    jwt_secret: String,
    session_manager: SessionManager,
}

impl AuthService {
    pub fn authenticate_user(&self, credentials: &Credentials) -> Result<AuthToken, AuthError> {
        // Verify credentials
        if !self.verify_credentials(credentials) {
            return Err(AuthError::InvalidCredentials);
        }
        
        // Generate JWT token
        let claims = Claims {
            sub: credentials.user_id.clone(),
            exp: chrono::Utc::now().timestamp() + 3600, // 1 hour expiry
            role: self.get_user_role(&credentials.user_id)?,
        };
        
        let token = jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            &claims,
            &jsonwebtoken::EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )?;
        
        Ok(AuthToken { token, expires_at: claims.exp })
    }
    
    pub fn authorize_request(&self, token: &str, required_role: &str) -> Result<bool, AuthError> {
        let decoding_key = jsonwebtoken::DecodingKey::from_secret(self.jwt_secret.as_bytes());
        let validation = jsonwebtoken::Validation::default();
        
        match jsonwebtoken::decode::<Claims>(token, &decoding_key, &validation) {
            Ok(token_data) => {
                let user_role = token_data.claims.role;
                Ok(self.check_role_permissions(&user_role, required_role))
            }
            Err(_) => Err(AuthError::InvalidToken),
        }
    }
}
```

### Input Validation
```rust
// Validate all inputs
pub struct InputValidator;

impl InputValidator {
    pub fn validate_address(address: &str) -> Result<(), ValidationError> {
        if address.is_empty() {
            return Err(ValidationError::EmptyAddress);
        }
        
        if address.len() != 42 || !address.starts_with("0x") {
            return Err(ValidationError::InvalidAddressFormat);
        }
        
        // Additional validation logic
        Ok(())
    }
    
    pub fn validate_amount(amount: f64) -> Result<(), ValidationError> {
        if amount <= 0.0 {
            return Err(ValidationError::InvalidAmount);
        }
        
        if amount.is_infinite() || amount.is_nan() {
            return Err(ValidationError::InvalidAmount);
        }
        
        if amount > 1e18 {
            return Err(ValidationError::AmountTooLarge);
        }
        
        Ok(())
    }
    
    pub fn validate_transaction(tx: &L2Transaction) -> Result<(), ValidationError> {
        self.validate_address(&tx.from)?;
        self.validate_address(&tx.to)?;
        self.validate_amount(tx.amount)?;
        
        if tx.nonce > u64::MAX - 1000 {
            return Err(ValidationError::InvalidNonce);
        }
        
        if tx.signature.is_empty() {
            return Err(ValidationError::MissingSignature);
        }
        
        Ok(())
    }
}
```

### Rate Limiting
```rust
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct RateLimiter {
    limits: HashMap<String, (usize, Duration)>, // (max_requests, window)
    requests: HashMap<String, Vec<Instant>>,
}

impl RateLimiter {
    pub fn new() -> Self {
        let mut limits = HashMap::new();
        limits.insert("default".to_string(), (100, Duration::from_secs(60))); // 100 requests per minute
        limits.insert("premium".to_string(), (1000, Duration::from_secs(60))); // 1000 requests per minute
        
        Self {
            limits,
            requests: HashMap::new(),
        }
    }
    
    pub fn check_rate_limit(&mut self, user_id: &str, tier: &str) -> Result<(), RateLimitError> {
        let (max_requests, window) = self.limits.get(tier).unwrap_or(&self.limits["default"]);
        let now = Instant::now();
        
        let user_requests = self.requests.entry(user_id.to_string()).or_insert_with(Vec::new);
        
        // Remove old requests outside the window
        user_requests.retain(|&time| now.duration_since(time) <= *window);
        
        // Check if limit exceeded
        if user_requests.len() >= *max_requests {
            let reset_time = user_requests.first().map(|&time| {
                time + *window - now
            }).unwrap_or(Duration::from_secs(0));
            
            return Err(RateLimitError::LimitExceeded { reset_time });
        }
        
        // Record this request
        user_requests.push(now);
        Ok(())
    }
}
```

### Secure Key Management
```rust
use ring::rand::SystemRandom;
use ring::aead::{AES_256_GCM, LessSafeKey, Nonce, UnboundKey};

pub struct KeyManager {
    encryption_key: LessSafeKey,
    key_rotation_interval: Duration,
    last_rotation: Instant,
}

impl KeyManager {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let rng = SystemRandom::new();
        let mut key_bytes = [0u8; 32];
        rng.fill(&mut key_bytes)?;
        
        let unbound_key = UnboundKey::new(&AES_256_GCM, &key_bytes)?;
        let encryption_key = LessSafeKey::new(unbound_key);
        
        Ok(Self {
            encryption_key,
            key_rotation_interval: Duration::from_secs(86400), // 24 hours
            last_rotation: Instant::now(),
        })
    }
    
    pub fn encrypt_data(&self, plaintext: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let nonce_bytes: [u8; 12] = rand::random();
        let nonce = Nonce::assume_unique_for_key(nonce_bytes);
        
        let mut in_out = plaintext.to_vec();
        self.encryption_key.seal_in_place_append_tag(nonce, &[], &mut in_out)?;
        
        let mut result = nonce_bytes.to_vec();
        result.append(&mut in_out);
        Ok(result)
    }
    
    pub fn decrypt_data(&self, ciphertext: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        if ciphertext.len() < 12 {
            return Err("Invalid ciphertext".into());
        }
        
        let nonce_bytes: [u8; 12] = ciphertext[..12].try_into()?;
        let nonce = Nonce::assume_unique_for_key(nonce_bytes);
        
        let mut in_out = ciphertext[12..].to_vec();
        let plaintext = self.encryption_key.open_in_place(nonce, &[], &mut in_out)?;
        Ok(plaintext.to_vec())
    }
}
```

## Troubleshooting

### Common Integration Issues

#### 1. Connection Problems
```rust
// Handle network connectivity issues
async fn resilient_api_call<T>(&self, url: &str) -> Result<T, IntegrationError>
where
    T: serde::de::DeserializeOwned,
{
    let mut attempts = 0;
    let max_attempts = 3;
    
    loop {
        match self.client.get(url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    return Ok(response.json().await?);
                } else {
                    return Err(IntegrationError::ApiError(response.status()));
                }
            }
            Err(e) => {
                attempts += 1;
                if attempts >= max_attempts {
                    return Err(IntegrationError::NetworkError(e));
                }
                
                // Exponential backoff
                let delay = Duration::from_secs(2_u64.pow(attempts));
                tokio::time::sleep(delay).await;
            }
        }
    }
}
```

#### 2. Transaction Failures
```rust
// Handle transaction failures gracefully
async fn submit_transaction_with_retry(&self, tx: &L2Transaction) -> Result<String, IntegrationError> {
    let mut attempts = 0;
    let max_attempts = 5;
    
    loop {
        match self.rollup.submit_transaction(tx).await {
            Ok(tx_hash) => return Ok(tx_hash),
            Err(e) => {
                attempts += 1;
                if attempts >= max_attempts {
                    return Err(IntegrationError::TransactionFailed(e));
                }
                
                // Check if error is retryable
                if !self.is_retryable_error(&e) {
                    return Err(IntegrationError::TransactionFailed(e));
                }
                
                // Wait before retry
                let delay = Duration::from_secs(attempts);
                tokio::time::sleep(delay).await;
            }
        }
    }
}
```

#### 3. Data Synchronization Issues
```rust
// Handle data consistency problems
async fn sync_with_retry<T, F>(&self, mut operation: F) -> Result<T, IntegrationError>
where
    F: FnMut() -> Result<T, IntegrationError>,
{
    let mut attempts = 0;
    let max_attempts = 3;
    
    loop {
        match operation() {
            Ok(result) => return Ok(result),
            Err(e) => {
                attempts += 1;
                if attempts >= max_attempts {
                    return Err(e);
                }
                
                // Refresh data and retry
                self.refresh_state().await?;
                
                // Wait before retry
                let delay = Duration::from_millis(500 * attempts);
                tokio::time::sleep(delay).await;
            }
        }
    }
}
```

### Debugging Tools

#### 1. Logging Integration
```rust
use log::{debug, info, warn, error};

pub struct IntegrationLogger {
    logger: slog::Logger,
}

impl IntegrationLogger {
    pub fn new() -> Self {
        let decorator = slog_term::TermDecorator::new().build();
        let drain = slog_term::FullFormat::new(decorator).build().fuse();
        let drain = slog_async::Async::new(drain).build().fuse();
        
        let logger = slog::Logger::root(drain, slog::o!());
        
        Self { logger }
    }
    
    pub fn log_transaction(&self, tx: &L2Transaction, status: &str) {
        info!(self.logger, "Transaction processed";
            "tx_hash" => &tx.hash,
            "from" => &tx.from,
            "to" => &tx.to,
            "amount" => tx.amount,
            "status" => status
        );
    }
    
    pub fn log_error(&self, error: &dyn std::error::Error, context: &str) {
        error!(self.logger, "Integration error";
            "error" => error.to_string(),
            "context" => context
        );
    }
}
```

#### 2. Monitoring and Metrics
```rust
use prometheus::{IntCounter, Histogram, Registry};

pub struct IntegrationMetrics {
    registry: Registry,
    transactions_processed: IntCounter,
    transaction_duration: Histogram,
    api_calls: IntCounter,
    api_errors: IntCounter,
}

impl IntegrationMetrics {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let registry = Registry::new();
        
        let transactions_processed = IntCounter::new(
            "transactions_processed_total",
            "Total number of transactions processed"
        )?;
        
        let transaction_duration = Histogram::with_opts(
            prometheus::HistogramOpts::new(
                "transaction_duration_seconds",
                "Transaction processing duration in seconds"
            )
        )?;
        
        registry.register(Box::new(transactions_processed.clone()))?;
        registry.register(Box::new(transaction_duration.clone()))?;
        
        Ok(Self {
            registry,
            transactions_processed,
            transaction_duration,
            api_calls: IntCounter::new("api_calls_total", "Total API calls")?,
            api_errors: IntCounter::new("api_errors_total", "Total API errors")?,
        })
    }
    
    pub fn record_transaction(&self, duration: f64) {
        self.transactions_processed.inc();
        self.transaction_duration.observe(duration);
    }
    
    pub fn record_api_call(&self) {
        self.api_calls.inc();
    }
    
    pub fn record_api_error(&self) {
        self.api_errors.inc();
    }
}
```

### Performance Optimization

#### 1. Caching Strategies
```rust
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct DataCache<T> {
    data: HashMap<String, (T, Instant)>,
    ttl: Duration,
}

impl<T> DataCache<T> {
    pub fn new(ttl: Duration) -> Self {
        Self {
            data: HashMap::new(),
            ttl,
        }
    }
    
    pub fn get(&self, key: &str) -> Option<&T> {
        if let Some((value, timestamp)) = self.data.get(key) {
            if Instant::now().duration_since(*timestamp) < self.ttl {
                return Some(value);
            }
        }
        None
    }
    
    pub fn set(&mut self, key: String, value: T) {
        self.data.insert(key, (value, Instant::now()));
    }
    
    pub fn cleanup(&mut self) {
        let now = Instant::now();
        self.data.retain(|_, (_, timestamp)| {
            now.duration_since(*timestamp) < self.ttl
        });
    }
}
```

#### 2. Batch Processing
```rust
pub struct BatchProcessor<T> {
    items: Vec<T>,
    max_batch_size: usize,
    batch_timeout: Duration,
    last_batch_time: Instant,
}

impl<T> BatchProcessor<T> {
    pub fn new(max_batch_size: usize, batch_timeout: Duration) -> Self {
        Self {
            items: Vec::new(),
            max_batch_size,
            batch_timeout,
            last_batch_time: Instant::now(),
        }
    }
    
    pub fn add_item(&mut self, item: T) -> Option<Vec<T>> {
        self.items.push(item);
        
        if self.items.len() >= self.max_batch_size || 
           Instant::now().duration_since(self.last_batch_time) >= self.batch_timeout {
            self.last_batch_time = Instant::now();
            Some(self.items.drain(..).collect())
        } else {
            None
        }
    }
}
```

This comprehensive integration guide provides third-party developers with all the information needed to successfully integrate with P-Project contracts, covering everything from basic interactions to advanced security considerations.