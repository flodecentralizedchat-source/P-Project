use chrono::{Duration, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Configuration for the Credit & Micro-Loans service.
#[derive(Debug, Clone)]
pub struct CreditServiceConfig {
    pub min_credit_score: f64,
    pub max_loan_amount: f64,
    pub collateral_ratio: f64,
    pub default_interest_rate: f64,
    pub base_score: f64,
    pub max_duration_days: u32,
}

/// Social impact event used to boost credit profiles.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialImpactEvent {
    pub id: String,
    pub description: String,
    pub impact_score: f64,
    pub verified_by: Option<String>,
    pub timestamp: NaiveDateTime,
}

/// Credit score profile derived from verified social impact events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditScoreProfile {
    pub user_id: String,
    pub score: f64,
    pub events: Vec<SocialImpactEvent>,
    pub updated_at: NaiveDateTime,
}

/// Input data for registering NGO partners.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NGORegistration {
    pub name: String,
    pub region: String,
    pub max_loan_amount: f64,
    pub approved: bool,
}

/// Internal representation of an NGO partner.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NGOProfile {
    pub id: String,
    pub name: String,
    pub region: String,
    pub approved: bool,
    pub max_loan_amount: f64,
    pub created_at: NaiveDateTime,
}

/// Loan lifecycle state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum LoanStatus {
    Pending,
    Active,
    Repaid,
    Defaulted,
}

/// Represents a P-Coin collateral micro loan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicroLoan {
    pub loan_id: String,
    pub borrower_id: String,
    pub ngo_id: String,
    pub principal_amount: f64,
    pub collateral_amount: f64,
    pub interest_rate: f64,
    pub due_date: NaiveDateTime,
    pub status: LoanStatus,
    pub created_at: NaiveDateTime,
    pub repaid_amount: f64,
}

impl MicroLoan {
    pub fn total_due(&self) -> f64 {
        self.principal_amount * (1.0 + self.interest_rate)
    }
}

/// Service that manages micro-loans, credit profiles and NGO partners.
pub struct CreditService {
    config: CreditServiceConfig,
    ngos: HashMap<String, NGOProfile>,
    loans: HashMap<String, MicroLoan>,
    credit_profiles: HashMap<String, CreditScoreProfile>,
}

impl CreditService {
    /// Create a new credit service instance.
    pub fn new(config: CreditServiceConfig) -> Self {
        Self {
            config,
            ngos: HashMap::new(),
            loans: HashMap::new(),
            credit_profiles: HashMap::new(),
        }
    }

    fn ensure_profile(&mut self, user_id: &str) -> &mut CreditScoreProfile {
        let now = Utc::now().naive_utc();
        self.credit_profiles
            .entry(user_id.to_string())
            .or_insert_with(|| CreditScoreProfile {
                user_id: user_id.to_string(),
                score: self.config.base_score,
                events: vec![],
                updated_at: now,
            })
    }

    fn recompute_score(&self, profile: &CreditScoreProfile) -> f64 {
        let event_score: f64 = profile.events.iter().map(|event| event.impact_score).sum();
        let capped = (self.config.base_score + event_score).max(0.0).min(100.0);
        (capped * 100.0).round() / 100.0
    }

    /// Register an NGO partner that can back micro-loans.
    pub fn register_ngo(&mut self, registration: NGORegistration) -> NGOProfile {
        let ngo_id = format!("ngo_{}", Uuid::new_v4());
        let profile = NGOProfile {
            id: ngo_id.clone(),
            name: registration.name,
            region: registration.region,
            approved: registration.approved,
            max_loan_amount: registration.max_loan_amount,
            created_at: Utc::now().naive_utc(),
        };
        self.ngos.insert(ngo_id.clone(), profile.clone());
        profile
    }

    /// Add a verified social impact event to a user's credit profile.
    pub fn add_social_impact_event(
        &mut self,
        user_id: &str,
        event: SocialImpactEvent,
    ) -> Result<f64, Box<dyn std::error::Error>> {
        let base_score = self.config.base_score;
        let now = Utc::now().naive_utc();
        let profile = self.ensure_profile(user_id);
        profile.events.push(event);
        // Recompute score without immutably borrowing self
        let event_score: f64 = profile.events.iter().map(|e| e.impact_score).sum();
        let capped = (base_score + event_score).max(0.0).min(100.0);
        profile.score = (capped * 100.0).round() / 100.0;
        profile.updated_at = now;
        Ok(profile.score)
    }

    /// Get the current credit score for a user.
    pub fn get_credit_score(&mut self, user_id: &str) -> f64 {
        self.ensure_profile(user_id).score
    }

    /// Request a new micro-loan backed by P-Coin collateral.
    pub fn request_micro_loan(
        &mut self,
        borrower_id: &str,
        amount: f64,
        collateral_amount: f64,
        ngo_id: &str,
    ) -> Result<MicroLoan, Box<dyn std::error::Error>> {
        if amount <= 0.0 {
            return Err("Loan amount must be greater than zero".into());
        }
        if collateral_amount <= 0.0 {
            return Err("Collateral amount must be greater than zero".into());
        }

        let ngo = self.ngos.get(ngo_id).ok_or("NGO not found")?;
        if !ngo.approved {
            return Err("NGO is not approved to back loans".into());
        }
        if amount > ngo.max_loan_amount {
            return Err("Requested amount exceeds NGO limit".into());
        }
        if amount > self.config.max_loan_amount {
            return Err("Requested amount exceeds service limit".into());
        }

        let profile = self.ensure_profile(borrower_id);
        if profile.score < self.config.min_credit_score {
            return Err("Credit score below minimum threshold".into());
        }

        let required_collateral = amount * self.config.collateral_ratio;
        if collateral_amount < required_collateral {
            return Err("Collateral amount is insufficient".into());
        }

        let loan_id = format!("loan_{}", Uuid::new_v4());
        let now = Utc::now().naive_utc();
        let due_date =
            (Utc::now() + Duration::days(self.config.max_duration_days as i64)).naive_utc();

        let loan = MicroLoan {
            loan_id: loan_id.clone(),
            borrower_id: borrower_id.to_string(),
            ngo_id: ngo_id.to_string(),
            principal_amount: amount,
            collateral_amount,
            interest_rate: self.config.default_interest_rate,
            due_date,
            status: LoanStatus::Active,
            created_at: now,
            repaid_amount: 0.0,
        };

        self.loans.insert(loan_id.clone(), loan.clone());
        Ok(loan)
    }

    /// Repay an active micro-loan.
    pub fn repay_micro_loan(
        &mut self,
        loan_id: &str,
        amount: f64,
    ) -> Result<MicroLoan, Box<dyn std::error::Error>> {
        let loan = self.loans.get_mut(loan_id).ok_or("Loan not found")?;
        if loan.status != LoanStatus::Active {
            return Err("Loan is not active".into());
        }
        if amount <= 0.0 {
            return Err("Repayment amount must be positive".into());
        }

        loan.repaid_amount += amount;
        if loan.repaid_amount >= loan.total_due() {
            loan.repaid_amount = loan.total_due();
            loan.status = LoanStatus::Repaid;
        }

        Ok(loan.clone())
    }

    /// Get a micro-loan by its ID.
    pub fn get_micro_loan(&self, loan_id: &str) -> Option<&MicroLoan> {
        self.loans.get(loan_id)
    }

    /// Get all loans for a borrower.
    pub fn get_loans_by_borrower(&self, borrower_id: &str) -> Vec<&MicroLoan> {
        self.loans
            .values()
            .filter(|loan| loan.borrower_id == borrower_id)
            .collect()
    }
}
