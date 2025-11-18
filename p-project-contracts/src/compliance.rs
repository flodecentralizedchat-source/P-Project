use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComplianceError {
    Paused,
    SenderBlocked,
    RecipientBlocked,
    TokenNotAllowed,
    KycRequired,
    KycMissingSender,
    KycMissingRecipient,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceManager {
    paused: bool,
    kyc_required: bool,
    enforce_token_allowlist: bool,
    approved_kyc: HashSet<String>,
    blocklist: HashSet<String>,
    token_allowlist: HashSet<String>,
    selective_kyc: HashSet<String>,
}

impl Default for ComplianceManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ComplianceManager {
    pub fn new() -> Self {
        Self {
            paused: false,
            kyc_required: false,
            enforce_token_allowlist: false,
            approved_kyc: HashSet::new(),
            blocklist: HashSet::new(),
            token_allowlist: HashSet::new(),
            selective_kyc: HashSet::new(),
        }
    }

    // Global toggles
    pub fn set_paused(&mut self, paused: bool) {
        self.paused = paused;
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    pub fn set_kyc_required(&mut self, required: bool) {
        self.kyc_required = required;
    }

    pub fn is_kyc_required(&self) -> bool {
        self.kyc_required
    }

    pub fn set_enforce_token_allowlist(&mut self, enforce: bool) {
        self.enforce_token_allowlist = enforce;
    }

    pub fn is_enforcing_token_allowlist(&self) -> bool {
        self.enforce_token_allowlist
    }

    // KYC management
    pub fn approve_kyc(&mut self, account: &str) {
        self.approved_kyc.insert(account.to_string());
    }

    pub fn revoke_kyc(&mut self, account: &str) {
        self.approved_kyc.remove(account);
    }

    pub fn is_kyc_approved(&self, account: &str) -> bool {
        self.approved_kyc.contains(account)
    }

    pub fn require_selective_kyc(&mut self, account: &str) {
        self.selective_kyc.insert(account.to_string());
    }

    pub fn clear_selective_kyc(&mut self, account: &str) {
        self.selective_kyc.remove(account);
    }

    pub fn is_selective_kyc(&self, account: &str) -> bool {
        self.selective_kyc.contains(account)
    }

    // Blocklist management
    pub fn block_address(&mut self, account: &str) {
        self.blocklist.insert(account.to_string());
    }

    pub fn unblock_address(&mut self, account: &str) {
        self.blocklist.remove(account);
    }

    pub fn is_blocked(&self, account: &str) -> bool {
        self.blocklist.contains(account)
    }

    // Token allowlist
    pub fn allow_token(&mut self, token: &str) {
        self.token_allowlist.insert(token.to_string());
    }

    pub fn disallow_token(&mut self, token: &str) {
        self.token_allowlist.remove(token);
    }

    pub fn is_token_allowed(&self, token: &str) -> bool {
        // If enforcement is off, default to allow
        if !self.enforce_token_allowlist {
            return true;
        }
        self.token_allowlist.contains(token)
    }

    // Core enforcement hook
    pub fn check_transfer(
        &self,
        sender: &str,
        recipient: &str,
        token: &str,
    ) -> Result<(), ComplianceError> {
        if self.paused {
            return Err(ComplianceError::Paused);
        }
        if self.blocklist.contains(sender) {
            return Err(ComplianceError::SenderBlocked);
        }
        if self.blocklist.contains(recipient) {
            return Err(ComplianceError::RecipientBlocked);
        }
        let selective_trigger =
            self.selective_kyc.contains(sender) || self.selective_kyc.contains(recipient);
        if self.kyc_required || selective_trigger {
            if !self.approved_kyc.contains(sender) && !self.approved_kyc.contains(recipient) {
                // Give a precise error when neither is approved
                return Err(ComplianceError::KycRequired);
            }
            if !self.approved_kyc.contains(sender) {
                return Err(ComplianceError::KycMissingSender);
            }
            if !self.approved_kyc.contains(recipient) {
                return Err(ComplianceError::KycMissingRecipient);
            }
        }
        if !self.is_token_allowed(token) {
            return Err(ComplianceError::TokenNotAllowed);
        }
        Ok(())
    }
}
