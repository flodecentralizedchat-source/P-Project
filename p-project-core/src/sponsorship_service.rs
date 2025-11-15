//! Peace-building sponsorship service for P-Project
//!
//! Provides:
//! - Student candidate registration and sponsorship
//! - Peace-builder profile registration and sponsorship
//! - Peace education program funding
//! - Sponsorship transaction tracking

use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Configuration for the sponsorship service
#[derive(Debug, Clone)]
pub struct SponsorshipServiceConfig {
    pub currency: String, // Only this currency is accepted for sponsorships
    pub max_single_transaction: f64, // Maximum amount for a single sponsorship
}

/// Student candidate for peace-building education sponsorship
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudentCandidate {
    pub id: String,
    pub full_name: String,
    pub country: String,
    pub conflict_zone: bool,
    pub field_of_study: String,
    pub education_level: String, // "Undergraduate", "Master's", "PhD", etc.
    pub required_amount: f64,
    pub currency: String,
    pub funded_amount: f64,
    pub sponsor_count: usize,
    pub description: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub is_active: bool,
}

/// Peace-builder profile (activist, mediator, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeaceBuilderProfile {
    pub id: String,
    pub name: String,
    pub location: String,
    pub focus_area: String, // "Youth mediation", "Community dialogue", etc.
    pub years_of_experience: u8,
    pub required_amount: f64,
    pub currency: String,
    pub funded_amount: f64,
    pub sponsor_count: usize,
    pub is_active: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub statement: Option<String>,
}

/// Peace education program
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeaceEducationProgram {
    pub id: String,
    pub title: String,
    pub organizer: String,
    pub region: String,
    pub curriculum_focus: String,
    pub funding_goal: f64,
    pub currency: String,
    pub funds_received: f64,
    pub participant_capacity: usize,
    pub is_active: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// Types of sponsorship targets
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SponsorshipTargetType {
    Student,
    PeaceBuilder,
    EducationProgram,
}

/// Sponsorship transaction record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SponsorshipTransaction {
    pub transaction_id: String,
    pub sponsor_id: String,
    pub target_id: String,
    pub target_type: SponsorshipTargetType,
    pub amount: f64,
    pub currency: String,
    pub status: String, // "completed", "pending", "failed"
    pub message: Option<String>,
    pub timestamp: NaiveDateTime,
}

/// Sponsorship request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SponsorshipRequest {
    pub target_id: String,
    pub sponsor_id: String,
    pub amount: f64,
    pub currency: String,
    pub message: Option<String>,
}

/// Peace-building sponsorship service
pub struct SponsorshipService {
    config: SponsorshipServiceConfig,
    students: HashMap<String, StudentCandidate>,
    builders: HashMap<String, PeaceBuilderProfile>,
    programs: HashMap<String, PeaceEducationProgram>,
    transactions: HashMap<String, SponsorshipTransaction>,
}

impl SponsorshipService {
    /// Create a new sponsorship service
    pub fn new(config: SponsorshipServiceConfig) -> Self {
        Self {
            config,
            students: HashMap::new(),
            builders: HashMap::new(),
            programs: HashMap::new(),
            transactions: HashMap::new(),
        }
    }

    fn now(&self) -> NaiveDateTime {
        Utc::now().naive_utc()
    }

    fn ensure_currency(&self, currency: &str) -> Result<(), Box<dyn std::error::Error>> {
        if currency != self.config.currency {
            Err(format!("Only {} is supported in sponsorships", self.config.currency).into())
        } else {
            Ok(())
        }
    }

    fn ensure_amount(&self, amount: f64) -> Result<(), Box<dyn std::error::Error>> {
        if amount <= 0.0 {
            Err("Amount must be positive".into())
        } else if amount > self.config.max_single_transaction {
            Err(format!(
                "Amount exceeds maximum allowed: {}",
                self.config.max_single_transaction
            )
            .into())
        } else {
            Ok(())
        }
    }

    fn record_transaction(
        &mut self,
        sponsor_id: String,
        target_id: String,
        target_type: SponsorshipTargetType,
        amount: f64,
        message: Option<String>,
    ) -> SponsorshipTransaction {
        let transaction_id = format!("sponsor_{}", Uuid::new_v4());
        let tx = SponsorshipTransaction {
            transaction_id: transaction_id.clone(),
            sponsor_id,
            target_id,
            target_type,
            amount,
            currency: self.config.currency.clone(),
            status: "completed".to_string(),
            message,
            timestamp: self.now(),
        };
        self.transactions.insert(transaction_id.clone(), tx.clone());
        tx
    }

    fn apply_sponsorship(
        &mut self,
        amount: f64,
        remaining_need: f64,
    ) -> Result<f64, Box<dyn std::error::Error>> {
        if remaining_need <= 0.0 {
            return Err("Target is already fully funded".into());
        }
        if amount > remaining_need {
            Err("Amount exceeds remaining funding need".into())
        } else {
            Ok(amount)
        }
    }

    /// Register a student who needs peace-building sponsorship.
    pub fn register_student(
        &mut self,
        full_name: String,
        country: String,
        conflict_zone: bool,
        field_of_study: String,
        education_level: String,
        required_amount: f64,
        currency: String,
        description: Option<String>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        self.ensure_currency(&currency)?;
        if required_amount <= 0.0 {
            return Err("Required amount must be positive".into());
        }

        let id = format!("student_{}", Uuid::new_v4());
        let now = self.now();
        let profile = StudentCandidate {
            id: id.clone(),
            full_name,
            country,
            conflict_zone,
            field_of_study,
            education_level,
            required_amount,
            currency,
            funded_amount: 0.0,
            sponsor_count: 0,
            description,
            created_at: now,
            updated_at: now,
            is_active: true,
        };
        self.students.insert(id.clone(), profile);
        Ok(id)
    }

    /// Register a peace-builder, activist, or mediator.
    pub fn register_peace_builder(
        &mut self,
        name: String,
        location: String,
        focus_area: String,
        years_of_experience: u8,
        required_amount: f64,
        currency: String,
        statement: Option<String>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        self.ensure_currency(&currency)?;
        if required_amount <= 0.0 {
            return Err("Required amount must be positive".into());
        }

        let id = format!("builder_{}", Uuid::new_v4());
        let now = self.now();
        let profile = PeaceBuilderProfile {
            id: id.clone(),
            name,
            location,
            focus_area,
            years_of_experience,
            required_amount,
            currency,
            funded_amount: 0.0,
            sponsor_count: 0,
            is_active: true,
            created_at: now,
            updated_at: now,
            statement,
        };
        self.builders.insert(id.clone(), profile);
        Ok(id)
    }

    /// Register a peace education program.
    pub fn register_program(
        &mut self,
        title: String,
        organizer: String,
        region: String,
        curriculum_focus: String,
        funding_goal: f64,
        currency: String,
        participant_capacity: usize,
    ) -> Result<String, Box<dyn std::error::Error>> {
        self.ensure_currency(&currency)?;
        if funding_goal <= 0.0 {
            return Err("Funding goal must be positive".into());
        }

        let id = format!("program_{}", Uuid::new_v4());
        let now = self.now();
        let program = PeaceEducationProgram {
            id: id.clone(),
            title,
            organizer,
            region,
            curriculum_focus,
            funding_goal,
            currency,
            funds_received: 0.0,
            participant_capacity,
            is_active: true,
            created_at: now,
            updated_at: now,
        };
        self.programs.insert(id.clone(), program);
        Ok(id)
    }

    /// Sponsor a registered student.
    pub fn sponsor_student(
        &mut self,
        request: SponsorshipRequest,
    ) -> Result<SponsorshipTransaction, Box<dyn std::error::Error>> {
        self.ensure_currency(&request.currency)?;
        self.ensure_amount(request.amount)?;

        // Check if student exists and is active, and calculate remaining amount
        let remaining = {
            let student = self
                .students
                .get(&request.target_id)
                .ok_or("Student candidate not found")?;

            if !student.is_active {
                return Err("Student is no longer accepting sponsorships".into());
            }

            student.required_amount - student.funded_amount
        };

        let applied_amount = self.apply_sponsorship(request.amount, remaining)?;

        // Get mutable reference to student and update it
        let now = self.now();
        let student = self.students.get_mut(&request.target_id).unwrap(); // Safe because we already checked it exists

        student.funded_amount += applied_amount;
        student.sponsor_count += 1;
        student.updated_at = now;
        if (student.funded_amount - student.required_amount).abs() < f64::EPSILON
            || student.funded_amount >= student.required_amount
        {
            student.is_active = false;
        }

        Ok(self.record_transaction(
            request.sponsor_id,
            request.target_id,
            SponsorshipTargetType::Student,
            applied_amount,
            request.message,
        ))
    }

    /// Sponsor a peace-builder profile.
    pub fn sponsor_peace_builder(
        &mut self,
        request: SponsorshipRequest,
    ) -> Result<SponsorshipTransaction, Box<dyn std::error::Error>> {
        self.ensure_currency(&request.currency)?;
        self.ensure_amount(request.amount)?;

        // Check if builder exists and is active, and calculate remaining amount
        let remaining = {
            let builder = self
                .builders
                .get(&request.target_id)
                .ok_or("Peace-builder profile not found")?;

            if !builder.is_active {
                return Err("Peace-builder is not accepting sponsorships".into());
            }

            builder.required_amount - builder.funded_amount
        };

        let applied_amount = self.apply_sponsorship(request.amount, remaining)?;

        // Get mutable reference to builder and update it
        let now = self.now();
        let builder = self.builders.get_mut(&request.target_id).unwrap(); // Safe because we already checked it exists

        builder.funded_amount += applied_amount;
        builder.sponsor_count += 1;
        builder.updated_at = now;
        if builder.funded_amount >= builder.required_amount {
            builder.is_active = false;
        }

        Ok(self.record_transaction(
            request.sponsor_id,
            request.target_id,
            SponsorshipTargetType::PeaceBuilder,
            applied_amount,
            request.message,
        ))
    }

    /// Fund a peace education program.
    pub fn fund_program(
        &mut self,
        request: SponsorshipRequest,
    ) -> Result<SponsorshipTransaction, Box<dyn std::error::Error>> {
        self.ensure_currency(&request.currency)?;
        self.ensure_amount(request.amount)?;

        // Check if program exists and is active, and calculate remaining amount
        let remaining = {
            let program = self
                .programs
                .get(&request.target_id)
                .ok_or("Program not found")?;

            if !program.is_active {
                return Err("Program is not accepting funds".into());
            }

            program.funding_goal - program.funds_received
        };

        let applied_amount = self.apply_sponsorship(request.amount, remaining)?;

        // Get mutable reference to program and update it
        let now = self.now();
        let program = self.programs.get_mut(&request.target_id).unwrap(); // Safe because we already checked it exists

        program.funds_received += applied_amount;
        program.updated_at = now;
        if program.funds_received >= program.funding_goal {
            program.is_active = false;
        }

        Ok(self.record_transaction(
            request.sponsor_id,
            request.target_id,
            SponsorshipTargetType::EducationProgram,
            applied_amount,
            request.message,
        ))
    }

    pub fn get_student(&self, student_id: &str) -> Option<&StudentCandidate> {
        self.students.get(student_id)
    }

    pub fn list_students(&self) -> Vec<&StudentCandidate> {
        self.students.values().collect()
    }

    pub fn get_builder(&self, builder_id: &str) -> Option<&PeaceBuilderProfile> {
        self.builders.get(builder_id)
    }

    pub fn list_builders(&self) -> Vec<&PeaceBuilderProfile> {
        self.builders.values().collect()
    }

    pub fn get_program(&self, program_id: &str) -> Option<&PeaceEducationProgram> {
        self.programs.get(program_id)
    }

    pub fn list_programs(&self) -> Vec<&PeaceEducationProgram> {
        self.programs.values().collect()
    }

    pub fn get_transaction(&self, transaction_id: &str) -> Option<&SponsorshipTransaction> {
        self.transactions.get(transaction_id)
    }
}
