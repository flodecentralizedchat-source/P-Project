use crate::charity::{CharityAllocator, CharityError};
use chrono::{Duration, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Errors for the savings vault workflow
#[derive(Debug, Clone, PartialEq)]
pub enum SavingsError {
    InvalidAmount,
    InvalidDuration,
    VaultAlreadyExists,
    PositionNotFound,
    PrematureWithdrawal(NaiveDateTime),
    InsufficientRewardReserve,
    InvalidCharityShare,
}

impl fmt::Display for SavingsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SavingsError::InvalidAmount => write!(f, "Amount must be positive"),
            SavingsError::InvalidDuration => {
                write!(f, "Duration must be at least the minimum lock")
            }
            SavingsError::VaultAlreadyExists => write!(f, "Vault already exists"),
            SavingsError::PositionNotFound => write!(f, "User position not found"),
            SavingsError::PrematureWithdrawal(date) => write!(
                f,
                "Position matures at {} and cannot be withdrawn early",
                date
            ),
            SavingsError::InsufficientRewardReserve => {
                write!(f, "Vault reward reserve is insufficient to honor interest")
            }
            SavingsError::InvalidCharityShare => {
                write!(f, "Charity share must be between 0% and 100%")
            }
        }
    }
}

impl std::error::Error for SavingsError {}

/// Vault configuration that describes the savings product
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavingsConfig {
    pub vault_id: String,
    pub apr: f64,
    pub charity_share_percent: f64, // share of earned interest reserved for NGOs (0.0-1.0)
    pub min_duration_days: i64,
    pub reward_reserve: f64, // tokens set aside to pay interest
}

/// Individual user position inside a vault
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultPosition {
    pub user_id: String,
    pub vault_id: String,
    pub principal: f64,
    pub duration_days: i64,
    pub deposit_time: NaiveDateTime,
}

/// Single savings vault implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavingsVault {
    pub config: SavingsConfig,
    pub total_principal: f64,
    pub total_accrued_interest: f64,
    pub charity_yield_pool: f64,
    pub positions: HashMap<String, VaultPosition>,
}

impl SavingsVault {
    pub fn new(
        vault_id: String,
        apr: f64,
        charity_share_percent: f64,
        min_duration_days: i64,
        reward_reserve: f64,
    ) -> Result<Self, SavingsError> {
        if apr < 0.0 {
            return Err(SavingsError::InvalidAmount);
        }

        if !(0.0..=1.0).contains(&charity_share_percent) {
            return Err(SavingsError::InvalidCharityShare);
        }

        if min_duration_days <= 0 {
            return Err(SavingsError::InvalidDuration);
        }

        Ok(Self {
            config: SavingsConfig {
                vault_id,
                apr,
                charity_share_percent,
                min_duration_days,
                reward_reserve,
            },
            total_principal: 0.0,
            total_accrued_interest: 0.0,
            charity_yield_pool: 0.0,
            positions: HashMap::new(),
        })
    }

    /// Adds tokens to the reward reserve so interest can be paid later
    pub fn fund_reward_reserve(&mut self, amount: f64) -> Result<(), SavingsError> {
        if amount <= 0.0 {
            return Err(SavingsError::InvalidAmount);
        }

        self.config.reward_reserve += amount;
        Ok(())
    }

    /// Deposit P-Coin into the vault; duration must meet min lock
    pub fn deposit(
        &mut self,
        user_id: String,
        amount: f64,
        duration_days: i64,
    ) -> Result<(), SavingsError> {
        if amount <= 0.0 {
            return Err(SavingsError::InvalidAmount);
        }

        if duration_days < self.config.min_duration_days {
            return Err(SavingsError::InvalidDuration);
        }

        let start_time = Utc::now().naive_utc();
        let position = VaultPosition {
            user_id: user_id.clone(),
            vault_id: self.config.vault_id.clone(),
            principal: amount,
            duration_days,
            deposit_time: start_time,
        };

        self.positions.insert(user_id, position);
        self.total_principal += amount;
        Ok(())
    }

    /// Advances the deposit timestamp, allowing tests or simulations to reach maturity quickly.
    pub fn fast_forward_position(
        &mut self,
        user_id: &str,
        duration: Duration,
    ) -> Result<(), SavingsError> {
        if let Some(position) = self.positions.get_mut(user_id) {
            position.deposit_time -= duration;
            Ok(())
        } else {
            Err(SavingsError::PositionNotFound)
        }
    }

    /// Calculates simple interest for a provided principal and duration (days)
    pub fn calculate_interest(&self, principal: f64, duration_days: f64) -> f64 {
        principal * self.config.apr * (duration_days / 365.0)
    }

    /// Withdraw principal and interest once the lock period has finished.
    /// Returns (principal, interest_paid_to_user, charity_share).
    pub fn withdraw(&mut self, user_id: &str) -> Result<(f64, f64, f64), SavingsError> {
        let position = self
            .positions
            .remove(user_id)
            .ok_or(SavingsError::PositionNotFound)?;

        let maturity_time = position.deposit_time + Duration::days(position.duration_days);
        let now = Utc::now().naive_utc();
        if now < maturity_time {
            self.positions.insert(user_id.to_string(), position.clone());
            return Err(SavingsError::PrematureWithdrawal(maturity_time));
        }

        let interest = self.calculate_interest(position.principal, position.duration_days as f64);
        if interest > self.config.reward_reserve {
            self.positions.insert(user_id.to_string(), position.clone());
            return Err(SavingsError::InsufficientRewardReserve);
        }

        let charity_share = interest * self.config.charity_share_percent;
        let user_interest = interest - charity_share;

        self.config.reward_reserve -= interest;
        self.total_accrued_interest += interest;
        self.charity_yield_pool += charity_share;
        self.total_principal -= position.principal;

        Ok((position.principal, user_interest, charity_share))
    }

    /// Sends the pooled charity yield to the registered charity allocator
    pub fn claim_charity_yield(
        &mut self,
        allocator: &mut CharityAllocator,
    ) -> Result<f64, CharityError> {
        if self.charity_yield_pool <= 0.0 {
            return Ok(0.0);
        }

        allocator.add_funds(self.charity_yield_pool)?;
        let distributed = self.charity_yield_pool;
        self.charity_yield_pool = 0.0;
        Ok(distributed)
    }

    /// Get position for a user without removing it
    pub fn get_position(&self, user_id: &str) -> Option<&VaultPosition> {
        self.positions.get(user_id)
    }
}
