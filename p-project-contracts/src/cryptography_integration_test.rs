//! Integration test for the complete advanced cryptography workflow

#[cfg(test)]
mod tests {
    use crate::advanced_cryptography::{post_quantum, zero_knowledge, threshold_signatures};
    
    #[test]
    fn test_complete_cryptography_workflow() {
        // Test a complete workflow combining all cryptographic features
        
        // 1. Generate post-quantum keypair
        let pq_keypair = post_quantum::generate_keypair().expect("Failed to generate PQ keypair");
        
        // 2. Encrypt a message with post-quantum cryptography
        let message = b"Secret message for zero-knowledge proof";
        let encrypted = post_quantum::encrypt(&pq_keypair.public_key, message).expect("Failed to encrypt message");
        
        // 3. Create a zero-knowledge proof about the encrypted data
        let proof_system = zero_knowledge::new_proof_system().expect("Failed to create proof system");
        let witness = &encrypted.ciphertext;
        let public_inputs = b"public verification data";
        let proof = zero_knowledge::generate_proof(witness, public_inputs).expect("Failed to generate proof");
        
        // 4. Verify the zero-knowledge proof
        let is_proof_valid = zero_knowledge::verify_proof(&proof, &proof_system).expect("Failed to verify proof");
        assert!(is_proof_valid);
        
        // 5. Use threshold signatures to sign the proof
        let scheme = threshold_signatures::new_scheme(2, 3);
        let participants = threshold_signatures::generate_key_shares(&scheme).expect("Failed to generate key shares");
        
        let partial_signatures = vec![
            threshold_signatures::generate_partial_signature(&participants[0], &proof.proof_data).expect("Failed to generate partial signature 1"),
            threshold_signatures::generate_partial_signature(&participants[1], &proof.proof_data).expect("Failed to generate partial signature 2"),
        ];
        let threshold_signature = threshold_signatures::combine_signatures(&partial_signatures, vec![0, 1]).expect("Failed to combine signatures");
        
        // 6. Verify the threshold signature
        let is_signature_valid = threshold_signatures::verify_signature(&threshold_signature, &proof.proof_data, &scheme).expect("Failed to verify signature");
        assert!(is_signature_valid);
        
        // 7. Decrypt the original message
        let decrypted = post_quantum::decrypt(&pq_keypair.private_key, &encrypted).expect("Failed to decrypt message");
        assert_eq!(decrypted, message);
        
        println!("Complete cryptography workflow test passed!");
    }
}