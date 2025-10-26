use p_project_contracts::airdrop::AirdropContract;
use p_project_core::database::MySqlDatabase;

pub struct AirdropService {
    airdrop_contract: AirdropContract,
    db: MySqlDatabase,
}

impl AirdropService {
    pub fn new(airdrop_contract: AirdropContract, db: MySqlDatabase) -> Self {
        Self {
            airdrop_contract,
            db,
        }
    }
    
    /// Add recipients to the airdrop
    pub fn add_recipients(&mut self, recipients: Vec<(String, f64)>) -> Result<(), String> {
        self.airdrop_contract.add_recipients(recipients)
    }
    
    /// Claim airdrop tokens
    pub fn claim_airdrop(&mut self, user_id: &str) -> Result<f64, String> {
        self.airdrop_contract.claim(user_id)
    }
    
    /// Check if user has claimed their airdrop
    pub fn is_claimed(&self, user_id: &str) -> bool {
        self.airdrop_contract.is_claimed(user_id)
    }
    
    /// Get airdrop status
    pub fn get_status(&self) -> p_project_contracts::airdrop::AirdropStatus {
        self.airdrop_contract.get_status()
    }
}