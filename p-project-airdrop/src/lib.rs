use chrono::{NaiveDateTime, Utc};
use p_project_contracts::airdrop::AirdropContract;
use p_project_core::database::MySqlDatabase;

pub struct AirdropService {
    airdrop_contract: AirdropContract,
    db: MySqlDatabase,
}

impl AirdropService {
    pub async fn new(airdrop_contract: AirdropContract, db: MySqlDatabase) -> Result<Self, String> {
        // Save the airdrop to the database
        let airdrop_id = airdrop_contract.get_airdrop_id();
        let total_amount = airdrop_contract.get_status().total_amount;
        let start_time = airdrop_contract.get_start_time();
        let end_time = airdrop_contract.get_end_time();

        db.create_airdrop(airdrop_id, total_amount, start_time, end_time)
            .await
            .map_err(|e| format!("Failed to create airdrop in database: {}", e))?;

        Ok(Self {
            airdrop_contract,
            db,
        })
    }

    /// Add recipients to the airdrop
    pub async fn add_recipients(&mut self, recipients: Vec<(String, f64)>) -> Result<(), String> {
        self.add_recipients_with_category(recipients, None).await
    }

    /// Add recipients to the airdrop with category
    pub async fn add_recipients_with_category(
        &mut self,
        recipients: Vec<(String, f64)>,
        category: Option<String>,
    ) -> Result<(), String> {
        // Add to contract first
        let result = if let Some(ref cat) = category {
            self.airdrop_contract
                .add_recipients_with_category(recipients.clone(), Some(cat.clone()))
        } else {
            self.airdrop_contract.add_recipients(recipients.clone())
        };

        if result.is_ok() {
            // Save to database
            let airdrop_id = self.airdrop_contract.get_airdrop_id();
            let category_str = category.as_deref();
            self.db
                .add_airdrop_recipients(airdrop_id, &recipients, category_str)
                .await
                .map_err(|e| format!("Failed to add recipients to database: {}", e))?;
        }

        result
    }

    /// Claim airdrop tokens
    pub async fn claim_airdrop(&mut self, user_id: &str) -> Result<f64, String> {
        // Claim from contract first
        let result = self.airdrop_contract.claim(user_id);

        if result.is_ok() {
            // Save to database
            let airdrop_id = self.airdrop_contract.get_airdrop_id();
            self.db
                .claim_airdrop(airdrop_id, user_id)
                .await
                .map_err(|e| format!("Failed to claim airdrop in database: {}", e))?;
        }

        result
    }

    /// Batch claim airdrops for multiple users
    pub async fn batch_claim_airdrops(
        &mut self,
        user_ids: Vec<String>,
    ) -> Result<Vec<(String, f64)>, String> {
        // Batch claim from contract first
        let result = self.airdrop_contract.batch_claim(user_ids.clone());

        if result.is_ok() {
            // Save to database
            let airdrop_id = self.airdrop_contract.get_airdrop_id();
            self.db
                .batch_claim_airdrops(airdrop_id, &user_ids)
                .await
                .map_err(|e| format!("Failed to batch claim airdrops in database: {}", e))?;
        }

        result
    }

    /// Check if user has claimed their airdrop
    pub async fn is_claimed(&self, user_id: &str) -> Result<bool, String> {
        // Check in database
        let airdrop_id = self.airdrop_contract.get_airdrop_id();
        self.db
            .is_airdrop_claimed(airdrop_id, user_id)
            .await
            .map_err(|e| format!("Failed to check claim status in database: {}", e))
    }

    /// Get airdrop status
    pub async fn get_status(&self) -> Result<p_project_contracts::airdrop::AirdropStatus, String> {
        // Get from database
        let airdrop_id = self.airdrop_contract.get_airdrop_id();
        let (total_amount, distributed_amount, total_recipients, claimed_recipients) = self
            .db
            .get_airdrop_status(airdrop_id)
            .await
            .map_err(|e| format!("Failed to get airdrop status from database: {}", e))?;

        Ok(p_project_contracts::airdrop::AirdropStatus {
            airdrop_id: airdrop_id.to_string(),
            total_amount,
            distributed_amount,
            total_recipients,
            claimed_recipients,
        })
    }

    /// Get user's category
    pub fn get_user_category(&self, user_id: &str) -> Option<&String> {
        self.airdrop_contract.get_user_category(user_id)
    }
}
