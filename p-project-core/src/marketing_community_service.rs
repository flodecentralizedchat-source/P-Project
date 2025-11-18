//! Marketing & Community budget controls
//! Handles supply-based budgets, community rewards, influencer campaigns, organic activity logs, and DAO proposals.
use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MarketingCategory {
    MarketingBudget,
    CommunityRewards,
    OrganicMarketing,
    InfluencerStrategy,
    CommunityDao,
}

impl MarketingCategory {
    fn all() -> &'static [MarketingCategory] {
        &[
            MarketingCategory::MarketingBudget,
            MarketingCategory::CommunityRewards,
            MarketingCategory::OrganicMarketing,
            MarketingCategory::InfluencerStrategy,
            MarketingCategory::CommunityDao,
        ]
    }

    fn label(&self) -> &'static str {
        match self {
            MarketingCategory::MarketingBudget => "Marketing Budget",
            MarketingCategory::CommunityRewards => "Community Rewards",
            MarketingCategory::OrganicMarketing => "Organic Marketing",
            MarketingCategory::InfluencerStrategy => "Influencer Strategy",
            MarketingCategory::CommunityDao => "Community DAO",
        }
    }
}

#[derive(Debug, Clone)]
pub struct BudgetState {
    total: f64,
    spent: f64,
}

impl BudgetState {
    fn new(total: f64) -> Self {
        Self { total, spent: 0.0 }
    }

    fn remaining(&self) -> f64 {
        (self.total - self.spent).max(0.0)
    }

    fn spend(&mut self, amount: f64) -> Result<(), String> {
        if amount <= 0.0 {
            return Err("amount_must_be_positive".to_string());
        }
        if amount > self.remaining() + f64::EPSILON {
            return Err("insufficient_budget".to_string());
        }
        self.spent += amount;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketingCommunityConfig {
    pub total_supply: f64,
    pub marketing_budget_pct: f64,
    pub community_rewards_pct: f64,
    pub organic_marketing_pct: f64,
    pub influencer_strategy_pct: f64,
    pub dao_fund_pct: f64,
}

impl MarketingCommunityConfig {
    pub fn default(total_supply: f64) -> Self {
        Self {
            total_supply,
            marketing_budget_pct: 0.05,
            community_rewards_pct: 0.02,
            organic_marketing_pct: 0.01,
            influencer_strategy_pct: 0.01,
            dao_fund_pct: 0.01,
        }
    }

    fn validate(&self) -> Result<(), String> {
        if self.total_supply <= 0.0 {
            return Err("total_supply_required".to_string());
        }
        let sum = self.marketing_budget_pct
            + self.community_rewards_pct
            + self.organic_marketing_pct
            + self.influencer_strategy_pct
            + self.dao_fund_pct;
        if sum > 1.0 {
            return Err("percentages_must_sum_to_one_or_less".to_string());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganicActivity {
    pub id: String,
    pub focus: String,
    pub channel: String,
    pub notes: Vec<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CampaignStatus {
    Proposed,
    Running,
    Completed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfluencerCampaign {
    pub id: String,
    pub influencer_handle: String,
    pub influence_channel: String,
    pub budget_allocated: f64,
    pub deliverables: Vec<String>,
    pub status: CampaignStatus,
    pub created_at: NaiveDateTime,
    pub completed_at: Option<NaiveDateTime>,
    pub report: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProposalStatus {
    Draft,
    Active,
    Passed,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityProposal {
    pub id: String,
    pub title: String,
    pub description: String,
    pub requested_tokens: f64,
    pub status: ProposalStatus,
    pub votes_for: usize,
    pub votes_against: usize,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Default)]
pub struct MarketingCommunityService {
    config: MarketingCommunityConfig,
    budgets: HashMap<MarketingCategory, BudgetState>,
    rewards_ledger: HashMap<String, f64>,
    organic_activity: Vec<OrganicActivity>,
    influencer_campaigns: HashMap<String, InfluencerCampaign>,
    proposals: HashMap<String, CommunityProposal>,
}

impl MarketingCommunityService {
    pub fn with_supply(total_supply: f64) -> Result<Self, String> {
        let config = MarketingCommunityConfig::default(total_supply);
        Self::new(config)
    }

    pub fn new(config: MarketingCommunityConfig) -> Result<Self, String> {
        config.validate()?;
        let mut budgets = HashMap::new();
        budgets.insert(
            MarketingCategory::MarketingBudget,
            BudgetState::new(config.total_supply * config.marketing_budget_pct),
        );
        budgets.insert(
            MarketingCategory::CommunityRewards,
            BudgetState::new(config.total_supply * config.community_rewards_pct),
        );
        budgets.insert(
            MarketingCategory::OrganicMarketing,
            BudgetState::new(config.total_supply * config.organic_marketing_pct),
        );
        budgets.insert(
            MarketingCategory::InfluencerStrategy,
            BudgetState::new(config.total_supply * config.influencer_strategy_pct),
        );
        budgets.insert(
            MarketingCategory::CommunityDao,
            BudgetState::new(config.total_supply * config.dao_fund_pct),
        );
        Ok(Self {
            config,
            budgets,
            rewards_ledger: HashMap::new(),
            organic_activity: Vec::new(),
            influencer_campaigns: HashMap::new(),
            proposals: HashMap::new(),
        })
    }

    pub fn get_config(&self) -> &MarketingCommunityConfig {
        &self.config
    }

    pub fn remaining_budget(&self, category: MarketingCategory) -> f64 {
        self.budgets
            .get(&category)
            .map(|b| b.remaining())
            .unwrap_or(0.0)
    }

    pub fn spend_budget(&mut self, category: MarketingCategory, amount: f64) -> Result<(), String> {
        let budget = self
            .budgets
            .get_mut(&category)
            .ok_or_else(|| "category_missing".to_string())?;
        budget.spend(amount)
    }

    pub fn award_community_reward(&mut self, user_id: &str, amount: f64) -> Result<(), String> {
        self.spend_budget(MarketingCategory::CommunityRewards, amount)?;
        *self.rewards_ledger.entry(user_id.to_string()).or_default() += amount;
        Ok(())
    }

    pub fn reward_balance(&self, user_id: &str) -> f64 {
        *self.rewards_ledger.get(user_id).unwrap_or(&0.0)
    }

    pub fn log_organic_activity(
        &mut self,
        focus: String,
        channel: String,
        notes: Vec<String>,
    ) -> OrganicActivity {
        let activity = OrganicActivity {
            id: crate::utils::generate_id(),
            focus,
            channel,
            notes,
            created_at: Utc::now().naive_utc(),
        };
        self.organic_activity.push(activity.clone());
        activity
    }

    pub fn list_organic_activity(&self, limit: usize) -> Vec<OrganicActivity> {
        let mut v = self.organic_activity.clone();
        v.sort_by_key(|a| std::cmp::Reverse(a.created_at));
        v.into_iter().take(limit.max(1)).collect()
    }

    pub fn create_influencer_campaign(
        &mut self,
        influencer_handle: String,
        channel: String,
        budget: f64,
        deliverables: Vec<String>,
    ) -> Result<InfluencerCampaign, String> {
        self.spend_budget(MarketingCategory::InfluencerStrategy, budget)?;
        let campaign = InfluencerCampaign {
            id: crate::utils::generate_id(),
            influencer_handle,
            influence_channel: channel,
            budget_allocated: budget,
            deliverables,
            status: CampaignStatus::Proposed,
            created_at: Utc::now().naive_utc(),
            completed_at: None,
            report: None,
        };
        self.influencer_campaigns
            .insert(campaign.id.clone(), campaign.clone());
        Ok(campaign)
    }

    pub fn update_campaign_status(
        &mut self,
        campaign_id: &str,
        status: CampaignStatus,
        report: Option<String>,
    ) -> Result<InfluencerCampaign, String> {
        let campaign = self
            .influencer_campaigns
            .get_mut(campaign_id)
            .ok_or_else(|| "campaign_not_found".to_string())?;
        campaign.status = status;
        if campaign.status == CampaignStatus::Completed {
            campaign.completed_at = Some(Utc::now().naive_utc());
        }
        if report.is_some() {
            campaign.report = report;
        }
        Ok(campaign.clone())
    }

    pub fn list_campaigns(&self) -> Vec<InfluencerCampaign> {
        let mut v: Vec<_> = self.influencer_campaigns.values().cloned().collect();
        v.sort_by_key(|c| std::cmp::Reverse(c.created_at));
        v
    }

    pub fn submit_community_proposal(
        &mut self,
        title: String,
        description: String,
        requested_tokens: f64,
    ) -> Result<String, String> {
        if requested_tokens <= 0.0 {
            return Err("requested_tokens_must_be_positive".to_string());
        }
        let id = crate::utils::generate_id();
        let proposal = CommunityProposal {
            id: id.clone(),
            title,
            description,
            requested_tokens,
            status: ProposalStatus::Active,
            votes_for: 0,
            votes_against: 0,
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
        };
        self.proposals.insert(id.clone(), proposal);
        Ok(id)
    }

    pub fn vote_proposal(&mut self, proposal_id: &str, approve: bool) -> Result<(), String> {
        let proposal = self
            .proposals
            .get_mut(proposal_id)
            .ok_or_else(|| "proposal_not_found".to_string())?;
        if proposal.status != ProposalStatus::Active {
            return Err("proposal_not_active".to_string());
        }
        if approve {
            proposal.votes_for += 1;
        } else {
            proposal.votes_against += 1;
        }
        proposal.updated_at = Utc::now().naive_utc();
        Ok(())
    }

    pub fn finalize_proposal(&mut self, proposal_id: &str) -> Result<CommunityProposal, String> {
        let proposal = self
            .proposals
            .get_mut(proposal_id)
            .ok_or_else(|| "proposal_not_found".to_string())?;
        if proposal.status != ProposalStatus::Active {
            return Err("proposal_not_active".to_string());
        }
        if proposal.votes_for <= proposal.votes_against {
            proposal.status = ProposalStatus::Rejected;
        } else {
            self.spend_budget(MarketingCategory::CommunityDao, proposal.requested_tokens)?;
            proposal.status = ProposalStatus::Passed;
        }
        proposal.updated_at = Utc::now().naive_utc();
        Ok(proposal.clone())
    }

    pub fn list_proposals(&self) -> Vec<CommunityProposal> {
        let mut v: Vec<_> = self.proposals.values().cloned().collect();
        v.sort_by_key(|p| std::cmp::Reverse(p.created_at));
        v
    }
}
