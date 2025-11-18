use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

// Custom error types for vesting operations
#[derive(Debug, Clone, PartialEq)]
pub enum VestingError {
    InsufficientTokens,
    NoVestingSchedule,
    VestingNotStarted,
    VestingAlreadyClaimed,
    UnauthorizedAction(String),
    DatabaseError(String),
    SerializationError(String),
}

impl std::fmt::Display for VestingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VestingError::InsufficientTokens => write!(f, "Not enough tokens for vesting"),
            VestingError::NoVestingSchedule => write!(f, "No vesting schedule found for user"),
            VestingError::VestingNotStarted => write!(f, "Vesting has not started yet"),
            VestingError::VestingAlreadyClaimed => write!(f, "Vesting tokens already claimed"),
            VestingError::UnauthorizedAction(msg) => write!(f, "Unauthorized action: {}", msg),
            VestingError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            VestingError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
        }
    }
}

impl std::error::Error for VestingError {}

/// Supported release mechanisms for vesting schedules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReleaseMechanism {
    Linear,
    PerBlock {
        block_time_seconds: f64,
        tokens_per_block: f64,
    },
    PerEpoch {
        epoch_seconds: i64,
        tokens_per_epoch: f64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VestingSchedule {
    pub user_id: String,
    pub total_amount: f64,
    pub cliff_duration_months: i64,
    pub vesting_duration_months: i64,
    pub start_date: NaiveDateTime,
    pub claimed_amount: f64,
    pub is_linear: bool,            // Linear vesting or other schedule
    pub schedule_type: VestingType, // Type of vesting schedule
    pub release_mechanism: ReleaseMechanism,
    pub circuit_breaker_engaged: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VestingType {
    Founder,
    FoundingMember,
    Team,
    Advisor,
    Investor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VestingContract {
    schedules: HashMap<String, VestingSchedule>,
    total_vesting_tokens: f64,
    claimed_tokens: f64,
    // Multisig settings for founder actions
    founder_multisig_signers: Vec<String>,
    founder_multisig_required: usize,
}

impl VestingContract {
    pub fn new(total_vesting_tokens: f64) -> Self {
        Self {
            schedules: HashMap::new(),
            total_vesting_tokens,
            claimed_tokens: 0.0,
            founder_multisig_signers: vec![
                "founder1".to_string(),
                "founder2".to_string(),
                "legal_representative".to_string(),
            ],
            founder_multisig_required: 2,
        }
    }

    /// Set multisig signers for founder wallets
    pub fn set_founder_multisig_signers(&mut self, signers: Vec<String>, required: usize) {
        self.founder_multisig_signers = signers;
        self.founder_multisig_required = required;
    }

    /// Get multisig signers for founder wallets
    pub fn get_founder_multisig_signers(&self) -> (&Vec<String>, usize) {
        (
            &self.founder_multisig_signers,
            self.founder_multisig_required,
        )
    }

    /// Check if a signer is authorized for founder actions
    pub fn is_founder_signer(&self, signer: &str) -> bool {
        self.founder_multisig_signers.contains(&signer.to_string())
    }

    fn months_to_seconds(months: i64) -> i64 {
        months * 30 * 24 * 60 * 60
    }

    fn register_schedule(
        &mut self,
        user_id: String,
        amount: f64,
        start_date: NaiveDateTime,
        cliff_months: i64,
        vesting_months: i64,
        release_mechanism: ReleaseMechanism,
        schedule_type: VestingType,
    ) -> Result<(), VestingError> {
        if self.claimed_tokens + amount > self.total_vesting_tokens {
            return Err(VestingError::InsufficientTokens);
        }

        let schedule = VestingSchedule {
            user_id: user_id.clone(),
            total_amount: amount,
            cliff_duration_months: cliff_months,
            vesting_duration_months: vesting_months,
            start_date,
            claimed_amount: 0.0,
            is_linear: matches!(release_mechanism, ReleaseMechanism::Linear),
            schedule_type,
            release_mechanism,
            circuit_breaker_engaged: false,
        };

        self.schedules.insert(user_id, schedule);
        Ok(())
    }

    /// Create a vesting schedule for founders (4-year linear vesting, 1-year cliff)
    pub fn create_founder_vesting(
        &mut self,
        user_id: String,
        amount: f64,
        start_date: NaiveDateTime,
    ) -> Result<(), VestingError> {
        self.register_schedule(
            user_id,
            amount,
            start_date,
            12,
            48,
            ReleaseMechanism::Linear,
            VestingType::Founder,
        )
    }

    /// Create a vesting schedule for founding members (3-year linear vesting, 6-month cliff)
    pub fn create_founding_member_vesting(
        &mut self,
        user_id: String,
        amount: f64,
        start_date: NaiveDateTime,
    ) -> Result<(), VestingError> {
        self.register_schedule(
            user_id,
            amount,
            start_date,
            6,
            36,
            ReleaseMechanism::Linear,
            VestingType::FoundingMember,
        )
    }

    /// Create a vesting schedule for team members (12m cliff + 24m linear)
    pub fn create_team_vesting(
        &mut self,
        user_id: String,
        amount: f64,
        start_date: NaiveDateTime,
    ) -> Result<(), VestingError> {
        self.register_schedule(
            user_id,
            amount,
            start_date,
            12,
            24,
            ReleaseMechanism::Linear,
            VestingType::Team,
        )
    }

    /// Create a vesting schedule for advisors (6m cliff + 12m linear)
    pub fn create_advisor_vesting(
        &mut self,
        user_id: String,
        amount: f64,
        start_date: NaiveDateTime,
    ) -> Result<(), VestingError> {
        self.register_schedule(
            user_id,
            amount,
            start_date,
            6,
            12,
            ReleaseMechanism::Linear,
            VestingType::Advisor,
        )
    }

    /// Create a vesting schedule for seed round investors (18-month linear vesting, 3-month cliff)
    pub fn create_seed_investor_vesting(
        &mut self,
        user_id: String,
        amount: f64,
        start_date: NaiveDateTime,
    ) -> Result<(), VestingError> {
        self.register_schedule(
            user_id,
            amount,
            start_date,
            3,
            18,
            ReleaseMechanism::Linear,
            VestingType::Investor,
        )
    }

    /// Create a vesting schedule for private round investors (24-month linear vesting, 6-month cliff)
    pub fn create_private_investor_vesting(
        &mut self,
        user_id: String,
        amount: f64,
        start_date: NaiveDateTime,
    ) -> Result<(), VestingError> {
        self.register_schedule(
            user_id,
            amount,
            start_date,
            6,
            24,
            ReleaseMechanism::Linear,
            VestingType::Investor,
        )
    }

    /// Create a public sale vesting schedule (12-month linear vesting, no cliff)
    pub fn create_public_sale_vesting(
        &mut self,
        user_id: String,
        amount: f64,
        start_date: NaiveDateTime,
    ) -> Result<(), VestingError> {
        self.register_schedule(
            user_id,
            amount,
            start_date,
            0,
            12,
            ReleaseMechanism::Linear,
            VestingType::Investor,
        )
    }

    /// Create a per-block emission vesting schedule
    pub fn create_block_emission_vesting(
        &mut self,
        user_id: String,
        amount: f64,
        start_date: NaiveDateTime,
        cliff_months: i64,
        block_time_seconds: f64,
        tokens_per_block: f64,
        schedule_type: VestingType,
    ) -> Result<(), VestingError> {
        self.register_schedule(
            user_id,
            amount,
            start_date,
            cliff_months,
            0,
            ReleaseMechanism::PerBlock {
                block_time_seconds,
                tokens_per_block,
            },
            schedule_type,
        )
    }

    /// Create a per-epoch unlock vesting schedule
    pub fn create_epoch_unlock_vesting(
        &mut self,
        user_id: String,
        amount: f64,
        start_date: NaiveDateTime,
        cliff_months: i64,
        epoch_seconds: i64,
        tokens_per_epoch: f64,
        schedule_type: VestingType,
    ) -> Result<(), VestingError> {
        self.register_schedule(
            user_id,
            amount,
            start_date,
            cliff_months,
            0,
            ReleaseMechanism::PerEpoch {
                epoch_seconds,
                tokens_per_epoch,
            },
            schedule_type,
        )
    }

    /// Toggle the anti-dump circuit breaker on team schedules
    pub fn toggle_team_circuit_breaker(
        &mut self,
        user_id: &str,
        signers: Vec<String>,
        engage: bool,
    ) -> Result<(), VestingError> {
        let unique_signers: HashSet<_> = signers.into_iter().collect();
        if unique_signers.len() < self.founder_multisig_required {
            return Err(VestingError::UnauthorizedAction(
                "insufficient multisig approvals".to_string(),
            ));
        }

        for signer in &unique_signers {
            if !self.is_founder_signer(signer) {
                return Err(VestingError::UnauthorizedAction(format!(
                    "{} is not authorized",
                    signer
                )));
            }
        }

        if let Some(schedule) = self.schedules.get_mut(user_id) {
            if schedule.schedule_type != VestingType::Team {
                return Err(VestingError::UnauthorizedAction(
                    "circuit breaker reserved for team wallets".to_string(),
                ));
            }
            schedule.circuit_breaker_engaged = engage;
            Ok(())
        } else {
            Err(VestingError::NoVestingSchedule)
        }
    }

    /// Calculate the amount of tokens that can be claimed by a user
    pub fn calculate_claimable_amount(&self, user_id: &str) -> Result<f64, VestingError> {
        let schedule = self
            .schedules
            .get(user_id)
            .ok_or(VestingError::NoVestingSchedule)?;

        let now = Utc::now().naive_utc();

        if now < schedule.start_date {
            return Err(VestingError::VestingNotStarted);
        }

        if schedule.circuit_breaker_engaged {
            return Ok(0.0);
        }

        let elapsed_secs = (now - schedule.start_date).num_seconds().max(0);
        let cliff_secs = Self::months_to_seconds(schedule.cliff_duration_months);

        if elapsed_secs < cliff_secs {
            return Ok(0.0);
        }

        let release_secs = elapsed_secs - cliff_secs;
        let vested = match &schedule.release_mechanism {
            ReleaseMechanism::Linear => {
                let total_secs = Self::months_to_seconds(schedule.vesting_duration_months);
                if total_secs <= 0 {
                    schedule.total_amount
                } else {
                    let ratio = (release_secs as f64 / total_secs as f64).min(1.0);
                    schedule.total_amount * ratio
                }
            }
            ReleaseMechanism::PerBlock {
                block_time_seconds,
                tokens_per_block,
            } => {
                if *block_time_seconds <= 0.0 || *tokens_per_block <= 0.0 {
                    schedule.total_amount
                } else {
                    let blocks = (release_secs as f64 / block_time_seconds).floor();
                    (blocks * tokens_per_block).max(0.0)
                }
            }
            ReleaseMechanism::PerEpoch {
                epoch_seconds,
                tokens_per_epoch,
            } => {
                if *epoch_seconds <= 0 || *tokens_per_epoch <= 0.0 {
                    schedule.total_amount
                } else {
                    let epochs = release_secs / epoch_seconds;
                    (epochs as f64 * tokens_per_epoch).max(0.0)
                }
            }
        };

        let fully_vested = vested.min(schedule.total_amount);
        let remaining = (schedule.total_amount - schedule.claimed_amount).max(0.0);
        let claimable = (fully_vested - schedule.claimed_amount)
            .max(0.0)
            .min(remaining);
        Ok(claimable)
    }

    /// Claim vested tokens
    pub fn claim_vested_tokens(&mut self, user_id: &str) -> Result<f64, VestingError> {
        let claimable_amount = self.calculate_claimable_amount(user_id)?;

        if claimable_amount <= 0.0 {
            return Ok(0.0);
        }

        if let Some(schedule) = self.schedules.get_mut(user_id) {
            schedule.claimed_amount += claimable_amount;
            self.claimed_tokens += claimable_amount;
            Ok(claimable_amount)
        } else {
            Err(VestingError::NoVestingSchedule)
        }
    }

    /// Get vesting schedule for a user
    pub fn get_vesting_schedule(&self, user_id: &str) -> Option<&VestingSchedule> {
        self.schedules.get(user_id)
    }

    /// Get total vesting tokens
    pub fn get_total_vesting_tokens(&self) -> f64 {
        self.total_vesting_tokens
    }

    /// Get claimed tokens
    pub fn get_claimed_tokens(&self) -> f64 {
        self.claimed_tokens
    }
}
