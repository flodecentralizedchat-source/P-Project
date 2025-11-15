//! DAO governance services and helpers.
//!
//! Implements proposal creation, voting, tallying, and execution for charity allocations,
//! treasury decisions, partnership expansion, and NGO onboarding.

use chrono::{Duration, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct DaoGovernanceConfig {
    pub required_quorum: usize,
    pub vote_duration_days: i64,
    pub majority_percentage: f64,
}

impl Default for DaoGovernanceConfig {
    fn default() -> Self {
        Self {
            required_quorum: 3,
            vote_duration_days: 7,
            majority_percentage: 0.5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DaoProposalType {
    CharityAllocation,
    TreasuryDecision,
    PartnershipExpansion,
    NewNgo,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DaoProposalStatus {
    Active,
    Passed,
    Rejected,
    Executed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum VoteChoice {
    For,
    Against,
    Abstain,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaoProposal {
    pub id: String,
    pub title: String,
    pub description: String,
    pub proposal_type: DaoProposalType,
    pub target_details: Value,
    pub submitted_by: String,
    pub status: DaoProposalStatus,
    pub created_at: NaiveDateTime,
    pub voting_deadline: NaiveDateTime,
    pub executed_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone)]
pub struct DaoGovernanceService {
    config: DaoGovernanceConfig,
    proposals: HashMap<String, DaoProposal>,
    votes: HashMap<String, HashMap<String, VoteChoice>>,
}

impl DaoGovernanceService {
    pub fn new(config: DaoGovernanceConfig) -> Self {
        Self {
            config,
            proposals: HashMap::new(),
            votes: HashMap::new(),
        }
    }

    fn now() -> NaiveDateTime {
        Utc::now().naive_utc()
    }

    fn ensure_valid_percentage(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.config.majority_percentage <= 0.0 || self.config.majority_percentage > 1.0 {
            return Err("majority percentage must be > 0 and <= 1".into());
        }
        if self.config.vote_duration_days <= 0 {
            return Err("vote duration must be positive".into());
        }
        Ok(())
    }

    /// Create new proposal.
    pub fn create_proposal(
        &mut self,
        title: String,
        description: String,
        proposal_type: DaoProposalType,
        target_details: Option<Value>,
        submitted_by: String,
    ) -> Result<String, Box<dyn std::error::Error>> {
        if title.trim().is_empty() || description.trim().is_empty() {
            return Err("title and description are required".into());
        }

        self.ensure_valid_percentage()?;

        let id = format!("dao_{}", Uuid::new_v4());
        let created_at = Self::now();
        let voting_deadline = created_at + Duration::days(self.config.vote_duration_days);
        let proposal = DaoProposal {
            id: id.clone(),
            title,
            description,
            proposal_type,
            target_details: target_details.unwrap_or(Value::Null),
            submitted_by,
            status: DaoProposalStatus::Active,
            created_at,
            voting_deadline,
            executed_at: None,
        };

        self.proposals.insert(id.clone(), proposal);
        Ok(id)
    }

    fn ensure_active(&self, proposal: &DaoProposal) -> Result<(), Box<dyn std::error::Error>> {
        if proposal.status != DaoProposalStatus::Active {
            return Err("proposal is not in an active voting state".into());
        }
        if Self::now() > proposal.voting_deadline {
            return Err("voting period has ended for this proposal".into());
        }
        Ok(())
    }

    pub fn cast_vote(
        &mut self,
        proposal_id: &str,
        voter_id: String,
        choice: VoteChoice,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let proposal = self
            .proposals
            .get(proposal_id)
            .ok_or("proposal not found")?;

        self.ensure_active(proposal)?;

        let vote_map = self.votes.entry(proposal_id.to_string()).or_default();
        vote_map.insert(voter_id, choice);
        Ok(())
    }

    fn count_votes(&self, proposal_id: &str) -> (usize, usize, usize) {
        let votes = self.votes.get(proposal_id);
        let mut for_votes = 0;
        let mut against_votes = 0;
        let mut abstain_votes = 0;
        if let Some(map) = votes {
            for choice in map.values() {
                match choice {
                    VoteChoice::For => for_votes += 1,
                    VoteChoice::Against => against_votes += 1,
                    VoteChoice::Abstain => abstain_votes += 1,
                }
            }
        }
        (for_votes, against_votes, abstain_votes)
    }

    fn has_quorum(&self, total_votes: usize) -> bool {
        total_votes >= self.config.required_quorum
    }

    fn has_majority(&self, for_votes: usize, against_votes: usize) -> bool {
        let total = for_votes + against_votes;
        if total == 0 {
            return false;
        }
        let ratio = for_votes as f64 / total as f64;
        ratio >= self.config.majority_percentage
    }

    pub fn finalize_proposal(
        &mut self,
        proposal_id: &str,
    ) -> Result<DaoProposalStatus, Box<dyn std::error::Error>> {
        // First, check if proposal exists and is active, and get the necessary data
        let majority_percentage = {
            let proposal = self
                .proposals
                .get(proposal_id)
                .ok_or("proposal not found")?;

            if proposal.status != DaoProposalStatus::Active {
                return Err("proposal already finalized".into());
            }

            self.config.majority_percentage
        };

        // Get vote counts
        let (for_votes, against_votes, abstain_votes) = self.count_votes(proposal_id);
        let total_votes = for_votes + against_votes + abstain_votes;

        // Check quorum
        let has_quorum = total_votes >= self.config.required_quorum;

        if !has_quorum {
            return Err("quorum not met for this proposal".into());
        }

        // Determine status
        let has_majority = {
            let total = for_votes + against_votes;
            if total == 0 {
                false
            } else {
                let ratio = for_votes as f64 / total as f64;
                ratio >= majority_percentage
            }
        };

        let status = if has_majority {
            DaoProposalStatus::Passed
        } else {
            DaoProposalStatus::Rejected
        };

        // Now get a mutable reference to update the proposal
        let proposal = self.proposals.get_mut(proposal_id).unwrap(); // Safe because we already checked it exists

        proposal.status = status.clone();
        Ok(status)
    }

    pub fn execute_proposal(
        &mut self,
        proposal_id: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let proposal = self
            .proposals
            .get_mut(proposal_id)
            .ok_or("proposal not found")?;

        if proposal.status != DaoProposalStatus::Passed {
            return Err("only passed proposals can be executed".into());
        }

        proposal.status = DaoProposalStatus::Executed;
        proposal.executed_at = Some(Self::now());
        Ok(())
    }

    pub fn get_proposal(&self, proposal_id: &str) -> Option<&DaoProposal> {
        self.proposals.get(proposal_id)
    }

    pub fn list_proposals(&self) -> Vec<&DaoProposal> {
        self.proposals.values().collect()
    }

    pub fn get_votes(&self, proposal_id: &str) -> Option<&HashMap<String, VoteChoice>> {
        self.votes.get(proposal_id)
    }
}
