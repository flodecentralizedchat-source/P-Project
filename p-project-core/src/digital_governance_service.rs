//! Digital governance layers for virtual town halls, on-chain votes, and district funding proposals.

use chrono::{Duration, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct DigitalGovernanceConfig {
    pub required_quorum: usize,
    pub majority_threshold: f64,
    pub vote_duration_days: i64,
    pub max_district_funding: f64,
}

impl Default for DigitalGovernanceConfig {
    fn default() -> Self {
        Self {
            required_quorum: 5,
            majority_threshold: 0.6,
            vote_duration_days: 3,
            max_district_funding: 1_000_000.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualTownHall {
    pub id: String,
    pub title: String,
    pub district: String,
    pub host: String,
    pub agenda: Vec<String>,
    pub scheduled_at: NaiveDateTime,
    pub duration_minutes: u32,
    pub attendee_count: usize,
    pub recording_url: Option<String>,
    pub summary: Option<String>,
    pub is_active: bool,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum VoteChoice {
    For,
    Against,
    Abstain,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProposalStatus {
    Active,
    Passed,
    Rejected,
    Closed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnChainVoteProposal {
    pub id: String,
    pub title: String,
    pub description: String,
    pub district: String,
    pub metadata: Value,
    pub status: ProposalStatus,
    pub created_at: NaiveDateTime,
    pub closing_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistrictFundingProposal {
    pub id: String,
    pub district: String,
    pub title: String,
    pub amount_requested: f64,
    pub purpose: String,
    pub status: ProposalStatus,
    pub amount_allocated: f64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

pub struct DigitalGovernanceService {
    config: DigitalGovernanceConfig,
    town_halls: HashMap<String, VirtualTownHall>,
    votes: HashMap<String, OnChainVoteProposal>,
    vote_records: HashMap<String, HashMap<String, VoteChoice>>,
    funding_proposals: HashMap<String, DistrictFundingProposal>,
}

impl DigitalGovernanceService {
    pub fn new(config: DigitalGovernanceConfig) -> Self {
        Self {
            config,
            town_halls: HashMap::new(),
            votes: HashMap::new(),
            vote_records: HashMap::new(),
            funding_proposals: HashMap::new(),
        }
    }

    fn now() -> NaiveDateTime {
        Utc::now().naive_utc()
    }

    pub fn schedule_town_hall(
        &mut self,
        title: String,
        district: String,
        host: String,
        agenda: Vec<String>,
        scheduled_at: NaiveDateTime,
        duration_minutes: u32,
    ) -> Result<String, Box<dyn std::error::Error>> {
        if scheduled_at <= Self::now() {
            return Err("Town hall must be scheduled in the future".into());
        }
        if duration_minutes == 0 {
            return Err("Duration must be positive".into());
        }
        let id = format!("townhall_{}", Uuid::new_v4());
        let hall = VirtualTownHall {
            id: id.clone(),
            title,
            district,
            host,
            agenda,
            scheduled_at,
            duration_minutes,
            attendee_count: 0,
            recording_url: None,
            summary: None,
            is_active: true,
            created_at: Self::now(),
        };
        self.town_halls.insert(id.clone(), hall);
        Ok(id)
    }

    pub fn record_town_hall_attendance(
        &mut self,
        hall_id: &str,
        attendees: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let hall = self
            .town_halls
            .get_mut(hall_id)
            .ok_or("Town hall not found")?;

        if !hall.is_active {
            return Err("Town hall is no longer active".into());
        }

        hall.attendee_count += attendees;
        Ok(())
    }

    pub fn conclude_town_hall(
        &mut self,
        hall_id: &str,
        summary: Option<String>,
        recording_url: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let hall = self
            .town_halls
            .get_mut(hall_id)
            .ok_or("Town hall not found")?;

        hall.summary = summary;
        hall.recording_url = recording_url;
        hall.is_active = false;
        Ok(())
    }

    pub fn create_onchain_vote(
        &mut self,
        title: String,
        description: String,
        district: String,
        metadata: Option<Value>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        if title.trim().is_empty() || description.trim().is_empty() {
            return Err("Title and description required".into());
        }

        let id = format!("vote_{}", Uuid::new_v4());
        let created = Self::now();
        let closing_at = created + Duration::days(self.config.vote_duration_days);
        let proposal = OnChainVoteProposal {
            id: id.clone(),
            title,
            description,
            district,
            metadata: metadata.unwrap_or(Value::Null),
            status: ProposalStatus::Active,
            created_at: created,
            closing_at,
        };
        self.votes.insert(id.clone(), proposal);
        Ok(id)
    }

    pub fn cast_vote(
        &mut self,
        vote_id: &str,
        voter_id: String,
        choice: VoteChoice,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let proposal = self.votes.get(vote_id).ok_or("Vote not found")?;
        if proposal.status != ProposalStatus::Active {
            return Err("Vote is not active".into());
        }
        if Self::now() > proposal.closing_at {
            return Err("Voting period closed".into());
        }
        let vote_map = self.vote_records.entry(vote_id.to_string()).or_default();
        vote_map.insert(voter_id, choice);
        Ok(())
    }

    pub fn tally_votes(
        &mut self,
        vote_id: &str,
    ) -> Result<ProposalStatus, Box<dyn std::error::Error>> {
        let proposal = self.votes.get_mut(vote_id).ok_or("Vote not found")?;
        if proposal.status != ProposalStatus::Active {
            return Err("Vote already finalized".into());
        }
        let records = self.vote_records.get(vote_id);
        let (mut for_votes, mut against_votes, mut abstain) = (0usize, 0usize, 0usize);
        if let Some(map) = records {
            for choice in map.values() {
                match choice {
                    VoteChoice::For => for_votes += 1,
                    VoteChoice::Against => against_votes += 1,
                    VoteChoice::Abstain => abstain += 1,
                }
            }
        }
        let total = for_votes + against_votes;
        if total + abstain < self.config.required_quorum {
            return Err("Quorum not reached".into());
        }
        let status =
            if total > 0 && (for_votes as f64 / total as f64) >= self.config.majority_threshold {
                ProposalStatus::Passed
            } else {
                ProposalStatus::Rejected
            };
        proposal.status = status.clone();
        Ok(status)
    }

    pub fn create_funding_proposal(
        &mut self,
        district: String,
        title: String,
        purpose: String,
        amount_requested: f64,
    ) -> Result<String, Box<dyn std::error::Error>> {
        if amount_requested <= 0.0 {
            return Err("Amount must be positive".into());
        }
        if amount_requested > self.config.max_district_funding {
            return Err("Requested amount exceeds maximum district funding".into());
        }

        let id = format!("district_{}", Uuid::new_v4());
        let now = Self::now();
        let proposal = DistrictFundingProposal {
            id: id.clone(),
            district,
            title,
            amount_requested,
            purpose,
            status: ProposalStatus::Active,
            amount_allocated: 0.0,
            created_at: now,
            updated_at: now,
        };
        self.funding_proposals.insert(id.clone(), proposal);
        Ok(id)
    }

    pub fn allocate_district_funding(
        &mut self,
        proposal_id: &str,
        amount: f64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let proposal = self
            .funding_proposals
            .get_mut(proposal_id)
            .ok_or("Funding proposal not found")?;
        if proposal.status != ProposalStatus::Active {
            return Err("Funding proposal is not active".into());
        }
        if amount <= 0.0 {
            return Err("Allocation must be positive".into());
        }
        let remaining = proposal.amount_requested - proposal.amount_allocated;
        if amount > remaining {
            return Err("Allocation exceeds requested amount".into());
        }
        proposal.amount_allocated += amount;
        if (proposal.amount_allocated - proposal.amount_requested).abs() < f64::EPSILON {
            proposal.status = ProposalStatus::Passed;
        }
        proposal.updated_at = Self::now();
        Ok(())
    }

    pub fn decline_funding(&mut self, proposal_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let proposal = self
            .funding_proposals
            .get_mut(proposal_id)
            .ok_or("Funding proposal not found")?;
        proposal.status = ProposalStatus::Rejected;
        proposal.updated_at = Self::now();
        Ok(())
    }

    pub fn get_town_hall(&self, hall_id: &str) -> Option<&VirtualTownHall> {
        self.town_halls.get(hall_id)
    }

    pub fn get_vote(&self, vote_id: &str) -> Option<&OnChainVoteProposal> {
        self.votes.get(vote_id)
    }

    pub fn get_funding_proposal(&self, proposal_id: &str) -> Option<&DistrictFundingProposal> {
        self.funding_proposals.get(proposal_id)
    }
}
