//! Developer Grants service for funding builders and projects.
use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProgramStatus {
    Draft,
    Open,
    Paused,
    Closed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrantProgram {
    pub id: String,
    pub title: String,
    pub description: String,
    pub total_budget: f64,
    pub allocated_budget: f64,
    pub start_date: NaiveDateTime,
    pub end_date: Option<NaiveDateTime>,
    pub status: ProgramStatus,
    pub criteria: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ApplicationStatus {
    Submitted,
    UnderReview,
    Approved,
    Rejected,
    Funded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Review {
    pub reviewer_id: String,
    pub technical: u8,   // 0-10
    pub impact: u8,      // 0-10
    pub feasibility: u8, // 0-10
    pub comments: Option<String>,
    pub submitted_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MilestoneStatus {
    Pending,
    Submitted,
    Approved,
    Paid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Milestone {
    pub id: String,
    pub title: String,
    pub description: String,
    pub due_date: Option<NaiveDateTime>,
    pub payout_amount: f64,
    pub status: MilestoneStatus,
    pub proof_url: Option<String>,
    pub submitted_at: Option<NaiveDateTime>,
    pub approved_at: Option<NaiveDateTime>,
    pub paid_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrantApplication {
    pub id: String,
    pub program_id: String,
    pub applicant_id: String,
    pub project_name: String,
    pub summary: String,
    pub requested_amount: f64,
    pub proposal_url: Option<String>,
    pub submitted_at: NaiveDateTime,
    pub status: ApplicationStatus,
    pub awarded_amount: f64,
    pub reviews: Vec<Review>,
    pub average_score: f64,
    pub milestones: Vec<Milestone>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayoutRecord {
    pub id: String,
    pub application_id: String,
    pub milestone_id: Option<String>,
    pub amount: f64,
    pub timestamp: NaiveDateTime,
    pub tx_hash: Option<String>,
}

#[derive(Default)]
pub struct DeveloperGrantsService {
    pub programs: HashMap<String, GrantProgram>,
    pub applications: HashMap<String, GrantApplication>,
    pub payouts: Vec<PayoutRecord>,
}

impl DeveloperGrantsService {
    pub fn new() -> Self {
        Self::default()
    }

    fn now() -> NaiveDateTime {
        Utc::now().naive_utc()
    }

    pub fn create_program(
        &mut self,
        title: String,
        description: String,
        total_budget: f64,
        start_date: Option<NaiveDateTime>,
        end_date: Option<NaiveDateTime>,
        criteria: Option<String>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        if title.trim().is_empty() || description.trim().is_empty() {
            return Err("title and description are required".into());
        }
        if total_budget <= 0.0 {
            return Err("total budget must be positive".into());
        }

        let id = format!("grantprog_{}", Uuid::new_v4());
        let now = Self::now();
        let program = GrantProgram {
            id: id.clone(),
            title,
            description,
            total_budget,
            allocated_budget: 0.0,
            start_date: start_date.unwrap_or(now),
            end_date,
            status: ProgramStatus::Draft,
            criteria,
            created_at: now,
            updated_at: now,
        };
        self.programs.insert(id.clone(), program);
        Ok(id)
    }

    pub fn set_program_status(
        &mut self,
        program_id: &str,
        status: ProgramStatus,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let program = self
            .programs
            .get_mut(program_id)
            .ok_or("program not found")?;
        program.status = status;
        program.updated_at = Self::now();
        Ok(())
    }

    pub fn submit_application(
        &mut self,
        program_id: &str,
        applicant_id: String,
        project_name: String,
        summary: String,
        requested_amount: f64,
        proposal_url: Option<String>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let program = self.programs.get(program_id).ok_or("program not found")?;

        if program.status != ProgramStatus::Open {
            return Err("program is not open for applications".into());
        }
        if requested_amount <= 0.0 {
            return Err("requested amount must be positive".into());
        }
        if project_name.trim().is_empty() || summary.trim().is_empty() {
            return Err("project name and summary are required".into());
        }

        let id = format!("grantapp_{}", Uuid::new_v4());
        let app = GrantApplication {
            id: id.clone(),
            program_id: program_id.to_string(),
            applicant_id,
            project_name,
            summary,
            requested_amount,
            proposal_url,
            submitted_at: Self::now(),
            status: ApplicationStatus::Submitted,
            awarded_amount: 0.0,
            reviews: Vec::new(),
            average_score: 0.0,
            milestones: Vec::new(),
        };
        self.applications.insert(id.clone(), app);
        Ok(id)
    }

    pub fn start_review(&mut self, application_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let app = self
            .applications
            .get_mut(application_id)
            .ok_or("application not found")?;
        if app.status != ApplicationStatus::Submitted {
            return Err("application is not in submitted state".into());
        }
        app.status = ApplicationStatus::UnderReview;
        Ok(())
    }

    pub fn add_review(
        &mut self,
        application_id: &str,
        reviewer_id: String,
        technical: u8,
        impact: u8,
        feasibility: u8,
        comments: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if technical > 10 || impact > 10 || feasibility > 10 {
            return Err("scores must be between 0 and 10".into());
        }
        let app = self
            .applications
            .get_mut(application_id)
            .ok_or("application not found")?;
        if app.status != ApplicationStatus::UnderReview {
            return Err("application must be under review to add reviews".into());
        }

        let review = Review {
            reviewer_id,
            technical,
            impact,
            feasibility,
            comments,
            submitted_at: Self::now(),
        };
        app.reviews.push(review);
        let mut total = 0.0;
        let mut count = 0.0;
        for r in &app.reviews {
            total += (r.technical as f64 + r.impact as f64 + r.feasibility as f64) / 3.0;
            count += 1.0;
        }
        app.average_score = if count > 0.0 { total / count } else { 0.0 };
        Ok(())
    }

    pub fn decide_application(
        &mut self,
        application_id: &str,
        approve: bool,
        awarded_amount: Option<f64>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (program_id, requested_amount);
        {
            let app = self
                .applications
                .get(application_id)
                .ok_or("application not found")?;
            program_id = app.program_id.clone();
            requested_amount = app.requested_amount;
        }

        let program = self
            .programs
            .get_mut(&program_id)
            .ok_or("program not found")?;

        let app = self
            .applications
            .get_mut(application_id)
            .ok_or("application not found")?;

        if app.status != ApplicationStatus::UnderReview {
            return Err("application must be under review to decide".into());
        }

        if !approve {
            app.status = ApplicationStatus::Rejected;
            return Ok(());
        }

        let grant = awarded_amount.unwrap_or(requested_amount);
        if grant <= 0.0 {
            return Err("awarded amount must be positive".into());
        }

        if program.allocated_budget + grant > program.total_budget {
            return Err("insufficient program budget".into());
        }

        program.allocated_budget += grant;
        program.updated_at = Self::now();
        app.awarded_amount = grant;
        app.status = ApplicationStatus::Approved;
        Ok(())
    }

    pub fn add_milestone(
        &mut self,
        application_id: &str,
        title: String,
        description: String,
        due_date: Option<NaiveDateTime>,
        payout_amount: f64,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let app = self
            .applications
            .get_mut(application_id)
            .ok_or("application not found")?;
        if app.status != ApplicationStatus::Approved && app.status != ApplicationStatus::Funded {
            return Err("application must be approved or funded to add milestones".into());
        }
        if payout_amount <= 0.0 {
            return Err("payout amount must be positive".into());
        }
        let already_assigned: f64 = app.milestones.iter().map(|m| m.payout_amount).sum();
        if already_assigned + payout_amount > app.awarded_amount + 1e-9 {
            return Err("milestones exceed awarded amount".into());
        }
        let id = format!("milestone_{}", Uuid::new_v4());
        let m = Milestone {
            id: id.clone(),
            title,
            description,
            due_date,
            payout_amount,
            status: MilestoneStatus::Pending,
            proof_url: None,
            submitted_at: None,
            approved_at: None,
            paid_at: None,
        };
        app.milestones.push(m);
        Ok(id)
    }

    pub fn submit_milestone_proof(
        &mut self,
        application_id: &str,
        milestone_id: &str,
        proof_url: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let app = self
            .applications
            .get_mut(application_id)
            .ok_or("application not found")?;
        let m = app
            .milestones
            .iter_mut()
            .find(|m| m.id == milestone_id)
            .ok_or("milestone not found")?;
        if m.status != MilestoneStatus::Pending {
            return Err("milestone must be pending to submit proof".into());
        }
        m.proof_url = Some(proof_url);
        m.submitted_at = Some(Self::now());
        m.status = MilestoneStatus::Submitted;
        Ok(())
    }

    pub fn approve_milestone(
        &mut self,
        application_id: &str,
        milestone_id: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let app = self
            .applications
            .get_mut(application_id)
            .ok_or("application not found")?;
        let m = app
            .milestones
            .iter_mut()
            .find(|m| m.id == milestone_id)
            .ok_or("milestone not found")?;
        if m.status != MilestoneStatus::Submitted {
            return Err("milestone must be submitted to approve".into());
        }
        m.status = MilestoneStatus::Approved;
        m.approved_at = Some(Self::now());
        Ok(())
    }

    pub fn pay_milestone(
        &mut self,
        application_id: &str,
        milestone_id: &str,
    ) -> Result<PayoutRecord, Box<dyn std::error::Error>> {
        let app = self
            .applications
            .get_mut(application_id)
            .ok_or("application not found")?;
        let m = app
            .milestones
            .iter_mut()
            .find(|m| m.id == milestone_id)
            .ok_or("milestone not found")?;
        if m.status != MilestoneStatus::Approved {
            return Err("milestone must be approved to pay".into());
        }
        m.status = MilestoneStatus::Paid;
        m.paid_at = Some(Self::now());

        let payout = PayoutRecord {
            id: format!("payout_{}", Uuid::new_v4()),
            application_id: application_id.to_string(),
            milestone_id: Some(milestone_id.to_string()),
            amount: m.payout_amount,
            timestamp: Self::now(),
            tx_hash: Some(format!("0x{}", Uuid::new_v4().simple())),
        };
        self.payouts.push(payout.clone());
        if app.status == ApplicationStatus::Approved {
            app.status = ApplicationStatus::Funded;
        }
        Ok(payout)
    }

    pub fn get_program_stats(
        &self,
        program_id: &str,
    ) -> Result<HashMap<String, f64>, Box<dyn std::error::Error>> {
        let program = self.programs.get(program_id).ok_or("program not found")?;
        let mut stats = HashMap::new();
        let total_apps = self
            .applications
            .values()
            .filter(|a| a.program_id == program.id)
            .count() as f64;
        stats.insert("total_applications".to_string(), total_apps);
        stats.insert("total_budget".to_string(), program.total_budget);
        stats.insert("allocated_budget".to_string(), program.allocated_budget);
        stats.insert(
            "remaining_budget".to_string(),
            (program.total_budget - program.allocated_budget).max(0.0),
        );
        Ok(stats)
    }
}
