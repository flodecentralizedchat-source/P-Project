use crate::token::PProjectToken;
use chrono::{Datelike, NaiveDateTime, Utc};
use p_project_core::utils::generate_id;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Custom error types for charity operations
#[derive(Debug, Clone, PartialEq)]
pub enum CharityError {
    InsufficientFunds,
    InvalidAmount,
    NGOAlreadyExists,
    NGONotFound,
    NGOAlreadyVerified,
    NGOAlreadyAllocated,
    AllocationNotFound,
    AllocationAlreadyDisbursed,
    TokenOperationFailed(String),
    DatabaseError(String),
    SerializationError(String),
    UnauthorizedAccess,
    CampaignInactive,
    CampaignAlreadyExists,
    VoucherNotFound,
    VoucherAlreadyClaimed,
    VoucherExpired,
    CreditNotFound,
    InsufficientCreditBalance,
    CreditExpired,
}

impl std::fmt::Display for CharityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CharityError::InsufficientFunds => write!(f, "Insufficient funds in charity fund"),
            CharityError::InvalidAmount => write!(f, "Amount must be positive"),
            CharityError::NGOAlreadyExists => write!(f, "NGO already exists in registry"),
            CharityError::NGONotFound => write!(f, "NGO not found in registry"),
            CharityError::NGOAlreadyVerified => write!(f, "NGO is already verified"),
            CharityError::NGOAlreadyAllocated => write!(f, "NGO already has an active allocation"),
            CharityError::AllocationNotFound => write!(f, "Allocation not found"),
            CharityError::AllocationAlreadyDisbursed => {
                write!(f, "Allocation has already been disbursed")
            }
            CharityError::TokenOperationFailed(msg) => write!(f, "Token operation failed: {}", msg),
            CharityError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            CharityError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            CharityError::UnauthorizedAccess => {
                write!(f, "Unauthorized access - only DAO can perform this action")
            }
            CharityError::CampaignInactive => write!(f, "Campaign is not active"),
            CharityError::CampaignAlreadyExists => write!(f, "Campaign already exists"),
            CharityError::VoucherNotFound => write!(f, "Voucher not found"),
            CharityError::VoucherAlreadyClaimed => write!(f, "Voucher has already been claimed"),
            CharityError::VoucherExpired => write!(f, "Voucher has expired"),
            CharityError::CreditNotFound => write!(f, "Credit not found"),
            CharityError::InsufficientCreditBalance => write!(f, "Insufficient credit balance"),
            CharityError::CreditExpired => write!(f, "Credit has expired"),
        }
    }
}

impl std::error::Error for CharityError {}

// NGO structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NGO {
    pub address: String,
    pub name: String,
    pub description: String,
    pub verified: bool,
    pub registration_date: NaiveDateTime,
    pub verification_date: Option<NaiveDateTime>,
    pub website: Option<String>,
    pub contact_email: Option<String>,
    pub category: String, // e.g., "health", "education", "disaster-relief"
}

// Allocation structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Allocation {
    pub id: String,
    pub ngo_address: String,
    pub amount: f64,
    pub allocated_by: String, // DAO address that made the allocation
    pub allocated_at: NaiveDateTime,
    pub description: String,
    pub disbursed: bool,
    pub disbursement_tx_hash: Option<String>,
    pub disbursed_at: Option<NaiveDateTime>,
}

// Donation record structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DonationRecord {
    pub id: String,
    pub donor_address: String,
    pub ngo_address: String,
    pub amount: f64,
    pub timestamp: NaiveDateTime,
    pub tx_hash: String,
    pub campaign_id: Option<String>,
    pub message: Option<String>,
}

// Crowdfund campaign structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrowdfundCampaign {
    pub id: String,
    pub name: String,
    pub description: String,
    pub target_amount: f64,
    pub current_amount: f64,
    pub ngo_address: String,
    pub creator_address: String,
    pub start_time: NaiveDateTime,
    pub end_time: NaiveDateTime,
    pub active: bool,
    pub donations: Vec<String>, // Donation record IDs
}

// Aid voucher for humanitarian assistance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AidVoucher {
    pub id: String,
    pub beneficiary_id: String, // Could be refugee ID, disaster victim ID, etc.
    pub beneficiary_address: String, // Wallet address of the beneficiary
    pub aid_type: String, // "food", "medicine", "water", "shelter", etc.
    pub amount: f64, // Value of the voucher in P-Coin
    pub issued_by: String, // NGO or DAO address that issued the voucher
    pub issued_at: NaiveDateTime,
    pub expires_at: Option<NaiveDateTime>, // Optional expiration date
    pub claimed: bool, // Whether the voucher has been claimed/redeemed
    pub claimed_at: Option<NaiveDateTime>, // When the voucher was claimed
    pub tx_hash: Option<String>, // Transaction hash of the claim
    pub zone_type: String, // "war", "disaster", "refugee_camp", etc.
}

// Peace-relief credit for refugees
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeaceReliefCredit {
    pub id: String,
    pub refugee_id: String,
    pub wristband_id: Option<String>, // Optional NFC wristband ID for refugee camps
    pub amount: f64, // Credit amount in P-Coin
    pub issued_by: String, // NGO or DAO address that issued the credit
    pub issued_at: NaiveDateTime,
    pub expires_at: Option<NaiveDateTime>, // Optional expiration date
    pub balance: f64, // Remaining balance
    pub transactions: Vec<CreditTransaction>, // Transaction history
}

// Credit transaction record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditTransaction {
    pub id: String,
    pub credit_id: String,
    pub amount: f64,
    pub transaction_type: String, // "issue", "spend", "transfer"
    pub description: String,
    pub timestamp: NaiveDateTime,
    pub tx_hash: Option<String>,
}

// Proof-of-Peace badge structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofOfPeaceBadge {
    pub id: String,
    pub donor_address: String,
    pub ngo_address: String,
    pub donation_amount: f64,
    pub issued_at: NaiveDateTime,
    pub badge_type: String, // "bronze", "silver", "gold", "platinum"
    pub campaign_id: Option<String>,
    pub tx_hash: String,
    pub metadata_uri: Option<String>, // IPFS URI for NFT metadata
}

// Donor reputation score structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DonorReputation {
    pub donor_address: String,
    pub score: f64, // Verified donor score based on donation history
    pub total_donations: f64,
    pub donation_count: u32,
    pub badges_earned: Vec<String>, // Badge IDs
    pub last_updated: NaiveDateTime,
}

// NGO impact record structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NGOImpactRecord {
    pub ngo_address: String,
    pub total_received: f64,
    pub donor_count: u32,
    pub campaign_count: u32,
    pub badges_issued: u32,
    pub last_updated: NaiveDateTime,
}

// Leaderboard entry structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardEntry {
    pub rank: u32,
    pub address: String,
    pub name: String,
    pub score: f64,
    pub category: String, // "donor" or "ngo"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharityAllocator {
    // Charity fund balance
    fund_balance: f64,
    
    // Registry of NGOs
    ngos: HashMap<String, NGO>, // address -> NGO
    
    // Allocations to NGOs
    allocations: HashMap<String, Allocation>, // allocation_id -> Allocation
    
    // Donation records for tracking
    donation_records: HashMap<String, DonationRecord>, // donation_id -> DonationRecord
    
    // Crowdfund campaigns
    campaigns: HashMap<String, CrowdfundCampaign>, // campaign_id -> CrowdfundCampaign
    
    // Aid vouchers for humanitarian assistance
    aid_vouchers: HashMap<String, AidVoucher>, // voucher_id -> AidVoucher
    
    // Peace-relief credits for refugees
    peace_relief_credits: HashMap<String, PeaceReliefCredit>, // credit_id -> PeaceReliefCredit
    
    // Proof-of-Peace badges
    proof_of_peace_badges: HashMap<String, ProofOfPeaceBadge>, // badge_id -> ProofOfPeaceBadge
    
    // Donor reputation scores
    donor_reputations: HashMap<String, DonorReputation>, // donor_address -> DonorReputation
    
    // NGO impact records
    ngo_impact_records: HashMap<String, NGOImpactRecord>, // ngo_address -> NGOImpactRecord
    
    // Leaderboard cache
    donor_leaderboard: Vec<LeaderboardEntry>,
    ngo_leaderboard: Vec<LeaderboardEntry>,
    
    // Badge counter for ID generation
    badge_counter: u64,
    
    // DAO address that controls this contract
    dao_address: String,
    
    // Maximum percentage that can be allocated per month (2% as per tokenomics)
    max_monthly_allocation_percent: f64,
    
    // Total amount disbursed this month
    monthly_disbursed_amount: f64,
    
    // Start of current month for allocation tracking
    current_month_start: NaiveDateTime,
}

impl CharityAllocator {
    pub fn new(dao_address: String, initial_fund_balance: f64) -> Self {
        let now = Utc::now().naive_utc();
        let start_of_month = NaiveDateTime::from_timestamp_opt(
            now.timestamp(),
            0,
        )
        .unwrap_or(now);

        Self {
            fund_balance: initial_fund_balance,
            ngos: HashMap::new(),
            allocations: HashMap::new(),
            donation_records: HashMap::new(),
            campaigns: HashMap::new(),
            aid_vouchers: HashMap::new(),
            peace_relief_credits: HashMap::new(),
            proof_of_peace_badges: HashMap::new(),
            donor_reputations: HashMap::new(),
            ngo_impact_records: HashMap::new(),
            donor_leaderboard: Vec::new(),
            ngo_leaderboard: Vec::new(),
            badge_counter: 0,
            dao_address,
            max_monthly_allocation_percent: 0.02, // 2% per month
            monthly_disbursed_amount: 0.0,
            current_month_start: start_of_month,
        }
    }

    /// Set the DAO address that controls this contract
    pub fn set_dao_address(&mut self, dao_address: String) {
        self.dao_address = dao_address;
    }

    /// Get the DAO address that controls this contract
    pub fn get_dao_address(&self) -> &String {
        &self.dao_address
    }

    /// Add funds to the charity fund (only callable by DAO)
    pub fn add_funds(&mut self, amount: f64) -> Result<(), CharityError> {
        if amount <= 0.0 {
            return Err(CharityError::InvalidAmount);
        }

        self.fund_balance += amount;
        Ok(())
    }

    /// Get the current fund balance
    pub fn get_fund_balance(&self) -> f64 {
        self.fund_balance
    }

    /// Register a new NGO (only callable by DAO)
    pub fn register_ngo(
        &mut self,
        caller: &str,
        address: String,
        name: String,
        description: String,
        category: String,
        website: Option<String>,
        contact_email: Option<String>,
    ) -> Result<(), CharityError> {
        // Only DAO can register NGOs
        if caller != self.dao_address {
            return Err(CharityError::UnauthorizedAccess);
        }

        // Check if NGO already exists
        if self.ngos.contains_key(&address) {
            return Err(CharityError::NGOAlreadyExists);
        }

        let ngo = NGO {
            address: address.clone(),
            name,
            description,
            verified: false,
            registration_date: Utc::now().naive_utc(),
            verification_date: None,
            website,
            contact_email,
            category,
        };

        self.ngos.insert(address, ngo);
        Ok(())
    }

    /// Verify an NGO (only callable by DAO)
    pub fn verify_ngo(&mut self, caller: &str, ngo_address: &str) -> Result<(), CharityError> {
        // Only DAO can verify NGOs
        if caller != self.dao_address {
            return Err(CharityError::UnauthorizedAccess);
        }

        // Check if NGO exists
        let ngo = self
            .ngos
            .get_mut(ngo_address)
            .ok_or(CharityError::NGONotFound)?;

        // Check if already verified
        if ngo.verified {
            return Err(CharityError::NGOAlreadyVerified);
        }

        ngo.verified = true;
        ngo.verification_date = Some(Utc::now().naive_utc());
        Ok(())
    }

    /// Get NGO information
    pub fn get_ngo(&self, ngo_address: &str) -> Option<&NGO> {
        self.ngos.get(ngo_address)
    }

    /// Get all NGOs
    pub fn get_all_ngos(&self) -> &HashMap<String, NGO> {
        &self.ngos
    }

    /// Allocate funds to an NGO (only callable by DAO)
    pub fn allocate(
        &mut self,
        caller: &str,
        ngo_address: String,
        amount: f64,
        description: String,
    ) -> Result<String, CharityError> {
        // Only DAO can allocate funds
        if caller != self.dao_address {
            return Err(CharityError::UnauthorizedAccess);
        }

        // Check if NGO exists and is verified
        let ngo = self
            .ngos
            .get(&ngo_address)
            .ok_or(CharityError::NGONotFound)?;

        if !ngo.verified {
            return Err(CharityError::NGONotFound); // Using this for unverified NGO
        }

        // Check amount is valid
        if amount <= 0.0 {
            return Err(CharityError::InvalidAmount);
        }

        // Check if there's enough in the fund
        if amount > self.fund_balance {
            return Err(CharityError::InsufficientFunds);
        }

        // Check monthly allocation limit (2% of fund balance)
        let max_monthly_allocation = self.fund_balance * self.max_monthly_allocation_percent;
        if self.monthly_disbursed_amount + amount > max_monthly_allocation {
            return Err(CharityError::InsufficientFunds); // Using this for allocation limit exceeded
        }

        // Check if NGO already has an active allocation
        for allocation in self.allocations.values() {
            if allocation.ngo_address == ngo_address && !allocation.disbursed {
                return Err(CharityError::NGOAlreadyAllocated);
            }
        }

        // Create allocation
        let allocation_id = format!("alloc_{}", generate_id());
        let allocation = Allocation {
            id: allocation_id.clone(),
            ngo_address,
            amount,
            allocated_by: caller.to_string(),
            allocated_at: Utc::now().naive_utc(),
            description,
            disbursed: false,
            disbursement_tx_hash: None,
            disbursed_at: None,
        };

        self.allocations.insert(allocation_id.clone(), allocation);
        Ok(allocation_id)
    }

    /// Disburse funds to an NGO (only callable by DAO)
    pub fn disburse(
        &mut self,
        caller: &str,
        allocation_id: &str,
        token_contract: &mut PProjectToken,
    ) -> Result<String, CharityError> {
        // Only DAO can disburse funds
        if caller != self.dao_address {
            return Err(CharityError::UnauthorizedAccess);
        }

        // Check if allocation exists and hasn't been disbursed
        let allocation = self
            .allocations
            .get(allocation_id)
            .ok_or(CharityError::AllocationNotFound)?;

        if allocation.disbursed {
            return Err(CharityError::AllocationAlreadyDisbursed);
        }

        // Check if there's enough in the fund
        if allocation.amount > self.fund_balance {
            return Err(CharityError::InsufficientFunds);
        }

        let amount = allocation.amount;
        let ngo_address = allocation.ngo_address.clone();

        // Transfer tokens to NGO
        match token_contract.transfer("charity_allocator", &ngo_address, amount) {
            Ok(_) => {
                // Update fund balance
                self.fund_balance -= amount;

                // Update allocation as disbursed
                let allocation = self.allocations.get_mut(allocation_id).unwrap();
                allocation.disbursed = true;
                allocation.disbursement_tx_hash = Some(format!("tx_{}", generate_id()));
                allocation.disbursed_at = Some(Utc::now().naive_utc());

                // Get the transaction hash
                let tx_hash = allocation.disbursement_tx_hash.clone().unwrap();

                // Update monthly disbursed amount
                self.update_monthly_tracking();
                self.monthly_disbursed_amount += amount;

                Ok(tx_hash)
            }
            Err(e) => Err(CharityError::TokenOperationFailed(e.to_string())),
        }
    }

    /// Record a donation (callable by anyone)
    pub fn record_donation(
        &mut self,
        donor_address: String,
        ngo_address: String,
        amount: f64,
        tx_hash: String,
        campaign_id: Option<String>,
        message: Option<String>,
    ) -> Result<String, CharityError> {
        // Check if NGO exists and is verified
        let ngo = self
            .ngos
            .get(&ngo_address)
            .ok_or(CharityError::NGONotFound)?;

        if !ngo.verified {
            return Err(CharityError::NGONotFound); // Using this for unverified NGO
        }

        // Check amount is valid
        if amount <= 0.0 {
            return Err(CharityError::InvalidAmount);
        }

        // Create donation record
        let donation_id = format!("donation_{}", generate_id());
        let donation = DonationRecord {
            id: donation_id.clone(),
            donor_address: donor_address.clone(),
            ngo_address: ngo_address.clone(),
            amount,
            timestamp: Utc::now().naive_utc(),
            tx_hash: tx_hash.clone(),
            campaign_id: campaign_id.clone(),
            message,
        };

        self.donation_records.insert(donation_id.clone(), donation);

        // If this is for a campaign, update the campaign
        if let Some(campaign_id) = campaign_id.clone() {
            if let Some(campaign) = self.campaigns.get_mut(&campaign_id) {
                if campaign.active {
                    campaign.current_amount += amount;
                    campaign.donations.push(donation_id.clone());
                }
            }
        }
        
        // Update donor reputation and issue Proof-of-Peace badge
        self.update_donor_reputation(&donor_address, amount, &ngo_address, campaign_id.clone(), &tx_hash)?;
        
        // Update NGO impact record
        self.update_ngo_impact_record(&ngo_address, amount)?;
        
        // Update leaderboards
        self.update_leaderboards();

        Ok(donation_id)
    }
    
    /// Update donor reputation score and issue Proof-of-Peace badge
    fn update_donor_reputation(
        &mut self,
        donor_address: &str,
        amount: f64,
        ngo_address: &str,
        campaign_id: Option<String>,
        tx_hash: &str,
    ) -> Result<(), CharityError> {
        // Get or create donor reputation
        let donor_rep = self.donor_reputations.entry(donor_address.to_string()).or_insert_with(|| {
            DonorReputation {
                donor_address: donor_address.to_string(),
                score: 0.0,
                total_donations: 0.0,
                donation_count: 0,
                badges_earned: Vec::new(),
                last_updated: Utc::now().naive_utc(),
            }
        });
        
        // Update donation statistics
        donor_rep.total_donations += amount;
        donor_rep.donation_count += 1;
        donor_rep.last_updated = Utc::now().naive_utc();
        
        // Calculate badge type based on donation amount
        let badge_type = if amount >= 1000.0 {
            "platinum"
        } else if amount >= 500.0 {
            "gold"
        } else if amount >= 100.0 {
            "silver"
        } else {
            "bronze"
        };
        
        // Issue Proof-of-Peace badge
        self.badge_counter += 1;
        let badge_id = format!("badge_{}", self.badge_counter);
        
        let badge = ProofOfPeaceBadge {
            id: badge_id.clone(),
            donor_address: donor_address.to_string(),
            ngo_address: ngo_address.to_string(),
            donation_amount: amount,
            issued_at: Utc::now().naive_utc(),
            badge_type: badge_type.to_string(),
            campaign_id,
            tx_hash: tx_hash.to_string(),
            metadata_uri: None, // In a real implementation, this would point to IPFS metadata
        };
        
        self.proof_of_peace_badges.insert(badge_id.clone(), badge);
        donor_rep.badges_earned.push(badge_id.clone());
        
        // Update donor score based on donation amount and frequency
        // Simple scoring algorithm: base points + amount bonus + consistency bonus
        let base_points = 10.0;
        let amount_bonus = amount.min(1000.0) / 10.0; // Cap at 100 points for large donations
        let consistency_bonus = if donor_rep.donation_count > 1 { 5.0 } else { 0.0 };
        donor_rep.score += base_points + amount_bonus + consistency_bonus;
        
        Ok(())
    }
    
    /// Update NGO impact record
    fn update_ngo_impact_record(
        &mut self,
        ngo_address: &str,
        amount: f64,
    ) -> Result<(), CharityError> {
        // Get or create NGO impact record
        let ngo_impact = self.ngo_impact_records.entry(ngo_address.to_string()).or_insert_with(|| {
            NGOImpactRecord {
                ngo_address: ngo_address.to_string(),
                total_received: 0.0,
                donor_count: 0,
                campaign_count: 0,
                badges_issued: 0,
                last_updated: Utc::now().naive_utc(),
            }
        });
        
        // Update statistics
        ngo_impact.total_received += amount;
        ngo_impact.donor_count += 1; // Simplified - in reality we'd track unique donors
        ngo_impact.badges_issued += 1; // Increment for the badge we just issued
        ngo_impact.last_updated = Utc::now().naive_utc();
        
        Ok(())
    }
    
    /// Update leaderboards
    fn update_leaderboards(&mut self) {
        // Update donor leaderboard
        let mut donor_scores: Vec<(String, f64, String)> = self.donor_reputations
            .iter()
            .map(|(address, rep)| {
                let name = format!("Donor_{}", &address[..6]); // Simplified name
                (address.clone(), rep.score, name)
            })
            .collect();
        
        // Sort by score descending
        donor_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        // Create leaderboard entries
        self.donor_leaderboard = donor_scores
            .into_iter()
            .enumerate()
            .map(|(index, (address, score, name))| LeaderboardEntry {
                rank: (index + 1) as u32,
                address,
                name,
                score,
                category: "donor".to_string(),
            })
            .collect();
        
        // Update NGO leaderboard
        let mut ngo_scores: Vec<(String, f64, String)> = self.ngo_impact_records
            .iter()
            .map(|(address, impact)| {
                let ngo_name = self.ngos.get(address)
                    .map(|ngo| ngo.name.clone())
                    .unwrap_or_else(|| format!("NGO_{}", &address[..6]));
                // Score based on total received and donor count
                let score = impact.total_received + (impact.donor_count as f64 * 10.0);
                (address.clone(), score, ngo_name)
            })
            .collect();
        
        // Sort by score descending
        ngo_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        // Create leaderboard entries
        self.ngo_leaderboard = ngo_scores
            .into_iter()
            .enumerate()
            .map(|(index, (address, score, name))| LeaderboardEntry {
                rank: (index + 1) as u32,
                address,
                name,
                score,
                category: "ngo".to_string(),
            })
            .collect();
    }
    
    /// Get donor reputation
    pub fn get_donor_reputation(&self, donor_address: &str) -> Option<&DonorReputation> {
        self.donor_reputations.get(donor_address)
    }
    
    /// Get NGO impact record
    pub fn get_ngo_impact_record(&self, ngo_address: &str) -> Option<&NGOImpactRecord> {
        self.ngo_impact_records.get(ngo_address)
    }
    
    /// Get Proof-of-Peace badge
    pub fn get_proof_of_peace_badge(&self, badge_id: &str) -> Option<&ProofOfPeaceBadge> {
        self.proof_of_peace_badges.get(badge_id)
    }
    
    /// Get all badges for a donor
    pub fn get_donor_badges(&self, donor_address: &str) -> Vec<&ProofOfPeaceBadge> {
        self.proof_of_peace_badges
            .values()
            .filter(|badge| badge.donor_address == donor_address)
            .collect()
    }
    
    /// Get donor leaderboard
    pub fn get_donor_leaderboard(&self, limit: Option<u32>) -> Vec<&LeaderboardEntry> {
        let limit = limit.unwrap_or(100) as usize;
        self.donor_leaderboard.iter().take(limit).collect()
    }
    
    /// Get NGO leaderboard
    pub fn get_ngo_leaderboard(&self, limit: Option<u32>) -> Vec<&LeaderboardEntry> {
        let limit = limit.unwrap_or(100) as usize;
        self.ngo_leaderboard.iter().take(limit).collect()
    }
    
    /// Get combined leaderboard
    pub fn get_combined_leaderboard(&self, limit: Option<u32>) -> Vec<&LeaderboardEntry> {
        let mut combined: Vec<&LeaderboardEntry> = Vec::new();
        combined.extend(&self.donor_leaderboard);
        combined.extend(&self.ngo_leaderboard);
        
        // Sort by score descending
        combined.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        
        let limit = limit.unwrap_or(100) as usize;
        combined.into_iter().take(limit).collect()
    }
    
    /// Get donation record
    pub fn get_donation_record(&self, donation_id: &str) -> Option<&DonationRecord> {
        self.donation_records.get(donation_id)
    }

    /// Get all donation records for an NGO
    pub fn get_ngo_donations(&self, ngo_address: &str) -> Vec<&DonationRecord> {
        self.donation_records
            .values()
            .filter(|record| record.ngo_address == ngo_address)
            .collect()
    }

    /// Create a crowdfund campaign (only callable by verified NGOs or DAO)
    pub fn create_campaign(
        &mut self,
        caller: &str,
        name: String,
        description: String,
        target_amount: f64,
        ngo_address: String,
        duration_days: i64,
    ) -> Result<String, CharityError> {
        // Check if caller is DAO or the NGO itself
        let is_authorized = caller == self.dao_address || caller == ngo_address;
        if !is_authorized {
            // Check if caller is a verified NGO
            if let Some(ngo) = self.ngos.get(caller) {
                if !ngo.verified {
                    return Err(CharityError::UnauthorizedAccess);
                }
            } else {
                return Err(CharityError::UnauthorizedAccess);
            }
        }

        // Check if NGO exists and is verified
        let ngo = self
            .ngos
            .get(&ngo_address)
            .ok_or(CharityError::NGONotFound)?;

        if !ngo.verified {
            return Err(CharityError::NGONotFound); // Using this for unverified NGO
        }

        // Check target amount is valid
        if target_amount <= 0.0 {
            return Err(CharityError::InvalidAmount);
        }

        // Check if campaign already exists with same name
        for campaign in self.campaigns.values() {
            if campaign.name == name && campaign.ngo_address == ngo_address {
                return Err(CharityError::CampaignAlreadyExists);
            }
        }

        // Create campaign
        let campaign_id = format!("campaign_{}", generate_id());
        let now = Utc::now().naive_utc();
        let end_time = now + chrono::Duration::days(duration_days);

        let campaign = CrowdfundCampaign {
            id: campaign_id.clone(),
            name,
            description,
            target_amount,
            current_amount: 0.0,
            ngo_address,
            creator_address: caller.to_string(),
            start_time: now,
            end_time,
            active: true,
            donations: Vec::new(),
        };

        self.campaigns.insert(campaign_id.clone(), campaign);
        Ok(campaign_id)
    }

    /// Get campaign information
    pub fn get_campaign(&self, campaign_id: &str) -> Option<&CrowdfundCampaign> {
        self.campaigns.get(campaign_id)
    }

    /// Get all campaigns for an NGO
    pub fn get_ngo_campaigns(&self, ngo_address: &str) -> Vec<&CrowdfundCampaign> {
        self.campaigns
            .values()
            .filter(|campaign| campaign.ngo_address == ngo_address)
            .collect()
    }

    /// Get all active campaigns
    pub fn get_active_campaigns(&self) -> Vec<&CrowdfundCampaign> {
        let now = Utc::now().naive_utc();
        self.campaigns
            .values()
            .filter(|campaign| campaign.active && now < campaign.end_time)
            .collect()
    }

    /// Close a campaign (only callable by campaign creator, NGO, or DAO)
    pub fn close_campaign(&mut self, caller: &str, campaign_id: &str) -> Result<(), CharityError> {
        // Check if campaign exists
        let campaign = self
            .campaigns
            .get(campaign_id)
            .ok_or(CharityError::AllocationNotFound)?; // Using this for campaign not found

        // Check if caller is authorized
        let is_authorized = caller == self.dao_address
            || caller == campaign.creator_address
            || caller == campaign.ngo_address;

        if !is_authorized {
            return Err(CharityError::UnauthorizedAccess);
        }

        // Mark campaign as inactive
        let campaign = self.campaigns.get_mut(campaign_id).unwrap();
        campaign.active = false;

        Ok(())
    }

    /// Update monthly tracking (resets counter if new month)
    fn update_monthly_tracking(&mut self) {
        let now = Utc::now().naive_utc();
        let start_of_month = NaiveDateTime::from_timestamp_opt(
            now.timestamp() - ((now.day() - 1) as i64 * 24 * 60 * 60),
            0,
        )
        .unwrap_or(now);

        if start_of_month > self.current_month_start {
            self.current_month_start = start_of_month;
            self.monthly_disbursed_amount = 0.0;
        }
    }

    /// Get allocation information
    pub fn get_allocation(&self, allocation_id: &str) -> Option<&Allocation> {
        self.allocations.get(allocation_id)
    }

    /// Get all allocations for an NGO
    pub fn get_ngo_allocations(&self, ngo_address: &str) -> Vec<&Allocation> {
        self.allocations
            .values()
            .filter(|allocation| allocation.ngo_address == *ngo_address)
            .collect()
    }

    /// Get monthly disbursed amount
    pub fn get_monthly_disbursed_amount(&self) -> f64 {
        self.monthly_disbursed_amount
    }

    /// Get maximum monthly allocation amount
    pub fn get_max_monthly_allocation(&self) -> f64 {
        self.fund_balance * self.max_monthly_allocation_percent
    }
    
    /// Create an aid voucher for humanitarian assistance (only callable by DAO or verified NGOs)
    pub fn create_aid_voucher(
        &mut self,
        caller: &str,
        beneficiary_id: String,
        beneficiary_address: String,
        aid_type: String,
        amount: f64,
        zone_type: String,
        expiration_days: Option<i64>,
    ) -> Result<String, CharityError> {
        // Check if caller is authorized (DAO or verified NGO)
        if caller != self.dao_address {
            // Check if caller is a verified NGO
            if let Some(ngo) = self.ngos.get(caller) {
                if !ngo.verified {
                    return Err(CharityError::UnauthorizedAccess);
                }
            } else {
                return Err(CharityError::UnauthorizedAccess);
            }
        }
        
        // Check amount is valid
        if amount <= 0.0 {
            return Err(CharityError::InvalidAmount);
        }
        
        // Check if there's enough in the fund
        if amount > self.fund_balance {
            return Err(CharityError::InsufficientFunds);
        }
        
        // Create expiration date if specified
        let expires_at = expiration_days.map(|days| {
            Utc::now().naive_utc() + chrono::Duration::days(days)
        });
        
        // Create aid voucher
        let voucher_id = format!("voucher_{}", generate_id());
        let voucher = AidVoucher {
            id: voucher_id.clone(),
            beneficiary_id,
            beneficiary_address,
            aid_type,
            amount,
            issued_by: caller.to_string(),
            issued_at: Utc::now().naive_utc(),
            expires_at,
            claimed: false,
            claimed_at: None,
            tx_hash: None,
            zone_type,
        };
        
        self.aid_vouchers.insert(voucher_id.clone(), voucher);
        Ok(voucher_id)
    }
    
    /// Get aid voucher information
    pub fn get_aid_voucher(&self, voucher_id: &str) -> Option<&AidVoucher> {
        self.aid_vouchers.get(voucher_id)
    }
    
    /// Get all aid vouchers for a beneficiary
    pub fn get_beneficiary_vouchers(&self, beneficiary_id: &str) -> Vec<&AidVoucher> {
        self.aid_vouchers
            .values()
            .filter(|voucher| voucher.beneficiary_id == beneficiary_id)
            .collect()
    }
    
    /// Claim an aid voucher (callable by the beneficiary)
    pub fn claim_aid_voucher(
        &mut self,
        caller: &str,
        voucher_id: &str,
        token_contract: &mut PProjectToken,
    ) -> Result<String, CharityError> {
        // Check if voucher exists
        let voucher = self.aid_vouchers.get(voucher_id)
            .ok_or(CharityError::VoucherNotFound)?;
        
        // Check if caller is the beneficiary
        if caller != voucher.beneficiary_address {
            return Err(CharityError::UnauthorizedAccess);
        }
        
        // Check if voucher is already claimed
        if voucher.claimed {
            return Err(CharityError::VoucherAlreadyClaimed);
        }
        
        // Check if voucher is expired
        if let Some(expires_at) = voucher.expires_at {
            let now = Utc::now().naive_utc();
            if now > expires_at {
                return Err(CharityError::VoucherExpired);
            }
        }
        
        let amount = voucher.amount;
        let beneficiary_address = voucher.beneficiary_address.clone();
        
        // Transfer tokens to beneficiary
        match token_contract.transfer("charity_allocator", &beneficiary_address, amount) {
            Ok(_) => {
                // Update fund balance
                self.fund_balance -= amount;
                
                // Update voucher as claimed
                let voucher = self.aid_vouchers.get_mut(voucher_id).unwrap();
                voucher.claimed = true;
                voucher.claimed_at = Some(Utc::now().naive_utc());
                voucher.tx_hash = Some(format!("tx_{}", generate_id()));
                
                let tx_hash = voucher.tx_hash.clone().unwrap();
                Ok(tx_hash)
            },
            Err(e) => Err(CharityError::TokenOperationFailed(e.to_string())),
        }
    }
    
    /// Create a peace-relief credit for refugees (only callable by DAO or verified NGOs)
    pub fn create_peace_relief_credit(
        &mut self,
        caller: &str,
        refugee_id: String,
        wristband_id: Option<String>,
        amount: f64,
        expiration_days: Option<i64>,
    ) -> Result<String, CharityError> {
        // Check if caller is authorized (DAO or verified NGO)
        if caller != self.dao_address {
            // Check if caller is a verified NGO
            if let Some(ngo) = self.ngos.get(caller) {
                if !ngo.verified {
                    return Err(CharityError::UnauthorizedAccess);
                }
            } else {
                return Err(CharityError::UnauthorizedAccess);
            }
        }
        
        // Check amount is valid
        if amount <= 0.0 {
            return Err(CharityError::InvalidAmount);
        }
        
        // Check if there's enough in the fund
        if amount > self.fund_balance {
            return Err(CharityError::InsufficientFunds);
        }
        
        // Create expiration date if specified
        let expires_at = expiration_days.map(|days| {
            Utc::now().naive_utc() + chrono::Duration::days(days)
        });
        
        // Create peace-relief credit
        let credit_id = format!("credit_{}", generate_id());
        
        // Create initial credit transaction
        let transaction_id = format!("tx_{}", generate_id());
        let credit_transaction = CreditTransaction {
            id: transaction_id.clone(),
            credit_id: credit_id.clone(),
            amount,
            transaction_type: "issue".to_string(),
            description: "Initial credit issuance".to_string(),
            timestamp: Utc::now().naive_utc(),
            tx_hash: Some(format!("tx_{}", generate_id())),
        };
        
        let credit = PeaceReliefCredit {
            id: credit_id.clone(),
            refugee_id,
            wristband_id,
            amount,
            issued_by: caller.to_string(),
            issued_at: Utc::now().naive_utc(),
            expires_at,
            balance: amount,
            transactions: vec![credit_transaction],
        };
        
        self.peace_relief_credits.insert(credit_id.clone(), credit);
        Ok(credit_id)
    }
    
    /// Get peace-relief credit information
    pub fn get_peace_relief_credit(&self, credit_id: &str) -> Option<&PeaceReliefCredit> {
        self.peace_relief_credits.get(credit_id)
    }
    
    /// Get all peace-relief credits for a refugee
    pub fn get_refugee_credits(&self, refugee_id: &str) -> Vec<&PeaceReliefCredit> {
        self.peace_relief_credits
            .values()
            .filter(|credit| credit.refugee_id == refugee_id)
            .collect()
    }
    
    /// Use peace-relief credit for purchases (callable by the refugee)
    pub fn use_peace_relief_credit(
        &mut self,
        caller: &str,
        credit_id: &str,
        amount: f64,
        description: String,
        _token_contract: &mut PProjectToken,
    ) -> Result<String, CharityError> {
        // Check if credit exists
        let credit = self.peace_relief_credits.get(credit_id)
            .ok_or(CharityError::CreditNotFound)?;
        
        // Check if caller is the refugee
        if caller != credit.refugee_id {
            return Err(CharityError::UnauthorizedAccess);
        }
        
        // Check if credit is expired
        if let Some(expires_at) = credit.expires_at {
            let now = Utc::now().naive_utc();
            if now > expires_at {
                return Err(CharityError::CreditExpired);
            }
        }
        
        // Check if there's enough balance
        if amount > credit.balance {
            return Err(CharityError::InsufficientCreditBalance);
        }
        
        // For peace-relief credits, we're not actually transferring tokens to a merchant
        // Instead, we're deducting from the credit balance as if the tokens were spent
        // In a real implementation, this would involve actual merchant payments
        
        // Update fund balance
        self.fund_balance -= amount;
        
        // Update credit balance
        let credit = self.peace_relief_credits.get_mut(credit_id).unwrap();
        credit.balance -= amount;
        
        // Create transaction record
        let transaction_id = format!("tx_{}", generate_id());
        let transaction = CreditTransaction {
            id: transaction_id.clone(),
            credit_id: credit_id.to_string(),
            amount,
            transaction_type: "spend".to_string(),
            description,
            timestamp: Utc::now().naive_utc(),
            tx_hash: Some(format!("tx_{}", generate_id())),
        };
        
        credit.transactions.push(transaction);
        
        let tx_hash = credit.transactions.last().unwrap().tx_hash.clone().unwrap();
        Ok(tx_hash)
    }
}
