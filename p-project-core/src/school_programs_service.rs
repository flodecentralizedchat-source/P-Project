//! School & University Programs service for P-Project
//! Implements:
//! - Peace clubs receiving treasury grants
//! - Student reward system
//! - Blockchain education integration

use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct SchoolProgramsConfig {
    pub currency: String,
    pub max_grant_amount: f64,
    pub reward_points_per_course: f64,
}

impl Default for SchoolProgramsConfig {
    fn default() -> Self {
        Self {
            currency: "P".into(),
            max_grant_amount: 10_000.0,
            reward_points_per_course: 120.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct School {
    pub id: String,
    pub name: String,
    pub location: Option<String>,
    pub wallet_address: String,
    pub verified: bool,
    pub created_at: NaiveDateTime,
    pub verified_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeaceClub {
    pub id: String,
    pub school_id: String,
    pub name: String,
    pub description: Option<String>,
    pub verified: bool,
    pub created_at: NaiveDateTime,
    pub verified_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum GrantStatus {
    Requested,
    Approved,
    Disbursed,
    Declined,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrantRequest {
    pub id: String,
    pub club_id: String,
    pub club_name: String,
    pub school_id: String,
    pub amount: f64,
    pub currency: String,
    pub purpose: String,
    pub status: GrantStatus,
    pub requested_at: NaiveDateTime,
    pub approved_at: Option<NaiveDateTime>,
    pub disbursed_at: Option<NaiveDateTime>,
    pub tx_hash: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudentRewardAccount {
    pub student_id: String,
    pub name: String,
    pub school_id: String,
    pub wallet_address: String,
    pub points: f64,
    pub badges: Vec<String>,
    pub history: Vec<RewardTransaction>,
    pub created_at: NaiveDateTime,
    pub last_updated: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardTransaction {
    pub id: String,
    pub student_id: String,
    pub points: f64,
    pub description: String,
    pub kind: RewardKind,
    pub timestamp: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RewardKind {
    Earned,
    Redeemed,
    CourseCompletion,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainCourse {
    pub id: String,
    pub school_id: String,
    pub title: String,
    pub description: String,
    pub modules: Vec<String>,
    pub published: bool,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CourseModuleStatus {
    Locked,
    Unlocked,
    Completed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourseEnrollment {
    pub id: String,
    pub student_id: String,
    pub course_id: String,
    pub module_statuses: HashMap<String, CourseModuleStatus>,
    pub enrolled_at: NaiveDateTime,
    pub last_updated: NaiveDateTime,
    pub completed: bool,
    pub certificate: Option<String>,
}

pub struct SchoolProgramsService {
    pub config: SchoolProgramsConfig,
    pub schools: HashMap<String, School>,
    pub peace_clubs: HashMap<String, PeaceClub>,
    pub grants: HashMap<String, GrantRequest>,
    pub reward_accounts: HashMap<String, StudentRewardAccount>,
    pub courses: HashMap<String, BlockchainCourse>,
    pub enrollments: HashMap<String, CourseEnrollment>,
}

impl SchoolProgramsService {
    pub fn new(config: SchoolProgramsConfig) -> Self {
        Self {
            config,
            schools: HashMap::new(),
            peace_clubs: HashMap::new(),
            grants: HashMap::new(),
            reward_accounts: HashMap::new(),
            courses: HashMap::new(),
            enrollments: HashMap::new(),
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

    fn ensure_positive_amount(&self, amount: f64) -> Result<(), Box<dyn std::error::Error>> {
        if amount <= 0.0 {
            Err("Amount must be positive".into())
        } else if amount > self.config.max_grant_amount {
            Err("Amount exceeds grant cap".into())
        } else {
            Ok(())
        }
    }

    pub fn register_school(
        &mut self,
        name: String,
        wallet_address: String,
        location: Option<String>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let id = format!("school_{}", Uuid::new_v4());
        let now = self.now();
        let school = School {
            id: id.clone(),
            name,
            location,
            wallet_address,
            verified: false,
            created_at: now,
            verified_at: None,
        };
        self.schools.insert(id.clone(), school);
        Ok(id)
    }

    pub fn verify_school(&mut self, school_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let now = self.now();
        let school = self.schools.get_mut(school_id).ok_or("School not found")?;
        school.verified = true;
        school.verified_at = Some(now);
        Ok(())
    }

    pub fn register_peace_club(
        &mut self,
        school_id: String,
        name: String,
        description: Option<String>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let school = self.schools.get(&school_id).ok_or("School not found")?;
        if !school.verified {
            return Err("School must be verified to host peace clubs".into());
        }
        let id = format!("club_{}", Uuid::new_v4());
        let now = self.now();
        let club = PeaceClub {
            id: id.clone(),
            school_id,
            name,
            description,
            verified: false,
            created_at: now,
            verified_at: None,
        };
        self.peace_clubs.insert(id.clone(), club);
        Ok(id)
    }

    pub fn verify_peace_club(&mut self, club_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let now = self.now();
        let club = self
            .peace_clubs
            .get_mut(club_id)
            .ok_or("Peace club not found")?;
        club.verified = true;
        club.verified_at = Some(now);
        Ok(())
    }

    pub fn request_treasury_grant(
        &mut self,
        club_id: String,
        amount: f64,
        purpose: String,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let club = self
            .peace_clubs
            .get(&club_id)
            .ok_or("Peace club not found")?;
        if !club.verified {
            return Err("Peace club must be verified before requesting grants".into());
        }
        self.ensure_positive_amount(amount)?;
        let id = format!("grant_{}", Uuid::new_v4());
        let now = self.now();
        let grant = GrantRequest {
            id: id.clone(),
            club_id: club.id.clone(),
            club_name: club.name.clone(),
            school_id: club.school_id.clone(),
            amount,
            currency: self.config.currency.clone(),
            purpose,
            status: GrantStatus::Requested,
            requested_at: now,
            approved_at: None,
            disbursed_at: None,
            tx_hash: None,
            notes: None,
        };
        self.grants.insert(id.clone(), grant);
        Ok(id)
    }

    pub fn approve_grant(
        &mut self,
        grant_id: &str,
        notes: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let now = self.now();
        let grant = self
            .grants
            .get_mut(grant_id)
            .ok_or("Grant request not found")?;
        if grant.status != GrantStatus::Requested {
            return Err("Grant must be in requested state".into());
        }
        grant.status = GrantStatus::Approved;
        grant.approved_at = Some(now);
        grant.notes = notes;
        Ok(())
    }

    pub fn disburse_grant(&mut self, grant_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let now = self.now();
        let grant = self
            .grants
            .get_mut(grant_id)
            .ok_or("Grant request not found")?;
        if grant.status != GrantStatus::Approved {
            return Err("Grant must be approved before disbursement".into());
        }
        grant.disbursed_at = Some(now);
        grant.status = GrantStatus::Disbursed;
        grant.tx_hash = Some(format!("0x{}", Uuid::new_v4().simple()));
        Ok(())
    }

    pub fn get_grant(&self, grant_id: &str) -> Option<&GrantRequest> {
        self.grants.get(grant_id)
    }

    pub fn register_student_reward_account(
        &mut self,
        student_id: String,
        name: String,
        school_id: String,
        wallet_address: String,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let school = self.schools.get(&school_id).ok_or("School not found")?;
        if !school.verified {
            return Err("School must be verified to register students".into());
        }
        let now = self.now();
        let account = StudentRewardAccount {
            student_id: student_id.clone(),
            name,
            school_id,
            wallet_address,
            points: 0.0,
            badges: Vec::new(),
            history: Vec::new(),
            created_at: now,
            last_updated: now,
        };
        self.reward_accounts.insert(student_id.clone(), account);
        Ok(student_id)
    }

    pub fn award_reward_points(
        &mut self,
        student_id: &str,
        points: f64,
        description: String,
    ) -> Result<f64, Box<dyn std::error::Error>> {
        if points <= 0.0 {
            return Err("Points must be positive".into());
        }
        let now = self.now();
        let account = self
            .reward_accounts
            .get_mut(student_id)
            .ok_or("Student reward account not found")?;
        account.points += points;
        account.last_updated = now;
        let transaction = RewardTransaction {
            id: format!("reward_tx_{}", Uuid::new_v4()),
            student_id: student_id.to_string(),
            points,
            description,
            kind: RewardKind::Earned,
            timestamp: now,
        };
        account.history.push(transaction);
        Ok(account.points)
    }

    pub fn redeem_reward_points(
        &mut self,
        student_id: &str,
        points: f64,
        description: String,
    ) -> Result<f64, Box<dyn std::error::Error>> {
        if points <= 0.0 {
            return Err("Points must be positive".into());
        }
        let now = self.now();
        let account = self
            .reward_accounts
            .get_mut(student_id)
            .ok_or("Student reward account not found")?;
        if account.points < points {
            return Err("Insufficient reward points".into());
        }
        account.points -= points;
        account.last_updated = now;
        let transaction = RewardTransaction {
            id: format!("reward_tx_{}", Uuid::new_v4()),
            student_id: student_id.to_string(),
            points: -points,
            description,
            kind: RewardKind::Redeemed,
            timestamp: now,
        };
        account.history.push(transaction);
        Ok(account.points)
    }

    pub fn award_badge(
        &mut self,
        student_id: &str,
        badge: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let now = self.now();
        let account = self
            .reward_accounts
            .get_mut(student_id)
            .ok_or("Student reward account not found")?;
        if !account.badges.contains(&badge) {
            account.badges.push(badge);
        }
        account.last_updated = now;
        Ok(())
    }

    pub fn get_reward_account(&self, student_id: &str) -> Option<&StudentRewardAccount> {
        self.reward_accounts.get(student_id)
    }

    pub fn create_blockchain_course(
        &mut self,
        school_id: String,
        title: String,
        description: String,
        modules: Vec<String>,
        publish: bool,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let school = self.schools.get(&school_id).ok_or("School not found")?;
        if !school.verified {
            return Err("School must be verified to publish courses".into());
        }
        if modules.is_empty() {
            return Err("Course must include at least one module".into());
        }
        let id = format!("course_{}", Uuid::new_v4());
        let course = BlockchainCourse {
            id: id.clone(),
            school_id,
            title,
            description,
            modules,
            published: publish,
            created_at: self.now(),
        };
        self.courses.insert(id.clone(), course);
        Ok(id)
    }

    pub fn publish_course(&mut self, course_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let course = self.courses.get_mut(course_id).ok_or("Course not found")?;
        course.published = true;
        Ok(())
    }

    pub fn enroll_student_in_course(
        &mut self,
        student_id: String,
        course_id: String,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let course = self.courses.get(&course_id).ok_or("Course not found")?;
        if !course.published {
            return Err("Course must be published before enrolling students".into());
        }
        let account = self
            .reward_accounts
            .get(&student_id)
            .ok_or("Student reward account not found")?;
        if account.school_id != course.school_id {
            return Err("Student must belong to the course school".into());
        }
        let mut statuses = HashMap::new();
        for module in &course.modules {
            statuses.insert(module.clone(), CourseModuleStatus::Unlocked);
        }
        let id = format!("enroll_{}", Uuid::new_v4());
        let now = self.now();
        let enrollment = CourseEnrollment {
            id: id.clone(),
            student_id,
            course_id,
            module_statuses: statuses,
            enrolled_at: now,
            last_updated: now,
            completed: false,
            certificate: None,
        };
        self.enrollments.insert(id.clone(), enrollment);
        Ok(id)
    }

    pub fn complete_course_module(
        &mut self,
        enrollment_id: &str,
        module_name: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let now = self.now();
        let (student_id, cert_id) = {
            let enroll = self
                .enrollments
                .get_mut(enrollment_id)
                .ok_or("Enrollment not found")?;
            let course = self
                .courses
                .get(&enroll.course_id)
                .ok_or("Course not found")?;
            if !course.modules.contains(&module_name.to_string()) {
                return Err("Module not part of this course".into());
            }
            enroll
                .module_statuses
                .entry(module_name.to_string())
                .and_modify(|status| *status = CourseModuleStatus::Completed);
            enroll.last_updated = now;

            let all_completed = course.modules.iter().all(|module| {
                matches!(
                    enroll.module_statuses.get(module),
                    Some(CourseModuleStatus::Completed)
                )
            });
            if all_completed && !enroll.completed {
                enroll.completed = true;
                let cert_id = format!("cert_{}", Uuid::new_v4());
                enroll.certificate = Some(cert_id.clone());
                let student_id = enroll.student_id.clone();
                (student_id, Some(cert_id))
            } else {
                (String::new(), None)
            }
        };

        // Award points for course completion if needed
        if let Some(cert_id) = cert_id {
            self.award_points_for_course_completion(&student_id, cert_id)?;
        }

        Ok(())
    }

    pub fn get_enrollment(&self, enrollment_id: &str) -> Option<&CourseEnrollment> {
        self.enrollments.get(enrollment_id)
    }

    pub fn get_course(&self, course_id: &str) -> Option<&BlockchainCourse> {
        self.courses.get(course_id)
    }

    fn award_points_for_course_completion(
        &mut self,
        student_id: &str,
        certificate_id: String,
    ) -> Result<f64, Box<dyn std::error::Error>> {
        self.award_reward_points(
            student_id,
            self.config.reward_points_per_course,
            format!("Course completion {}", certificate_id),
        )
    }
}
