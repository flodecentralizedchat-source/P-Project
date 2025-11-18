use chrono::{Duration, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents an audit that must be completed prior to a launch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditRequirement {
    firm: Option<String>,
    report_uri: Option<String>,
    report_hash: Option<String>,
    finalized: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub enum AuditError {
    AlreadyFinalized,
    MissingDetails,
}

impl AuditRequirement {
    pub fn new() -> Self {
        Self {
            firm: None,
            report_uri: None,
            report_hash: None,
            finalized: false,
        }
    }

    pub fn record_audit(&mut self, firm: &str, report_uri: &str, report_hash: &str) {
        self.firm = Some(firm.to_string());
        self.report_uri = Some(report_uri.to_string());
        self.report_hash = Some(report_hash.to_string());
        self.finalized = false;
    }

    pub fn finalize(&mut self) -> Result<(), AuditError> {
        if self.finalized {
            return Err(AuditError::AlreadyFinalized);
        }
        if self.firm.is_none() || self.report_uri.is_none() || self.report_hash.is_none() {
            return Err(AuditError::MissingDetails);
        }
        self.finalized = true;
        Ok(())
    }

    pub fn is_ready(&self) -> bool {
        self.finalized
            && self.firm.is_some()
            && self.report_uri.is_some()
            && self.report_hash.is_some()
    }
}

/// Protects funds from rug pulls by enforcing custodial locks.
#[derive(Debug, Clone)]
pub struct RugPullProtector {
    custodian: String,
    locked_until: Option<NaiveDateTime>,
    smart_lock_engaged: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub enum RugPullError {
    NotCustodian,
    InvalidDuration,
    StillLocked,
}

impl RugPullProtector {
    pub fn new(custodian: &str) -> Self {
        Self {
            custodian: custodian.to_string(),
            locked_until: None,
            smart_lock_engaged: false,
        }
    }

    pub fn set_custodian(&mut self, custodian: &str) {
        self.custodian = custodian.to_string();
    }

    pub fn lock(&mut self, duration_secs: i64) -> Result<(), RugPullError> {
        if duration_secs < 0 {
            return Err(RugPullError::InvalidDuration);
        }
        let until = Utc::now() + Duration::seconds(duration_secs);
        self.locked_until = Some(until);
        self.smart_lock_engaged = true;
        Ok(())
    }

    pub fn unlock(&mut self, caller: &str) -> Result<(), RugPullError> {
        if caller != self.custodian {
            return Err(RugPullError::NotCustodian);
        }
        if let Some(until) = self.locked_until {
            if Utc::now() < until {
                return Err(RugPullError::StillLocked);
            }
        }
        self.smart_lock_engaged = false;
        self.locked_until = None;
        Ok(())
    }

    pub fn is_locked(&self) -> bool {
        self.smart_lock_engaged
    }
}

/// Enforces governance delays using a time lock controller.
#[derive(Debug)]
pub struct TimeLockController {
    delay_seconds: i64,
    scheduled: HashMap<String, NaiveDateTime>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum TimeLockError {
    NotScheduled,
    NotReady,
}

impl TimeLockController {
    pub fn new(delay_seconds: i64) -> Self {
        Self {
            delay_seconds,
            scheduled: HashMap::new(),
        }
    }

    pub fn delay_seconds(&self) -> i64 {
        self.delay_seconds
    }

    pub fn schedule(&mut self, operation: &str) -> NaiveDateTime {
        let when = Utc::now() + Duration::seconds(self.delay_seconds);
        self.scheduled.insert(operation.to_string(), when);
        when
    }

    pub fn schedule_at(&mut self, operation: &str, ready_at: NaiveDateTime) {
        self.scheduled.insert(operation.to_string(), ready_at);
    }

    pub fn can_execute(&self, operation: &str) -> bool {
        self.scheduled
            .get(operation)
            .map(|ready_at| Utc::now() >= *ready_at)
            .unwrap_or(false)
    }

    pub fn execute(&mut self, operation: &str) -> Result<(), TimeLockError> {
        let ready_at = self.scheduled.get(operation).copied();
        match ready_at {
            Some(ready) => {
                if Utc::now() >= ready {
                    self.scheduled.remove(operation);
                    Ok(())
                } else {
                    Err(TimeLockError::NotReady)
                }
            }
            None => Err(TimeLockError::NotScheduled),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audit_requirement_tracks_prelaunch_status() {
        let mut audit = AuditRequirement::new();
        assert!(!audit.is_ready());

        audit.record_audit("AuditCorp", "https://audit", "hash123");
        assert!(!audit.is_ready());

        audit.finalize().unwrap();
        assert!(audit.is_ready());
        assert_eq!(
            audit.finalize().expect_err("already finalized"),
            AuditError::AlreadyFinalized
        );
    }

    #[test]
    fn rug_pull_protector_enforces_custody_lock() {
        let mut protector = RugPullProtector::new("alice");
        protector.lock(0).unwrap();
        assert!(protector.is_locked());

        assert_eq!(
            protector.unlock("mallory").expect_err("wrong custodian"),
            RugPullError::NotCustodian
        );
        protector.unlock("alice").unwrap();
        assert!(!protector.is_locked());
    }

    #[test]
    fn time_lock_controller_delays_and_executes() {
        let mut controller = TimeLockController::new(0);
        let ready_at = controller.schedule("upgrade");
        assert!(controller.can_execute("upgrade"));
        controller.execute("upgrade").unwrap();
        assert!(!controller.can_execute("upgrade"));

        controller.schedule_at("budget", ready_at);
        assert!(controller.can_execute("budget"));
        controller.execute("budget").unwrap();

        let result = controller.execute("nothing");
        assert_eq!(
            result.expect_err("missing schedule"),
            TimeLockError::NotScheduled
        );
    }
}
