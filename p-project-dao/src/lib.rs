// P-Project DAO Governance Implementation
use p_project_core::models::{Proposal, ProposalStatus};
use p_project_core::database::Database;
use p_project_contracts::token::PProjectToken;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaoGovernance<D: Database> {
    database: D,
    token_contract: PProjectToken,
    proposals: HashMap<String, Proposal>,
    votes: HashMap<String, HashMap<String, bool>>,
    delegates: HashMap<String, String>, // user_id -> delegate_id
}

impl<D: Database> DaoGovernance<D> {
    pub fn new(database: D, token_contract: PProjectToken) -> Self {
        Self {
            database,
            token_contract,
            proposals: HashMap::new(),
            votes: HashMap::new(),
            delegates: HashMap::new(),
        }
    }

    /// Create a new governance proposal
    pub async fn create_proposal(
        &mut self,
        title: String,
        description: String,
        creator_id: String,
    ) -> Result<String, String> {
        let proposal_id = uuid::Uuid::new_v4().to_string();
        let voting_end_time = Utc::now().naive_utc() + chrono::Duration::days(7); // 7-day voting period
        
        let proposal = Proposal {
            id: proposal_id.clone(),
            title,
            description,
            creator_id,
            created_at: Utc::now().naive_utc(),
            voting_end_time,
            status: ProposalStatus::Active,
            execution_type: None,
            execution_data: None,
            executed_at: None,
        };
        
        // Save to database first
        self.database.save_proposal(&proposal).await.map_err(|e| e.to_string())?;
        
        self.proposals.insert(proposal_id.clone(), proposal);
        self.votes.insert(proposal_id.clone(), HashMap::new());
        
        Ok(proposal_id)
    }

    /// Vote on a proposal
    pub fn vote_on_proposal(
        &mut self,
        proposal_id: &str,
        user_id: &str,
        approve: bool,
    ) -> Result<(), String> {
        let proposal = self
            .proposals
            .get_mut(proposal_id)
            .ok_or("Proposal not found")?;

        if proposal.status != ProposalStatus::Active {
            return Err("Voting is not active for this proposal".to_string());
        }

        // Record the vote
        if let Some(votes) = self.votes.get_mut(proposal_id) {
            votes.insert(user_id.to_string(), approve);
        }

        Ok(())
    }

    /// Get vote count for a proposal
    pub fn get_vote_count(&self, proposal_id: &str) -> Result<(u64, u64), String> {
        let votes = self
            .votes
            .get(proposal_id)
            .ok_or("No votes found for proposal")?;

        let mut approve_votes = 0;
        let mut reject_votes = 0;

        // Count weighted votes based on token balance
        for (user_id, &approve) in votes {
            // If user has delegated their vote, don't count their balance
            if !self.delegates.contains_key(user_id) {
                let token_balance = self.token_contract.get_balance(user_id) as u64;
                if approve {
                    approve_votes += token_balance;
                } else {
                    reject_votes += token_balance;
                }
            }
        }

        // Add delegated votes
        for (delegator, delegate) in &self.delegates {
            if votes.contains_key(delegate) {
                let approve = *votes.get(delegate).unwrap_or(&false);
                let token_balance = self.token_contract.get_balance(delegator) as u64;
                if approve {
                    approve_votes += token_balance;
                } else {
                    reject_votes += token_balance;
                }
            }
        }

        Ok((approve_votes, reject_votes))
    }

    /// Tally votes for a proposal
    pub fn tally_votes(&mut self, proposal_id: &str) -> Result<bool, String> {
        // Check if proposal exists and is active
        let is_active = {
            let proposal = self.proposals.get(proposal_id)
                .ok_or("Proposal not found")?;
            matches!(proposal.status, ProposalStatus::Active)
        };
            
        if !is_active {
            return Err("Proposal is not active".to_string());
        }

        let (approve_votes, reject_votes) = self.get_vote_count(proposal_id)?;
        let total_votes = approve_votes + reject_votes;
        
        // Require minimum participation (e.g., 10% of total supply)
        let total_supply = self.token_contract.get_total_supply() as u64;
        let min_participation = total_supply / 10;
        
        if total_votes < min_participation {
            return Err("Not enough participation to tally votes".to_string());
        }

        // Simple majority wins
        let passed = approve_votes > reject_votes;
        
        // Update the proposal status
        let proposal = self
            .proposals
            .get_mut(proposal_id)
            .ok_or("Proposal not found")?;
        if passed {
            proposal.status = ProposalStatus::Passed;
        } else {
            proposal.status = ProposalStatus::Rejected;
        }

        Ok(passed)
    }

    /// Execute a passed proposal
    pub async fn execute_proposal(&mut self, proposal_id: &str) -> Result<(), String> {
        let proposal = self
            .proposals
            .get_mut(proposal_id)
            .ok_or("Proposal not found")?;

        if proposal.status != ProposalStatus::Passed {
            return Err("Proposal has not passed or is already executed".to_string());
        }

        // In a real implementation, this would execute the proposal's action
        // For now, we'll just mark it as executed
        proposal.status = ProposalStatus::Executed;
        proposal.executed_at = Some(Utc::now().naive_utc());
        
        // Save to database
        self.database.update_proposal(proposal).await.map_err(|e| e.to_string())?;
        
        Ok(())
    }

    /// Delegate voting power to another user
    pub fn delegate_vote(&mut self, from_user_id: &str, to_user_id: &str) -> Result<(), String> {
        // Check that both users exist and have token balances
        let from_balance = self.token_contract.get_balance(from_user_id);
        let to_balance = self.token_contract.get_balance(to_user_id);
        
        if from_balance <= 0.0 || to_balance <= 0.0 {
            return Err("Both users must have token balances".to_string());
        }
        
        // Prevent circular delegation
        if self.check_circular_delegation(to_user_id, from_user_id) {
            return Err("Circular delegation detected".to_string());
        }
        
        self.delegates.insert(from_user_id.to_string(), to_user_id.to_string());
        Ok(())
    }

    /// Remove a delegation
    pub fn remove_delegation(&mut self, user_id: &str) {
        self.delegates.remove(user_id);
    }

    /// Get the delegate for a user
    pub fn get_delegate(&self, user_id: &str) -> Option<&String> {
        self.delegates.get(user_id)
    }

    /// Check for circular delegation
    fn check_circular_delegation(&self, user_a: &str, user_b: &str) -> bool {
        let mut current = user_b;
        while let Some(delegate) = self.delegates.get(current) {
            if delegate == user_a {
                return true;
            }
            current = delegate;
        }
        false
    }

    /// Implement quadratic voting calculation
    /// In quadratic voting, the voting power is the square root of the tokens staked
    pub fn calculate_quadratic_voting_power(&self, user_id: &str) -> f64 {
        let token_balance = self.token_contract.get_balance(user_id);
        // For quadratic voting, voting power = sqrt(token_balance)
        token_balance.sqrt()
    }

    /// Vote on a proposal using quadratic voting
    pub fn quadratic_vote_on_proposal(
        &mut self,
        proposal_id: &str,
        user_id: &str,
        approve: bool,
        _voting_power: f64,
    ) -> Result<(), String> {
        let proposal = self
            .proposals
            .get_mut(proposal_id)
            .ok_or("Proposal not found")?;

        if proposal.status != ProposalStatus::Active {
            return Err("Voting is not active for this proposal".to_string());
        }

        // Record the quadratic vote
        if let Some(votes) = self.votes.get_mut(proposal_id) {
            // Store the voting power along with the vote
            votes.insert(user_id.to_string(), approve);
            // In a real implementation, we would also track the voting power used
        }

        Ok(())
    }

    /// Get quadratic vote count for a proposal
    pub fn get_quadratic_vote_count(&self, proposal_id: &str) -> Result<(f64, f64), String> {
        let votes = self
            .votes
            .get(proposal_id)
            .ok_or("No votes found for proposal")?;

        let mut approve_votes = 0.0;
        let mut reject_votes = 0.0;

        // Count weighted votes based on quadratic voting power
        for (user_id, &approve) in votes {
            let voting_power = self.calculate_quadratic_voting_power(user_id);
            
            // If user has delegated their vote, don't count their balance
            if !self.delegates.contains_key(user_id) {
                if approve {
                    approve_votes += voting_power;
                } else {
                    reject_votes += voting_power;
                }
            }
        }

        Ok((approve_votes, reject_votes))
    }

    /// Batch delegate votes to multiple delegates
    pub fn batch_delegate_votes(&mut self, delegations: Vec<(&str, &str)>) -> Result<(), String> {
        for (from_user_id, to_user_id) in delegations {
            self.delegate_vote(from_user_id, to_user_id)?;
        }
        Ok(())
    }

    /// Get all delegations for a user (who they've delegated to and who has delegated to them)
    pub fn get_user_delegations(&self, user_id: &str) -> (Option<&String>, Vec<&String>) {
        // Who this user has delegated to
        let delegated_to = self.delegates.get(user_id);
        
        // Who has delegated to this user
        let delegated_by: Vec<&String> = self
            .delegates
            .iter()
            .filter(|(_, delegate)| *delegate == user_id)
            .map(|(voter, _)| voter)
            .collect();
        
        (delegated_to, delegated_by)
    }

    /// Remove all delegations for a user
    pub fn remove_all_delegations(&mut self, user_id: &str) {
        // Remove delegation this user has made
        self.delegates.remove(user_id);
        
        // Remove delegations others have made to this user
        self.delegates.retain(|_, delegate| delegate != user_id);
    }

    /// Get all active proposals
    pub async fn get_active_proposals(&self) -> Result<Vec<Proposal>, String> 
    where
        D: Database,
    {
        self.database.get_active_proposals().await.map_err(|e: D::Error| e.to_string())
    }

    /// Get a specific proposal by ID
    pub async fn get_proposal(&self, proposal_id: &str) -> Result<Option<Proposal>, String>
    where
        D: Database,
    {
        self.database.get_proposal(proposal_id).await.map_err(|e: D::Error| e.to_string())
    }
}
