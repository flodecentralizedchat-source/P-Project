use p_project_core::{models::{Proposal, ProposalStatus}, database::mongodb::MongoDatabase};
use p_project_contracts::token::PProjectToken;
use std::collections::HashMap;

pub struct DaoGovernance {
    mongo_db: MongoDatabase,
    token_contract: PProjectToken,
    proposals: HashMap<String, Proposal>,
    votes: HashMap<String, HashMap<String, bool>>, // proposal_id -> (user_id -> approve)
}

impl DaoGovernance {
    pub fn new(mongo_db: MongoDatabase, token_contract: PProjectToken) -> Self {
        Self {
            mongo_db,
            token_contract,
            proposals: HashMap::new(),
            votes: HashMap::new(),
        }
    }
    
    /// Create a new proposal
    pub async fn create_proposal(&mut self, title: String, description: String, creator_id: String) -> Result<String, String> {
        let proposal_id = p_project_core::utils::generate_id();
        
        let proposal = Proposal {
            id: proposal_id.clone(),
            title,
            description,
            creator_id,
            created_at: chrono::Utc::now().naive_utc(),
            voting_end_time: chrono::Utc::now().naive_utc() + chrono::Duration::days(7), // 1 week voting
            status: ProposalStatus::Active,
        };
        
        // Save to MongoDB
        self.mongo_db.save_proposal(&proposal).await
            .map_err(|e| format!("Failed to save proposal: {}", e))?;
        
        self.proposals.insert(proposal_id.clone(), proposal);
        self.votes.insert(proposal_id.clone(), HashMap::new());
        
        Ok(proposal_id)
    }
    
    /// Vote on a proposal
    pub fn vote_on_proposal(&mut self, proposal_id: &str, user_id: &str, approve: bool) -> Result<(), String> {
        let proposal = self.proposals.get_mut(proposal_id).ok_or("Proposal not found")?;
        
        if matches!(proposal.status, ProposalStatus::Active) == false {
            return Err("Voting is not active for this proposal".to_string());
        }
        
        // Record the vote
        if let Some(votes) = self.votes.get_mut(proposal_id) {
            votes.insert(user_id.to_string(), approve);
        }
        
        Ok(())
    }
    
    /// Tally votes for a proposal and update status
    pub fn tally_votes(&mut self, proposal_id: &str) -> Result<(), String> {
        let proposal = self.proposals.get_mut(proposal_id).ok_or("Proposal not found")?;
        
        if matches!(proposal.status, ProposalStatus::Active) == false {
            return Err("Proposal is not active".to_string());
        }
        
        let votes = self.votes.get(proposal_id).ok_or("No votes found for proposal")?;
        
        let mut approve_votes = 0.0;
        let mut reject_votes = 0.0;
        
        // Count weighted votes based on token holdings
        for (user_id, &approve) in votes {
            let balance = self.token_contract.get_balance(user_id);
            if approve {
                approve_votes += balance;
            } else {
                reject_votes += balance;
            }
        }
        
        // Update proposal status based on vote results
        if approve_votes > reject_votes {
            proposal.status = ProposalStatus::Passed;
        } else {
            proposal.status = ProposalStatus::Rejected;
        }
        
        Ok(())
    }
    
    /// Get all active proposals
    pub async fn get_active_proposals(&self) -> Result<Vec<Proposal>, String> {
        self.mongo_db.get_active_proposals().await
            .map_err(|e| format!("Failed to get active proposals: {}", e))
    }
}