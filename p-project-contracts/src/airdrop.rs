use chrono::{NaiveDateTime, Utc};
use p_project_core::utils::generate_id;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub struct AirdropContract {
    airdrop_id: String,
    total_amount: f64,
    distributed_amount: f64,
    recipients: HashMap<String, f64>,    // user_id -> amount
    claimed: HashMap<String, bool>,      // user_id -> claimed
    start_time: Option<NaiveDateTime>,   // Time-limited airdrop start
    end_time: Option<NaiveDateTime>,     // Time-limited airdrop end
    categories: HashMap<String, String>, // user_id -> category
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
        }
    }

    /// Add recipients to the airdrop
    pub fn add_recipients(&mut self, recipients: Vec<(String, f64)>) -> Result<(), String> {
        self.add_recipients_with_category(recipients, None)
    }

    /// Add recipients to the airdrop with category
    pub fn add_recipients_with_category(
        &mut self,
        recipients: Vec<(String, f64)>,
        category: Option<String>,
    ) -> Result<(), String> {
        let total_new_amount: f64 = recipients.iter().map(|(_, amount)| amount).sum();

        if self.distributed_amount + total_new_amount > self.total_amount {
            return Err("Not enough tokens for airdrop".to_string());
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
    pub fn set_merkle_proof(&mut self, _user_id: &str, _proof: String) {
        // In a real implementation, this would store the merkle proof
        // For now, we'll just acknowledge the function exists
    }

    /// Check if airdrop is active (for time-limited airdrops)
    pub fn is_active(&self) -> bool {
        let now = Utc::now().naive_utc();

        match (self.start_time, self.end_time) {
            (Some(start), Some(end)) => now >= start && now <= end,
            (Some(start), None) => now >= start,
            (None, Some(end)) => now <= end,
            (None, None) => true, // No time limits, always active
        }
    }

    /// Claim airdrop tokens
    pub fn claim(&mut self, user_id: &str) -> Result<f64, String> {
        // Check if airdrop is active (for time-limited airdrops)
        if !self.is_active() {
            return Err("Airdrop is not currently active".to_string());
        }

        if !self.recipients.contains_key(user_id) {
            return Err("User not eligible for airdrop".to_string());
        }

        if self.claimed.get(user_id) == Some(&true) {
            return Err("Airdrop already claimed".to_string());
        }

        let amount = *self.recipients.get(user_id).unwrap();
        self.claimed.insert(user_id.to_string(), true);

        Ok(amount)
    }

    /// Batch claim airdrops for multiple users
    pub fn batch_claim(&mut self, user_ids: Vec<String>) -> Result<Vec<(String, f64)>, String> {
        let mut claimed_amounts = Vec::new();

        for user_id in user_ids {
            match self.claim(&user_id) {
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
}

// Merkle tree implementation for airdrop verification
pub struct MerkleTree {
    leaves: Vec<String>,
    layers: Vec<Vec<String>>,
}

impl MerkleTree {
    pub fn new(leaves: Vec<String>) -> Self {
        let mut tree = MerkleTree {
            leaves,
            layers: Vec::new(),
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
                let combined = format!("{}{}", left, right);
                let hash = format!("{:x}", md5::compute(combined));
                next_layer.push(hash);
            }

            self.layers.push(next_layer.clone());
            current_layer = next_layer;
        }
    }

    pub fn get_root(&self) -> Option<&String> {
        self.layers.last().and_then(|layer| layer.first())
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
}
