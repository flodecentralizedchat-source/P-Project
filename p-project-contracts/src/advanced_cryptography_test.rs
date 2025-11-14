//! Integration tests for the advanced cryptography module

#[cfg(test)]
mod tests {
    use crate::advanced_cryptography::{post_quantum, threshold_signatures, zero_knowledge};

    #[test]
    fn test_post_quantum_keypair_generation() {
        let keypair = post_quantum::generate_keypair().expect("Failed to generate keypair");
        assert_eq!(keypair.public_key.len(), 1184); // Kyber-768 public key size
        assert_eq!(keypair.private_key.len(), 2400); // Kyber-768 private key size
    }

    #[test]
    fn test_post_quantum_encryption_decryption() {
        let keypair = post_quantum::generate_keypair().expect("Failed to generate keypair");
        let data = b"Hello, post-quantum world!";
        let encrypted =
            post_quantum::encrypt(&keypair.public_key, data).expect("Failed to encrypt");
        let decrypted =
            post_quantum::decrypt(&keypair.private_key, &encrypted).expect("Failed to decrypt");
        assert_eq!(decrypted, data);
    }

    #[test]
    fn test_zk_proof_system() {
        let proof_system =
            zero_knowledge::new_proof_system().expect("Failed to create proof system");
        assert_eq!(proof_system.system_parameters.len(), 256);

        let witness = b"secret witness data";
        let public_inputs = b"public inputs";
        let proof = zero_knowledge::generate_proof(witness, public_inputs)
            .expect("Failed to generate proof");
        assert_eq!(proof.proof_data.len(), witness.len() + public_inputs.len());
        assert_eq!(proof.public_inputs, public_inputs);

        let is_valid =
            zero_knowledge::verify_proof(&proof, &proof_system).expect("Failed to verify proof");
        assert!(is_valid); // Placeholder result
    }

    #[test]
    fn test_threshold_signature_scheme() {
        let scheme = threshold_signatures::new_scheme(2, 3);
        assert_eq!(scheme.threshold, 2);
        assert_eq!(scheme.total_parties, 3);

        let participants = threshold_signatures::generate_key_shares(&scheme)
            .expect("Failed to generate key shares");
        assert_eq!(participants.len(), 3);

        let message = b"Message to sign";
        let partial_signature =
            threshold_signatures::generate_partial_signature(&participants[0], message)
                .expect("Failed to generate partial signature");
        assert_eq!(partial_signature.len(), 8 + message.len()); // 8 bytes for participant ID + message length

        let partial_signatures = vec![
            threshold_signatures::generate_partial_signature(&participants[0], message)
                .expect("Failed to generate partial signature 1"),
            threshold_signatures::generate_partial_signature(&participants[1], message)
                .expect("Failed to generate partial signature 2"),
        ];
        let combined_signature =
            threshold_signatures::combine_signatures(&partial_signatures, vec![0, 1])
                .expect("Failed to combine signatures");
        assert_eq!(
            combined_signature.signature_data.len(),
            2 * (8 + message.len())
        ); // Two signatures combined
        assert_eq!(combined_signature.participants, vec![0, 1]);

        let is_valid =
            threshold_signatures::verify_signature(&combined_signature, message, &scheme)
                .expect("Failed to verify signature");
        assert!(is_valid); // Placeholder result
    }

    #[test]
    fn test_complete_cryptography_workflow() {
        // Test a complete workflow combining all cryptographic features

        // 1. Generate post-quantum keypair
        let pq_keypair = post_quantum::generate_keypair().expect("Failed to generate PQ keypair");

        // 2. Encrypt a message with post-quantum cryptography
        let message = b"Secret message for zero-knowledge proof";
        let encrypted = post_quantum::encrypt(&pq_keypair.public_key, message)
            .expect("Failed to encrypt message");

        // 3. Create a zero-knowledge proof about the encrypted data
        let proof_system =
            zero_knowledge::new_proof_system().expect("Failed to create proof system");
        let witness = &encrypted.ciphertext;
        let public_inputs = b"public verification data";
        let proof = zero_knowledge::generate_proof(witness, public_inputs)
            .expect("Failed to generate proof");

        // 4. Verify the zero-knowledge proof
        let is_proof_valid =
            zero_knowledge::verify_proof(&proof, &proof_system).expect("Failed to verify proof");
        assert!(is_proof_valid);

        // 5. Use threshold signatures to sign the proof
        let scheme = threshold_signatures::new_scheme(2, 3);
        let participants = threshold_signatures::generate_key_shares(&scheme)
            .expect("Failed to generate key shares");

        let partial_signatures = vec![
            threshold_signatures::generate_partial_signature(&participants[0], &proof.proof_data)
                .expect("Failed to generate partial signature 1"),
            threshold_signatures::generate_partial_signature(&participants[1], &proof.proof_data)
                .expect("Failed to generate partial signature 2"),
        ];
        let threshold_signature =
            threshold_signatures::combine_signatures(&partial_signatures, vec![0, 1])
                .expect("Failed to combine signatures");

        // 6. Verify the threshold signature
        let is_signature_valid = threshold_signatures::verify_signature(
            &threshold_signature,
            &proof.proof_data,
            &scheme,
        )
        .expect("Failed to verify signature");
        assert!(is_signature_valid);

        // 7. Decrypt the original message
        let decrypted = post_quantum::decrypt(&pq_keypair.private_key, &encrypted)
            .expect("Failed to decrypt message");
        assert_eq!(decrypted, message);
    }
}
