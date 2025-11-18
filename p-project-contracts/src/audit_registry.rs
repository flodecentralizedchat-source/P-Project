use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuditError {
    NotFound,
    AlreadyFinalized,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditInfo {
    pub firm: String,
    pub report_uri: String,
    pub report_hash: String,
    pub timestamp: NaiveDateTime,
    pub notes: Option<String>,
    pub finalized: bool,
}

impl AuditInfo {
    pub fn new<S: Into<String>>(firm: S, report_uri: S, report_hash: S, notes: Option<S>) -> Self {
        Self {
            firm: firm.into(),
            report_uri: report_uri.into(),
            report_hash: report_hash.into(),
            timestamp: Utc::now().naive_utc(),
            notes: notes.map(|n| n.into()),
            finalized: false,
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct AuditRegistry {
    entries: HashMap<String, AuditInfo>,
}

impl AuditRegistry {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn set_audit<S: Into<String>>(
        &mut self,
        contract_id: S,
        info: AuditInfo,
    ) -> Result<(), AuditError> {
        let id = contract_id.into();
        if let Some(existing) = self.entries.get(&id) {
            if existing.finalized {
                return Err(AuditError::AlreadyFinalized);
            }
        }
        self.entries.insert(id, info);
        Ok(())
    }

    pub fn finalize<S: Into<String>>(&mut self, contract_id: S) -> Result<(), AuditError> {
        let id = contract_id.into();
        let entry = self.entries.get_mut(&id).ok_or(AuditError::NotFound)?;
        if entry.finalized {
            return Err(AuditError::AlreadyFinalized);
        }
        entry.finalized = true;
        Ok(())
    }

    pub fn get<S: Into<String>>(&self, contract_id: S) -> Option<&AuditInfo> {
        self.entries.get(&contract_id.into())
    }

    pub fn is_finalized<S: Into<String>>(&self, contract_id: S) -> Result<bool, AuditError> {
        Ok(self
            .entries
            .get(&contract_id.into())
            .ok_or(AuditError::NotFound)?
            .finalized)
    }
}
