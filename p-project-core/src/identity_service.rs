//! Identity & Reputation service for peace-building participants.
//! Implements Peace Passport soulbound NFTs, humanitarian profiles, and verified volunteer hours.

use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct IdentityServiceConfig {
    pub currency: String,
    pub score_per_contribution: f64,
    pub score_per_volunteer_hour: f64,
}

impl Default for IdentityServiceConfig {
    fn default() -> Self {
        Self {
            currency: "P".into(),
            score_per_contribution: 0.1,
            score_per_volunteer_hour: 0.5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanitarianProfile {
    pub id: String,
    pub name: String,
    pub wallet_address: String,
    pub organization: Option<String>,
    pub bio: Option<String>,
    pub created_at: NaiveDateTime,
    pub contributions_total: f64,
    pub volunteer_hours_total: f64,
    pub humanitarian_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanitarianContribution {
    pub id: String,
    pub profile_id: String,
    pub amount: f64,
    pub currency: String,
    pub description: String,
    pub kind: ContributionKind,
    pub verified: bool,
    pub timestamp: NaiveDateTime,
    pub tx_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ContributionKind {
    Financial,
    Material,
    Advocacy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeacePassport {
    pub id: String,
    pub profile_id: String,
    pub wallet_address: String,
    pub token_uri: String,
    pub metadata: HashMap<String, String>,
    pub minted_at: NaiveDateTime,
    pub verified_on_chain: bool,
    pub verification_tx: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum VolunteerHourStatus {
    Logged,
    Verified,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolunteerHourEntry {
    pub id: String,
    pub profile_id: String,
    pub description: String,
    pub hours: f64,
    pub location: Option<String>,
    pub status: VolunteerHourStatus,
    pub timestamp: NaiveDateTime,
    pub tx_hash: Option<String>,
}

pub struct IdentityService {
    pub config: IdentityServiceConfig,
    pub profiles: HashMap<String, HumanitarianProfile>,
    pub contributions: HashMap<String, HumanitarianContribution>,
    pub passports: HashMap<String, PeacePassport>,
    pub volunteer_hours: HashMap<String, VolunteerHourEntry>,
}

impl IdentityService {
    pub fn new(config: IdentityServiceConfig) -> Self {
        Self {
            config,
            profiles: HashMap::new(),
            contributions: HashMap::new(),
            passports: HashMap::new(),
            volunteer_hours: HashMap::new(),
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

    fn profile_score_increment(&self, amount: f64, hours: f64) -> f64 {
        self.config.score_per_contribution * amount + self.config.score_per_volunteer_hour * hours
    }

    pub fn register_profile(
        &mut self,
        name: String,
        wallet_address: String,
        organization: Option<String>,
        bio: Option<String>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let id = format!("profile_{}", Uuid::new_v4());
        let now = self.now();
        let profile = HumanitarianProfile {
            id: id.clone(),
            name,
            wallet_address,
            organization,
            bio,
            created_at: now,
            contributions_total: 0.0,
            volunteer_hours_total: 0.0,
            humanitarian_score: 0.0,
        };
        self.profiles.insert(id.clone(), profile);
        Ok(id)
    }

    pub fn record_contribution(
        &mut self,
        profile_id: String,
        amount: f64,
        currency: String,
        description: String,
        kind: ContributionKind,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // First, validate the inputs and get necessary data
        self.ensure_currency(&currency)?;
        if amount <= 0.0 {
            return Err("Amount must be positive".into());
        }

        // Get the score increment value before getting a mutable reference to the profile
        let score_increment = self.profile_score_increment(amount, 0.0);

        // Now get a mutable reference to update the profile
        let now = self.now();
        let profile = self
            .profiles
            .get_mut(&profile_id)
            .ok_or("Profile not found")?;
        profile.contributions_total += amount;
        profile.humanitarian_score += score_increment;

        let id = format!("contrib_{}", Uuid::new_v4());
        let contribution = HumanitarianContribution {
            id: id.clone(),
            profile_id: profile_id.clone(),
            amount,
            currency,
            description,
            kind,
            verified: true,
            timestamp: now,
            tx_hash: Some(format!("0x{}", Uuid::new_v4().simple())),
        };
        self.contributions.insert(id.clone(), contribution);
        Ok(id)
    }

    pub fn mint_peace_passport(
        &mut self,
        profile_id: String,
        token_uri: String,
        metadata: HashMap<String, String>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let profile = self.profiles.get(&profile_id).ok_or("Profile not found")?;
        if self.passports.values().any(|p| p.profile_id == profile_id) {
            return Err("Passport already minted for this profile".into());
        }
        let id = format!("passport_{}", Uuid::new_v4());
        let now = self.now();
        let passport = PeacePassport {
            id: id.clone(),
            profile_id: profile.id.clone(),
            wallet_address: profile.wallet_address.clone(),
            token_uri,
            metadata,
            minted_at: now,
            verified_on_chain: false,
            verification_tx: None,
        };
        self.passports.insert(id.clone(), passport);
        Ok(id)
    }

    pub fn verify_passport(
        &mut self,
        passport_id: &str,
        tx_hash: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let passport = self
            .passports
            .get_mut(passport_id)
            .ok_or("Passport not found")?;
        if passport.verified_on_chain {
            return Err("Passport already verified".into());
        }
        passport.verified_on_chain = true;
        passport.verification_tx = Some(tx_hash);
        Ok(())
    }

    pub fn log_volunteer_hours(
        &mut self,
        profile_id: String,
        description: String,
        hours: f64,
        location: Option<String>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        if hours <= 0.0 {
            return Err("Hours must be positive".into());
        }

        // Get the score increment value before getting a mutable reference to the profile
        let score_increment = self.profile_score_increment(0.0, hours);

        // Now get a mutable reference to update the profile
        let now = self.now();
        let profile = self
            .profiles
            .get_mut(&profile_id)
            .ok_or("Profile not found")?;
        profile.volunteer_hours_total += hours;
        profile.humanitarian_score += score_increment;

        let id = format!("vol_hour_{}", Uuid::new_v4());
        let entry = VolunteerHourEntry {
            id: id.clone(),
            profile_id: profile_id.clone(),
            description,
            hours,
            location,
            status: VolunteerHourStatus::Logged,
            timestamp: now,
            tx_hash: None,
        };
        self.volunteer_hours.insert(id.clone(), entry);
        Ok(id)
    }

    pub fn verify_volunteer_hours(
        &mut self,
        entry_id: &str,
        tx_hash: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let entry = self
            .volunteer_hours
            .get_mut(entry_id)
            .ok_or("Volunteer entry not found")?;
        if entry.status == VolunteerHourStatus::Verified {
            return Err("Volunteer hours already verified".into());
        }
        entry.status = VolunteerHourStatus::Verified;
        entry.tx_hash = Some(tx_hash);
        Ok(())
    }

    pub fn get_profile(&self, profile_id: &str) -> Option<&HumanitarianProfile> {
        self.profiles.get(profile_id)
    }

    pub fn get_passport(&self, passport_id: &str) -> Option<&PeacePassport> {
        self.passports.get(passport_id)
    }

    pub fn get_volunteer_entry(&self, entry_id: &str) -> Option<&VolunteerHourEntry> {
        self.volunteer_hours.get(entry_id)
    }
}
