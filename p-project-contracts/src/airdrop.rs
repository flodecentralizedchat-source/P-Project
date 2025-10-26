use p_project_core::utils::generate_id;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

pub struct AirdropContract {
    airdrop_id: String,
    total_amount: f64,
    distributed_amount: f64,
    recipients: HashMap<String, f64>, // user_id -> amount
    claimed: HashMap<String, bool>,   // user_id -> claimed
}

impl AirdropContract {
    pub fn new(total_amount: f64) -> Self {
        Self {
            airdrop_id: generate_id(),
            total_amount,
            distributed_amount: 0.0,
            recipients: HashMap::new(),
            claimed: HashMap::new(),
        }
    }
    
    /// Add recipients to the airdrop
    pub fn add_recipients(&mut self, recipients: Vec<(String, f64)>) -> Result<(), String> {
        let total_new_amount: f64 = recipients.iter().map(|(_, amount)| amount).sum();
        
        if self.distributed_amount + total_new_amount > self.total_amount {
            return Err("Not enough tokens for airdrop".to_string());
        }
        
        for (user_id, amount) in recipients {
            self.recipients.insert(user_id.clone(), amount);
            self.claimed.insert(user_id, false);
            self.distributed_amount += amount;
        }
        
        Ok(())
    }
    
    /// Claim airdrop tokens
    pub fn claim(&mut self, user_id: &str) -> Result<f64, String> {
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
    
    /// Check if user has claimed their airdrop
    pub fn is_claimed(&self, user_id: &str) -> bool {
        *self.claimed.get(user_id).unwrap_or(&false)
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
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AirdropStatus {
    pub airdrop_id: String,
    pub total_amount: f64,
    pub distributed_amount: f64,
    pub total_recipients: usize,
    pub claimed_recipients: usize,
}