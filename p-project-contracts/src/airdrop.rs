use chrono::{NaiveDateTime, Utc};
use md5;
use p_project_core::utils::generate_id;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Custom error types for airdrop operations
#[derive(Debug, Clone, PartialEq)]
pub enum AirdropError {
    InsufficientTokens,
    UserNotEligible,
    AirdropAlreadyClaimed,
    InvalidMerkleProof,
    InvalidSignature,
    AirdropNotActive,
    EmergencyWithdrawalsDisabled,
    DatabaseError(String),
    SerializationError(String),
}

impl std::fmt::Display for AirdropError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AirdropError::InsufficientTokens => write!(f, "Not enough tokens for airdrop"),
            AirdropError::UserNotEligible => write!(f, "User not eligible for airdrop"),
            AirdropError::AirdropAlreadyClaimed => write!(f, "Airdrop already claimed"),
            AirdropError::InvalidMerkleProof => write!(f, "Invalid merkle proof"),
            AirdropError::InvalidSignature => write!(f, "Invalid signature"),
            AirdropError::AirdropNotActive => write!(f, "Airdrop is not currently active"),
            AirdropError::EmergencyWithdrawalsDisabled => {
                write!(f, "Emergency withdrawals are currently disabled")
            }
            AirdropError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            AirdropError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
        }
    }
}

impl std::error::Error for AirdropError {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AirdropContract {
    airdrop_id: String,
    total_amount: f64,
    distributed_amount: f64,
    recipients: HashMap<String, f64>,            // user_id -> amount
    claimed: HashMap<String, bool>,              // user_id -> claimed
    start_time: Option<NaiveDateTime>,           // Time-limited airdrop start
    end_time: Option<NaiveDateTime>,             // Time-limited airdrop end
    categories: HashMap<String, String>,         // user_id -> category
    merkle_proofs: HashMap<String, Vec<String>>, // user_id -> merkle proof
    referrals: HashMap<String, String>,          // user_id -> referrer_id
    paused: bool,                                // Airdrop pause status
    signatures: HashMap<String, String>,         // user_id -> claim signature
}

impl AirdropContract {
    pub fn new(total_amount: f64) -> Self {
        Self {
            airdrop_id: generate_id(),
            total_amount,
            distributed_amount: 0.0,
            recipients: HashMap::new(),
            claimed: HashMap::new(),
            start_time: None,
            end_time: None,
            categories: HashMap::new(),
            merkle_proofs: HashMap::new(),
            referrals: HashMap::new(),
            paused: false,
            signatures: HashMap::new(),
        }
    }

    /// Create a time-limited airdrop
    pub fn new_timed(
        total_amount: f64,
        start_time: NaiveDateTime,
        end_time: NaiveDateTime,
    ) -> Self {
        Self {
            airdrop_id: generate_id(),
            total_amount,
            distributed_amount: 0.0,
            recipients: HashMap::new(),
            claimed: HashMap::new(),
            start_time: Some(start_time),
            end_time: Some(end_time),
            categories: HashMap::new(),
            merkle_proofs: HashMap::new(),
            referrals: HashMap::new(),
            paused: false,
            signatures: HashMap::new(),
        }
    }

    /// Add recipients to the airdrop
    pub fn add_recipients(&mut self, recipients: Vec<(String, f64)>) -> Result<(), AirdropError> {
        self.add_recipients_with_category(recipients, None)
    }

    /// Add recipients to the airdrop with category
    pub fn add_recipients_with_category(
        &mut self,
        recipients: Vec<(String, f64)>,
        category: Option<String>,
    ) -> Result<(), AirdropError> {
        let total_new_amount: f64 = recipients.iter().map(|(_, amount)| amount).sum();

        if self.distributed_amount + total_new_amount > self.total_amount {
            return Err(AirdropError::InsufficientTokens);
        }

        for (user_id, amount) in recipients {
            self.recipients.insert(user_id.clone(), amount);
            self.claimed.insert(user_id.clone(), false);
            if let Some(ref cat) = category {
                self.categories.insert(user_id, cat.clone());
            }
            self.distributed_amount += amount;
        }

        Ok(())
    }

    /// Set merkle proof for a recipient
    pub fn set_merkle_proof(&mut self, user_id: &str, proof: Vec<String>) {
        self.merkle_proofs.insert(user_id.to_string(), proof);
    }

    /// Verify merkle proof for a recipient
    pub fn verify_merkle_proof(&self, user_id: &str, proof: &[String], root: &str) -> bool {
        if !self.recipients.contains_key(user_id) {
            return false;
        }

        // Get the leaf hash for the user
        let leaf_data = format!("{}:{}", user_id, self.recipients.get(user_id).unwrap());
        let mut current_hash = format!("{:x}", md5::compute(leaf_data));

        // Verify the proof against the root
        for proof_element in proof {
            let combined = if current_hash < *proof_element {
                format!("{}{}", current_hash, proof_element)
            } else {
                format!("{}{}", proof_element, current_hash)
            };
            current_hash = format!("{:x}", md5::compute(combined));
        }

        current_hash == root
    }

    /// Set signature for a recipient
    pub fn set_claim_signature(&mut self, user_id: &str, signature: String) {
        self.signatures.insert(user_id.to_string(), signature);
    }

    /// Verify signature for a recipient
    pub fn verify_signature(&self, user_id: &str, signature: &str) -> bool {
        if let Some(expected_signature) = self.signatures.get(user_id) {
            expected_signature == signature
        } else {
            false
        }
    }

    /// Add referral relationship
    pub fn add_referral(&mut self, user_id: &str, referrer_id: String) {
        self.referrals.insert(user_id.to_string(), referrer_id);
    }

    /// Get referrer for a user
    pub fn get_referrer(&self, user_id: &str) -> Option<&String> {
        self.referrals.get(user_id)
    }

    /// Calculate referral bonus
    pub fn calculate_referral_bonus(&self, user_id: &str, amount: f64) -> f64 {
        if self.referrals.contains_key(user_id) {
            amount * 0.05 // 5% referral bonus
        } else {
            0.0
        }
    }

    /// Pause the airdrop
    pub fn pause(&mut self) {
        self.paused = true;
    }

    /// Resume the airdrop
    pub fn resume(&mut self) {
        self.paused = false;
    }

    /// Check if airdrop is paused
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// Check if airdrop is active (for time-limited airdrops)
    pub fn is_active(&self) -> bool {
        if self.paused {
            return false;
        }

        let now = Utc::now().naive_utc();

        match (self.start_time, self.end_time) {
            (Some(start), Some(end)) => now >= start && now <= end,
            (Some(start), None) => now >= start,
            (None, Some(end)) => now <= end,
            (None, None) => true, // No time limits, always active
        }
    }

    /// Claim airdrop tokens with merkle proof and signature verification
    pub fn claim_with_verification(
        &mut self,
        user_id: &str,
        proof: Option<&[String]>,
        root: Option<&str>,
        signature: Option<&str>,
    ) -> Result<f64, AirdropError> {
        // Check if airdrop is active (for time-limited airdrops)
        if !self.is_active() {
            return Err(AirdropError::AirdropNotActive);
        }

        if !self.recipients.contains_key(user_id) {
            return Err(AirdropError::UserNotEligible);
        }

        if self.claimed.get(user_id) == Some(&true) {
            return Err(AirdropError::AirdropAlreadyClaimed);
        }

        // Verify merkle proof if provided
        if let (Some(proof), Some(root)) = (proof, root) {
            if !self.verify_merkle_proof(user_id, proof, root) {
                return Err(AirdropError::InvalidMerkleProof);
            }
        }

        // Verify signature if provided
        if let Some(signature) = signature {
            if !self.verify_signature(user_id, signature) {
                return Err(AirdropError::InvalidSignature);
            }
        }

        let base_amount = *self.recipients.get(user_id).unwrap();
        let referral_bonus = self.calculate_referral_bonus(user_id, base_amount);
        let total_amount = base_amount + referral_bonus;

        self.claimed.insert(user_id.to_string(), true);

        // If user has a referrer, award them a bonus
        if let Some(referrer_id) = self.referrals.get(user_id) {
            if let Some(referrer_amount) = self.recipients.get_mut(referrer_id) {
                *referrer_amount += referral_bonus;
            } else {
                // Add referrer to recipients if not already there
                self.recipients.insert(referrer_id.clone(), referral_bonus);
                self.claimed.insert(referrer_id.clone(), false);
            }
        }

        Ok(total_amount)
    }

    /// Claim airdrop tokens (simplified version for backward compatibility)
    pub fn claim(&mut self, user_id: &str) -> Result<f64, AirdropError> {
        self.claim_with_verification(user_id, None, None, None)
    }

    /// Batch claim airdrops for multiple users
    pub fn batch_claim(
        &mut self,
        user_ids: Vec<String>,
    ) -> Result<Vec<(String, f64)>, AirdropError> {
        let mut claimed_amounts = Vec::new();

        for user_id in user_ids {
            match self.claim(&user_id) {
                Ok(amount) => claimed_amounts.push((user_id, amount)),
                Err(_) => continue, // Skip failed claims
            }
        }

        Ok(claimed_amounts)
    }

    /// Batch claim airdrops with verification
    pub fn batch_claim_with_verification(
        &mut self,
        claims: Vec<(String, Option<Vec<String>>, Option<String>, Option<String>)>,
    ) -> Result<Vec<(String, f64)>, AirdropError> {
        let mut claimed_amounts = Vec::new();

        for (user_id, proof, root, signature) in claims {
            match self.claim_with_verification(
                &user_id,
                proof.as_deref(),
                root.as_deref(),
                signature.as_deref(),
            ) {
                Ok(amount) => claimed_amounts.push((user_id, amount)),
                Err(_) => continue, // Skip failed claims
            }
        }

        Ok(claimed_amounts)
    }

    /// Check if user has claimed their airdrop
    pub fn is_claimed(&self, user_id: &str) -> bool {
        *self.claimed.get(user_id).unwrap_or(&false)
    }

    /// Get user's category
    pub fn get_user_category(&self, user_id: &str) -> Option<&String> {
        self.categories.get(user_id)
    }

    /// Get airdrop status
    pub fn get_status(&self) -> AirdropStatus {
        AirdropStatus {
            airdrop_id: self.airdrop_id.clone(),
            total_amount: self.total_amount,
            distributed_amount: self.distributed_amount,
            total_recipients: self.recipients.len(),
            claimed_recipients: self.claimed.values().filter(|&&claimed| claimed).count(),
            is_paused: self.paused,
        }
    }

    /// Get airdrop ID
    pub fn get_airdrop_id(&self) -> &str {
        &self.airdrop_id
    }

    /// Get start time
    pub fn get_start_time(&self) -> Option<NaiveDateTime> {
        self.start_time
    }

    /// Get end time
    pub fn get_end_time(&self) -> Option<NaiveDateTime> {
        self.end_time
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AirdropStatus {
    pub airdrop_id: String,
    pub total_amount: f64,
    pub distributed_amount: f64,
    pub total_recipients: usize,
    pub claimed_recipients: usize,
    pub is_paused: bool,
}

// Merkle tree implementation for airdrop verification
pub struct MerkleTree {
    leaves: Vec<String>,
    layers: Vec<Vec<String>>,
    root: String,
}

impl MerkleTree {
    pub fn new(leaves: Vec<String>) -> Self {
        let mut tree = MerkleTree {
            leaves,
            layers: Vec::new(),
            root: String::new(),
        };
        tree.build_tree();
        tree
    }

    fn build_tree(&mut self) {
        if self.leaves.is_empty() {
            return;
        }

        // Start with the leaves as the first layer
        let mut current_layer = self.leaves.clone();
        self.layers.push(current_layer.clone());

        // Build each layer until we have a single root
        while current_layer.len() > 1 {
            let mut next_layer = Vec::new();

            // Process pairs of nodes
            for chunk in current_layer.chunks(2) {
                let left = &chunk[0];
                let right = if chunk.len() > 1 {
                    &chunk[1]
                } else {
                    left // Duplicate if odd number
                };

                // Hash the pair (simplified implementation)
                let combined = if left < right {
                    format!("{}{}", left, right)
                } else {
                    format!("{}{}", right, left)
                };
                let hash = format!("{:x}", md5::compute(combined));
                next_layer.push(hash);
            }

            self.layers.push(next_layer.clone());
            current_layer = next_layer;
        }

        // Set the root
        if let Some(last_layer) = self.layers.last() {
            if let Some(root) = last_layer.first() {
                self.root = root.clone();
            }
        }
    }

    pub fn get_root(&self) -> &str {
        &self.root
    }

    pub fn get_proof(&self, index: usize) -> Option<Vec<String>> {
        if index >= self.leaves.len() {
            return None;
        }

        let mut proof = Vec::new();
        let mut current_index = index;

        // For each layer except the root
        for layer in &self.layers[..self.layers.len() - 1] {
            let sibling_index = if current_index % 2 == 0 {
                current_index + 1
            } else {
                current_index - 1
            };

            // Add sibling if it exists
            if sibling_index < layer.len() {
                proof.push(layer[sibling_index].clone());
            }

            current_index /= 2;
        }

        Some(proof)
    }

    pub fn verify_proof(&self, leaf: &str, proof: &[String]) -> bool {
        let mut current_hash = format!("{:x}", md5::compute(leaf));

        for proof_element in proof {
            let combined = if current_hash < *proof_element {
                format!("{}{}", current_hash, proof_element)
            } else {
                format!("{}{}", proof_element, current_hash)
            };
            current_hash = format!("{:x}", md5::compute(combined));
        }

        current_hash == self.root
    }
}
