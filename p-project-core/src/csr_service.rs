//! Corporate Social Responsibility utilities for tracking contributions, employee deeds, and matches.

use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct CsrServiceConfig {
    pub currency: String,
    pub max_contribution: f64,
    pub match_rate_cap: f64,
    pub good_deed_point_value: f64,
}

impl Default for CsrServiceConfig {
    fn default() -> Self {
        Self {
            currency: "P".into(),
            max_contribution: 100_000.0,
            match_rate_cap: 1.0,
            good_deed_point_value: 0.05,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ContributionStatus {
    Recorded,
    Matched,
    Settled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsrContribution {
    pub id: String,
    pub project: String,
    pub contributor_id: String,
    pub contributor_name: String,
    pub wallet_address: String,
    pub amount: f64,
    pub currency: String,
    pub status: ContributionStatus,
    pub purpose: Option<String>,
    pub timestamp: NaiveDateTime,
    pub match_campaign_id: Option<String>,
    pub matched_amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CampaignStatus {
    Active,
    Completed,
    Paused,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DonationMatchCampaign {
    pub id: String,
    pub name: String,
    pub target_amount: f64,
    pub match_rate: f64,
    pub currency: String,
    pub status: CampaignStatus,
    pub matched_amount: f64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchRecord {
    pub id: String,
    pub match_campaign_id: String,
    pub contribution_id: String,
    pub matched_amount: f64,
    pub timestamp: NaiveDateTime,
    pub tx_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoodDeedRecord {
    pub id: String,
    pub description: String,
    pub points: f64,
    pub timestamp: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmployeeWallet {
    pub employee_id: String,
    pub name: String,
    pub wallet_address: String,
    pub points: f64,
    pub coin_balance: f64,
    pub deeds: Vec<GoodDeedRecord>,
    pub created_at: NaiveDateTime,
    pub last_updated: NaiveDateTime,
}

pub struct CsrService {
    pub config: CsrServiceConfig,
    pub contributions: HashMap<String, CsrContribution>,
    pub campaigns: HashMap<String, DonationMatchCampaign>,
    pub match_records: HashMap<String, MatchRecord>,
    pub employee_wallets: HashMap<String, EmployeeWallet>,
}

impl CsrService {
    pub fn new(config: CsrServiceConfig) -> Self {
        Self {
            config,
            contributions: HashMap::new(),
            campaigns: HashMap::new(),
            match_records: HashMap::new(),
            employee_wallets: HashMap::new(),
        }
    }

    fn now(&self) -> NaiveDateTime {
        Utc::now().naive_utc()
    }

    fn ensure_currency(&self, currency: &str) -> Result<(), Box<dyn std::error::Error>> {
        if currency != self.config.currency {
            Err(format!("Only {} currency is supported", self.config.currency).into())
        } else {
            Ok(())
        }
    }

    fn ensure_amount(&self, amount: f64) -> Result<(), Box<dyn std::error::Error>> {
        if amount <= 0.0 {
            Err("Contribution amount must be positive".into())
        } else if amount > self.config.max_contribution {
            Err("Contribution exceeds configured maximum".into())
        } else {
            Ok(())
        }
    }

    pub fn register_employee_wallet(
        &mut self,
        employee_id: String,
        name: String,
        wallet_address: String,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let now = self.now();
        let wallet = EmployeeWallet {
            employee_id: employee_id.clone(),
            name,
            wallet_address,
            points: 0.0,
            coin_balance: 0.0,
            deeds: Vec::new(),
            created_at: now,
            last_updated: now,
        };
        self.employee_wallets.insert(employee_id.clone(), wallet);
        Ok(employee_id)
    }

    pub fn award_good_deed(
        &mut self,
        employee_id: &str,
        description: String,
        points: f64,
    ) -> Result<f64, Box<dyn std::error::Error>> {
        if points <= 0.0 {
            return Err("Good deed points must be positive".into());
        }
        let now = self.now();
        let wallet = self
            .employee_wallets
            .get_mut(employee_id)
            .ok_or("Employee wallet not found")?;
        wallet.points += points;
        wallet.last_updated = now;
        let deed_record = GoodDeedRecord {
            id: format!("deed_{}", Uuid::new_v4()),
            description,
            points,
            timestamp: now,
        };
        wallet.deeds.push(deed_record);
        Ok(wallet.points)
    }

    pub fn redeem_good_deed_points(
        &mut self,
        employee_id: &str,
        points: f64,
    ) -> Result<f64, Box<dyn std::error::Error>> {
        if points <= 0.0 {
            return Err("Points to redeem must be positive".into());
        }
        let now = self.now();
        let wallet = self
            .employee_wallets
            .get_mut(employee_id)
            .ok_or("Employee wallet not found")?;
        if wallet.points < points {
            return Err("Not enough points available".into());
        }
        wallet.points -= points;
        let minted = points * self.config.good_deed_point_value;
        wallet.coin_balance += minted;
        wallet.last_updated = now;
        let deed_record = GoodDeedRecord {
            id: format!("deed_{}", Uuid::new_v4()),
            description: format!("Redeemed {} points for P-Coin", points),
            points: -points,
            timestamp: now,
        };
        wallet.deeds.push(deed_record);
        Ok(minted)
    }

    pub fn record_contribution(
        &mut self,
        project: String,
        contributor_id: String,
        contributor_name: String,
        wallet_address: String,
        amount: f64,
        currency: String,
        purpose: Option<String>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        self.ensure_currency(&currency)?;
        self.ensure_amount(amount)?;
        let id = format!("csr_{}", Uuid::new_v4());
        let contribution = CsrContribution {
            id: id.clone(),
            project,
            contributor_id,
            contributor_name,
            wallet_address,
            amount,
            currency,
            status: ContributionStatus::Recorded,
            purpose,
            timestamp: self.now(),
            match_campaign_id: None,
            matched_amount: 0.0,
        };
        self.contributions.insert(id.clone(), contribution);
        Ok(id)
    }

    pub fn register_match_campaign(
        &mut self,
        name: String,
        target_amount: f64,
        match_rate: f64,
    ) -> Result<String, Box<dyn std::error::Error>> {
        if target_amount <= 0.0 {
            return Err("Target amount must be positive".into());
        }
        if !(0.0..=self.config.match_rate_cap).contains(&match_rate) {
            return Err("Match rate is out of bounds".into());
        }
        let id = format!("campaign_{}", Uuid::new_v4());
        let now = self.now();
        let campaign = DonationMatchCampaign {
            id: id.clone(),
            name,
            target_amount,
            match_rate,
            currency: self.config.currency.clone(),
            status: CampaignStatus::Active,
            matched_amount: 0.0,
            created_at: now,
            updated_at: now,
        };
        self.campaigns.insert(id.clone(), campaign);
        Ok(id)
    }

    pub fn match_contribution(
        &mut self,
        campaign_id: &str,
        contribution_id: &str,
    ) -> Result<f64, Box<dyn std::error::Error>> {
        let now = self.now();
        let contribution = self
            .contributions
            .get_mut(contribution_id)
            .ok_or("Contribution not found")?;
        if contribution.status != ContributionStatus::Recorded {
            return Err("Contribution is already matched or settled".into());
        }
        let campaign = self
            .campaigns
            .get_mut(campaign_id)
            .ok_or("Campaign not found")?;
        if campaign.status != CampaignStatus::Active {
            return Err("Campaign is not active".into());
        }
        let available = campaign.target_amount - campaign.matched_amount;
        if available <= 0.0 {
            campaign.status = CampaignStatus::Completed;
            return Err("Campaign budget exhausted".into());
        }
        let wanted = contribution.amount * campaign.match_rate;
        let match_amount = wanted.min(available);
        if match_amount <= 0.0 {
            return Err("No match amount available".into());
        }
        campaign.matched_amount += match_amount;
        campaign.updated_at = now;
        if (campaign.matched_amount - campaign.target_amount).abs() < f64::EPSILON
            || campaign.matched_amount >= campaign.target_amount
        {
            campaign.status = CampaignStatus::Completed;
        }
        contribution.match_campaign_id = Some(campaign_id.to_string());
        contribution.matched_amount = match_amount;
        contribution.status = ContributionStatus::Matched;
        let match_id = format!("match_{}", Uuid::new_v4());
        let record = MatchRecord {
            id: match_id.clone(),
            match_campaign_id: campaign_id.to_string(),
            contribution_id: contribution_id.to_string(),
            matched_amount: match_amount,
            timestamp: now,
            tx_hash: Some(format!("0x{}", Uuid::new_v4().simple())),
        };
        self.match_records.insert(match_id, record);
        Ok(match_amount)
    }

    pub fn settle_contribution(
        &mut self,
        contribution_id: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let contribution = self
            .contributions
            .get_mut(contribution_id)
            .ok_or("Contribution not found")?;
        contribution.status = ContributionStatus::Settled;
        Ok(())
    }

    pub fn get_contribution(&self, contribution_id: &str) -> Option<&CsrContribution> {
        self.contributions.get(contribution_id)
    }

    pub fn get_campaign(&self, campaign_id: &str) -> Option<&DonationMatchCampaign> {
        self.campaigns.get(campaign_id)
    }

    pub fn get_wallet(&self, employee_id: &str) -> Option<&EmployeeWallet> {
        self.employee_wallets.get(employee_id)
    }

    pub fn get_match_record(&self, record_id: &str) -> Option<&MatchRecord> {
        self.match_records.get(record_id)
    }
}
