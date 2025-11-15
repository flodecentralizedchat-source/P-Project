//! Government use cases service for P-Project
//!
//! Supports peace incentive programs, youth grants, blockchain welfare disbursement,
//! and anti-corruption transparency tracking.

use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct GovernmentServiceConfig {
    pub supported_currency: String,
    pub max_disbursement: f64,
}

impl Default for GovernmentServiceConfig {
    fn default() -> Self {
        Self {
            supported_currency: "P".to_string(),
            max_disbursement: 1000.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum GovernmentProgramType {
    PeaceIncentive,
    YouthGrant,
    WelfareDisbursement,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernmentProgram {
    pub program_id: String,
    pub name: String,
    pub description: String,
    pub region: String,
    pub program_type: GovernmentProgramType,
    pub total_budget: f64,
    pub currency: String,
    pub distributed: f64,
    pub metadata: Option<serde_json::Value>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum YouthGrantStatus {
    Submitted,
    Approved,
    Rejected,
    Funded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YouthGrantApplication {
    pub application_id: String,
    pub applicant_id: String,
    pub program_id: String,
    pub amount_requested: f64,
    pub amount_awarded: f64,
    pub status: YouthGrantStatus,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub narrative: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WelfareDisbursementRecord {
    pub disbursement_id: String,
    pub program_id: String,
    pub recipient_id: String,
    pub amount: f64,
    pub wallet_address: String,
    pub timestamp: NaiveDateTime,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransparencyEvent {
    pub event_id: String,
    pub description: String,
    pub program_id: String,
    pub recorded_at: NaiveDateTime,
    pub data_hash: String,
    pub status: String,
}

pub struct GovernmentService {
    config: GovernmentServiceConfig,
    programs: HashMap<String, GovernmentProgram>,
    youth_applications: HashMap<String, YouthGrantApplication>,
    disbursements: HashMap<String, WelfareDisbursementRecord>,
    transparency_events: Vec<TransparencyEvent>,
}

impl GovernmentService {
    pub fn new(config: GovernmentServiceConfig) -> Self {
        Self {
            config,
            programs: HashMap::new(),
            youth_applications: HashMap::new(),
            disbursements: HashMap::new(),
            transparency_events: Vec::new(),
        }
    }

    fn now() -> NaiveDateTime {
        Utc::now().naive_utc()
    }

    fn ensure_currency(&self, currency: &str) -> Result<(), Box<dyn std::error::Error>> {
        if currency != self.config.supported_currency {
            Err(format!(
                "Only {} currency is supported for government programs",
                self.config.supported_currency
            )
            .into())
        } else {
            Ok(())
        }
    }

    pub fn create_program(
        &mut self,
        name: String,
        description: String,
        region: String,
        program_type: GovernmentProgramType,
        total_budget: f64,
        currency: String,
        metadata: Option<serde_json::Value>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        self.ensure_currency(&currency)?;

        if total_budget <= 0.0 {
            return Err("Budget must be positive".into());
        }

        let id = format!("govprog_{}", Uuid::new_v4());
        let now = Self::now();
        let program = GovernmentProgram {
            program_id: id.clone(),
            name,
            description,
            region,
            program_type,
            total_budget,
            currency,
            distributed: 0.0,
            metadata,
            created_at: now,
            updated_at: now,
        };

        self.programs.insert(id.clone(), program);
        Ok(id)
    }

    pub fn award_incentive(
        &mut self,
        program_id: &str,
        amount: f64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if amount <= 0.0 {
            return Err("Award amount must be positive".into());
        }

        let program = self
            .programs
            .get_mut(program_id)
            .ok_or("Program not found")?;

        if program.program_type != GovernmentProgramType::PeaceIncentive {
            return Err("Awards can only be processed for peace incentive initiatives".into());
        }

        let remaining = program.total_budget - program.distributed;
        if amount > remaining {
            return Err("Not enough budget remaining".into());
        }

        program.distributed += amount;
        program.updated_at = Self::now();
        Ok(())
    }

    pub fn submit_youth_grant_application(
        &mut self,
        applicant_id: String,
        program_id: String,
        amount_requested: f64,
        narrative: Option<String>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        if amount_requested <= 0.0 {
            return Err("Requested amount must be positive".into());
        }

        let program = self.programs.get(&program_id).ok_or("Program not found")?;

        if program.program_type != GovernmentProgramType::YouthGrant {
            return Err("Applications can only be submitted to youth grant programs".into());
        }

        let id = format!("youthapp_{}", Uuid::new_v4());
        let now = Self::now();
        let application = YouthGrantApplication {
            application_id: id.clone(),
            applicant_id,
            program_id,
            amount_requested,
            amount_awarded: 0.0,
            status: YouthGrantStatus::Submitted,
            created_at: now,
            updated_at: now,
            narrative,
        };

        self.youth_applications.insert(id.clone(), application);
        Ok(id)
    }

    pub fn approve_youth_grant(
        &mut self,
        application_id: &str,
        amount: f64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let app = self
            .youth_applications
            .get_mut(application_id)
            .ok_or("Application not found")?;

        if app.status != YouthGrantStatus::Submitted {
            return Err("Only submitted applications can be approved".into());
        }

        if amount <= 0.0 || amount > app.amount_requested {
            return Err("Approval amount must be positive and not exceed requested amount".into());
        }

        let program = self
            .programs
            .get_mut(&app.program_id)
            .ok_or("Program not found")?;

        let remaining = program.total_budget - program.distributed;
        if amount > remaining {
            return Err("Insufficient program budget".into());
        }

        program.distributed += amount;
        program.updated_at = Self::now();
        app.amount_awarded = amount;
        app.status = YouthGrantStatus::Approved;
        app.updated_at = Self::now();
        Ok(())
    }

    pub fn reject_youth_grant(
        &mut self,
        application_id: &str,
        reason: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let app = self
            .youth_applications
            .get_mut(application_id)
            .ok_or("Application not found")?;

        if app.status != YouthGrantStatus::Submitted {
            return Err("Only submitted applications can be rejected".into());
        }

        app.status = YouthGrantStatus::Rejected;
        app.updated_at = Self::now();
        if let Some(notes) = reason {
            app.narrative = Some(notes);
        }
        Ok(())
    }

    pub fn disburse_welfare_funds(
        &mut self,
        program_id: &str,
        recipient_id: String,
        amount: f64,
        wallet_address: String,
        metadata: Option<serde_json::Value>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        if amount <= 0.0 || amount > self.config.max_disbursement {
            return Err("Disbursement amount invalid".into());
        }

        let program = self
            .programs
            .get_mut(program_id)
            .ok_or("Program not found")?;

        if program.program_type != GovernmentProgramType::WelfareDisbursement {
            return Err("Disbursements must target welfare programs".into());
        }

        let remaining = program.total_budget - program.distributed;
        if amount > remaining {
            return Err("Program budget exhausted".into());
        }

        program.distributed += amount;
        program.updated_at = Self::now();

        let disbursement_id = format!("welfare_{}", Uuid::new_v4());
        let record = WelfareDisbursementRecord {
            disbursement_id: disbursement_id.clone(),
            program_id: program_id.to_string(),
            recipient_id,
            amount,
            wallet_address,
            timestamp: Self::now(),
            metadata,
        };
        self.disbursements.insert(disbursement_id.clone(), record);
        Ok(disbursement_id)
    }

    pub fn log_transparency_event(
        &mut self,
        program_id: &str,
        description: String,
        data_hash: String,
        status: String,
    ) -> Result<String, Box<dyn std::error::Error>> {
        if !self.programs.contains_key(program_id) {
            return Err("Program not found".into());
        }

        let event_id = format!("transp_{}", Uuid::new_v4());
        let event = TransparencyEvent {
            event_id: event_id.clone(),
            description,
            program_id: program_id.to_string(),
            recorded_at: Self::now(),
            data_hash,
            status,
        };
        self.transparency_events.push(event);
        Ok(event_id)
    }

    pub fn get_program(&self, program_id: &str) -> Option<&GovernmentProgram> {
        self.programs.get(program_id)
    }

    pub fn get_application(&self, application_id: &str) -> Option<&YouthGrantApplication> {
        self.youth_applications.get(application_id)
    }

    pub fn get_disbursement(&self, disbursement_id: &str) -> Option<&WelfareDisbursementRecord> {
        self.disbursements.get(disbursement_id)
    }

    pub fn transparency_feed(&self) -> &[TransparencyEvent] {
        &self.transparency_events
    }
}
