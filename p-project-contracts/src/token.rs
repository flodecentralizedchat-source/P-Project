use p_project_core::utils::generate_id;
use std::collections::HashMap;

pub struct PProjectToken {
    total_supply: f64,
    balances: HashMap<String, f64>, // user_id -> balance
    frozen_balances: HashMap<String, f64>, // user_id -> frozen balance
    burn_rate: f64, // percentage to burn on each transaction
    reward_rate: f64, // percentage to distribute to holders
    holders: Vec<String>, // list of user_ids who hold tokens
}

impl PProjectToken {
    pub fn new(total_supply: f64, burn_rate: f64, reward_rate: f64) -> Self {
        Self {
            total_supply,
            balances: HashMap::new(),
            frozen_balances: HashMap::new(),
            burn_rate,
            reward_rate,
            holders: Vec::new(),
        }
    }
    
    /// Initialize token distribution to users
    pub fn initialize_distribution(&mut self, allocations: Vec<(String, f64)>) {
        for (user_id, amount) in allocations {
            self.balances.insert(user_id.clone(), amount);
            if amount > 0.0 && !self.holders.contains(&user_id) {
                self.holders.push(user_id);
            }
        }
    }
    
    /// Transfer tokens from one user to another
    pub fn transfer(&mut self, from_user_id: &str, to_user_id: &str, amount: f64) -> Result<(), String> {
        // Check if sender has enough balance
        let sender_balance = self.balances.get(from_user_id).unwrap_or(&0.0);
        if *sender_balance < amount {
            return Err("Insufficient balance".to_string());
        }
        
        // Calculate burn amount
        let burn_amount = amount * self.burn_rate;
        let transfer_amount = amount - burn_amount;
        
        // Update balances
        let new_sender_balance = sender_balance - amount;
        self.balances.insert(from_user_id.to_string(), new_sender_balance);
        
        // Add to recipient balance
        let recipient_balance = self.balances.get(to_user_id).unwrap_or(&0.0);
        let new_recipient_balance = recipient_balance + transfer_amount;
        self.balances.insert(to_user_id.to_string(), new_recipient_balance);
        
        // Update total supply (burn)
        self.total_supply -= burn_amount;
        
        // Add recipient to holders if not already present
        if new_recipient_balance > 0.0 && !self.holders.contains(&to_user_id.to_string()) {
            self.holders.push(to_user_id.to_string());
        }
        
        // Distribute rewards to holders
        self.distribute_rewards(burn_amount * self.reward_rate);
        
        Ok(())
    }
    
    /// Distribute rewards to all holders
    fn distribute_rewards(&mut self, reward_pool: f64) {
        if self.holders.is_empty() || reward_pool <= 0.0 {
            return;
        }
        
        let reward_per_holder = reward_pool / self.holders.len() as f64;
        
        for holder_id in &self.holders {
            if let Some(balance) = self.balances.get_mut(holder_id) {
                *balance += reward_per_holder;
            }
        }
    }
    
    /// Get user balance
    pub fn get_balance(&self, user_id: &str) -> f64 {
        *self.balances.get(user_id).unwrap_or(&0.0)
    }
    
    /// Get total supply
    pub fn get_total_supply(&self) -> f64 {
        self.total_supply
    }
    
    /// Freeze user tokens (for staking or other purposes)
    pub fn freeze_tokens(&mut self, user_id: &str, amount: f64) -> Result<(), String> {
        let balance = self.balances.get(user_id).unwrap_or(&0.0);
        if *balance < amount {
            return Err("Insufficient balance to freeze".to_string());
        }
        
        // Update balances
        let new_balance = balance - amount;
        self.balances.insert(user_id.to_string(), new_balance);
        
        // Update frozen balance
        let frozen_balance = self.frozen_balances.get(user_id).unwrap_or(&0.0);
        let new_frozen_balance = frozen_balance + amount;
        self.frozen_balances.insert(user_id.to_string(), new_frozen_balance);
        
        Ok(())
    }
    
    /// Unfreeze user tokens
    pub fn unfreeze_tokens(&mut self, user_id: &str, amount: f64) -> Result<(), String> {
        let frozen_balance = self.frozen_balances.get(user_id).unwrap_or(&0.0);
        if *frozen_balance < amount {
            return Err("Insufficient frozen balance to unfreeze".to_string());
        }
        
        // Update frozen balance
        let new_frozen_balance = frozen_balance - amount;
        self.frozen_balances.insert(user_id.to_string(), new_frozen_balance);
        
        // Update available balance
        let balance = self.balances.get(user_id).unwrap_or(&0.0);
        let new_balance = balance + amount;
        self.balances.insert(user_id.to_string(), new_balance);
        
        Ok(())
    }
}