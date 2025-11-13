use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Custom error types for vesting operations
#[derive(Debug, Clone, PartialEq)]
pub enum VestingError {
    InsufficientTokens,
    NoVestingSchedule,
    VestingNotStarted,
    VestingAlreadyClaimed,
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
            VestingError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            VestingError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
        }
    }
}

impl std::error::Error for VestingError {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VestingSchedule {
    pub user_id: String,
    pub total_amount: f64,
    pub cliff_duration_months: i64,
    pub vesting_duration_months: i64,
    pub start_date: NaiveDateTime,
    pub claimed_amount: f64,
    pub is_linear: bool, // Linear vesting or other schedule
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VestingContract {
    schedules: HashMap<String, VestingSchedule>,
    total_vesting_tokens: f64,
    claimed_tokens: f64,
}

impl VestingContract {
    pub fn new(total_vesting_tokens: f64) -> Self {
        Self {
            schedules: HashMap::new(),
            total_vesting_tokens,
            claimed_tokens: 0.0,
        }
    }

    /// Create a vesting schedule for team members (12m cliff + 24m linear)
    pub fn create_team_vesting(
        &mut self,
        user_id: String,
        amount: f64,
        start_date: NaiveDateTime,
    ) -> Result<(), VestingError> {
        if self.claimed_tokens + amount > self.total_vesting_tokens {
            return Err(VestingError::InsufficientTokens);
        }

        let schedule = VestingSchedule {
            user_id: user_id.clone(),
            total_amount: amount,
            cliff_duration_months: 12, // 12 months cliff
            vesting_duration_months: 24, // 24 months linear vesting after cliff
            start_date,
            claimed_amount: 0.0,
            is_linear: true,
        };

        self.schedules.insert(user_id, schedule);
        Ok(())
    }

    /// Create a vesting schedule for advisors (6m cliff + 12m linear)
    pub fn create_advisor_vesting(
        &mut self,
        user_id: String,
        amount: f64,
        start_date: NaiveDateTime,
    ) -> Result<(), VestingError> {
        if self.claimed_tokens + amount > self.total_vesting_tokens {
            return Err(VestingError::InsufficientTokens);
        }

        let schedule = VestingSchedule {
            user_id: user_id.clone(),
            total_amount: amount,
            cliff_duration_months: 6, // 6 months cliff
            vesting_duration_months: 12, // 12 months linear vesting after cliff
            start_date,
            claimed_amount: 0.0,
            is_linear: true,
        };

        self.schedules.insert(user_id, schedule);
        Ok(())
    }

    /// Calculate the amount of tokens that can be claimed by a user
    pub fn calculate_claimable_amount(&self, user_id: &str) -> Result<f64, VestingError> {
        let schedule = self
            .schedules
            .get(user_id)
            .ok_or(VestingError::NoVestingSchedule)?;

        let now = Utc::now().naive_utc();
        
        // Check if vesting has started
        if now < schedule.start_date {
            return Err(VestingError::VestingNotStarted);
        }

        // Calculate time elapsed since start
        let elapsed_duration = now - schedule.start_date;
        let elapsed_months = elapsed_duration.num_days() / 30; // Approximate months

        // Check if cliff period has passed
        if elapsed_months < schedule.cliff_duration_months {
            return Ok(0.0); // Nothing claimable during cliff
        }

        // Calculate vested amount after cliff
        let months_after_cliff = elapsed_months - schedule.cliff_duration_months;
        
        if months_after_cliff >= schedule.vesting_duration_months {
            // Fully vested
            Ok(schedule.total_amount - schedule.claimed_amount)
        } else if schedule.is_linear {
            // Linear vesting
            let vested_percentage = months_after_cliff as f64 / schedule.vesting_duration_months as f64;
            let vested_amount = schedule.total_amount * vested_percentage;
            Ok(vested_amount - schedule.claimed_amount)
        } else {
            // For other vesting schedules, return 0 for now
            Ok(0.0)
        }
    }

    /// Claim vested tokens
    pub fn claim_vested_tokens(&mut self, user_id: &str) -> Result<f64, VestingError> {
        let claimable_amount = self.calculate_claimable_amount(user_id)?;
        
        if claimable_amount <= 0.0 {
            return Ok(0.0);
        }

        // Update schedule
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