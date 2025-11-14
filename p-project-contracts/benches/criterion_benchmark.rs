// Performance benchmarks using Criterion.rs
use criterion::{black_box, criterion_group, criterion_main, Bencher, Criterion};
use p_project_contracts::advanced_cryptography::post_quantum;
use p_project_contracts::advanced_cryptography::threshold_signatures;
use p_project_contracts::advanced_cryptography::zero_knowledge;
use p_project_contracts::l2_cross_chain::L2CrossChainProtocol;
use p_project_contracts::l2_rollup::{L2Rollup, L2Transaction, RollupConfig};
use p_project_contracts::liquidity_pool::LiquidityPool;

// Benchmark post-quantum cryptography operations
fn bench_post_quantum_keypair_generation(b: &mut Bencher) {
    b.iter(|| {
        let keypair = post_quantum::generate_keypair().expect("Failed to generate keypair");
        black_box(keypair);
    });
}

fn bench_post_quantum_encryption(b: &mut Bencher) {
    let keypair = post_quantum::generate_keypair().expect("Failed to generate keypair");
    let data = b"Test data for encryption benchmark";

    b.iter(|| {
        let encrypted =
            post_quantum::encrypt(&keypair.public_key, data).expect("Failed to encrypt");
        black_box(encrypted);
    });
}

fn bench_post_quantum_decryption(b: &mut Bencher) {
    let keypair = post_quantum::generate_keypair().expect("Failed to generate keypair");
    let data = b"Test data for decryption benchmark";
    let encrypted = post_quantum::encrypt(&keypair.public_key, data).expect("Failed to encrypt");

    b.iter(|| {
        let decrypted =
            post_quantum::decrypt(&keypair.private_key, &encrypted).expect("Failed to decrypt");
        black_box(decrypted);
    });
}

// Benchmark zero-knowledge proof operations
fn bench_zk_proof_generation(b: &mut Bencher) {
    let witness = b"test witness data";
    let public_inputs = b"test public inputs";

    b.iter(|| {
        let proof = zero_knowledge::generate_proof(witness, public_inputs)
            .expect("Failed to generate proof");
        black_box(proof);
    });
}

fn bench_zk_proof_verification(b: &mut Bencher) {
    let witness = b"test witness data";
    let public_inputs = b"test public inputs";
    let proof =
        zero_knowledge::generate_proof(witness, public_inputs).expect("Failed to generate proof");
    let proof_system = zero_knowledge::new_proof_system().expect("Failed to create proof system");

    b.iter(|| {
        let is_valid =
            zero_knowledge::verify_proof(&proof, &proof_system).expect("Failed to verify proof");
        black_box(is_valid);
    });
}

// Benchmark threshold signature operations
fn bench_threshold_signature_key_generation(b: &mut Bencher) {
    let scheme = threshold_signatures::new_scheme(2, 3);

    b.iter(|| {
        let participants = threshold_signatures::generate_key_shares(&scheme)
            .expect("Failed to generate key shares");
        black_box(participants);
    });
}

fn bench_threshold_signature_partial_generation(b: &mut Bencher) {
    let scheme = threshold_signatures::new_scheme(2, 3);
    let participants =
        threshold_signatures::generate_key_shares(&scheme).expect("Failed to generate key shares");
    let message = b"Test message for threshold signature";

    b.iter(|| {
        let partial_signature =
            threshold_signatures::generate_partial_signature(&participants[0], message)
                .expect("Failed to generate partial signature");
        black_box(partial_signature);
    });
}

fn bench_threshold_signature_combination(b: &mut Bencher) {
    let scheme = threshold_signatures::new_scheme(2, 3);
    let participants =
        threshold_signatures::generate_key_shares(&scheme).expect("Failed to generate key shares");
    let message = b"Test message for threshold signature";

    let partial_signatures = vec![
        threshold_signatures::generate_partial_signature(&participants[0], message)
            .expect("Failed to generate partial signature 1"),
        threshold_signatures::generate_partial_signature(&participants[1], message)
            .expect("Failed to generate partial signature 2"),
    ];

    b.iter(|| {
        let combined_signature =
            threshold_signatures::combine_signatures(&partial_signatures, vec![0, 1])
                .expect("Failed to combine signatures");
        black_box(combined_signature);
    });
}

// Benchmark liquidity pool operations
fn bench_liquidity_pool_swap(b: &mut Bencher) {
    let mut pool = LiquidityPool::new(
        "TEST_POOL".to_string(),
        "TOKEN_A".to_string(),
        "TOKEN_B".to_string(),
        0.003,
        "REWARD_TOKEN".to_string(),
        1000000.0,
        0.1,
    );
    pool.add_liquidity("user1".to_string(), 1000000.0, 1000000.0, 30)
        .expect("Failed to add liquidity");

    b.iter(|| {
        let output = pool.swap("TOKEN_A", 1000.0).expect("Failed to swap");
        black_box(output);
    });
}

// Benchmark L2 rollup operations
fn bench_l2_transaction_processing(b: &mut Bencher) {
    let config = RollupConfig {
        chain_id: "test_chain".to_string(),
        operator_address: "operator".to_string(),
        batch_submission_interval: 300,
        max_batch_size: 100,
        gas_price: 1.0,
    };
    let mut rollup = L2Rollup::new(config);

    // Initialize accounts
    rollup.initialize_account("user1".to_string(), 1000.0);
    rollup.initialize_account("user2".to_string(), 1000.0);

    b.iter(|| {
        let tx = L2Transaction {
            from: "user1".to_string(),
            to: "user2".to_string(),
            amount: 100.0,
            nonce: 1,
            signature: "signature".to_string(),
            timestamp: chrono::Utc::now().naive_utc(),
        };
        let _result = rollup.add_transaction(tx);
        black_box(_result);
    });
}

// Benchmark cross-chain operations
fn bench_cross_chain_message_creation(b: &mut Bencher) {
    let config = RollupConfig {
        chain_id: "test_chain".to_string(),
        operator_address: "operator".to_string(),
        batch_submission_interval: 300,
        max_batch_size: 100,
        gas_price: 1.0,
    };
    let rollup = L2Rollup::new(config);
    let mut protocol = L2CrossChainProtocol::new(rollup, "chain_a".to_string());
    protocol.add_connected_chain("chain_b".to_string());
    let message_data = b"Test cross-chain message";

    b.iter(|| {
        let message = protocol
            .create_cross_chain_message(
                "chain_a".to_string(),
                "chain_b".to_string(),
                "user1".to_string(),
                "user2".to_string(),
                100.0,
                "TOKEN".to_string(),
                message_data.to_vec(),
            )
            .expect("Failed to create message");
        black_box(message);
    });
}

// Main benchmark function
fn criterion_benchmark(c: &mut Criterion) {
    // Cryptography benchmarks
    c.bench_function(
        "post_quantum_keypair_generation",
        bench_post_quantum_keypair_generation,
    );
    c.bench_function("post_quantum_encryption", bench_post_quantum_encryption);
    c.bench_function("post_quantum_decryption", bench_post_quantum_decryption);

    c.bench_function("zk_proof_generation", bench_zk_proof_generation);
    c.bench_function("zk_proof_verification", bench_zk_proof_verification);

    c.bench_function(
        "threshold_signature_key_generation",
        bench_threshold_signature_key_generation,
    );
    c.bench_function(
        "threshold_signature_partial_generation",
        bench_threshold_signature_partial_generation,
    );
    c.bench_function(
        "threshold_signature_combination",
        bench_threshold_signature_combination,
    );

    // Smart contract benchmarks
    c.bench_function("liquidity_pool_swap", bench_liquidity_pool_swap);
    c.bench_function("l2_transaction_processing", bench_l2_transaction_processing);
    c.bench_function(
        "cross_chain_message_creation",
        bench_cross_chain_message_creation,
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
