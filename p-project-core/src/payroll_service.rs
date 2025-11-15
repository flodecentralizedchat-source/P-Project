//! Payroll & Salaries service module for P-Project
//!
//! Features implemented:
//! - Crypto payroll for peace organizations
//! - School payments for tuition/books
//! - Stipends for volunteers

use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OrganizationType {
    PeaceOrganization,
    School,
    VolunteerOrganization,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Organization {
    pub id: String,
    pub name: String,
    pub org_type: OrganizationType,
    pub wallet_address: String,
    pub is_verified: bool,
    pub created_at: NaiveDateTime,
    pub verified_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone)]
pub struct PayrollServiceConfig {
    pub currency: String,       // e.g., "P"
    pub fee_percentage: f64,    // platform fee percentage for disbursements
    pub max_single_payout: f64, // max allowed per payout
}

impl Default for PayrollServiceConfig {
    fn default() -> Self {
        Self {
            currency: "P".to_string(),
            fee_percentage: 0.0,
            max_single_payout: 250_000.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Frequency {
    OneTime,
    Weekly,
    Monthly,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ReceiptKind {
    Payroll,
    Stipend,
    TuitionPayment,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PaymentStatus {
    Pending,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentReceipt {
    pub id: String,
    pub organization_id: String,
    pub person_id: String, // employee_id, volunteer_id, or student_id
    pub wallet_address: String,
    pub gross_amount: f64,
    pub fee_amount: f64,
    pub net_amount: f64,
    pub currency: String,
    pub kind: ReceiptKind,
    pub status: PaymentStatus,
    pub timestamp: NaiveDateTime,
    pub tx_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Employee {
    pub id: String,
    pub organization_id: String,
    pub name: String,
    pub wallet_address: String,
    pub salary_amount: f64,
    pub currency: String,
    pub active: bool,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Volunteer {
    pub id: String,
    pub organization_id: String,
    pub name: String,
    pub wallet_address: String,
    pub stipend_amount: f64,
    pub currency: String,
    pub frequency: Frequency,
    pub active: bool,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Student {
    pub id: String,
    pub school_id: String,
    pub name: String,
    pub wallet_address: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum InvoiceItemType {
    Tuition,
    Books,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceItem {
    pub item_type: InvoiceItemType,
    pub description: String,
    pub amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum InvoiceStatus {
    Pending,
    Paid,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invoice {
    pub id: String,
    pub school_id: String,
    pub student_id: String,
    pub items: Vec<InvoiceItem>,
    pub total_amount: f64,
    pub currency: String,
    pub status: InvoiceStatus,
    pub created_at: NaiveDateTime,
    pub paid_at: Option<NaiveDateTime>,
    pub payment_receipt_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaySchedule {
    pub id: String,
    pub organization_id: String,
    pub frequency: Frequency,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PayRunStatus {
    Created,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayRun {
    pub id: String,
    pub organization_id: String,
    pub run_date: NaiveDateTime,
    pub status: PayRunStatus,
    pub receipts: Vec<PaymentReceipt>,
}

pub struct PayrollService {
    pub config: PayrollServiceConfig,
    // storage
    pub organizations: HashMap<String, Organization>,
    pub employees: HashMap<String, Employee>,
    pub volunteers: HashMap<String, Volunteer>,
    pub students: HashMap<String, Student>,
    pub schedules: HashMap<String, PaySchedule>,
    pub invoices: HashMap<String, Invoice>,
    pub receipts: HashMap<String, PaymentReceipt>,
    pub pay_runs: HashMap<String, PayRun>,
}

impl PayrollService {
    pub fn new(config: PayrollServiceConfig) -> Self {
        Self {
            config,
            organizations: HashMap::new(),
            employees: HashMap::new(),
            volunteers: HashMap::new(),
            students: HashMap::new(),
            schedules: HashMap::new(),
            invoices: HashMap::new(),
            receipts: HashMap::new(),
            pay_runs: HashMap::new(),
        }
    }

    fn now(&self) -> NaiveDateTime {
        Utc::now().naive_utc()
    }

    fn ensure_amount(&self, amount: f64) -> Result<(), Box<dyn std::error::Error>> {
        if amount <= 0.0 {
            return Err("Amount must be positive".into());
        }
        if amount > self.config.max_single_payout {
            return Err("Amount exceeds maximum allowed payout".into());
        }
        Ok(())
    }

    fn ensure_currency(&self, currency: &str) -> Result<(), Box<dyn std::error::Error>> {
        if currency != self.config.currency {
            Err(format!("Only {} currency is supported", self.config.currency).into())
        } else {
            Ok(())
        }
    }

    // Organizations
    pub fn register_organization(
        &mut self,
        name: String,
        org_type: OrganizationType,
        wallet_address: String,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let id = format!("org_{}", Uuid::new_v4());
        let org = Organization {
            id: id.clone(),
            name,
            org_type,
            wallet_address,
            is_verified: false,
            created_at: self.now(),
            verified_at: None,
        };
        self.organizations.insert(id.clone(), org);
        Ok(id)
    }

    pub fn verify_organization(&mut self, org_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let now = self.now();
        let org = self
            .organizations
            .get_mut(org_id)
            .ok_or("Organization not found")?;
        org.is_verified = true;
        org.verified_at = Some(now);
        Ok(())
    }

    pub fn get_organization(&self, org_id: &str) -> Option<&Organization> {
        self.organizations.get(org_id)
    }

    // Payroll employees
    pub fn add_employee(
        &mut self,
        organization_id: String,
        name: String,
        wallet_address: String,
        salary_amount: f64,
        currency: String,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let org = self
            .organizations
            .get(&organization_id)
            .ok_or("Organization not found")?;
        if org.org_type != OrganizationType::PeaceOrganization {
            return Err("Employees can only be added to peace organizations".into());
        }
        self.ensure_amount(salary_amount)?;
        self.ensure_currency(&currency)?;
        if !org.is_verified {
            return Err("Organization is not verified".into());
        }

        let id = format!("emp_{}", Uuid::new_v4());
        let emp = Employee {
            id: id.clone(),
            organization_id,
            name,
            wallet_address,
            salary_amount,
            currency,
            active: true,
            created_at: self.now(),
        };
        self.employees.insert(id.clone(), emp);
        Ok(id)
    }

    pub fn create_pay_schedule(
        &mut self,
        organization_id: String,
        frequency: Frequency,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let org = self
            .organizations
            .get(&organization_id)
            .ok_or("Organization not found")?;
        if org.org_type != OrganizationType::PeaceOrganization {
            return Err("Pay schedules are for peace organizations".into());
        }
        let id = format!("sched_{}", Uuid::new_v4());
        let sched = PaySchedule {
            id: id.clone(),
            organization_id,
            frequency,
            created_at: self.now(),
        };
        self.schedules.insert(id.clone(), sched);
        Ok(id)
    }

    pub fn run_payroll(
        &mut self,
        organization_id: String,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let org = self
            .organizations
            .get(&organization_id)
            .ok_or("Organization not found")?;
        if org.org_type != OrganizationType::PeaceOrganization {
            return Err("Payroll can only be run for peace organizations".into());
        }
        if !org.is_verified {
            return Err("Organization is not verified".into());
        }

        let mut receipts = Vec::new();
        for emp in self
            .employees
            .values()
            .filter(|e| e.organization_id == organization_id && e.active)
        {
            let fee = (emp.salary_amount * self.config.fee_percentage / 100.0).max(0.0);
            let net = (emp.salary_amount - fee).max(0.0);
            let mut receipt = PaymentReceipt {
                id: format!("rcpt_{}", Uuid::new_v4()),
                organization_id: organization_id.clone(),
                person_id: emp.id.clone(),
                wallet_address: emp.wallet_address.clone(),
                gross_amount: emp.salary_amount,
                fee_amount: fee,
                net_amount: net,
                currency: emp.currency.clone(),
                kind: ReceiptKind::Payroll,
                status: PaymentStatus::Pending,
                timestamp: self.now(),
                tx_hash: None,
            };
            // simulate chain tx
            receipt.tx_hash = Some(format!("0x{}", Uuid::new_v4().simple()));
            receipt.status = PaymentStatus::Completed;
            self.receipts.insert(receipt.id.clone(), receipt.clone());
            receipts.push(receipt);
        }

        let run_id = format!("payrun_{}", Uuid::new_v4());
        let run = PayRun {
            id: run_id.clone(),
            organization_id,
            run_date: self.now(),
            status: PayRunStatus::Completed,
            receipts: receipts.clone(),
        };
        self.pay_runs.insert(run_id.clone(), run);
        Ok(run_id)
    }

    pub fn get_pay_run(&self, run_id: &str) -> Option<&PayRun> {
        self.pay_runs.get(run_id)
    }

    // School payments (invoices)
    pub fn add_student(
        &mut self,
        school_id: String,
        name: String,
        wallet_address: String,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let org = self
            .organizations
            .get(&school_id)
            .ok_or("School not found")?;
        if org.org_type != OrganizationType::School {
            return Err("Students can only be added to schools".into());
        }
        if !org.is_verified {
            return Err("School is not verified".into());
        }
        let id = format!("student_{}", Uuid::new_v4());
        let s = Student {
            id: id.clone(),
            school_id,
            name,
            wallet_address,
            created_at: self.now(),
        };
        self.students.insert(id.clone(), s);
        Ok(id)
    }

    pub fn create_invoice(
        &mut self,
        school_id: String,
        student_id: String,
        items: Vec<InvoiceItem>,
        currency: String,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let school = self
            .organizations
            .get(&school_id)
            .ok_or("School not found")?;
        if school.org_type != OrganizationType::School {
            return Err("Invoices can only be created by schools".into());
        }
        if !school.is_verified {
            return Err("School is not verified".into());
        }
        let student = self.students.get(&student_id).ok_or("Student not found")?;
        if student.school_id != school_id {
            return Err("Student not associated with this school".into());
        }
        self.ensure_currency(&currency)?;
        if items.is_empty() {
            return Err("Invoice must have at least one item".into());
        }
        let mut total = 0.0;
        for it in &items {
            if it.amount <= 0.0 {
                return Err("Invoice item amount must be positive".into());
            }
            total += it.amount;
        }
        self.ensure_amount(total)?;
        let id = format!("inv_{}", Uuid::new_v4());
        let inv = Invoice {
            id: id.clone(),
            school_id,
            student_id,
            items,
            total_amount: total,
            currency,
            status: InvoiceStatus::Pending,
            created_at: self.now(),
            paid_at: None,
            payment_receipt_id: None,
        };
        self.invoices.insert(id.clone(), inv);
        Ok(id)
    }

    pub fn pay_invoice(
        &mut self,
        invoice_id: String,
        payer_wallet: String,
    ) -> Result<PaymentReceipt, Box<dyn std::error::Error>> {
        // First, read immutable invoice data needed to compute the receipt
        let (school_id, student_id, total_amount, currency, status) = {
            let inv = self.invoices.get(&invoice_id).ok_or("Invoice not found")?;
            (
                inv.school_id.clone(),
                inv.student_id.clone(),
                inv.total_amount,
                inv.currency.clone(),
                inv.status.clone(),
            )
        };
        if status != InvoiceStatus::Pending {
            return Err("Invoice not payable".into());
        }

        let student = self.students.get(&student_id).ok_or("Student not found")?;
        let school = self
            .organizations
            .get(&school_id)
            .ok_or("School not found")?;

        let fee = (total_amount * self.config.fee_percentage / 100.0).max(0.0);
        let net = (total_amount - fee).max(0.0);
        let now = self.now();
        let mut receipt = PaymentReceipt {
            id: format!("rcpt_{}", Uuid::new_v4()),
            organization_id: school.id.clone(),
            person_id: student.id.clone(),
            wallet_address: payer_wallet,
            gross_amount: total_amount,
            fee_amount: fee,
            net_amount: net,
            currency,
            kind: ReceiptKind::TuitionPayment,
            status: PaymentStatus::Pending,
            timestamp: now,
            tx_hash: None,
        };
        receipt.tx_hash = Some(format!("0x{}", Uuid::new_v4().simple()));
        receipt.status = PaymentStatus::Completed;

        // Now, update the invoice with a mutable borrow
        if let Some(inv) = self.invoices.get_mut(&invoice_id) {
            inv.status = InvoiceStatus::Paid;
            inv.paid_at = Some(now);
            inv.payment_receipt_id = Some(receipt.id.clone());
        }

        self.receipts.insert(receipt.id.clone(), receipt.clone());
        Ok(receipt)
    }

    pub fn get_invoice(&self, invoice_id: &str) -> Option<&Invoice> {
        self.invoices.get(invoice_id)
    }

    // Volunteer stipends
    pub fn add_volunteer(
        &mut self,
        organization_id: String,
        name: String,
        wallet_address: String,
        stipend_amount: f64,
        currency: String,
        frequency: Frequency,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let org = self
            .organizations
            .get(&organization_id)
            .ok_or("Organization not found")?;
        if org.org_type != OrganizationType::VolunteerOrganization
            && org.org_type != OrganizationType::PeaceOrganization
        {
            return Err("Volunteers can only be added to volunteer or peace organizations".into());
        }
        if !org.is_verified {
            return Err("Organization is not verified".into());
        }
        self.ensure_amount(stipend_amount)?;
        self.ensure_currency(&currency)?;
        let id = format!("vol_{}", Uuid::new_v4());
        let v = Volunteer {
            id: id.clone(),
            organization_id,
            name,
            wallet_address,
            stipend_amount,
            currency,
            frequency,
            active: true,
            created_at: self.now(),
        };
        self.volunteers.insert(id.clone(), v);
        Ok(id)
    }

    pub fn run_stipend_disbursement(
        &mut self,
        organization_id: String,
    ) -> Result<Vec<PaymentReceipt>, Box<dyn std::error::Error>> {
        let org = self
            .organizations
            .get(&organization_id)
            .ok_or("Organization not found")?;
        if org.org_type != OrganizationType::VolunteerOrganization
            && org.org_type != OrganizationType::PeaceOrganization
        {
            return Err(
                "Stipends can only be disbursed by volunteer or peace organizations".into(),
            );
        }
        if !org.is_verified {
            return Err("Organization is not verified".into());
        }
        let mut receipts = Vec::new();
        for vol in self
            .volunteers
            .values()
            .filter(|v| v.organization_id == organization_id && v.active)
        {
            let fee = (vol.stipend_amount * self.config.fee_percentage / 100.0).max(0.0);
            let net = (vol.stipend_amount - fee).max(0.0);
            let mut receipt = PaymentReceipt {
                id: format!("rcpt_{}", Uuid::new_v4()),
                organization_id: organization_id.clone(),
                person_id: vol.id.clone(),
                wallet_address: vol.wallet_address.clone(),
                gross_amount: vol.stipend_amount,
                fee_amount: fee,
                net_amount: net,
                currency: vol.currency.clone(),
                kind: ReceiptKind::Stipend,
                status: PaymentStatus::Pending,
                timestamp: self.now(),
                tx_hash: None,
            };
            receipt.tx_hash = Some(format!("0x{}", Uuid::new_v4().simple()));
            receipt.status = PaymentStatus::Completed;
            self.receipts.insert(receipt.id.clone(), receipt.clone());
            receipts.push(receipt);
        }
        Ok(receipts)
    }

    pub fn get_receipt(&self, receipt_id: &str) -> Option<&PaymentReceipt> {
        self.receipts.get(receipt_id)
    }
}
